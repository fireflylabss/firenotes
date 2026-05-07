<script lang="ts">
  import { onDestroy } from 'svelte';
  import { documentStore } from '../stores/documents';
  import { parseMarkdown } from '../lib/markdown';
  import type { Document } from '../stores/documents';

  export let document: Document;

  let content = document.content;
  let title = document.title;
  let showPreview = true;
  let currentDocumentId = document.id;
  let saveTimeout: ReturnType<typeof setTimeout> | undefined;

  $: htmlContent = parseMarkdown(content);

  $: if (document.id !== currentDocumentId) {
    currentDocumentId = document.id;
    if (saveTimeout) clearTimeout(saveTimeout);
    title = document.title;
    content = document.content;
  }

  function saveChanges() {
    documentStore.updateDocument(document.id, {
      title,
      content
    });
  }

  // Auto-save on change
  function scheduleSave() {
    if (saveTimeout) clearTimeout(saveTimeout);
    saveTimeout = setTimeout(saveChanges, 500);
  }

  onDestroy(() => {
    if (saveTimeout) clearTimeout(saveTimeout);
  });

  function getWordCount(): number {
    return content.split(/\s+/).filter(word => word.length > 0).length;
  }

  function getCharacterCount(): number {
    return content.length;
  }

  function getLineCount(): number {
    return content.split('\n').length;
  }
</script>

<div class="flex flex-col h-full">
  <div class="border-b border-gray-700 p-4">
    <input 
      type="text" 
      class="w-full bg-transparent text-2xl font-bold text-white focus:outline-none"
      placeholder="Document title"
      bind:value={title}
      on:input={scheduleSave}
    />
  </div>
  
  <div class="flex-1 flex overflow-hidden">
    <div class="flex-1 flex flex-col {showPreview ? 'w-1/2' : 'w-full'}">
      <textarea 
        class="flex-1 bg-gray-900 text-gray-100 p-4 resize-none focus:outline-none font-mono text-sm leading-relaxed"
        placeholder="Start writing in Markdown..."
        bind:value={content}
        on:input={scheduleSave}
      ></textarea>
    </div>
    
    {#if showPreview}
      <div class="flex-1 border-l border-gray-700 overflow-y-auto bg-gray-850 p-4">
        <div class="prose prose-invert max-w-none">
          {@html htmlContent}
        </div>
      </div>
    {/if}
  </div>
  
  <div class="border-t border-gray-700 px-4 py-2 flex items-center justify-between text-xs text-gray-400">
    <div class="flex items-center space-x-4">
      <span>{getWordCount()} words</span>
      <span>{getCharacterCount()} characters</span>
      <span>{getLineCount()} lines</span>
    </div>
    
    <button 
      on:click={() => showPreview = !showPreview}
      class="px-3 py-1 bg-gray-700 hover:bg-gray-600 rounded text-white"
    >
      {showPreview ? 'Hide Preview' : 'Show Preview'}
    </button>
  </div>
</div>

<style>
  .prose {
    color: #e5e7eb;
    line-height: 1.75;
  }
  
  .prose :global(h1) {
    font-size: 2.25em;
    font-weight: 700;
    margin-bottom: 0.5em;
    margin-top: 0;
    color: white;
  }
  
  .prose :global(h2) {
    font-size: 1.75em;
    font-weight: 600;
    margin-bottom: 0.5em;
    margin-top: 1.5em;
    color: white;
  }
  
  .prose :global(h3) {
    font-size: 1.375em;
    font-weight: 600;
    margin-bottom: 0.5em;
    margin-top: 1.25em;
    color: white;
  }
  
  .prose :global(p) {
    margin-bottom: 1em;
  }
  
  .prose :global(code) {
    background-color: #1f2937;
    color: #e5e7eb;
    padding: 0.2em 0.4em;
    border-radius: 0.25em;
    font-size: 0.875em;
  }
  
  .prose :global(pre) {
    background-color: #1f2937;
    color: #e5e7eb;
    padding: 1em;
    border-radius: 0.5em;
    overflow-x: auto;
    margin-bottom: 1em;
  }
  
  .prose :global(pre code) {
    background-color: transparent;
    padding: 0;
    color: inherit;
  }
  
  .prose :global(ul), .prose :global(ol) {
    padding-left: 1.5em;
    margin-bottom: 1em;
  }
  
  .prose :global(li) {
    margin-bottom: 0.25em;
  }
  
  .prose :global(a) {
    color: #f97316;
    text-decoration: underline;
  }
  
  .prose :global(blockquote) {
    border-left: 4px solid #f97316;
    padding-left: 1em;
    margin-left: 0;
    color: #9ca3af;
    font-style: italic;
  }
  
  .prose :global(table) {
    width: 100%;
    border-collapse: collapse;
    margin-bottom: 1em;
  }
  
  .prose :global(th), .prose :global(td) {
    border: 1px solid #374151;
    padding: 0.5em;
  }
  
  .prose :global(th) {
    background-color: #1f2937;
    font-weight: 600;
  }
  
  .bg-gray-850 {
    background-color: #1f2937;
  }
</style>