<script module lang="ts">
  import { defineMeta } from '@storybook/addon-svelte-csf';
  import GlassCard from './GlassCard.svelte';

  const { Story } = defineMeta({
    title: 'Primitives/GlassCard',
    component: GlassCard,
    tags: ['autodocs'],
    args: {
      title: 'Welcome',
      elevated: false,
    },
  });
</script>

<Story name="Default" args={{ title: 'Welcome to onQ' }}>
  {#snippet children()}
    <p>This is the standard glass card with body content.</p>
  {/snippet}
</Story>

<Story name="Elevated" args={{ title: 'Featured prompt', elevated: true }}>
  {#snippet children()}
    <p>Elevated glass card with stronger blur for primary content.</p>
  {/snippet}
</Story>

<!--
  Stories that need both `children` and a second snippet (e.g. `footer`)
  must use a `template` snippet — addon-svelte-csf's <Story> only
  recognises the two well-known snippet names `template` and `children`,
  so any other named snippet must be composed inside the template body.
-->
<Story name="WithFooter" args={{ title: 'Are you sure?' }}>
  {#snippet template()}
    <GlassCard title="Are you sure?">
      {#snippet children()}
        <p>This action will lock the prompt and encrypt the body.</p>
      {/snippet}
      {#snippet footer()}
        <button type="button" style="background: transparent; border: 0; color: var(--glass-text-dim); cursor: pointer;">Cancel</button>
        <button type="button" style="background: var(--glass-accent); border: 0; color: #fff; padding: 6px 12px; border-radius: 8px; cursor: pointer;">Lock</button>
      {/snippet}
    </GlassCard>
  {/snippet}
</Story>

<Story name="NoTitle" args={{ title: '' }}>
  {#snippet children()}
    <p>A plain glass card without a title heading.</p>
  {/snippet}
</Story>
