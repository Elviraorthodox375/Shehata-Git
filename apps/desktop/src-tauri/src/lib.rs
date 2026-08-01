//! Tauri bridge.
//!
//! Command handlers here are THIN: they call shehata-core and serialize the
//! result. No business logic lives in this crate, and no secret value ever
//! crosses to the frontend.

use serde::Serialize;
use shehata_core::{accounts as core_accounts, repositories as core_repositories, Doctor};
use shehata_github::{GhLoginEvent, GhRunner};
use shehata_storage::{queries, Database};

#[derive(Debug, Serialize)]
struct McpInfo {
    executable_path: Option<String>,
    available: bool,
    config_snippet: String,
}

fn open_db() -> Result<Database, String> {
    Database::open_default().map_err(|e| shehata_core::redact::redact_github_tokens(&e.to_string()))
}

#[tauri::command]
async fn doctor_run() -> Result<shehata_core::DoctorReport, String> {
    Ok(Doctor::new().run().await)
}

#[tauri::command]
async fn accounts_list() -> Result<Vec<shehata_core::AccountInfo>, String> {
    let gh = GhRunner::locate()
        .map_err(|e| shehata_core::redact::redact_github_tokens(&e.to_string()))?;
    let accounts = core_accounts::list_accounts(&gh)
        .await
        .map_err(|e| shehata_core::redact::redact_github_tokens(&e.to_string()))?;
    if let Ok(db) = open_db() {
        core_accounts::mirror_accounts(&db, &accounts);
    }
    Ok(accounts)
}

#[tauri::command]
async fn accounts_add(
    on_event: tauri::ipc::Channel<GhLoginEvent>,
) -> Result<Vec<shehata_core::AccountInfo>, String> {
    let gh = GhRunner::locate()
        .map_err(|e| shehata_core::redact::redact_github_tokens(&e.to_string()))?;
    let progress = on_event.clone();
    gh.login_web(move |event| {
        // The window may close while gh is waiting. A disconnected progress
        // channel must not turn a successful browser login into a failure.
        let _ = progress.send(event);
    })
    .await
    .map_err(|e| shehata_core::redact::redact_github_tokens(&e.to_string()))?;

    let accounts = core_accounts::list_accounts(&gh)
        .await
        .map_err(|e| shehata_core::redact::redact_github_tokens(&e.to_string()))?;
    if let Ok(db) = open_db() {
        core_accounts::mirror_accounts(&db, &accounts);
    }
    Ok(accounts)
}

#[tauri::command]
fn repositories_list() -> Result<Vec<core_repositories::RepositorySummary>, String> {
    let db = open_db()?;
    core_repositories::list_repository_summaries(&db)
        .map_err(|e| shehata_core::redact::redact_github_tokens(&e.to_string()))
}

#[tauri::command]
async fn repositories_add(path: String) -> Result<core_repositories::RepositorySummary, String> {
    let discovered = core_repositories::discover_selected_repository(&path)
        .await
        .map_err(|e| shehata_core::redact::redact_github_tokens(&e.to_string()))?;
    let db = open_db()?;
    let saved = core_repositories::save_discovered_repository(&db, &discovered)
        .map_err(|e| shehata_core::redact::redact_github_tokens(&e.to_string()))?;
    core_repositories::repository_summary(&db, saved)
        .map_err(|e| shehata_core::redact::redact_github_tokens(&e.to_string()))
}

#[tauri::command]
fn audit_list() -> Result<Vec<shehata_storage::AuditEventRecord>, String> {
    let db = open_db()?;
    queries::list_audit_events(&db, 200)
        .map_err(|e| shehata_core::redact::redact_github_tokens(&e.to_string()))
}

#[tauri::command]
fn mcp_info() -> McpInfo {
    let exe = locate_mcp_binary();
    let command_path = exe
        .as_ref()
        .map(|p| p.display().to_string())
        .unwrap_or_else(|| "shehata-mcp".to_string());
    // Escape backslashes for JSON embedding.
    let escaped = command_path.replace('\\', "\\\\");
    let config_snippet = format!(
        "{{\n  \"mcpServers\": {{\n    \"shehata-git\": {{\n      \"command\": \"{escaped}\"\n    }}\n  }}\n}}"
    );
    McpInfo {
        available: exe.is_some(),
        executable_path: exe.map(|p| p.display().to_string()),
        config_snippet,
    }
}

fn locate_mcp_binary() -> Option<std::path::PathBuf> {
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            let candidate = dir.join("shehata-mcp.exe");
            if candidate.exists() {
                return Some(candidate);
            }
        }
    }
    which::which("shehata-mcp").ok()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    init_tracing();
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            doctor_run,
            accounts_list,
            accounts_add,
            repositories_list,
            repositories_add,
            audit_list,
            mcp_info,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Shehata Git");
}

fn init_tracing() {
    let filter = tracing_subscriber::EnvFilter::try_from_env("SHEHATA_LOG")
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("warn"));
    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_writer(std::io::stderr)
        .with_ansi(false)
        .init();
}
