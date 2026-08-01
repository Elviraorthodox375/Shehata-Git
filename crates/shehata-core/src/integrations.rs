//! Safe local integration discovery for supported AI coding clients.

use serde::Serialize;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct AiClientInfo {
    pub id: String,
    pub name: String,
    pub available: bool,
    pub executable_path: Option<String>,
}

pub fn detect_ai_clients() -> Vec<AiClientInfo> {
    [
        ("codex", "Codex", &["codex"][..]),
        ("claude", "Claude Code", &["claude"][..]),
        ("cursor", "Cursor", &["cursor"][..]),
        (
            "vscode",
            "Visual Studio Code",
            &["code", "code-insiders"][..],
        ),
    ]
    .into_iter()
    .map(|(id, name, candidates)| {
        let executable = candidates
            .iter()
            .find_map(|candidate| which::which(candidate).ok());
        AiClientInfo {
            id: id.to_string(),
            name: name.to_string(),
            available: executable.is_some(),
            executable_path: executable.map(|path| path.to_string_lossy().into_owned()),
        }
    })
    .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exposes_only_the_reviewed_client_catalog() {
        let clients = detect_ai_clients();
        assert_eq!(clients.len(), 4);
        assert_eq!(
            clients
                .iter()
                .map(|client| client.id.as_str())
                .collect::<Vec<_>>(),
            ["codex", "claude", "cursor", "vscode"]
        );
        assert!(clients.iter().all(|client| client
            .executable_path
            .as_deref()
            .is_none_or(|path| !path.contains(['\r', '\n', '\0']))));
    }
}
