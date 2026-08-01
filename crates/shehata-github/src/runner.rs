//! Safe execution of the system `gh` binary.

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use secrecy::SecretString;
use serde::Serialize;
use thiserror::Error;
use tokio::io::{AsyncBufReadExt, AsyncRead, BufReader};
use tokio::process::Command;

use crate::models::GhAuthStatus;

const DEFAULT_TIMEOUT: Duration = Duration::from_secs(30);
const TOKEN_TIMEOUT: Duration = Duration::from_secs(15);
const LOGIN_TIMEOUT: Duration = Duration::from_secs(10 * 60);

/// Curated browser-login progress. Raw gh output never crosses the backend
/// boundary; only a device code matching GitHub's one-time-code shape may be
/// sent to callers.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum GhLoginEvent {
    Started,
    WaitingForBrowser,
    Code { code: String },
}

#[derive(Debug, Error)]
pub enum GhError {
    #[error("GitHub CLI (gh) executable not found on PATH")]
    NotFound,
    #[error("failed to spawn gh: {0}")]
    Spawn(String),
    #[error("gh command timed out after {0} seconds")]
    Timeout(u64),
    #[error("gh exited with code {code}: {message}")]
    Exit { code: i32, message: String },
    #[error("gh output was not valid UTF-8")]
    InvalidOutput,
    #[error("could not parse gh auth status JSON")]
    InvalidStatusJson,
    #[error("no token available for user '{login}' on host '{host}'")]
    TokenUnavailable { host: String, login: String },
}

/// A runner bound to a specific `gh` executable path.
#[derive(Debug, Clone)]
pub struct GhRunner {
    gh_path: PathBuf,
}

impl GhRunner {
    /// Locate the system gh executable on PATH.
    pub fn locate() -> Result<Self, GhError> {
        let path = which::which("gh").map_err(|_| GhError::NotFound)?;
        Ok(Self { gh_path: path })
    }

    /// Bind to an explicit executable path (used in tests with fake gh).
    pub fn with_path(path: impl Into<PathBuf>) -> Self {
        Self {
            gh_path: path.into(),
        }
    }

    pub fn executable(&self) -> &Path {
        &self.gh_path
    }

    async fn run(&self, args: &[&str], timeout: Duration) -> Result<(String, i32), GhError> {
        let mut command = Command::new(&self.gh_path);
        command
            .args(args)
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped());

        // Args contain at most a login name — never tokens.
        tracing::debug!(args = ?args, "running gh");

        let child = command.spawn().map_err(|e| GhError::Spawn(e.to_string()))?;
        let result = tokio::time::timeout(timeout, child.wait_with_output())
            .await
            .map_err(|_| GhError::Timeout(timeout.as_secs()))?
            .map_err(|e| GhError::Spawn(e.to_string()))?;

        let stdout = String::from_utf8(result.stdout).map_err(|_| GhError::InvalidOutput)?;
        Ok((stdout, result.status.code().unwrap_or(-1)))
    }

    /// `gh --version` first line, e.g. "gh version 2.97.0 (2026-07-31)".
    pub async fn version(&self) -> Result<String, GhError> {
        let (stdout, code) = self.run(&["--version"], DEFAULT_TIMEOUT).await?;
        if code != 0 {
            return Err(GhError::Exit {
                code,
                message: "gh --version failed".to_string(),
            });
        }
        Ok(stdout.lines().next().unwrap_or_default().trim().to_string())
    }

    /// All authenticated accounts across all hosts.
    pub async fn auth_status(&self) -> Result<GhAuthStatus, GhError> {
        // gh exits nonzero when no hosts are authenticated but still prints
        // valid JSON — so parse stdout first and only fail on bad JSON.
        let (stdout, _code) = self
            .run(&["auth", "status", "--json", "hosts"], DEFAULT_TIMEOUT)
            .await?;
        let trimmed = stdout.trim();
        if trimmed.is_empty() {
            return Ok(GhAuthStatus {
                hosts: Default::default(),
            });
        }
        serde_json::from_str(trimmed).map_err(|_| GhError::InvalidStatusJson)
    }

    /// Start the official GitHub CLI browser login for github.com.
    ///
    /// The GitHub CLI remains the credential source of truth. This method
    /// never receives or persists the resulting token and never forwards raw
    /// command output to the frontend.
    pub async fn login_web<F>(&self, on_event: F) -> Result<(), GhError>
    where
        F: Fn(GhLoginEvent) + Send + Sync + 'static,
    {
        let on_event: Arc<dyn Fn(GhLoginEvent) + Send + Sync> = Arc::new(on_event);
        on_event(GhLoginEvent::Started);

        let mut command = Command::new(&self.gh_path);
        command
            .args([
                "auth",
                "login",
                "--hostname",
                "github.com",
                "--git-protocol",
                "https",
                "--web",
                "--clipboard",
            ])
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .kill_on_drop(true);

        tracing::debug!("starting GitHub CLI browser login");
        let mut child = command.spawn().map_err(|e| GhError::Spawn(e.to_string()))?;
        on_event(GhLoginEvent::WaitingForBrowser);

        let stdout = child.stdout.take().ok_or_else(|| {
            GhError::Spawn("could not capture GitHub CLI login output".to_string())
        })?;
        let stderr = child.stderr.take().ok_or_else(|| {
            GhError::Spawn("could not capture GitHub CLI login diagnostics".to_string())
        })?;
        let stdout_task = tokio::spawn(read_login_stream(stdout, Arc::clone(&on_event)));
        let stderr_task = tokio::spawn(read_login_stream(stderr, Arc::clone(&on_event)));

        let status = tokio::time::timeout(LOGIN_TIMEOUT, child.wait())
            .await
            .map_err(|_| GhError::Timeout(LOGIN_TIMEOUT.as_secs()))?
            .map_err(|e| GhError::Spawn(e.to_string()))?;
        let _ = stdout_task.await;
        let _ = stderr_task.await;

        if !status.success() {
            return Err(GhError::Exit {
                code: status.code().unwrap_or(-1),
                message: "GitHub browser login was cancelled or failed".to_string(),
            });
        }
        Ok(())
    }

    /// Fetch a token for one exact account. The token is returned as a secret
    /// and must be dropped by the caller as soon as possible.
    ///
    /// This never logs the token, never writes it to disk, and never includes
    /// it in errors.
    pub async fn token_for(&self, host: &str, login: &str) -> Result<SecretString, GhError> {
        validate_host_and_login(host, login)?;
        let (stdout, code) = self
            .run(
                &["auth", "token", "--hostname", host, "--user", login],
                TOKEN_TIMEOUT,
            )
            .await?;
        if code != 0 {
            return Err(GhError::TokenUnavailable {
                host: host.to_string(),
                login: login.to_string(),
            });
        }
        // Trim newline characters only, per spec.
        let token = stdout.trim_end_matches(['\r', '\n']).to_string();
        if token.is_empty() {
            return Err(GhError::TokenUnavailable {
                host: host.to_string(),
                login: login.to_string(),
            });
        }
        Ok(SecretString::from(token))
    }
}

async fn read_login_stream<R>(reader: R, on_event: Arc<dyn Fn(GhLoginEvent) + Send + Sync>)
where
    R: AsyncRead + Unpin,
{
    let mut lines = BufReader::new(reader).lines();
    while let Ok(Some(line)) = lines.next_line().await {
        if let Some(code) = extract_device_code(&line) {
            on_event(GhLoginEvent::Code { code });
        }
    }
}

fn extract_device_code(line: &str) -> Option<String> {
    line.split_whitespace()
        .map(|part| part.trim_matches(|c: char| !c.is_ascii_alphanumeric() && c != '-'))
        .find(|part| {
            let bytes = part.as_bytes();
            bytes.len() == 9
                && bytes[4] == b'-'
                && bytes[..4]
                    .iter()
                    .chain(&bytes[5..])
                    .all(|byte| byte.is_ascii_uppercase() || byte.is_ascii_digit())
        })
        .map(str::to_string)
}

/// Hostnames and logins become command arguments — validate them so a
/// malicious value can never become a flag (e.g. "--help") or contain
/// whitespace surprises.
fn validate_host_and_login(host: &str, login: &str) -> Result<(), GhError> {
    fn sane(value: &str) -> bool {
        !value.is_empty()
            && !value.starts_with('-')
            && value
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.'))
    }
    if !sane(host) || !sane(login) {
        return Err(GhError::TokenUnavailable {
            host: host.to_string(),
            login: login.to_string(),
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_dangerous_logins() {
        assert!(validate_host_and_login("github.com", "--help").is_err());
        assert!(validate_host_and_login("github.com", "evil user").is_err());
        assert!(validate_host_and_login("", "user").is_err());
        assert!(validate_host_and_login("github.com", "ok-user_1").is_ok());
    }

    #[test]
    fn extracts_only_github_shaped_device_codes() {
        assert_eq!(
            extract_device_code("First copy your one-time code: ABCD-1EFG"),
            Some("ABCD-1EFG".to_string())
        );
        assert_eq!(extract_device_code("https://github.com/login/device"), None);
        assert_eq!(extract_device_code("token ghp_not-a-device-code"), None);
        assert_eq!(extract_device_code("abcd-1efg"), None);
    }

    #[cfg(target_os = "windows")]
    #[tokio::test]
    async fn browser_login_streams_only_curated_events_from_fake_gh() {
        use std::sync::Mutex;

        let dir = tempfile::tempdir().unwrap();
        let fake_gh = dir.path().join("gh.cmd");
        std::fs::write(
            &fake_gh,
            "@echo off\r\necho First copy your one-time code: TEST-1234\r\necho raw diagnostic that must stay private 1>&2\r\nexit /b 0\r\n",
        )
        .unwrap();

        let events = Arc::new(Mutex::new(Vec::new()));
        let captured = Arc::clone(&events);
        GhRunner::with_path(fake_gh)
            .login_web(move |event| captured.lock().unwrap().push(event))
            .await
            .unwrap();

        let events = events.lock().unwrap();
        assert_eq!(events.first(), Some(&GhLoginEvent::Started));
        assert!(events.contains(&GhLoginEvent::WaitingForBrowser));
        assert!(events.contains(&GhLoginEvent::Code {
            code: "TEST-1234".to_string(),
        }));
        assert_eq!(events.len(), 3, "raw gh lines must never become events");
    }
}
