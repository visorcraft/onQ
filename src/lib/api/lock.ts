import { invoke } from '@tauri-apps/api/core';
import type { PromptSummary } from '../types/prompt';

/// Lock the prompt identified by `id`: encrypts its body with a fresh
/// per-prompt AES-256-GCM DEK, writes the envelope to
/// `<vault>/.onq/locked/<id>.enc`, stores the DEK in the OS keychain,
/// and marks the row as locked. The body in the .md vault file is cleared
/// so the plaintext is no longer at rest.
export async function lockPrompt(id: string): Promise<PromptSummary> {
  return invoke<PromptSummary>('lock_prompt', { id });
}

/// Reverse of [`lockPrompt`]: fetches the DEK from the OS keychain, decrypts
/// the .enc envelope, writes the plaintext back to the vault .md, deletes
/// the .enc and keychain entries, and marks the row as unlocked.
export async function unlockPrompt(id: string): Promise<PromptSummary> {
  return invoke<PromptSummary>('unlock_prompt', { id });
}