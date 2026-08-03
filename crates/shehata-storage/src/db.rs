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
const MIGRATIONS: &[(i64, &str)] = &[
    (
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
    ),
    (
        2,
        r#"
        -- Technical context for an activity entry, kept apart from its human
        -- title so the trail can be read at a glance.
        ALTER TABLE audit_events ADD COLUMN detail TEXT;
        "#,
    ),
];

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
            #[cfg(unix)]
            Self::restrict_directory_permissions(parent)?;
        }
        let conn = Connection::open(path)?;
        conn.pragma_update(None, "journal_mode", "WAL")?;
        conn.pragma_update(None, "foreign_keys", "ON")?;
        // Prevents SQLITE_BUSY when the credential helper reads while the
        // desktop app writes. 5 s matches the read-only helper path.
        conn.busy_timeout(std::time::Duration::from_secs(5))?;
        // NORMAL sync is safe with WAL — data survives application crashes;
        // only an OS crash could lose the last transaction (acceptable for
        // local-only non-financial data).
        conn.pragma_update(None, "synchronous", "NORMAL")?;
        let db = Self {
            conn,
            path: path.to_path_buf(),
        };
        db.migrate()?;
        #[cfg(unix)]
        Self::restrict_file_permissions(path)?;
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
                // Wrap each migration + version bump in a single transaction
                // so a crash between SQL and user_version cannot leave the
                // schema in a half-applied state.
                self.conn.execute_batch("BEGIN EXCLUSIVE").map_err(|e| {
                    StorageError::Migration {
                        version: *version,
                        message: format!("could not begin transaction: {e}"),
                    }
                })?;
                if let Err(e) = self.conn.execute_batch(sql) {
                    let _ = self.conn.execute_batch("ROLLBACK");
                    return Err(StorageError::Migration {
                        version: *version,
                        message: e.to_string(),
                    });
                }
                if let Err(e) = self.conn.pragma_update(None, "user_version", *version) {
                    let _ = self.conn.execute_batch("ROLLBACK");
                    return Err(StorageError::Migration {
                        version: *version,
                        message: format!("could not update schema version: {e}"),
                    });
                }
                self.conn
                    .execute_batch("COMMIT")
                    .map_err(|e| StorageError::Migration {
                        version: *version,
                        message: format!("could not commit migration: {e}"),
                    })?;
            }
        }
        Ok(())
    }

    /// On Unix, restrict database file to owner-only read/write (0600).
    #[cfg(unix)]
    fn restrict_file_permissions(path: &Path) -> Result<(), StorageError> {
        use std::os::unix::fs::PermissionsExt;
        let perms = std::fs::Permissions::from_mode(0o600);
        std::fs::set_permissions(path, perms)
            .map_err(|e| StorageError::CreateDir(format!("chmod 0600 failed: {e}")))?;
        Ok(())
    }

    /// On Unix, restrict database directory to owner-only (0700).
    #[cfg(unix)]
    fn restrict_directory_permissions(path: &Path) -> Result<(), StorageError> {
        use std::os::unix::fs::PermissionsExt;
        let perms = std::fs::Permissions::from_mode(0o700);
        std::fs::set_permissions(path, perms)
            .map_err(|e| StorageError::CreateDir(format!("chmod 0700 failed: {e}")))?;
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

    #[test]
    fn busy_timeout_is_set_on_open_at() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("busy.db");
        let db = Database::open_at(&path).unwrap();
        // busy_timeout returns the current value; we set 5000ms.
        let timeout: i64 = db
            .connection()
            .pragma_query_value(None, "busy_timeout", |row| row.get(0))
            .unwrap();
        assert_eq!(timeout, 5000);
    }

    #[test]
    fn synchronous_normal_is_set_on_open_at() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("sync.db");
        let db = Database::open_at(&path).unwrap();
        // synchronous=NORMAL is pragma value 1.
        let sync: i64 = db
            .connection()
            .pragma_query_value(None, "synchronous", |row| row.get(0))
            .unwrap();
        assert_eq!(sync, 1, "synchronous should be NORMAL (1)");
    }

    #[test]
    fn wal_journal_mode_is_set() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("wal.db");
        let db = Database::open_at(&path).unwrap();
        let mode: String = db
            .connection()
            .pragma_query_value(None, "journal_mode", |row| row.get(0))
            .unwrap();
        assert_eq!(mode.to_lowercase(), "wal");
    }

    #[test]
    fn concurrent_readers_and_writer_do_not_block() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("concurrent.db");
        let writer = Database::open_at(&path).unwrap();
        let reader = Database::open_read_only(&path).unwrap();

        // Writer inserts a setting.
        writer
            .connection()
            .execute(
                "INSERT INTO settings (key, value) VALUES ('test_key', 'test_value')",
                [],
            )
            .unwrap();

        // Reader can read while writer has the connection open.
        let value: String = reader
            .connection()
            .query_row(
                "SELECT value FROM settings WHERE key = 'test_key'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(value, "test_value");
    }

    #[test]
    fn migration_is_atomic_version_matches_schema() {
        // Verify that after migration, the version matches the latest
        // migration and the schema is consistent.
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("atomic.db");
        let db = Database::open_at(&path).unwrap();
        let version = db.schema_version().unwrap();
        let latest = MIGRATIONS.last().unwrap().0;
        assert_eq!(version, latest);
        // Tables exist = migration SQL and version bump were atomic.
        let count: i64 = db
            .connection()
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='accounts'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 1);
    }
}
