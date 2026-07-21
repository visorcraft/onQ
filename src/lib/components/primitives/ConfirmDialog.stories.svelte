<script module lang="ts">
  import { defineMeta } from '@storybook/addon-svelte-csf';
  import ConfirmDialog from './ConfirmDialog.svelte';

  const { Story } = defineMeta({
    title: 'Primitives/ConfirmDialog',
    component: ConfirmDialog,
    tags: ['autodocs'],
    parameters: {
      layout: 'fullscreen',
    },
  });
</script>

<Story name="DeletePrompt" args={{ open: true, title: 'Delete this prompt?', description: 'The prompt will be permanently removed from your vault.', itemLabel: 'Weekly status update', itemKind: 'Prompt', confirmLabel: 'Delete prompt' }} />

<Story name="DeleteProject" args={{ open: true, title: 'Delete this project?', description: 'Sub-projects will be removed too. Prompts inside move to Unfiled.', itemLabel: 'Writing/Blog Posts', itemKind: 'Project', confirmLabel: 'Delete project' }} />

<Story name="DeleteSmartFolder" args={{ open: true, title: 'Delete this smart folder?', description: 'Only the saved search is removed — matching prompts stay in your vault.', itemLabel: 'Favorites only', itemKind: 'Smart folder', confirmLabel: 'Delete smart folder' }} />

<Story name="Busy" args={{ open: true, title: 'Delete this prompt?', description: 'The prompt will be permanently removed from your vault.', itemLabel: 'Draft', itemKind: 'Prompt', busy: true }} />
