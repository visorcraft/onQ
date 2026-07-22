import { invoke } from '@tauri-apps/api/core';

/**
 * Hide the main window to the system tray. Wired to the Rust
 * `hide_to_tray` command which routes through the same KDE-aware
 * `hide_main_window` helper the close-button handler uses.
 *
 * Fire-and-forget — the promise resolves once the window-hide call
 * returns. The Rust command never rejects on user-visible failure (it
 * already swallows `set_skip_taskbar` / `hide` errors), so callers can
 * `void` it without a try/catch.
 */
export function hideToTray(): Promise<void> {
  return invoke<void>('hide_to_tray');
}