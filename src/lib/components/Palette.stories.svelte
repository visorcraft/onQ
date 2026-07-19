<script module lang="ts">
  import { defineMeta } from '@storybook/addon-svelte-csf';
  import Palette from './Palette.svelte';

  // Svelte action: dispatch a synthetic ⌘K once the host mounts so the
  // Palette flips from closed to open and renders its canvas.
  // eslint-disable-next-line no-unused-vars
  function openPaletteOnMount(_node: HTMLElement) {
    queueMicrotask(() => {
      window.dispatchEvent(new KeyboardEvent('keydown', { key: 'k', metaKey: true }));
    });
    return {};
  }

  const { Story } = defineMeta({
    title: 'Components/Palette',
    component: Palette,
    tags: ['autodocs'],
    parameters: {
      layout: 'fullscreen',
    },
  });
</script>

<!--
  Palette is a singleton that listens for ⌘K on `window`. The Closed story
  renders the default state; the Opened story dispatches a synthetic ⌘K on
  mount so the canvas appears with mock data.
-->
<Story name="Closed">
  {#snippet children()}
    <Palette />
    <p style="color: var(--glass-text-dim); font-family: 'JetBrains Mono', monospace; padding: 24px;">
      Palette is closed by default. Press <kbd>⌘K</kbd> to open.
    </p>
  {/snippet}
</Story>

<Story name="Opened">
  {#snippet children()}
    <div use:openPaletteOnMount>
      <Palette />
    </div>
    <p style="color: var(--glass-text-dim); padding: 24px;">
      Palette opened via simulated ⌘K. Mock data hydrated via the Tauri mock.
    </p>
  {/snippet}
</Story>
