import { Channel, invoke } from "@tauri-apps/api/core";
import type { AuditEvent, DoctorReport, GhAccount, GhLoginEvent, RepositorySummary } from "./types";

/**
 * Thin wrappers around Tauri commands.
 * All business logic lives in the Rust backend; these functions only
 * transport typed data. Tokens never cross this boundary.
 */

export function runDoctor(): Promise<DoctorReport> {
  return invoke<DoctorReport>("doctor_run");
}

export function listAccounts(): Promise<GhAccount[]> {
  return invoke<GhAccount[]>("accounts_list");
}

export function addAccount(onEvent: (event: GhLoginEvent) => void): Promise<GhAccount[]> {
  const channel = new Channel<GhLoginEvent>(onEvent);
  return invoke<GhAccount[]>("accounts_add", { onEvent: channel });
}

export function listRepositories(): Promise<RepositorySummary[]> {
  return invoke<RepositorySummary[]>("repositories_list");
}

export function addRepository(path: string): Promise<RepositorySummary> {
  return invoke<RepositorySummary>("repositories_add", { path });
}

export function assignRepository(request: AssignRepositoryRequest): Promise<AssignmentResult> {
  return invoke<AssignmentResult>("repositories_assign", { request });
}

export function linkRepository(repositoryId: string): Promise<RoutingResult> {
  return invoke<RoutingResult>("repositories_link", {
    request: { repository_id: repositoryId },
  });
}

export function testRepositoryConnection(repositoryId: string): Promise<ConnectionTestResult> {
  return invoke<ConnectionTestResult>("repositories_test", { repositoryId });
}

export function unlinkRepository(
  repositoryId: string,
  restoreIdentity: boolean,
): Promise<UnlinkResult> {
  return invoke<UnlinkResult>("repositories_unlink", {
    request: { repository_id: repositoryId, restore_identity: restoreIdentity },
  });
}

export function getRepositoryStatus(repositoryId: string): Promise<RepositoryActionStatus> {
  return invoke<RepositoryActionStatus>("repositories_status", { repositoryId });
}

export function stageRepositoryPaths(
  repositoryId: string,
  paths: string[],
): Promise<GitActionResult> {
  return invoke<GitActionResult>("repositories_stage", {
    request: { repository_id: repositoryId, paths },
  });
}

export function unstageRepositoryPaths(
  repositoryId: string,
  paths: string[],
): Promise<GitActionResult> {
  return invoke<GitActionResult>("repositories_unstage", {
    request: { repository_id: repositoryId, paths },
  });
}

export function commitRepository(repositoryId: string, message: string): Promise<GitActionResult> {
  return invoke<GitActionResult>("repositories_commit", {
    request: { repository_id: repositoryId, message },
  });
}

export function pullRepository(repositoryId: string): Promise<NetworkActionResult> {
  return invoke<NetworkActionResult>("repositories_pull", {
    request: { repository_id: repositoryId },
  });
}

export function pushRepository(repositoryId: string): Promise<NetworkActionResult> {
  return invoke<NetworkActionResult>("repositories_push", {
    request: { repository_id: repositoryId, caller: "desktop", approved: true },
  });
}

export type PushPolicy = "allow_normal_push" | "ask_before_push" | "block_ai_push";

export function setRepositoryPushPolicy(
  repositoryId: string,
  pushPolicy: PushPolicy,
): Promise<PushPolicyResult> {
  return invoke<PushPolicyResult>("repositories_set_push_policy", {
    request: { repository_id: repositoryId, push_policy: pushPolicy },
  });
}

export function listAuditEvents(): Promise<AuditEvent[]> {
  return invoke<AuditEvent[]>("audit_list");
}

export function getMcpInfo(): Promise<McpInfo> {
  return invoke<McpInfo>("mcp_info");
}

export function generateRepositoryAgents(repositoryId: string): Promise<GenerateAgentsResult> {
  return invoke<GenerateAgentsResult>("repositories_generate_agents", {
    request: { repository_id: repositoryId },
  });
}

export interface McpInfo {
  executable_path: string | null;
  available: boolean;
  config_snippet: string;
}

export interface GenerateAgentsResult {
  repository_id: string;
  path: string;
  created: boolean;
}

export interface AssignRepositoryRequest {
  repository_id: string;
  host: string;
  login: string;
  commit_name: string | null;
  commit_email: string | null;
}

export interface AssignmentResult {
  repository: RepositorySummary;
  marker_path: string;
  identity_changed: boolean;
}

export interface RoutingResult {
  repository_id: string;
  helper_path: string;
  configured: boolean;
}

export interface ConnectionTestResult {
  repository_id: string;
  remote_name: string;
  account_login: string;
  success: boolean;
}

export interface UnlinkResult {
  repository_id: string;
  restored_keys: string[];
  identity_preserved: boolean;
}

export interface ChangeEntry {
  path: string;
  index_status: string;
  worktree_status: string;
}

export interface RepositoryActionStatus {
  repository_id: string;
  branch: string | null;
  detached_head: boolean;
  changes: ChangeEntry[];
}

export interface GitActionResult {
  repository_id: string;
  action: string;
  changed_paths: number;
  commit: string | null;
}

export interface NetworkActionResult {
  repository_id: string;
  action: string;
  remote_name: string;
  branch: string;
  account_login: string;
  head_commit: string;
  ahead_before: number;
  behind_before: number;
}

export interface PushPolicyResult {
  repository_id: string;
  push_policy: PushPolicy;
}
