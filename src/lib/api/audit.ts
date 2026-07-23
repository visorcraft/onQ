import { invoke } from '@tauri-apps/api/core';

export type AuditEvent = {
  at: string;
  kind: string;
  detail?: string | null;
};

export async function readAuditLog(limit = 50): Promise<AuditEvent[]> {
  return invoke<AuditEvent[]>('read_audit_log', { limit });
}
