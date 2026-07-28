import { defineConfig } from 'vitepress'

export default defineConfig({
  title: 'onQ',
  description: 'Public documentation for the local-first onQ prompt vault.',
  base: '/onQ/',
  lastUpdated: true,
  cleanUrls: true,
  head: [
    ['meta', { name: 'theme-color', content: '#101827' }],
  ],
  themeConfig: {
    siteTitle: 'onQ',
    nav: [
      {
        text: 'Guide',
        items: [
          { text: 'Install', link: '/installation' },
          { text: 'Quick start', link: '/quick-start' },
          { text: 'Feature guide', link: '/usage' },
        ],
      },
      {
        text: 'Reference',
        items: [
          { text: 'Prompt file format', link: '/file-format' },
          { text: 'Vault layout', link: '/vault-layout' },
          { text: 'Feature status', link: '/feature-status' },
          { text: 'Troubleshooting', link: '/troubleshooting' },
        ],
      },
      {
        text: 'Develop',
        items: [
          { text: 'Build and test', link: '/development' },
          { text: 'Architecture', link: '/architecture' },
          { text: 'Plugin authoring', link: '/plugins' },
        ],
      },
      { text: 'GitHub', link: 'https://github.com/visorcraft/onQ' },
    ],
    sidebar: [
      {
        text: 'Getting Started',
        items: [
          { text: 'Overview', link: '/' },
          { text: 'Installation', link: '/installation' },
          { text: 'Quick start', link: '/quick-start' },
          { text: 'Feature guide', link: '/usage' },
        ],
      },
      {
        text: 'User Guide',
        items: [
          { text: 'Palette and search', link: '/search' },
          { text: 'Library and organization', link: '/library' },
          { text: 'Editing and templates', link: '/editing' },
          { text: 'Settings', link: '/settings' },
        ],
      },
      {
        text: 'Data and Security',
        items: [
          { text: 'Vault security and recovery', link: '/security' },
          { text: 'Backups and portability', link: '/backups' },
          { text: 'Privacy and network access', link: '/privacy' },
        ],
      },
      {
        text: 'Reference',
        items: [
          { text: 'Prompt file format', link: '/file-format' },
          { text: 'Vault layout', link: '/vault-layout' },
          { text: 'Feature status', link: '/feature-status' },
          { text: 'Troubleshooting', link: '/troubleshooting' },
        ],
      },
      {
        text: 'Development',
        items: [
          { text: 'Build and test', link: '/development' },
          { text: 'Architecture', link: '/architecture' },
          { text: 'Plugin authoring', link: '/plugins' },
          {
            text: 'Contributing',
            link: 'https://github.com/visorcraft/onQ/blob/master/CONTRIBUTING.md',
          },
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
    search: {
      provider: 'local',
    },
    outline: {
      level: [2, 3],
      label: 'On this page',
    },
    editLink: {
      pattern: 'https://github.com/visorcraft/onQ/edit/master/docs-site/:path',
      text: 'Edit this page on GitHub',
    },
    docFooter: {
      prev: 'Previous page',
      next: 'Next page',
    },
    footer: {
      message: 'Released under the GPL-3.0-only License.',
      copyright: 'Copyright (c) visorcraft contributors',
    },
  },
})
