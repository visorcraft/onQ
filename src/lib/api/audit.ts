import { invoke } from '@tauri-apps/api/core';

export type AuditEvent = {
  at: string;
  kind: string;
  detail?: string | null;
};

export async function readAuditLog(limit = 50): Promise<AuditEvent[]> {
  return invoke<AuditEvent[]>('read_audit_log', { limit });
}

export async function getAuditEnabled(): Promise<boolean> {
  return invoke<boolean>('get_audit_enabled');
}

export async function setAuditEnabled(enabled: boolean): Promise<void> {
  return invoke('set_audit_enabled', { enabled });
}
