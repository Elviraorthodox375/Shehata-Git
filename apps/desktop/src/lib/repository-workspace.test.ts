import { describe, expect, it } from "vitest";
import {
  decideSmartSync,
  displayRepositoryPath,
  filterWorkspaceChanges,
  isStagedChange,
} from "./repository-workspace";

const changes = [
  { path: "src/app.tsx", index_status: " ", worktree_status: "M" },
  { path: "src/staged.ts", index_status: "M", worktree_status: " " },
  { path: "docs/new.md", index_status: "?", worktree_status: "?" },
  { path: "src/both.ts", index_status: "M", worktree_status: "M" },
];

describe("repository workspace helpers", () => {
  it("chooses only bounded Smart Sync outcomes", () => {
    expect(decideSmartSync(0, 0)).toBe("in_sync");
    expect(decideSmartSync(0, 2)).toBe("pull");
    expect(decideSmartSync(3, 0)).toBe("push");
    expect(decideSmartSync(1, 1)).toBe("diverged");
  });

  it("shows a human Windows path without the verbatim prefix", () => {
    expect(displayRepositoryPath("\\\\?\\D:\\Work\\Repo")).toBe("D:\\Work\\Repo");
    expect(displayRepositoryPath("D:\\Work\\Repo")).toBe("D:\\Work\\Repo");
  });

  it("filters changed files by state", () => {
    expect(filterWorkspaceChanges(changes, "", "staged").map((change) => change.path)).toEqual([
      "src/staged.ts",
      "src/both.ts",
    ]);
    expect(filterWorkspaceChanges(changes, "", "changed").map((change) => change.path)).toEqual([
      "src/app.tsx",
      "src/both.ts",
    ]);
    expect(filterWorkspaceChanges(changes, "", "untracked")).toHaveLength(1);
  });

  it("combines search with the selected filter", () => {
    expect(filterWorkspaceChanges(changes, "both", "staged")).toEqual([changes[3]]);
    expect(isStagedChange(changes[1])).toBe(true);
    expect(isStagedChange(changes[0])).toBe(false);
  });
});
