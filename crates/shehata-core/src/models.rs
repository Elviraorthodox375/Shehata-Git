//! Shared domain models exchanged between core and its callers.

use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CheckStatus {
    Ready,
    Missing,
    NeedsAttention,
}

#[derive(Debug, Clone, Serialize)]
pub struct SystemCheck {
    pub id: String,
    pub label: String,
    pub status: CheckStatus,
    /// Plain-language explanation, safe for non-programmers.
    pub detail: String,
    /// Simple repair instruction when not ready.
    pub repair_hint: Option<String>,
    pub version: Option<String>,
    /// Accounts this check can repair in place, so the UI can offer a button
    /// instead of an instruction. Empty for checks the app cannot fix itself.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub repairable_accounts: Vec<AccountScopeRepair>,
}

/// One account that is missing a single OAuth scope Shehata Git can request.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct AccountScopeRepair {
    pub host: String,
    pub login: String,
    pub scope: String,
}

impl SystemCheck {
    pub fn ready(
        id: &str,
        label: &str,
        detail: impl Into<String>,
        version: Option<String>,
    ) -> Self {
        Self {
            id: id.to_string(),
            label: label.to_string(),
            status: CheckStatus::Ready,
            detail: detail.into(),
            repair_hint: None,
            version,
            repairable_accounts: Vec::new(),
        }
    }

    pub fn missing(
        id: &str,
        label: &str,
        detail: impl Into<String>,
        repair: impl Into<String>,
    ) -> Self {
        Self {
            id: id.to_string(),
            label: label.to_string(),
            status: CheckStatus::Missing,
            detail: detail.into(),
            repair_hint: Some(repair.into()),
            version: None,
            repairable_accounts: Vec::new(),
        }
    }

    pub fn attention(
        id: &str,
        label: &str,
        detail: impl Into<String>,
        repair: impl Into<String>,
        version: Option<String>,
    ) -> Self {
        Self {
            id: id.to_string(),
            label: label.to_string(),
            status: CheckStatus::NeedsAttention,
            detail: detail.into(),
            repair_hint: Some(repair.into()),
            version,
            repairable_accounts: Vec::new(),
        }
    }

    /// Attach the accounts this check can repair through the app.
    pub fn repairing(mut self, accounts: Vec<AccountScopeRepair>) -> Self {
        self.repairable_accounts = accounts;
        self
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct DoctorReport {
    pub os: String,
    pub app_version: String,
    pub healthy: bool,
    pub checks: Vec<SystemCheck>,
}

/// One authenticated GitHub account as seen by the GitHub CLI.
#[derive(Debug, Clone, Serialize)]
pub struct AccountInfo {
    pub host: String,
    pub login: String,
    /// Whether this is the globally active account in gh (informational only;
    /// Shehata Git routes by assignment, not by the active account).
    pub active: bool,
    /// Whether a token could actually be retrieved right now.
    pub token_available: bool,
}

/// Push policies for a repository.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PushPolicy {
    AllowNormalPush,
    AskBeforePush,
    BlockAiPush,
}

impl PushPolicy {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::AllowNormalPush => "allow_normal_push",
            Self::AskBeforePush => "ask_before_push",
            Self::BlockAiPush => "block_ai_push",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "allow_normal_push" => Some(Self::AllowNormalPush),
            "ask_before_push" => Some(Self::AskBeforePush),
            "block_ai_push" => Some(Self::BlockAiPush),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn push_policy_roundtrip() {
        for policy in [
            PushPolicy::AllowNormalPush,
            PushPolicy::AskBeforePush,
            PushPolicy::BlockAiPush,
        ] {
            assert_eq!(PushPolicy::parse(policy.as_str()), Some(policy));
        }
        assert_eq!(PushPolicy::parse("force_push"), None);
    }
}
