//! Repository orchestration shared by desktop, CLI, and MCP callers.
//!
//! Async Git discovery is deliberately separated from synchronous SQLite
//! persistence so a rusqlite connection is never held across an await point.

use chrono::Utc;
use serde::Serialize;
use shehata_git::{
    parse_remote_url, DiscoveredRepository, GitRunner, RemoteProtocol, RepositoryRemote,
};
use shehata_storage::{queries, Database, RepositoryRecord};
use uuid::Uuid;

use crate::Result;

#[derive(Debug, Clone, Serialize)]
pub struct RepositorySummary {
    pub id: String,
    pub display_name: String,
    pub canonical_path: String,
    pub host: Option<String>,
    pub owner: Option<String>,
    pub repo_name: Option<String>,
    pub remote_name: Option<String>,
    pub remote_url: Option<String>,
    pub remote_protocol: Option<String>,
    pub current_branch: Option<String>,
    pub assigned_login: Option<String>,
    pub commit_name: Option<String>,
    pub commit_email: Option<String>,
    pub push_policy: String,
}

pub async fn discover_selected_repository(path: &str) -> Result<DiscoveredRepository> {
    let git = GitRunner::locate()?;
    Ok(shehata_git::discover_repository(&git, std::path::Path::new(path)).await?)
}

pub fn save_discovered_repository(
    db: &Database,
    discovered: &DiscoveredRepository,
) -> Result<RepositoryRecord> {
    let canonical_path = discovered.canonical_path.to_string_lossy().into_owned();
    let existing = queries::find_repository_by_path(db, &canonical_path)?;
    let primary_remote = primary_remote(discovered);
    let now = Utc::now().to_rfc3339();

    let record = RepositoryRecord {
        id: existing
            .as_ref()
            .map(|repo| repo.id.clone())
            .unwrap_or_else(|| Uuid::new_v4().to_string()),
        canonical_path: canonical_path.clone(),
        git_dir: Some(discovered.git_dir.to_string_lossy().into_owned()),
        git_common_dir: Some(discovered.git_common_dir.to_string_lossy().into_owned()),
        display_name: discovered.display_name.clone(),
        host: primary_remote.and_then(|remote| remote.host.clone()),
        owner: primary_remote.and_then(|remote| remote.owner.clone()),
        repo_name: primary_remote.and_then(|remote| remote.repo_name.clone()),
        remote_name: primary_remote.map(|remote| remote.name.clone()),
        remote_url: primary_remote.map(|remote| remote.url.clone()),
        current_branch: discovered.current_branch.clone(),
        assigned_account_id: existing.as_ref().and_then(|repo| repo.assigned_account_id),
        commit_name: discovered.commit_name.clone(),
        commit_email: discovered.commit_email.clone(),
        push_policy: existing
            .as_ref()
            .map(|repo| repo.push_policy.clone())
            .unwrap_or_else(|| "allow_normal_push".to_string()),
        created_at: existing
            .as_ref()
            .map(|repo| repo.created_at.clone())
            .unwrap_or_else(|| now.clone()),
        updated_at: now.clone(),
        last_seen_at: Some(now),
    };

    queries::upsert_repository(db, &record)?;
    Ok(queries::find_repository_by_path(db, &canonical_path)?
        .expect("repository must exist immediately after upsert"))
}

fn primary_remote(discovered: &DiscoveredRepository) -> Option<&RepositoryRemote> {
    discovered
        .primary_remote_name
        .as_deref()
        .and_then(|name| discovered.remotes.iter().find(|remote| remote.name == name))
}

pub fn list_repository_summaries(db: &Database) -> Result<Vec<RepositorySummary>> {
    queries::list_repositories(db)?
        .into_iter()
        .map(|repo| repository_summary(db, repo))
        .collect()
}

pub fn repository_summary(db: &Database, repo: RepositoryRecord) -> Result<RepositorySummary> {
    let assigned_login = repo
        .assigned_account_id
        .and_then(|id| queries::find_account_by_id(db, id).ok().flatten())
        .map(|account| account.login);
    let remote_protocol = repo
        .remote_url
        .as_deref()
        .and_then(|url| parse_remote_url(url).ok())
        .map(|remote| match remote.protocol {
            RemoteProtocol::Https => "https".to_string(),
            RemoteProtocol::Ssh => "ssh".to_string(),
        });

    Ok(RepositorySummary {
        id: repo.id,
        display_name: repo.display_name,
        canonical_path: repo.canonical_path,
        host: repo.host,
        owner: repo.owner,
        repo_name: repo.repo_name,
        remote_name: repo.remote_name,
        remote_url: repo.remote_url,
        remote_protocol,
        current_branch: repo.current_branch,
        assigned_login,
        commit_name: repo.commit_name,
        commit_email: repo.commit_email,
        push_policy: repo.push_policy,
    })
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use shehata_git::{RepositoryRemoteProtocol, WorktreeStatus};
    use shehata_storage::queries;

    use super::*;

    fn discovery(path: PathBuf) -> DiscoveredRepository {
        DiscoveredRepository {
            canonical_path: path.clone(),
            git_dir: path.join(".git"),
            git_common_dir: path.join(".git"),
            display_name: "example".to_string(),
            current_branch: Some("main".to_string()),
            detached_head: false,
            head_commit: None,
            upstream: None,
            remotes: vec![RepositoryRemote {
                name: "origin".to_string(),
                url: "https://github.com/acme/example.git".to_string(),
                host: Some("github.com".to_string()),
                owner: Some("acme".to_string()),
                repo_name: Some("example".to_string()),
                protocol: RepositoryRemoteProtocol::Https,
            }],
            primary_remote_name: Some("origin".to_string()),
            status: WorktreeStatus {
                changed_files: 0,
                conflicts: 0,
                untracked_files: 0,
                ahead: 0,
                behind: 0,
            },
            commit_name: Some("Test User".to_string()),
            commit_email: Some("test@example.com".to_string()),
            credential_helpers: Vec::new(),
            credential_use_http_path: None,
        }
    }

    #[test]
    fn saves_remote_metadata_and_reuses_stable_id() {
        let db = Database::open_in_memory().unwrap();
        let dir = tempfile::tempdir().unwrap();
        let first = save_discovered_repository(&db, &discovery(dir.path().to_path_buf())).unwrap();
        assert_eq!(first.host.as_deref(), Some("github.com"));
        assert_eq!(first.owner.as_deref(), Some("acme"));

        let mut refreshed = discovery(dir.path().to_path_buf());
        refreshed.current_branch = Some("feature/test".to_string());
        let second = save_discovered_repository(&db, &refreshed).unwrap();
        assert_eq!(second.id, first.id);
        assert_eq!(second.current_branch.as_deref(), Some("feature/test"));
        assert_eq!(queries::list_repositories(&db).unwrap().len(), 1);
    }
}
