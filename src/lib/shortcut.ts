/** Coarse platform for shortcut labels and defaults. */
export type ShortcutPlatform = 'mac' | 'windows' | 'linux';

/**
 * Detect which OS-style modifier labels and defaults to use.
 * Accepts optional overrides so unit tests stay deterministic.
 */
export function detectShortcutPlatform(
  platform = typeof navigator !== 'undefined' ? navigator.platform : '',
  userAgent = typeof navigator !== 'undefined' ? navigator.userAgent : '',
): ShortcutPlatform {
  if (/Mac|iPhone|iPad|iPod/i.test(platform) || /Mac OS X/i.test(userAgent)) {
    return 'mac';
  }
  if (/Win/i.test(platform) || /Windows/i.test(userAgent)) {
    return 'windows';
  }
  return 'linux';
}

/** OS meta key as shown in the UI (Win / Meta / ⌘). */
export function metaModifierLabel(
  platform: ShortcutPlatform = detectShortcutPlatform(),
): 'Win' | 'Meta' | '⌘' {
  switch (platform) {
    case 'mac':
      return '⌘';
    case 'windows':
      return 'Win';
    default:
      return 'Meta';
  }
}

/**
 * Default global shortcut for a fresh install.
 * Windows: Win+Q · Linux: Meta+Q · macOS: ⌘+Q
 */
export function defaultGlobalShortcut(
  platform: ShortcutPlatform = detectShortcutPlatform(),
): string {
  return `${metaModifierLabel(platform)}+Q`;
}

/** Display string for the default palette / tray shortcut (same as default). */
export function paletteShortcut(
  platformHint = typeof navigator !== 'undefined' ? navigator.platform : '',
): string {
  return defaultGlobalShortcut(detectShortcutPlatform(platformHint));
}

/** Tokens that all mean the OS meta / Windows / Command key. */
const META_ALIASES = new Set(['Super', 'Meta', 'Win', 'Command', 'Cmd', '⌘']);

/**
 * Normalize a stored or captured shortcut to this platform's meta label.
 * Migrates older installs that used Super (or other aliases) for the OS key.
 */
export function normalizeShortcut(
  shortcut: string,
  platform: ShortcutPlatform = detectShortcutPlatform(),
): string {
  const meta = metaModifierLabel(platform);
  return shortcut
    .split('+')
    .map((part) => (META_ALIASES.has(part) ? meta : part))
    .join('+');
}

/**
 * Convert a UI shortcut into a string global-hotkey / tauri accepts.
 * Tauri only parses Super/Command/Cmd for the OS meta key — not Win or Meta.
 */
export function toNativeShortcut(shortcut: string): string {
  return shortcut
    .split('+')
    .map((part) => (META_ALIASES.has(part) ? 'Super' : part))
    .join('+');
}
