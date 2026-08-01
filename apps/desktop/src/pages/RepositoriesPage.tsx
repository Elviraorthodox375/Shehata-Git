import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { open } from "@tauri-apps/plugin-dialog";
import {
  AlertCircle,
  FolderGit2,
  FolderOpen,
  GitBranch,
  Globe,
  Loader2,
  RefreshCw,
} from "lucide-react";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardContent } from "@/components/ui/card";
import { addRepository, listRepositories } from "@/lib/tauri";

export function RepositoriesPage() {
  const queryClient = useQueryClient();
  const repos = useQuery({
    queryKey: ["repositories"],
    queryFn: listRepositories,
  });
  const addRepo = useMutation({
    mutationFn: addRepository,
    onSuccess: async () => {
      await queryClient.invalidateQueries({ queryKey: ["repositories"] });
    },
  });

  async function chooseRepository() {
    addRepo.reset();
    const selected = await open({
      directory: true,
      multiple: false,
      title: "Choose a Git repository",
    });
    if (selected) {
      addRepo.mutate(selected);
    }
  }

  return (
    <div className="mx-auto max-w-3xl space-y-4">
      <div className="flex flex-wrap items-center justify-between gap-3">
        <p className="max-w-xl text-sm text-muted-foreground">
          Add a local Git repository. Shehata Git reads its configuration but does not change it
          until you explicitly assign an account in the next step.
        </p>
        <div className="flex gap-2">
          <Button
            variant="outline"
            size="sm"
            onClick={() => repos.refetch()}
            disabled={repos.isFetching}
          >
            <RefreshCw className={repos.isFetching ? "animate-spin" : undefined} aria-hidden />
            Refresh
          </Button>
          <Button size="sm" onClick={chooseRepository} disabled={addRepo.isPending}>
            {addRepo.isPending ? (
              <Loader2 className="animate-spin" aria-hidden />
            ) : (
              <FolderOpen aria-hidden />
            )}
            {addRepo.isPending ? "Checking…" : "Add repository"}
          </Button>
        </div>
      </div>

      {(repos.isError || addRepo.isError) && (
        <Card className="border-destructive/40">
          <CardContent className="flex gap-3 py-4">
            <AlertCircle className="mt-0.5 h-5 w-5 shrink-0 text-destructive" aria-hidden />
            <div>
              <p className="text-sm font-medium text-destructive">Could not add that folder</p>
              <p className="mt-1 text-sm text-muted-foreground">
                {addRepo.error instanceof Error
                  ? addRepo.error.message
                  : repos.error instanceof Error
                    ? repos.error.message
                    : "Choose a folder that contains a Git repository."}
              </p>
            </div>
          </CardContent>
        </Card>
      )}

      {repos.isLoading && <p className="text-sm text-muted-foreground">Reading repositories…</p>}

      {repos.data?.length === 0 && (
        <Card>
          <CardContent className="flex flex-col items-center gap-3 py-10 text-center">
            <FolderGit2 className="h-8 w-8 text-muted-foreground/50" aria-hidden />
            <div>
              <p className="font-medium">No repositories linked yet</p>
              <p className="mt-1 max-w-sm text-sm text-muted-foreground">
                Choose a repository folder. It will be inspected safely before anything is saved.
              </p>
            </div>
            <Button onClick={chooseRepository} disabled={addRepo.isPending}>
              <FolderOpen aria-hidden />
              Choose repository
            </Button>
          </CardContent>
        </Card>
      )}

      <div className="grid gap-3">
        {repos.data?.map((repo) => (
          <Card key={repo.id}>
            <CardContent className="space-y-3 py-4">
              <div className="flex items-start gap-3">
                <FolderGit2 className="mt-0.5 h-5 w-5 shrink-0 text-primary" aria-hidden />
                <div className="min-w-0 flex-1">
                  <div className="flex flex-wrap items-center gap-2">
                    <p className="font-medium">{repo.display_name}</p>
                    {repo.remote_protocol === "ssh" && <Badge variant="warning">SSH</Badge>}
                    {repo.remote_protocol === "https" && <Badge variant="secondary">HTTPS</Badge>}
                  </div>
                  <p className="mt-1 truncate font-mono text-xs text-muted-foreground">
                    {repo.canonical_path}
                  </p>
                </div>
                {repo.assigned_login ? (
                  <Badge variant="success">@{repo.assigned_login}</Badge>
                ) : (
                  <Badge variant="warning">account needed</Badge>
                )}
              </div>

              <div className="grid gap-2 border-t border-border pt-3 text-xs text-muted-foreground sm:grid-cols-2">
                <div className="flex min-w-0 items-center gap-2">
                  <GitBranch className="h-3.5 w-3.5 shrink-0" aria-hidden />
                  <span className="truncate">{repo.current_branch ?? "No commits yet"}</span>
                </div>
                <div className="flex min-w-0 items-center gap-2">
                  <Globe className="h-3.5 w-3.5 shrink-0" aria-hidden />
                  <span className="truncate">
                    {repo.host && repo.owner && repo.repo_name
                      ? `${repo.host}/${repo.owner}/${repo.repo_name}`
                      : "No supported GitHub remote detected"}
                  </span>
                </div>
              </div>
            </CardContent>
          </Card>
        ))}
      </div>
    </div>
  );
}
