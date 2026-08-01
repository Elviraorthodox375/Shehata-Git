//! Account discovery: read live state from the GitHub CLI and mirror safe
//! metadata into the local database. Tokens are probed but never stored —
//! `token_available` is the only signal that leaves this module.

use serde::Deserialize;
use shehata_github::GhRunner;
use shehata_storage::{queries, Database};

use crate::error::{Result, ShehataError};
use crate::models::AccountInfo;

#[derive(Debug, Clone, Deserialize)]
pub struct RemoveAccountRequest {
    pub host: String,
    pub login: String,
}

/// List all accounts known to the GitHub CLI, checking whether a token can
/// actually be retrieved for each. This async discovery step never holds a
/// SQLite connection across an await point.
pub async fn list_accounts(gh: &GhRunner) -> Result<Vec<AccountInfo>> {
    let status = gh.auth_status().await.map_err(ShehataError::Github)?;

    let mut accounts = Vec::new();
    for (host, entries) in &status.hosts {
        for entry in entries {
            // Probe token retrieval only for accounts whose state is healthy.
            // The token itself is dropped immediately — it never leaves here.
            let token_available = entry.token_usable()
                && gh
                    .token_for(host, &entry.login)
                    .await
                    .map(|secret| {
                        drop(secret);
                        true
                    })
                    .unwrap_or(false);

            accounts.push(AccountInfo {
                host: host.clone(),
                login: entry.login.clone(),
                active: entry.active,
                token_available,
            });
        }
    }
    Ok(accounts)
}

/// Sign one exact account out of the local GitHub CLI store, then return the
/// fresh live account list. Repository assignments are deliberately retained
/// so a removed credential can never silently fall through to another login.
pub async fn remove_account(
    gh: &GhRunner,
    request: &RemoveAccountRequest,
) -> Result<Vec<AccountInfo>> {
    gh.logout(&request.host, &request.login)
        .await
        .map_err(ShehataError::Github)?;
    list_accounts(gh).await
}

/// Mirror safe account metadata after discovery has completed. A database
/// hiccup is intentionally non-fatal so it can never hide live gh state.
pub fn mirror_accounts(db: &Database, accounts: &[AccountInfo]) {
    for account in accounts {
        let state = if account.token_available {
            "valid"
        } else {
            "attention"
        };
        let _ = queries::upsert_account(db, &account.host, &account.login, state);
    }
}
