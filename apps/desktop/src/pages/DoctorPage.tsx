import { useQuery } from "@tanstack/react-query";
import { AlertTriangle, CheckCircle2, RefreshCw, XCircle } from "lucide-react";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { runDoctor } from "@/lib/tauri";
import type { CheckStatus, SystemCheck } from "@/lib/types";

const STATUS_META: Record<
  CheckStatus,
  { label: string; icon: typeof CheckCircle2; badge: "success" | "destructive" | "warning" }
> = {
  ready: { label: "Ready", icon: CheckCircle2, badge: "success" },
  missing: { label: "Missing", icon: XCircle, badge: "destructive" },
  needs_attention: { label: "Needs attention", icon: AlertTriangle, badge: "warning" },
};

function CheckCard({ check }: { check: SystemCheck }) {
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
          <div className="rounded-md border border-warning/30 bg-warning/10 px-3 py-2">
            <p className="text-xs font-medium text-warning">How to fix</p>
            <p className="mt-0.5 text-sm text-foreground/90">{check.repair_hint}</p>
          </div>
        )}
      </CardContent>
    </Card>
  );
}

export function DoctorPage() {
  const doctor = useQuery({ queryKey: ["doctor"], queryFn: runDoctor });

  return (
    <div className="mx-auto max-w-2xl space-y-4">
      <div className="flex items-center justify-between">
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
          disabled={doctor.isFetching}
        >
          <RefreshCw className={doctor.isFetching ? "animate-spin" : undefined} aria-hidden />
          Re-check
        </Button>
      </div>

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

      <div className="grid gap-3">
        {doctor.data?.checks.map((check) => (
          <CheckCard key={check.id} check={check} />
        ))}
      </div>
    </div>
  );
}
