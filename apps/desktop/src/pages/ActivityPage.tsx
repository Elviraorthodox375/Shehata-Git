import { useQuery } from "@tanstack/react-query";
import { ScrollText } from "lucide-react";
import { Badge } from "@/components/ui/badge";
import { Card, CardContent } from "@/components/ui/card";
import { listAuditEvents } from "@/lib/tauri";

/**
 * Activity page — the safe audit log.
 * Never contains tokens, credentials, environment dumps, or file contents.
 */
export function ActivityPage() {
  const events = useQuery({ queryKey: ["audit"], queryFn: listAuditEvents });

  return (
    <div className="mx-auto max-w-2xl space-y-4">
      <p className="text-sm text-muted-foreground">
        A safe history of what Shehata Git did. Secrets are never recorded here.
      </p>

      {events.isLoading && <p className="text-sm text-muted-foreground">Reading activity…</p>}

      {events.isError && (
        <Card className="border-destructive/40">
          <CardContent className="py-4">
            <p className="text-sm text-destructive">
              {events.error instanceof Error ? events.error.message : "Could not read activity."}
            </p>
          </CardContent>
        </Card>
      )}

      {events.data?.length === 0 && (
        <Card>
          <CardContent className="flex flex-col items-center gap-3 py-10 text-center">
            <ScrollText className="h-8 w-8 text-muted-foreground/50" aria-hidden />
            <div>
              <p className="font-medium">Nothing yet</p>
              <p className="mt-1 max-w-sm text-sm text-muted-foreground">
                Actions like linking a repository, testing a connection, or pushing will show up
                here with their result.
              </p>
            </div>
          </CardContent>
        </Card>
      )}

      <div className="grid gap-2">
        {events.data?.map((event) => (
          <Card key={event.id}>
            <CardContent className="flex items-center gap-3 py-3">
              <div className="min-w-0 flex-1">
                <p className="truncate text-sm font-medium">{event.summary}</p>
                <p className="font-mono text-xs text-muted-foreground">
                  {new Date(event.timestamp).toLocaleString()}
                  {event.account_login ? ` · @${event.account_login}` : ""}
                </p>
              </div>
              <Badge variant={event.result === "success" ? "success" : "destructive"}>
                {event.result}
              </Badge>
            </CardContent>
          </Card>
        ))}
      </div>
    </div>
  );
}
