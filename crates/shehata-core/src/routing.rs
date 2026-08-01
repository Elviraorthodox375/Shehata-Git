//! Repository-scoped HTTPS credential routing.
//!
//! Tokens never enter this module. Git invokes `git-credential-shehata`, and
//! that helper obtains the assigned account's token just in time from `gh`.

use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;

use serde::{Deserialize, Serialize};
use shehata_git::{
    parse_remote_url, read_local_config_values, replace_local_config_values, GitRunner,
    RemoteProtocol,
};
use shehata_storage::{queries, AccountRecord, Database, NewAuditEvent, RepositoryRecord};

use crate::error::{Result, ShehataError};

const HELPER_KEY: &str = "credential.helper";
const HTTP_PATH_KEY: &str = "credential.useHttpPath";

#[derive(Debug, Clone, Deserialize)]
pub struct LinkRepositoryRequest {
    pub repository_id: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct RoutingResult {
    pub repository_id: String,
    pub helper_path: String,
    pub configured: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct ConnectionTestResult {
    pub repository_id: String,
    pub remote_name: String,
    pub account_login: String,
    pub success: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UnlinkRepositoryRequest {
    pub repository_id: String,
    #[serde(default)]
    pub restore_identity: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct UnlinkResult {
    pub repository_id: String,
    pub restored_keys: Vec<String>,
    pub identity_preserved: bool,
}

#[derive(Debug)]
struct RoutingPlan {
    repository: RepositoryRecord,
    account: AccountRecord,
    remote_name: String,
}

pub async fn link_repository(request: LinkRepositoryRequest) -> Result<RoutingResult> {
    let db_path = Database::default_path()?;
    let helper_path = locate_helper()?;
    link_repository_at(&db_path, &helper_path, request).await
}

pub async fn link_repository_at(
    db_path: &Path,
    helper_path: &Path,
    request: LinkRepositoryRequest,
) -> Result<RoutingResult> {
    let plan = load_plan(db_path, &request.repository_id)?;
    let repo_path = PathBuf::from(&plan.repository.canonical_path);
    let git = GitRunner::locate()?;
    let helper = helper_config_value(helper_path, &plan.repository.id)?;

    let previous_helpers = read_local_config_values(&git, &repo_path, HELPER_KEY).await?;
    let previous_http_path = read_local_config_values(&git, &repo_path, HTTP_PATH_KEY).await?;
    {
        let db = Database::open_at(db_path)?;
        ensure_backup(&db, &plan.repository.id, HELPER_KEY, &previous_helpers)?;
        ensure_backup(&db, &plan.repository.id, HTTP_PATH_KEY, &previous_http_path)?;
    }

    let expected_helpers = vec![String::new(), helper.clone()];
    if let Err(error) =
        replace_local_config_values(&git, &repo_path, HELPER_KEY, &expected_helpers).await
    {
        return Err(error.into());
    }
    if let Err(error) =
        replace_local_config_values(&git, &repo_path, HTTP_PATH_KEY, &["true".to_string()]).await
    {
        restore_routing_config(&git, &repo_path, &previous_helpers, &previous_http_path).await;
        return Err(error.into());
    }

    let actual_helpers = read_local_config_values(&git, &repo_path, HELPER_KEY).await?;
    let actual_http_path = read_local_config_values(&git, &repo_path, HTTP_PATH_KEY).await?;
    if actual_helpers != expected_helpers || actual_http_path != ["true"] {
        restore_routing_config(&git, &repo_path, &previous_helpers, &previous_http_path).await;
        return Err(ShehataError::Internal(
            "Git credential routing verification failed".to_string(),
        ));
    }

    audit(
        db_path,
        &plan.repository.id,
        Some(&plan.account.login),
        "credential_routing_enabled",
        "Configured repository-scoped credential routing",
        "success",
        Some(0),
        None,
    )?;

    Ok(RoutingResult {
        repository_id: plan.repository.id,
        helper_path: helper_path.to_string_lossy().into_owned(),
        configured: true,
    })
}

pub async fn test_connection(repository_id: &str) -> Result<ConnectionTestResult> {
    let db_path = Database::default_path()?;
    test_connection_at(&db_path, repository_id).await
}

pub async fn test_connection_at(
    db_path: &Path,
    repository_id: &str,
) -> Result<ConnectionTestResult> {
    let plan = load_plan(db_path, repository_id)?;
    let repo_path = PathBuf::from(&plan.repository.canonical_path);
    let git = GitRunner::locate()?;
    let started = Instant::now();
    let output = git
        .run_in(Some(&repo_path), &["ls-remote", &plan.remote_name, "HEAD"])
        .await?;
    let duration = started.elapsed().as_millis().min(i64::MAX as u128) as i64;

    if !output.success() {
        audit(
            db_path,
            &plan.repository.id,
            Some(&plan.account.login),
            "credential_connection_test",
            "Remote authentication test failed",
            "failure",
            Some(output.code.into()),
            Some(duration),
        )?;
        return Err(ShehataError::AuthenticationFailed);
    }

    audit(
        db_path,
        &plan.repository.id,
        Some(&plan.account.login),
        "credential_connection_test",
        "Remote authentication test succeeded",
        "success",
        Some(0),
        Some(duration),
    )?;
    Ok(ConnectionTestResult {
        repository_id: plan.repository.id,
        remote_name: plan.remote_name,
        account_login: plan.account.login,
        success: true,
    })
}

pub async fn unlink_repository(request: UnlinkRepositoryRequest) -> Result<UnlinkResult> {
    let db_path = Database::default_path()?;
    unlink_repository_at(&db_path, request).await
}

pub async fn unlink_repository_at(
    db_path: &Path,
    request: UnlinkRepositoryRequest,
) -> Result<UnlinkResult> {
    let repository = {
        let db = Database::open_at(db_path)?;
        queries::find_repository_by_id(&db, request.repository_id.trim())?
            .ok_or_else(|| ShehataError::RepositoryNotFound(request.repository_id.clone()))?
    };
    let repo_path = PathBuf::from(&repository.canonical_path);
    let backups = {
        let db = Database::open_at(db_path)?;
        queries::pending_backups(&db, &repository.id)?
    };
    if !backups.iter().any(|backup| backup.config_key == HELPER_KEY) {
        return Err(ShehataError::RepositoryNotLinked(repository.id));
    }

    let git = GitRunner::locate()?;
    let mut restored_keys = Vec::new();
    for backup in &backups {
        let is_identity = matches!(backup.config_key.as_str(), "user.name" | "user.email");
        if is_identity && !request.restore_identity {
            continue;
        }
        if !matches!(
            backup.config_key.as_str(),
            HELPER_KEY | HTTP_PATH_KEY | "user.name" | "user.email"
        ) {
            continue;
        }
        let values: Vec<String> = serde_json::from_str(&backup.previous_values_json)
            .map_err(|error| ShehataError::Internal(error.to_string()))?;
        replace_local_config_values(&git, &repo_path, &backup.config_key, &values).await?;
        restored_keys.push(backup.config_key.clone());
    }

    remove_marker(&repository)?;
    {
        let db = Database::open_at(db_path)?;
        for backup in &backups {
            queries::mark_backup_restored(&db, backup.id)?;
        }
        queries::clear_repository_assignment(&db, &repository.id)?;
        queries::insert_audit_event(
            &db,
            &NewAuditEvent {
                event_type: "repository_unlinked",
                repository_id: Some(&repository.id),
                account_login: None,
                summary: "Restored repository Git configuration and removed routing",
                result: "success",
                exit_code: Some(0),
                duration_ms: None,
            },
        )?;
    }

    Ok(UnlinkResult {
        repository_id: repository.id,
        restored_keys,
        identity_preserved: !request.restore_identity,
    })
}

fn load_plan(db_path: &Path, repository_id: &str) -> Result<RoutingPlan> {
    let db = Database::open_at(db_path)?;
    let repository = queries::find_repository_by_id(&db, repository_id.trim())?
        .ok_or_else(|| ShehataError::RepositoryNotFound(repository_id.to_string()))?;
    let account_id = repository
        .assigned_account_id
        .ok_or_else(|| ShehataError::RepositoryNotLinked(repository.id.clone()))?;
    let account = queries::find_account_by_id(&db, account_id)?
        .ok_or_else(|| ShehataError::RepositoryNotLinked(repository.id.clone()))?;
    if account.status != "valid" {
        return Err(ShehataError::AccountNotAvailable {
            host: account.host,
            login: account.login,
        });
    }
    let remote_name = repository.remote_name.clone().ok_or_else(|| {
        ShehataError::InvalidInput("repository has no primary remote".to_string())
    })?;
    let remote_url = repository.remote_url.as_deref().ok_or_else(|| {
        ShehataError::InvalidInput("repository has no primary remote URL".to_string())
    })?;
    let parsed = parse_remote_url(remote_url)
        .map_err(|_| ShehataError::InvalidInput("repository remote URL is invalid".to_string()))?;
    if parsed.protocol != RemoteProtocol::Https {
        return Err(ShehataError::InvalidInput(
            "credential routing requires an HTTPS remote".to_string(),
        ));
    }
    if !parsed.host.eq_ignore_ascii_case(&account.host) {
        return Err(ShehataError::InvalidInput(
            "repository host does not match assigned account".to_string(),
        ));
    }
    Ok(RoutingPlan {
        repository,
        account,
        remote_name,
    })
}

pub(crate) fn locate_helper() -> Result<PathBuf> {
    if let Some(path) = std::env::var_os("SHEHATA_HELPER_PATH") {
        let path = PathBuf::from(path);
        if path.is_file() {
            return Ok(path);
        }
    }
    if let Ok(current) = std::env::current_exe() {
        let sibling = current.with_file_name(if cfg!(windows) {
            "git-credential-shehata.exe"
        } else {
            "git-credential-shehata"
        });
        if sibling.is_file() {
            return Ok(sibling);
        }
    }
    which::which("git-credential-shehata").map_err(|_| ShehataError::CredentialHelperMissing)
}

pub(crate) fn helper_config_value(path: &Path, repository_id: &str) -> Result<String> {
    if !path.is_file() {
        return Err(ShehataError::CredentialHelperMissing);
    }
    let canonical = fs::canonicalize(path).map_err(|_| ShehataError::CredentialHelperMissing)?;
    let raw = canonical.to_string_lossy();
    let path = raw.strip_prefix(r"\\?\").unwrap_or(&raw).replace('\\', "/");
    if path.contains(['\r', '\n']) || uuid::Uuid::parse_str(repository_id).is_err() {
        return Err(ShehataError::InvalidInput(
            "unsafe credential helper path or repository id".to_string(),
        ));
    }
    // `!` tells Git to execute the following fixed command as-is. This is
    // required for absolute Windows paths, which otherwise get rewritten to
    // `git-credential-<path>`. Both path and UUID are validated above.
    Ok(format!("!{} --repo-id {repository_id}", shell_quote(&path)))
}

fn shell_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
}

fn ensure_backup(db: &Database, repository_id: &str, key: &str, values: &[String]) -> Result<()> {
    if !queries::pending_backups(db, repository_id)?
        .iter()
        .any(|backup| backup.config_key == key)
    {
        let json = serde_json::to_string(values)
            .map_err(|error| ShehataError::Internal(error.to_string()))?;
        queries::insert_config_backup(db, repository_id, key, &json)?;
    }
    Ok(())
}

async fn restore_routing_config(
    git: &GitRunner,
    repo_path: &Path,
    helpers: &[String],
    http_path: &[String],
) {
    let _ = replace_local_config_values(git, repo_path, HELPER_KEY, helpers).await;
    let _ = replace_local_config_values(git, repo_path, HTTP_PATH_KEY, http_path).await;
}

fn remove_marker(repository: &RepositoryRecord) -> Result<()> {
    let Some(git_dir) = repository.git_dir.as_deref() else {
        return Ok(());
    };
    let marker = Path::new(git_dir).join("shehata-git").join("repository-id");
    if !marker.exists() {
        return Ok(());
    }
    let existing = fs::read_to_string(&marker)
        .map_err(|error| ShehataError::RepositoryMarker(error.to_string()))?;
    if existing.trim() != repository.id {
        return Err(ShehataError::RepositoryMarker(
            "marker belongs to a different repository record".to_string(),
        ));
    }
    fs::remove_file(marker).map_err(|error| ShehataError::RepositoryMarker(error.to_string()))?;
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn audit(
    db_path: &Path,
    repository_id: &str,
    account_login: Option<&str>,
    event_type: &str,
    summary: &str,
    result: &str,
    exit_code: Option<i64>,
    duration_ms: Option<i64>,
) -> Result<()> {
    let db = Database::open_at(db_path)?;
    queries::insert_audit_event(
        &db,
        &NewAuditEvent {
            event_type,
            repository_id: Some(repository_id),
            account_login,
            summary,
            result,
            exit_code,
            duration_ms,
        },
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use shehata_storage::RepositoryRecord;
    use tempfile::TempDir;
    use uuid::Uuid;

    fn git(repo: &Path, args: &[&str]) {
        let status = std::process::Command::new("git")
            .arg("-C")
            .arg(repo)
            .args(args)
            .status()
            .unwrap();
        assert!(status.success());
    }

    fn fixture() -> (TempDir, PathBuf, PathBuf, String) {
        let temp = tempfile::tempdir().unwrap();
        let repo = temp.path().join("repo");
        fs::create_dir(&repo).unwrap();
        git(&repo, &["init"]);
        git(
            &repo,
            &[
                "remote",
                "add",
                "origin",
                "https://github.com/acme/demo.git",
            ],
        );
        git(
            &repo,
            &["config", "--local", "credential.helper", "manager"],
        );
        let git_dir = repo.join(".git");
        let id = Uuid::new_v4().to_string();
        fs::create_dir_all(git_dir.join("shehata-git")).unwrap();
        fs::write(git_dir.join("shehata-git/repository-id"), &id).unwrap();
        let db_path = temp.path().join("db.sqlite");
        let db = Database::open_at(&db_path).unwrap();
        let account_id = queries::upsert_account(&db, "github.com", "alice", "valid").unwrap();
        let now = Utc::now().to_rfc3339();
        queries::insert_repository(
            &db,
            &RepositoryRecord {
                id: id.clone(),
                canonical_path: fs::canonicalize(&repo)
                    .unwrap()
                    .to_string_lossy()
                    .into_owned(),
                git_dir: Some(
                    fs::canonicalize(&git_dir)
                        .unwrap()
                        .to_string_lossy()
                        .into_owned(),
                ),
                git_common_dir: None,
                display_name: "demo".into(),
                host: Some("github.com".into()),
                owner: Some("acme".into()),
                repo_name: Some("demo".into()),
                remote_name: Some("origin".into()),
                remote_url: Some("https://github.com/acme/demo.git".into()),
                current_branch: Some("main".into()),
                assigned_account_id: Some(account_id),
                commit_name: None,
                commit_email: None,
                push_policy: "allow_normal_push".into(),
                created_at: now.clone(),
                updated_at: now.clone(),
                last_seen_at: Some(now),
            },
        )
        .unwrap();
        (temp, repo, db_path, id)
    }

    #[tokio::test]
    async fn links_and_unlinks_with_exact_restore() {
        let (_temp, repo, db_path, id) = fixture();
        let helper = std::env::current_exe().unwrap();
        link_repository_at(
            &db_path,
            &helper,
            LinkRepositoryRequest {
                repository_id: id.clone(),
            },
        )
        .await
        .unwrap();

        let runner = GitRunner::locate().unwrap();
        let helpers = read_local_config_values(&runner, &repo, HELPER_KEY)
            .await
            .unwrap();
        assert_eq!(helpers.len(), 2);
        assert_eq!(helpers[0], "");
        assert!(helpers[1].contains("--repo-id"));
        assert_eq!(
            read_local_config_values(&runner, &repo, HTTP_PATH_KEY)
                .await
                .unwrap(),
            ["true"]
        );

        let result = unlink_repository_at(
            &db_path,
            UnlinkRepositoryRequest {
                repository_id: id.clone(),
                restore_identity: false,
            },
        )
        .await
        .unwrap();
        assert!(result.restored_keys.contains(&HELPER_KEY.to_string()));
        assert_eq!(
            read_local_config_values(&runner, &repo, HELPER_KEY)
                .await
                .unwrap(),
            ["manager"]
        );
        assert!(read_local_config_values(&runner, &repo, HTTP_PATH_KEY)
            .await
            .unwrap()
            .is_empty());
        let db = Database::open_at(&db_path).unwrap();
        assert!(queries::find_repository_by_id(&db, &id)
            .unwrap()
            .unwrap()
            .assigned_account_id
            .is_none());
    }
}
