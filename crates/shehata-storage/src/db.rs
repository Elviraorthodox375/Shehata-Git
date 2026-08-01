//! Database open + embedded versioned migrations.

use std::path::{Path, PathBuf};

use rusqlite::Connection;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum StorageError {
    #[error("could not determine application data directory")]
    NoAppDataDir,
    #[error("could not create application data directory: {0}")]
    CreateDir(String),
    #[error("database error: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("migration {version} failed: {message}")]
    Migration { version: i64, message: String },
}

/// Embedded migrations, applied in order. Never edit an applied migration —
/// add a new one.
const MIGRATIONS: &[(i64, &str)] = &[(
    1,
    r#"
        CREATE TABLE accounts (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            host TEXT NOT NULL,
            login TEXT NOT NULL,
            display_name TEXT,
            avatar_url TEXT,
            auth_source TEXT NOT NULL DEFAULT 'gh-cli',
            status TEXT NOT NULL DEFAULT 'unknown',
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            last_validated_at TEXT,
            UNIQUE (host, login)
        );

        CREATE TABLE repositories (
            id TEXT PRIMARY KEY,
            canonical_path TEXT NOT NULL UNIQUE,
            git_dir TEXT,
            git_common_dir TEXT,
            display_name TEXT NOT NULL,
            host TEXT,
            owner TEXT,
            repo_name TEXT,
            remote_name TEXT,
            remote_url TEXT,
            current_branch TEXT,
            assigned_account_id INTEGER REFERENCES accounts (id),
            commit_name TEXT,
            commit_email TEXT,
            push_policy TEXT NOT NULL DEFAULT 'allow_normal_push',
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            last_seen_at TEXT
        );

        CREATE TABLE repository_config_backups (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            repository_id TEXT NOT NULL REFERENCES repositories (id),
            config_key TEXT NOT NULL,
            previous_values_json TEXT NOT NULL,
            created_at TEXT NOT NULL,
            restored_at TEXT
        );

        CREATE TABLE audit_events (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            timestamp TEXT NOT NULL,
            repository_id TEXT,
            event_type TEXT NOT NULL,
            account_login TEXT,
            summary TEXT NOT NULL,
            result TEXT NOT NULL,
            exit_code INTEGER,
            duration_ms INTEGER
        );

        CREATE TABLE settings (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL
        );

        CREATE INDEX idx_audit_events_timestamp ON audit_events (timestamp DESC);
        CREATE INDEX idx_backups_repo ON repository_config_backups (repository_id);
        "#,
)];

pub struct Database {
    conn: Connection,
    path: PathBuf,
}

impl Database {
    /// Default database location in the OS app-data directory.
    pub fn default_path() -> Result<PathBuf, StorageError> {
        let dirs = directories::ProjectDirs::from("dev", "Shehata", "shehata-git")
            .ok_or(StorageError::NoAppDataDir)?;
        Ok(dirs.data_dir().join("shehata.db"))
    }

    /// Open (creating if needed) the database at the default location.
    pub fn open_default() -> Result<Self, StorageError> {
        let path = Self::default_path()?;
        Self::open_at(&path)
    }

    /// Open (creating if needed) a database at an explicit path.
    pub fn open_at(path: &Path) -> Result<Self, StorageError> {
        if let Some(parent) = path
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
        {
            std::fs::create_dir_all(parent).map_err(|e| StorageError::CreateDir(e.to_string()))?;
        }
        let conn = Connection::open(path)?;
        conn.pragma_update(None, "journal_mode", "WAL")?;
        conn.pragma_update(None, "foreign_keys", "ON")?;
        let db = Self {
            conn,
            path: path.to_path_buf(),
        };
        db.migrate()?;
        Ok(db)
    }

    /// Open an existing database read-only (used by the credential helper,
    /// which must never write and must not create files).
    pub fn open_read_only(path: &Path) -> Result<Self, StorageError> {
        let conn = Connection::open_with_flags(
            path,
            rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY | rusqlite::OpenFlags::SQLITE_OPEN_NO_MUTEX,
        )?;
        conn.busy_timeout(std::time::Duration::from_secs(5))?;
        let db = Self {
            conn,
            path: path.to_path_buf(),
        };
        Ok(db)
    }

    /// In-memory database for tests.
    pub fn open_in_memory() -> Result<Self, StorageError> {
        let conn = Connection::open_in_memory()?;
        conn.pragma_update(None, "foreign_keys", "ON")?;
        let db = Self {
            conn,
            path: PathBuf::from(":memory:"),
        };
        db.migrate()?;
        Ok(db)
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn connection(&self) -> &Connection {
        &self.conn
    }

    pub fn schema_version(&self) -> Result<i64, StorageError> {
        let version: i64 = self
            .conn
            .query_row("PRAGMA user_version", [], |row| row.get(0))?;
        Ok(version)
    }

    fn migrate(&self) -> Result<(), StorageError> {
        let current = self.schema_version()?;
        for (version, sql) in MIGRATIONS {
            if *version > current {
                self.conn
                    .execute_batch(sql)
                    .map_err(|e| StorageError::Migration {
                        version: *version,
                        message: e.to_string(),
                    })?;
                self.conn.pragma_update(None, "user_version", *version)?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn migrates_to_latest_version() {
        let db = Database::open_in_memory().unwrap();
        let latest = MIGRATIONS.last().unwrap().0;
        assert_eq!(db.schema_version().unwrap(), latest);
    }

    #[test]
    fn migration_is_idempotent() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.db");
        let version_first = Database::open_at(&path).unwrap().schema_version().unwrap();
        let version_second = Database::open_at(&path).unwrap().schema_version().unwrap();
        assert_eq!(version_first, version_second);
    }

    #[test]
    fn open_at_creates_missing_parent_directories() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("nested").join("data").join("test.db");
        let db = Database::open_at(&path).unwrap();
        assert_eq!(db.path(), path);
        assert!(path.exists());
    }

    #[test]
    fn all_expected_tables_exist() {
        let db = Database::open_in_memory().unwrap();
        for table in [
            "accounts",
            "repositories",
            "repository_config_backups",
            "audit_events",
            "settings",
        ] {
            let count: i64 = db
                .connection()
                .query_row(
                    "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name=?1",
                    [table],
                    |row| row.get(0),
                )
                .unwrap();
            assert_eq!(count, 1, "missing table: {table}");
        }
    }

    /// Guard test: the schema must never gain a column that could hold a
    /// credential. If this test fails, review the migration that added it.
    #[test]
    fn schema_contains_no_secret_columns() {
        let db = Database::open_in_memory().unwrap();
        let mut stmt = db
            .connection()
            .prepare("SELECT name FROM sqlite_master WHERE type='table'")
            .unwrap();
        let tables: Vec<String> = stmt
            .query_map([], |row| row.get(0))
            .unwrap()
            .collect::<Result<_, _>>()
            .unwrap();
        for table in tables {
            let mut col_stmt = db
                .connection()
                .prepare(&format!("PRAGMA table_info({table})"))
                .unwrap();
            let columns: Vec<String> = col_stmt
                .query_map([], |row| row.get::<_, String>(1))
                .unwrap()
                .collect::<Result<_, _>>()
                .unwrap();
            for column in columns {
                let lowered = column.to_lowercase();
                for forbidden in ["token", "password", "secret", "credential", "auth_header"] {
                    assert!(
                        !lowered.contains(forbidden),
                        "column {table}.{column} looks like it could store a secret"
                    );
                }
            }
        }
    }
}
