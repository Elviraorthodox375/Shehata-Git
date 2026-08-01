//! Phase 7 safe local Git actions.
//!
//! This surface intentionally exposes only selected-path staging, unstaging,
//! and normal commits. Every dynamic value is passed as an argument, never as
//! shell text, and `--` terminates options before file paths.

use std::path::{Component, Path};
use std::time::Instant;

use serde::{Deserialize, Serialize};
use shehata_git::{GitError, GitRunner};
use shehata_storage::{queries, Database, NewAuditEvent, RepositoryRecord};

use crate::error::{Result, ShehataError};

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct ChangeEntry {
    pub path: String,
    pub index_status: String,
    pub worktree_status: String,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct RepositoryActionStatus {
    pub repository_id: String,
    pub branch: Option<String>,
    pub detached_head: bool,
    pub changes: Vec<ChangeEntry>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PathsRequest {
    pub repository_id: String,
    pub paths: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CommitRequest {
    pub repository_id: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct GitActionResult {
    pub repository_id: String,
    pub action: String,
    pub changed_paths: usize,
    pub commit: Option<String>,
}

pub async fn status(repository_id: &str) -> Result<RepositoryActionStatus> {
    let db_path = Database::default_path()?;
    status_at(&db_path, repository_id).await
}

pub async fn status_at(db_path: &Path, repository_id: &str) -> Result<RepositoryActionStatus> {
    let repository = load_repository(db_path, repository_id)?;
    let path = Path::new(&repository.canonical_path);
    let git = GitRunner::locate()?;
    let branch_output = git
        .run_in(path.into(), &["symbolic-ref", "--quiet", "--short", "HEAD"])
        .await?;
    let branch = branch_output
        .success()
        .then(|| branch_output.stdout.trim().to_string())
        .filter(|value| !value.is_empty());
    let head_exists = git
        .run_in(path.into(), &["rev-parse", "--verify", "HEAD"])
        .await?
        .success();
    let output = git
        .run_checked(
            Some(path),
            &["status", "--porcelain=v1", "-z", "--untracked-files=all"],
        )
        .await?;
    Ok(RepositoryActionStatus {
        repository_id: repository.id,
        detached_head: branch.is_none() && head_exists,
        branch,
        changes: parse_porcelain_v1_z(&output.stdout),
    })
}

pub async fn stage(request: PathsRequest) -> Result<GitActionResult> {
    let db_path = Database::default_path()?;
    stage_at(&db_path, request).await
}

pub async fn stage_at(db_path: &Path, request: PathsRequest) -> Result<GitActionResult> {
    run_paths_action(db_path, request, "stage", "add").await
}

pub async fn unstage(request: PathsRequest) -> Result<GitActionResult> {
    let db_path = Database::default_path()?;
    unstage_at(&db_path, request).await
}

pub async fn unstage_at(db_path: &Path, request: PathsRequest) -> Result<GitActionResult> {
    run_paths_action(db_path, request, "unstage", "restore").await
}

pub async fn commit(request: CommitRequest) -> Result<GitActionResult> {
    let db_path = Database::default_path()?;
    commit_at(&db_path, request).await
}

pub async fn commit_at(db_path: &Path, request: CommitRequest) -> Result<GitActionResult> {
    let repository = load_repository(db_path, &request.repository_id)?;
    require_assignment(&repository)?;
    let message = validate_commit_message(&request.message)?;
    let path = Path::new(&repository.canonical_path);
    let git = GitRunner::locate()?;

    let conflicts = git
        .run_checked(
            Some(path),
            &["diff", "--cached", "--name-only", "--diff-filter=U"],
        )
        .await?;
    if !conflicts.stdout.trim().is_empty() {
        return Err(ShehataError::ConflictsPresent);
    }
    let staged = git
        .run_in(Some(path), &["diff", "--cached", "--quiet", "--exit-code"])
        .await?;
    if staged.code == 0 {
        return Err(ShehataError::InvalidInput(
            "there are no staged changes to commit".to_string(),
        ));
    }
    if staged.code != 1 {
        return Err(GitError::Exit {
            code: staged.code,
            message: staged.stderr.trim().to_string(),
        }
        .into());
    }

    let started = Instant::now();
    let output = git
        .run_checked(Some(path), &["commit", "-m", &message])
        .await;
    match output {
        Ok(_) => {
            let commit = git
                .run_checked(Some(path), &["rev-parse", "HEAD"])
                .await?
                .stdout
                .trim()
                .to_string();
            write_audit(
                db_path,
                &repository.id,
                "commit",
                "Created a normal commit",
                "success",
                Some(0),
                started,
            )?;
            Ok(GitActionResult {
                repository_id: repository.id,
                action: "commit".to_string(),
                changed_paths: 0,
                commit: Some(commit),
            })
        }
        Err(error) => {
            write_audit(
                db_path,
                &repository.id,
                "commit",
                "Normal commit failed",
                "failure",
                git_error_code(&error),
                started,
            )?;
            Err(error.into())
        }
    }
}

async fn run_paths_action(
    db_path: &Path,
    request: PathsRequest,
    action: &str,
    git_action: &str,
) -> Result<GitActionResult> {
    let repository = load_repository(db_path, &request.repository_id)?;
    require_assignment(&repository)?;
    let paths = validate_paths(request.paths)?;
    let git = GitRunner::locate()?;
    let repository_path = Path::new(&repository.canonical_path);
    let mut args = match git_action {
        "add" => vec!["add".to_string(), "--".to_string()],
        "restore" => {
            let has_head = git
                .run_in(Some(repository_path), &["rev-parse", "--verify", "HEAD"])
                .await?
                .success();
            if has_head {
                vec![
                    "restore".to_string(),
                    "--staged".to_string(),
                    "--".to_string(),
                ]
            } else {
                // Unstage an unborn branch without touching the worktree.
                vec![
                    "rm".to_string(),
                    "--cached".to_string(),
                    "--ignore-unmatch".to_string(),
                    "--".to_string(),
                ]
            }
        }
        _ => {
            return Err(ShehataError::Internal(
                "unsupported safe action".to_string(),
            ))
        }
    };
    args.extend(paths.iter().cloned());
    let refs = args.iter().map(String::as_str).collect::<Vec<_>>();
    let started = Instant::now();
    let result = git.run_checked(Some(repository_path), &refs).await;
    match result {
        Ok(_) => {
            write_audit(
                db_path,
                &repository.id,
                action,
                if action == "stage" {
                    "Staged selected paths"
                } else {
                    "Unstaged selected paths"
                },
                "success",
                Some(0),
                started,
            )?;
            Ok(GitActionResult {
                repository_id: repository.id,
                action: action.to_string(),
                changed_paths: paths.len(),
                commit: None,
            })
        }
        Err(error) => {
            write_audit(
                db_path,
                &repository.id,
                action,
                "Selected-path Git action failed",
                "failure",
                git_error_code(&error),
                started,
            )?;
            Err(error.into())
        }
    }
}

fn load_repository(db_path: &Path, repository_id: &str) -> Result<RepositoryRecord> {
    let db = Database::open_at(db_path)?;
    queries::find_repository_by_id(&db, repository_id.trim())?
        .ok_or_else(|| ShehataError::RepositoryNotFound(repository_id.to_string()))
}

fn require_assignment(repository: &RepositoryRecord) -> Result<()> {
    if repository.assigned_account_id.is_none() {
        return Err(ShehataError::RepositoryNotLinked(repository.id.clone()));
    }
    Ok(())
}

fn validate_paths(paths: Vec<String>) -> Result<Vec<String>> {
    if paths.is_empty() || paths.len() > 500 {
        return Err(ShehataError::InvalidInput(
            "select between 1 and 500 paths".to_string(),
        ));
    }
    let mut clean = Vec::with_capacity(paths.len());
    for value in paths {
        if value.is_empty()
            || value.len() > 4096
            || value.contains(['\0', '\r', '\n'])
            || Path::new(&value).is_absolute()
        {
            return Err(ShehataError::InvalidInput(
                "invalid repository path".to_string(),
            ));
        }
        let mut meaningful = false;
        for component in Path::new(&value).components() {
            match component {
                Component::Normal(part) => {
                    meaningful = true;
                    if part.to_string_lossy().eq_ignore_ascii_case(".git") {
                        return Err(ShehataError::InvalidInput(
                            "Git metadata paths cannot be selected".to_string(),
                        ));
                    }
                }
                Component::CurDir => {}
                _ => {
                    return Err(ShehataError::InvalidInput(
                        "paths must stay inside the repository".to_string(),
                    ))
                }
            }
        }
        if !meaningful {
            return Err(ShehataError::InvalidInput(
                "invalid repository path".to_string(),
            ));
        }
        clean.push(value);
    }
    Ok(clean)
}

fn validate_commit_message(message: &str) -> Result<String> {
    let message = message.trim();
    if message.is_empty()
        || message.len() > 1_000
        || message.contains('\0')
        || message.chars().any(|character| character == '\r')
    {
        return Err(ShehataError::InvalidInput(
            "commit message must be 1-1000 safe characters".to_string(),
        ));
    }
    Ok(message.to_string())
}

fn parse_porcelain_v1_z(raw: &str) -> Vec<ChangeEntry> {
    let mut records = raw.split_terminator('\0');
    let mut changes = Vec::new();
    while let Some(record) = records.next() {
        if record.len() < 4 {
            continue;
        }
        let mut status = record.chars();
        let index_status = status.next().unwrap_or(' ').to_string();
        let worktree_status = status.next().unwrap_or(' ').to_string();
        let path = record[3..].to_string();
        let renamed = matches!(index_status.as_str(), "R" | "C")
            || matches!(worktree_status.as_str(), "R" | "C");
        if renamed {
            let _old_path = records.next();
        }
        changes.push(ChangeEntry {
            path,
            index_status,
            worktree_status,
        });
    }
    changes
}

fn git_error_code(error: &GitError) -> Option<i64> {
    match error {
        GitError::Exit { code, .. } => Some((*code).into()),
        _ => None,
    }
}

fn write_audit(
    db_path: &Path,
    repository_id: &str,
    event_type: &str,
    summary: &str,
    result: &str,
    exit_code: Option<i64>,
    started: Instant,
) -> Result<()> {
    let db = Database::open_at(db_path)?;
    queries::insert_audit_event(
        &db,
        &NewAuditEvent {
            event_type,
            repository_id: Some(repository_id),
            account_login: None,
            summary,
            result,
            exit_code,
            duration_ms: Some(started.elapsed().as_millis().min(i64::MAX as u128) as i64),
        },
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;
    use std::process::Command;

    use chrono::Utc;
    use tempfile::TempDir;
    use uuid::Uuid;

    use super::*;

    fn git(repo: &Path, args: &[&str]) {
        assert!(Command::new("git")
            .arg("-C")
            .arg(repo)
            .args(args)
            .status()
            .unwrap()
            .success());
    }

    fn fixture() -> (TempDir, PathBuf, PathBuf, String) {
        let temp = tempfile::tempdir().unwrap();
        let repo = temp.path().join("repo");
        fs::create_dir(&repo).unwrap();
        git(&repo, &["init"]);
        git(&repo, &["config", "user.name", "Test User"]);
        git(&repo, &["config", "user.email", "test@example.com"]);
        let id = Uuid::new_v4().to_string();
        let db_path = temp.path().join("db.sqlite");
        let db = Database::open_at(&db_path).unwrap();
        let account = queries::upsert_account(&db, "github.com", "alice", "valid").unwrap();
        let now = Utc::now().to_rfc3339();
        queries::insert_repository(
            &db,
            &RepositoryRecord {
                id: id.clone(),
                canonical_path: fs::canonicalize(&repo)
                    .unwrap()
                    .to_string_lossy()
                    .into_owned(),
                git_dir: Some(repo.join(".git").to_string_lossy().into_owned()),
                git_common_dir: None,
                display_name: "repo".into(),
                host: Some("github.com".into()),
                owner: Some("acme".into()),
                repo_name: Some("repo".into()),
                remote_name: Some("origin".into()),
                remote_url: Some("https://github.com/acme/repo.git".into()),
                current_branch: Some("main".into()),
                assigned_account_id: Some(account),
                commit_name: Some("Test User".into()),
                commit_email: Some("test@example.com".into()),
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
    async fn stages_commits_and_reports_status() {
        let (_temp, repo, db_path, id) = fixture();
        fs::write(repo.join("hello.txt"), "hello").unwrap();
        let before = status_at(&db_path, &id).await.unwrap();
        assert_eq!(before.changes[0].worktree_status, "?");
        stage_at(
            &db_path,
            PathsRequest {
                repository_id: id.clone(),
                paths: vec!["hello.txt".into()],
            },
        )
        .await
        .unwrap();
        unstage_at(
            &db_path,
            PathsRequest {
                repository_id: id.clone(),
                paths: vec!["hello.txt".into()],
            },
        )
        .await
        .unwrap();
        assert_eq!(
            status_at(&db_path, &id).await.unwrap().changes[0].index_status,
            "?"
        );
        stage_at(
            &db_path,
            PathsRequest {
                repository_id: id.clone(),
                paths: vec!["hello.txt".into()],
            },
        )
        .await
        .unwrap();
        let result = commit_at(
            &db_path,
            CommitRequest {
                repository_id: id.clone(),
                message: "feat: add hello".into(),
            },
        )
        .await
        .unwrap();
        assert_eq!(result.commit.as_deref().map(str::len), Some(40));
        assert!(status_at(&db_path, &id).await.unwrap().changes.is_empty());
    }

    #[test]
    fn rejects_escaping_and_git_metadata_paths() {
        assert!(validate_paths(vec!["../secret".into()]).is_err());
        assert!(validate_paths(vec![".git/config".into()]).is_err());
        assert!(validate_paths(vec!["-danger".into()]).is_ok());
    }
}
