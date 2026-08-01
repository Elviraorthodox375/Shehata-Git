import { useMutation, useQuery } from "@tanstack/react-query";
import {
  Bot,
  Check,
  CheckCircle2,
  ChevronDown,
  Copy,
  FileCode2,
  FolderGit2,
  Loader2,
  Radar,
  Search,
  ShieldCheck,
  TerminalSquare,
  XCircle,
} from "lucide-react";
import { useMemo, useState } from "react";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { generateRepositoryAgents, getMcpInfo, listRepositories } from "@/lib/tauri";
import type { RepositorySummary } from "@/lib/types";
import { cn } from "@/lib/utils";

const PERMISSIONS = [
  ["Inspect", "Repository status and branch metadata"],
  ["Prepare", "Stage, unstage, and create normal commits"],
  ["Sync", "Fast-forward pull and policy-checked normal push"],
  ["Blocked", "Force push, destructive reset, and token access"],
] as const;

export function AiIntegrationPage() {
  const mcp = useQuery({ queryKey: ["mcp-info"], queryFn: getMcpInfo });
  const repositories = useQuery({ queryKey: ["repositories"], queryFn: listRepositories });
  const [copied, setCopied] = useState(false);
  const [repositoryId, setRepositoryId] = useState("");
  const generateAgents = useMutation({ mutationFn: generateRepositoryAgents });
  const selectedId = repositoryId || repositories.data?.[0]?.id || "";
  const detectedCount = mcp.data?.detected_clients.filter((client) => client.available).length ?? 0;

  async function copyConfig() {
    if (!mcp.data) return;
    await navigator.clipboard.writeText(mcp.data.config_snippet);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  }

  return (
    <div className="mx-auto w-full max-w-6xl space-y-5">
      <section className="liquid-hero overflow-hidden rounded-[1rem]">
        <div className="grid gap-8 p-6 sm:p-8 lg:grid-cols-[minmax(0,1fr)_20rem] lg:items-end">
          <div>
            <div className="flex items-center gap-2 text-primary">
              <Radar className="h-4 w-4" aria-hidden />
              <p className="eyebrow">Local agent bridge</p>
            </div>
            <h2 className="mt-4 max-w-2xl font-display text-3xl font-semibold tracking-[-0.04em] sm:text-4xl">
              Give coding agents Git access without giving away control.
            </h2>
            <p className="mt-4 max-w-2xl text-sm leading-6 text-muted-foreground">
              Every operation passes through the same repository identity, push policy, and audit
              rules as the desktop app. Tokens stay inside GitHub CLI.
            </p>
          </div>
          <div className="grid grid-cols-2 overflow-hidden rounded-[0.8rem] border border-white/10 bg-background/20">
            <BridgeMetric label="SERVER" value={mcp.data?.available ? "READY" : "OFFLINE"} />
            <BridgeMetric label="CLIENTS" value={String(detectedCount).padStart(2, "0")} />
          </div>
        </div>
      </section>

      <div className="grid gap-5 lg:grid-cols-[minmax(0,1.25fr)_minmax(19rem,0.75fr)]">
        <Card>
          <CardHeader>
            <div className="flex flex-wrap items-center justify-between gap-3">
              <div className="flex items-center gap-2.5">
                <TerminalSquare className="h-5 w-5 text-primary" aria-hidden />
                <CardTitle>Detected coding clients</CardTitle>
              </div>
              <Badge variant={mcp.data?.available ? "success" : "warning"}>
                {mcp.data?.available ? (
                  <>
                    <CheckCircle2 className="h-3 w-3" aria-hidden /> bridge ready
                  </>
                ) : (
                  <>
                    <XCircle className="h-3 w-3" aria-hidden /> bridge not built
                  </>
                )}
              </Badge>
            </div>
            <CardDescription>
              Fixed, reviewed client checks only. No filesystem-wide scanning is performed.
            </CardDescription>
          </CardHeader>
          <CardContent className="space-y-3">
            <div className="grid gap-2 sm:grid-cols-2">
              {mcp.data?.detected_clients.map((client) => (
                <div
                  key={client.id}
                  className="flex min-h-20 items-center gap-3 rounded-[0.7rem] border border-white/10 bg-background/20 p-3"
                >
                  <span className="flex h-10 w-10 shrink-0 items-center justify-center rounded-[0.65rem] border border-white/10 bg-white/[0.04]">
                    <Bot className="h-4 w-4 text-primary" aria-hidden />
                  </span>
                  <div className="min-w-0 flex-1">
                    <p className="text-sm font-semibold">{client.name}</p>
                    <p className="mt-1 truncate font-mono text-[0.65rem] text-muted-foreground">
                      {client.available ? client.executable_path : "Not detected"}
                    </p>
                  </div>
                  <span
                    className={`h-2.5 w-2.5 rounded-full ${client.available ? "bg-success" : "bg-muted-foreground/30"}`}
                    role="img"
                    aria-label={client.available ? "Detected" : "Not detected"}
                  />
                </div>
              ))}
            </div>

            {mcp.data && (
              <div className="rounded-[0.7rem] border border-white/10 bg-background/25 p-3">
                <div className="mb-2 flex items-center justify-between gap-3">
                  <p className="data-label">MCP client configuration</p>
                  <Button variant="ghost" size="sm" onClick={copyConfig}>
                    {copied ? <Check aria-hidden /> : <Copy aria-hidden />}
                    {copied ? "Copied" : "Copy config"}
                  </Button>
                </div>
                <pre className="max-h-52 overflow-auto whitespace-pre-wrap break-all rounded-[0.55rem] border border-white/10 bg-background/45 p-3 font-mono text-[0.68rem] leading-5 text-muted-foreground">
                  {mcp.data.config_snippet}
                </pre>
              </div>
            )}
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <div className="flex items-center gap-2.5">
              <ShieldCheck className="h-5 w-5 text-primary" aria-hidden />
              <CardTitle>Permission envelope</CardTitle>
            </div>
            <CardDescription>What connected agents can and cannot request.</CardDescription>
          </CardHeader>
          <CardContent className="space-y-2">
            {PERMISSIONS.map(([label, detail]) => (
              <div key={label} className="border-b border-white/10 py-3 last:border-0">
                <div className="flex items-center justify-between gap-3">
                  <p className="text-sm font-semibold">{label}</p>
                  <Badge variant={label === "Blocked" ? "warning" : "secondary"}>
                    {label === "Blocked" ? "never exposed" : "guarded"}
                  </Badge>
                </div>
                <p className="mt-1 text-xs leading-5 text-muted-foreground">{detail}</p>
              </div>
            ))}
          </CardContent>
        </Card>
      </div>

      <Card>
        <CardHeader>
          <div className="flex items-center gap-2.5">
            <FileCode2 className="h-5 w-5 text-primary" aria-hidden />
            <CardTitle>Repository instructions</CardTitle>
          </div>
          <CardDescription>
            Add or update only Shehata Git's bounded section in AGENTS.md. Existing project
            instructions remain untouched.
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-3">
          <div className="flex flex-col gap-2 sm:flex-row">
            <RepositoryPicker
              repositories={repositories.data ?? []}
              selectedId={selectedId}
              onSelect={(id) => {
                setRepositoryId(id);
                generateAgents.reset();
              }}
              disabled={!repositories.data?.length || generateAgents.isPending}
            />
            <Button
              onClick={() => generateAgents.mutate(selectedId)}
              disabled={!selectedId || generateAgents.isPending}
            >
              {generateAgents.isPending ? (
                <Loader2 className="animate-spin" aria-hidden />
              ) : (
                <FileCode2 aria-hidden />
              )}
              Generate AGENTS.md
            </Button>
          </div>
          {generateAgents.data && (
            <p className="text-sm text-success">
              {generateAgents.data.created ? "Created" : "Updated"}: {generateAgents.data.path}
            </p>
          )}
          {generateAgents.isError && (
            <p className="text-sm text-destructive">
              {generateAgents.error instanceof Error
                ? generateAgents.error.message
                : String(generateAgents.error)}
            </p>
          )}
        </CardContent>
      </Card>
    </div>
  );
}

function RepositoryPicker({
  repositories,
  selectedId,
  onSelect,
  disabled,
}: {
  repositories: RepositorySummary[];
  selectedId: string;
  onSelect: (id: string) => void;
  disabled: boolean;
}) {
  const [open, setOpen] = useState(false);
  const [search, setSearch] = useState("");
  const selected = repositories.find((repository) => repository.id === selectedId);
  const filtered = useMemo(() => {
    const needle = search.trim().toLowerCase();
    if (!needle) return repositories;
    return repositories.filter((repository) =>
      [repository.display_name, repository.canonical_path, repository.owner, repository.repo_name]
        .filter(Boolean)
        .some((value) => value?.toLowerCase().includes(needle)),
    );
  }, [repositories, search]);

  return (
    <div className="relative min-w-0 flex-1">
      <button
        type="button"
        disabled={disabled}
        onClick={() => setOpen((value) => !value)}
        aria-haspopup="listbox"
        aria-expanded={open}
        className={cn(
          "glass-input flex min-h-11 w-full items-center gap-3 rounded-[0.65rem] px-3 text-left transition-all disabled:cursor-not-allowed disabled:opacity-50",
          open && "border-primary/55 shadow-[0_0_0_3px_hsl(var(--primary)/0.1)]",
        )}
      >
        <FolderGit2 className="h-4 w-4 shrink-0 text-primary" aria-hidden />
        <span className="min-w-0 flex-1">
          <span className="block truncate text-sm font-semibold">
            {selected?.display_name ?? "No registered repositories"}
          </span>
          {selected && (
            <span className="block truncate font-mono text-[0.65rem] text-muted-foreground">
              {selected.canonical_path}
            </span>
          )}
        </span>
        <ChevronDown
          className={cn(
            "h-4 w-4 shrink-0 text-muted-foreground transition-transform",
            open && "rotate-180",
          )}
          aria-hidden
        />
      </button>

      {open && (
        <div className="absolute left-0 right-0 top-[calc(100%+0.5rem)] z-40 overflow-hidden rounded-[0.8rem] border border-white/10 bg-surface-elevated/95 shadow-[0_24px_70px_rgba(0,0,0,0.55)] backdrop-blur-2xl">
          <div className="border-b border-white/10 p-2.5">
            <label className="glass-input flex min-h-11 items-center gap-2.5 px-3 focus-within:border-primary/45">
              <Search className="h-4 w-4 shrink-0 text-muted-foreground" aria-hidden />
              <span className="sr-only">Search repositories</span>
              <input
                type="search"
                value={search}
                onChange={(event) => setSearch(event.target.value)}
                placeholder="Search repositories…"
                className="min-w-0 flex-1 bg-transparent text-sm outline-none placeholder:text-muted-foreground/55 focus-visible:ring-0 focus-visible:ring-offset-0 [&::-webkit-search-cancel-button]:hidden"
              />
              <span className="font-mono text-[0.65rem] text-muted-foreground/60">
                {filtered.length}/{repositories.length}
              </span>
            </label>
          </div>
          <div className="scrollbar-thin max-h-64 overflow-y-auto p-2" role="listbox">
            {filtered.length ? (
              filtered.map((repository) => {
                const active = repository.id === selectedId;
                return (
                  <button
                    key={repository.id}
                    type="button"
                    role="option"
                    aria-selected={active}
                    onClick={() => {
                      onSelect(repository.id);
                      setOpen(false);
                      setSearch("");
                    }}
                    className={cn(
                      "flex min-h-14 w-full items-center gap-3 rounded-[0.6rem] px-3 py-2 text-left transition-colors",
                      active
                        ? "bg-primary/[0.11] text-foreground"
                        : "text-muted-foreground hover:bg-white/[0.04] hover:text-foreground",
                    )}
                  >
                    <span className="flex h-9 w-9 shrink-0 items-center justify-center rounded-[0.55rem] border border-white/10 bg-background/30">
                      <FolderGit2 className="h-4 w-4" aria-hidden />
                    </span>
                    <span className="min-w-0 flex-1">
                      <span className="block truncate text-sm font-semibold">
                        {repository.display_name}
                      </span>
                      <span className="block truncate font-mono text-[0.65rem] text-muted-foreground">
                        {repository.canonical_path}
                      </span>
                    </span>
                    {active && (
                      <span className="flex h-6 w-6 items-center justify-center rounded-full bg-primary text-primary-foreground">
                        <Check className="h-3.5 w-3.5" aria-hidden />
                      </span>
                    )}
                  </button>
                );
              })
            ) : (
              <p className="px-3 py-8 text-center text-sm text-muted-foreground">
                No repository matches “{search}”.
              </p>
            )}
          </div>
        </div>
      )}
    </div>
  );
}

function BridgeMetric({ label, value }: { label: string; value: string }) {
  return (
    <div className="border-r border-white/10 p-4 last:border-0">
      <p className="data-label">{label}</p>
      <p className="mt-2 font-mono text-lg font-semibold text-primary">{value}</p>
    </div>
  );
}
