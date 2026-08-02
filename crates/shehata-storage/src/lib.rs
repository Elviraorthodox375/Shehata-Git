// Copyright (c) 2026 Dr Mohamed Shehata. All rights reserved.
// Licensed under the MIT License. See LICENSE in the project root.

//! shehata-storage — SQLite persistence.
//!
//! Hard rule: this crate has no column that can hold a credential.
//! Tokens, passwords, and authorization headers must never be passed here.

pub mod db;
pub mod queries;
pub mod records;

pub use db::{Database, StorageError};
pub use records::{
    AccountRecord, AuditEventRecord, ConfigBackupRecord, NewAuditEvent, RepositoryRecord,
};
