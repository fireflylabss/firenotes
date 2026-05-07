<script lang="ts">
  import { createEventDispatcher, tick } from 'svelte';

  type PaletteCommand = {
    id: string;
    label: string;
    desc: string;
    run: () => void;
  };

  export let show = false;
  export let commands: PaletteCommand[] = [];

  const dispatch = createEventDispatcher<{ close: void; run: PaletteCommand }>();

  let filter = '';
  let inputEl: HTMLInputElement | null = null;

  $: filtered = commands.filter((cmd) =>
    `${cmd.label} ${cmd.desc}`.toLowerCase().includes(filter.toLowerCase())
  );

  $: if (show) {
    filter = '';
    tick().then(() => inputEl?.focus());
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') {
      e.preventDefault();
      dispatch('close');
      return;
    }
    if (e.key === 'Enter') {
      e.preventDefault();
      const first = filtered[0];
      if (first) {
        dispatch('run', first);
      }
    }
  }
</script>

{#if show}
  <div class="palette-backdrop" role="presentation" on:click={() => dispatch('close')}></div>
  <div class="palette" role="dialog" aria-modal="true" aria-label="Command palette">
    <div class="palette-input-wrap">
      <input
        bind:this={inputEl}
        class="palette-input"
        type="text"
        placeholder="Type a command..."
        bind:value={filter}
        on:keydown={handleKeydown}
      />
    </div>
    <div class="palette-list">
      {#if filtered.length === 0}
        <div class="palette-empty">No matches</div>
      {:else}
        {#each filtered as cmd}
          <button class="palette-item" on:click={() => dispatch('run', cmd)}>
            <div class="palette-title">{cmd.label}</div>
            <div class="palette-desc">{cmd.desc}</div>
          </button>
        {/each}
      {/if}
    </div>
  </div>
{/if}

<style>
  .palette-backdrop {
    position: fixed;
    inset: 0;
    background: rgba(15, 23, 42, 0.35);
    backdrop-filter: blur(4px);
    z-index: 50;
  }

  .palette {
    position: fixed;
    top: 10%;
    left: 50%;
    transform: translateX(-50%);
    width: min(700px, 94vw);
    border: 1px solid var(--border-subtle);
    border-radius: 8px;
    background: var(--bg-surface);
    box-shadow: var(--shadow-lg);
    overflow: hidden;
    z-index: 51;
  }

  .palette-input-wrap {
    padding: 14px;
    border-bottom: 1px solid var(--border-subtle);
    background: color-mix(in srgb, var(--bg-sidebar) 65%, var(--bg-surface));
  }

  .palette-input {
    width: 100%;
    border: 1px solid var(--border-subtle);
    border-radius: 6px;
    background: var(--bg-surface);
    color: var(--text-primary);
    padding: 0.75rem 0.85rem;
    font-size: 0.92rem;
    outline: none;
  }

  .palette-input:focus {
    border-color: var(--accent-primary);
    box-shadow: 0 0 0 3px color-mix(in srgb, var(--accent-primary) 14%, transparent);
  }

  .palette-list {
    max-height: 380px;
    overflow: auto;
    padding: 6px;
  }

  .palette-item {
    width: 100%;
    border: 1px solid transparent;
    border-radius: 6px;
    background: transparent;
    color: var(--text-primary);
    text-align: left;
    padding: 10px 12px;
    cursor: pointer;
    display: flex;
    flex-direction: column;
    gap: 3px;
    transition: all var(--transition-fast);
  }

  .palette-item:hover {
    border-color: var(--border-subtle);
    background: var(--bg-hover);
  }

  .palette-title {
    font-size: 0.88rem;
    font-weight: 700;
  }

  .palette-desc {
    font-size: 0.8rem;
    color: var(--text-secondary);
  }

  .palette-empty {
    padding: 14px;
    color: var(--text-secondary);
    font-size: 0.9rem;
  }
</style>
