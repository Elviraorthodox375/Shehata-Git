use std::io::{BufRead, BufReader, Write};
use std::process::{Command, Stdio};

fn send(stdin: &mut impl Write, value: serde_json::Value) {
    writeln!(stdin, "{value}").unwrap();
    stdin.flush().unwrap();
}

fn receive(reader: &mut impl BufRead) -> serde_json::Value {
    let mut line = String::new();
    reader.read_line(&mut line).unwrap();
    assert!(!line.is_empty(), "MCP server closed before responding");
    serde_json::from_str(&line).unwrap()
}

#[test]
fn server_lists_only_the_reviewed_safe_tool_surface() {
    let unregistered = tempfile::tempdir().unwrap();
    let mut child = Command::new(env!("CARGO_BIN_EXE_shehata-mcp"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();
    let mut stdin = child.stdin.take().unwrap();
    let mut stdout = BufReader::new(child.stdout.take().unwrap());

    send(
        &mut stdin,
        serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {
                "protocolVersion": "2025-06-18",
                "capabilities": {},
                "clientInfo": { "name": "contract-test", "version": "0.1" }
            }
        }),
    );
    let initialized = receive(&mut stdout);
    assert_eq!(initialized["id"], 1);
    assert!(initialized.get("result").is_some(), "{initialized}");

    send(
        &mut stdin,
        serde_json::json!({
            "jsonrpc": "2.0",
            "method": "notifications/initialized",
            "params": {}
        }),
    );
    send(
        &mut stdin,
        serde_json::json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "tools/list",
            "params": {}
        }),
    );
    let listed = receive(&mut stdout);
    assert_eq!(listed["id"], 2);
    let tools = listed["result"]["tools"].as_array().unwrap();
    let names = tools
        .iter()
        .filter_map(|tool| tool["name"].as_str())
        .collect::<Vec<_>>();
    for expected in [
        "shehata_git_doctor",
        "shehata_git_list_accounts",
        "shehata_git_list_repositories",
        "shehata_git_get_repository",
        "shehata_git_status",
        "shehata_git_diff_summary",
        "shehata_git_check_identity",
        "shehata_git_test_connection",
        "shehata_git_commit",
        "shehata_git_pull_ff_only",
        "shehata_git_push",
    ] {
        assert!(names.contains(&expected), "missing {expected}: {names:?}");
    }
    assert_eq!(names.len(), 11, "unexpected tool exposed: {names:?}");
    for name in names {
        assert!(!name.contains("force"));
        assert!(!name.contains("shell"));
        assert!(!name.contains("reset"));
        assert!(!name.contains("delete"));
    }

    send(
        &mut stdin,
        serde_json::json!({
            "jsonrpc": "2.0",
            "id": 3,
            "method": "tools/call",
            "params": {
                "name": "shehata_git_status",
                "arguments": { "path": unregistered.path() }
            }
        }),
    );
    let called = receive(&mut stdout);
    assert_eq!(called["id"], 3);
    let envelope = called["result"]
        .get("structuredContent")
        .cloned()
        .or_else(|| {
            called["result"]["content"][0]["text"]
                .as_str()
                .and_then(|text| serde_json::from_str(text).ok())
        })
        .expect("tool call must return a structured envelope");
    assert_eq!(envelope["ok"], false);
    assert!(envelope["code"].is_string());
    assert!(envelope["summary"].is_string());
    assert!(!envelope.to_string().contains("password="));

    drop(stdin);
    child.kill().ok();
    child.wait().ok();
}
