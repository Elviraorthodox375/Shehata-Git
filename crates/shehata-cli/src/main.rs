//! shehata — the Shehata Git CLI.
//!
//! Exit codes:
//!   0  success
//!   1  operation failed
//!   2  usage error (clap)
//!   3  not implemented yet (tracked, stable)
//!   4  doctor found a missing prerequisite
//!
//! Human-readable output by default; `--json` for agents and automation.
//! No command ever prints a token.

use std::process::ExitCode;

use clap::{Parser, Subcommand};
use shehata_core::{accounts as core_accounts, Doctor};
use shehata_github::GhRunner;
use shehata_storage::{queries, Database};

const EXIT_FAILURE: u8 = 1;
const EXIT_NOT_IMPLEMENTED: u8 = 3;
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
    /// Check everything Shehata Git needs and how to fix what is missing.
    Doctor,

    /// Manage GitHub accounts (via the GitHub CLI).
    #[command(subcommand)]
    Accounts(AccountsCommands),

    /// Manage linked repositories.
    #[command(subcommand)]
    Repos(ReposCommands),

    /// Show git status of a linked repository. [Phase 7]
    Status {
        /// Repository path (defaults to current directory).
        path: Option<String>,
    },

    /// Test the connection for a linked repository without changing anything. [Phase 6]
    Test {
        /// Repository path (defaults to current directory).
        path: Option<String>,
    },

    /// Push with full preflight checks through the assigned account. [Phase 7]
    Push {
        /// Repository path (defaults to current directory).
        path: Option<String>,
    },

    /// Start the MCP server on stdio. [Phase 9]
    Mcp,
}

#[derive(Subcommand)]
enum AccountsCommands {
    /// List all GitHub accounts authenticated in the GitHub CLI.
    List,
    /// Re-check accounts and token availability.
    Refresh,
}

#[derive(Subcommand)]
enum ReposCommands {
    /// List linked repositories.
    List,
    /// Link a local repository. [Phase 4]
    Add {
        /// Path to the repository folder.
        path: String,
    },
    /// Show details of a linked repository. [Phase 4]
    Show {
        /// Repository path or id.
        path_or_id: Option<String>,
    },
    /// Assign a GitHub account to a repository. [Phase 5]
    Assign {
        /// Repository path or id.
        path_or_id: String,
        /// Account login.
        #[arg(long)]
        account: String,
        /// Host (default: github.com).
        #[arg(long, default_value = "github.com")]
        host: String,
    },
    /// Unlink a repository and restore its previous git configuration. [Phase 6]
    Unlink {
        /// Repository path or id.
        path_or_id: String,
    },
}

#[tokio::main]
async fn main() -> ExitCode {
    init_tracing();
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Doctor => cmd_doctor(cli.json).await,
        Commands::Accounts(AccountsCommands::List) => cmd_accounts_list(cli.json).await,
        Commands::Accounts(AccountsCommands::Refresh) => cmd_accounts_list(cli.json).await,
        Commands::Repos(ReposCommands::List) => cmd_repos_list(cli.json),
        Commands::Repos(ReposCommands::Add { .. })
        | Commands::Repos(ReposCommands::Show { .. })
        | Commands::Repos(ReposCommands::Assign { .. })
        | Commands::Repos(ReposCommands::Unlink { .. })
        | Commands::Status { .. }
        | Commands::Test { .. }
        | Commands::Push { .. }
        | Commands::Mcp => not_implemented(cli.json),
    };

    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(code) => ExitCode::from(code),
    }
}

async fn cmd_doctor(json: bool) -> Result<(), u8> {
    let report = Doctor::new().run().await;
    if json {
        let text = serde_json::to_string_pretty(&report).map_err(|_| EXIT_FAILURE)?;
        println!("{text}");
    } else {
        println!("Shehata Git system check ({})\n", report.os);
        for check in &report.checks {
            let symbol = match check.status {
                shehata_core::CheckStatus::Ready => "  OK  ",
                shehata_core::CheckStatus::Missing => " MISS ",
                shehata_core::CheckStatus::NeedsAttention => " WARN ",
            };
            println!("[{symbol}] {}", check.label);
            if let Some(version) = &check.version {
                println!("         {version}");
            }
            println!("         {}", check.detail);
            if let Some(hint) = &check.repair_hint {
                println!("         fix: {hint}");
            }
        }
        println!();
        if report.healthy {
            println!("Everything Shehata Git needs is in place.");
        } else {
            println!("Some checks need attention — see the fix lines above.");
        }
    }
    if report.healthy {
        Ok(())
    } else {
        Err(EXIT_UNHEALTHY)
    }
}

async fn cmd_accounts_list(json: bool) -> Result<(), u8> {
    let gh = GhRunner::locate().map_err(|e| {
        eprintln!("error: {e}");
        EXIT_FAILURE
    })?;
    let accounts = core_accounts::list_accounts(&gh).await.map_err(|e| {
        eprintln!("error: {e}");
        EXIT_FAILURE
    })?;
    if let Ok(db) = Database::open_default() {
        core_accounts::mirror_accounts(&db, &accounts);
    }

    if json {
        let text = serde_json::to_string_pretty(&accounts).map_err(|_| EXIT_FAILURE)?;
        println!("{text}");
    } else if accounts.is_empty() {
        println!("No GitHub accounts are signed in.");
        println!("Sign in from the Shehata Git app (Accounts page) or run: gh auth login");
    } else {
        for account in &accounts {
            let active = if account.active {
                " (active in gh)"
            } else {
                ""
            };
            let token = if account.token_available {
                "token ok"
            } else {
                "TOKEN UNAVAILABLE"
            };
            println!(
                "@{} on {}{} — {}",
                account.login, account.host, active, token
            );
        }
    }
    Ok(())
}

fn cmd_repos_list(json: bool) -> Result<(), u8> {
    let db = Database::open_default().map_err(|e| {
        eprintln!("error: {e}");
        EXIT_FAILURE
    })?;
    let repos = queries::list_repositories(&db).map_err(|e| {
        eprintln!("error: {e}");
        EXIT_FAILURE
    })?;

    if json {
        let text = serde_json::to_string_pretty(&repos).map_err(|_| EXIT_FAILURE)?;
        println!("{text}");
    } else if repos.is_empty() {
        println!("No repositories linked yet.");
        println!("Link one from the Shehata Git app (Repositories page).");
    } else {
        for repo in &repos {
            let assigned = match repo.assigned_account_id {
                Some(_) => "assigned",
                None => "NO ACCOUNT",
            };
            println!(
                "{} — {} [{}]",
                repo.display_name, repo.canonical_path, assigned
            );
        }
    }
    Ok(())
}

fn not_implemented(json: bool) -> Result<(), u8> {
    let message = "this command is part of an upcoming milestone and is not implemented yet";
    if json {
        println!(
            "{}",
            serde_json::json!({ "error": { "code": "not_implemented", "message": message } })
        );
    } else {
        eprintln!("not implemented: {message}");
        eprintln!("See docs/ROADMAP.md for the milestone plan.");
    }
    Err(EXIT_NOT_IMPLEMENTED)
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
