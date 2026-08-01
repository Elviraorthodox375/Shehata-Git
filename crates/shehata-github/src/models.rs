//! Shapes of `gh auth status --json hosts` output.
//!
//! Parsed defensively: unknown fields are ignored, missing fields fall back
//! to safe defaults, and a host with zero accounts simply does not appear.

use serde::Deserialize;
use std::collections::BTreeMap;

#[derive(Debug, Clone, Deserialize)]
pub struct GhAuthStatus {
    #[serde(default)]
    pub hosts: BTreeMap<String, Vec<GhAuthAccount>>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GhAuthAccount {
    /// "success" or "error" — anything other than success means the token
    /// cannot currently be used.
    #[serde(default)]
    pub state: String,
    #[serde(default)]
    pub active: bool,
    #[serde(default)]
    pub host: String,
    #[serde(default)]
    pub login: String,
    #[serde(default)]
    pub token_source: String,
    #[serde(default)]
    pub scopes: String,
    #[serde(default)]
    pub git_protocol: String,
    /// Present when state == "error".
    #[serde(default)]
    pub error: String,
}

impl GhAuthAccount {
    pub fn token_usable(&self) -> bool {
        self.state == "success"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = r#"{
        "hosts": {
            "github.com": [
                {
                    "state": "success",
                    "active": true,
                    "host": "github.com",
                    "login": "first-user",
                    "tokenSource": "keyring",
                    "scopes": "repo, read:org, gist, workflow",
                    "gitProtocol": "https"
                },
                {
                    "state": "success",
                    "active": false,
                    "host": "github.com",
                    "login": "second-user",
                    "tokenSource": "keyring",
                    "scopes": "repo, read:org",
                    "gitProtocol": "https"
                }
            ],
            "git.enterprise.example": [
                {
                    "state": "error",
                    "active": false,
                    "host": "git.enterprise.example",
                    "login": "ent-user",
                    "error": "token validation failed"
                }
            ]
        }
    }"#;

    #[test]
    fn parses_multiple_accounts_and_hosts() {
        let status: GhAuthStatus = serde_json::from_str(SAMPLE).unwrap();
        assert_eq!(status.hosts.len(), 2);

        let github = &status.hosts["github.com"];
        assert_eq!(github.len(), 2);
        assert_eq!(github[0].login, "first-user");
        assert!(github[0].active);
        assert!(github[0].token_usable());
        assert_eq!(github[1].login, "second-user");
        assert!(!github[1].active);

        let ent = &status.hosts["git.enterprise.example"];
        assert_eq!(ent[0].login, "ent-user");
        assert!(!ent[0].token_usable());
    }

    #[test]
    fn tolerates_empty_hosts() {
        let status: GhAuthStatus = serde_json::from_str(r#"{"hosts": {}}"#).unwrap();
        assert!(status.hosts.is_empty());
    }

    #[test]
    fn tolerates_missing_fields() {
        let status: GhAuthStatus =
            serde_json::from_str(r#"{"hosts": {"github.com": [{"login": "x"}]}}"#).unwrap();
        let account = &status.hosts["github.com"][0];
        assert_eq!(account.login, "x");
        assert!(!account.active);
        assert!(!account.token_usable());
    }

    #[test]
    fn rejects_non_json() {
        assert!(serde_json::from_str::<GhAuthStatus>("not json").is_err());
    }
}
