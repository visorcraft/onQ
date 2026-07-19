<script module lang="ts">
  import { defineMeta } from '@storybook/addon-svelte-csf';
  import ConflictResolver from './ConflictResolver.svelte';

  const conflictText = [
    'intro paragraph',
    '<<<<<<< ours',
    'our first edit',
    '=======',
    'their first edit',
    '>>>>>>> theirs',
    'shared paragraph',
    '<<<<<<< ours',
    'our second edit',
    '=======',
    'their second edit',
    '>>>>>>> theirs',
    'closing paragraph',
  ].join('\n');

  const base = [
    'intro paragraph',
    'shared first edit',
    'shared paragraph',
    'shared second edit',
    'closing paragraph',
  ].join('\n');

  const { Story } = defineMeta({
    title: 'Components/ConflictResolver',
    component: ConflictResolver,
    parameters: { layout: 'fullscreen' },
    args: {
      conflictText,
      base,
      onResolved: (): void => {},
    },
  });
</script>

<Story name="TwoHunks" args={{ conflictText, base }} />
<Story name="NoConflicts" args={{ conflictText: 'just clean text', base: 'just clean text' }} />
