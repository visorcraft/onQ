<script module lang="ts">
  import { defineMeta } from '@storybook/addon-svelte-csf';
  import Input from './Input.svelte';

  const { Story } = defineMeta({
    title: 'Primitives/Input',
    component: Input,
    tags: ['autodocs'],
    argTypes: {
      type: {
        control: { type: 'select' },
        options: ['text', 'password', 'email', 'search', 'url'],
      },
    },
    args: {
      label: 'Prompt title',
      placeholder: 'Enter a title…',
      type: 'text',
    },
  });
</script>

<Story name="Default" args={{ label: 'Prompt title', placeholder: 'Enter a title…' }} />
<Story name="NoLabel" args={{ placeholder: 'Search prompts…' }} />
<Story name="Password" args={{ label: 'Master passphrase', type: 'password', placeholder: '••••••••' }} />
<Story name="Disabled" args={{ label: 'Locked field', disabled: true, value: 'Cannot edit' }} />
