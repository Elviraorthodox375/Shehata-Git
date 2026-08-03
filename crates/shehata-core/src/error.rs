//! Shared error model.
//!
//! Every fallible operation across CLI, desktop, credential helper, and MCP
//! surfaces a `ShehataError` with a **stable machine code**. Error text must
//! never contain credentials — see `redact`.

use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ShehataError {
    // --- stable domain errors (code-stable, mapped by MCP/CLI) ---
    #[error("repository not found: {0}")]
    RepositoryNotFound(String),

    #[error("repository is not linked to Shehata Git: {0}")]
    RepositoryNotLinked(String),

    #[error("assigned account is not available: {login} on {host}")]
    AccountNotAvailable { host: String, login: String },

    #[error("credential helper binary is missing or not on PATH")]
    CredentialHelperMissing,

    #[error("authentication failed for the assigned account")]
    AuthenticationFailed,

    #[error("this repository requires approval before pushing")]
    ApprovalRequired,

    #[error("repository has unresolved merge conflicts")]
    ConflictsPresent,

    #[error("repository is in detached HEAD state")]
    DetachedHead,

    #[error("current branch has no upstream configured")]
    NoUpstream,

    #[error("push would not be a fast-forward; pull first")]
    NonFastForward,

    #[error("another operation is already running for this repository: {0}")]
    OperationInProgress(String),

    #[error("operation is blocked by policy: {0}")]
    OperationBlocked(String),

    #[error("invalid input: {0}")]
    InvalidInput(String),

    #[error("could not update repository marker: {0}")]
    RepositoryMarker(String),

    #[error(
        "Windows Package Manager (winget) is unavailable. Install \"App Installer\" from the \
         Microsoft Store, then run this check again."
    )]
    PackageManagerMissing,

    #[error("automatic setup failed for {tool} (installer exit code {code})")]
    PrerequisiteInstallFailed { tool: String, code: i32 },

    // --- wrapped subsystem errors ---
    #[error(transparent)]
    Git(#[from] shehata_git::GitError),

    #[error(transparent)]
    RepositoryDiscovery(#[from] shehata_git::RepositoryDiscoveryError),

    #[error(transparent)]
    Github(#[from] shehata_github::GhError),

    #[error(transparent)]
    Storage(#[from] shehata_storage::StorageError),

    #[error("internal error: {0}")]
    Internal(String),
}

impl ShehataError {
    /// Stable machine-readable code for MCP results and CLI `--json` output.
    pub fn code(&self) -> &'static str {
        match self {
            Self::RepositoryNotFound(_) => "repository_not_found",
            Self::RepositoryNotLinked(_) => "repository_not_linked",
            Self::AccountNotAvailable { .. } => "account_not_available",
            Self::CredentialHelperMissing => "credential_helper_missing",
            Self::AuthenticationFailed => "authentication_failed",
            Self::ApprovalRequired => "approval_required",
            Self::ConflictsPresent => "conflicts_present",
            Self::DetachedHead => "detached_head",
            Self::NoUpstream => "no_upstream",
            Self::NonFastForward => "non_fast_forward",
            Self::OperationInProgress(_) => "operation_in_progress",
            Self::OperationBlocked(_) => "operation_blocked",
            Self::InvalidInput(_) => "invalid_input",
            Self::RepositoryMarker(_) => "repository_marker_error",
            Self::PackageManagerMissing => "package_manager_missing",
            Self::PrerequisiteInstallFailed { .. } => "prerequisite_install_failed",
            Self::Git(_) => "git_error",
            Self::RepositoryDiscovery(_) => "repository_discovery_error",
            Self::Github(_) => "github_cli_error",
            Self::Storage(_) => "storage_error",
            Self::Internal(_) => "internal_error",
        }
    }
}

impl Serialize for ShehataError {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("ShehataError", 2)?;
        state.serialize_field("code", self.code())?;
        state.serialize_field("message", &self.to_string())?;
        state.end()
    }
}

pub type Result<T> = std::result::Result<T, ShehataError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn codes_are_stable() {
        assert_eq!(
            ShehataError::RepositoryNotFound("x".into()).code(),
            "repository_not_found"
        );
        assert_eq!(ShehataError::ApprovalRequired.code(), "approval_required");
        assert_eq!(ShehataError::NonFastForward.code(), "non_fast_forward");
        assert_eq!(
            ShehataError::OperationBlocked("force push".into()).code(),
            "operation_blocked"
        );
    }

    #[test]
    fn serializes_with_code_and_message() {
        let err = ShehataError::DetachedHead;
        let json = serde_json::to_value(&err).unwrap();
        assert_eq!(json["code"], "detached_head");
        assert!(json["message"].as_str().unwrap().contains("detached HEAD"));
    }
}
