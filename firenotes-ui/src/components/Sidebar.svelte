<script lang="ts">
  import { documentStore } from '../stores/documents';
  import { Search, Trash2, FileText } from 'lucide-svelte';
  import type { Document } from '../stores/documents';

  let searchQuery = '';
  let showDeleteConfirm = false;
  let documentToDelete: string | null = null;

  $: filteredDocuments = $documentStore.documents.filter(doc => 
    doc.title.toLowerCase().includes(searchQuery.toLowerCase()) ||
    doc.content.toLowerCase().includes(searchQuery.toLowerCase())
  );

  function selectDocument(doc: Document) {
    documentStore.setCurrentDocument(doc);
  }

  function handleDocumentKeydown(event: KeyboardEvent, doc: Document) {
    if (event.key === 'Enter' || event.key === ' ') {
      event.preventDefault();
      selectDocument(doc);
    }
  }

  function confirmDelete(docId: string) {
    documentToDelete = docId;
    showDeleteConfirm = true;
  }

  async function deleteDocument() {
    if (documentToDelete) {
      await documentStore.deleteDocument(documentToDelete);
      showDeleteConfirm = false;
      documentToDelete = null;
    }
  }
</script>

<aside class="w-64 bg-gray-800 border-r border-gray-700 flex flex-col">
  <div class="p-4 border-b border-gray-700">
    <div class="relative">
      <Search class="w-4 h-4 absolute left-3 top-1/2 transform -translate-y-1/2 text-gray-400" />
      <input 
        type="text" 
        placeholder="Search documents..." 
        class="w-full bg-gray-700 text-white pl-10 pr-3 py-2 rounded text-sm focus:outline-none focus:ring-2 focus:ring-blue-500"
        bind:value={searchQuery}
      />
    </div>
  </div>
  
  <div class="flex-1 overflow-y-auto">
    {#if filteredDocuments.length === 0}
      <div class="p-4 text-center text-gray-400 text-sm">
        {searchQuery ? 'No results found' : 'No documents yet'}
      </div>
    {:else}
      <div class="p-2 space-y-1">
        {#each filteredDocuments as doc (doc.id)}
          <div 
            class="group flex items-center justify-between p-2 rounded hover:bg-gray-700 cursor-pointer {doc.id === $documentStore.currentDocument?.id ? 'bg-gray-700' : ''}"
            role="button"
            tabindex="0"
            on:click={() => selectDocument(doc)}
            on:keydown={(event) => handleDocumentKeydown(event, doc)}
          >
            <div class="flex items-center space-x-2 flex-1 min-w-0">
              <FileText class="w-4 h-4 text-gray-400 flex-shrink-0" />
              <span class="text-sm truncate">{doc.title}</span>
            </div>
            <button 
              class="opacity-0 group-hover:opacity-100 p-1 hover:bg-gray-600 rounded text-red-400"
              on:click|stopPropagation={() => confirmDelete(doc.id)}
            >
              <Trash2 class="w-4 h-4" />
            </button>
          </div>
        {/each}
      </div>
    {/if}
  </div>
  
  <div class="p-4 border-t border-gray-700 text-xs text-gray-400">
    <div class="flex justify-between">
      <span>{filteredDocuments.length} documents</span>
      <span>FireNotes v0.1.0</span>
    </div>
  </div>
</aside>

{#if showDeleteConfirm}
  <div class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
    <div class="bg-gray-800 rounded-lg p-6 w-96 max-w-full">
      <h3 class="text-lg font-bold mb-2">Delete Document?</h3>
      <p class="text-gray-400 text-sm mb-4">This action cannot be undone.</p>
      <div class="flex space-x-3">
        <button 
          on:click={() => { showDeleteConfirm = false; documentToDelete = null; }}
          class="flex-1 bg-gray-700 hover:bg-gray-600 text-white py-2 rounded"
        >
          Cancel
        </button>
        <button 
          on:click={deleteDocument}
          class="flex-1 bg-red-600 hover:bg-red-700 text-white py-2 rounded"
        >
          Delete
        </button>
      </div>
    </div>
  </div>
{/if}