<script module lang="ts">
  import { defineMeta } from '@storybook/addon-svelte-csf';
  import Button from './Button.svelte';

  const { Story } = defineMeta({
    title: 'Primitives/Button',
    component: Button,
    tags: ['autodocs'],
    argTypes: {
      variant: {
        control: { type: 'select' },
        options: ['primary', 'secondary', 'danger', 'quiet'],
      },
      size: {
        control: { type: 'select' },
        options: ['small', 'medium', 'large'],
      },
    },
    args: {
      label: 'Button',
      variant: 'secondary',
      size: 'medium',
      disabled: false,
    },
  });
</script>

<Story name="Primary" args={{ label: 'Save prompt', variant: 'primary' }} />
<Story name="Secondary" args={{ label: 'Cancel' }} />
<Story name="Danger" args={{ label: 'Delete', variant: 'danger' }} />
<Story name="Quiet" args={{ label: 'Skip', variant: 'quiet' }} />
<Story name="Small" args={{ label: 'Compact', size: 'small' }} />
<Story name="Large" args={{ label: 'Huge call to action', size: 'large' }} />
<Story name="Disabled" args={{ label: 'Unavailable', disabled: true }} />
