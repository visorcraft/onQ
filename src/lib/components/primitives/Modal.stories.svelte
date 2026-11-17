<script module lang="ts">
  import { defineMeta } from '@storybook/addon-svelte-csf';
  import Modal from './Modal.svelte';

  const { Story } = defineMeta({
    title: 'Primitives/Modal',
    component: Modal,
    tags: ['autodocs'],
    args: {
      open: true,
      title: 'Confirm deletion',
    },
  });
</script>

<Story name="Default" args={{ open: true, title: 'Confirm deletion' }}>
  <p>Are you sure you want to delete this prompt? This cannot be undone.</p>
</Story>

<Story name="NoTitle" args={{ open: true, title: '' }}>
  <p>Modal without a title — just body content.</p>
</Story>

<Story name="Closed" args={{ open: false, title: 'Hidden modal' }} />
