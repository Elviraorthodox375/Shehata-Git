//! Phase 7 safe local Git actions.
//!
//! This surface intentionally exposes only selected-path staging, unstaging,
//! and normal commits. Every dynamic value is passed as an argument, never as
//! shell text, and `--` terminates options before file paths.

use std::path::{Component, Path};
use std::time::Instant;

use serde::{Deserialize, Serialize};
use shehata_git::{
    parse_remote_url, read_local_config_values, GitError, GitRunner, RemoteProtocol,
};
use shehata_github::GhRunner;
use shehata_storage::{queries, AccountRecord, Database, NewAuditEvent, RepositoryRecord};

use crate::error::{Result, ShehataError};
use crate::models::PushPolicy;

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

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct DiffSummary {
    pub repository_id: String,
    pub changed_paths: usize,
    pub staged_paths: usize,
    pub unstaged_paths: usize,
    pub untracked_paths: usize,
    pub conflict_paths: usize,
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

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ActionCaller {
    Desktop,
    Cli,
    Mcp,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RepositoryActionRequest {
    pub repository_id: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PushRequest {
    pub repository_id: String,
    pub caller: ActionCaller,
    #[serde(default)]
    pub approved: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SetPushPolicyRequest {
    pub repository_id: String,
    pub push_policy: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct PushPolicyResult {
    pub repository_id: String,
    pub push_policy: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct NetworkActionResult {
    pub repository_id: String,
    pub action: String,
    pub remote_name: String,
    pub branch: String,
    pub account_login: String,
    pub head_commit: String,
    pub ahead_before: usize,
    pub behind_before: usize,
}

#[derive(Debug)]
struct NetworkPlan {
    repository: RepositoryRecord,
    account: AccountRecord,
    remote_name: String,
    remote_branch: String,
    branch: String,
    ahead: usize,
    behind: usize,
}

pub async fn status(repository_id: &str) -> Result<RepositoryActionStatus> {
    let db_path = Database::default_path()?;
    status_at(&db_path, repository_id).await
}

pub async fn diff_summary(repository_id: &str) -> Result<DiffSummary> {
    let status = status(repository_id).await?;
    Ok(summarize_status(status))
}

fn summarize_status(status: RepositoryActionStatus) -> DiffSummary {
    let staged_paths = status
        .changes
        .iter()
        .filter(|change| change.index_status != " " && change.index_status != "?")
        .count();
    let unstaged_paths = status
        .changes
        .iter()
        .filter(|change| change.worktree_status != " " && change.worktree_status != "?")
        .count();
    let untracked_paths = status
        .changes
        .iter()
        .filter(|change| change.index_status == "?" && change.worktree_status == "?")
        .count();
    let conflict_paths = status
        .changes
        .iter()
        .filter(|change| {
            matches!(
                (
                    change.index_status.as_str(),
                    change.worktree_status.as_str()
                ),
                ("D", "D")
                    | ("A", "U")
                    | ("U", "D")
                    | ("U", "A")
                    | ("D", "U")
                    | ("A", "A")
                    | ("U", "U")
            )
        })
        .count();
    DiffSummary {
        repository_id: status.repository_id,
        changed_paths: status.changes.len(),
        staged_paths,
        unstaged_paths,
        untracked_paths,
        conflict_paths,
    }
}

pub fn set_push_policy(request: SetPushPolicyRequest) -> Result<PushPolicyResult> {
    let db_path = Database::default_path()?;
    set_push_policy_at(&db_path, request)
}

fn set_push_policy_at(db_path: &Path, request: SetPushPolicyRequest) -> Result<PushPolicyResult> {
    let repository = load_repository(db_path, &request.repository_id)?;
    require_assignment(&repository)?;
    let policy = PushPolicy::parse(request.push_policy.trim()).ok_or_else(|| {
        ShehataError::InvalidInput("unsupported repository push policy".to_string())
    })?;
    let db = Database::open_at(db_path)?;
    queries::update_repository_push_policy(&db, &repository.id, policy.as_str())?;
    queries::insert_audit_event(
        &db,
        &NewAuditEvent {
            event_type: "push_policy_changed",
            repository_id: Some(&repository.id),
            account_login: None,
            summary: "Updated repository push policy",
            result: "success",
            exit_code: Some(0),
            duration_ms: None,
        },
    )?;
    Ok(PushPolicyResult {
        repository_id: repository.id,
        push_policy: policy.as_str().to_string(),
    })
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

pub async fn pull_ff_only(request: RepositoryActionRequest) -> Result<NetworkActionResult> {
    let db_path = Database::default_path()?;
    pull_ff_only_at(&db_path, request, true).await
}

async fn pull_ff_only_at(
    db_path: &Path,
    request: RepositoryActionRequest,
    check_token: bool,
) -> Result<NetworkActionResult> {
    let started = Instant::now();
    let plan =
        prepare_network_plan(db_path, &request.repository_id, None, false, check_token).await?;
    let git = GitRunner::locate()?;
    let path = Path::new(&plan.repository.canonical_path);
    let result = execute_pull(&git, path, &plan.remote_name, &plan.remote_branch).await;
    finish_network_action(db_path, plan, "pull_ff_only", result, started).await
}

pub async fn push(request: PushRequest) -> Result<NetworkActionResult> {
    let db_path = Database::default_path()?;
    push_at(&db_path, request, true).await
}

async fn push_at(
    db_path: &Path,
    request: PushRequest,
    check_token: bool,
) -> Result<NetworkActionResult> {
    let started = Instant::now();
    let plan_result = prepare_network_plan(
        db_path,
        &request.repository_id,
        Some(request.caller),
        request.approved,
        check_token,
    )
    .await;
    let plan = match plan_result {
        Ok(plan) => plan,
        Err(error) => {
            if uuid::Uuid::parse_str(request.repository_id.trim()).is_ok() {
                write_network_audit(
                    db_path,
                    request.repository_id.trim(),
                    None,
                    "push_preflight",
                    "Push preflight blocked the operation",
                    "blocked",
                    None,
                    started,
                )?;
            }
            return Err(error);
        }
    };

    write_network_audit(
        db_path,
        &plan.repository.id,
        Some(&plan.account.login),
        "push_preflight",
        "Push preflight passed",
        "success",
        Some(0),
        started,
    )?;
    let git = GitRunner::locate()?;
    let path = Path::new(&plan.repository.canonical_path);
    let result = execute_push(&git, path, &plan.remote_name, &plan.remote_branch).await;
    finish_network_action(db_path, plan, "push", result, started).await
}

async fn prepare_network_plan(
    db_path: &Path,
    repository_id: &str,
    push_caller: Option<ActionCaller>,
    approved: bool,
    check_token: bool,
) -> Result<NetworkPlan> {
    let (repository, account) = load_linked_repository(db_path, repository_id)?;
    let repo_path = Path::new(&repository.canonical_path);
    let git = GitRunner::locate()?;

    let helper_path = crate::routing::locate_helper()?;
    let expected_helper = crate::routing::helper_config_value(&helper_path, &repository.id)?;
    ensure_routing_configured(&git, repo_path, &expected_helper, &repository.id).await?;

    let expected_url = repository.remote_url.as_deref().ok_or_else(|| {
        ShehataError::InvalidInput("repository has no expected remote URL".to_string())
    })?;
    let expected = parse_remote_url(expected_url)
        .map_err(|_| ShehataError::InvalidInput("expected remote URL is invalid".to_string()))?;
    if expected.protocol != RemoteProtocol::Https
        || !expected.host.eq_ignore_ascii_case(&account.host)
    {
        return Err(ShehataError::AuthenticationFailed);
    }

    let branch_output = git
        .run_in(
            Some(repo_path),
            &["symbolic-ref", "--quiet", "--short", "HEAD"],
        )
        .await?;
    let branch = if branch_output.success() {
        branch_output.stdout.trim().to_string()
    } else {
        return Err(ShehataError::DetachedHead);
    };
    validate_git_ref(&git, repo_path, &branch).await?;

    let conflicts = git
        .run_checked(Some(repo_path), &["diff", "--name-only", "--diff-filter=U"])
        .await?;
    if !conflicts.stdout.trim().is_empty() {
        return Err(ShehataError::ConflictsPresent);
    }

    let upstream_output = git
        .run_in(
            Some(repo_path),
            &[
                "rev-parse",
                "--abbrev-ref",
                "--symbolic-full-name",
                "@{upstream}",
            ],
        )
        .await?;
    if !upstream_output.success() {
        return Err(ShehataError::NoUpstream);
    }
    let upstream = upstream_output.stdout.trim();
    let (remote_name, remote_branch) = upstream
        .split_once('/')
        .filter(|(remote, branch)| !remote.is_empty() && !branch.is_empty())
        .ok_or(ShehataError::NoUpstream)?;
    validate_remote_name(remote_name)?;
    validate_git_ref(&git, repo_path, remote_branch).await?;

    let actual_url = git
        .run_checked(Some(repo_path), &["remote", "get-url", remote_name])
        .await?
        .stdout;
    let actual = parse_remote_url(actual_url.trim())
        .map_err(|_| ShehataError::InvalidInput("upstream remote must use HTTPS".to_string()))?;
    if actual.protocol != RemoteProtocol::Https
        || !actual.host.eq_ignore_ascii_case(&expected.host)
        || !actual.owner.eq_ignore_ascii_case(&expected.owner)
        || !actual.repo.eq_ignore_ascii_case(&expected.repo)
    {
        return Err(ShehataError::OperationBlocked(
            "upstream remote does not match the linked repository".to_string(),
        ));
    }

    if check_token {
        let gh = GhRunner::locate().map_err(|_| ShehataError::AuthenticationFailed)?;
        let token = gh
            .token_for(&account.host, &account.login)
            .await
            .map_err(|_| ShehataError::AuthenticationFailed)?;
        drop(token);
    }

    // Refresh only remote-tracking refs. No merge, checkout, or worktree write.
    git.run_checked(
        Some(repo_path),
        &["fetch", "--quiet", "--prune", remote_name],
    )
    .await?;
    let counts = git
        .run_in(
            Some(repo_path),
            &["rev-list", "--left-right", "--count", "HEAD...@{upstream}"],
        )
        .await?;
    if !counts.success() {
        return Err(ShehataError::NoUpstream);
    }
    let (ahead, behind) = parse_ahead_behind(&counts.stdout)?;

    if push_caller.is_some() && behind > 0 {
        return Err(ShehataError::NonFastForward);
    }
    if let Some(caller) = push_caller {
        enforce_push_policy(&repository.push_policy, caller, approved)?;
    }

    Ok(NetworkPlan {
        repository,
        account,
        remote_name: remote_name.to_string(),
        remote_branch: remote_branch.to_string(),
        branch,
        ahead,
        behind,
    })
}

async fn finish_network_action(
    db_path: &Path,
    plan: NetworkPlan,
    action: &str,
    result: std::result::Result<shehata_git::CommandOutput, GitError>,
    started: Instant,
) -> Result<NetworkActionResult> {
    match result {
        Ok(_) => {
            let head_commit = GitRunner::locate()?
                .run_checked(
                    Some(Path::new(&plan.repository.canonical_path)),
                    &["rev-parse", "HEAD"],
                )
                .await?
                .stdout
                .trim()
                .to_string();
            write_network_audit(
                db_path,
                &plan.repository.id,
                Some(&plan.account.login),
                action,
                if action == "push" {
                    "Normal push completed"
                } else {
                    "Fast-forward-only pull completed"
                },
                "success",
                Some(0),
                started,
            )?;
            Ok(NetworkActionResult {
                repository_id: plan.repository.id,
                action: action.to_string(),
                remote_name: plan.remote_name,
                branch: plan.branch,
                account_login: plan.account.login,
                head_commit,
                ahead_before: plan.ahead,
                behind_before: plan.behind,
            })
        }
        Err(error) => {
            write_network_audit(
                db_path,
                &plan.repository.id,
                Some(&plan.account.login),
                action,
                "Network Git action failed",
                "failure",
                git_error_code(&error),
                started,
            )?;
            Err(error.into())
        }
    }
}

async fn ensure_routing_configured(
    git: &GitRunner,
    repo_path: &Path,
    expected_helper: &str,
    repository_id: &str,
) -> Result<()> {
    let helpers = read_local_config_values(git, repo_path, "credential.helper").await?;
    let use_http_path = read_local_config_values(git, repo_path, "credential.useHttpPath").await?;
    let configured = helpers.first().is_some_and(String::is_empty)
        && helpers.iter().any(|helper| helper == expected_helper)
        && use_http_path.iter().any(|value| value == "true");
    if !configured {
        return Err(ShehataError::RepositoryNotLinked(repository_id.to_string()));
    }
    Ok(())
}

fn load_linked_repository(
    db_path: &Path,
    repository_id: &str,
) -> Result<(RepositoryRecord, AccountRecord)> {
    let repository = load_repository(db_path, repository_id)?;
    let db = Database::open_at(db_path)?;
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
    Ok((repository, account))
}

async fn validate_git_ref(git: &GitRunner, repo_path: &Path, value: &str) -> Result<()> {
    if value.contains(['\0', '\r', '\n']) {
        return Err(ShehataError::InvalidInput("invalid Git ref".to_string()));
    }
    let output = git
        .run_in(Some(repo_path), &["check-ref-format", "--branch", value])
        .await?;
    if !output.success() {
        return Err(ShehataError::InvalidInput("invalid Git ref".to_string()));
    }
    Ok(())
}

fn validate_remote_name(value: &str) -> Result<()> {
    if value.is_empty()
        || value.starts_with('-')
        || value.len() > 255
        || value.contains(['\0', '\r', '\n'])
    {
        return Err(ShehataError::InvalidInput(
            "invalid upstream remote name".to_string(),
        ));
    }
    Ok(())
}

fn parse_ahead_behind(value: &str) -> Result<(usize, usize)> {
    let mut values = value.split_whitespace();
    let ahead = values
        .next()
        .and_then(|value| value.parse().ok())
        .ok_or_else(|| ShehataError::Internal("invalid ahead/behind output".to_string()))?;
    let behind = values
        .next()
        .and_then(|value| value.parse().ok())
        .ok_or_else(|| ShehataError::Internal("invalid ahead/behind output".to_string()))?;
    if values.next().is_some() {
        return Err(ShehataError::Internal(
            "invalid ahead/behind output".to_string(),
        ));
    }
    Ok((ahead, behind))
}

fn enforce_push_policy(policy: &str, caller: ActionCaller, approved: bool) -> Result<()> {
    let policy = PushPolicy::parse(policy).ok_or_else(|| {
        ShehataError::OperationBlocked("repository push policy is invalid".to_string())
    })?;
    match policy {
        PushPolicy::AllowNormalPush => Ok(()),
        PushPolicy::AskBeforePush if approved => Ok(()),
        PushPolicy::AskBeforePush => Err(ShehataError::ApprovalRequired),
        PushPolicy::BlockAiPush if caller == ActionCaller::Mcp => Err(
            ShehataError::OperationBlocked("AI pushes are blocked for this repository".to_string()),
        ),
        PushPolicy::BlockAiPush => Ok(()),
    }
}

async fn execute_pull(
    git: &GitRunner,
    repo_path: &Path,
    remote_name: &str,
    remote_branch: &str,
) -> std::result::Result<shehata_git::CommandOutput, GitError> {
    git.run_checked(
        Some(repo_path),
        &["pull", "--ff-only", "--no-edit", remote_name, remote_branch],
    )
    .await
}

async fn execute_push(
    git: &GitRunner,
    repo_path: &Path,
    remote_name: &str,
    remote_branch: &str,
) -> std::result::Result<shehata_git::CommandOutput, GitError> {
    let destination = format!("HEAD:refs/heads/{remote_branch}");
    git.run_checked(
        Some(repo_path),
        &["push", "--porcelain", remote_name, &destination],
    )
    .await
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
    let repository_id = repository_id.trim();
    uuid::Uuid::parse_str(repository_id)
        .map_err(|_| ShehataError::InvalidInput("invalid repository id".to_string()))?;
    let db = Database::open_at(db_path)?;
    queries::find_repository_by_id(&db, repository_id)?
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

#[allow(clippy::too_many_arguments)]
fn write_network_audit(
    db_path: &Path,
    repository_id: &str,
    account_login: Option<&str>,
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
            account_login,
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

    fn git_output(repo: &Path, args: &[&str]) -> String {
        let output = Command::new("git")
            .arg("-C")
            .arg(repo)
            .args(args)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        String::from_utf8(output.stdout).unwrap().trim().to_string()
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
        let summary = summarize_status(before.clone());
        assert_eq!(summary.untracked_paths, 1);
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

    #[test]
    fn enforces_push_policies_by_caller_and_approval() {
        assert!(enforce_push_policy("allow_normal_push", ActionCaller::Mcp, false).is_ok());
        assert!(matches!(
            enforce_push_policy("ask_before_push", ActionCaller::Desktop, false),
            Err(ShehataError::ApprovalRequired)
        ));
        assert!(enforce_push_policy("ask_before_push", ActionCaller::Desktop, true).is_ok());
        assert!(matches!(
            enforce_push_policy("block_ai_push", ActionCaller::Mcp, true),
            Err(ShehataError::OperationBlocked(_))
        ));
        assert!(enforce_push_policy("block_ai_push", ActionCaller::Cli, false).is_ok());
    }

    #[test]
    fn persists_only_supported_push_policies() {
        let (_temp, _repo, db_path, id) = fixture();
        let result = set_push_policy_at(
            &db_path,
            SetPushPolicyRequest {
                repository_id: id.clone(),
                push_policy: "ask_before_push".into(),
            },
        )
        .unwrap();
        assert_eq!(result.push_policy, "ask_before_push");
        let db = Database::open_at(&db_path).unwrap();
        assert_eq!(
            queries::find_repository_by_id(&db, &id)
                .unwrap()
                .unwrap()
                .push_policy,
            "ask_before_push"
        );
        assert!(set_push_policy_at(
            &db_path,
            SetPushPolicyRequest {
                repository_id: id,
                push_policy: "force_push".into(),
            },
        )
        .is_err());
    }

    #[test]
    fn parses_ahead_and_behind_strictly() {
        assert_eq!(parse_ahead_behind("2\t3\n").unwrap(), (2, 3));
        assert!(parse_ahead_behind("2").is_err());
        assert!(parse_ahead_behind("2 3 extra").is_err());
    }

    #[tokio::test]
    async fn fixed_network_commands_pull_ff_only_and_push_normally() {
        let temp = tempfile::tempdir().unwrap();
        let remote = temp.path().join("remote.git");
        assert!(Command::new("git")
            .args(["init", "--bare"])
            .arg(&remote)
            .status()
            .unwrap()
            .success());

        let first = temp.path().join("first");
        fs::create_dir(&first).unwrap();
        git(&first, &["init", "--initial-branch=main"]);
        git(&first, &["config", "user.name", "First User"]);
        git(&first, &["config", "user.email", "first@example.com"]);
        fs::write(first.join("one.txt"), "one").unwrap();
        git(&first, &["add", "--", "one.txt"]);
        git(&first, &["commit", "-m", "feat: first"]);
        git(
            &first,
            &["remote", "add", "origin", &remote.to_string_lossy()],
        );
        git(&first, &["push", "-u", "origin", "main"]);

        let second = temp.path().join("second");
        assert!(Command::new("git")
            .args(["clone", "--branch", "main"])
            .arg(&remote)
            .arg(&second)
            .status()
            .unwrap()
            .success());
        git(&second, &["config", "user.name", "Second User"]);
        git(&second, &["config", "user.email", "second@example.com"]);
        fs::write(second.join("two.txt"), "two").unwrap();
        git(&second, &["add", "--", "two.txt"]);
        git(&second, &["commit", "-m", "feat: second"]);
        git(&second, &["push", "origin", "main"]);

        let runner = GitRunner::locate().unwrap();
        execute_pull(&runner, &first, "origin", "main")
            .await
            .unwrap();
        assert!(first.join("two.txt").is_file());

        fs::write(first.join("three.txt"), "three").unwrap();
        git(&first, &["add", "--", "three.txt"]);
        git(&first, &["commit", "-m", "feat: third"]);
        execute_push(&runner, &first, "origin", "main")
            .await
            .unwrap();
        assert_eq!(
            git_output(&first, &["rev-parse", "HEAD"]),
            git_output(&remote, &["rev-parse", "refs/heads/main"])
        );
    }
}
