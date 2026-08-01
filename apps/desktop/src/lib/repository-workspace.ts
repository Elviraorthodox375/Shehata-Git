import type { ChangeEntry } from "./tauri";

export type WorkspaceFileFilter = "all" | "changed" | "staged" | "untracked";
export type SmartSyncDecision = "in_sync" | "pull" | "push" | "diverged";

export function decideSmartSync(ahead: number, behind: number): SmartSyncDecision {
  if (ahead === 0 && behind === 0) return "in_sync";
  if (ahead > 0 && behind > 0) return "diverged";
  if (behind > 0) return "pull";
  return "push";
}

export function isStagedChange(change: Pick<ChangeEntry, "index_status">): boolean {
  return change.index_status !== " " && change.index_status !== "?";
}

export function isUntrackedChange(
  change: Pick<ChangeEntry, "index_status" | "worktree_status">,
): boolean {
  return change.index_status === "?" && change.worktree_status === "?";
}

export function filterWorkspaceChanges(
  changes: ChangeEntry[],
  search: string,
  filter: WorkspaceFileFilter,
): ChangeEntry[] {
  const needle = search.trim().toLocaleLowerCase();
  return changes.filter((change) => {
    if (needle && !change.path.toLocaleLowerCase().includes(needle)) return false;

    if (filter === "staged") return isStagedChange(change);
    if (filter === "untracked") return isUntrackedChange(change);
    if (filter === "changed") {
      return change.worktree_status !== " " && !isUntrackedChange(change);
    }
    return true;
  });
}

export function displayRepositoryPath(path: string): string {
  if (path.startsWith("\\\\?\\")) return path.slice(4);
  if (path.startsWith("//?/")) return path.slice(4);
  return path;
}
