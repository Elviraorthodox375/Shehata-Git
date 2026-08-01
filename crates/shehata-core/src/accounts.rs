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

#[derive(Debug, Clone, Deserialize)]
pub struct SwitchAccountRequest {
    pub host: String,
    pub login: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GrantScopeRequest {
    pub host: String,
    pub login: String,
    pub scope: String,
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

/// Change GitHub CLI's explicit default account for one host. Shehata Git's
/// repository routes remain unchanged and continue to select exact accounts.
pub async fn switch_active_account(
    gh: &GhRunner,
    request: &SwitchAccountRequest,
) -> Result<Vec<AccountInfo>> {
    gh.switch_active_account(&request.host, &request.login)
        .await
        .map_err(ShehataError::Github)?;
    list_accounts(gh).await
}

/// Add one missing OAuth scope to an exact account through GitHub's browser
/// flow.
///
/// `gh auth refresh` only ever acts on the host's active account, so the
/// account being repaired is made active for the duration and the previous
/// default is restored afterwards — including when authorization fails or the
/// user cancels. The scope itself is validated by the GitHub CLI runner.
pub async fn grant_scope<F>(
    gh: &GhRunner,
    request: &GrantScopeRequest,
    on_event: F,
    cancel: tokio::sync::oneshot::Receiver<()>,
) -> Result<Vec<AccountInfo>>
where
    F: Fn(shehata_github::GhLoginEvent) + Send + Sync + 'static,
{
    let before = list_accounts(gh).await?;
    let previous_default = before
        .iter()
        .find(|account| account.active && account.host.eq_ignore_ascii_case(&request.host))
        .map(|account| account.login.clone());
    let needs_switch = previous_default
        .as_deref()
        .is_none_or(|login| !login.eq_ignore_ascii_case(&request.login));

    if needs_switch {
        gh.switch_active_account(&request.host, &request.login)
            .await
            .map_err(ShehataError::Github)?;
    }

    let granted = gh
        .refresh_scope_cancellable(&request.host, &request.scope, on_event, cancel)
        .await
        .map_err(ShehataError::Github);

    if needs_switch {
        if let Some(login) = previous_default {
            // Restoring the user's chosen CLI default matters more than
            // reporting a switch-back failure, so the grant result wins.
            let _ = gh.switch_active_account(&request.host, &login).await;
        }
    }

    granted?;
    list_accounts(gh).await
}

/// Run a GitHub CLI command as one exact account, then restore the previous
/// CLI default.
///
/// `gh` has no per-repository account concept: every command uses whichever
/// account is active for the host. This makes the assigned identity apply for
/// the duration of a single command instead, so `gh pr create` in a routed
/// repository does not depend on which account happens to be the default.
///
/// The previous default is restored even when the command fails, and the
/// GitHub CLI's own exit code is returned unchanged.
pub async fn run_gh_as(gh: &GhRunner, host: &str, login: &str, args: &[String]) -> Result<i32> {
    let accounts = list_accounts(gh).await?;
    let target = accounts
        .iter()
        .find(|account| {
            account.host.eq_ignore_ascii_case(host) && account.login.eq_ignore_ascii_case(login)
        })
        .ok_or_else(|| ShehataError::AccountNotAvailable {
            host: host.to_string(),
            login: login.to_string(),
        })?;
    if !target.token_available {
        return Err(ShehataError::AccountNotAvailable {
            host: host.to_string(),
            login: login.to_string(),
        });
    }

    let previous_default = accounts
        .iter()
        .find(|account| account.active && account.host.eq_ignore_ascii_case(host))
        .map(|account| account.login.clone());
    let needs_switch = previous_default
        .as_deref()
        .is_none_or(|active| !active.eq_ignore_ascii_case(login));

    if needs_switch {
        gh.switch_active_account(host, login)
            .await
            .map_err(ShehataError::Github)?;
    }

    let outcome = gh.run_passthrough(args).await.map_err(ShehataError::Github);

    if needs_switch {
        if let Some(active) = previous_default {
            // Leaving the user's chosen default changed would be worse than
            // losing a restore error, so the command's own result wins.
            let _ = gh.switch_active_account(host, &active).await;
        }
    }

    outcome
}

/// Resolve a repository's assigned account, then run one GitHub CLI command
/// as that account. The database connection is closed before any await.
pub async fn run_gh_for_repository(
    gh: &GhRunner,
    repository_id: &str,
    args: &[String],
) -> Result<i32> {
    let (host, login) = {
        let db = Database::open_default()?;
        let repository = queries::find_repository_by_id(&db, repository_id)?
            .ok_or_else(|| ShehataError::RepositoryNotFound(repository_id.to_string()))?;
        let account_id = repository
            .assigned_account_id
            .ok_or_else(|| ShehataError::RepositoryNotLinked(repository.id.clone()))?;
        let account = queries::find_account_by_id(&db, account_id)?
            .ok_or_else(|| ShehataError::RepositoryNotLinked(repository.id.clone()))?;
        (account.host, account.login)
    };
    run_gh_as(gh, &host, &login, args).await
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
