<script module lang="ts">
  import { defineMeta } from '@storybook/addon-svelte-csf';
  import Editor from './Editor.svelte';

  const { Story } = defineMeta({
    title: 'Components/Editor',
    component: Editor,
    tags: ['autodocs'],
    args: {
      id: 'prompt-1',
    },
    parameters: {
      // The editor dialog uses `position: fixed; inset: 0` so it needs a
      // fullscreen canvas to render correctly inside Storybook.
      layout: 'fullscreen',
    },
  });
</script>

<Story name="ExistingPrompt" args={{ id: 'prompt-1' }} />
<Story name="LockedPrompt" args={{ id: 'prompt-2' }} />
