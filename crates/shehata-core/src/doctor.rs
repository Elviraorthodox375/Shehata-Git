//! The system doctor: verifies everything Shehata Git needs, live.
//!
//! Each check produces a `SystemCheck` with a plain-language explanation and
//! a repair hint. The doctor never installs or changes anything by itself.

use std::path::PathBuf;

use shehata_git::GitRunner;
use shehata_github::GhRunner;
use shehata_storage::Database;

use crate::models::{CheckStatus, DoctorReport, SystemCheck};

pub const APP_VERSION: &str = env!("CARGO_PKG_VERSION");

pub struct Doctor {
    git: Option<GitRunner>,
    gh: Option<GhRunner>,
    db_path: Option<PathBuf>,
}

impl Default for Doctor {
    fn default() -> Self {
        Self::new()
    }
}

impl Doctor {
    /// System defaults: locate git/gh on PATH, database in app-data.
    pub fn new() -> Self {
        Self {
            git: GitRunner::locate().ok(),
            gh: GhRunner::locate().ok(),
            db_path: Database::default_path().ok(),
        }
    }

    /// Explicit dependencies — used by tests with fake binaries and temp dirs.
    pub fn with_dependencies(
        git: Option<GitRunner>,
        gh: Option<GhRunner>,
        db_path: Option<PathBuf>,
    ) -> Self {
        Self { git, gh, db_path }
    }

    pub async fn run(&self) -> DoctorReport {
        let mut checks = Vec::new();

        checks.push(self.check_git().await);
        checks.push(self.check_gh().await);
        checks.push(self.check_gh_accounts().await);
        checks.push(self.check_database());
        checks.push(self.check_credential_helper());
        checks.push(self.check_webview());
        checks.push(self.check_path());
        checks.push(self.check_mcp_server());

        let healthy = checks.iter().all(|c| c.status == CheckStatus::Ready);

        DoctorReport {
            os: os_string(),
            app_version: APP_VERSION.to_string(),
            healthy,
            checks,
        }
    }

    async fn check_git(&self) -> SystemCheck {
        match &self.git {
            None => SystemCheck::missing(
                "git",
                "Git",
                "Git is not installed or not on PATH. Shehata Git drives the same git your terminal uses.",
                "Choose Set up this PC above to install Git automatically with Windows Package Manager.",
            ),
            Some(git) => match git.version().await {
                Ok(version) => SystemCheck::ready(
                    "git",
                    "Git",
                    format!("Found at {}", git.executable().display()),
                    Some(version),
                ),
                Err(_) => SystemCheck::attention(
                    "git",
                    "Git",
                    format!(
                        "Found at {} but it did not answer --version.",
                        git.executable().display()
                    ),
                    "Reinstall Git for Windows, then restart Shehata Git.",
                    None,
                ),
            },
        }
    }

    async fn check_gh(&self) -> SystemCheck {
        match &self.gh {
            None => SystemCheck::missing(
                "gh",
                "GitHub CLI",
                "GitHub CLI is not installed. It is the sign-in and credential source for all your GitHub accounts.",
                "Choose Set up this PC above to install GitHub CLI automatically with Windows Package Manager.",
            ),
            Some(gh) => match gh.version().await {
                Ok(version) => SystemCheck::ready(
                    "gh",
                    "GitHub CLI",
                    format!("Found at {}", gh.executable().display()),
                    Some(version),
                ),
                Err(_) => SystemCheck::attention(
                    "gh",
                    "GitHub CLI",
                    format!(
                        "Found at {} but it did not answer --version.",
                        gh.executable().display()
                    ),
                    "Reinstall with: winget install GitHub.cli",
                    None,
                ),
            },
        }
    }

    async fn check_gh_accounts(&self) -> SystemCheck {
        let Some(gh) = &self.gh else {
            return SystemCheck::missing(
                "gh-accounts",
                "GitHub accounts",
                "Cannot read accounts because the GitHub CLI is missing.",
                "Install the GitHub CLI first.",
            );
        };
        match gh.auth_status().await {
            Ok(status) => {
                let count: usize = status.hosts.values().map(Vec::len).sum();
                if count == 0 {
                    SystemCheck::attention(
                        "gh-accounts",
                        "GitHub accounts",
                        "No GitHub accounts are signed in yet.",
                        "Add an account from the Accounts page — the browser will open and no password is typed into this app.",
                        None,
                    )
                } else {
                    SystemCheck::ready(
                        "gh-accounts",
                        "GitHub accounts",
                        format!(
                            "{count} account{} signed in.",
                            if count == 1 { "" } else { "s" }
                        ),
                        None,
                    )
                }
            }
            Err(_) => SystemCheck::attention(
                "gh-accounts",
                "GitHub accounts",
                "Could not read account status from the GitHub CLI.",
                "Open a terminal and run: gh auth status — it will show what is wrong.",
                None,
            ),
        }
    }

    fn check_database(&self) -> SystemCheck {
        let Some(path) = &self.db_path else {
            return SystemCheck::missing(
                "database",
                "Local database",
                "Could not determine the application data folder.",
                "Check that your Windows user profile is healthy.",
            );
        };
        match Database::open_at(path) {
            Ok(_) => SystemCheck::ready(
                "database",
                "Local database",
                format!("SQLite database at {}", path.display()),
                None,
            ),
            Err(e) => SystemCheck::attention(
                "database",
                "Local database",
                format!("Database exists at {} but could not be opened: {e}", path.display()),
                "Close other Shehata Git windows and try again. If it persists, run: shehata doctor",
                None,
            ),
        }
    }

    fn check_credential_helper(&self) -> SystemCheck {
        match locate_binary("git-credential-shehata") {
            Some(path) => SystemCheck::ready(
                "credential-helper",
                "Credential helper",
                "The helper that routes credentials per repository is available.",
                Some(path.display().to_string()),
            ),
            None => SystemCheck::attention(
                "credential-helper",
                "Credential helper",
                "git-credential-shehata was not found next to the app or on PATH. Repository pushes cannot be routed until it is available.",
                "Reinstall Shehata Git, or build the workspace with: cargo build --release",
                None,
            ),
        }
    }

    fn check_webview(&self) -> SystemCheck {
        #[cfg(target_os = "windows")]
        {
            match webview2_version() {
                Some(version) => SystemCheck::ready(
                    "webview",
                    "Windows WebView2",
                    "The runtime that renders this window is installed.",
                    Some(version),
                ),
                None => SystemCheck::missing(
                    "webview",
                    "Windows WebView2",
                    "The WebView2 runtime is missing; the app window cannot render without it.",
                    "Install it from https://developer.microsoft.com/microsoft-edge/webview2/ then restart.",
                ),
            }
        }
        #[cfg(not(target_os = "windows"))]
        {
            SystemCheck::ready(
                "webview",
                "WebView",
                "Platform webview is provided by the OS.",
                None,
            )
        }
    }

    fn check_path(&self) -> SystemCheck {
        let Ok(exe) = std::env::current_exe() else {
            return SystemCheck::attention(
                "path",
                "User PATH",
                "Could not determine where this app is running from.",
                "Reinstall Shehata Git.",
                None,
            );
        };
        let dir = exe.parent().map(|p| p.to_path_buf()).unwrap_or_default();
        let on_path = directory_is_on_path(&dir);
        if on_path {
            SystemCheck::ready(
                "path",
                "User PATH",
                "The app directory is on your user PATH, so git can find the credential helper.",
                None,
            )
        } else {
            SystemCheck::attention(
                "path",
                "User PATH",
                format!(
                    "{} is not on your user PATH. Terminal and AI-tool pushes need it to find the credential helper.",
                    dir.display()
                ),
                "The Windows installer will add the installed app directory to your user PATH. In development, use the binaries in target\\debug directly.",
                None,
            )
        }
    }

    fn check_mcp_server(&self) -> SystemCheck {
        match locate_binary("shehata-mcp") {
            Some(path) => SystemCheck::ready(
                "mcp",
                "MCP server",
                "The AI-integration server is available.",
                Some(path.display().to_string()),
            ),
            None => SystemCheck::attention(
                "mcp",
                "MCP server",
                "shehata-mcp was not found next to the app or on PATH. AI tools cannot use Shehata Git until it is available.",
                "Reinstall Shehata Git, or build the workspace with: cargo build --release",
                None,
            ),
        }
    }
}

fn directory_is_on_path(directory: &std::path::Path) -> bool {
    let expected = normalized_path(directory);
    let process_has_path = std::env::var_os("PATH")
        .map(|paths| std::env::split_paths(&paths).any(|path| normalized_path(&path) == expected))
        .unwrap_or(false);
    if process_has_path {
        return true;
    }

    #[cfg(target_os = "windows")]
    {
        use winreg::enums::HKEY_CURRENT_USER;
        use winreg::RegKey;

        let user = RegKey::predef(HKEY_CURRENT_USER);
        if let Ok(environment) = user.open_subkey("Environment") {
            if let Ok(path) = environment.get_value::<String, _>("Path") {
                return std::env::split_paths(&path)
                    .any(|entry| normalized_path(&entry) == expected);
            }
        }
    }

    false
}

fn normalized_path(path: &std::path::Path) -> String {
    path.to_string_lossy()
        .trim()
        .trim_matches('"')
        .trim_end_matches(['\\', '/'])
        .to_lowercase()
}

/// Find a binary next to the current executable first, then on PATH.
fn locate_binary(name: &str) -> Option<PathBuf> {
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            let candidate = dir.join(format!("{name}.exe"));
            if candidate.exists() {
                return Some(candidate);
            }
            let candidate = dir.join(name);
            if candidate.exists() {
                return Some(candidate);
            }
        }
    }
    which::which(name).ok()
}

#[cfg(target_os = "windows")]
fn webview2_version() -> Option<String> {
    // Registry check without extra crates: `reg query` with argument arrays.
    let output = std::process::Command::new("reg")
        .args([
            "query",
            r"HKLM\SOFTWARE\WOW6432Node\Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}",
            "/v",
            "pv",
        ])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let text = String::from_utf8_lossy(&output.stdout);
    text.lines()
        .find(|line| line.contains("REG_SZ"))
        .and_then(|line| line.split_whitespace().last())
        .map(|v| v.to_string())
}

fn os_string() -> String {
    format!("{} {}", std::env::consts::OS, std::env::consts::ARCH)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn doctor_runs_on_dev_machine() {
        // The dev machine has git + gh installed (verified in Phase 0).
        let report = Doctor::new().run().await;
        assert_eq!(report.checks.len(), 8);
        let git = report.checks.iter().find(|c| c.id == "git").unwrap();
        assert_eq!(git.status, CheckStatus::Ready);
        let gh = report.checks.iter().find(|c| c.id == "gh").unwrap();
        assert_eq!(gh.status, CheckStatus::Ready);
    }

    #[tokio::test]
    async fn missing_git_produces_repair_hint() {
        let doctor = Doctor::with_dependencies(None, None, None);
        let report = doctor.run().await;
        let git = report.checks.iter().find(|c| c.id == "git").unwrap();
        assert_eq!(git.status, CheckStatus::Missing);
        assert!(git.repair_hint.is_some());
        assert!(!report.healthy);
    }

    #[test]
    fn path_comparison_is_case_and_separator_insensitive() {
        assert_eq!(
            normalized_path(std::path::Path::new(r"C:\Program Files\Shehata Git\")),
            normalized_path(std::path::Path::new(r"c:\program files\shehata git")),
        );
    }
}
