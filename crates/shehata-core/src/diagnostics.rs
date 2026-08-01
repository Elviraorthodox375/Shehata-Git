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
    pub ai_clients: Vec<AiClientInfo>,
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
        ai_clients: detect_ai_clients(),
    })
}
