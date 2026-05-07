<script lang="ts">
  import { Settings, Plus, Moon, Sun } from 'lucide-svelte';
  import { documentStore } from '../stores/documents';
  import { modeStore } from '../stores/mode';

  let isDark = true;

  function createNewDocument() {
    documentStore.createDocument('Untitled Document');
  }

  function toggleTheme() {
    isDark = !isDark;
    document.documentElement.classList.toggle('dark');
  }
</script>

<header class="bg-gray-800 border-b border-gray-700 px-4 py-3 flex items-center justify-between">
  <div class="flex items-center space-x-3">
    <h1 class="text-xl font-bold text-orange-500">FireNotes</h1>
    <span class="text-xs text-gray-400 bg-gray-700 px-2 py-1 rounded">
      {$modeStore.mode === 'standalone' ? 'Standalone' : 'Core'}
    </span>
  </div>
  
  <div class="flex items-center space-x-2">
    <button 
      on:click={createNewDocument}
      class="flex items-center space-x-1 bg-blue-600 hover:bg-blue-700 text-white px-3 py-1.5 rounded text-sm font-medium"
    >
      <Plus class="w-4 h-4" />
      <span>New</span>
    </button>
    
    <button 
      on:click={toggleTheme}
      class="p-2 hover:bg-gray-700 rounded text-gray-400 hover:text-white"
    >
      {#if isDark}
        <Sun class="w-5 h-5" />
      {:else}
        <Moon class="w-5 h-5" />
      {/if}
    </button>
  </div>
</header>