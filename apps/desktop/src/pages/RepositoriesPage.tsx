import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { confirm as confirmDialog, open } from "@tauri-apps/plugin-dialog";
import {
  AlertCircle,
  ArrowRight,
  CheckCircle2,
  FolderGit2,
  FolderOpen,
  GitBranch,
  Globe,
  KeyRound,
  Loader2,
  PlugZap,
  RefreshCw,
  ShieldCheck,
  Unplug,
  UserRound,
  X,
} from "lucide-react";
import { useState } from "react";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import {
  addRepository,
  assignRepository,
  linkRepository,
  listAccounts,
  listRepositories,
  testRepositoryConnection,
  unlinkRepository,
} from "@/lib/tauri";
import type { GhAccount, RepositorySummary } from "@/lib/types";
import { cn } from "@/lib/utils";

export function RepositoriesPage() {
  const queryClient = useQueryClient();
  const [selectedRepo, setSelectedRepo] = useState<RepositorySummary | null>(null);
  const [assignmentNotice, setAssignmentNotice] = useState<string | null>(null);
  const repos = useQuery({ queryKey: ["repositories"], queryFn: listRepositories });
  const accounts = useQuery({ queryKey: ["accounts"], queryFn: listAccounts });
  const addRepo = useMutation({
    mutationFn: addRepository,
    onSuccess: async () => {
      await queryClient.invalidateQueries({ queryKey: ["repositories"] });
    },
  });
  const assign = useMutation({
    mutationFn: assignRepository,
    onSuccess: async (result) => {
      await queryClient.invalidateQueries({ queryKey: ["repositories"] });
      setAssignmentNotice(
        `${result.repository.display_name} is now locked to @${result.repository.assigned_login}.`,
      );
      setSelectedRepo(null);
    },
  });
  const link = useMutation({
    mutationFn: linkRepository,
    onSuccess: async (result) => {
      await queryClient.invalidateQueries({ queryKey: ["repositories"] });
      setAssignmentNotice(`Credential routing is active for repository ${result.repository_id}.`);
    },
  });
  const connectionTest = useMutation({
    mutationFn: testRepositoryConnection,
    onSuccess: (result) => {
      setAssignmentNotice(
        `Connection verified through @${result.account_login} on ${result.remote_name}.`,
      );
    },
  });
  const unlink = useMutation({
    mutationFn: (repositoryId: string) => unlinkRepository(repositoryId, false),
    onSuccess: async () => {
      await queryClient.invalidateQueries({ queryKey: ["repositories"] });
      setAssignmentNotice("Repository routing was removed and original Git settings restored.");
    },
  });

  async function confirmUnlink(repo: RepositorySummary) {
    const approved = await confirmDialog(
      `Unlink ${repo.display_name}? Credential settings will be restored; local commit identity will be kept.`,
      { title: "Unlink repository", kind: "warning" },
    );
    if (approved) unlink.mutate(repo.id);
  }

  async function chooseRepository() {
    addRepo.reset();
    setAssignmentNotice(null);
    const selected = await open({
      directory: true,
      multiple: false,
      title: "Choose a Git repository",
    });
    if (selected) addRepo.mutate(selected);
  }

  const assignedCount = repos.data?.filter((repo) => repo.assigned_login).length ?? 0;

  return (
    <div className="mx-auto w-full max-w-6xl space-y-5">
      <section className="instrument-panel overflow-hidden rounded-[0.75rem]">
        <div className="flex flex-col gap-5 p-5 sm:p-6 lg:flex-row lg:items-end lg:justify-between">
          <div>
            <p className="eyebrow">Repository registry / local machine</p>
            <h2 className="mt-2 font-display text-2xl font-semibold tracking-tight">
              Identity routes begin here.
            </h2>
            <p className="mt-2 max-w-2xl text-sm leading-6 text-muted-foreground">
              Inspection is read-only. Git configuration changes only after you review and confirm
              an account assignment.
            </p>
          </div>
          <div className="flex flex-wrap gap-2">
            <Button variant="outline" onClick={() => repos.refetch()} disabled={repos.isFetching}>
              <RefreshCw className={repos.isFetching ? "animate-spin" : undefined} aria-hidden />
              Rescan
            </Button>
            <Button onClick={chooseRepository} disabled={addRepo.isPending}>
              {addRepo.isPending ? (
                <Loader2 className="animate-spin" aria-hidden />
              ) : (
                <FolderOpen aria-hidden />
              )}
              {addRepo.isPending ? "Inspecting…" : "Add repository"}
            </Button>
          </div>
        </div>
        <div className="grid border-t border-border bg-background/20 sm:grid-cols-3 sm:divide-x sm:divide-border">
          <RegistryMetric label="REGISTERED" value={repos.data?.length ?? 0} />
          <RegistryMetric label="ASSIGNED" value={assignedCount} tone="success" />
          <RegistryMetric
            label="AWAITING ROUTE"
            value={(repos.data?.length ?? 0) - assignedCount}
            tone="warning"
          />
        </div>
      </section>

      {assignmentNotice && (
        <div className="flex items-center justify-between gap-4 border border-success/25 bg-success/[0.07] px-4 py-3 text-sm">
          <span className="flex items-center gap-2 text-success">
            <CheckCircle2 className="h-4 w-4" aria-hidden />
            {assignmentNotice}
          </span>
          <button
            type="button"
            className="text-muted-foreground hover:text-foreground"
            onClick={() => setAssignmentNotice(null)}
            aria-label="Dismiss message"
          >
            <X className="h-4 w-4" aria-hidden />
          </button>
        </div>
      )}

      {(repos.isError ||
        addRepo.isError ||
        link.isError ||
        connectionTest.isError ||
        unlink.isError) && (
        <div className="flex gap-3 border border-destructive/35 bg-destructive/[0.06] p-4">
          <AlertCircle className="mt-0.5 h-5 w-5 shrink-0 text-destructive" aria-hidden />
          <div>
            <p className="text-sm font-semibold text-destructive">Repository inspection failed</p>
            <p className="mt-1 text-sm text-muted-foreground">
              {errorMessage(
                addRepo.error ?? repos.error ?? link.error ?? connectionTest.error ?? unlink.error,
              )}
            </p>
          </div>
        </div>
      )}

      {repos.isLoading && (
        <div className="flex items-center gap-2 py-8 text-sm text-muted-foreground">
          <Loader2 className="h-4 w-4 animate-spin" aria-hidden /> Reading repository registry…
        </div>
      )}

      {repos.data?.length === 0 && (
        <section className="relative flex min-h-72 flex-col items-center justify-center overflow-hidden border border-dashed border-border bg-card/45 p-8 text-center">
          <span className="absolute left-0 top-0 h-5 w-5 border-l border-t border-primary/50" />
          <span className="absolute right-0 top-0 h-5 w-5 border-r border-t border-primary/50" />
          <span className="absolute bottom-0 left-0 h-5 w-5 border-b border-l border-primary/50" />
          <span className="absolute bottom-0 right-0 h-5 w-5 border-b border-r border-primary/50" />
          <div className="flex h-14 w-14 items-center justify-center border border-border bg-background/50">
            <FolderGit2 className="h-6 w-6 text-primary" aria-hidden />
          </div>
          <p className="eyebrow mt-5">Registry empty</p>
          <h3 className="mt-2 font-display text-xl font-semibold">Inspect your first repository</h3>
          <p className="mt-2 max-w-md text-sm leading-6 text-muted-foreground">
            Select a project folder. Shehata Git reads its Git metadata and stores no source files.
          </p>
          <Button className="mt-6" onClick={chooseRepository} disabled={addRepo.isPending}>
            <FolderOpen aria-hidden /> Choose repository folder
          </Button>
        </section>
      )}

      <div className="space-y-3">
        {repos.data?.map((repo, index) => (
          <RepositoryRow
            key={repo.id}
            repo={repo}
            index={index}
            onAssign={() => {
              assign.reset();
              setSelectedRepo(repo);
            }}
            onLink={() => link.mutate(repo.id)}
            onTest={() => connectionTest.mutate(repo.id)}
            onUnlink={() => confirmUnlink(repo)}
            pending={
              (link.isPending && link.variables === repo.id) ||
              (connectionTest.isPending && connectionTest.variables === repo.id) ||
              (unlink.isPending && unlink.variables === repo.id)
            }
          />
        ))}
      </div>

      {selectedRepo && (
        <AssignmentDialog
          repo={selectedRepo}
          accounts={accounts.data ?? []}
          pending={assign.isPending}
          error={assign.isError ? errorMessage(assign.error) : null}
          onClose={() => setSelectedRepo(null)}
          onSubmit={(account, commitName, commitEmail) =>
            assign.mutate({
              repository_id: selectedRepo.id,
              host: account.host,
              login: account.login,
              commit_name: commitName.trim() || null,
              commit_email: commitEmail.trim() || null,
            })
          }
        />
      )}
    </div>
  );
}

function RegistryMetric({
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
        className={cn(
          "font-mono text-lg tabular-nums",
          tone === "success" && "text-success",
          tone === "warning" && "text-warning",
        )}
      >
        {String(value).padStart(2, "0")}
      </span>
    </div>
  );
}

function RepositoryRow({
  repo,
  index,
  onAssign,
  onLink,
  onTest,
  onUnlink,
  pending,
}: {
  repo: RepositorySummary;
  index: number;
  onAssign: () => void;
  onLink: () => void;
  onTest: () => void;
  onUnlink: () => void;
  pending: boolean;
}) {
  return (
    <article className="instrument-panel group relative overflow-hidden rounded-[0.7rem] transition-colors hover:border-muted-foreground/35">
      <span
        className={cn(
          "absolute inset-y-0 left-0 w-0.5",
          repo.assigned_login ? "bg-success" : "bg-warning",
        )}
      />
      <div className="grid gap-5 p-5 lg:grid-cols-[minmax(0,1fr)_15rem_auto] lg:items-center">
        <div className="flex min-w-0 items-start gap-4">
          <span className="font-mono text-[0.65rem] text-muted-foreground/50">
            {String(index + 1).padStart(2, "0")}
          </span>
          <div className="flex h-10 w-10 shrink-0 items-center justify-center border border-border bg-background/35">
            <FolderGit2 className="h-4 w-4 text-primary" aria-hidden />
          </div>
          <div className="min-w-0">
            <div className="flex flex-wrap items-center gap-2">
              <h3 className="font-display font-semibold tracking-tight">{repo.display_name}</h3>
              {repo.remote_protocol && (
                <Badge variant={repo.remote_protocol === "https" ? "secondary" : "warning"}>
                  {repo.remote_protocol.toUpperCase()}
                </Badge>
              )}
            </div>
            <p className="mt-1 truncate font-mono text-[0.7rem] text-muted-foreground">
              {repo.canonical_path}
            </p>
          </div>
        </div>

        <div className="grid grid-cols-2 gap-3 lg:grid-cols-1">
          <div className="flex min-w-0 items-center gap-2 text-xs text-muted-foreground">
            <GitBranch className="h-3.5 w-3.5 shrink-0" aria-hidden />
            <span className="truncate">{repo.current_branch ?? "No commits"}</span>
          </div>
          <div className="flex min-w-0 items-center gap-2 text-xs text-muted-foreground">
            <Globe className="h-3.5 w-3.5 shrink-0" aria-hidden />
            <span className="truncate">
              {repo.owner && repo.repo_name
                ? `${repo.owner}/${repo.repo_name}`
                : "Remote unavailable"}
            </span>
          </div>
        </div>

        <div className="flex flex-wrap items-center justify-between gap-3 lg:justify-end">
          {repo.assigned_login ? (
            <div className="text-left lg:text-right">
              <p className="data-label">
                {repo.routing_configured ? "ROUTE ACTIVE" : "IDENTITY ONLY"}
              </p>
              <p
                className={cn(
                  "mt-1 text-sm font-semibold",
                  repo.routing_configured ? "text-success" : "text-warning",
                )}
              >
                @{repo.assigned_login}
              </p>
            </div>
          ) : (
            <div className="text-left lg:text-right">
              <p className="data-label">ROUTING STATE</p>
              <p className="mt-1 text-sm font-semibold text-warning">Unassigned</p>
            </div>
          )}
          <div className="flex flex-wrap justify-end gap-2">
            {repo.assigned_login &&
              repo.remote_protocol === "https" &&
              (!repo.routing_configured ? (
                <Button size="sm" onClick={onLink} disabled={pending}>
                  {pending ? (
                    <Loader2 className="animate-spin" aria-hidden />
                  ) : (
                    <PlugZap aria-hidden />
                  )}
                  Enable route
                </Button>
              ) : (
                <>
                  <Button size="sm" variant="outline" onClick={onTest} disabled={pending}>
                    {pending ? (
                      <Loader2 className="animate-spin" aria-hidden />
                    ) : (
                      <ShieldCheck aria-hidden />
                    )}
                    Verify
                  </Button>
                  <Button size="sm" variant="ghost" onClick={onUnlink} disabled={pending}>
                    <Unplug aria-hidden /> Unlink
                  </Button>
                </>
              ))}
            <Button
              size="sm"
              variant={repo.assigned_login ? "outline" : "default"}
              onClick={onAssign}
              disabled={pending}
            >
              <KeyRound aria-hidden />
              {repo.assigned_login ? "Edit" : "Assign identity"}
              {!repo.assigned_login && <ArrowRight aria-hidden />}
            </Button>
          </div>
        </div>
      </div>
    </article>
  );
}

function AssignmentDialog({
  repo,
  accounts,
  pending,
  error,
  onClose,
  onSubmit,
}: {
  repo: RepositorySummary;
  accounts: GhAccount[];
  pending: boolean;
  error: string | null;
  onClose: () => void;
  onSubmit: (account: GhAccount, commitName: string, commitEmail: string) => void;
}) {
  const available = accounts.filter(
    (account) => account.token_available && (!repo.host || account.host === repo.host),
  );
  const initial =
    available.find((account) => account.login === repo.assigned_login) ?? available[0];
  const [selectedKey, setSelectedKey] = useState(initial ? `${initial.host}:${initial.login}` : "");
  const [commitName, setCommitName] = useState(repo.commit_name ?? "");
  const [commitEmail, setCommitEmail] = useState(repo.commit_email ?? "");
  const selected = available.find((account) => `${account.host}:${account.login}` === selectedKey);

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-background/85 p-4 backdrop-blur-sm">
      <section
        role="dialog"
        aria-modal="true"
        aria-labelledby="assignment-title"
        className="instrument-panel max-h-[calc(100vh-2rem)] w-full max-w-2xl overflow-y-auto rounded-[0.8rem]"
      >
        <header className="flex items-start justify-between gap-5 border-b border-border p-5 sm:p-6">
          <div className="flex gap-4">
            <div className="flex h-11 w-11 shrink-0 items-center justify-center border border-primary/30 bg-primary/[0.08]">
              <ShieldCheck className="h-5 w-5 text-primary" aria-hidden />
            </div>
            <div>
              <p className="eyebrow">Phase 5 / repository assignment</p>
              <h2 id="assignment-title" className="mt-1 font-display text-xl font-semibold">
                Lock identity for {repo.display_name}
              </h2>
              <p className="mt-1 text-sm text-muted-foreground">
                Remote: {repo.host ?? "unknown"}/{repo.owner ?? "—"}/{repo.repo_name ?? "—"}
              </p>
            </div>
          </div>
          <button
            type="button"
            className="flex h-10 w-10 items-center justify-center border border-transparent text-muted-foreground hover:border-border hover:text-foreground"
            onClick={onClose}
            disabled={pending}
            aria-label="Close assignment"
          >
            <X className="h-4 w-4" aria-hidden />
          </button>
        </header>

        <form
          onSubmit={(event) => {
            event.preventDefault();
            if (selected) onSubmit(selected, commitName, commitEmail);
          }}
        >
          <div className="space-y-6 p-5 sm:p-6">
            <fieldset>
              <legend className="data-label">01 / GitHub identity</legend>
              {available.length === 0 ? (
                <div className="mt-3 flex gap-3 border border-warning/30 bg-warning/[0.06] p-4">
                  <AlertCircle className="mt-0.5 h-4 w-4 shrink-0 text-warning" aria-hidden />
                  <div>
                    <p className="text-sm font-semibold">
                      No usable account for {repo.host ?? "this remote"}
                    </p>
                    <p className="mt-1 text-xs leading-5 text-muted-foreground">
                      Add or refresh a GitHub account first. Token values never enter this dialog.
                    </p>
                  </div>
                </div>
              ) : (
                <div className="mt-3 grid gap-2 sm:grid-cols-2">
                  {available.map((account) => {
                    const key = `${account.host}:${account.login}`;
                    const active = key === selectedKey;
                    return (
                      <label
                        key={key}
                        className={cn(
                          "flex cursor-pointer items-center gap-3 border p-3 transition-colors",
                          active
                            ? "border-primary/45 bg-primary/[0.07]"
                            : "border-border bg-background/25 hover:border-muted-foreground/40",
                        )}
                      >
                        <input
                          type="radio"
                          name="account"
                          value={key}
                          checked={active}
                          onChange={() => setSelectedKey(key)}
                          className="sr-only"
                        />
                        <span className="flex h-9 w-9 items-center justify-center border border-border bg-card">
                          <UserRound className="h-4 w-4 text-muted-foreground" aria-hidden />
                        </span>
                        <span className="min-w-0 flex-1">
                          <span className="block truncate text-sm font-semibold">
                            @{account.login}
                          </span>
                          <span className="block font-mono text-[0.65rem] text-muted-foreground">
                            {account.host}
                          </span>
                        </span>
                        <span
                          className={cn(
                            "h-2.5 w-2.5 rounded-full border",
                            active ? "border-primary bg-primary" : "border-muted-foreground",
                          )}
                        />
                      </label>
                    );
                  })}
                </div>
              )}
            </fieldset>

            <fieldset className="border-t border-border pt-6">
              <legend className="data-label">02 / Local commit author</legend>
              <p className="mt-2 text-xs leading-5 text-muted-foreground">
                Optional. These values are written to this repository only. Existing values are
                backed up before they change.
              </p>
              <div className="mt-4 grid gap-4 sm:grid-cols-2">
                <label className="space-y-2 text-sm font-medium">
                  <span>Author name</span>
                  <input
                    value={commitName}
                    onChange={(event) => setCommitName(event.target.value)}
                    maxLength={128}
                    placeholder="e.g. Mohamed Shehata"
                    className="h-11 w-full rounded-[0.5rem] border border-input bg-background/45 px-3 text-sm outline-none transition-colors placeholder:text-muted-foreground/50 focus:border-primary focus:ring-2 focus:ring-primary/15"
                  />
                </label>
                <label className="space-y-2 text-sm font-medium">
                  <span>Author email</span>
                  <input
                    type="email"
                    value={commitEmail}
                    onChange={(event) => setCommitEmail(event.target.value)}
                    maxLength={254}
                    placeholder="name@example.com"
                    className="h-11 w-full rounded-[0.5rem] border border-input bg-background/45 px-3 text-sm outline-none transition-colors placeholder:text-muted-foreground/50 focus:border-primary focus:ring-2 focus:ring-primary/15"
                  />
                </label>
              </div>
            </fieldset>

            {repo.remote_protocol === "ssh" && (
              <div className="flex gap-3 border border-warning/25 bg-warning/[0.05] p-3 text-xs leading-5 text-muted-foreground">
                <AlertCircle className="mt-0.5 h-4 w-4 shrink-0 text-warning" aria-hidden />
                Account assignment will be saved, but automatic credential routing needs an HTTPS
                remote in Phase 6.
              </div>
            )}

            {error && (
              <div className="flex gap-3 border border-destructive/30 bg-destructive/[0.06] p-3">
                <AlertCircle className="mt-0.5 h-4 w-4 shrink-0 text-destructive" aria-hidden />
                <p className="text-sm text-destructive">{error}</p>
              </div>
            )}
          </div>

          <footer className="flex flex-col-reverse gap-3 border-t border-border bg-background/20 p-5 sm:flex-row sm:items-center sm:justify-between sm:px-6">
            <p className="flex items-center gap-2 text-xs text-muted-foreground">
              <KeyRound className="h-3.5 w-3.5" aria-hidden />
              Original Git identity remains restorable
            </p>
            <div className="flex gap-2">
              <Button type="button" variant="ghost" onClick={onClose} disabled={pending}>
                Cancel
              </Button>
              <Button type="submit" disabled={!selected || pending}>
                {pending ? (
                  <Loader2 className="animate-spin" aria-hidden />
                ) : (
                  <ShieldCheck aria-hidden />
                )}
                {pending ? "Applying…" : "Confirm assignment"}
              </Button>
            </div>
          </footer>
        </form>
      </section>
    </div>
  );
}

function errorMessage(error: unknown): string {
  if (error instanceof Error) return error.message;
  if (typeof error === "string") return error;
  return "Choose a valid Git repository folder and try again.";
}
