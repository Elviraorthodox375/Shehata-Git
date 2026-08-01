//! Tauri bridge.
//!
//! Command handlers here are THIN: they call shehata-core and serialize the
//! result. No business logic lives in this crate, and no secret value ever
//! crosses to the frontend.

use serde::Serialize;
use shehata_core::{accounts as core_accounts, Doctor};
use shehata_github::{GhLoginEvent, GhRunner};
use shehata_storage::{queries, Database};

/// Frontend-facing repository summary (joined with the assigned login).
#[derive(Debug, Serialize)]
struct RepositorySummary {
    id: String,
    display_name: String,
    canonical_path: String,
    host: Option<String>,
    owner: Option<String>,
    repo_name: Option<String>,
    current_branch: Option<String>,
    assigned_login: Option<String>,
    push_policy: String,
}

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
fn repositories_list() -> Result<Vec<RepositorySummary>, String> {
    let db = open_db()?;
    let repos = queries::list_repositories(&db)
        .map_err(|e| shehata_core::redact::redact_github_tokens(&e.to_string()))?;
    let mut out = Vec::with_capacity(repos.len());
    for repo in repos {
        let assigned_login = repo
            .assigned_account_id
            .and_then(|id| queries::find_account_by_id(&db, id).ok().flatten())
            .map(|a| a.login);
        out.push(RepositorySummary {
            id: repo.id,
            display_name: repo.display_name,
            canonical_path: repo.canonical_path,
            host: repo.host,
            owner: repo.owner,
            repo_name: repo.repo_name,
            current_branch: repo.current_branch,
            assigned_login,
            push_policy: repo.push_policy,
        });
    }
    Ok(out)
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
