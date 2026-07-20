<script module lang="ts">
  import { defineMeta } from '@storybook/addon-svelte-csf';
  import TutorialOverlay from './TutorialOverlay.svelte';

  const { Story } = defineMeta({
    title: 'Primitives/TutorialOverlay',
    component: TutorialOverlay,
    tags: ['autodocs'],
    args: {
      step: 1,
      totalSteps: 4,
      title: 'Welcome to onQ',
      body: 'A local-first prompt vault. Press ⌘+Q / Meta+Q / Win+Q anywhere to search your prompts.',
    },
  });
</script>

<Story
  name="FirstStep"
  args={{
    step: 1,
    totalSteps: 4,
    title: 'Welcome to onQ',
    body: 'A local-first prompt vault. Press ⌘+Q / Meta+Q / Win+Q anywhere to search your prompts.',
  }}
/>
<Story
  name="MiddleStep"
  args={{
    step: 2,
    totalSteps: 4,
    title: 'Search is the app',
    body: 'Type a few characters — results fuse keyword and semantic matches in real time.',
  }}
/>
<Story
  name="FinalStep"
  args={{
    step: 4,
    totalSteps: 4,
    title: 'You are ready',
    body: 'Your vault is encrypted at rest. Have fun.',
  }}
/>
