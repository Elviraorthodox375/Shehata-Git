import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { confirm as confirmDialog } from "@tauri-apps/plugin-dialog";
import {
  AlertTriangle,
  CheckCircle2,
  Download,
  Loader2,
  RefreshCw,
  ShieldCheck,
  XCircle,
} from "lucide-react";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { installPrerequisites, type PrerequisiteId, runDoctor } from "@/lib/tauri";
import type { CheckStatus, SystemCheck } from "@/lib/types";

const STATUS_META: Record<
  CheckStatus,
  { label: string; icon: typeof CheckCircle2; badge: "success" | "destructive" | "warning" }
> = {
  ready: { label: "Ready", icon: CheckCircle2, badge: "success" },
  missing: { label: "Missing", icon: XCircle, badge: "destructive" },
  needs_attention: { label: "Needs attention", icon: AlertTriangle, badge: "warning" },
};

const INSTALLABLE_CHECKS: Record<string, PrerequisiteId> = {
  git: "git",
  gh: "github_cli",
};

function AttentionCheckCard({ check }: { check: SystemCheck }) {
  const meta = STATUS_META[check.status];
  const Icon = meta.icon;
  return (
    <Card>
      <CardHeader className="flex-row items-start justify-between space-y-0">
        <div className="flex items-center gap-3">
          <Icon
            className={
              check.status === "ready"
                ? "h-5 w-5 text-success"
                : check.status === "missing"
                  ? "h-5 w-5 text-destructive"
                  : "h-5 w-5 text-warning"
            }
            aria-hidden
          />
          <div>
            <CardTitle>{check.label}</CardTitle>
            {check.version && (
              <CardDescription className="mt-0.5 font-mono text-xs">
                {check.version}
              </CardDescription>
            )}
          </div>
        </div>
        <Badge variant={meta.badge}>{meta.label}</Badge>
      </CardHeader>
      <CardContent className="space-y-2">
        <p className="text-sm text-muted-foreground">{check.detail}</p>
        {check.status !== "ready" && check.repair_hint && (
          <div className="rounded-[0.6rem] border border-warning/30 bg-warning/10 px-3 py-2">
            <p className="text-xs font-medium text-warning">How to fix</p>
            <p className="mt-0.5 text-sm text-foreground/90">{check.repair_hint}</p>
          </div>
        )}
      </CardContent>
    </Card>
  );
}

function ReadyCheck({ check }: { check: SystemCheck }) {
  return (
    <div className="group flex min-w-0 items-start gap-3 border-b border-border/70 px-4 py-3.5 last:border-b-0 sm:[&:nth-last-child(-n+2)]:border-b-0 sm:[&:nth-child(odd)]:border-r">
      <span className="mt-0.5 flex h-7 w-7 shrink-0 items-center justify-center rounded-full border border-success/25 bg-success/10 text-success">
        <CheckCircle2 className="h-4 w-4" aria-hidden />
      </span>
      <div className="min-w-0 flex-1">
        <div className="flex min-w-0 items-center justify-between gap-3">
          <p className="truncate text-sm font-semibold">{check.label}</p>
          <span className="shrink-0 font-mono text-[0.65rem] uppercase tracking-wider text-success">
            Ready
          </span>
        </div>
        <p
          className="mt-1 truncate text-xs leading-5 text-muted-foreground"
          title={check.version ?? check.detail}
        >
          {check.version ?? check.detail}
        </p>
      </div>
    </div>
  );
}

export function DoctorPage() {
  const queryClient = useQueryClient();
  const doctor = useQuery({ queryKey: ["doctor"], queryFn: runDoctor });
  const installable = (doctor.data?.checks ?? [])
    .filter((check) => check.status !== "ready" && INSTALLABLE_CHECKS[check.id])
    .map((check) => INSTALLABLE_CHECKS[check.id]);
  const readyChecks = doctor.data?.checks.filter((check) => check.status === "ready") ?? [];
  const attentionChecks = doctor.data?.checks.filter((check) => check.status !== "ready") ?? [];
  const setup = useMutation({
    mutationFn: installPrerequisites,
    onSuccess: async () => {
      await Promise.all([
        doctor.refetch(),
        queryClient.invalidateQueries({ queryKey: ["accounts"] }),
      ]);
    },
  });

  async function confirmAutomaticSetup() {
    const labels = installable.map((id) => (id === "git" ? "Git" : "GitHub CLI")).join(" and ");
    const approved = await confirmDialog(
      `Download and install ${labels} using Microsoft Windows Package Manager? Package and source agreements will be accepted for these exact packages only.`,
      { title: "Set up this PC", kind: "info" },
    );
    if (approved) setup.mutate(installable);
  }

  return (
    <div className="mx-auto max-w-3xl space-y-4">
      <div className="flex items-center justify-between gap-4">
        <div>
          {doctor.data && (
            <p className="text-sm text-muted-foreground">
              {doctor.data.healthy
                ? "Everything Shehata Git needs is in place."
                : "Some things need attention before Shehata Git can work fully."}
            </p>
          )}
          {doctor.data && (
            <p className="mt-0.5 font-mono text-xs text-muted-foreground/70">
              {doctor.data.os} · app v{doctor.data.app_version}
            </p>
          )}
        </div>
        <Button
          variant="outline"
          size="sm"
          onClick={() => doctor.refetch()}
          disabled={doctor.isFetching || setup.isPending}
        >
          <RefreshCw className={doctor.isFetching ? "animate-spin" : undefined} aria-hidden />
          Re-check
        </Button>
      </div>

      {installable.length > 0 && (
        <section className="liquid-hero overflow-hidden rounded-[0.9rem] border border-primary/25 p-5 sm:p-6">
          <div className="flex flex-col gap-5 sm:flex-row sm:items-center sm:justify-between">
            <div className="flex gap-4">
              <span className="flex h-11 w-11 shrink-0 items-center justify-center rounded-[0.7rem] border border-primary/25 bg-primary/10 text-primary">
                <Download className="h-5 w-5" aria-hidden />
              </span>
              <div>
                <p className="eyebrow">Automatic setup</p>
                <h2 className="mt-1 font-display text-xl font-semibold">
                  Let Shehata Git prepare this PC
                </h2>
                <p className="mt-2 max-w-xl text-sm leading-6 text-muted-foreground">
                  Downloads only the missing official Git tools through Microsoft WinGet, then
                  checks the system again. Windows may ask for permission.
                </p>
              </div>
            </div>
            <Button onClick={confirmAutomaticSetup} disabled={setup.isPending} className="shrink-0">
              {setup.isPending ? (
                <Loader2 className="animate-spin" aria-hidden />
              ) : (
                <ShieldCheck aria-hidden />
              )}
              {setup.isPending ? "Installing…" : "Set up this PC"}
            </Button>
          </div>
          {setup.isError && (
            <p className="mt-4 rounded-[0.55rem] border border-destructive/30 bg-destructive/10 p-3 text-sm text-destructive">
              {setup.error instanceof Error ? setup.error.message : String(setup.error)}
            </p>
          )}
          {setup.isSuccess && (
            <p className="mt-4 rounded-[0.55rem] border border-success/25 bg-success/10 p-3 text-sm text-success">
              Installed {setup.data.installed.map((tool) => tool.label).join(" and ")}. System check
              refreshed.
            </p>
          )}
        </section>
      )}

      {doctor.isLoading && <p className="text-sm text-muted-foreground">Checking your system…</p>}

      {doctor.isError && (
        <Card className="border-destructive/40">
          <CardHeader>
            <CardTitle className="text-destructive">System check failed</CardTitle>
            <CardDescription>
              {doctor.error instanceof Error
                ? doctor.error.message
                : "An unknown error occurred while running the system check."}
            </CardDescription>
          </CardHeader>
        </Card>
      )}

      {attentionChecks.length > 0 && (
        <div className="grid gap-3">
          {attentionChecks.map((check) => (
            <AttentionCheckCard key={check.id} check={check} />
          ))}
        </div>
      )}

      {readyChecks.length > 0 && (
        <section className="instrument-panel overflow-hidden rounded-[0.8rem]">
          <div className="flex items-center justify-between gap-4 border-b border-border px-4 py-3.5 sm:px-5">
            <div>
              <p className="eyebrow">Verified on this machine</p>
              <p className="mt-1 text-sm text-muted-foreground">
                {readyChecks.length} system checks passed
              </p>
            </div>
            <Badge variant="success">All ready</Badge>
          </div>
          <div className="grid sm:grid-cols-2">
            {readyChecks.map((check) => (
              <ReadyCheck key={check.id} check={check} />
            ))}
          </div>
        </section>
      )}
    </div>
  );
}
