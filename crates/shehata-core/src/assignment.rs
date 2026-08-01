//! Repository assignment and local commit identity.
//!
//! The operation is intentionally local-only: it writes a marker under the
//! repository's Git metadata, updates local `user.name` / `user.email` when
//! requested, and records rollback values before either setting changes.

use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use shehata_git::{read_local_config_values, replace_local_config_values, GitRunner};
use shehata_storage::{queries, AccountRecord, Database, RepositoryRecord};

use crate::error::{Result, ShehataError};
use crate::repositories::{repository_summary, RepositorySummary};

#[derive(Debug, Clone, Deserialize)]
pub struct AssignRepositoryRequest {
    pub repository_id: String,
    pub host: String,
    pub login: String,
    pub commit_name: Option<String>,
    pub commit_email: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct AssignmentResult {
    pub repository: RepositorySummary,
    pub marker_path: String,
    pub identity_changed: bool,
}

#[derive(Debug)]
struct AssignmentPlan {
    repository: RepositoryRecord,
    account: AccountRecord,
    commit_name: Option<String>,
    commit_email: Option<String>,
}

pub async fn assign_repository(request: AssignRepositoryRequest) -> Result<AssignmentResult> {
    let path = Database::default_path()?;
    assign_repository_at(&path, request).await
}

pub async fn assign_repository_at(
    db_path: &Path,
    request: AssignRepositoryRequest,
) -> Result<AssignmentResult> {
    let plan = {
        let db = Database::open_at(db_path)?;
        prepare_assignment(&db, request)?
    };

    let repo_path = PathBuf::from(&plan.repository.canonical_path);
    let git = GitRunner::locate()?;
    let discovered = shehata_git::discover_repository(&git, &repo_path).await?;

    let previous_name = read_local_config_values(&git, &repo_path, "user.name").await?;
    let previous_email = read_local_config_values(&git, &repo_path, "user.email").await?;

    {
        let db = Database::open_at(db_path)?;
        if plan.commit_name.is_some() {
            ensure_backup(&db, &plan.repository.id, "user.name", &previous_name)?;
        }
        if plan.commit_email.is_some() {
            ensure_backup(&db, &plan.repository.id, "user.email", &previous_email)?;
        }
    }

    let marker = ensure_repository_marker(&discovered.git_dir, &plan.repository.id)?;
    let identity_result = apply_identity(
        &git,
        &repo_path,
        plan.commit_name.as_deref(),
        plan.commit_email.as_deref(),
    )
    .await;
    if let Err(error) = identity_result {
        rollback_identity(
            &git,
            &repo_path,
            plan.commit_name
                .is_some()
                .then_some(previous_name.as_slice()),
            plan.commit_email
                .is_some()
                .then_some(previous_email.as_slice()),
        )
        .await;
        rollback_marker(&marker);
        return Err(error);
    }

    let final_name = plan
        .commit_name
        .clone()
        .or_else(|| previous_name.first().cloned());
    let final_email = plan
        .commit_email
        .clone()
        .or_else(|| previous_email.first().cloned());

    let stored = {
        let db = Database::open_at(db_path)?;
        if let Err(error) = queries::update_repository_assignment_and_identity(
            &db,
            &plan.repository.id,
            plan.account.id,
            final_name.as_deref(),
            final_email.as_deref(),
        ) {
            drop(db);
            rollback_identity(
                &git,
                &repo_path,
                plan.commit_name
                    .is_some()
                    .then_some(previous_name.as_slice()),
                plan.commit_email
                    .is_some()
                    .then_some(previous_email.as_slice()),
            )
            .await;
            rollback_marker(&marker);
            return Err(error.into());
        }
        queries::find_repository_by_id(&db, &plan.repository.id)?
            .ok_or_else(|| ShehataError::RepositoryNotFound(plan.repository.id.clone()))?
    };

    let summary = {
        let db = Database::open_at(db_path)?;
        repository_summary(&db, stored)?
    };

    Ok(AssignmentResult {
        repository: summary,
        marker_path: marker.path.to_string_lossy().into_owned(),
        identity_changed: plan.commit_name.is_some() || plan.commit_email.is_some(),
    })
}

fn prepare_assignment(db: &Database, request: AssignRepositoryRequest) -> Result<AssignmentPlan> {
    let repository = queries::find_repository_by_id(db, request.repository_id.trim())?
        .ok_or_else(|| ShehataError::RepositoryNotFound(request.repository_id.clone()))?;
    let host = clean_required("account host", &request.host, 255)?;
    let login = clean_required("account login", &request.login, 255)?;
    let account = queries::find_account(db, &host, &login)?.ok_or_else(|| {
        ShehataError::AccountNotAvailable {
            host: host.clone(),
            login: login.clone(),
        }
    })?;

    if account.status != "valid" {
        return Err(ShehataError::AccountNotAvailable { host, login });
    }
    match repository.host.as_deref() {
        Some(repo_host) if repo_host.eq_ignore_ascii_case(&account.host) => {}
        Some(repo_host) => {
            return Err(ShehataError::InvalidInput(format!(
                "repository host {repo_host} does not match account host {}",
                account.host
            )))
        }
        None => {
            return Err(ShehataError::InvalidInput(
                "repository has no supported GitHub remote".to_string(),
            ))
        }
    }

    let commit_name = clean_optional("commit name", request.commit_name, 128)?;
    let commit_email = clean_optional("commit email", request.commit_email, 254)?;
    if let Some(email) = &commit_email {
        validate_email(email)?;
    }

    Ok(AssignmentPlan {
        repository,
        account,
        commit_name,
        commit_email,
    })
}

fn clean_required(label: &str, value: &str, max: usize) -> Result<String> {
    clean_optional(label, Some(value.to_string()), max)?
        .ok_or_else(|| ShehataError::InvalidInput(format!("{label} is required")))
}

fn clean_optional(label: &str, value: Option<String>, max: usize) -> Result<Option<String>> {
    let Some(value) = value else {
        return Ok(None);
    };
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Ok(None);
    }
    if trimmed.len() > max || trimmed.chars().any(|c| c == '\0' || c == '\r' || c == '\n') {
        return Err(ShehataError::InvalidInput(format!("invalid {label}")));
    }
    Ok(Some(trimmed.to_string()))
}

fn validate_email(email: &str) -> Result<()> {
    let valid = !email.contains(char::is_whitespace)
        && email.split_once('@').is_some_and(|(local, domain)| {
            !local.is_empty() && domain.contains('.') && !domain.ends_with('.')
        });
    if !valid {
        return Err(ShehataError::InvalidInput(
            "commit email is not valid".to_string(),
        ));
    }
    Ok(())
}

fn ensure_backup(db: &Database, repo_id: &str, key: &str, values: &[String]) -> Result<()> {
    let exists = queries::pending_backups(db, repo_id)?
        .iter()
        .any(|backup| backup.config_key == key);
    if !exists {
        let json = serde_json::to_string(values)
            .map_err(|error| ShehataError::Internal(error.to_string()))?;
        queries::insert_config_backup(db, repo_id, key, &json)?;
    }
    Ok(())
}

async fn apply_identity(
    git: &GitRunner,
    repo_path: &Path,
    name: Option<&str>,
    email: Option<&str>,
) -> Result<()> {
    if let Some(name) = name {
        replace_local_config_values(git, repo_path, "user.name", &[name.to_string()]).await?;
    }
    if let Some(email) = email {
        replace_local_config_values(git, repo_path, "user.email", &[email.to_string()]).await?;
    }
    Ok(())
}

async fn rollback_identity(
    git: &GitRunner,
    repo_path: &Path,
    name: Option<&[String]>,
    email: Option<&[String]>,
) {
    if let Some(values) = name {
        let _ = replace_local_config_values(git, repo_path, "user.name", values).await;
    }
    if let Some(values) = email {
        let _ = replace_local_config_values(git, repo_path, "user.email", values).await;
    }
}

#[derive(Debug)]
struct MarkerWrite {
    path: PathBuf,
    created: bool,
}

fn ensure_repository_marker(git_dir: &Path, repository_id: &str) -> Result<MarkerWrite> {
    let marker_dir = git_dir.join("shehata-git");
    let marker_path = marker_dir.join("repository-id");
    fs::create_dir_all(&marker_dir)
        .map_err(|error| ShehataError::RepositoryMarker(error.to_string()))?;

    if marker_path.exists() {
        let existing = fs::read_to_string(&marker_path)
            .map_err(|error| ShehataError::RepositoryMarker(error.to_string()))?;
        if existing.trim() != repository_id {
            return Err(ShehataError::RepositoryMarker(
                "marker belongs to a different repository record".to_string(),
            ));
        }
        return Ok(MarkerWrite {
            path: marker_path,
            created: false,
        });
    }

    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&marker_path)
        .map_err(|error| ShehataError::RepositoryMarker(error.to_string()))?;
    writeln!(file, "{repository_id}")
        .and_then(|_| file.sync_all())
        .map_err(|error| ShehataError::RepositoryMarker(error.to_string()))?;
    Ok(MarkerWrite {
        path: marker_path,
        created: true,
    })
}

fn rollback_marker(marker: &MarkerWrite) {
    if marker.created {
        let _ = fs::remove_file(&marker.path);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repositories::save_discovered_repository;

    #[test]
    fn validates_identity_inputs() {
        assert!(clean_optional("name", Some("Jane Doe".into()), 128).is_ok());
        assert!(clean_optional("name", Some("bad\nname".into()), 128).is_err());
        assert!(validate_email("jane@example.com").is_ok());
        assert!(validate_email("not-an-email").is_err());
    }

    #[tokio::test]
    async fn assigns_account_writes_marker_and_backs_up_identity() {
        let dir = tempfile::tempdir().unwrap();
        let repo_path = dir.path().join("repo");
        fs::create_dir(&repo_path).unwrap();
        let db_path = dir.path().join("data").join("shehata.db");
        let git = GitRunner::locate().unwrap();
        git.run_checked(Some(&repo_path), &["init", "-b", "main"])
            .await
            .unwrap();
        git.run_checked(
            Some(&repo_path),
            &[
                "remote",
                "add",
                "origin",
                "https://github.com/acme/example.git",
            ],
        )
        .await
        .unwrap();
        git.run_checked(
            Some(&repo_path),
            &["config", "--local", "user.name", "Original Name"],
        )
        .await
        .unwrap();
        git.run_checked(
            Some(&repo_path),
            &["config", "--local", "user.email", "original@example.com"],
        )
        .await
        .unwrap();

        let discovered = shehata_git::discover_repository(&git, &repo_path)
            .await
            .unwrap();
        let (repository_id, account_id) = {
            let db = Database::open_at(&db_path).unwrap();
            let repository = save_discovered_repository(&db, &discovered).unwrap();
            let account_id =
                queries::upsert_account(&db, "github.com", "janedoe", "valid").unwrap();
            (repository.id, account_id)
        };

        let result = assign_repository_at(
            &db_path,
            AssignRepositoryRequest {
                repository_id: repository_id.clone(),
                host: "github.com".into(),
                login: "janedoe".into(),
                commit_name: Some("Jane Doe".into()),
                commit_email: Some("jane@example.com".into()),
            },
        )
        .await
        .unwrap();

        assert_eq!(result.repository.assigned_login.as_deref(), Some("janedoe"));
        assert!(Path::new(&result.marker_path).exists());
        assert_eq!(
            fs::read_to_string(&result.marker_path).unwrap().trim(),
            repository_id
        );
        assert_eq!(
            read_local_config_values(&git, &repo_path, "user.name")
                .await
                .unwrap(),
            vec!["Jane Doe"]
        );

        let db = Database::open_at(&db_path).unwrap();
        let stored = queries::find_repository_by_id(&db, &repository_id)
            .unwrap()
            .unwrap();
        assert_eq!(stored.assigned_account_id, Some(account_id));
        let backups = queries::pending_backups(&db, &repository_id).unwrap();
        assert_eq!(backups.len(), 2);
        assert!(backups.iter().any(|backup| {
            backup.config_key == "user.name" && backup.previous_values_json == "[\"Original Name\"]"
        }));
    }
}
