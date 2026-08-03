// Copyright (c) 2026 Dr Mohamed Shehata. All rights reserved.
// Licensed under the MIT License. See LICENSE in the project root.

//! One state-changing operation per repository at a time.
//!
//! The desktop app, the CLI, an MCP client, and a terminal can all reach the
//! same repository at once. Git itself guards its index with `index.lock`, but
//! that produces a raw, confusing failure part-way through an operation — and
//! it does not protect the steps around git, such as reading a plan, writing
//! configuration, and recording an audit event.
//!
//! Locks are per-process and in-memory on purpose. They serialise the surfaces
//! this process owns; they are not a claim to have locked the repository
//! against other programs, and nothing here is a security boundary.

use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};

use tokio::sync::{Mutex as AsyncMutex, OwnedMutexGuard};

use crate::error::{Result, ShehataError};

type RepositoryLocks = Mutex<HashMap<String, Arc<AsyncMutex<()>>>>;

fn registry() -> &'static RepositoryLocks {
    static REGISTRY: OnceLock<RepositoryLocks> = OnceLock::new();
    REGISTRY.get_or_init(|| Mutex::new(HashMap::new()))
}

/// Held for the duration of one state-changing operation.
///
/// Dropping the guard releases the repository, including on error, early
/// return, or panic — there is no unlock call to forget.
#[derive(Debug)]
pub struct RepositoryGuard {
    _inner: OwnedMutexGuard<()>,
}

fn lock_for(repository_id: &str) -> Arc<AsyncMutex<()>> {
    let mut locks = registry()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    locks
        .entry(repository_id.to_string())
        .or_insert_with(|| Arc::new(AsyncMutex::new(())))
        .clone()
}

/// Take the repository lock, refusing rather than queueing.
///
/// Refusing is the honest answer for a second push arriving while the first is
/// still running: the caller learns immediately instead of waiting on a
/// network operation it cannot see.
pub fn try_lock_repository(repository_id: &str) -> Result<RepositoryGuard> {
    lock_for(repository_id)
        .try_lock_owned()
        .map(|inner| RepositoryGuard { _inner: inner })
        .map_err(|_| ShehataError::OperationInProgress(repository_id.to_string()))
}

/// Take the repository lock, waiting for the current operation to finish.
pub async fn lock_repository(repository_id: &str) -> RepositoryGuard {
    RepositoryGuard {
        _inner: lock_for(repository_id).lock_owned().await,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn second_operation_on_the_same_repository_is_refused() {
        let id = "11111111-1111-4111-8111-111111111111";
        let held = try_lock_repository(id).unwrap();

        let refused = try_lock_repository(id);
        assert!(matches!(refused, Err(ShehataError::OperationInProgress(_))));

        drop(held);
        assert!(try_lock_repository(id).is_ok());
    }

    #[tokio::test]
    async fn different_repositories_do_not_block_each_other() {
        let first = try_lock_repository("22222222-2222-4222-8222-222222222222").unwrap();
        let second = try_lock_repository("33333333-3333-4333-8333-333333333333");
        assert!(second.is_ok());
        drop(first);
    }

    #[tokio::test]
    async fn the_lock_is_released_when_an_operation_fails() {
        let id = "44444444-4444-4444-8444-444444444444";

        // Simulate an operation that takes the lock and then errors out.
        let outcome: Result<()> = async {
            let _guard = try_lock_repository(id)?;
            Err(ShehataError::NonFastForward)
        }
        .await;
        assert!(outcome.is_err());

        // The guard was dropped by unwinding out of the block, not by an
        // explicit unlock the error path could have skipped.
        assert!(try_lock_repository(id).is_ok());
    }

    #[tokio::test]
    async fn waiting_callers_proceed_once_the_guard_drops() {
        let id = "55555555-5555-4555-8555-555555555555";
        let held = lock_repository(id).await;
        let waiter = tokio::spawn(async move { lock_repository(id).await });

        // The waiter cannot make progress while the first guard is alive.
        assert!(!waiter.is_finished());
        drop(held);
        waiter.await.unwrap();
    }
}
