import { useQuery } from "@tanstack/react-query";
import { FolderGit2, FolderOpen, RefreshCw } from "lucide-react";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardContent } from "@/components/ui/card";
import { listRepositories } from "@/lib/tauri";

/**
 * Repositories page.
 * Lists repositories registered in Shehata Git's local database.
 * Repository linking + account assignment arrives in Phases 4–5.
 */
export function RepositoriesPage() {
  const repos = useQuery({
    queryKey: ["repositories"],
    queryFn: listRepositories,
  });

  return (
    <div className="mx-auto max-w-2xl space-y-4">
      <div className="flex items-center justify-between">
        <p className="text-sm text-muted-foreground">
          Repositories registered in Shehata Git, each with its assigned account.
        </p>
        <Button
          variant="outline"
          size="sm"
          onClick={() => repos.refetch()}
          disabled={repos.isFetching}
        >
          <RefreshCw className={repos.isFetching ? "animate-spin" : undefined} aria-hidden />
          Refresh
        </Button>
      </div>

      {repos.isLoading && <p className="text-sm text-muted-foreground">Reading repositories…</p>}

      {repos.isError && (
        <Card className="border-destructive/40">
          <CardContent className="py-4">
            <p className="text-sm text-destructive">
              {repos.error instanceof Error ? repos.error.message : "Could not read repositories."}
            </p>
          </CardContent>
        </Card>
      )}

      {repos.data?.length === 0 && (
        <Card>
          <CardContent className="flex flex-col items-center gap-3 py-10 text-center">
            <FolderGit2 className="h-8 w-8 text-muted-foreground/50" aria-hidden />
            <div>
              <p className="font-medium">No repositories linked yet</p>
              <p className="mt-1 max-w-sm text-sm text-muted-foreground">
                When you link a repository, you pick exactly one GitHub account for it. From then
                on, every push uses that account — no matter which tool pushes.
              </p>
            </div>
            <Button disabled title="Arrives in the repositories milestone (Phase 4)">
              <FolderOpen aria-hidden />
              Add repository (soon)
            </Button>
          </CardContent>
        </Card>
      )}

      <div className="grid gap-3">
        {repos.data?.map((repo) => (
          <Card key={repo.id}>
            <CardContent className="flex items-center gap-3 py-4">
              <FolderGit2 className="h-5 w-5 shrink-0 text-muted-foreground" aria-hidden />
              <div className="min-w-0 flex-1">
                <p className="truncate font-medium">{repo.display_name}</p>
                <p className="truncate font-mono text-xs text-muted-foreground">
                  {repo.canonical_path}
                </p>
              </div>
              {repo.assigned_login ? (
                <Badge variant="success">@{repo.assigned_login}</Badge>
              ) : (
                <Badge variant="warning">no account assigned</Badge>
              )}
            </CardContent>
          </Card>
        ))}
      </div>
    </div>
  );
}
