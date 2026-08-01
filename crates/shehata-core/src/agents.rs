//! Safe generation of the bounded Shehata Git section in repository AGENTS.md.

use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use shehata_storage::{queries, Database, NewAuditEvent};

use crate::{Result, ShehataError};

const START: &str = "<!-- SHEHATA-GIT:START -->";
const END: &str = "<!-- SHEHATA-GIT:END -->";
const MAX_AGENTS_BYTES: u64 = 1024 * 1024;
const BLOCK: &str = r#"<!-- SHEHATA-GIT:START -->
## Git operations

Use the Shehata Git MCP tools for repository status, commits, pull, and push.

Before pushing:
1. Verify the assigned account.
2. Verify the remote and branch.
3. Never force push.
4. Do not change Git credentials.
5. Do not change the remote unless the user explicitly requests it.
<!-- SHEHATA-GIT:END -->"#;

#[derive(Debug, Clone, Deserialize)]
pub struct GenerateAgentsRequest {
    pub repository_id: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct GenerateAgentsResult {
    pub repository_id: String,
    pub path: String,
    pub created: bool,
}

pub fn generate_agents(request: GenerateAgentsRequest) -> Result<GenerateAgentsResult> {
    let db_path = Database::default_path()?;
    generate_agents_at(&db_path, request)
}

fn generate_agents_at(
    db_path: &Path,
    request: GenerateAgentsRequest,
) -> Result<GenerateAgentsResult> {
    let repository_id = request.repository_id.trim();
    uuid::Uuid::parse_str(repository_id)
        .map_err(|_| ShehataError::InvalidInput("invalid repository id".to_string()))?;
    let repository = {
        let db = Database::open_at(db_path)?;
        queries::find_repository_by_id(&db, repository_id)?
            .ok_or_else(|| ShehataError::RepositoryNotFound(repository_id.to_string()))?
    };
    let root = PathBuf::from(&repository.canonical_path);
    let target = root.join("AGENTS.md");
    let created = !target.exists();
    let existing = if created {
        String::new()
    } else {
        let metadata =
            fs::metadata(&target).map_err(|error| ShehataError::Internal(error.to_string()))?;
        if !metadata.is_file() || metadata.len() > MAX_AGENTS_BYTES {
            return Err(ShehataError::OperationBlocked(
                "existing AGENTS.md is not a regular text file under 1 MB".to_string(),
            ));
        }
        fs::read_to_string(&target).map_err(|_| {
            ShehataError::OperationBlocked("existing AGENTS.md is not valid UTF-8".to_string())
        })?
    };
    let updated = merge_block(&existing)?;
    replace_recoverably(&target, updated.as_bytes())?;

    let db = Database::open_at(db_path)?;
    queries::insert_audit_event(
        &db,
        &NewAuditEvent {
            event_type: "agents_instructions_generated",
            repository_id: Some(&repository.id),
            account_login: None,
            summary: "Generated the bounded Shehata Git AGENTS.md section",
            result: "success",
            exit_code: Some(0),
            duration_ms: None,
        },
    )?;
    Ok(GenerateAgentsResult {
        repository_id: repository.id,
        path: target.to_string_lossy().into_owned(),
        created,
    })
}

fn merge_block(existing: &str) -> Result<String> {
    let starts = existing.match_indices(START).collect::<Vec<_>>();
    let ends = existing.match_indices(END).collect::<Vec<_>>();
    match (starts.as_slice(), ends.as_slice()) {
        ([], []) => {
            let trimmed = existing.trim_end();
            if trimmed.is_empty() {
                Ok(format!("{BLOCK}\n"))
            } else {
                Ok(format!("{trimmed}\n\n{BLOCK}\n"))
            }
        }
        ([(start, _)], [(end, _)]) if start < end => {
            let after = end + END.len();
            Ok(format!(
                "{}{}{}",
                &existing[..*start],
                BLOCK,
                &existing[after..]
            ))
        }
        _ => Err(ShehataError::OperationBlocked(
            "AGENTS.md has malformed or duplicate Shehata Git markers".to_string(),
        )),
    }
}

fn replace_recoverably(target: &Path, bytes: &[u8]) -> Result<()> {
    let temporary = target.with_file_name(".AGENTS.md.shehata-new");
    let backup = target.with_file_name(".AGENTS.md.shehata-backup");
    if temporary.exists() || backup.exists() {
        return Err(ShehataError::OperationBlocked(
            "a previous AGENTS.md update needs manual recovery".to_string(),
        ));
    }
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temporary)
        .map_err(|error| ShehataError::Internal(error.to_string()))?;
    file.write_all(bytes)
        .and_then(|_| file.sync_all())
        .map_err(|error| ShehataError::Internal(error.to_string()))?;
    drop(file);

    if target.exists() {
        if let Err(error) = fs::rename(target, &backup) {
            let _ = fs::remove_file(&temporary);
            return Err(ShehataError::Internal(error.to_string()));
        }
    }
    if let Err(error) = fs::rename(&temporary, target) {
        if backup.exists() {
            let _ = fs::rename(&backup, target);
        }
        let _ = fs::remove_file(&temporary);
        return Err(ShehataError::Internal(error.to_string()));
    }
    if backup.exists() {
        fs::remove_file(backup).map_err(|error| ShehataError::Internal(error.to_string()))?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use chrono::Utc;
    use shehata_storage::RepositoryRecord;

    use super::*;

    #[test]
    fn appends_updates_and_rejects_bad_markers() {
        let original = "# Existing rules\n\nKeep this.\n";
        let first = merge_block(original).unwrap();
        assert!(first.starts_with(original.trim_end()));
        assert!(first.contains(START));
        assert!(first.contains("Never force push."));
        assert_eq!(merge_block(&first).unwrap(), first);
        assert!(merge_block(&format!("{START}\nmissing end")).is_err());
        assert!(merge_block(&format!("{START}\n{END}\n{START}\n{END}")).is_err());
    }

    #[test]
    fn generates_recoverably_without_overwriting_existing_rules() {
        let temp = tempfile::tempdir().unwrap();
        let repository_id = uuid::Uuid::new_v4().to_string();
        let repository_path = temp.path().join("repo");
        fs::create_dir(&repository_path).unwrap();
        fs::write(
            repository_path.join("AGENTS.md"),
            "# Owner rules\n\nKeep this.\n",
        )
        .unwrap();
        let db_path = temp.path().join("db.sqlite");
        let db = Database::open_at(&db_path).unwrap();
        let now = Utc::now().to_rfc3339();
        queries::insert_repository(
            &db,
            &RepositoryRecord {
                id: repository_id.clone(),
                canonical_path: repository_path.to_string_lossy().into_owned(),
                git_dir: None,
                git_common_dir: None,
                display_name: "repo".into(),
                host: None,
                owner: None,
                repo_name: None,
                remote_name: None,
                remote_url: None,
                current_branch: None,
                assigned_account_id: None,
                commit_name: None,
                commit_email: None,
                push_policy: "allow_normal_push".into(),
                created_at: now.clone(),
                updated_at: now.clone(),
                last_seen_at: Some(now),
            },
        )
        .unwrap();
        drop(db);

        let result = generate_agents_at(
            &db_path,
            GenerateAgentsRequest {
                repository_id: repository_id.clone(),
            },
        )
        .unwrap();
        assert!(!result.created);
        let content = fs::read_to_string(repository_path.join("AGENTS.md")).unwrap();
        assert!(content.starts_with("# Owner rules\n\nKeep this."));
        assert_eq!(content.matches(START).count(), 1);
        assert_eq!(content.matches(END).count(), 1);
        assert!(!repository_path.join(".AGENTS.md.shehata-new").exists());
        assert!(!repository_path.join(".AGENTS.md.shehata-backup").exists());
    }
}
