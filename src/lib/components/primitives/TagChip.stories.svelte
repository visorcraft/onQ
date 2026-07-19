<script module lang="ts">
  import { defineMeta } from '@storybook/addon-svelte-csf';
  import TagChip from './TagChip.svelte';

  const { Story } = defineMeta({
    title: 'Primitives/TagChip',
    component: TagChip,
    tags: ['autodocs'],
    args: {
      label: 'writing',
      active: false,
      removable: false,
    },
  });
</script>

<Story name="Default" args={{ label: 'writing' }} />
<Story name="Active" args={{ label: 'writing', active: true }} />
<Story name="Removable" args={{ label: 'drafts', removable: true }} />
<Story name="RemovableActive" args={{ label: 'favorites', active: true, removable: true }} />
<Story name="Multiple">
  {#snippet children()}
    <div style="display: flex; gap: 6px; flex-wrap: wrap;">
      <TagChip label="writing" />
      <TagChip label="code-review" active />
      <TagChip label="meetings" />
      <TagChip label="research" removable />
      <TagChip label="brainstorm" removable />
    </div>
  {/snippet}
</Story>
