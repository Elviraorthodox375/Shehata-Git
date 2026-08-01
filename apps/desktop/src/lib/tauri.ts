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
