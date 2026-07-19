export function paletteShortcut(platform = navigator.platform): '⌘K' | 'Ctrl+K' {
  return platform.startsWith('Mac') ? '⌘K' : 'Ctrl+K';
}
