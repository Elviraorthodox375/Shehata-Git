//! `shehata` — safe command-line access to the same core used by the desktop.
//!
//! Human-readable output is the default; `--json` produces stable structured
//! output. Credentials never cross this process boundary or reach stdout.

use std::path::PathBuf;
use std::process::{ExitCode, Stdio};

use clap::{Parser, Subcommand};
use serde::Serialize;
use shehata_core::{
    accounts as core_accounts, actions as core_actions, assignment as core_assignment,
    repositories as core_repositories, routing as core_routing, Doctor, ShehataError,
};
use shehata_github::GhRunner;
use shehata_storage::Database;

const EXIT_FAILURE: u8 = 1;
const EXIT_UNHEALTHY: u8 = 4;

#[derive(Parser)]
#[command(
    name = "shehata",
    version,
    about = "Shehata Git — one repo, one identity, zero switching.",
    long_about = None
)]
struct Cli {
    /// Machine-readable JSON output.
    #[arg(long, global = true)]
    json: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Check prerequisites and show simple repair guidance.
    Doctor,
    /// Manage GitHub accounts through the official GitHub CLI.
    #[command(subcommand)]
    Accounts(AccountsCommands),
    /// Manage registered repositories and their identity route.
    #[command(subcommand)]
    Repos(ReposCommands),
    /// Show working-tree status for a registered repository.
    Status {
        /// Repository path (defaults to the current directory).
        path: Option<String>,
    },
    /// Test the assigned account using non-mutating `git ls-remote`.
    Test {
        /// Repository path (defaults to the current directory).
        path: Option<String>,
    },
    /// Perform a normal push after full preflight. Force is never available.
    Push {
        /// Repository path (defaults to the current directory).
        path: Option<String>,
        /// Approve a repository whose policy is `ask_before_push`.
        #[arg(long)]
        yes: bool,
    },
    /// Start the native Shehata MCP server on stdio.
    Mcp,
}

#[derive(Subcommand)]
enum AccountsCommands {
    /// List authenticated GitHub CLI accounts.
    List,
    /// Re-check accounts and refresh the safe local mirror.
    Refresh,
}

#[derive(Subcommand)]
enum ReposCommands {
    /// List registered repositories.
    List,
    /// Inspect and register a local Git worktree.
    Add { path: String },
    /// Show one repository by path, path-inside-worktree, or UUID.
    Show { path_or_id: Option<String> },
    /// Assign an account and enable repository-scoped credential routing.
    Assign {
        path_or_id: String,
        #[arg(long)]
        account: String,
        #[arg(long, default_value = "github.com")]
        host: String,
    },
    /// Unlink and restore original local Git configuration and identity.
    Unlink { path_or_id: String },
}

#[derive(Serialize)]
struct AssignmentOutput {
    assignment: core_assignment::AssignmentResult,
    routing: core_routing::RoutingResult,
}

#[tokio::main]
async fn main() -> ExitCode {
    init_tracing();
    let cli = Cli::parse();
    let result = match cli.command {
        Commands::Doctor => cmd_doctor(cli.json).await,
        Commands::Accounts(AccountsCommands::List | AccountsCommands::Refresh) => {
            cmd_accounts_list(cli.json).await
        }
        Commands::Repos(ReposCommands::List) => cmd_repos_list(cli.json).await,
        Commands::Repos(ReposCommands::Add { path }) => cmd_repos_add(cli.json, &path).await,
        Commands::Repos(ReposCommands::Show { path_or_id }) => {
            cmd_repos_show(cli.json, path_or_id.as_deref()).await
        }
        Commands::Repos(ReposCommands::Assign {
            path_or_id,
            account,
            host,
        }) => cmd_repos_assign(cli.json, &path_or_id, &host, &account).await,
        Commands::Repos(ReposCommands::Unlink { path_or_id }) => {
            cmd_repos_unlink(cli.json, &path_or_id).await
        }
        Commands::Status { path } => cmd_status(cli.json, path.as_deref()).await,
        Commands::Test { path } => cmd_test(cli.json, path.as_deref()).await,
        Commands::Push { path, yes } => cmd_push(cli.json, path.as_deref(), yes).await,
        Commands::Mcp => cmd_mcp(cli.json).await,
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(code) => ExitCode::from(code),
    }
}

async fn cmd_doctor(json: bool) -> Result<(), u8> {
    let report = Doctor::new().run().await;
    if json {
        print_json(&report)?;
    } else {
        println!("Shehata Git system check ({})\n", safe_text(&report.os));
        for check in &report.checks {
            let symbol = match check.status {
                shehata_core::CheckStatus::Ready => "  OK  ",
                shehata_core::CheckStatus::Missing => " MISS ",
                shehata_core::CheckStatus::NeedsAttention => " WARN ",
            };
            println!("[{symbol}] {}", safe_text(&check.label));
            if let Some(version) = &check.version {
                println!("         {}", safe_text(version));
            }
            println!("         {}", safe_text(&check.detail));
            if let Some(hint) = &check.repair_hint {
                println!("         fix: {}", safe_text(hint));
            }
        }
        println!();
        println!(
            "{}",
            if report.healthy {
                "Everything Shehata Git needs is in place."
            } else {
                "Some checks need attention — see the fix lines above."
            }
        );
    }
    if report.healthy {
        Ok(())
    } else {
        Err(EXIT_UNHEALTHY)
    }
}

async fn cmd_accounts_list(json: bool) -> Result<(), u8> {
    let gh = GhRunner::locate().map_err(|error| fail_message(json, "github_cli_error", &error))?;
    let accounts = core_accounts::list_accounts(&gh)
        .await
        .map_err(|error| fail(json, &error))?;
    if let Ok(db) = Database::open_default() {
        core_accounts::mirror_accounts(&db, &accounts);
    }
    if json {
        print_json(&accounts)
    } else if accounts.is_empty() {
        println!("No GitHub accounts are signed in.");
        println!("Sign in from the desktop app or run: gh auth login");
        Ok(())
    } else {
        for account in &accounts {
            println!(
                "@{} on {}{} — {}",
                safe_text(&account.login),
                safe_text(&account.host),
                if account.active {
                    " (active in gh)"
                } else {
                    ""
                },
                if account.token_available {
                    "token available"
                } else {
                    "TOKEN UNAVAILABLE"
                }
            );
        }
        Ok(())
    }
}

async fn cmd_repos_list(json: bool) -> Result<(), u8> {
    let repositories = core_repositories::list_repository_summaries_with_routing()
        .await
        .map_err(|error| fail(json, &error))?;
    if json {
        print_json(&repositories)
    } else if repositories.is_empty() {
        println!("No repositories registered yet.");
        Ok(())
    } else {
        for repository in repositories {
            let state = if repository.routing_configured {
                "routed"
            } else if repository.assigned_login.is_some() {
                "assigned"
            } else {
                "NO ACCOUNT"
            };
            println!(
                "{} — {} [{}]",
                safe_text(&repository.display_name),
                safe_text(&repository.canonical_path),
                state
            );
        }
        Ok(())
    }
}

async fn cmd_repos_add(json: bool, path: &str) -> Result<(), u8> {
    let discovered = core_repositories::discover_selected_repository(path)
        .await
        .map_err(|error| fail(json, &error))?;
    let db =
        Database::open_default().map_err(|error| fail_message(json, "storage_error", &error))?;
    let record = core_repositories::save_discovered_repository(&db, &discovered)
        .map_err(|error| fail(json, &error))?;
    let summary =
        core_repositories::repository_summary(&db, record).map_err(|error| fail(json, &error))?;
    if json {
        print_json(&summary)
    } else {
        println!("Registered {}.", safe_text(&summary.display_name));
        println!("Path: {}", safe_text(&summary.canonical_path));
        println!("Next: assign an account with `shehata repos assign`. ");
        Ok(())
    }
}

async fn cmd_repos_show(json: bool, reference: Option<&str>) -> Result<(), u8> {
    let repository = core_repositories::resolve_repository_reference(reference)
        .await
        .map_err(|error| fail(json, &error))?;
    let summary = core_repositories::list_repository_summaries_with_routing()
        .await
        .map_err(|error| fail(json, &error))?
        .into_iter()
        .find(|summary| summary.id == repository.id)
        .ok_or_else(|| {
            fail_message_text(
                json,
                "repository_not_found",
                "repository disappeared while it was being read",
            )
        })?;
    if json {
        print_json(&summary)
    } else {
        println!("{}", safe_text(&summary.display_name));
        println!("  id: {}", safe_text(&summary.id));
        println!("  path: {}", safe_text(&summary.canonical_path));
        println!(
            "  remote: {}",
            safe_text(summary.remote_url.as_deref().unwrap_or("not configured"))
        );
        println!(
            "  account: {}",
            safe_text(summary.assigned_login.as_deref().unwrap_or("not assigned"))
        );
        println!(
            "  routing: {}",
            if summary.routing_configured {
                "configured"
            } else {
                "not configured"
            }
        );
        println!("  push policy: {}", safe_text(&summary.push_policy));
        Ok(())
    }
}

async fn cmd_repos_assign(json: bool, reference: &str, host: &str, login: &str) -> Result<(), u8> {
    let repository = core_repositories::resolve_repository_reference(Some(reference))
        .await
        .map_err(|error| fail(json, &error))?;
    let assignment = core_assignment::assign_repository(core_assignment::AssignRepositoryRequest {
        repository_id: repository.id.clone(),
        host: host.to_string(),
        login: login.to_string(),
        commit_name: None,
        commit_email: None,
    })
    .await
    .map_err(|error| fail(json, &error))?;
    let routing = core_routing::link_repository(core_routing::LinkRepositoryRequest {
        repository_id: repository.id,
    })
    .await
    .map_err(|error| fail(json, &error))?;
    let output = AssignmentOutput {
        assignment,
        routing,
    };
    if json {
        print_json(&output)
    } else {
        println!(
            "Assigned @{} and enabled credential routing for {}.",
            safe_text(login),
            safe_text(&output.assignment.repository.display_name)
        );
        Ok(())
    }
}

async fn cmd_repos_unlink(json: bool, reference: &str) -> Result<(), u8> {
    let repository = core_repositories::resolve_repository_reference(Some(reference))
        .await
        .map_err(|error| fail(json, &error))?;
    let result = core_routing::unlink_repository(core_routing::UnlinkRepositoryRequest {
        repository_id: repository.id,
        restore_identity: true,
    })
    .await
    .map_err(|error| fail(json, &error))?;
    if json {
        print_json(&result)
    } else {
        println!("Repository unlinked; original local Git configuration was restored.");
        Ok(())
    }
}

async fn cmd_status(json: bool, reference: Option<&str>) -> Result<(), u8> {
    let repository = core_repositories::resolve_repository_reference(reference)
        .await
        .map_err(|error| fail(json, &error))?;
    let status = core_actions::status(&repository.id)
        .await
        .map_err(|error| fail(json, &error))?;
    if json {
        print_json(&status)
    } else {
        println!(
            "Branch: {}{}",
            safe_text(status.branch.as_deref().unwrap_or("none")),
            if status.detached_head {
                " (detached)"
            } else {
                ""
            }
        );
        if status.changes.is_empty() {
            println!("Working tree clean.");
        } else {
            for change in status.changes {
                println!(
                    "{}{} {}",
                    safe_text(&change.index_status),
                    safe_text(&change.worktree_status),
                    safe_text(&change.path)
                );
            }
        }
        Ok(())
    }
}

async fn cmd_test(json: bool, reference: Option<&str>) -> Result<(), u8> {
    let repository = core_repositories::resolve_repository_reference(reference)
        .await
        .map_err(|error| fail(json, &error))?;
    let result = core_routing::test_connection(&repository.id)
        .await
        .map_err(|error| fail(json, &error))?;
    if json {
        print_json(&result)
    } else {
        println!(
            "Connection verified through @{} on {}.",
            safe_text(&result.account_login),
            safe_text(&result.remote_name)
        );
        Ok(())
    }
}

async fn cmd_push(json: bool, reference: Option<&str>, approved: bool) -> Result<(), u8> {
    let repository = core_repositories::resolve_repository_reference(reference)
        .await
        .map_err(|error| fail(json, &error))?;
    let result = core_actions::push(core_actions::PushRequest {
        repository_id: repository.id,
        caller: core_actions::ActionCaller::Cli,
        approved,
    })
    .await
    .map_err(|error| fail(json, &error))?;
    if json {
        print_json(&result)
    } else {
        println!(
            "Normal push completed: {}/{} through @{}.",
            safe_text(&result.remote_name),
            safe_text(&result.branch),
            safe_text(&result.account_login)
        );
        Ok(())
    }
}

async fn cmd_mcp(json: bool) -> Result<(), u8> {
    let executable = locate_mcp_binary().ok_or_else(|| {
        fail_message_text(
            json,
            "mcp_not_found",
            "shehata-mcp was not found beside the CLI or on PATH",
        )
    })?;
    let status = tokio::process::Command::new(executable)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .await
        .map_err(|error| fail_message(json, "mcp_spawn_error", &error))?;
    if status.success() {
        Ok(())
    } else {
        Err(status.code().unwrap_or(EXIT_FAILURE.into()).clamp(1, 255) as u8)
    }
}

fn locate_mcp_binary() -> Option<PathBuf> {
    let filename = if cfg!(windows) {
        "shehata-mcp.exe"
    } else {
        "shehata-mcp"
    };
    if let Ok(current) = std::env::current_exe() {
        if let Some(directory) = current.parent() {
            let sibling = directory.join(filename);
            if sibling.is_file() {
                return Some(sibling);
            }
        }
    }
    which::which("shehata-mcp").ok()
}

fn print_json<T: Serialize>(value: &T) -> Result<(), u8> {
    let text = serde_json::to_string_pretty(value).map_err(|_| EXIT_FAILURE)?;
    println!("{text}");
    Ok(())
}

fn fail(json: bool, error: &ShehataError) -> u8 {
    if json {
        println!(
            "{}",
            serde_json::json!({ "error": { "code": error.code(), "message": error.to_string() } })
        );
    } else {
        eprintln!(
            "error [{}]: {}",
            error.code(),
            safe_text(&error.to_string())
        );
        if matches!(error, ShehataError::ApprovalRequired) {
            eprintln!("repair: review the push, then rerun with --yes");
        }
    }
    EXIT_FAILURE
}

fn fail_message(json: bool, code: &str, error: &impl std::fmt::Display) -> u8 {
    fail_message_text(json, code, &error.to_string())
}

fn fail_message_text(json: bool, code: &str, message: &str) -> u8 {
    if json {
        println!(
            "{}",
            serde_json::json!({ "error": { "code": code, "message": message } })
        );
    } else {
        eprintln!("error [{}]: {}", safe_text(code), safe_text(message));
    }
    EXIT_FAILURE
}

fn safe_text(value: &str) -> String {
    let mut safe = String::with_capacity(value.len());
    for character in value.chars() {
        if character.is_control() {
            safe.extend(character.escape_default());
        } else {
            safe.push(character);
        }
    }
    safe
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cli_shape_is_stable() {
        use clap::CommandFactory;
        Cli::command().debug_assert();
    }

    #[test]
    fn terminal_text_escapes_control_characters() {
        assert_eq!(safe_text("ok\u{1b}[31m"), "ok\\u{1b}[31m");
    }
}
