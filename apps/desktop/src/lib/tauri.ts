import { Channel, invoke } from "@tauri-apps/api/core";
import type {
  AccountScopeRepair,
  AuditEvent,
  DoctorReport,
  GhAccount,
  GhLoginEvent,
  RepositorySummary,
} from "./types";

/**
 * Thin wrappers around Tauri commands.
 * All business logic lives in the Rust backend; these functions only
 * transport typed data. Tokens never cross this boundary.
 */

export function runDoctor(): Promise<DoctorReport> {
  return invoke<DoctorReport>("doctor_run");
}

export type PrerequisiteId = "git" | "github_cli";

export interface InstalledPrerequisite {
  id: PrerequisiteId;
  label: string;
}

export interface InstallPrerequisitesResult {
  installed: InstalledPrerequisite[];
}

export function installPrerequisites(ids: PrerequisiteId[]): Promise<InstallPrerequisitesResult> {
  return invoke<InstallPrerequisitesResult>("prerequisites_install", { request: { ids } });
}

export function packageManagerAvailable(): Promise<boolean> {
  return invoke<boolean>("prerequisites_available");
}

export function listAccounts(): Promise<GhAccount[]> {
  return invoke<GhAccount[]>("accounts_list");
}

export function addAccount(onEvent: (event: GhLoginEvent) => void): Promise<GhAccount[]> {
  const channel = new Channel<GhLoginEvent>(onEvent);
  return invoke<GhAccount[]>("accounts_add", { onEvent: channel });
}

export function cancelAccountLogin(): Promise<boolean> {
  return invoke<boolean>("accounts_cancel_login");
}

export function removeAccount(account: Pick<GhAccount, "host" | "login">): Promise<GhAccount[]> {
  return invoke<GhAccount[]>("accounts_remove", { request: account });
}

export function switchAccount(account: Pick<GhAccount, "host" | "login">): Promise<GhAccount[]> {
  return invoke<GhAccount[]>("accounts_switch", { request: account });
}

export function grantAccountScope(
  request: AccountScopeRepair,
  onEvent: (event: GhLoginEvent) => void,
): Promise<GhAccount[]> {
  const channel = new Channel<GhLoginEvent>(onEvent);
  return invoke<GhAccount[]>("accounts_grant_scope", { request, onEvent: channel });
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

export function getRepositoryFileDiff(
  repositoryId: string,
  path: string,
  staged: boolean,
): Promise<FileDiff> {
  return invoke<FileDiff>("repositories_file_diff", {
    request: { repository_id: repositoryId, path, staged },
  });
}

export function previewRepositorySync(repositoryId: string): Promise<SyncPreview> {
  return invoke<SyncPreview>("repositories_sync_preview", { repositoryId });
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

export function deleteAuditEvent(id: number): Promise<boolean> {
  return invoke<boolean>("audit_delete", { id });
}

export function clearAuditEvents(): Promise<number> {
  return invoke<number>("audit_clear");
}

export function getMcpInfo(): Promise<McpInfo> {
  return invoke<McpInfo>("mcp_info");
}

export function getDiagnosticReport(): Promise<SafeDiagnosticReport> {
  return invoke<SafeDiagnosticReport>("diagnostics_report");
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
  detected_clients: AiClientInfo[];
}

export interface AiClientInfo {
  id: string;
  name: string;
  available: boolean;
  executable_path: string | null;
}

export interface DiagnosticCheck {
  id: string;
  status: string;
  version: string | null;
}

export interface DiagnosticAiClient {
  id: string;
  name: string;
  available: boolean;
}

export interface SafeDiagnosticReport {
  generated_at: string;
  app_version: string;
  os: string;
  healthy: boolean;
  checks: DiagnosticCheck[];
  repository_count: number;
  assigned_repository_count: number;
  routed_repository_count: number;
  ai_clients: DiagnosticAiClient[];
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

export interface FileDiff {
  repository_id: string;
  path: string;
  staged: boolean;
  content: string;
  truncated: boolean;
  sensitive: boolean;
  blocked_reason: string | null;
}

export interface SyncPreview {
  repository_id: string;
  remote_name: string;
  branch: string;
  account_login: string;
  ahead: number;
  behind: number;
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
