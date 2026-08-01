//! shehata-github — wrapper around the official GitHub CLI.
//!
//! The GitHub CLI is the credential source of truth. This crate:
//! - discovers authenticated accounts via `gh auth status --json hosts`
//! - fetches short-lived tokens just-in-time via `gh auth token`
//!
//! Tokens are returned as `secrecy::SecretString`, never logged, never
//! persisted, and must be dropped by the caller as soon as possible.

pub mod models;
pub mod runner;

pub use models::{GhAuthAccount, GhAuthStatus};
pub use runner::{GhError, GhLoginEvent, GhRunner};
