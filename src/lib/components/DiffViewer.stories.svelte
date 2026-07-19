<script module lang="ts">
  import { defineMeta } from '@storybook/addon-svelte-csf';
  import DiffViewer from './DiffViewer.svelte';

  // Mock three-way diff: a shared base, an "ours" edit, a "theirs" edit,
  // and one conflicting hunk on the middle line. Mirrors the typical sync
  // conflict the resolver surfaces in production.
  const base = [
    '# Project brief',
    '',
    'Build a Svelte 5 editor with glass styling.',
    'Tests must cover the prompt lifecycle end-to-end.',
    '',
  ].join('\n');

  const ours = [
    '# Project brief',
    '',
    'Build a Svelte 5 editor with glass styling and runes.',
    'Tests must cover the prompt lifecycle end-to-end.',
    '',
  ].join('\n');

  const theirs = [
    '# Project brief',
    '',
    'Build a Svelte 5 editor with glass styling.',
    'Tests must cover the prompt lifecycle end-to-end, including lock flows.',
    '',
  ].join('\n');

  const { Story } = defineMeta({
    title: 'Components/DiffViewer',
    component: DiffViewer,
    tags: ['autodocs'],
    parameters: {
      layout: 'fullscreen',
    },
    args: {
      base,
      ours,
      theirs,
      embedded: false,
    },
  });
</script>

<Story name="Standalone" args={{ base, ours, theirs }} />
<Story name="Embedded" args={{ base, ours, theirs, embedded: true }} />
<Story
  name="AllIdentical"
  args={{
    base: 'unchanged\n',
    ours: 'unchanged\n',
    theirs: 'unchanged\n',
  }}
/>
<Story
  name="HeavyConflict"
  args={{
    base: 'line one\nline two\nline three\nline four\n',
    ours: 'line one (ours)\nline two\nline three (ours)\nline four\n',
    theirs: 'line one\nline two (theirs)\nline three\nline four (theirs)\n',
  }}
/>
