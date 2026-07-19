<script module lang="ts">
  import { defineMeta } from '@storybook/addon-svelte-csf';
  import EmptyState from './EmptyState.svelte';

  const { Story } = defineMeta({
    title: 'Components/EmptyState',
    component: EmptyState,
    parameters: { layout: 'fullscreen' },
    args: {
      onVaultReady: (): void => {},
    },
  });
</script>

<Story name="Welcome" />
