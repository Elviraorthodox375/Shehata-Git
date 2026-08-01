use std::fs;
use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};

use chrono::Utc;
use shehata_storage::{queries, Database, RepositoryRecord};
use uuid::Uuid;

fn run(repo: &Path, args: &[&str]) {
    let status = Command::new("git")
        .arg("-C")
        .arg(repo)
        .args(args)
        .status()
        .unwrap();
    assert!(status.success());
}

fn build_fake_gh(directory: &Path) -> std::path::PathBuf {
    let source = directory.join("fake-gh.rs");
    let executable = directory.join(if cfg!(windows) { "gh.exe" } else { "gh" });
    fs::write(
        &source,
        r#"fn main() {
            let args: Vec<String> = std::env::args().skip(1).collect();
            assert_eq!(args, ["auth", "token", "--hostname", "github.com", "--user", "alice"]);
            println!("test-credential-value");
        }"#,
    )
    .unwrap();
    let status = Command::new("rustc")
        .args(["--edition=2021", "-o"])
        .arg(&executable)
        .arg(&source)
        .status()
        .unwrap();
    assert!(status.success());
    executable
}

#[test]
fn git_fill_routes_to_the_assigned_account() {
    let temp = tempfile::tempdir().unwrap();
    let repo = temp.path().join("repo with spaces");
    fs::create_dir(&repo).unwrap();
    run(&repo, &["init"]);

    let repository_id = Uuid::new_v4().to_string();
    let db_path = temp.path().join("shehata.db");
    let db = Database::open_at(&db_path).unwrap();
    let account_id = queries::upsert_account(&db, "github.com", "alice", "valid").unwrap();
    let now = Utc::now().to_rfc3339();
    queries::insert_repository(
        &db,
        &RepositoryRecord {
            id: repository_id.clone(),
            canonical_path: fs::canonicalize(&repo)
                .unwrap()
                .to_string_lossy()
                .into_owned(),
            git_dir: Some(repo.join(".git").to_string_lossy().into_owned()),
            git_common_dir: None,
            display_name: "demo".into(),
            host: Some("github.com".into()),
            owner: Some("acme".into()),
            repo_name: Some("demo".into()),
            remote_name: Some("origin".into()),
            remote_url: Some("https://github.com/acme/demo.git".into()),
            current_branch: Some("main".into()),
            assigned_account_id: Some(account_id),
            commit_name: None,
            commit_email: None,
            push_policy: "allow_normal_push".into(),
            created_at: now.clone(),
            updated_at: now.clone(),
            last_seen_at: Some(now),
        },
    )
    .unwrap();
    drop(db);

    let helper = Path::new(env!("CARGO_BIN_EXE_git-credential-shehata"));
    let canonical_helper = fs::canonicalize(helper).unwrap();
    let raw_helper = canonical_helper.to_string_lossy();
    let normalized_helper = raw_helper
        .strip_prefix(r"\\?\")
        .unwrap_or(&raw_helper)
        .replace('\\', "/");
    let helper_value = format!(
        "!'{}' --repo-id {repository_id}",
        normalized_helper.replace('\'', "'\\''")
    );
    run(
        &repo,
        &["config", "--local", "--add", "credential.helper", ""],
    );
    run(
        &repo,
        &[
            "config",
            "--local",
            "--add",
            "credential.helper",
            &helper_value,
        ],
    );
    run(
        &repo,
        &["config", "--local", "credential.useHttpPath", "true"],
    );

    let fake_gh = build_fake_gh(temp.path());
    let inherited_path = std::env::var_os("PATH").unwrap_or_default();
    let joined_path = std::env::join_paths(
        std::iter::once(fake_gh.parent().unwrap().to_path_buf())
            .chain(std::env::split_paths(&inherited_path)),
    )
    .unwrap();
    let mut child = Command::new("git")
        .arg("-C")
        .arg(&repo)
        .args(["credential", "fill"])
        .env("PATH", joined_path)
        .env("SHEHATA_DB_PATH", &db_path)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(b"protocol=https\nhost=github.com\npath=acme/demo.git\n\n")
        .unwrap();
    let output = child.wait_with_output().unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("username=alice"));
    assert!(stdout.contains("password=test-credential-value"));
}
