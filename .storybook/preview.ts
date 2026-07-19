import type { Preview } from '@storybook/svelte-vite';

import '../src/lib/styles/tokens.css';
import '../src/lib/styles/tokens.light.css';
import '../src/lib/styles/glass.css';
import '../src/lib/styles/motion.css';
import { installTauriMock } from './tauri-mock';

installTauriMock();

const preview: Preview = {
  parameters: {
    controls: {
      matchers: {
        color: /(background|color)$/i,
        date: /Date$/i,
      },
    },
    backgrounds: {
      default: 'glass',
      values: [
        { name: 'glass', value: '#1a2138' },
        { name: 'light', value: '#f0f4ff' },
      ],
    },
    layout: 'centered',
  },
};

export default preview;
