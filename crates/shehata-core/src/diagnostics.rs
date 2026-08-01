//! Credential-free diagnostic report suitable for copying into a support issue.

use chrono::Utc;
use serde::Serialize;

use crate::integrations::{detect_ai_clients, AiClientInfo};
use crate::repositories::list_repository_summaries_with_routing;
use crate::{CheckStatus, Doctor, Result};

#[derive(Debug, Clone, Serialize)]
pub struct DiagnosticCheck {
    pub id: String,
    pub status: String,
    pub version: Option<String>,
}

/// Availability-only view used by the copyable support report. Executable
/// paths are intentionally excluded because they commonly contain usernames.
#[derive(Debug, Clone, Serialize)]
pub struct DiagnosticAiClient {
    pub id: String,
    pub name: String,
    pub available: bool,
}

impl From<AiClientInfo> for DiagnosticAiClient {
    fn from(client: AiClientInfo) -> Self {
        Self {
            id: client.id,
            name: client.name,
            available: client.available,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct SafeDiagnosticReport {
    pub generated_at: String,
    pub app_version: String,
    pub os: String,
    pub healthy: bool,
    pub checks: Vec<DiagnosticCheck>,
    pub repository_count: usize,
    pub assigned_repository_count: usize,
    pub routed_repository_count: usize,
    pub ai_clients: Vec<DiagnosticAiClient>,
}

pub async fn safe_diagnostic_report() -> Result<SafeDiagnosticReport> {
    let doctor = Doctor::new().run().await;
    let repositories = list_repository_summaries_with_routing()
        .await
        .unwrap_or_default();
    Ok(SafeDiagnosticReport {
        generated_at: Utc::now().to_rfc3339(),
        app_version: doctor.app_version,
        os: doctor.os,
        healthy: doctor.healthy,
        checks: doctor
            .checks
            .into_iter()
            .map(|check| DiagnosticCheck {
                id: check.id,
                status: match check.status {
                    CheckStatus::Ready => "ready",
                    CheckStatus::Missing => "missing",
                    CheckStatus::NeedsAttention => "needs_attention",
                }
                .to_string(),
                version: check.version,
            })
            .collect(),
        repository_count: repositories.len(),
        assigned_repository_count: repositories
            .iter()
            .filter(|repository| repository.assigned_login.is_some())
            .count(),
        routed_repository_count: repositories
            .iter()
            .filter(|repository| repository.routing_configured)
            .count(),
        ai_clients: detect_ai_clients().into_iter().map(Into::into).collect(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn support_client_metadata_omits_executable_paths() {
        let safe = DiagnosticAiClient::from(AiClientInfo {
            id: "codex".to_string(),
            name: "Codex".to_string(),
            available: true,
            executable_path: Some(r"C:\Users\private-name\bin\codex.exe".to_string()),
        });
        let serialized = serde_json::to_string(&safe).unwrap();

        assert!(!serialized.contains("private-name"));
        assert!(!serialized.contains("executable_path"));
    }
}
