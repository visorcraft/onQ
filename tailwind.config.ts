import type { Config } from 'tailwindcss';

export default {
  content: ['./src/**/*.{svelte,ts}'],
  theme: {
    extend: {
      colors: {
        // Cool Indigo palette
        'glass-base': '#1a2138',
        'glass-surface': '#2a3454',
        'glass-elevated': '#3a4a6e',
        'glass-periwinkle': '#b8d2ff',
        'glass-accent': '#5b8fff',
        'glass-text': '#e0e8ff',
        'glass-text-dim': '#a8b8d8',
        'glass-text-faint': '#7a8aa8',
        'glass-border': 'rgba(255,255,255,0.12)',
        'glass-border-strong': 'rgba(255,255,255,0.18)',
      },
      borderRadius: { glass: '14px', 'glass-lg': '20px' },
      backdropBlur: { glass: '20px', 'glass-lg': '30px' },
    },
  },
} satisfies Config;
