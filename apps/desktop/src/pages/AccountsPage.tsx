import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import {
  CheckCircle2,
  ExternalLink,
  KeyRound,
  LoaderCircle,
  Plus,
  RefreshCw,
  ShieldCheck,
  UserRound,
  XCircle,
} from "lucide-react";
import { useState } from "react";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { addAccount, listAccounts } from "@/lib/tauri";
import type { GhLoginEvent } from "@/lib/types";

/**
 * Accounts are discovered from the official GitHub CLI. Shehata Git never
 * receives a password and never sends a token across the Tauri boundary.
 */
export function AccountsPage() {
  const queryClient = useQueryClient();
  const [dialogOpen, setDialogOpen] = useState(false);
  const [loginEvent, setLoginEvent] = useState<GhLoginEvent | null>(null);
  const accounts = useQuery({ queryKey: ["accounts"], queryFn: listAccounts });
  const login = useMutation({
    mutationFn: () => addAccount(setLoginEvent),
    onSuccess: (data) => {
      queryClient.setQueryData(["accounts"], data);
      void queryClient.invalidateQueries({ queryKey: ["doctor"] });
    },
  });

  function startLogin() {
    login.reset();
    setLoginEvent(null);
    setDialogOpen(true);
    login.mutate();
  }

  return (
    <div className="mx-auto max-w-2xl space-y-4">
      <div className="flex flex-col gap-3 sm:flex-row sm:items-center sm:justify-between">
        <p className="max-w-lg text-sm leading-relaxed text-muted-foreground">
          Accounts stay in the official GitHub CLI. Shehata Git only checks which identities are
          available for repository routing.
        </p>
        <div className="flex flex-wrap gap-2">
          <Button
            variant="outline"
            size="sm"
            className="min-h-11 sm:min-h-8"
            onClick={() => accounts.refetch()}
            disabled={accounts.isFetching || login.isPending}
          >
            <RefreshCw className={accounts.isFetching ? "animate-spin" : undefined} aria-hidden />
            Refresh
          </Button>
          <Button
            size="sm"
            className="min-h-11 sm:min-h-8"
            onClick={startLogin}
            disabled={login.isPending}
          >
            <Plus aria-hidden />
            Add GitHub Account
          </Button>
        </div>
      </div>

      {accounts.isLoading && <p className="text-sm text-muted-foreground">Reading accounts…</p>}

      {accounts.isError && (
        <Card className="border-destructive/40">
          <CardHeader>
            <CardTitle className="text-destructive">Could not read accounts</CardTitle>
          </CardHeader>
          <CardContent>
            <p className="text-sm text-muted-foreground">{errorMessage(accounts.error)}</p>
          </CardContent>
        </Card>
      )}

      {accounts.data?.length === 0 && (
        <Card className="border-dashed">
          <CardContent className="flex flex-col items-center gap-4 py-10 text-center">
            <div className="flex h-12 w-12 items-center justify-center rounded-xl border border-primary/20 bg-primary/10">
              <UserRound className="h-6 w-6 text-primary" aria-hidden />
            </div>
            <div>
              <p className="font-medium">No GitHub accounts yet</p>
              <p className="mt-1 max-w-sm text-sm leading-relaxed text-muted-foreground">
                Add an account to open GitHub&apos;s browser sign-in. Your password and token never
                enter Shehata Git.
              </p>
            </div>
            <Button className="min-h-11" onClick={startLogin} disabled={login.isPending}>
              <Plus aria-hidden />
              Add your first account
            </Button>
          </CardContent>
        </Card>
      )}

      <div className="grid gap-3">
        {accounts.data?.map((account) => (
          <Card key={`${account.host}:${account.login}`}>
            <CardContent className="flex flex-col items-start gap-3 py-4 sm:flex-row sm:items-center">
              <div className="flex h-10 w-10 shrink-0 items-center justify-center rounded-full bg-secondary">
                <UserRound className="h-5 w-5 text-muted-foreground" aria-hidden />
              </div>
              <div className="min-w-0 flex-1">
                <p className="truncate font-medium">@{account.login}</p>
                <p className="font-mono text-xs text-muted-foreground">{account.host}</p>
              </div>
              <div className="flex flex-wrap items-center gap-2">
                {account.active && <Badge variant="secondary">active in gh</Badge>}
                <Badge variant={account.token_available ? "success" : "warning"}>
                  {account.token_available ? "token ok" : "token unavailable"}
                </Badge>
              </div>
            </CardContent>
          </Card>
        ))}
      </div>

      {dialogOpen && (
        <AccountLoginDialog
          event={loginEvent}
          pending={login.isPending}
          success={login.isSuccess}
          error={login.isError ? errorMessage(login.error) : null}
          onClose={() => setDialogOpen(false)}
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
}

function AccountLoginDialog({ event, pending, success, error, onClose }: AccountLoginDialogProps) {
  const code = event?.type === "code" ? event.code : null;

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
                Sign in with GitHub
              </h2>
              <p className="mt-1 text-sm leading-relaxed text-muted-foreground">
                GitHub CLI opens your browser. Shehata Git never sees your password or stores your
                token.
              </p>
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
                <p className="text-sm font-medium">Waiting for browser sign-in</p>
                <p className="mt-1 text-sm leading-relaxed text-muted-foreground">
                  Finish the GitHub page that just opened. Keep this window open while the CLI
                  verifies your account.
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
              <p className="mt-2 text-xs text-muted-foreground">
                The GitHub CLI also copied this code to your clipboard.
              </p>
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
          {!pending && (
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
