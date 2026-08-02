//! Parsing of Git's credential-helper protocol input.
//!
//! Git sends key=value lines on stdin, terminated by a blank line:
//!   protocol=https
//!   host=github.com
//!   path=owner/repo.git        (when credential.useHttpPath is true)
//!   username=...               (optional)
//!
//! Parsed defensively: unknown keys ignored, input capped, values validated.

use std::collections::BTreeMap;

/// Maximum bytes we will ever read from stdin. Git sends a few hundred bytes;
/// anything larger is an attack or a bug, not a credential request.
pub const MAX_INPUT_BYTES: usize = 16 * 1024;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CredentialRequest {
    pub protocol: Option<String>,
    pub host: Option<String>,
    pub path: Option<String>,
    pub username: Option<String>,
    /// Full URL if git sent one (takes precedence in git, we keep it parsed).
    pub url: Option<String>,
}

impl CredentialRequest {
    pub fn parse(input: &str) -> Self {
        let mut map = BTreeMap::new();
        for line in input.lines() {
            let line = line.trim_end_matches('\r');
            if line.is_empty() {
                break;
            }
            if let Some((key, value)) = line.split_once('=') {
                // Last writer wins, matching git's semantics.
                map.insert(key.trim().to_string(), value.to_string());
            }
        }
        Self {
            protocol: map.get("protocol").cloned(),
            host: map.get("host").cloned(),
            path: map.get("path").cloned(),
            username: map.get("username").cloned(),
            url: map.get("url").cloned(),
        }
    }

    /// Only HTTPS is routable by Shehata Git in v0.1.
    pub fn is_supported(&self) -> bool {
        matches!(self.protocol.as_deref(), Some("https")) && self.host.is_some()
    }

    /// Normalize the `path` field for comparison: strip optional `.git` suffix,
    /// lowercase, and trim leading/trailing slashes.
    pub fn normalized_repo_path(&self) -> Option<String> {
        self.path.as_deref().map(|p| {
            let trimmed = p.trim_matches('/');
            let stripped = trimmed.strip_suffix(".git").unwrap_or(trimmed);
            stripped.to_ascii_lowercase()
        })
    }

    /// Returns true if the `url` field contains embedded credentials
    /// (username or password in the URL).
    pub fn has_embedded_credentials(&self) -> bool {
        if let Some(ref url_str) = self.url {
            if let Ok(parsed) = url::Url::parse(url_str) {
                return !parsed.username().is_empty() || parsed.password().is_some();
            }
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_standard_request() {
        let req =
            CredentialRequest::parse("protocol=https\nhost=github.com\npath=owner/repo.git\n\n");
        assert_eq!(req.protocol.as_deref(), Some("https"));
        assert_eq!(req.host.as_deref(), Some("github.com"));
        assert_eq!(req.path.as_deref(), Some("owner/repo.git"));
    }

    #[test]
    fn stops_at_blank_line_and_ignores_garbage() {
        let req = CredentialRequest::parse(
            "protocol=https\nhost=github.com\n\nevil=injected\nnoequalsline\n",
        );
        assert_eq!(req.host.as_deref(), Some("github.com"));
        assert!(req.path.is_none());
    }

    #[test]
    fn handles_crlf() {
        let req = CredentialRequest::parse("protocol=https\r\nhost=github.com\r\n\r\n");
        assert_eq!(req.host.as_deref(), Some("github.com"));
    }

    #[test]
    fn unsupported_protocols_rejected() {
        let ssh = CredentialRequest::parse("protocol=ssh\nhost=github.com\n\n");
        assert!(!ssh.is_supported());
        let https = CredentialRequest::parse("protocol=https\nhost=github.com\n\n");
        assert!(https.is_supported());
    }

    #[test]
    fn missing_host_rejected() {
        let req = CredentialRequest::parse("protocol=https\n\n");
        assert!(!req.is_supported());
    }
}
