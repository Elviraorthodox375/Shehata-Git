//! CRUD queries. Everything here is plain SQL with bound parameters.

use rusqlite::{params, OptionalExtension};

use crate::db::{Database, StorageError};
use crate::records::{AccountRecord, AuditEventRecord, ConfigBackupRecord, RepositoryRecord};

fn now() -> String {
    chrono::Utc::now().to_rfc3339()
}

// ---------------------------------------------------------------- accounts

pub fn upsert_account(
    db: &Database,
    host: &str,
    login: &str,
    status: &str,
) -> Result<i64, StorageError> {
    let ts = now();
    db.connection().execute(
        "INSERT INTO accounts (host, login, status, created_at, updated_at, last_validated_at)
         VALUES (?1, ?2, ?3, ?4, ?4, ?4)
         ON CONFLICT (host, login) DO UPDATE SET
            status = excluded.status,
            updated_at = excluded.updated_at,
            last_validated_at = excluded.last_validated_at",
        params![host, login, status, ts],
    )?;
    let id: i64 = db.connection().query_row(
        "SELECT id FROM accounts WHERE host = ?1 AND login = ?2",
        params![host, login],
        |row| row.get(0),
    )?;
    Ok(id)
}

pub fn list_accounts(db: &Database) -> Result<Vec<AccountRecord>, StorageError> {
    let mut stmt = db.connection().prepare(
        "SELECT id, host, login, display_name, avatar_url, auth_source, status,
                created_at, updated_at, last_validated_at
         FROM accounts ORDER BY host, login",
    )?;
    let rows = stmt
        .query_map([], |row| {
            Ok(AccountRecord {
                id: row.get(0)?,
                host: row.get(1)?,
                login: row.get(2)?,
                display_name: row.get(3)?,
                avatar_url: row.get(4)?,
                auth_source: row.get(5)?,
                status: row.get(6)?,
                created_at: row.get(7)?,
                updated_at: row.get(8)?,
                last_validated_at: row.get(9)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

pub fn find_account(
    db: &Database,
    host: &str,
    login: &str,
) -> Result<Option<AccountRecord>, StorageError> {
    let record = db
        .connection()
        .query_row(
            "SELECT id, host, login, display_name, avatar_url, auth_source, status,
                    created_at, updated_at, last_validated_at
             FROM accounts WHERE host = ?1 AND login = ?2",
            params![host, login],
            |row| {
                Ok(AccountRecord {
                    id: row.get(0)?,
                    host: row.get(1)?,
                    login: row.get(2)?,
                    display_name: row.get(3)?,
                    avatar_url: row.get(4)?,
                    auth_source: row.get(5)?,
                    status: row.get(6)?,
                    created_at: row.get(7)?,
                    updated_at: row.get(8)?,
                    last_validated_at: row.get(9)?,
                })
            },
        )
        .optional()?;
    Ok(record)
}

pub fn find_account_by_id(db: &Database, id: i64) -> Result<Option<AccountRecord>, StorageError> {
    let record = db
        .connection()
        .query_row(
            "SELECT id, host, login, display_name, avatar_url, auth_source, status,
                    created_at, updated_at, last_validated_at
             FROM accounts WHERE id = ?1",
            params![id],
            |row| {
                Ok(AccountRecord {
                    id: row.get(0)?,
                    host: row.get(1)?,
                    login: row.get(2)?,
                    display_name: row.get(3)?,
                    avatar_url: row.get(4)?,
                    auth_source: row.get(5)?,
                    status: row.get(6)?,
                    created_at: row.get(7)?,
                    updated_at: row.get(8)?,
                    last_validated_at: row.get(9)?,
                })
            },
        )
        .optional()?;
    Ok(record)
}

// ------------------------------------------------------------ repositories

pub fn insert_repository(db: &Database, repo: &RepositoryRecord) -> Result<(), StorageError> {
    db.connection().execute(
        "INSERT INTO repositories (
            id, canonical_path, git_dir, git_common_dir, display_name, host, owner,
            repo_name, remote_name, remote_url, current_branch, assigned_account_id,
            commit_name, commit_email, push_policy, created_at, updated_at, last_seen_at
         ) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16,?17,?18)",
        params![
            repo.id,
            repo.canonical_path,
            repo.git_dir,
            repo.git_common_dir,
            repo.display_name,
            repo.host,
            repo.owner,
            repo.repo_name,
            repo.remote_name,
            repo.remote_url,
            repo.current_branch,
            repo.assigned_account_id,
            repo.commit_name,
            repo.commit_email,
            repo.push_policy,
            repo.created_at,
            repo.updated_at,
            repo.last_seen_at,
        ],
    )?;
    Ok(())
}

/// Insert a newly discovered repository or refresh its non-routing metadata.
/// Existing account assignment, push policy, stable id, and creation time are
/// intentionally preserved on a canonical-path conflict.
pub fn upsert_repository(db: &Database, repo: &RepositoryRecord) -> Result<(), StorageError> {
    db.connection().execute(
        "INSERT INTO repositories (
            id, canonical_path, git_dir, git_common_dir, display_name, host, owner,
            repo_name, remote_name, remote_url, current_branch, assigned_account_id,
            commit_name, commit_email, push_policy, created_at, updated_at, last_seen_at
         ) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16,?17,?18)
         ON CONFLICT (canonical_path) DO UPDATE SET
            git_dir = excluded.git_dir,
            git_common_dir = excluded.git_common_dir,
            display_name = excluded.display_name,
            host = excluded.host,
            owner = excluded.owner,
            repo_name = excluded.repo_name,
            remote_name = excluded.remote_name,
            remote_url = excluded.remote_url,
            current_branch = excluded.current_branch,
            commit_name = excluded.commit_name,
            commit_email = excluded.commit_email,
            updated_at = excluded.updated_at,
            last_seen_at = excluded.last_seen_at",
        params![
            repo.id,
            repo.canonical_path,
            repo.git_dir,
            repo.git_common_dir,
            repo.display_name,
            repo.host,
            repo.owner,
            repo.repo_name,
            repo.remote_name,
            repo.remote_url,
            repo.current_branch,
            repo.assigned_account_id,
            repo.commit_name,
            repo.commit_email,
            repo.push_policy,
            repo.created_at,
            repo.updated_at,
            repo.last_seen_at,
        ],
    )?;
    Ok(())
}

const REPO_COLUMNS: &str = "id, canonical_path, git_dir, git_common_dir, display_name,
     host, owner, repo_name, remote_name, remote_url, current_branch,
     assigned_account_id, commit_name, commit_email, push_policy,
     created_at, updated_at, last_seen_at";

fn map_repo(row: &rusqlite::Row) -> rusqlite::Result<RepositoryRecord> {
    Ok(RepositoryRecord {
        id: row.get(0)?,
        canonical_path: row.get(1)?,
        git_dir: row.get(2)?,
        git_common_dir: row.get(3)?,
        display_name: row.get(4)?,
        host: row.get(5)?,
        owner: row.get(6)?,
        repo_name: row.get(7)?,
        remote_name: row.get(8)?,
        remote_url: row.get(9)?,
        current_branch: row.get(10)?,
        assigned_account_id: row.get(11)?,
        commit_name: row.get(12)?,
        commit_email: row.get(13)?,
        push_policy: row.get(14)?,
        created_at: row.get(15)?,
        updated_at: row.get(16)?,
        last_seen_at: row.get(17)?,
    })
}

pub fn list_repositories(db: &Database) -> Result<Vec<RepositoryRecord>, StorageError> {
    let mut stmt = db.connection().prepare(&format!(
        "SELECT {REPO_COLUMNS} FROM repositories ORDER BY display_name"
    ))?;
    let rows = stmt
        .query_map([], map_repo)?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

pub fn find_repository_by_id(
    db: &Database,
    id: &str,
) -> Result<Option<RepositoryRecord>, StorageError> {
    let record = db
        .connection()
        .query_row(
            &format!("SELECT {REPO_COLUMNS} FROM repositories WHERE id = ?1"),
            params![id],
            map_repo,
        )
        .optional()?;
    Ok(record)
}

pub fn find_repository_by_path(
    db: &Database,
    canonical_path: &str,
) -> Result<Option<RepositoryRecord>, StorageError> {
    let record = db
        .connection()
        .query_row(
            &format!("SELECT {REPO_COLUMNS} FROM repositories WHERE canonical_path = ?1"),
            params![canonical_path],
            map_repo,
        )
        .optional()?;
    Ok(record)
}

pub fn assign_account(
    db: &Database,
    repository_id: &str,
    account_id: i64,
) -> Result<(), StorageError> {
    db.connection().execute(
        "UPDATE repositories SET assigned_account_id = ?1, updated_at = ?2 WHERE id = ?3",
        params![account_id, now(), repository_id],
    )?;
    Ok(())
}

pub fn update_repository_assignment_and_identity(
    db: &Database,
    repository_id: &str,
    account_id: i64,
    commit_name: Option<&str>,
    commit_email: Option<&str>,
) -> Result<(), StorageError> {
    db.connection().execute(
        "UPDATE repositories SET
            assigned_account_id = ?1,
            commit_name = ?2,
            commit_email = ?3,
            updated_at = ?4,
            last_seen_at = ?4
         WHERE id = ?5",
        params![account_id, commit_name, commit_email, now(), repository_id],
    )?;
    Ok(())
}

pub fn clear_repository_assignment(db: &Database, repository_id: &str) -> Result<(), StorageError> {
    db.connection().execute(
        "UPDATE repositories SET assigned_account_id = NULL, updated_at = ?1 WHERE id = ?2",
        params![now(), repository_id],
    )?;
    Ok(())
}

pub fn update_repository_push_policy(
    db: &Database,
    repository_id: &str,
    push_policy: &str,
) -> Result<(), StorageError> {
    db.connection().execute(
        "UPDATE repositories SET push_policy = ?1, updated_at = ?2 WHERE id = ?3",
        params![push_policy, now(), repository_id],
    )?;
    Ok(())
}

pub fn delete_repository(db: &Database, id: &str) -> Result<(), StorageError> {
    db.connection()
        .execute("DELETE FROM repositories WHERE id = ?1", params![id])?;
    Ok(())
}

// ----------------------------------------------------------------- backups

pub fn insert_config_backup(
    db: &Database,
    repository_id: &str,
    config_key: &str,
    previous_values_json: &str,
) -> Result<(), StorageError> {
    db.connection().execute(
        "INSERT INTO repository_config_backups
            (repository_id, config_key, previous_values_json, created_at)
         VALUES (?1, ?2, ?3, ?4)",
        params![repository_id, config_key, previous_values_json, now()],
    )?;
    Ok(())
}

pub fn pending_backups(
    db: &Database,
    repository_id: &str,
) -> Result<Vec<ConfigBackupRecord>, StorageError> {
    let mut stmt = db.connection().prepare(
        "SELECT id, repository_id, config_key, previous_values_json, created_at, restored_at
         FROM repository_config_backups
         WHERE repository_id = ?1 AND restored_at IS NULL
         ORDER BY id",
    )?;
    let rows = stmt
        .query_map(params![repository_id], |row| {
            Ok(ConfigBackupRecord {
                id: row.get(0)?,
                repository_id: row.get(1)?,
                config_key: row.get(2)?,
                previous_values_json: row.get(3)?,
                created_at: row.get(4)?,
                restored_at: row.get(5)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

pub fn mark_backup_restored(db: &Database, backup_id: i64) -> Result<(), StorageError> {
    db.connection().execute(
        "UPDATE repository_config_backups SET restored_at = ?1 WHERE id = ?2",
        params![now(), backup_id],
    )?;
    Ok(())
}

// ------------------------------------------------------------------- audit

pub fn insert_audit_event(
    db: &Database,
    event: &crate::records::NewAuditEvent<'_>,
) -> Result<(), StorageError> {
    db.connection().execute(
        "INSERT INTO audit_events
            (timestamp, repository_id, event_type, account_login, summary, detail, result, exit_code, duration_ms)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        params![
            now(),
            event.repository_id,
            event.event_type,
            event.account_login,
            event.summary,
            event.detail,
            event.result,
            event.exit_code,
            event.duration_ms
        ],
    )?;
    Ok(())
}

pub fn list_audit_events(db: &Database, limit: i64) -> Result<Vec<AuditEventRecord>, StorageError> {
    let mut stmt = db.connection().prepare(
        "SELECT id, timestamp, repository_id, event_type, account_login, summary, detail, result, exit_code, duration_ms
         FROM audit_events ORDER BY timestamp DESC, id DESC LIMIT ?1",
    )?;
    let rows = stmt
        .query_map(params![limit], |row| {
            Ok(AuditEventRecord {
                id: row.get(0)?,
                timestamp: row.get(1)?,
                repository_id: row.get(2)?,
                event_type: row.get(3)?,
                account_login: row.get(4)?,
                summary: row.get(5)?,
                detail: row.get(6)?,
                result: row.get(7)?,
                exit_code: row.get(8)?,
                duration_ms: row.get(9)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

pub fn delete_audit_event(db: &Database, id: i64) -> Result<usize, StorageError> {
    Ok(db
        .connection()
        .execute("DELETE FROM audit_events WHERE id = ?1", params![id])?)
}

pub fn clear_audit_events(db: &Database) -> Result<usize, StorageError> {
    Ok(db.connection().execute("DELETE FROM audit_events", [])?)
}

// ---------------------------------------------------------------- settings

pub fn get_setting(db: &Database, key: &str) -> Result<Option<String>, StorageError> {
    let value = db
        .connection()
        .query_row(
            "SELECT value FROM settings WHERE key = ?1",
            params![key],
            |row| row.get::<_, String>(0),
        )
        .optional()?;
    Ok(value)
}

pub fn set_setting(db: &Database, key: &str, value: &str) -> Result<(), StorageError> {
    db.connection().execute(
        "INSERT INTO settings (key, value) VALUES (?1, ?2)
         ON CONFLICT (key) DO UPDATE SET value = excluded.value",
        params![key, value],
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_repo(id: &str) -> RepositoryRecord {
        let ts = now();
        RepositoryRecord {
            id: id.to_string(),
            canonical_path: "D:/code/example".to_string(),
            git_dir: Some("D:/code/example/.git".to_string()),
            git_common_dir: None,
            display_name: "example".to_string(),
            host: Some("github.com".to_string()),
            owner: Some("owner".to_string()),
            repo_name: Some("example".to_string()),
            remote_name: Some("origin".to_string()),
            remote_url: Some("https://github.com/owner/example.git".to_string()),
            current_branch: Some("main".to_string()),
            assigned_account_id: None,
            commit_name: None,
            commit_email: None,
            push_policy: "allow_normal_push".to_string(),
            created_at: ts.clone(),
            updated_at: ts,
            last_seen_at: None,
        }
    }

    #[test]
    fn account_roundtrip() {
        let db = Database::open_in_memory().unwrap();
        let id = upsert_account(&db, "github.com", "octocat", "valid").unwrap();
        let account = find_account(&db, "github.com", "octocat").unwrap().unwrap();
        assert_eq!(account.id, id);
        assert_eq!(account.status, "valid");

        // Upsert updates instead of duplicating.
        upsert_account(&db, "github.com", "octocat", "stale").unwrap();
        assert_eq!(list_accounts(&db).unwrap().len(), 1);
        assert_eq!(
            find_account(&db, "github.com", "octocat")
                .unwrap()
                .unwrap()
                .status,
            "stale"
        );
    }

    #[test]
    fn repository_roundtrip_and_assignment() {
        let db = Database::open_in_memory().unwrap();
        let account_id = upsert_account(&db, "github.com", "octocat", "valid").unwrap();
        insert_repository(&db, &sample_repo("repo-1")).unwrap();

        let repo = find_repository_by_id(&db, "repo-1").unwrap().unwrap();
        assert_eq!(repo.display_name, "example");
        assert!(repo.assigned_account_id.is_none());

        assign_account(&db, "repo-1", account_id).unwrap();
        let repo = find_repository_by_path(&db, "D:/code/example")
            .unwrap()
            .unwrap();
        assert_eq!(repo.assigned_account_id, Some(account_id));

        delete_repository(&db, "repo-1").unwrap();
        assert!(find_repository_by_id(&db, "repo-1").unwrap().is_none());
    }

    #[test]
    fn repository_upsert_preserves_routing_fields() {
        let db = Database::open_in_memory().unwrap();
        let account_id = upsert_account(&db, "github.com", "octocat", "valid").unwrap();
        let mut repo = sample_repo("repo-1");
        repo.assigned_account_id = Some(account_id);
        repo.push_policy = "ask_before_push".to_string();
        insert_repository(&db, &repo).unwrap();

        let mut refreshed = sample_repo("different-id");
        refreshed.current_branch = Some("feature/refreshed".to_string());
        upsert_repository(&db, &refreshed).unwrap();

        let stored = find_repository_by_path(&db, "D:/code/example")
            .unwrap()
            .unwrap();
        assert_eq!(stored.id, "repo-1");
        assert_eq!(stored.assigned_account_id, Some(account_id));
        assert_eq!(stored.push_policy, "ask_before_push");
        assert_eq!(stored.current_branch.as_deref(), Some("feature/refreshed"));
    }

    #[test]
    fn backup_lifecycle() {
        let db = Database::open_in_memory().unwrap();
        insert_repository(&db, &sample_repo("repo-1")).unwrap();
        insert_config_backup(&db, "repo-1", "credential.helper", "[\"manager\"]").unwrap();

        let backups = pending_backups(&db, "repo-1").unwrap();
        assert_eq!(backups.len(), 1);
        assert_eq!(backups[0].config_key, "credential.helper");

        mark_backup_restored(&db, backups[0].id).unwrap();
        assert!(pending_backups(&db, "repo-1").unwrap().is_empty());
    }

    #[test]
    fn audit_roundtrip() {
        let db = Database::open_in_memory().unwrap();
        insert_audit_event(
            &db,
            &crate::records::NewAuditEvent {
                event_type: "connection_test",
                repository_id: Some("repo-1"),
                account_login: Some("octocat"),
                summary: "Tested connection for example",
                detail: None,
                result: "success",
                exit_code: Some(0),
                duration_ms: Some(812),
            },
        )
        .unwrap();
        let events = list_audit_events(&db, 10).unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].summary, "Tested connection for example");
        assert_eq!(events[0].exit_code, Some(0));

        let id = events[0].id;
        assert_eq!(delete_audit_event(&db, id).unwrap(), 1);
        assert!(list_audit_events(&db, 10).unwrap().is_empty());
    }

    #[test]
    fn clear_audit_history() {
        let db = Database::open_in_memory().unwrap();
        for summary in ["First", "Second"] {
            insert_audit_event(
                &db,
                &crate::records::NewAuditEvent {
                    event_type: "test",
                    repository_id: None,
                    account_login: None,
                    summary,
                    detail: None,
                    result: "success",
                    exit_code: Some(0),
                    duration_ms: None,
                },
            )
            .unwrap();
        }
        assert_eq!(clear_audit_events(&db).unwrap(), 2);
        assert!(list_audit_events(&db, 10).unwrap().is_empty());
    }

    #[test]
    fn settings_roundtrip() {
        let db = Database::open_in_memory().unwrap();
        assert_eq!(get_setting(&db, "theme").unwrap(), None);
        set_setting(&db, "theme", "dark").unwrap();
        set_setting(&db, "theme", "light").unwrap();
        assert_eq!(get_setting(&db, "theme").unwrap().as_deref(), Some("light"));
    }
}
