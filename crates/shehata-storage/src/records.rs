//! Row types. All timestamps are RFC 3339 UTC strings for determinism.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountRecord {
    pub id: i64,
    pub host: String,
    pub login: String,
    pub display_name: Option<String>,
    pub avatar_url: Option<String>,
    /// Where the credential lives. Always "gh-cli" in v0.1.
    pub auth_source: String,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
    pub last_validated_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositoryRecord {
    pub id: String,
    pub canonical_path: String,
    pub git_dir: Option<String>,
    pub git_common_dir: Option<String>,
    pub display_name: String,
    pub host: Option<String>,
    pub owner: Option<String>,
    pub repo_name: Option<String>,
    pub remote_name: Option<String>,
    pub remote_url: Option<String>,
    pub current_branch: Option<String>,
    pub assigned_account_id: Option<i64>,
    pub commit_name: Option<String>,
    pub commit_email: Option<String>,
    /// allow_normal_push | ask_before_push | block_ai_push
    pub push_policy: String,
    pub created_at: String,
    pub updated_at: String,
    pub last_seen_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigBackupRecord {
    pub id: i64,
    pub repository_id: String,
    /// e.g. "credential.helper", "credential.useHttpPath", "user.name",
    /// "user.email", "remote.origin.url"
    pub config_key: String,
    /// JSON array of previous values (git config is multi-valued).
    pub previous_values_json: String,
    pub created_at: String,
    /// True once an unlink/restore consumed this backup.
    pub restored_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEventRecord {
    pub id: i64,
    pub timestamp: String,
    pub repository_id: Option<String>,
    pub event_type: String,
    pub account_login: Option<String>,
    /// Safe human summary — never contains secrets or file contents.
    pub summary: String,
    /// Repository, branch, and commit context. Same safety rules as `summary`.
    pub detail: Option<String>,
    /// "success" | "failure" | "blocked"
    pub result: String,
    pub exit_code: Option<i64>,
    pub duration_ms: Option<i64>,
}

/// Safe fields accepted when writing an audit event. Keeping these fields in
/// one value makes it harder for callers to mix up the optional identifiers
/// and timing values.
#[derive(Debug, Clone)]
pub struct NewAuditEvent<'a> {
    pub event_type: &'a str,
    pub repository_id: Option<&'a str>,
    pub account_login: Option<&'a str>,
    pub summary: &'a str,
    pub detail: Option<&'a str>,
    pub result: &'a str,
    pub exit_code: Option<i64>,
    pub duration_ms: Option<i64>,
}
