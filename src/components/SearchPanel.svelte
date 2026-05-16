<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import type { Document } from '../stores/stores';

  export let show = false;
  export let documents: Document[] = [];

  const dispatch = createEventDispatcher<{ close: void; jumpTo: string; replaceAll: { query: string; replacement: string } }>();

  let searchQuery = '';
  let replaceQuery = '';

  function escapeRegExp(value: string) {
    return value.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
  }

  $: results = searchQuery.trim()
    ? documents
        .filter((doc) =>
          doc.content.toLowerCase().includes(searchQuery.toLowerCase()) ||
          doc.name.toLowerCase().includes(searchQuery.toLowerCase())
        )
        .map((doc) => ({
          id: doc.id,
          name: doc.name,
          matches: (doc.content.match(new RegExp(escapeRegExp(searchQuery), 'gi')) || []).length,
        }))
    : [];

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') {
      e.preventDefault();
      dispatch('close');
      return;
    }
    if (e.key === 'Enter') {
      e.preventDefault();
      const first = results[0];
      if (first) dispatch('jumpTo', first.id);
    }
  }

  function handleReplaceAll() {
    if (!searchQuery.trim()) return;
    dispatch('replaceAll', { query: searchQuery, replacement: replaceQuery });
  }
</script>

{#if show}
  <div class="palette-backdrop" role="presentation" on:click={() => dispatch('close')}></div>
  <div class="floating-panel">
    <div class="search-panel-header">
      <h3>Global search</h3>
      <button class="status-icon-btn" on:click={() => dispatch('close')} title="Close">×</button>
    </div>
    <input class="panel-input" bind:value={searchQuery} placeholder="Search in all documents" on:keydown={handleKeydown} />
    <input class="panel-input" bind:value={replaceQuery} placeholder="Replace with" on:keydown={handleKeydown} />
    <div class="search-panel-actions">
      <button class="btn" on:click={handleReplaceAll}>Replace all</button>
      <span class="palette-desc">{results.length} result(s)</span>
    </div>
    <div class="search-list">
      {#each results as result}
        <button class="palette-item" on:click={() => dispatch('jumpTo', result.id)}>
          <div class="palette-title">{result.name}</div>
          <div class="palette-desc">{result.matches} matches</div>
        </button>
      {/each}
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

  .floating-panel {
    position: fixed;
    top: 50%;
    left: 50%;
    transform: translate(-50%, -50%);
    width: min(560px, calc(100vw - 28px));
    max-height: calc(100vh - 160px);
    overflow: auto;
    border: 1px solid var(--border-subtle);
    border-radius: 8px;
    background: color-mix(in srgb, var(--bg-surface) 92%, transparent);
    box-shadow: var(--shadow-lg);
    z-index: 55;
    padding: 12px;
    display: flex;
    flex-direction: column;
    gap: 8px;
    backdrop-filter: blur(8px);
    animation: searchPanelIn 180ms ease-out;
  }

  .floating-panel h3 {
    font-size: 0.93rem;
    font-weight: 700;
  }

  .search-panel-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    border-bottom: 1px solid var(--border-subtle);
    padding-bottom: 6px;
  }

  .search-panel-actions {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
  }

  .search-list {
    max-height: 320px;
    overflow: auto;
    display: flex;
    flex-direction: column;
    gap: 3px;
  }

  .panel-input {
    border: 1px solid var(--border-subtle);
    background: var(--bg-surface);
    color: var(--text-primary);
    border-radius: 6px;
    padding: 0.4rem 0.55rem;
    font-size: 0.78rem;
    min-height: 34px;
    outline: none;
    transition: border-color var(--transition-fast), box-shadow var(--transition-fast);
  }

  .panel-input:focus {
    border-color: var(--accent-primary);
    box-shadow: 0 0 0 3px color-mix(in srgb, var(--accent-primary) 14%, transparent);
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

  .btn {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    gap: 0.45rem;
    border: 1px solid transparent;
    border-radius: 6px;
    background: transparent;
    color: var(--text-secondary);
    padding: 0.5rem 0.8rem;
    font-size: 0.82rem;
    font-weight: 700;
    cursor: pointer;
    transition: all var(--transition-fast);
  }

  .btn:hover {
    border-color: var(--border-subtle);
    background: var(--bg-hover);
    color: var(--text-primary);
    transform: translateY(-1px);
  }

  .status-icon-btn {
    border: none;
    background: transparent;
    color: var(--text-secondary);
    min-height: 20px;
    height: 20px;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
    font-size: 0.75rem;
    font-weight: 700;
    padding: 0 4px;
  }

  .status-icon-btn:hover {
    color: var(--text-primary);
    background: transparent;
  }

  @keyframes searchPanelIn {
    from {
      opacity: 0;
      transform: translate(-50%, calc(-50% + 8px));
    }
    to {
      opacity: 1;
      transform: translate(-50%, -50%);
    }
  }

  @media (max-width: 900px) {
    .floating-panel {
      top: auto;
      bottom: 54px;
      left: 8px;
      right: 8px;
      transform: none;
      width: auto;
      max-height: 52vh;
    }
  }
</style>
