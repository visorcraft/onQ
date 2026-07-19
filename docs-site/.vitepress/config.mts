import { defineConfig } from 'vitepress'

export default defineConfig({
  title: 'onQ',
  description: 'Search-first prompt vault. Local-first. Hybrid Markdown + MongrelDB.',
  base: '/onQ/',
  lastUpdated: true,
  cleanUrls: true,
  themeConfig: {
    siteTitle: 'onQ',
    nav: [
      { text: 'Guide', link: '/installation' },
      { text: 'Usage', link: '/usage' },
      { text: 'GitHub', link: 'https://github.com/visorcraft/onQ' },
    ],
    sidebar: [
      {
        text: 'Getting Started',
        items: [
          { text: 'Overview', link: '/' },
          { text: 'Installation', link: '/installation' },
          { text: 'Usage', link: '/usage' },
        ],
      },
      {
        text: 'Project',
        items: [
          { text: 'GitHub', link: 'https://github.com/visorcraft/onQ' },
          { text: 'Releases', link: 'https://github.com/visorcraft/onQ/releases' },
        ],
      },
    ],
    socialLinks: [
      { icon: 'github', link: 'https://github.com/visorcraft/onQ' },
    ],
    footer: {
      message: 'Released under the GPL-3.0-only License.',
      copyright: 'Copyright (c) visorcraft contributors',
    },
  },
})