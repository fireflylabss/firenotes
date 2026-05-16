<script lang="ts">
  import { onMount, onDestroy, tick } from 'svelte';
  import { Editor } from '@tiptap/core';
  import StarterKit from '@tiptap/starter-kit';
  import { Markdown } from 'tiptap-markdown';

  export let content: string = '';
  export let onUpdate: (content: string) => void = () => {};
  
  let editorElement: HTMLDivElement;
  let editor: Editor | null = null;
  let isInternalUpdate = false;

  onMount(() => {
    editor = new Editor({
      element: editorElement,
      extensions: [
        StarterKit,
        Markdown.configure({
          html: false,
          tightLists: true,
          bulletListMarker: '-',
          linkify: false,
          breaks: true,
        }),
      ],
      content,
      contentType: 'markdown',
      onUpdate: ({ editor }) => {
        if (!isInternalUpdate) {
          const markdown = editor.storage.markdown.getMarkdown();
          onUpdate(markdown);
        }
      },
      editorProps: {
        attributes: {
          class: 'live-editor-content prose max-w-none focus:outline-none',
        },
      },
    });

    return () => {
      if (editor) {
        editor.destroy();
        editor = null;
      }
    };
  });

  onDestroy(() => {
    if (editor) {
      editor.destroy();
      editor = null;
    }
  });

  // Update editor content when prop changes (but not from internal updates)
  $: if (editor && content !== undefined && !isInternalUpdate) {
    const currentMarkdown = editor.storage.markdown.getMarkdown();
    if (currentMarkdown !== content) {
      isInternalUpdate = true;
      editor.commands.setContent(content, { contentType: 'markdown' });
      tick().then(() => {
        isInternalUpdate = false;
      });
    }
  }
</script>

<div class="live-editor" on:click={() => editor?.view.focus()}>
  <div class="experimental-banner">
    <span>⚗️ Experimental Mode - WYSIWYG editor is in beta</span>
  </div>
  <div bind:this={editorElement} class="live-editor-container"></div>
</div>

<style>
  .live-editor {
    flex: 1;
    overflow-y: auto;
    overflow-x: hidden;
    padding: 0;
    background: var(--bg-surface);
    display: flex;
    flex-direction: column;
    gap: 0;
    height: 100%;
  }

  .experimental-banner {
    background: linear-gradient(135deg, rgba(245, 158, 11, 0.1), rgba(245, 158, 11, 0.05));
    border-bottom: 1px solid rgba(245, 158, 11, 0.3);
    padding: 8px 16px;
    font-size: 0.75rem;
    color: #f59e0b;
    font-weight: 600;
    text-align: center;
    letter-spacing: 0.5px;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 6px;
  }

  .live-editor-container {
    flex: 1;
    overflow-y: auto;
    overflow-x: hidden;
    padding: 2.4rem max(2.4rem, calc(50% - 360px));
    background: var(--bg-surface);
    display: flex;
    flex-direction: column;
    gap: 2px;
    min-height: 100%;
  }

  .live-editor-content {
    color: var(--text-primary);
    line-height: 1.75;
    font-size: 16px;
    outline: none !important;
    border: none !important;
    box-shadow: none !important;
  }

  .live-editor-content:global(.ProseMirror) {
    outline: none !important;
    border: none !important;
    box-shadow: none !important;
  }

  .live-editor-content:global(.ProseMirror-focused) {
    outline: none !important;
    border: none !important;
    box-shadow: none !important;
  }

  .live-editor-container:global(.ProseMirror) {
    outline: none !important;
    border: none !important;
    box-shadow: none !important;
  }

  .live-editor-container:global(.ProseMirror-focused) {
    outline: none !important;
    border: none !important;
    box-shadow: none !important;
  }

  .live-editor-content :global(h1) {
    font-size: 1.9em;
    font-weight: 700;
    margin: 0.5em 0;
    line-height: 1.3;
    color: var(--text-primary);
  }

  .live-editor-content :global(h2) {
    font-size: 1.45em;
    font-weight: 700;
    margin: 0.5em 0;
    line-height: 1.3;
    color: var(--text-primary);
  }

  .live-editor-content :global(h3) {
    font-size: 1.18em;
    font-weight: 700;
    margin: 0.5em 0;
    line-height: 1.3;
    color: var(--text-primary);
  }

  .live-editor-content :global(p) {
    margin: 0.5em 0;
  }

  .live-editor-content :global(ul),
  .live-editor-content :global(ol) {
    margin: 0.5em 0;
    padding-left: 1.4em;
  }

  .live-editor-content :global(blockquote) {
    margin: 0.5em 0;
    padding: 0.3em 1em;
    border-left: 3px solid var(--accent-primary);
    color: var(--text-secondary);
    background: color-mix(in srgb, var(--accent-primary) 6%, transparent);
    border-radius: 0 4px 4px 0;
  }

  .live-editor-content :global(code) {
    font-family: 'JetBrains Mono', 'Fira Code', monospace;
    font-size: 0.88em;
    background: var(--bg-hover);
    padding: 0.1em 0.35em;
    border-radius: 3px;
  }

  .live-editor-content :global(pre) {
    background: var(--bg-hover);
    padding: 1em;
    border-radius: 6px;
    overflow-x: auto;
    margin: 0.5em 0;
  }

  .live-editor-content :global(pre code) {
    background: transparent;
    padding: 0;
  }

  .live-editor-content :global(hr) {
    border: none;
    border-top: 1px solid var(--border-subtle);
    margin: 1em 0;
  }

  .live-editor-content :global(a) {
    color: var(--accent-primary);
    text-decoration: underline;
  }

  .live-editor-content :global(a:hover) {
    color: var(--accent-hover);
  }

  /* Selected text */
  .live-editor-content ::selection {
    background: var(--accent-primary);
    color: #000;
  }

  /* Placeholder */
  .live-editor-content:global(.is-empty):before {
    color: var(--text-tertiary);
    content: attr(data-placeholder);
    pointer-events: none;
    position: absolute;
    height: 0;
  }
</style>