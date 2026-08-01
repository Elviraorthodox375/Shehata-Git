//! Secret redaction utilities.
//!
//! Applied to any free-form text before it is logged, shown in the UI, or
//! stored in the audit log. Defense in depth: by design no secret should
//! reach these paths at all — this is the safety net.

const REDACTED: &str = "[REDACTED]";

/// GitHub token prefixes we actively hunt for:
/// ghp_ (personal), gho_ (OAuth), ghu_ (user-to-server), ghs_ (server-to-server),
/// ghr_ (refresh), github_pat_ (fine-grained).
const TOKEN_PREFIXES: &[&str] = &["ghp_", "gho_", "ghu_", "ghs_", "ghr_", "github_pat_"];

/// Replace any GitHub-shaped token in `text` with `[REDACTED]`.
pub fn redact_github_tokens(text: &str) -> String {
    let mut result = text.to_string();
    for prefix in TOKEN_PREFIXES {
        result = redact_prefixed(&result, prefix);
    }
    result
}

fn redact_prefixed(text: &str, prefix: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(start) = rest.find(prefix) {
        out.push_str(&rest[..start]);
        let after = &rest[start..];
        let token_len = after
            .find(|c: char| !(c.is_ascii_alphanumeric() || c == '_'))
            .unwrap_or(after.len());
        out.push_str(REDACTED);
        rest = &after[token_len..];
    }
    out.push_str(rest);
    out
}

/// Redact a known exact secret value (e.g. a token we just handled) from
/// arbitrary text. Use before logging any third-party output that could
/// conceivably echo the secret.
pub fn redact_exact(text: &str, secret: &str) -> String {
    if secret.is_empty() {
        return text.to_string();
    }
    text.replace(secret, REDACTED)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn redacts_classic_pat() {
        // Keep the fixture split so repository secret scanners never mistake
        // a deliberate redaction test for a committed credential.
        let input = [
            "fatal: authentication failed for ghp_",
            "abcdefghijklmnopqrstuvwxyz1234",
        ]
        .concat();
        let out = redact_github_tokens(&input);
        assert!(!out.contains("ghp_"));
        assert!(out.contains(REDACTED));
    }

    #[test]
    fn redacts_fine_grained_pat() {
        let input = ["token github_pat_", "11ABCDEFG0_ijklmnopqrstuvwxyz leaked"].concat();
        let out = redact_github_tokens(&input);
        assert!(!out.contains("github_pat_11"));
    }

    #[test]
    fn redacts_all_prefixes() {
        for prefix in ["ghp_", "gho_", "ghu_", "ghs_", "ghr_", "github_pat_"] {
            let input = format!("value: {prefix}AAAA1111bbbb2222");
            let out = redact_github_tokens(&input);
            assert!(!out.contains(prefix), "prefix {prefix} not redacted");
        }
    }

    #[test]
    fn leaves_normal_text_alone() {
        let input = "git push to github.com/owner/repo failed with exit 128";
        assert_eq!(redact_github_tokens(input), input);
    }

    #[test]
    fn redacts_multiple_tokens_in_one_string() {
        let input = "a ghp_1111aaaa b gho_2222bbbb c";
        let out = redact_github_tokens(input);
        assert!(!out.contains("ghp_1111aaaa"));
        assert!(!out.contains("gho_2222bbbb"));
    }

    #[test]
    fn redact_exact_works() {
        let out = redact_exact("the secret is hunter2 ok?", "hunter2");
        assert_eq!(out, format!("the secret is {REDACTED} ok?"));
        assert_eq!(redact_exact("unchanged", ""), "unchanged");
    }
}
