import './lib/styles/tokens.css';
import './lib/styles/tokens.light.css';
import './lib/styles/glass.css';
import './lib/styles/motion.css';
import { mount } from 'svelte';
import App from './App.svelte';
import { loadGlobalShortcut } from './lib/stores/globalShortcut';
import { loadTheme } from './lib/stores/theme';

const app = mount(App, { target: document.getElementById('app')! });

// Apply the persisted theme as soon as the app boots. The promise is fire-
// and-forget because the default `dark` class is already active from
// `tokens.css`; this just brings the DOM in line with what the user last
// picked once the backend is reachable.
void loadTheme();
void loadGlobalShortcut().catch((error) => {
  console.error('Unable to register global shortcut:', error);
});

export default app;
