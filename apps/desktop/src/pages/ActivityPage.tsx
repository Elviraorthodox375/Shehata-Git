import { useQuery } from "@tanstack/react-query";
import {
  CheckCircle2,
  Clock3,
  RefreshCw,
  ScrollText,
  Search,
  ShieldCheck,
  XCircle,
} from "lucide-react";
import { useMemo, useState } from "react";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardContent } from "@/components/ui/card";
import { listAuditEvents } from "@/lib/tauri";

type ResultFilter = "all" | "success" | "failed";

export function ActivityPage() {
  const events = useQuery({ queryKey: ["audit"], queryFn: listAuditEvents });
  const [search, setSearch] = useState("");
  const [result, setResult] = useState<ResultFilter>("all");
  const filtered = useMemo(() => {
    const needle = search.trim().toLowerCase();
    return (events.data ?? []).filter((event) => {
      const matchesResult =
        result === "all" ||
        (result === "success" ? event.result === "success" : event.result !== "success");
      const matchesSearch =
        !needle ||
        event.summary.toLowerCase().includes(needle) ||
        event.event_type.toLowerCase().includes(needle) ||
        event.account_login?.toLowerCase().includes(needle);
      return matchesResult && matchesSearch;
    });
  }, [events.data, result, search]);
  const successCount = events.data?.filter((event) => event.result === "success").length ?? 0;
  const failedCount = (events.data?.length ?? 0) - successCount;

  return (
    <div className="mx-auto w-full max-w-5xl space-y-5">
      <section className="liquid-hero overflow-hidden rounded-[1rem]">
        <div className="grid gap-6 p-6 sm:grid-cols-[minmax(0,1fr)_auto] sm:items-end sm:p-8">
          <div>
            <p className="eyebrow">Redacted local audit</p>
            <h2 className="mt-3 font-display text-3xl font-semibold tracking-[-0.04em]">
              Every guarded action leaves a safe trace.
            </h2>
            <p className="mt-3 max-w-2xl text-sm leading-6 text-muted-foreground">
              Tokens, environment values, source contents, and credential material are never written
              to this history.
            </p>
          </div>
          <div className="flex gap-2">
            <AuditMetric icon={CheckCircle2} label="SUCCESS" value={successCount} />
            <AuditMetric icon={XCircle} label="FAILED" value={failedCount} />
          </div>
        </div>
      </section>

      <div className="liquid-panel flex flex-col gap-3 rounded-[0.8rem] p-3 sm:flex-row sm:items-center">
        <label className="glass-input flex h-10 min-w-0 flex-1 items-center gap-2 rounded-[0.55rem] px-3">
          <Search className="h-4 w-4 text-muted-foreground" aria-hidden />
          <span className="sr-only">Search activity</span>
          <input
            value={search}
            onChange={(event) => setSearch(event.target.value)}
            placeholder="Search action, account, or event…"
            className="min-w-0 flex-1 bg-transparent text-sm outline-none placeholder:text-muted-foreground/60"
          />
        </label>
        <div className="flex gap-1 rounded-[0.55rem] border border-white/10 bg-background/20 p-1">
          {(["all", "success", "failed"] as const).map((option) => (
            <button
              key={option}
              type="button"
              onClick={() => setResult(option)}
              className={`min-h-8 rounded-[0.4rem] px-3 text-xs font-semibold capitalize transition ${result === option ? "bg-primary text-primary-foreground" : "text-muted-foreground hover:text-foreground"}`}
            >
              {option}
            </button>
          ))}
        </div>
        <Button
          variant="ghost"
          size="sm"
          onClick={() => events.refetch()}
          disabled={events.isFetching}
        >
          <RefreshCw className={events.isFetching ? "animate-spin" : undefined} aria-hidden />{" "}
          Refresh
        </Button>
      </div>

      {events.isError && (
        <Card className="border-destructive/40">
          <CardContent className="py-4 text-sm text-destructive">
            {events.error instanceof Error ? events.error.message : "Could not read activity."}
          </CardContent>
        </Card>
      )}

      {!events.isLoading && filtered.length === 0 && (
        <Card>
          <CardContent className="flex min-h-52 flex-col items-center justify-center gap-3 text-center">
            <ScrollText className="h-8 w-8 text-muted-foreground/45" aria-hidden />
            <div>
              <p className="font-medium">
                {events.data?.length ? "No matching activity" : "Nothing yet"}
              </p>
              <p className="mt-1 max-w-sm text-sm text-muted-foreground">
                {events.data?.length
                  ? "Try a different search or result filter."
                  : "Repository routing and Git actions will appear here."}
              </p>
            </div>
          </CardContent>
        </Card>
      )}

      <div className="relative space-y-3 before:absolute before:bottom-6 before:left-[1.45rem] before:top-6 before:w-px before:bg-white/10 sm:before:left-[1.95rem]">
        {filtered.map((event) => {
          const succeeded = event.result === "success";
          return (
            <article
              key={event.id}
              className="liquid-panel relative rounded-[0.8rem] p-4 pl-14 sm:p-5 sm:pl-20"
            >
              <span
                className={`absolute left-[0.95rem] top-5 z-10 flex h-8 w-8 items-center justify-center rounded-full border sm:left-[1.45rem] ${succeeded ? "border-success/30 bg-success/10 text-success" : "border-destructive/30 bg-destructive/10 text-destructive"}`}
              >
                {succeeded ? (
                  <ShieldCheck className="h-3.5 w-3.5" aria-hidden />
                ) : (
                  <XCircle className="h-3.5 w-3.5" aria-hidden />
                )}
              </span>
              <div className="flex flex-col gap-3 sm:flex-row sm:items-start sm:justify-between">
                <div className="min-w-0">
                  <div className="flex flex-wrap items-center gap-2">
                    <p className="text-sm font-semibold">{event.summary}</p>
                    <Badge variant={succeeded ? "success" : "destructive"}>{event.result}</Badge>
                  </div>
                  <p className="mt-2 font-mono text-[0.68rem] uppercase tracking-[0.08em] text-muted-foreground">
                    {event.event_type.replaceAll("_", " ")}
                    {event.account_login ? ` · @${event.account_login}` : ""}
                  </p>
                </div>
                <time className="flex shrink-0 items-center gap-1.5 font-mono text-[0.68rem] text-muted-foreground">
                  <Clock3 className="h-3 w-3" aria-hidden />
                  {new Date(event.timestamp).toLocaleString()}
                </time>
              </div>
            </article>
          );
        })}
      </div>
    </div>
  );
}

function AuditMetric({
  icon: Icon,
  label,
  value,
}: {
  icon: typeof CheckCircle2;
  label: string;
  value: number;
}) {
  return (
    <div className="min-w-24 rounded-[0.7rem] border border-white/10 bg-background/20 p-3">
      <div className="flex items-center gap-1.5 text-muted-foreground">
        <Icon className="h-3.5 w-3.5" aria-hidden />
        <span className="data-label">{label}</span>
      </div>
      <p className="mt-2 font-mono text-xl">{String(value).padStart(2, "0")}</p>
    </div>
  );
}
