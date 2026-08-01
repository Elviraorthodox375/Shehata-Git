//! Safe, allowlisted prerequisite installation for the desktop setup flow.
//!
//! The frontend sends enum values, never command text or package identifiers.
//! WinGet is invoked directly with a fixed argument array and raw installer
//! output never crosses the core boundary.

use std::collections::HashSet;
use std::process::Stdio;
use std::time::Duration;

use serde::{Deserialize, Serialize};
use tokio::process::Command;

use crate::error::{Result, ShehataError};

const INSTALL_TIMEOUT: Duration = Duration::from_secs(15 * 60);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PrerequisiteId {
    Git,
    GithubCli,
}

impl PrerequisiteId {
    fn label(self) -> &'static str {
        match self {
            Self::Git => "Git",
            Self::GithubCli => "GitHub CLI",
        }
    }

    fn package_id(self) -> &'static str {
        match self {
            Self::Git => "Git.Git",
            Self::GithubCli => "GitHub.cli",
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct InstallPrerequisitesRequest {
    pub ids: Vec<PrerequisiteId>,
}

#[derive(Debug, Clone, Serialize)]
pub struct InstalledPrerequisite {
    pub id: PrerequisiteId,
    pub label: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct InstallPrerequisitesResult {
    pub installed: Vec<InstalledPrerequisite>,
}

pub async fn install_prerequisites(
    request: InstallPrerequisitesRequest,
) -> Result<InstallPrerequisitesResult> {
    if request.ids.is_empty() {
        return Err(ShehataError::InvalidInput(
            "choose at least one supported prerequisite".to_string(),
        ));
    }

    let mut unique = HashSet::new();
    let ids: Vec<_> = request
        .ids
        .into_iter()
        .filter(|id| unique.insert(*id))
        .collect();

    #[cfg(not(target_os = "windows"))]
    {
        let _ = ids;
        return Err(ShehataError::OperationBlocked(
            "automatic prerequisite setup is currently available on Windows only".to_string(),
        ));
    }

    #[cfg(target_os = "windows")]
    {
        let winget = which::which("winget").map_err(|_| ShehataError::PackageManagerMissing)?;
        let mut installed = Vec::with_capacity(ids.len());

        for id in ids {
            let status = tokio::time::timeout(
                INSTALL_TIMEOUT,
                Command::new(&winget)
                    .args(installation_args(id))
                    .stdin(Stdio::null())
                    .stdout(Stdio::null())
                    .stderr(Stdio::null())
                    .status(),
            )
            .await
            .map_err(|_| ShehataError::PrerequisiteInstallFailed {
                tool: id.label().to_string(),
                code: -1,
            })?
            .map_err(|_| ShehataError::PrerequisiteInstallFailed {
                tool: id.label().to_string(),
                code: -1,
            })?;

            if !status.success() {
                return Err(ShehataError::PrerequisiteInstallFailed {
                    tool: id.label().to_string(),
                    code: status.code().unwrap_or(-1),
                });
            }

            installed.push(InstalledPrerequisite {
                id,
                label: id.label().to_string(),
            });
        }

        refresh_windows_process_path();
        Ok(InstallPrerequisitesResult { installed })
    }
}

fn installation_args(id: PrerequisiteId) -> [&'static str; 11] {
    [
        "install",
        "--id",
        id.package_id(),
        "--exact",
        "--source",
        "winget",
        "--silent",
        "--accept-package-agreements",
        "--accept-source-agreements",
        "--disable-interactivity",
        "--no-upgrade",
    ]
}

#[cfg(target_os = "windows")]
fn refresh_windows_process_path() {
    use std::ffi::OsString;
    use std::path::PathBuf;
    use winreg::enums::{HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE};
    use winreg::RegKey;

    let mut values = vec![std::env::var_os("PATH").unwrap_or_default()];
    let user = RegKey::predef(HKEY_CURRENT_USER);
    if let Ok(environment) = user.open_subkey("Environment") {
        if let Ok(path) = environment.get_value::<String, _>("Path") {
            values.push(OsString::from(path));
        }
    }
    let machine = RegKey::predef(HKEY_LOCAL_MACHINE);
    if let Ok(environment) =
        machine.open_subkey(r"SYSTEM\CurrentControlSet\Control\Session Manager\Environment")
    {
        if let Ok(path) = environment.get_value::<String, _>("Path") {
            values.push(OsString::from(path));
        }
    }

    let mut seen = HashSet::new();
    let paths: Vec<PathBuf> = values
        .iter()
        .flat_map(std::env::split_paths)
        .filter(|path| seen.insert(path.to_string_lossy().to_lowercase()))
        .collect();
    if let Ok(path) = std::env::join_paths(paths) {
        std::env::set_var("PATH", path);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uses_only_exact_reviewed_winget_packages() {
        let git = installation_args(PrerequisiteId::Git);
        let gh = installation_args(PrerequisiteId::GithubCli);

        assert_eq!(git[2], "Git.Git");
        assert_eq!(gh[2], "GitHub.cli");
        for args in [git, gh] {
            assert!(args.contains(&"--exact"));
            assert!(args.contains(&"--source"));
            assert!(args.contains(&"--disable-interactivity"));
            assert!(!args.iter().any(|arg| arg.contains(['&', '|', ';'])));
        }
    }
}
