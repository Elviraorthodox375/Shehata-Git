//! shehata-git — safe wrapper around the system `git` executable.
//!
//! Rules enforced here:
//! - Commands always run with argument arrays, never shell strings.
//! - Output is treated as data; nothing is re-executed.
//! - Destructive operations are not exposed by this crate at all.

pub mod remote;
pub mod runner;

pub use remote::{parse_remote_url, RemoteUrl};
pub use runner::{CommandOutput, GitError, GitRunner};
