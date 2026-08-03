// Copyright (c) 2026 Dr Mohamed Shehata. All rights reserved.
// Licensed under the MIT License. See LICENSE in the project root.

//! git-credential-shehata — Git credential helper.
//!
//! Invoked by git as: `git-credential-shehata --repo-id <uuid> <operation>`
//! (git appends the operation after the configured helper string).
//!
//! Contract:
//! - `get`: resolve the repository's assigned account and emit
//!   `username=` / `password=` on stdout. Nothing else ever touches stdout.
//! - `store`: no-op — the GitHub CLI remains the credential source of truth.
//! - `erase`: no-op — credential lifecycle belongs to `gh auth logout`.
//!
//! Fail-closed: any missing mapping, account, host mismatch, or token failure
//! means NO output and a nonzero exit. Diagnostics go to stderr only and
//! never contain the token.

mod protocol;

use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::Instant;

use secrecy::ExposeSecret;
use shehata_github::GhRunner;
use shehata_storage::{queries, Database, NewAuditEvent};

use protocol::{CredentialRequest, MAX_INPUT_BYTES};

fn main() -> ExitCode {
    init_tracing();
    let args: Vec<String> = std::env::args().skip(1).collect();

    let repo_id = match parse_repo_id(&args) {
        Some(id) => id,
        None => {
            eprintln!("shehata: missing or invalid --repo-id <uuid> argument");
            return ExitCode::from(2);
        }
    };

    let operation = args
        .iter()
        .find(|a| matches!(a.as_str(), "get" | "store" | "erase"));

    match operation.map(String::as_str) {
        Some("get") => match tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
        {
            Ok(rt) => rt.block_on(handle_get(&repo_id)),
            Err(_) => ExitCode::from(70),
        },
        Some("store") => {
            // No-op by design: gh stays the source of truth.
            ExitCode::SUCCESS
        }
        Some("erase") => {
            eprintln!("shehata: erase ignored — sign out with `gh auth logout` if needed");
            ExitCode::SUCCESS
        }
        _ => {
            eprintln!("shehata: expected one of: get, store, erase");
            ExitCode::from(2)
        }
    }
}

fn parse_repo_id(args: &[String]) -> Option<String> {
    let pos = args.iter().position(|a| a == "--repo-id")?;
    let value = args.get(pos + 1)?;
    // UUID shape only — this value is later used in a SQL parameter, but
    // validate anyway: defense in depth.
    uuid::Uuid::parse_str(value).ok()?;
    Some(value.clone())
}

async fn handle_get(repo_id: &str) -> ExitCode {
    let started = Instant::now();

    // 1. Read the request from stdin (capped).
    let mut buffer = Vec::with_capacity(1024);
    let mut stdin = std::io::stdin().lock();
    let mut chunk = [0u8; 512];
    loop {
        match stdin.read(&mut chunk) {
            Ok(0) => break,
            Ok(n) => {
                buffer.extend_from_slice(&chunk[..n]);
                if buffer.len() > MAX_INPUT_BYTES {
                    eprintln!("shehata: credential request too large");
                    return ExitCode::from(65);
                }
                // Git terminates the request with a blank line.
                if buffer.windows(2).any(|w| w == b"\n\n")
                    || buffer.windows(4).any(|w| w == b"\r\n\r\n")
                {
                    break;
                }
            }
            Err(_) => return ExitCode::from(74),
        }
    }
    let Ok(text) = String::from_utf8(buffer) else {
        eprintln!("shehata: credential request was not UTF-8");
        return ExitCode::from(65);
    };
    let request = CredentialRequest::parse(&text);
    if !request.is_supported() {
        eprintln!("shehata: unsupported credential request (need protocol=https and host)");
        return ExitCode::from(65);
    }
    let host = request.host.as_ref().expect("checked by is_supported");

    // 2. Open the database read-only.
    let db_path = std::env::var_os("SHEHATA_DB_PATH")
        .map(PathBuf::from)
        .or_else(|| Database::default_path().ok());
    let Some(db_path) = db_path else {
        eprintln!("shehata: could not locate application database");
        return ExitCode::from(66);
    };
    let Ok(db) = Database::open_read_only(&db_path) else {
        eprintln!("shehata: application database not found or unreadable");
        return ExitCode::from(66);
    };

    // 3. Resolve repository + assignment.
    let Ok(Some(repo)) = queries::find_repository_by_id(&db, repo_id) else {
        eprintln!("shehata: repository id not found in database");
        write_credential_audit(
            &db_path,
            repo_id,
            None,
            "Credential denied — repository not found",
            None,
            "failure",
            started,
        );
        return ExitCode::from(66);
    };
    let display_name = repo.display_name.clone();
    if repo.host.as_deref() != Some(host.as_str()) {
        eprintln!("shehata: host mismatch for repository — refusing credentials");
        write_credential_audit(
            &db_path,
            repo_id,
            None,
            "Credential denied — host mismatch",
            Some(&display_name),
            "failure",
            started,
        );
        return ExitCode::from(65);
    }
    let Some(account_id) = repo.assigned_account_id else {
        eprintln!("shehata: repository has no assigned account");
        write_credential_audit(
            &db_path,
            repo_id,
            None,
            "Credential denied — no assigned account",
            Some(&display_name),
            "failure",
            started,
        );
        return ExitCode::from(66);
    };
    let Ok(Some(account)) = queries::find_account_by_id(&db, account_id) else {
        eprintln!("shehata: assigned account no longer exists");
        write_credential_audit(
            &db_path,
            repo_id,
            None,
            "Credential denied — account missing",
            Some(&display_name),
            "failure",
            started,
        );
        return ExitCode::from(66);
    };
    let login = account.login;
    if account.host != *host {
        eprintln!("shehata: assigned account host mismatch — refusing credentials");
        write_credential_audit(
            &db_path,
            repo_id,
            Some(&login),
            "Credential denied — account host mismatch",
            Some(&display_name),
            "failure",
            started,
        );
        return ExitCode::from(65);
    }

    // 3b. Enforce exact repository path scoping (P0 security fix).
    //
    // credential.useHttpPath=true is always set during link, so git sends
    // `path=owner/repo.git`. We compare the requested path against the
    // repository record's owner/repo_name. Missing path → deny (fail-closed).
    {
        let repo_owner = repo.owner.as_deref().unwrap_or("");
        let repo_name = repo.repo_name.as_deref().unwrap_or("");
        if repo_owner.is_empty() || repo_name.is_empty() {
            eprintln!("shehata: repository record has no owner/name — refusing credentials");
            write_credential_audit(
                &db_path,
                repo_id,
                Some(&login),
                "Credential denied — repository identity incomplete",
                Some(&display_name),
                "failure",
                started,
            );
            return ExitCode::from(65);
        }
        let expected = format!("{}/{}", repo_owner, repo_name).to_ascii_lowercase();
        let requested = request.normalized_repo_path();
        match requested {
            None => {
                eprintln!("shehata: credential request has no path — refusing (fail-closed)");
                write_credential_audit(
                    &db_path,
                    repo_id,
                    Some(&login),
                    "Credential denied — no path in request",
                    Some(&display_name),
                    "failure",
                    started,
                );
                return ExitCode::from(65);
            }
            Some(ref path) if *path != expected => {
                eprintln!("shehata: repository path mismatch — refusing credentials");
                write_credential_audit(
                    &db_path,
                    repo_id,
                    Some(&login),
                    "Credential denied — repository path mismatch",
                    Some(&display_name),
                    "failure",
                    started,
                );
                return ExitCode::from(65);
            }
            _ => { /* path matches — proceed */ }
        }

        // Reject requests with embedded credentials in the url field.
        if request.has_embedded_credentials() {
            eprintln!("shehata: credential request URL contains embedded credentials — refusing");
            write_credential_audit(
                &db_path,
                repo_id,
                Some(&login),
                "Credential denied — embedded credentials in URL",
                Some(&display_name),
                "failure",
                started,
            );
            return ExitCode::from(65);
        }
    }

    // 4. Fetch the token just-in-time from the GitHub CLI.
    let Ok(gh) = GhRunner::locate() else {
        eprintln!("shehata: GitHub CLI (gh) not found");
        write_credential_audit(
            &db_path,
            repo_id,
            Some(&login),
            "Credential denied — gh CLI not found",
            Some(&display_name),
            "failure",
            started,
        );
        return ExitCode::from(69);
    };
    let token = match gh.token_for(host, &login).await {
        Ok(token) => token,
        Err(_) => {
            eprintln!("shehata: could not retrieve token for the assigned account");
            write_credential_audit(
                &db_path,
                repo_id,
                Some(&login),
                "Credential denied — token retrieval failed",
                Some(&display_name),
                "failure",
                started,
            );
            return ExitCode::from(69);
        }
    };

    // 5. Emit credentials on stdout. Nothing else may print here.
    let mut stdout = std::io::stdout().lock();
    let output = format!("username={login}\npassword={}\n", token.expose_secret());
    if stdout.write_all(output.as_bytes()).is_err() || stdout.flush().is_err() {
        return ExitCode::from(74);
    }
    drop(token);

    // 6. Record that credentials were served — best effort, never blocks the
    //    primary credential flow.
    write_credential_audit(
        &db_path,
        repo_id,
        Some(&login),
        "Credentials served",
        Some(&display_name),
        "success",
        started,
    );
    ExitCode::SUCCESS
}

/// Best-effort audit write. Opens a separate read-write connection so the
/// primary read-only lookup path is unaffected. If the write fails for any
/// reason (locked DB, missing file, …) we log to stderr and move on — the
/// credential helper's job is to serve credentials, not to block on bookkeeping.
fn write_credential_audit(
    db_path: &Path,
    repo_id: &str,
    account_login: Option<&str>,
    summary: &str,
    detail: Option<&str>,
    result: &str,
    started: Instant,
) {
    let Ok(db) = Database::open_at(db_path) else {
        eprintln!("shehata: audit write skipped — could not open database");
        return;
    };
    let event = NewAuditEvent {
        event_type: if result == "success" {
            "credential_served"
        } else {
            "credential_denied"
        },
        repository_id: Some(repo_id),
        account_login,
        summary,
        detail,
        result,
        exit_code: None,
        duration_ms: Some(started.elapsed().as_millis().min(i64::MAX as u128) as i64),
    };
    if let Err(e) = queries::insert_audit_event(&db, &event) {
        eprintln!("shehata: audit write failed — {e}");
    }
}

fn init_tracing() {
    // stderr only, no ANSI, and filterable. Token values must never be logged
    // by design; redact::redact_secrets is the safety net for message
    // text.
    let filter = tracing_subscriber::EnvFilter::try_from_env("SHEHATA_LOG")
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("warn"));
    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_writer(std::io::stderr)
        .with_ansi(false)
        .init();
}
