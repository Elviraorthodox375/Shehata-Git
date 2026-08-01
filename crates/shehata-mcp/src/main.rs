//! shehata-mcp — stdio MCP server.
//!
//! Exposes safe, structured Shehata Git tools to AI coding agents.
//!
//! Hard rules enforced here:
//! - No arbitrary shell execution tool. Ever.
//! - No force push, remote deletion, reset --hard, or clean.
//! - No tokens in any tool result — credentials never cross this boundary.
//! - Every result is a structured envelope: { ok, code, summary, data }.
//!   `code` is stable and machine-readable; `summary` is for humans.

use rmcp::{
    handler::server::wrapper::Parameters, schemars, tool, tool_router, transport::stdio,
    ErrorData as McpError, Json, ServiceExt,
};
use serde::{Deserialize, Serialize};
use shehata_core::{accounts as core_accounts, Doctor, ShehataError};
use shehata_git::GitRunner;
use shehata_github::GhRunner;
use shehata_storage::{queries, Database};

// ------------------------------------------------------------------ envelope

/// Uniform tool result envelope. `code` values match ShehataError::code().
#[derive(Debug, Serialize, schemars::JsonSchema)]
struct Envelope {
    ok: bool,
    code: String,
    summary: String,
    data: Option<serde_json::Value>,
}

impl Envelope {
    fn success<T: Serialize>(summary: impl Into<String>, data: T) -> Result<Self, McpError> {
        let data = serde_json::to_value(data)
            .map_err(|_| McpError::internal_error("could not serialize tool result", None))?;
        Ok(Self {
            ok: true,
            code: "ok".to_string(),
            summary: summary.into(),
            data: Some(data),
        })
    }

    fn failure(error: &ShehataError) -> Self {
        Self {
            ok: false,
            code: error.code().to_string(),
            summary: error.to_string(),
            data: None,
        }
    }

    fn not_implemented(feature: &str) -> Self {
        Self {
            ok: false,
            code: "not_implemented".to_string(),
            summary: format!(
                "{feature} is not available yet in this build. See docs/ROADMAP.md for the milestone plan."
            ),
            data: None,
        }
    }
}

// ---------------------------------------------------------------------- args

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct RepositoryArgs {
    /// The Shehata Git repository id (UUID). Provide this or `path`.
    #[serde(default)]
    repository_id: Option<String>,
    /// The repository's canonical path. Provide this or `repository_id`.
    #[serde(default)]
    path: Option<String>,
}

// --------------------------------------------------------------------- state

#[derive(Clone)]
struct ShehataMcp {
    db_path: Option<std::path::PathBuf>,
}

impl ShehataMcp {
    fn new() -> Self {
        Self {
            db_path: Database::default_path().ok(),
        }
    }

    fn open_db(&self) -> Result<Database, McpError> {
        let path = self.db_path.clone().ok_or_else(|| {
            McpError::internal_error("application data directory unavailable", None)
        })?;
        Database::open_at(&path).map_err(|e| McpError::internal_error(e.to_string(), None))
    }
}

// --------------------------------------------------------------------- tools

#[tool_router(server_handler)]
impl ShehataMcp {
    /// Run the full Shehata Git system check: git, GitHub CLI, accounts,
    /// database, credential helper, WebView, PATH, and this MCP server.
    #[tool(
        name = "shehata_git_doctor",
        description = "Check everything Shehata Git needs and how to fix what is missing"
    )]
    async fn doctor(&self) -> Result<Json<Envelope>, McpError> {
        let report = Doctor::new().run().await;
        let summary = if report.healthy {
            "system healthy".to_string()
        } else {
            let problems: Vec<&str> = report
                .checks
                .iter()
                .filter(|c| c.status != shehata_core::CheckStatus::Ready)
                .map(|c| c.label.as_str())
                .collect();
            format!("needs attention: {}", problems.join(", "))
        };
        Ok(Json(Envelope::success(summary, report)?))
    }

    /// List GitHub accounts authenticated in the GitHub CLI, with token
    /// availability per account. Never returns tokens.
    #[tool(
        name = "shehata_git_list_accounts",
        description = "List GitHub accounts available on this machine (no tokens, ever)"
    )]
    async fn list_accounts(&self) -> Result<Json<Envelope>, McpError> {
        let gh = match GhRunner::locate() {
            Ok(gh) => gh,
            Err(e) => {
                let err = ShehataError::Github(e);
                return Ok(Json(Envelope {
                    ok: false,
                    code: err.code().to_string(),
                    summary: err.to_string(),
                    data: Some(serde_json::json!([])),
                }));
            }
        };
        match core_accounts::list_accounts(&gh).await {
            Ok(accounts) => {
                if let Ok(db) = self.open_db() {
                    core_accounts::mirror_accounts(&db, &accounts);
                }
                let summary = format!("{} account(s) available", accounts.len());
                Ok(Json(Envelope::success(summary, accounts)?))
            }
            Err(e) => Ok(Json(Envelope {
                ok: false,
                code: e.code().to_string(),
                summary: e.to_string(),
                data: Some(serde_json::json!([])),
            })),
        }
    }

    /// List repositories linked to Shehata Git.
    #[tool(
        name = "shehata_git_list_repositories",
        description = "List repositories linked to Shehata Git with their assigned accounts"
    )]
    async fn list_repositories(&self) -> Result<Json<Envelope>, McpError> {
        let db = self.open_db()?;
        match queries::list_repositories(&db) {
            Ok(repos) => {
                let summary = format!("{} linked repositorie(s)", repos.len());
                Ok(Json(Envelope::success(summary, repos)?))
            }
            Err(e) => {
                let err = ShehataError::Storage(e);
                Ok(Json(Envelope {
                    ok: false,
                    code: err.code().to_string(),
                    summary: err.to_string(),
                    data: Some(serde_json::json!([])),
                }))
            }
        }
    }

    /// Get one linked repository by id or canonical path.
    #[tool(
        name = "shehata_git_get_repository",
        description = "Get one linked repository by Shehata id or canonical path"
    )]
    async fn get_repository(
        &self,
        Parameters(args): Parameters<RepositoryArgs>,
    ) -> Result<Json<Envelope>, McpError> {
        if args.repository_id.is_none() && args.path.is_none() {
            return Err(McpError::invalid_params(
                "provide repository_id or path",
                None,
            ));
        }
        let db = self.open_db()?;
        let repo = match (&args.repository_id, &args.path) {
            (Some(id), _) => queries::find_repository_by_id(&db, id),
            (None, Some(path)) => queries::find_repository_by_path(&db, path),
            (None, None) => unreachable!("validated above"),
        };
        match repo {
            Ok(Some(repo)) => Ok(Json(Envelope::success(
                format!("repository {}", repo.display_name),
                repo,
            )?)),
            Ok(None) => Ok(Json(Envelope::failure(&ShehataError::RepositoryNotFound(
                args.repository_id
                    .or(args.path)
                    .unwrap_or_else(|| "unknown".to_string()),
            )))),
            Err(e) => Ok(Json(Envelope::failure(&ShehataError::Storage(e)))),
        }
    }

    /// Read identity state for a linked repository: assigned account plus the
    /// repository-local git user.name / user.email. Read-only.
    #[tool(
        name = "shehata_git_check_identity",
        description = "Show which account and commit identity a linked repository will use"
    )]
    async fn check_identity(
        &self,
        Parameters(args): Parameters<RepositoryArgs>,
    ) -> Result<Json<Envelope>, McpError> {
        // Finish all SQLite work before awaiting git so the non-Sync
        // rusqlite connection never lives across an await point.
        let (repo, assigned_login) = {
            let db = self.open_db()?;
            let repo = match (&args.repository_id, &args.path) {
                (Some(id), _) => queries::find_repository_by_id(&db, id),
                (None, Some(path)) => queries::find_repository_by_path(&db, path),
                (None, None) => {
                    return Err(McpError::invalid_params(
                        "provide repository_id or path",
                        None,
                    ))
                }
            };
            let repo = match repo {
                Ok(Some(repo)) => repo,
                Ok(None) => {
                    return Ok(Json(Envelope::failure(&ShehataError::RepositoryNotFound(
                        args.repository_id
                            .or(args.path)
                            .unwrap_or_else(|| "unknown".to_string()),
                    ))))
                }
                Err(e) => return Ok(Json(Envelope::failure(&ShehataError::Storage(e)))),
            };

            let assigned_login = match repo.assigned_account_id {
                Some(id) => queries::find_account_by_id(&db, id)
                    .ok()
                    .flatten()
                    .map(|a| format!("{}:{}", a.host, a.login)),
                None => None,
            };
            (repo, assigned_login)
        };

        // Read repository-local identity via git (read-only).
        let mut local_name = None;
        let mut local_email = None;
        if let Ok(git) = GitRunner::locate() {
            let dir = std::path::Path::new(&repo.canonical_path);
            if let Ok(out) = git
                .run_in(Some(dir), &["config", "--local", "--get", "user.name"])
                .await
            {
                if out.success() {
                    local_name = Some(out.stdout.trim().to_string());
                }
            }
            if let Ok(out) = git
                .run_in(Some(dir), &["config", "--local", "--get", "user.email"])
                .await
            {
                if out.success() {
                    local_email = Some(out.stdout.trim().to_string());
                }
            }
        }

        let data = serde_json::json!({
            "repository": repo.display_name,
            "assigned_account": assigned_login,
            "local_user_name": local_name,
            "local_user_email": local_email,
            "push_policy": repo.push_policy,
        });
        let summary = match &assigned_login {
            Some(account) => format!("pushes authenticate as {account}"),
            None => "no account assigned yet".to_string(),
        };
        Ok(Json(Envelope::success(summary, data)?))
    }

    // ----- milestones below return honest not_implemented envelopes -----

    #[tool(
        name = "shehata_git_status",
        description = "Working-tree status of a linked repository [Phase 7]"
    )]
    async fn status(
        &self,
        Parameters(_args): Parameters<RepositoryArgs>,
    ) -> Result<Json<Envelope>, McpError> {
        Ok(Json(Envelope::not_implemented("git status")))
    }

    #[tool(
        name = "shehata_git_diff_summary",
        description = "Summary of uncommitted changes [Phase 7]"
    )]
    async fn diff_summary(
        &self,
        Parameters(_args): Parameters<RepositoryArgs>,
    ) -> Result<Json<Envelope>, McpError> {
        Ok(Json(Envelope::not_implemented("diff summary")))
    }

    #[tool(
        name = "shehata_git_test_connection",
        description = "Non-mutating connection test through the assigned account [Phase 6]"
    )]
    async fn test_connection(
        &self,
        Parameters(_args): Parameters<RepositoryArgs>,
    ) -> Result<Json<Envelope>, McpError> {
        Ok(Json(Envelope::not_implemented("connection test")))
    }

    #[tool(
        name = "shehata_git_commit",
        description = "Create a normal commit [Phase 7]"
    )]
    async fn commit(
        &self,
        Parameters(_args): Parameters<RepositoryArgs>,
    ) -> Result<Json<Envelope>, McpError> {
        Ok(Json(Envelope::not_implemented("commit")))
    }

    #[tool(
        name = "shehata_git_pull_ff_only",
        description = "Pull with --ff-only [Phase 7]"
    )]
    async fn pull_ff_only(
        &self,
        Parameters(_args): Parameters<RepositoryArgs>,
    ) -> Result<Json<Envelope>, McpError> {
        Ok(Json(Envelope::not_implemented("pull --ff-only")))
    }

    #[tool(
        name = "shehata_git_push",
        description = "Normal push with full preflight through the assigned account [Phase 7]"
    )]
    async fn push(
        &self,
        Parameters(_args): Parameters<RepositoryArgs>,
    ) -> Result<Json<Envelope>, McpError> {
        Ok(Json(Envelope::not_implemented("push")))
    }
}

// ---------------------------------------------------------------------- main

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // stderr logging only — stdout is the MCP transport.
    let filter = tracing_subscriber::EnvFilter::try_from_env("SHEHATA_LOG")
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("warn"));
    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_writer(std::io::stderr)
        .with_ansi(false)
        .init();

    let service = ShehataMcp::new().serve(stdio()).await?;
    service.waiting().await?;
    Ok(())
}
