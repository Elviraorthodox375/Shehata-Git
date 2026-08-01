//! shehata-core — the shared brain of Shehata Git.
//!
//! Desktop (Tauri), CLI, credential helper, and MCP server all call into this
//! crate. Business logic never lives in command handlers or UI components.

pub mod accounts;
pub mod assignment;
pub mod doctor;
pub mod error;
pub mod models;
pub mod redact;
pub mod repositories;

pub use doctor::{Doctor, APP_VERSION};
pub use error::{Result, ShehataError};
pub use models::{AccountInfo, CheckStatus, DoctorReport, PushPolicy, SystemCheck};
