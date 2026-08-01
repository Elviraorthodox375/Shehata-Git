import { useQuery } from "@tanstack/react-query";
import {
  ArrowRight,
  Check,
  CircleDot,
  FolderGit2,
  GitBranch,
  ShieldCheck,
  TerminalSquare,
  Users,
} from "lucide-react";
import type { PageId } from "@/components/layout/Sidebar";
import { Button } from "@/components/ui/button";
import { listAccounts, listRepositories, runDoctor } from "@/lib/tauri";
import { cn } from "@/lib/utils";

interface HomePageProps {
  onNavigate: (page: PageId) => void;
}

export function HomePage({ onNavigate }: HomePageProps) {
  const doctor = useQuery({ queryKey: ["doctor"], queryFn: runDoctor });
  const accounts = useQuery({ queryKey: ["accounts"], queryFn: listAccounts });
  const repos = useQuery({ queryKey: ["repositories"], queryFn: listRepositories });

  const accountCount = accounts.data?.length ?? 0;
  const repoCount = repos.data?.length ?? 0;
  const assignedCount = repos.data?.filter((repo) => repo.assigned_login).length ?? 0;
  const systemHealthy = doctor.data?.healthy ?? false;

  return (
    <div className="mx-auto w-full max-w-6xl space-y-5">
      <section className="instrument-panel relative overflow-hidden rounded-[0.8rem]">
        <div className="absolute inset-y-0 left-0 w-0.5 bg-primary" aria-hidden />
        <div className="grid lg:grid-cols-[1.15fr_0.85fr]">
          <div className="px-5 py-6 sm:px-7 sm:py-8">
            <div className="mb-6 flex items-center gap-2">
              <span className="h-1.5 w-1.5 rounded-full bg-primary shadow-[0_0_14px_hsl(var(--primary)/0.8)]" />
              <span className="eyebrow">Identity routing workspace</span>
            </div>
            <h2 className="max-w-2xl font-display text-3xl font-semibold leading-[1.05] tracking-[-0.045em] sm:text-4xl">
              The right GitHub identity,
              <span className="block text-muted-foreground">locked to every repository.</span>
            </h2>
            <p className="mt-5 max-w-xl text-[0.95rem] leading-7 text-muted-foreground">
              Shehata Git sits between your local repositories and GitHub credentials. Configure a
              route once; terminals and coding agents inherit it automatically.
            </p>
            <div className="mt-7 flex flex-wrap gap-2">
              <Button onClick={() => onNavigate("repositories")}>
                <FolderGit2 aria-hidden />
                Configure a repository
              </Button>
              <Button variant="outline" onClick={() => onNavigate("doctor")}>
                Inspect system
              </Button>
            </div>
          </div>

          <RoutingDiagram accounts={accountCount} repos={repoCount} assigned={assignedCount} />
        </div>
      </section>

      <section className="instrument-panel grid overflow-hidden rounded-[0.7rem] md:grid-cols-3 md:divide-x md:divide-border">
        <Metric
          label="Runtime health"
          value={doctor.isLoading ? "Scanning" : systemHealthy ? "Operational" : "Attention"}
          detail={systemHealthy ? "Core prerequisites ready" : "Open system check"}
          icon={ShieldCheck}
          tone={systemHealthy ? "success" : "warning"}
        />
        <Metric
          label="Known identities"
          value={String(accountCount).padStart(2, "0")}
          detail="From official GitHub CLI"
          icon={Users}
        />
        <Metric
          label="Active routes"
          value={`${String(assignedCount).padStart(2, "0")} / ${String(repoCount).padStart(2, "0")}`}
          detail="Assigned / registered repos"
          icon={GitBranch}
        />
      </section>

      <div className="grid gap-5 lg:grid-cols-[1fr_20rem]">
        <section className="instrument-panel rounded-[0.7rem] p-5 sm:p-6">
          <div className="mb-5 flex items-end justify-between gap-4">
            <div>
              <p className="eyebrow">Commissioning sequence</p>
              <h3 className="mt-1 font-display text-lg font-semibold">Bring routing online</h3>
            </div>
            <span className="font-mono text-xs text-muted-foreground">
              {
                [systemHealthy, accountCount > 0, repoCount > 0, assignedCount > 0].filter(Boolean)
                  .length
              }
              /4 complete
            </span>
          </div>
          <div className="divide-y divide-border/70 border-y border-border/70">
            <SetupRow
              code="SYS"
              done={systemHealthy}
              title="Validate local runtime"
              detail="Git, GitHub CLI, database and helper binaries"
              action="Run doctor"
              onAction={() => onNavigate("doctor")}
            />
            <SetupRow
              code="ID"
              done={accountCount > 0}
              title="Register GitHub identities"
              detail="Authentication stays inside the official gh credential store"
              action="View identities"
              onAction={() => onNavigate("accounts")}
            />
            <SetupRow
              code="REP"
              done={repoCount > 0}
              title="Inspect a local repository"
              detail="Read its branch, remote and local author configuration"
              action="Add repository"
              onAction={() => onNavigate("repositories")}
            />
            <SetupRow
              code="MAP"
              done={assignedCount > 0}
              title="Lock repository identity"
              detail="Assign one account and an optional local commit author"
              action="Configure route"
              onAction={() => onNavigate("repositories")}
            />
          </div>
        </section>

        <aside className="instrument-panel rounded-[0.7rem] p-5">
          <div className="flex items-center justify-between">
            <p className="eyebrow">Trust boundary</p>
            <TerminalSquare className="h-4 w-4 text-primary" aria-hidden />
          </div>
          <div className="mt-5 space-y-5">
            <TrustItem
              number="01"
              title="No token database"
              text="Tokens stay in GitHub CLI and memory only."
            />
            <TrustItem
              number="02"
              title="Repository-local"
              text="No global Git identity is silently changed."
            />
            <TrustItem
              number="03"
              title="Fail closed"
              text="A missing mapping never falls through to another account."
            />
          </div>
        </aside>
      </div>
    </div>
  );
}

function RoutingDiagram({
  accounts,
  repos,
  assigned,
}: {
  accounts: number;
  repos: number;
  assigned: number;
}) {
  return (
    <div className="relative flex min-h-64 items-center justify-center overflow-hidden border-t border-border/70 bg-background/25 p-6 lg:border-l lg:border-t-0">
      <div className="absolute inset-0 opacity-40 [background-image:linear-gradient(hsl(var(--border)/0.35)_1px,transparent_1px),linear-gradient(90deg,hsl(var(--border)/0.35)_1px,transparent_1px)] [background-size:1.5rem_1.5rem]" />
      <div className="relative grid w-full max-w-sm grid-cols-[1fr_4rem_1fr] items-center">
        <div className="space-y-3">
          <DiagramNode icon={Users} label="IDENTITIES" value={accounts} />
          <DiagramNode icon={TerminalSquare} label="TOOLS" value="∞" />
        </div>
        <div className="relative h-px bg-border">
          <span className="absolute -top-1 left-1/2 h-2 w-2 -translate-x-1/2 rotate-45 border border-primary bg-background" />
        </div>
        <div className="border border-primary/35 bg-primary/[0.07] p-4 shadow-[0_0_35px_hsl(var(--primary)/0.06)]">
          <p className="data-label">ROUTED REPOS</p>
          <p className="mt-2 font-mono text-3xl font-medium tabular-nums text-primary">
            {String(assigned).padStart(2, "0")}
          </p>
          <p className="mt-1 font-mono text-[0.65rem] text-muted-foreground">{`of ${repos} registered`}</p>
        </div>
      </div>
    </div>
  );
}

function DiagramNode({
  icon: Icon,
  label,
  value,
}: {
  icon: typeof Users;
  label: string;
  value: number | string;
}) {
  return (
    <div className="flex items-center gap-3 border border-border bg-card/90 p-3">
      <Icon className="h-4 w-4 text-muted-foreground" aria-hidden />
      <div>
        <p className="data-label">{label}</p>
        <p className="font-mono text-sm text-foreground">{value}</p>
      </div>
    </div>
  );
}

function Metric({
  label,
  value,
  detail,
  icon: Icon,
  tone,
}: {
  label: string;
  value: string;
  detail: string;
  icon: typeof ShieldCheck;
  tone?: "success" | "warning";
}) {
  return (
    <div className="flex min-h-28 items-start gap-4 border-b border-border p-5 last:border-b-0 md:border-b-0">
      <div className="flex h-9 w-9 items-center justify-center border border-border bg-background/40">
        <Icon className="h-4 w-4 text-muted-foreground" aria-hidden />
      </div>
      <div className="min-w-0">
        <p className="data-label">{label}</p>
        <p
          className={cn(
            "mt-1 font-display text-xl font-semibold tracking-tight",
            tone === "success" && "text-success",
            tone === "warning" && "text-warning",
          )}
        >
          {value}
        </p>
        <p className="mt-1 truncate text-xs text-muted-foreground">{detail}</p>
      </div>
    </div>
  );
}

function SetupRow({
  code,
  done,
  title,
  detail,
  action,
  onAction,
}: {
  code: string;
  done: boolean;
  title: string;
  detail: string;
  action: string;
  onAction: () => void;
}) {
  return (
    <div className="grid gap-3 py-4 sm:grid-cols-[3rem_1fr_auto] sm:items-center">
      <span
        className={cn(
          "flex h-8 w-8 items-center justify-center border font-mono text-[0.65rem]",
          done
            ? "border-success/30 bg-success/10 text-success"
            : "border-border text-muted-foreground",
        )}
      >
        {done ? <Check className="h-4 w-4" aria-hidden /> : code}
      </span>
      <div className="min-w-0">
        <p className="text-sm font-semibold">{title}</p>
        <p className="mt-0.5 truncate text-xs text-muted-foreground">{detail}</p>
      </div>
      {!done && (
        <Button variant="ghost" size="sm" className="w-fit" onClick={onAction}>
          {action}
          <ArrowRight aria-hidden />
        </Button>
      )}
      {done && (
        <span className="flex items-center gap-1.5 font-mono text-[0.65rem] uppercase tracking-wider text-success">
          <CircleDot className="h-3 w-3" aria-hidden /> complete
        </span>
      )}
    </div>
  );
}

function TrustItem({ number, title, text }: { number: string; title: string; text: string }) {
  return (
    <div className="grid grid-cols-[2rem_1fr] gap-3">
      <span className="font-mono text-xs text-primary">{number}</span>
      <div>
        <p className="text-sm font-semibold">{title}</p>
        <p className="mt-1 text-xs leading-5 text-muted-foreground">{text}</p>
      </div>
    </div>
  );
}
