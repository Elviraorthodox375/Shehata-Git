import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { writeText } from "@tauri-apps/plugin-clipboard-manager";
import { openUrl } from "@tauri-apps/plugin-opener";
import {
  ArrowRightLeft,
  CheckCircle2,
  Copy,
  ExternalLink,
  KeyRound,
  LoaderCircle,
  Plus,
  RefreshCw,
  ShieldCheck,
  Trash2,
  UserRound,
  XCircle,
} from "lucide-react";
import { useEffect, useMemo, useState } from "react";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { ConfirmDialog } from "@/components/ui/ConfirmDialog";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { SearchField } from "@/components/ui/SearchField";
import {
  addAccount,
  cancelAccountLogin,
  listAccounts,
  removeAccount,
  switchAccount,
} from "@/lib/tauri";
import type { GhAccount, GhLoginEvent } from "@/lib/types";

/**
 * Accounts are discovered from the official GitHub CLI. Shehata Git never
 * receives a password and never sends a token across the Tauri boundary.
 */
export function AccountsPage() {
  const queryClient = useQueryClient();
  const [dialogOpen, setDialogOpen] = useState(false);
  const [loginEvent, setLoginEvent] = useState<GhLoginEvent | null>(null);
  const [search, setSearch] = useState("");
  const [accountToRemove, setAccountToRemove] = useState<GhAccount | null>(null);
  const [accountToActivate, setAccountToActivate] = useState<GhAccount | null>(null);
  const [cancelingLogin, setCancelingLogin] = useState(false);
  const accounts = useQuery({ queryKey: ["accounts"], queryFn: listAccounts });
  const filteredAccounts = useMemo(() => {
    const needle = search.trim().toLowerCase();
    if (!needle) return accounts.data ?? [];
    return (accounts.data ?? []).filter(
      (account) =>
        account.login.toLowerCase().includes(needle) || account.host.toLowerCase().includes(needle),
    );
  }, [accounts.data, search]);
  const login = useMutation({
    mutationFn: () => addAccount(setLoginEvent),
    onSuccess: (data) => {
      queryClient.setQueryData(["accounts"], data);
      void queryClient.invalidateQueries({ queryKey: ["doctor"] });
    },
  });
  const remove = useMutation({
    mutationFn: (account: GhAccount) => removeAccount(account),
    onSuccess: (data) => {
      queryClient.setQueryData(["accounts"], data);
      setAccountToRemove(null);
      void Promise.all([
        queryClient.invalidateQueries({ queryKey: ["doctor"] }),
        queryClient.invalidateQueries({ queryKey: ["repositories"] }),
      ]);
    },
  });
  const activate = useMutation({
    mutationFn: (account: GhAccount) => switchAccount(account),
    onSuccess: (data) => {
      queryClient.setQueryData(["accounts"], data);
      setAccountToActivate(null);
    },
  });

  function startLogin() {
    login.reset();
    setLoginEvent(null);
    setDialogOpen(true);
    login.mutate();
  }

  async function cancelLogin() {
    setCancelingLogin(true);
    try {
      await cancelAccountLogin();
      setDialogOpen(false);
      setLoginEvent(null);
    } finally {
      setCancelingLogin(false);
    }
  }

  return (
    <div className="mx-auto w-full max-w-6xl space-y-5">
      <section className="instrument-panel overflow-hidden rounded-[0.75rem]">
        <div className="flex flex-col gap-5 p-5 sm:p-6 lg:flex-row lg:items-end lg:justify-between">
          <div>
            <p className="eyebrow">Credential source / official GitHub CLI</p>
            <h2 className="mt-2 font-display text-2xl font-semibold tracking-tight">
              Authenticated identities, without copied tokens.
            </h2>
            <p className="mt-2 max-w-2xl text-sm leading-6 text-muted-foreground">
              Accounts remain in the official GitHub CLI store. Shehata Git reads availability and
              routes repositories to an exact login.
            </p>
          </div>
          <div className="flex flex-wrap gap-2">
            <Button
              variant="outline"
              onClick={() => accounts.refetch()}
              disabled={accounts.isFetching || login.isPending}
            >
              <RefreshCw className={accounts.isFetching ? "animate-spin" : undefined} aria-hidden />
              Refresh
            </Button>
            <Button onClick={startLogin} disabled={login.isPending}>
              <Plus aria-hidden />
              Add GitHub Account
            </Button>
          </div>
        </div>
        <div className="grid border-t border-border bg-background/20 sm:grid-cols-3 sm:divide-x sm:divide-border">
          <IdentityMetric label="DISCOVERED" value={accounts.data?.length ?? 0} />
          <IdentityMetric
            label="TOKEN READY"
            value={accounts.data?.filter((account) => account.token_available).length ?? 0}
            tone="success"
          />
          <IdentityMetric
            label="NEEDS ATTENTION"
            value={accounts.data?.filter((account) => !account.token_available).length ?? 0}
            tone="warning"
          />
        </div>
      </section>

      {(accounts.data?.length ?? 0) > 0 && (
        <div className="liquid-panel rounded-[0.8rem] p-3">
          <SearchField
            value={search}
            onChange={setSearch}
            label="Search identities"
            placeholder="Search by GitHub login or host…"
            resultCount={filteredAccounts.length}
          />
        </div>
      )}

      {accounts.isLoading && <p className="text-sm text-muted-foreground">Reading accounts…</p>}

      {accounts.isError && (
        <Card className="border-destructive/40 bg-destructive/[0.04]">
          <CardHeader>
            <CardTitle className="text-destructive">Could not read accounts</CardTitle>
          </CardHeader>
          <CardContent>
            <p className="text-sm text-muted-foreground">{errorMessage(accounts.error)}</p>
          </CardContent>
        </Card>
      )}

      {remove.isError && (
        <Card className="border-destructive/40 bg-destructive/[0.04]">
          <CardContent className="flex items-start gap-3 py-4">
            <XCircle className="mt-0.5 h-5 w-5 shrink-0 text-destructive" aria-hidden />
            <div>
              <p className="text-sm font-semibold text-destructive">Could not remove account</p>
              <p className="mt-1 text-sm text-muted-foreground">{errorMessage(remove.error)}</p>
            </div>
          </CardContent>
        </Card>
      )}

      {activate.isError && (
        <Card className="border-destructive/40 bg-destructive/[0.04]">
          <CardContent className="flex items-start gap-3 py-4">
            <XCircle className="mt-0.5 h-5 w-5 shrink-0 text-destructive" aria-hidden />
            <div>
              <p className="text-sm font-semibold text-destructive">
                Could not change the GitHub CLI default
              </p>
              <p className="mt-1 text-sm text-muted-foreground">{errorMessage(activate.error)}</p>
            </div>
          </CardContent>
        </Card>
      )}

      {accounts.data?.length === 0 && (
        <Card className="relative overflow-hidden border-dashed bg-card/45">
          <span className="absolute left-0 top-0 h-5 w-5 border-l border-t border-primary/50" />
          <span className="absolute right-0 top-0 h-5 w-5 border-r border-t border-primary/50" />
          <span className="absolute bottom-0 left-0 h-5 w-5 border-b border-l border-primary/50" />
          <span className="absolute bottom-0 right-0 h-5 w-5 border-b border-r border-primary/50" />
          <CardContent className="flex min-h-64 flex-col items-center justify-center gap-4 py-10 text-center">
            <div className="flex h-14 w-14 items-center justify-center border border-primary/25 bg-primary/[0.07]">
              <UserRound className="h-6 w-6 text-primary" aria-hidden />
            </div>
            <div>
              <p className="eyebrow">Identity registry empty</p>
              <p className="mt-2 font-display text-xl font-semibold">
                Add your first GitHub account
              </p>
              <p className="mt-1 max-w-sm text-sm leading-relaxed text-muted-foreground">
                Browser authentication runs through GitHub CLI. Your password and token never enter
                Shehata Git.
              </p>
            </div>
            <Button onClick={startLogin} disabled={login.isPending}>
              <Plus aria-hidden />
              Start browser sign-in
            </Button>
          </CardContent>
        </Card>
      )}

      {!accounts.isLoading && (accounts.data?.length ?? 0) > 0 && filteredAccounts.length === 0 && (
        <Card>
          <CardContent className="flex min-h-40 flex-col items-center justify-center text-center">
            <p className="font-display font-semibold">No matching identities</p>
            <p className="mt-1 text-sm text-muted-foreground">Try another login or host name.</p>
          </CardContent>
        </Card>
      )}

      <div className="grid gap-3">
        {filteredAccounts.map((account) => {
          const removing =
            remove.isPending &&
            remove.variables?.host === account.host &&
            remove.variables.login === account.login;
          return (
            <Card key={`${account.host}:${account.login}`} className="border-l-2 border-l-success">
              <CardContent className="flex flex-col items-start gap-4 py-4 sm:flex-row sm:items-center">
                <div className="flex h-10 w-10 shrink-0 items-center justify-center border border-border bg-background/35">
                  <UserRound className="h-5 w-5 text-muted-foreground" aria-hidden />
                </div>
                <div className="min-w-0 flex-1">
                  <p className="data-label">AUTHENTICATED IDENTITY</p>
                  <p className="mt-1 truncate font-display font-semibold">@{account.login}</p>
                  <p className="mt-0.5 font-mono text-[0.7rem] text-muted-foreground">
                    {account.host}
                  </p>
                  <p className="mt-1 text-xs text-muted-foreground/75">
                    {account.active
                      ? "Default for plain GitHub CLI commands."
                      : "Ready for repositories routed through Shehata Git."}
                  </p>
                </div>
                <div className="flex w-full flex-wrap items-center gap-2 sm:w-auto sm:justify-end">
                  {account.active && (
                    <Badge
                      variant="secondary"
                      title="The default account used by plain gh commands"
                    >
                      CLI default
                    </Badge>
                  )}
                  <Badge variant={account.token_available ? "success" : "warning"}>
                    {account.token_available ? "ready" : "needs sign-in"}
                  </Badge>
                  {!account.active && account.token_available && (
                    <Button
                      type="button"
                      size="sm"
                      variant="outline"
                      className="min-h-11 sm:min-h-9"
                      onClick={() => {
                        activate.reset();
                        setAccountToActivate(account);
                      }}
                      disabled={remove.isPending || login.isPending || activate.isPending}
                    >
                      <ArrowRightLeft aria-hidden /> Make CLI default
                    </Button>
                  )}
                  <Button
                    type="button"
                    size="sm"
                    variant="ghost"
                    className="ml-auto min-h-11 text-muted-foreground hover:text-destructive sm:ml-2 sm:min-h-9"
                    onClick={() => {
                      remove.reset();
                      setAccountToRemove(account);
                    }}
                    disabled={remove.isPending || login.isPending || activate.isPending}
                  >
                    {removing ? (
                      <LoaderCircle className="animate-spin" aria-hidden />
                    ) : (
                      <Trash2 aria-hidden />
                    )}
                    {removing ? "Removing…" : "Remove"}
                  </Button>
                </div>
              </CardContent>
            </Card>
          );
        })}
      </div>

      {dialogOpen && (
        <AccountLoginDialog
          event={loginEvent}
          pending={login.isPending}
          success={login.isSuccess}
          error={login.isError ? errorMessage(login.error) : null}
          onClose={() => setDialogOpen(false)}
          onCancel={cancelLogin}
          canceling={cancelingLogin}
        />
      )}

      {accountToRemove && (
        <ConfirmDialog
          eyebrow="Confirm local sign-out"
          title={`Remove @${accountToRemove.login}?`}
          description={
            <>
              This signs the account out of <strong className="text-foreground">GitHub CLI</strong>{" "}
              on this PC only. Shehata Git will stop using it immediately.
            </>
          }
          detail={
            <>
              Repository assignments stay safely locked. Reassign them or sign in again before the
              next push. Access is not revoked on GitHub.com.
            </>
          }
          confirmLabel="Remove from this PC"
          cancelLabel="Keep account"
          pendingLabel="Removing…"
          pending={remove.isPending}
          onCancel={() => setAccountToRemove(null)}
          onConfirm={() => remove.mutate(accountToRemove)}
        />
      )}

      {accountToActivate && (
        <ConfirmDialog
          eyebrow="GitHub CLI default"
          title={`Make @${accountToActivate.login} the CLI default?`}
          description={
            <>
              Plain <strong className="text-foreground">gh</strong> commands for{" "}
              <strong className="text-foreground">{accountToActivate.host}</strong> will use this
              account by default.
            </>
          }
          detail="Repository assignments in Shehata Git do not change. Each routed repository keeps using its assigned identity."
          confirmLabel="Make CLI default"
          cancelLabel="Keep current default"
          pendingLabel="Switching…"
          pending={activate.isPending}
          tone="primary"
          onCancel={() => setAccountToActivate(null)}
          onConfirm={() => activate.mutate(accountToActivate)}
        />
      )}
    </div>
  );
}

interface AccountLoginDialogProps {
  event: GhLoginEvent | null;
  pending: boolean;
  success: boolean;
  error: string | null;
  onClose: () => void;
  onCancel: () => void;
  canceling: boolean;
  /** Overrides for reuse when authorizing something other than a new sign-in. */
  title?: string;
  description?: string;
}

export function AccountLoginDialog({
  event,
  pending,
  success,
  error,
  onClose,
  onCancel,
  canceling,
  title = "Sign in with GitHub",
  description = "Authentication happens on GitHub. Shehata Git never sees your password or stores your token.",
}: AccountLoginDialogProps) {
  const code = event?.type === "code" ? event.code : null;
  const [browserError, setBrowserError] = useState<string | null>(null);
  const [browserState, setBrowserState] = useState<"idle" | "opening" | "opened" | "failed">(
    "idle",
  );
  const [copyState, setCopyState] = useState<"idle" | "copying" | "copied" | "failed">("idle");
  const [manualCopyConfirmed, setManualCopyConfirmed] = useState(false);

  useEffect(() => {
    let active = true;
    if (!code) {
      setCopyState("idle");
      return;
    }

    setCopyState("copying");
    void writeText(code, { label: "GitHub one-time sign-in code" }).then(
      () => {
        if (active) setCopyState("copied");
      },
      () => {
        if (active) setCopyState("failed");
      },
    );

    return () => {
      active = false;
    };
  }, [code]);

  useEffect(() => {
    let active = true;
    if (!code) {
      setBrowserState("idle");
      setBrowserError(null);
      return;
    }

    setBrowserState("opening");
    setBrowserError(null);
    void openUrl("https://github.com/login/device").then(
      () => {
        if (active) setBrowserState("opened");
      },
      (openError: unknown) => {
        if (!active) return;
        setBrowserState("failed");
        setBrowserError(errorMessage(openError));
      },
    );

    return () => {
      active = false;
    };
  }, [code]);

  async function copyOneTimeCode() {
    if (!code) return;
    setCopyState("copying");
    setManualCopyConfirmed(false);
    try {
      await writeText(code, { label: "GitHub one-time sign-in code" });
      setCopyState("copied");
      setManualCopyConfirmed(true);
      window.setTimeout(() => setManualCopyConfirmed(false), 1800);
    } catch {
      setCopyState("failed");
    }
  }

  async function reopenGitHub() {
    setBrowserState("opening");
    setBrowserError(null);
    try {
      await openUrl("https://github.com/login/device");
      setBrowserState("opened");
    } catch (openError) {
      setBrowserState("failed");
      setBrowserError(errorMessage(openError));
    }
  }

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-background/80 p-4 backdrop-blur-sm">
      <section
        role="dialog"
        aria-modal="true"
        aria-labelledby="github-login-title"
        className="w-full max-w-md overflow-hidden rounded-xl border border-border bg-card shadow-2xl"
      >
        <div className="border-b border-border px-5 py-4">
          <div className="flex items-start gap-3">
            <div className="mt-0.5 flex h-9 w-9 shrink-0 items-center justify-center rounded-lg bg-primary/10 text-primary">
              <ShieldCheck className="h-5 w-5" aria-hidden />
            </div>
            <div>
              <h2 id="github-login-title" className="font-semibold">
                {title}
              </h2>
              <p className="mt-1 text-sm leading-relaxed text-muted-foreground">{description}</p>
            </div>
          </div>
        </div>

        <div className="space-y-4 px-5 py-5">
          {pending && (
            <div className="flex gap-3 rounded-lg border border-primary/20 bg-primary/5 p-4">
              <LoaderCircle
                className="mt-0.5 h-5 w-5 shrink-0 animate-spin text-primary"
                aria-hidden
              />
              <div>
                <p className="text-sm font-medium">
                  {code ? "Waiting for approval on GitHub" : "Preparing secure browser sign-in"}
                </p>
                <p className="mt-1 text-sm leading-relaxed text-muted-foreground">
                  {code
                    ? browserState === "failed"
                      ? "The browser did not open automatically. Use Open GitHub below, paste the code, and approve access."
                      : "GitHub is opening automatically. Paste the code and approve access, then return here."
                    : "GitHub CLI is creating a one-time code and opening your default browser."}
                </p>
              </div>
            </div>
          )}

          {code && pending && (
            <div className="rounded-lg border border-border bg-secondary/50 p-4">
              <div className="flex items-center gap-2 text-xs font-medium uppercase tracking-wide text-muted-foreground">
                <KeyRound className="h-4 w-4" aria-hidden />
                One-time code
              </div>
              <p className="mt-2 font-mono text-2xl font-semibold tracking-[0.18em] text-foreground">
                {code}
              </p>
              <p
                className="mt-2 flex items-center gap-1.5 text-xs text-muted-foreground"
                aria-live="polite"
              >
                {manualCopyConfirmed ? (
                  <>
                    <CheckCircle2 className="h-3.5 w-3.5 text-success" aria-hidden />
                    Copied again — ready to paste.
                  </>
                ) : copyState === "copied" ? (
                  <>
                    <CheckCircle2 className="h-3.5 w-3.5 text-success" aria-hidden />
                    Copied — paste it into the GitHub page.
                  </>
                ) : null}
                {copyState === "copying" && "Copying the code…"}
                {copyState === "failed" && "Automatic copy failed. Use Copy code below."}
                {copyState === "idle" && "Copy this code into the GitHub page."}
              </p>
              <div className="mt-4 grid gap-2 sm:grid-cols-2">
                <Button type="button" className="min-h-11" onClick={copyOneTimeCode}>
                  {manualCopyConfirmed ? <CheckCircle2 aria-hidden /> : <Copy aria-hidden />}
                  {manualCopyConfirmed
                    ? "Copied!"
                    : copyState === "copied"
                      ? "Copy again"
                      : "Copy code"}
                </Button>
                <Button type="button" variant="outline" className="min-h-11" onClick={reopenGitHub}>
                  <ExternalLink aria-hidden />
                  {browserState === "opened" ? "Open GitHub again" : "Open GitHub"}
                </Button>
              </div>
              {browserError && (
                <p className="mt-2 text-xs leading-relaxed text-destructive">
                  Could not open the browser: {browserError}
                </p>
              )}
            </div>
          )}

          {success && (
            <div className="flex gap-3 rounded-lg border border-success/25 bg-success/10 p-4">
              <CheckCircle2 className="mt-0.5 h-5 w-5 shrink-0 text-success" aria-hidden />
              <div>
                <p className="text-sm font-medium">Account added</p>
                <p className="mt-1 text-sm text-muted-foreground">
                  The account list has been refreshed from GitHub CLI.
                </p>
              </div>
            </div>
          )}

          {error && (
            <div className="flex gap-3 rounded-lg border border-destructive/30 bg-destructive/10 p-4">
              <XCircle className="mt-0.5 h-5 w-5 shrink-0 text-destructive" aria-hidden />
              <div>
                <p className="text-sm font-medium text-destructive">Sign-in did not finish</p>
                <p className="mt-1 break-words text-sm text-muted-foreground">{error}</p>
              </div>
            </div>
          )}
        </div>

        <footer className="flex flex-col-reverse gap-2 border-t border-border px-5 py-4 sm:flex-row sm:items-center sm:justify-between">
          <p className="flex items-center gap-1.5 text-xs text-muted-foreground">
            <ExternalLink className="h-3.5 w-3.5" aria-hidden />
            Authentication stays in GitHub CLI
          </p>
          {pending ? (
            <Button
              type="button"
              variant="ghost"
              className="min-h-11 sm:min-h-9"
              onClick={onCancel}
              disabled={canceling}
            >
              {canceling && <LoaderCircle className="animate-spin" aria-hidden />}
              {canceling ? "Canceling…" : "Cancel"}
            </Button>
          ) : (
            <Button className="min-h-11 sm:min-h-9" onClick={onClose} autoFocus>
              {success ? "Done" : "Close"}
            </Button>
          )}
        </footer>
      </section>
    </div>
  );
}

function errorMessage(error: unknown): string {
  if (error instanceof Error) return error.message;
  if (typeof error === "string") return error;
  return "An unknown error occurred.";
}

function IdentityMetric({
  label,
  value,
  tone,
}: {
  label: string;
  value: number;
  tone?: "success" | "warning";
}) {
  return (
    <div className="flex items-center justify-between border-b border-border px-5 py-3 last:border-b-0 sm:border-b-0">
      <span className="data-label">{label}</span>
      <span
        className={
          tone === "success"
            ? "font-mono text-lg text-success"
            : tone === "warning"
              ? "font-mono text-lg text-warning"
              : "font-mono text-lg"
        }
      >
        {String(value).padStart(2, "0")}
      </span>
    </div>
  );
}
