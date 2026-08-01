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

export function listAuditEvents(): Promise<AuditEvent[]> {
  return invoke<AuditEvent[]>("audit_list");
}

export function getMcpInfo(): Promise<McpInfo> {
  return invoke<McpInfo>("mcp_info");
}

export interface McpInfo {
  executable_path: string | null;
  available: boolean;
  config_snippet: string;
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
