//! Secret redaction utilities.
//!
//! Applied to any free-form text before it is logged, shown in the UI, returned
//! over MCP, or stored in the audit log. Defense in depth: by design no secret
//! should reach these paths at all — this is the safety net for the cases where
//! a third-party process echoes something back at us.
//!
//! Redaction is deliberately conservative about what it treats as a secret.
//! Commit SHAs, repository paths, and account logins must survive untouched,
//! because the activity trail and error messages are useless without them.

const REDACTED: &str = "[REDACTED]";

/// GitHub token prefixes we actively hunt for:
/// ghp_ (personal), gho_ (OAuth), ghu_ (user-to-server), ghs_ (server-to-server),
/// ghr_ (refresh), github_pat_ (fine-grained).
///
/// GitHub Enterprise Server issues tokens with these same prefixes, so
/// enterprise deployments are covered by the same list.
const TOKEN_PREFIXES: &[&str] = &["ghp_", "gho_", "ghu_", "ghs_", "ghr_", "github_pat_"];

/// Authorization schemes whose value is always a credential.
const AUTH_SCHEMES: &[&str] = &["bearer ", "basic ", "token "];

/// Redact every secret shape this module knows about.
///
/// This is the function every boundary should call. It is intentionally
/// idempotent: running it twice produces the same text.
pub fn redact_secrets(text: &str) -> String {
    let mut result = redact_private_keys(text);
    result = redact_url_userinfo(&result);
    result = redact_authorization(&result);
    for prefix in TOKEN_PREFIXES {
        result = redact_prefixed(&result, prefix);
    }
    result
}

/// Replace any GitHub-shaped token in `text` with `[REDACTED]`.
///
/// Prefer [`redact_secrets`] at trust boundaries; this narrower entry point
/// exists for callers that only ever handle GitHub CLI output.
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

/// Strip `user:password@` (and `user@`) from any URL authority.
///
/// Only the userinfo section is removed — the scheme, host, and path stay
/// readable so the message still says which remote failed.
fn redact_url_userinfo(text: &str) -> String {
    const MARK: &str = "://";
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(start) = rest.find(MARK) {
        let (before, after_mark) = rest.split_at(start + MARK.len());
        out.push_str(before);

        // The authority ends at the first delimiter; userinfo must appear
        // before it, otherwise an '@' later in the path is not a credential.
        let authority_end = after_mark
            .find(['/', '?', '#', ' ', '\t', '\n', '\r', '"', '\''])
            .unwrap_or(after_mark.len());
        match after_mark[..authority_end].rfind('@') {
            Some(at) => {
                out.push_str(REDACTED);
                out.push('@');
                rest = &after_mark[at + 1..];
            }
            None => {
                out.push_str(&after_mark[..authority_end]);
                rest = &after_mark[authority_end..];
            }
        }
    }
    out.push_str(rest);
    out
}

/// Redact the credential in `Authorization:` headers and in bare
/// `Bearer`/`Basic`/`token` values.
fn redact_authorization(text: &str) -> String {
    let lower = text.to_ascii_lowercase();
    let mut out = String::with_capacity(text.len());
    let mut cursor = 0usize;

    while cursor < text.len() {
        let Some(found) = AUTH_SCHEMES
            .iter()
            .filter_map(|scheme| {
                lower[cursor..]
                    .find(scheme)
                    .map(|at| (cursor + at, *scheme))
            })
            .min_by_key(|(at, _)| *at)
        else {
            break;
        };
        let (scheme_at, scheme) = found;
        let value_at = scheme_at + scheme.len();
        out.push_str(&text[cursor..value_at]);

        // The credential runs to the end of the line.
        let value_end = text[value_at..]
            .find(['\n', '\r'])
            .map(|at| value_at + at)
            .unwrap_or(text.len());
        if value_end > value_at {
            out.push_str(REDACTED);
        }
        cursor = value_end;
    }

    out.push_str(&text[cursor..]);
    out
}

/// Replace a PEM private key block with a single marker.
fn redact_private_keys(text: &str) -> String {
    const BEGIN: &str = "-----BEGIN";
    const KEY_MARK: &str = "PRIVATE KEY-----";
    let mut out = String::with_capacity(text.len());
    let mut rest = text;

    while let Some(start) = rest.find(BEGIN) {
        // Only treat it as a key block when the header actually says so.
        let header_end = rest[start..]
            .find(KEY_MARK)
            .map(|at| start + at + KEY_MARK.len());
        let Some(header_end) = header_end else {
            out.push_str(&rest[..start + BEGIN.len()]);
            rest = &rest[start + BEGIN.len()..];
            continue;
        };

        out.push_str(&rest[..start]);
        out.push_str(REDACTED);

        // Consume through the matching END line when there is one.
        let after_header = &rest[header_end..];
        rest = match after_header.find("KEY-----") {
            Some(at) => &after_header[at + "KEY-----".len()..],
            None => "",
        };
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
        assert_eq!(redact_secrets(input), input);
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

    #[test]
    fn redacts_basic_auth_url_but_keeps_the_remote_readable() {
        let out = redact_secrets("remote https://octocat:hunter2@github.com/o/r.git rejected");
        assert!(!out.contains("hunter2"));
        assert!(!out.contains("octocat:"));
        assert!(out.contains("github.com/o/r.git"));
        assert!(out.contains("rejected"));
    }

    #[test]
    fn redacts_url_with_username_only() {
        let out = redact_secrets("https://x-access-token@github.com/o/r");
        assert!(!out.contains("x-access-token"));
        assert!(out.contains("github.com/o/r"));
    }

    #[test]
    fn leaves_ordinary_urls_and_emails_alone() {
        let input = "cloned https://github.com/o/r.git as dev@shehata.local";
        assert_eq!(redact_secrets(input), input);
    }

    #[test]
    fn redacts_authorization_headers() {
        for header in [
            "Authorization: Bearer abcdef123456",
            "authorization: basic dXNlcjpwYXNz",
            "Authorization: token abcdef123456",
        ] {
            let out = redact_secrets(header);
            assert!(!out.contains("abcdef123456"), "{header}");
            assert!(!out.contains("dXNlcjpwYXNz"), "{header}");
            assert!(out.contains(REDACTED), "{header}");
        }
    }

    #[test]
    fn redacts_only_the_credential_on_that_line() {
        let out = redact_secrets("Authorization: Bearer abcdef123456\nnext line stays");
        assert!(!out.contains("abcdef123456"));
        assert!(out.contains("next line stays"));
    }

    #[test]
    fn redacts_private_key_block() {
        let key = [
            "-----BEGIN RSA PRIVATE KEY-----\n",
            "MIIEowIBAAKCAQEA0000\n",
            "-----END RSA PRIVATE KEY-----",
        ]
        .concat();
        let out = redact_secrets(&format!("helper said:\n{key}\nafter"));
        assert!(!out.contains("MIIEowIBAAKCAQEA0000"));
        assert!(!out.contains("BEGIN RSA PRIVATE KEY"));
        assert!(out.contains("helper said:"));
        assert!(out.contains("after"));
    }

    #[test]
    fn keeps_commit_shas_and_branches_intact() {
        let input = "Normal push · Shehata Git · main · 0545b97 · docs: update readme";
        assert_eq!(redact_secrets(input), input);
    }

    #[test]
    fn is_idempotent() {
        let input = "https://user:pw@github.com/o/r ghp_aaaa1111 Authorization: Bearer zzz";
        let once = redact_secrets(input);
        assert_eq!(redact_secrets(&once), once);
    }
}
