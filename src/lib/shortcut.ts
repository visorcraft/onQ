export function paletteShortcut(platform = navigator.platform): '⌘Q' | 'Ctrl+Q' {
  return platform.startsWith('Mac') ? '⌘Q' : 'Ctrl+Q';
}
