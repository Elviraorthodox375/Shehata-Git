//! Parsing of Git remote URLs into (host, owner, repo) components.
//!
//! Supported forms (at minimum):
//! - https://github.com/owner/repo.git
//! - https://github.com/owner/repo
//! - git@github.com:owner/repo.git
//! - ssh://git@github.com/owner/repo.git

use thiserror::Error;
use url::Url;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemoteUrl {
    pub host: String,
    pub owner: String,
    pub repo: String,
    pub protocol: RemoteProtocol,
}

impl RemoteUrl {
    /// Reconstruct a safe canonical URL with no credentials, query, or fragment.
    pub fn canonical_url(&self) -> String {
        match self.protocol {
            RemoteProtocol::Https => {
                format!("https://{}/{}/{}.git", self.host, self.owner, self.repo)
            }
            RemoteProtocol::Ssh => format!("git@{}:{}/{}.git", self.host, self.owner, self.repo),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RemoteProtocol {
    Https,
    Ssh,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum RemoteParseError {
    #[error("remote URL is empty")]
    Empty,
    #[error("unsupported remote URL format: {0}")]
    UnsupportedFormat(String),
    #[error("remote URL has no host: {0}")]
    NoHost(String),
    #[error("remote path must look like owner/repo: {0}")]
    BadPath(String),
    #[error("repository owner or name is empty in: {0}")]
    EmptySegment(String),
    #[error("remote URL contains embedded credentials — remove userinfo before importing")]
    ContainsCredentials,
    #[error("remote URL has unsupported query or fragment: {0}")]
    HasQueryOrFragment(String),
    #[error("remote URL has extra path segments beyond owner/repo: {0}")]
    ExtraSegments(String),
}

/// Parse a Git remote URL. Defensive: trims whitespace, rejects anything
/// that does not resolve to host + owner + repo.
pub fn parse_remote_url(input: &str) -> Result<RemoteUrl, RemoteParseError> {
    let raw = input.trim();
    if raw.is_empty() {
        return Err(RemoteParseError::Empty);
    }

    // SCP-like SSH syntax: git@host:owner/repo(.git)
    if let Some((user_host, path)) = raw.split_once(':') {
        if !raw.contains("://") && user_host.contains('@') {
            let host = user_host.rsplit('@').next().unwrap_or_default().to_string();
            if host.is_empty() {
                return Err(RemoteParseError::NoHost(raw.to_string()));
            }
            return build(raw, host, path, RemoteProtocol::Ssh);
        }
    }

    // URL syntax: https://… or ssh://…
    if raw.contains("://") {
        let parsed =
            Url::parse(raw).map_err(|_| RemoteParseError::UnsupportedFormat(raw.to_string()))?;

        // Reject credentials embedded in HTTPS URLs (SSH legitimately uses
        // git@host syntax, so this check applies to HTTPS only).
        let protocol = match parsed.scheme() {
            "https" => {
                if !parsed.username().is_empty() || parsed.password().is_some() {
                    return Err(RemoteParseError::ContainsCredentials);
                }
                RemoteProtocol::Https
            }
            "ssh" | "git+ssh" => RemoteProtocol::Ssh,
            other => return Err(RemoteParseError::UnsupportedFormat(other.to_string())),
        };
        // Reject query strings and fragments.
        if parsed.query().is_some() || parsed.fragment().is_some() {
            return Err(RemoteParseError::HasQueryOrFragment(raw.to_string()));
        }
        let host = parsed
            .host_str()
            .ok_or_else(|| RemoteParseError::NoHost(raw.to_string()))?
            .to_string();
        let path = parsed.path().trim_start_matches('/');
        return build(raw, host, path, protocol);
    }

    Err(RemoteParseError::UnsupportedFormat(raw.to_string()))
}

fn build(
    raw: &str,
    host: String,
    path: &str,
    protocol: RemoteProtocol,
) -> Result<RemoteUrl, RemoteParseError> {
    let clean = path.strip_suffix(".git").unwrap_or(path);
    let mut segments = clean.split('/').filter(|s| !s.is_empty());
    let owner = segments.next();
    let repo = segments.next();
    match (owner, repo) {
        (Some(owner), Some(repo)) => {
            if owner.is_empty() || repo.is_empty() {
                return Err(RemoteParseError::EmptySegment(raw.to_string()));
            }
            // Reject extra path segments beyond owner/repo.
            if segments.next().is_some() {
                return Err(RemoteParseError::ExtraSegments(raw.to_string()));
            }
            Ok(RemoteUrl {
                host,
                owner: owner.to_string(),
                repo: repo.to_string(),
                protocol,
            })
        }
        _ => Err(RemoteParseError::BadPath(raw.to_string())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_https_with_git_suffix() {
        let r = parse_remote_url("https://github.com/owner/repo.git").unwrap();
        assert_eq!(r.host, "github.com");
        assert_eq!(r.owner, "owner");
        assert_eq!(r.repo, "repo");
        assert_eq!(r.protocol, RemoteProtocol::Https);
    }

    #[test]
    fn parses_https_without_suffix() {
        let r = parse_remote_url("https://github.com/owner/repo").unwrap();
        assert_eq!(r.repo, "repo");
        assert_eq!(r.protocol, RemoteProtocol::Https);
    }

    #[test]
    fn parses_scp_like_ssh() {
        let r = parse_remote_url("git@github.com:owner/repo.git").unwrap();
        assert_eq!(r.host, "github.com");
        assert_eq!(r.owner, "owner");
        assert_eq!(r.repo, "repo");
        assert_eq!(r.protocol, RemoteProtocol::Ssh);
    }

    #[test]
    fn parses_ssh_url() {
        let r = parse_remote_url("ssh://git@github.com/owner/repo.git").unwrap();
        assert_eq!(r.host, "github.com");
        assert_eq!(r.repo, "repo");
        assert_eq!(r.protocol, RemoteProtocol::Ssh);
    }

    #[test]
    fn parses_enterprise_host() {
        let r = parse_remote_url("https://git.mycompany.io/team/service.git").unwrap();
        assert_eq!(r.host, "git.mycompany.io");
        assert_eq!(r.owner, "team");
    }

    #[test]
    fn rejects_empty() {
        assert_eq!(parse_remote_url("   "), Err(RemoteParseError::Empty));
    }

    #[test]
    fn rejects_garbage() {
        assert!(parse_remote_url("not-a-url").is_err());
    }

    #[test]
    fn rejects_unencrypted_http() {
        assert!(parse_remote_url("http://github.com/acme/repo.git").is_err());
    }

    #[test]
    fn rejects_missing_repo() {
        assert!(parse_remote_url("https://github.com/owner").is_err());
    }

    #[test]
    fn rejects_username_in_url() {
        assert!(matches!(
            parse_remote_url("https://user@github.com/owner/repo.git"),
            Err(RemoteParseError::ContainsCredentials)
        ));
    }

    #[test]
    fn rejects_password_in_url() {
        assert!(matches!(
            parse_remote_url("https://user:token@github.com/owner/repo.git"),
            Err(RemoteParseError::ContainsCredentials)
        ));
    }

    #[test]
    fn rejects_query_string() {
        assert!(matches!(
            parse_remote_url("https://github.com/owner/repo.git?ref=main"),
            Err(RemoteParseError::HasQueryOrFragment(_))
        ));
    }

    #[test]
    fn rejects_fragment() {
        assert!(matches!(
            parse_remote_url("https://github.com/owner/repo.git#readme"),
            Err(RemoteParseError::HasQueryOrFragment(_))
        ));
    }

    #[test]
    fn rejects_extra_path_segments() {
        assert!(matches!(
            parse_remote_url("https://github.com/owner/repo/extra/path"),
            Err(RemoteParseError::ExtraSegments(_))
        ));
    }

    #[test]
    fn canonical_url_is_safe() {
        let r = parse_remote_url("https://github.com/owner/repo.git").unwrap();
        assert_eq!(r.canonical_url(), "https://github.com/owner/repo.git");
    }

    #[test]
    fn canonical_url_for_ssh() {
        let r = parse_remote_url("git@github.com:owner/repo.git").unwrap();
        assert_eq!(r.canonical_url(), "git@github.com:owner/repo.git");
    }
}
