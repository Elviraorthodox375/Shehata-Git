# Troubleshooting

Plain-language fixes for the most common problems. If none of this helps, use
**Settings → Copy diagnostic report** and open an issue. That report excludes
credentials, repository paths, installation paths, and executable paths.

## "Git is missing"

Git for Windows is not installed or not on PATH.
Install from <https://git-scm.com/download/win>, then restart Shehata Git.

## "GitHub CLI is missing"

Run `winget install GitHub.cli` in a terminal, then restart Shehata Git.

## "No GitHub accounts are signed in"

Open the **Accounts** page and add an account — the browser opens and no
password is ever typed into Shehata Git. Advanced users can also run
`gh auth login` in a terminal; the app picks it up on refresh.

## "Credential helper not found"

`git-credential-shehata` must sit next to the app or on your user PATH.

- Dev build: `cargo build --release` (binaries land in `target/release/`).
- Installed app: reinstall; the installer manages PATH for you.

## "Token unavailable" for an account

The GitHub CLI session for that account expired or was removed. Remove the
account from **Identities**, then add it again through the browser flow.
Shehata Git never sees your password.

## Push asks for a password in the terminal

The repository is not linked, or the helper is not reachable. Run
`shehata doctor` — it tells you exactly which piece is missing.

## Push used the wrong account

That should be impossible for a linked repository (the helper is bound to one
account and fails closed). It means the repository is **not** linked:
link it from the Repositories page, then retry.

## The app window is blank

WebView2 is missing or broken. Install/repair it from
<https://developer.microsoft.com/microsoft-edge/webview2/>.

## Where is my data?

- Database: `%APPDATA%\Shehata\shehata-git\data\shehata.db` (no tokens inside)
- Nothing is written into your repositories except a tiny marker file under
  `.git/shehata-git/` plus standard local git config entries, which
  **Unlink & Restore** removes.
