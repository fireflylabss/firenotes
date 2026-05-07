<script lang="ts">
  import { onMount, tick } from 'svelte';
  import { marked } from 'marked';
  import { markedHighlight } from 'marked-highlight';
  import hljs from 'highlight.js';
  import katex from 'katex';
  import 'katex/dist/katex.min.css';
  import EmojiPicker from './EmojiPicker.svelte';
  import { 
    Plus, 
    X, 
    Sun, 
    Moon, 
    Download, 
    FileText,
    WrapText,
    Search,
    Upload,
    FileUp,
    FileDown,
    Bold,
    Italic,
    Code,
    Link2,
    Heading1,
    Heading2,
    Heading3,
    Subscript,
    Superscript,
    List,
    ListOrdered,
    ListTodo,
    Quote,

    Strikethrough,
    Table2,
    ImagePlus,
    SquareCode,
    Minus,
    Smile,
    Ellipsis,
    ScanText,
    Underline,
    Maximize,
    Minimize,
    Eye,
    Columns,
    PenLine,
    LayoutTemplate,
    Zap,
    Undo2,
    Redo2,
    ListCollapse
  } from 'lucide-svelte';
  import { appStore, activeDocument, undo, redo, canUndo, canRedo, pushUndoIfNeeded, initUndoForDoc } from './stores';
  import type { Document } from './stores';
  import CommandPalette from './CommandPalette.svelte';
  import SearchPanel from './SearchPanel.svelte';

  type PaletteCommand = {
    id: string;
    label: string;
    desc: string;
    run: () => void;
  };
  
  // ============================================
  // MARKED CONFIGURATION
  // ============================================

  // Extension: definitional lists (Term + ": definition")
  const definitionListExtension = {
    name: 'definitionList',
    level: 'block',
    start(src: string) {
      return src.match(/^\S[^\n]*\n: /m)?.index ?? -1;
    },
    tokenizer(src: string) {
      const rule = /^(?<body>(?:\S[^\n]*\n: .+(?:\n|$))+)/;
      const match = rule.exec(src);
      if (!match?.groups?.body) return;

      const items: { term: string; description: string }[] = [];
      const lines = match.groups.body.trimEnd().split(/\n/);
      for (let i = 0; i < lines.length; i++) {
        const term = lines[i];
        const descLine = lines[i + 1];
        if (!descLine?.startsWith(': ')) continue;
        items.push({ term: term.trim(), description: descLine.replace(/^: /, '').trim() });
        i++; // skip description line
      }

      return {
        type: 'definitionList',
        raw: match.groups.body,
        items,
      } as const;
    },
    renderer(token: unknown): string {
      const { items } = token as { items: { term: string; description: string }[] };
      const rendered = items
        .map((item) => `<dt>${item.term}</dt><dd>${item.description}</dd>`)
        .join('');
      return `<dl>${rendered}</dl>`;
    },
  } as const;

  // Extension: callouts :::type Optional title\ncontent\n:::
  const calloutTypes = ['info', 'warn', 'warning', 'error', 'success', 'tip'];
  const calloutExtension = {
    name: 'callout',
    level: 'block',
    start(src: string) {
      return src.match(/^:::/m)?.index ?? -1;
    },
    tokenizer(this: any, src: string) {
      const rule = /^:::(?<type>\w+)(?:\s+(?<title>[^\n]+))?\n(?<body>[\s\S]+?)\n:::\s*(?:\n|$)/;
      const match = rule.exec(src);
      if (!match?.groups?.type || !match.groups.body) return;
      const type = match.groups.type.toLowerCase();
      if (!calloutTypes.includes(type)) return;

      return {
        type: 'callout',
        raw: match[0],
        calloutType: type,
        title: match.groups.title?.trim() || type.toUpperCase(),
        text: this.lexer.inlineTokens(match.groups.body.trim()),
      } as const;
    },
    renderer(this: any, token: unknown): string {
      const { calloutType, title, text } = token as {
        calloutType: string;
        title: string;
        text: any;
      };
      const inner: string = this.parser.parseInline(text);
      return `<div class="callout ${calloutType}"><div class="callout-title">${title}</div><div class="callout-body">${inner}</div></div>`;
    },
  } as const;

  // Extension: math (inline $...$ and block $$...$$) rendered with KaTeX
  const inlineMathExtension = {
    name: 'inlineMath',
    level: 'inline',
    start(src: string) {
      return src.indexOf('$');
    },
    tokenizer(src: string) {
      const rule = /^\$(.+?)\$/;
      const match = rule.exec(src);
      if (!match) return;
      if (!match[1].trim()) return;

      return {
        type: 'inlineMath',
        raw: match[0],
        text: match[1].trim(),
      } as const;
    },
    renderer(token: unknown): string {
      const { text } = token as { text: string };
      return katex.renderToString(text, { throwOnError: false, displayMode: false });
    },
  } as const;

  const blockMathExtension = {
    name: 'blockMath',
    level: 'block',
    start(src: string) {
      return src.match(/^\$\$/m)?.index ?? -1;
    },
    tokenizer(src: string) {
      const rule = /^\$\$\s*\n?([\s\S]+?)\n?\$\$\s*(?:\n|$)/;
      const match = rule.exec(src);
      if (!match?.[1]) return;

      return {
        type: 'blockMath',
        raw: match[0],
        text: match[1].trim(),
      } as const;
    },
    renderer(token: unknown): string {
      const { text } = token as { text: string };
      const html = katex.renderToString(text, { throwOnError: false, displayMode: true });
      return `<div class="math-block">${html}</div>`;
    },
  } as const;

  marked.use({ extensions: [definitionListExtension, calloutExtension, blockMathExtension, inlineMathExtension] });

  marked.use(
    markedHighlight({
      langPrefix: 'hljs language-',
      highlight(code, lang) {
        const language = hljs.getLanguage(lang) ? lang : 'plaintext';
        return hljs.highlight(code, { language }).value;
      },
    })
  );
  
  // Configure marked for GFM
  marked.setOptions({
    gfm: true,
    breaks: true,
    pedantic: false,
    async: false,
  });
  
  // ============================================
  // STATE
  // ============================================
  
  let editorTextarea: HTMLTextAreaElement;
  let lineNumbersEl: HTMLDivElement;
  let previewContainer: HTMLDivElement;
  let editingNameId: string | null = null;
  let editingNameValue = '';
  let isResizing = false;
  let sidebarWidth = 50;

  let liveMessage = '';
  let showPalette = false;
  let paletteFilter = '';
  let paletteInputEl: HTMLInputElement | null = null;
  let selectionLength = 0;
  let caretLine = 1;
  let caretCol = 1;
  let showSearchPanel = false;
  let searchQuery = '';
  let replaceQuery = '';
  let isFullscreen = false;
  let showTableMenu = false;
  let tableHoverRows = 0;
  let tableHoverCols = 0;
  let savedTableSelStart = 0;
  let savedTableSelEnd = 0;
  let showEmojiMenu = false;
  let savedEmojiSelStart = 0;
  let savedEmojiSelEnd = 0;

  let showToolbarOverflowMenu = false;
  let showImportMenu = false;
  let showExportMenu = false;
  let toolbarHostEl: HTMLDivElement | null = null;
  let importInputEl: HTMLInputElement | null = null;

  // Link / Image modals
  let showLinkModal = false;
  let linkModalText = '';
  let linkModalUrl = '';
  let showImageModal = false;
  let imageModalAlt = '';
  let imageModalUrl = '';

  // Live editor state
  let liveBlocks: string[] = [''];
  let liveActiveBlock: number | null = null;
  let liveBlockDraft = '';
  let liveBlockTextarea: HTMLTextAreaElement | null = null;
  let isEditorDragOver = false;
  let viewMode: 'write' | 'preview' | 'split' | 'live' = 'split';
  let uiZoom = 100;
  let showMobileDevNotice = false;
  const MAX_TABS_PER_WORKSPACE = 32;
  let showTemplateMenu = false;

  // Undo/Redo tracking
  let prevDocId: string | null = null;
  let lastTrackedContent: string = '';
  let undoBtnActive = false;
  let redoBtnActive = false;

  // Table of Contents
  let showToc = false;
  type TocEntry = { level: number; text: string; id: string };
  let tocEntries: TocEntry[] = [];

  type Template = {
    id: string;
    emoji: string;
    name: string;
    description: string;
    content: string;
  };

  const TEMPLATES: Template[] = [
    {
      id: 'meeting-notes',
      emoji: '🗓️',
      name: 'Meeting Notes',
      description: 'Structure your meetings with agenda, attendees, action items, and decisions.',
      content: `# Meeting Notes

**Date:** ${new Date().toLocaleDateString('en-US', { year: 'numeric', month: 'long', day: 'numeric' })}
**Project:** 
**Facilitator:** 

---

## Attendees

- 

## Agenda

1. 
2. 
3. 

## Discussion

### Topic 1

_Notes here..._

## Decisions Made

- [ ] Decision 1
- [ ] Decision 2

## Action Items

| Task | Owner | Due Date |
| --- | --- | --- |
|  |  |  |

## Next Meeting

**Date:** 
**Topics:**
`,
    },
    {
      id: 'weekly-review',
      emoji: '📊',
      name: 'Weekly Review',
      description: 'Reflect on wins, blockers, learnings, and set priorities for the next week.',
      content: `# Weekly Review

**Week of:** ${new Date().toLocaleDateString('en-US', { year: 'numeric', month: 'long', day: 'numeric' })}

---

## 🏆 Wins This Week

- 
- 

## 🚧 Blockers & Challenges

- 
- 

## 📚 Key Learnings

- 
- 

## 📈 Metrics

| Metric | Target | Actual |
| --- | --- | --- |
|  |  |  |

## 🎯 Top 3 Priorities for Next Week

1. 
2. 
3. 

## 💡 Ideas & Opportunities

- 

## 🔋 Energy & Mood

> Rate your week and add any personal notes.
`,
    },
    {
      id: 'project-spec',
      emoji: '📐',
      name: 'Project Spec',
      description: 'Define your project scope, goals, tech stack, and milestones clearly.',
      content: `# Project Specification

**Project Name:** 
**Version:** 1.0
**Author:** 
**Date:** ${new Date().toLocaleDateString('en-US', { year: 'numeric', month: 'long', day: 'numeric' })}

---

## Overview

_A brief description of what this project does and why it exists._

## Goals

- 
- 

## Non-Goals

- 

## Tech Stack

| Layer | Technology |
| --- | --- |
| Frontend |  |
| Backend |  |
| Database |  |
| Hosting |  |

## Architecture

_Describe the high-level architecture here._

## Milestones

| Milestone | Description | Due Date |
| --- | --- | --- |
| v0.1 Alpha |  |  |
| v1.0 Release |  |  |

## Open Questions

- [ ] 
- [ ] 

## References

- 
`,
    },
    {
      id: 'daily-planner',
      emoji: '☀️',
      name: 'Daily Planner',
      description: 'Start your day with intention: priorities, time blocks, and an evening reflection.',
      content: `# Daily Planner

**${new Date().toLocaleDateString('en-US', { weekday: 'long', year: 'numeric', month: 'long', day: 'numeric' })}**

---

## 🌅 Morning Intentions

> What would make today a great day?

_Write here..._

## 🎯 Top 3 Priorities

- [ ] 
- [ ] 
- [ ] 

## 🕐 Time Blocks

| Time | Task | Done |
| --- | --- | --- |
| 9:00 – 10:00 |  | ☐ |
| 10:00 – 11:00 |  | ☐ |
| 11:00 – 12:00 |  | ☐ |
| 12:00 – 13:00 | 🍽️ Lunch |  |
| 13:00 – 14:00 |  | ☐ |
| 14:00 – 16:00 |  | ☐ |
| 16:00 – 17:00 |  | ☐ |

## 📝 Notes & Quick Captures

- 

## 🌙 Evening Reflection

**What went well?** 

**What could be improved?** 

**Gratitude:** 
`,
    },
    {
      id: 'bug-report',
      emoji: '🐛',
      name: 'Bug Report',
      description: 'Document software bugs with reproduction steps, environment details, and expected behavior.',
      content: `# Bug Report

**Bug ID:** BUG-
**Severity:** 🔴 Critical / 🟠 High / 🟡 Medium / 🟢 Low
**Reported by:** 
**Date:** ${new Date().toLocaleDateString('en-US', { year: 'numeric', month: 'long', day: 'numeric' })}

---

## Summary

_One-sentence description of the bug._

## Steps to Reproduce

1. Go to...
2. Click on...
3. Observe...

## Expected Behavior

_What should happen._

## Actual Behavior

_What actually happens._

## Screenshots / Logs

_Paste screenshots or error logs here._

\`\`\`
Error output here
\`\`\`

## Environment

| Property | Value |
| --- | --- |
| OS |  |
| Browser / Runtime |  |
| App Version |  |
| Node / Bun |  |

## Possible Cause

_Optional: any hypothesis about the root cause._

## Fix Notes

_Added after resolution._
`,
    },
    {
      id: 'content-brief',
      emoji: '✍️',
      name: 'Content Brief',
      description: 'Plan blog posts, articles, or videos with audience, outline, and SEO in mind.',
      content: `# Content Brief

**Title:** 
**Format:** Blog Post / Video / Thread / Newsletter
**Author:** 
**Target Publish Date:** 

---

## Goal

_What should the reader/viewer take away from this piece?_

## Target Audience

- **Who are they?** 
- **Pain points:** 
- **What they know:** 

## Hook / Opening Line

_Write a compelling first sentence._

## Outline

### Introduction
- 

### Section 1: 
- 

### Section 2: 
- 

### Section 3: 
- 

### Conclusion / CTA
- 

## SEO Keywords

- Primary: 
- Secondary: 

## References & Research

- 

## Notes

_Any additional context for the writer._
`,
    },
    {
      id: 'retrospective',
      emoji: '🔄',
      name: 'Sprint Retrospective',
      description: 'Run team retrospectives with the classic What Went Well / Improve / Actions format.',
      content: `# Sprint Retrospective

**Sprint:** #
**Date:** ${new Date().toLocaleDateString('en-US', { year: 'numeric', month: 'long', day: 'numeric' })}
**Team:** 
**Facilitator:** 

---

## ✅ What Went Well

- 
- 
- 

## 🔧 What Could Be Improved

- 
- 
- 

## 💡 New Ideas & Experiments

- 
- 

## 🚫 Stop Doing

- 

## 📋 Action Items

| Action | Owner | By When |
| --- | --- | --- |
|  |  |  |

## 😊 Team Morale

> Rate the team's overall energy this sprint (1–10) and add comments.

**Score:** /10

**Comments:** 
`,
    },
    {
      id: 'book-notes',
      emoji: '📖',
      name: 'Book Notes',
      description: 'Capture key insights, quotes, and action points from what you read.',
      content: `# Book Notes

**Title:** 
**Author:** 
**Rating:** ⭐⭐⭐⭐⭐
**Finished:** ${new Date().toLocaleDateString('en-US', { year: 'numeric', month: 'long', day: 'numeric' })}

---

## In One Sentence

_The book in a single sentence._

## Key Ideas

### Idea 1: 

_Explanation..._

### Idea 2: 

_Explanation..._

### Idea 3: 

_Explanation..._

## Favourite Quotes

> "Quote here."
> — Author Name

> "Quote here."
> — Author Name

## What I'll Apply

- [ ] 
- [ ] 

## Who Should Read This

_Describe the ideal reader._

## Further Reading

- _Related book or resource_
`,
    },
  ];

  const editorStateByDoc = new Map<string, { selectionStart: number; selectionEnd: number; scrollTop: number }>();
  const previewScrollByDoc = new Map<string, number>();
  
  $: currentDoc = $activeDocument;
  $: visibleDocuments = $appStore.documents;
  $: workspaceTabCount = visibleDocuments.length;
  $: canCreateMoreTabs = workspaceTabCount < MAX_TABS_PER_WORKSPACE;
  $: renderedMarkdown = currentDoc ? (marked.parse(currentDoc.content) as string) : '';
  $: lineCount = currentDoc ? currentDoc.content.split('\n').length : 1;
  $: searchResults = searchQuery.trim()
    ? $appStore.documents
        .filter((doc) => doc.content.toLowerCase().includes(searchQuery.toLowerCase()) || doc.name.toLowerCase().includes(searchQuery.toLowerCase()))
        .map((doc) => ({
          id: doc.id,
          name: doc.name,
          matches: (doc.content.match(new RegExp(searchQuery.replace(/[.*+?^${}()|[\]\\]/g, '\\$&'), 'gi')) || []).length,
        }))
    : [];
  $: 
    if ($appStore) {
      sidebarWidth = $appStore.sidebarWidth;
    }

  $: filteredCommands = paletteCommands.filter((cmd) =>
    `${cmd.label} ${cmd.desc}`.toLowerCase().includes(paletteFilter.toLowerCase())
  );

  $: wordCount = currentDoc ? (currentDoc.content.trim() ? currentDoc.content.trim().split(/\s+/).length : 0) : 0;

  // Undo/Redo: track document switches
  $: if (currentDoc && currentDoc.id !== prevDocId) {
    if (prevDocId && lastTrackedContent) {
      pushUndoIfNeeded(prevDocId, lastTrackedContent);
    }
    prevDocId = currentDoc.id;
    lastTrackedContent = currentDoc.content;
    initUndoForDoc(currentDoc.id, currentDoc.content);
  }

  // Undo/Redo: track content changes
  $: if (currentDoc && currentDoc.content !== lastTrackedContent && prevDocId === currentDoc.id) {
    if (lastTrackedContent) {
      pushUndoIfNeeded(currentDoc.id, lastTrackedContent);
    }
    lastTrackedContent = currentDoc.content;
  }

  // Update undo/redo button states
  $: undoBtnActive = currentDoc ? canUndo(currentDoc.id) : false;
  $: redoBtnActive = currentDoc ? canRedo(currentDoc.id) : false;

  // Table of Contents: parse headings
  $: tocEntries = currentDoc
    ? currentDoc.content.split('\n').reduce<TocEntry[]>((acc, line, i) => {
        const match = line.match(/^(#{1,6})\s+(.+)/);
        if (match) {
          const text = match[2].replace(/[*_`~]/g, '').trim();
          const id = text.toLowerCase().replace(/[^\w\u00C0-\u024F]+/g, '-').replace(/(^-|-$)/g, '');
          acc.push({ level: match[1].length, text, id: id || `heading-${i}` });
        }
        return acc;
      }, [])
    : [];

  $: if (previewContainer && renderedMarkdown) {
    tick().then(() => {
      enhancePreviewBlocks();
    });
  }

  $: if (currentDoc) {
    // Restore editor/preview positions when switching docs
    tick().then(() => {
      restoreEditorState(currentDoc);
    });
  }

  const paletteCommands: PaletteCommand[] = [
    { id: 'new', label: 'New document', desc: 'Create empty doc', run: handleNewDocument },
    { id: 'undo', label: 'Undo', desc: 'Undo last change', run: handleUndo },
    { id: 'redo', label: 'Redo', desc: 'Redo last change', run: handleRedo },
    { id: 'download', label: 'Download markdown', desc: 'Save current doc', run: handleDownload },
    { id: 'toggle-theme', label: 'Toggle theme', desc: 'Light/Dark', run: () => appStore.toggleTheme() },
    { id: 'toggle-wrap', label: 'Toggle word wrap', desc: 'Wrap editor lines', run: () => appStore.toggleWordWrap() },
    { id: 'focus-editor', label: 'Focus editor', desc: 'Move caret to editor', run: () => editorTextarea?.focus() },
    { id: 'search', label: 'Global search', desc: 'Search across documents', run: () => (showSearchPanel = true) },
    { id: 'export-html', label: 'Export HTML', desc: 'Save rendered document as HTML', run: handleExportHtml },
    { id: 'export-pdf', label: 'Export PDF', desc: 'Print rendered document to PDF', run: handleExportPdf },
    { id: 'zoom-in', label: 'Zoom in', desc: 'Increase editor + preview scale', run: zoomIn },
    { id: 'zoom-out', label: 'Zoom out', desc: 'Decrease editor + preview scale', run: zoomOut },
    { id: 'zoom-reset', label: 'Reset zoom', desc: 'Back to 100%', run: resetZoom },
  ];

  function escapeRegExp(value: string) {
    return value.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
  }

  // Palette helpers
  function openPalette() {
    showPalette = true;
    paletteFilter = '';
    tick().then(() => paletteInputEl?.focus());
  }

  function closePalette() {
    showPalette = false;
    paletteFilter = '';
  }

  function runPaletteCommand(cmd: PaletteCommand) {
    closePalette();
    cmd.run();
  }

  function handlePaletteKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') {
      e.preventDefault();
      closePalette();
      return;
    }
    if (e.key === 'Enter') {
      e.preventDefault();
      const first = filteredCommands[0];
      if (first) runPaletteCommand(first);
    }
  }

  function handlePaletteInput(e: Event) {
    const target = e.target as HTMLInputElement | null;
    paletteFilter = target?.value ?? '';
  }

  // Paste handler: insert pasted images as markdown references
  function handlePaste(e: ClipboardEvent) {
    if (!currentDoc || !editorTextarea) return;
    const items = e.clipboardData?.items;
    if (!items) return;

    for (const item of items) {
      if (item.kind === 'file' && item.type.startsWith('image/')) {
        e.preventDefault();
        const file = item.getAsFile();
        if (!file) continue;
        const url = URL.createObjectURL(file);
        const label = `pasted-image-${Date.now()}`;

        const { selectionStart, selectionEnd, value } = editorTextarea;
        const before = value.slice(0, selectionStart ?? 0);
        const after = value.slice(selectionEnd ?? selectionStart ?? 0);
        const insertion = `![${label}](${url})`;

        const updated = `${before}${insertion}${after}`;
        appStore.updateDocumentContent(currentDoc.id, updated);

        tick().then(() => {
          const caret = (selectionStart ?? 0) + insertion.length;
          editorTextarea.selectionStart = caret;
          editorTextarea.selectionEnd = caret;
        });
        break;
      }
    }
  }

  // Format markdown tables in current document
  function handleFormatTables() {
    if (!currentDoc) return;
    const formatted = formatTables(currentDoc.content);
    appStore.updateDocumentContent(currentDoc.id, formatted);
    announce('Tables formatted');
  }

  // Undo/Redo handlers
  function handleUndo() {
    if (!currentDoc || !editorTextarea) return;
    const prev = undo(currentDoc.id, currentDoc.content);
    if (prev === null) return;
    const { selectionStart, selectionEnd } = editorTextarea;
    appStore.updateDocumentContent(currentDoc.id, prev);
    lastTrackedContent = prev;
    tick().then(() => {
      if (!editorTextarea) return;
      editorTextarea.selectionStart = Math.min(selectionStart, prev.length);
      editorTextarea.selectionEnd = Math.min(selectionEnd, prev.length);
    });
  }

  function handleRedo() {
    if (!currentDoc || !editorTextarea) return;
    const next = redo(currentDoc.id, currentDoc.content);
    if (next === null) return;
    const { selectionStart, selectionEnd } = editorTextarea;
    appStore.updateDocumentContent(currentDoc.id, next);
    lastTrackedContent = next;
    tick().then(() => {
      if (!editorTextarea) return;
      editorTextarea.selectionStart = Math.min(selectionStart, next.length);
      editorTextarea.selectionEnd = Math.min(selectionEnd, next.length);
    });
  }

  // TOC: scroll to heading in preview
  function scrollToHeading(entry: TocEntry) {
    if (!previewContainer) return;
    const headings = previewContainer.querySelectorAll('h1, h2, h3, h4, h5, h6');
    for (const el of Array.from(headings)) {
      const text = el.textContent?.replace(/[*_`~]/g, '').trim() || '';
      if (text === entry.text) {
        el.scrollIntoView({ behavior: 'smooth', block: 'start' });
        break;
      }
    }
  }

  function formatTables(markdown: string): string {
    const lines = markdown.split('\n');
    const result: string[] = [];
    let i = 0;

    while (i < lines.length) {
      const line = lines[i];
      const isTableRow = line.includes('|');
      if (!isTableRow) {
        result.push(line);
        i += 1;
        continue;
      }

      // Collect consecutive lines that look like table rows
      const tableLines: string[] = [];
      while (i < lines.length && lines[i].includes('|')) {
        tableLines.push(lines[i]);
        i += 1;
      }

      if (tableLines.length < 2 || !tableLines[1].match(/^-{3,}|\|\s*-{3,}/)) {
        result.push(...tableLines);
        continue;
      }

      // Split into cells and compute column widths
      const rows = tableLines.map((l) => l.trim().replace(/^\||\|$/g, '').split('|').map((c) => c.trim()));
      const colCount = Math.max(...rows.map((r) => r.length));
      const widths = new Array(colCount).fill(0);
      rows.forEach((r) => {
        for (let c = 0; c < colCount; c++) {
          const cell = r[c] ?? '';
          widths[c] = Math.max(widths[c], cell.length);
        }
      });

      // Rebuild rows
      const formattedRows = rows.map((r, idx) => {
        const padded = widths.map((w, c) => {
          const cell = r[c] ?? '';
          return cell.padEnd(w, ' ');
        });
        return `| ${padded.join(' | ')} |`;
      });

      // Ensure separator row has dashes sized to column width
      if (formattedRows.length >= 2) {
        const sepCells = widths.map((w) => '-'.repeat(Math.max(3, w)));
        formattedRows[1] = `| ${sepCells.join(' | ')} |`;
      }

      result.push(...formattedRows);
    }

    return result.join('\n');
  }

  function saveEditorState(doc: Document | null) {
    if (!doc || !editorTextarea) return;
    editorStateByDoc.set(doc.id, {
      selectionStart: editorTextarea.selectionStart ?? 0,
      selectionEnd: editorTextarea.selectionEnd ?? 0,
      scrollTop: editorTextarea.scrollTop,
    });
    if (previewContainer) {
      previewScrollByDoc.set(doc.id, previewContainer.scrollTop);
    }
  }

  function restoreEditorState(doc: Document | null) {
    if (!doc || !editorTextarea) return;
    const state = editorStateByDoc.get(doc.id);
    const previewScroll = previewScrollByDoc.get(doc.id);
    if (!state && previewScroll === undefined) return;

    editorTextarea.selectionStart = state?.selectionStart ?? 0;
    editorTextarea.selectionEnd = state?.selectionEnd ?? 0;
    editorTextarea.scrollTop = state?.scrollTop ?? 0;
    if (previewContainer && previewScroll !== undefined) {
      previewContainer.scrollTop = previewScroll;
    }
    handleSelectionChange();
  }

  function announce(message: string) {
    liveMessage = '';
    tick().then(() => {
      liveMessage = message;
    });
  }

  function enhancePreviewBlocks() {
    if (!previewContainer) return;
    const codes = previewContainer.querySelectorAll('pre > code');

    codes.forEach((code) => {
      const pre = code.parentElement as HTMLElement | null;
      if (!pre || pre.dataset.enhanced === 'true') return;

      pre.dataset.enhanced = 'true';
      pre.classList.add('code-block');
      const lines = (code.textContent || '').split('\n').length;
      
      // Force vertical scroll propagation for Webviews where code blocks swallow wheel events
      pre.addEventListener('wheel', (e) => {
        if (Math.abs(e.deltaY) > Math.abs(e.deltaX)) {
          e.preventDefault();
          if (previewContainer) {
            previewContainer.scrollTop += e.deltaY;
          }
        }
      }, { passive: false });

      // Toolbar
      const toolbar = document.createElement('div');
      toolbar.className = 'code-toolbar';

      // Copy button
      const copyBtn = document.createElement('button');
      copyBtn.type = 'button';
      copyBtn.className = 'code-action copy';
      copyBtn.textContent = 'Copy';
      copyBtn.addEventListener('click', async () => {
        try {
          await navigator.clipboard.writeText(code.textContent || '');
          const prev = copyBtn.textContent;
          copyBtn.textContent = 'Copied';
          setTimeout(() => (copyBtn.textContent = prev), 1400);
        } catch (err) {
          console.error('Copy failed', err);
        }
      });
      toolbar.appendChild(copyBtn);



      pre.insertBefore(toolbar, code);
    });

    // Make task-list checkboxes interactive (sync back to markdown)
    const checkboxes = previewContainer.querySelectorAll<HTMLInputElement>('input[type="checkbox"][data-task-list-item]');
    checkboxes.forEach((checkbox) => {
      if (checkbox.dataset.bound === 'true') return;
      checkbox.dataset.bound = 'true';
      checkbox.addEventListener('change', () => {
        if (!currentDoc) return;
        const { selectionStart, selectionEnd, scrollTop } = editorTextarea ?? { selectionStart: 0, selectionEnd: 0, scrollTop: 0 };
        const lines = currentDoc.content.split('\n');
        const lineIndex = Array.from(previewContainer.querySelectorAll('input[type="checkbox"][data-task-list-item]')).indexOf(checkbox);
        if (lineIndex < 0) return;

        let taskCounter = -1;
        const updatedLines = lines.map((line) => {
          if (!line.match(/^\s*[-*]\s+\[( |x|X)\]/)) return line;
          taskCounter += 1;
          if (taskCounter !== lineIndex) return line;
          return line.replace(/^(\s*[-*]\s+\[)( |x|X)(\])/, `$1${checkbox.checked ? 'x' : ' '}$3`);
        });

        appStore.updateDocumentContent(currentDoc.id, updatedLines.join('\n'));
        tick().then(() => {
          if (!editorTextarea) return;
          editorTextarea.selectionStart = selectionStart ?? 0;
          editorTextarea.selectionEnd = selectionEnd ?? 0;
          editorTextarea.scrollTop = scrollTop ?? 0;
        });
      });
    });
  }
  
  // Apply theme to body
  $: {
    if (typeof document !== 'undefined') {
      if ($appStore.theme === 'dark') {
        document.body.classList.add('dark-mode');
      } else {
        document.body.classList.remove('dark-mode');
      }
    }
  }
  
  // ============================================
  // HANDLERS
  // ============================================
  
  function handleContentChange(e: Event) {
    const target = e.target as HTMLTextAreaElement;
    if (currentDoc) {
      appStore.updateDocumentContent(currentDoc.id, target.value);
    }
    handleSelectionChange();
  }
  
  function handleNewDocument() {
    if (!canCreateMoreTabs) {
      announce(`Tab limit reached (${MAX_TABS_PER_WORKSPACE})`);
      return;
    }
    const docCount = workspaceTabCount;
    const newName = docCount === 0 ? 'Untitled' : `Untitled ${docCount + 1}`;
    const newId = appStore.createDocument(newName, '');
    if (!newId) {
      announce(`Tab limit reached (${MAX_TABS_PER_WORKSPACE})`);
      return;
    }
    announce(`Document ${newName} created`);
    
    // Focus editor after creation
    tick().then(() => {
      editorTextarea?.focus();
    });
  }
  
  function handleCloseDocument(e: MouseEvent, id: string) {
    e.stopPropagation();
    
    if ($appStore.documents.length === 1) {
      // Don't delete the last document, just clear it
      appStore.updateDocumentContent(id, '');
      appStore.renameDocument(id, 'Untitled');
    } else {
      appStore.deleteDocument(id);
    }
  }
  
  function handleTabClick(id: string) {
    saveEditorState(currentDoc);
    appStore.setActiveDocument(id);
  }
  
  function startEditingName(doc: Document) {
    editingNameId = doc.id;
    editingNameValue = doc.name;
    
    tick().then(() => {
      const input = document.getElementById(`rename-input-${doc.id}`) as HTMLInputElement;
      input?.focus();
      input?.select();
    });
  }
  
  function finishEditingName() {
    if (editingNameId && editingNameValue.trim()) {
      appStore.renameDocument(editingNameId, editingNameValue.trim());
    }
    editingNameId = null;
    editingNameValue = '';
  }
  
  function handleRenameKeydown(e: KeyboardEvent) {
    if (e.key === 'Enter') {
      finishEditingName();
    } else if (e.key === 'Escape') {
      editingNameId = null;
      editingNameValue = '';
    }
  }
  
  function handleDownload() {
    if (!currentDoc) return;
    
    const blob = new Blob([currentDoc.content], { type: 'text/markdown' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = `${currentDoc.name.replace(/[^a-z0-9]/gi, '_').toLowerCase()}.md`;
    document.body.appendChild(a);
    a.click();
    document.body.removeChild(a);
    URL.revokeObjectURL(url);
    announce(`Document ${currentDoc.name} downloaded`);
  }

  function buildExportHtml(content: string, title: string) {
    const previewHtml = marked.parse(content) as string;
    return `<!doctype html><html><head><meta charset="utf-8"/><meta name="viewport" content="width=device-width,initial-scale=1"/><title>${title}</title><style>body{font-family:Inter,system-ui,sans-serif;margin:2rem;color:#111}main{max-width:860px;margin:0 auto}pre{background:#f4f4f4;padding:1rem;border-radius:8px;overflow:auto}code{font-family:ui-monospace,SFMono-Regular,Menlo,monospace}table{border-collapse:collapse;width:100%}th,td{border:1px solid #ddd;padding:.5rem}.callout{border-left:4px solid #4f46e5;padding:.75rem 1rem;background:#f8f8ff}@media print{body{margin:0.5in}}</style></head><body><main>${previewHtml}</main></body></html>`;
  }

  function handleExportHtml() {
    if (!currentDoc) return;
    const html = buildExportHtml(currentDoc.content, currentDoc.name);
    const blob = new Blob([html], { type: 'text/html' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = `${currentDoc.name.replace(/[^a-z0-9]/gi, '_').toLowerCase()}.html`;
    document.body.appendChild(a);
    a.click();
    document.body.removeChild(a);
    URL.revokeObjectURL(url);
    announce('HTML exported');
  }

  function handleExportPdf() {
    if (!currentDoc) return;
    const html = buildExportHtml(currentDoc.content, currentDoc.name);
    const printWindow = window.open('', '_blank');
    if (!printWindow) return;
    printWindow.document.write(html);
    printWindow.document.close();
    printWindow.focus();
    setTimeout(() => {
      printWindow.print();
    }, 200);
    announce('PDF export opened');
  }

  function handleReplaceAll() {
    if (!searchQuery.trim()) return;
    const regex = new RegExp(escapeRegExp(searchQuery), 'gi');
    for (const doc of $appStore.documents) {
      if (!doc.content.match(regex)) continue;
      appStore.updateDocumentContent(doc.id, doc.content.replace(regex, replaceQuery));
    }
    announce('Replace applied across documents');
  }

  function jumpToSearchResult(docId: string) {
    const target = $appStore.documents.find((doc) => doc.id === docId);
    if (!target) return;
    appStore.setActiveDocument(target.id);
    showSearchPanel = false;
  }

  function wrapSelection(prefix: string, suffix = prefix, placeholder = 'text') {
    if (!currentDoc || !editorTextarea) return;
    const start = editorTextarea.selectionStart ?? 0;
    const end = editorTextarea.selectionEnd ?? 0;
    const selected = currentDoc.content.slice(start, end) || placeholder;
    const updated = `${currentDoc.content.slice(0, start)}${prefix}${selected}${suffix}${currentDoc.content.slice(end)}`;
    appStore.updateDocumentContent(currentDoc.id, updated);
    tick().then(() => {
      const newStart = start + prefix.length;
      const newEnd = newStart + selected.length;
      editorTextarea.focus();
      editorTextarea.selectionStart = newStart;
      editorTextarea.selectionEnd = newEnd;
      handleSelectionChange();
    });
  }

  function insertAtCursor(snippet: string) {
    if (!currentDoc || !editorTextarea) return;
    const start = editorTextarea.selectionStart ?? 0;
    const end = editorTextarea.selectionEnd ?? 0;
    const updated = `${currentDoc.content.slice(0, start)}${snippet}${currentDoc.content.slice(end)}`;
    appStore.updateDocumentContent(currentDoc.id, updated);
    tick().then(() => {
      const caret = start + snippet.length;
      editorTextarea.focus();
      editorTextarea.selectionStart = caret;
      editorTextarea.selectionEnd = caret;
      handleSelectionChange();
    });
  }

  function replaceSelection(snippet: string) {
    if (!currentDoc || !editorTextarea) return;
    const start = editorTextarea.selectionStart ?? 0;
    const end = editorTextarea.selectionEnd ?? 0;
    const updated = `${currentDoc.content.slice(0, start)}${snippet}${currentDoc.content.slice(end)}`;
    appStore.updateDocumentContent(currentDoc.id, updated);
    tick().then(() => {
      const caret = start + snippet.length;
      editorTextarea.focus();
      editorTextarea.selectionStart = caret;
      editorTextarea.selectionEnd = caret;
      handleSelectionChange();
    });
  }

  function applyPrefixToSelectedLines(prefix: string) {
    if (!currentDoc || !editorTextarea) return;
    const start = editorTextarea.selectionStart ?? 0;
    const end = editorTextarea.selectionEnd ?? 0;
    const text = currentDoc.content;
    const lineStart = text.lastIndexOf('\n', Math.max(0, start - 1)) + 1;
    const nextNewline = text.indexOf('\n', end);
    const lineEnd = nextNewline === -1 ? text.length : nextNewline;
    const block = text.slice(lineStart, lineEnd);
    const updatedBlock = block
      .split('\n')
      .map((line) => `${prefix}${line}`)
      .join('\n');
    const updated = `${text.slice(0, lineStart)}${updatedBlock}${text.slice(lineEnd)}`;
    appStore.updateDocumentContent(currentDoc.id, updated);
    tick().then(() => {
      editorTextarea.focus();
      editorTextarea.selectionStart = lineStart;
      editorTextarea.selectionEnd = lineStart + updatedBlock.length;
      handleSelectionChange();
    });
  }

  function applyOrderedList() {
    if (!currentDoc || !editorTextarea) return;
    const start = editorTextarea.selectionStart ?? 0;
    const end = editorTextarea.selectionEnd ?? 0;
    const text = currentDoc.content;
    const lineStart = text.lastIndexOf('\n', Math.max(0, start - 1)) + 1;
    const nextNewline = text.indexOf('\n', end);
    const lineEnd = nextNewline === -1 ? text.length : nextNewline;
    const block = text.slice(lineStart, lineEnd);
    const updatedBlock = block
      .split('\n')
      .map((line, idx) => `${idx + 1}. ${line}`)
      .join('\n');
    const updated = `${text.slice(0, lineStart)}${updatedBlock}${text.slice(lineEnd)}`;
    appStore.updateDocumentContent(currentDoc.id, updated);
    tick().then(() => {
      editorTextarea.focus();
      editorTextarea.selectionStart = lineStart;
      editorTextarea.selectionEnd = lineStart + updatedBlock.length;
      handleSelectionChange();
    });
  }

  function insertHeading(level: 1 | 2 | 3) {
    const prefix = '#'.repeat(level) + ' ';
    applyPrefixToSelectedLines(prefix);
  }

  function insertHorizontalRule() {
    insertAtCursor('\n---\n');
  }

  function insertCodeBlock() {
    if (!currentDoc || !editorTextarea) return;
    const start = editorTextarea.selectionStart ?? 0;
    const end = editorTextarea.selectionEnd ?? 0;
    const selected = currentDoc.content.slice(start, end).trim();
    const snippet = selected ? `\n\`\`\`text\n${selected}\n\`\`\`\n` : '\n```text\ncode\n```\n';
    replaceSelection(snippet);
  }

  function insertInlineCode() {
    wrapSelection('`', '`', 'code');
  }

  function normalizeUrl(url: string) {
    const trimmed = url.trim();
    if (!trimmed) return '';
    if (/^https?:\/\//i.test(trimmed)) return trimmed;
    return `https://${trimmed}`;
  }

  function isLikelyUrl(value: string) {
    return /^(https?:\/\/|www\.)\S+$/i.test(value.trim());
  }

  function insertImageSnippet() {
    const selected = currentDoc && editorTextarea
      ? currentDoc.content.slice(editorTextarea.selectionStart ?? 0, editorTextarea.selectionEnd ?? 0).trim()
      : '';
    imageModalAlt = selected || '';
    imageModalUrl = '';
    const wasOpen = showImageModal;
    closeAllToolbarMenus();
    showImageModal = !wasOpen;
  }

  function confirmImageInsert() {
    if (!imageModalUrl.trim()) return;
    replaceSelection(`![${imageModalAlt.trim() || 'image'}](${normalizeUrl(imageModalUrl)})`);
    showImageModal = false;
  }

  function insertLinkSnippet() {
    const selected = currentDoc && editorTextarea
      ? currentDoc.content.slice(editorTextarea.selectionStart ?? 0, editorTextarea.selectionEnd ?? 0).trim()
      : '';
    linkModalText = selected || '';
    linkModalUrl = '';
    const wasOpen = showLinkModal;
    closeAllToolbarMenus();
    showLinkModal = !wasOpen;
  }

  function confirmLinkInsert() {
    if (!linkModalUrl.trim()) return;
    replaceSelection(`[${linkModalText.trim() || 'link'}](${normalizeUrl(linkModalUrl)})`);
    showLinkModal = false;
  }

  function formatUrlAction() {
    const selected = currentDoc && editorTextarea
      ? currentDoc.content.slice(editorTextarea.selectionStart ?? 0, editorTextarea.selectionEnd ?? 0).trim()
      : '';
    linkModalText = isLikelyUrl(selected) ? '' : (selected || '');
    linkModalUrl = isLikelyUrl(selected) ? selected : '';
    const wasOpen = showLinkModal;
    closeAllToolbarMenus();
    showLinkModal = !wasOpen;
  }

  function buildTable(rows: number, cols: number) {
    const header = `| ${Array.from({ length: cols }, (_, idx) => `Col ${idx + 1}`).join(' | ')} |`;
    const separator = `| ${Array.from({ length: cols }, () => '---').join(' | ')} |`;
    const body = Array.from({ length: Math.max(0, rows - 1) }, () => `| ${Array.from({ length: cols }, () => ' ').join(' | ')} |`);
    return [header, separator, ...body].join('\n');
  }

  function insertTable(rows: number, cols: number) {
    // Restore cursor position that was saved when the modal opened
    if (editorTextarea) {
      editorTextarea.focus();
      editorTextarea.selectionStart = savedTableSelStart;
      editorTextarea.selectionEnd = savedTableSelEnd;
    }
    const snippet = `\n${buildTable(rows, cols)}\n`;
    replaceSelection(snippet);
    showTableMenu = false;
    tableHoverRows = 0;
    tableHoverCols = 0;
  }

  function closeAllToolbarMenus() {
    showTableMenu = false;
    showEmojiMenu = false;
    showToolbarOverflowMenu = false;
    showImportMenu = false;
    showExportMenu = false;
    showLinkModal = false;
    showImageModal = false;
    tableHoverRows = 0;
    tableHoverCols = 0;
  }

  function openTableMenu() {
    // Save cursor before textarea loses focus
    savedTableSelStart = editorTextarea?.selectionStart ?? 0;
    savedTableSelEnd = editorTextarea?.selectionEnd ?? 0;
    const wasOpen = showTableMenu;
    closeAllToolbarMenus();
    showTableMenu = !wasOpen;
  }

  function openEmojiMenu() {
    // Save cursor position now — editor still has focus at this point
    savedEmojiSelStart = editorTextarea?.selectionStart ?? 0;
    savedEmojiSelEnd = editorTextarea?.selectionEnd ?? 0;
    const wasOpen = showEmojiMenu;
    closeAllToolbarMenus();
    showEmojiMenu = !wasOpen;
  }

  function toggleToolbarOverflow() {
    const wasOpen = showToolbarOverflowMenu;
    closeAllToolbarMenus();
    showToolbarOverflowMenu = !wasOpen;
  }

  function toggleImportMenu() {
    const wasOpen = showImportMenu;
    closeAllToolbarMenus();
    showImportMenu = !wasOpen;
  }

  function toggleExportMenu() {
    const wasOpen = showExportMenu;
    closeAllToolbarMenus();
    showExportMenu = !wasOpen;
  }

  function toggleTemplateMenu() {
    showTemplateMenu = !showTemplateMenu;
  }

  function applyTemplate(template: Template) {
    if (!canCreateMoreTabs) {
      announce(`Tab limit reached (${MAX_TABS_PER_WORKSPACE})`);
      return;
    }
    const newId = appStore.createDocument(template.name, template.content);
    if (!newId) {
      announce(`Tab limit reached (${MAX_TABS_PER_WORKSPACE})`);
      return;
    }
    showTemplateMenu = false;
    announce(`Template "${template.name}" created`);
    tick().then(() => {
      editorTextarea?.focus();
    });
  }

  function handleEmojiPicked(event: CustomEvent<string>) {
    const emoji = event.detail;
    if (!emoji) return;
    showEmojiMenu = false;
    // Restore saved cursor position (textarea lost focus when picker opened)
    if (editorTextarea) {
      editorTextarea.focus();
      editorTextarea.selectionStart = savedEmojiSelStart;
      editorTextarea.selectionEnd = savedEmojiSelEnd;
    }
    replaceSelection(emoji);
  }

  // ============================================
  // LIVE MODE
  // ============================================

  function splitIntoLiveBlocks(content: string): string[] {
    const lines = content.split('\n');
    const blocks: string[] = [];
    let current: string[] = [];
    let inCode = false;
    for (const line of lines) {
      if (line.trimStart().startsWith('```')) inCode = !inCode;
      if (!inCode && line.trim() === '' && current.length > 0) {
        blocks.push(current.join('\n'));
        current = [];
      } else {
        current.push(line);
      }
    }
    if (current.length > 0) blocks.push(current.join('\n'));
    return blocks.length > 0 ? blocks : [''];
  }

  $: if (currentDoc && liveActiveBlock === null) {
    liveBlocks = splitIntoLiveBlocks(currentDoc.content);
  }

  function activateLiveBlock(idx: number) {
    if (!currentDoc) return;
    liveActiveBlock = idx;
    liveBlockDraft = liveBlocks[idx] ?? '';
    tick().then(() => {
      if (liveBlockTextarea) {
        liveBlockTextarea.focus();
        autoResizeLiveTextarea(liveBlockTextarea);
      }
    });
  }

  function commitLiveBlock() {
    if (liveActiveBlock === null || !currentDoc) return;
    const draft = liveBlockDraft;
    const subBlocks = draft.split(/\n{2,}/);
    const newBlocks = [...liveBlocks];
    newBlocks.splice(liveActiveBlock, 1, ...subBlocks);
    liveBlocks = newBlocks.filter((b) => b.trim() !== '' || newBlocks.length === 1);
    if (liveBlocks.length === 0) liveBlocks = [''];
    liveActiveBlock = null;
    liveBlockDraft = '';
    appStore.updateDocumentContent(currentDoc.id, liveBlocks.join('\n\n'));
  }

  function handleLiveBlockKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') {
      e.preventDefault();
      if (liveBlockTextarea) liveBlockTextarea.blur();
    }
  }

  function autoResizeLiveTextarea(el: HTMLTextAreaElement) {
    el.style.height = 'auto';
    el.style.height = `${el.scrollHeight}px`;
  }

  function addLiveBlock() {
    if (!currentDoc) return;
    liveBlocks = [...liveBlocks, ''];
    tick().then(() => activateLiveBlock(liveBlocks.length - 1));
  }

  function handleImportFiles(e: Event) {
    const input = e.target as HTMLInputElement | null;
    const files = Array.from(input?.files || []);
    if (files.length === 0) return;
    const availableSlots = Math.max(0, MAX_TABS_PER_WORKSPACE - workspaceTabCount);
    if (availableSlots === 0) {
      if (input) input.value = '';
      announce(`Tab limit reached (${MAX_TABS_PER_WORKSPACE})`);
      return;
    }
    files.slice(0, availableSlots).forEach((file) => {
      const reader = new FileReader();
      reader.onload = () => {
        appStore.createDocument(file.name.replace(/\.md$/i, ''), String(reader.result || ''));
      };
      reader.readAsText(file);
    });
    if (input) input.value = '';
    announce(`${Math.min(files.length, availableSlots)} file(s) queued for import`);
  }

  function handleDropImport(e: DragEvent) {
    e.preventDefault();
    isEditorDragOver = false;
    const files = Array.from(e.dataTransfer?.files || []).filter((file) => file.name.toLowerCase().endsWith('.md'));
    const availableSlots = Math.max(0, MAX_TABS_PER_WORKSPACE - workspaceTabCount);
    if (availableSlots === 0) {
      announce(`Tab limit reached (${MAX_TABS_PER_WORKSPACE})`);
      return;
    }
    files.slice(0, availableSlots).forEach((file) => {
      const reader = new FileReader();
      reader.onload = () => {
        appStore.createDocument(file.name.replace(/\.md$/i, ''), String(reader.result || ''));
      };
      reader.readAsText(file);
    });
    if (files.length > 0) announce(`${Math.min(files.length, availableSlots)} markdown file(s) imported`);
  }

  function zoomIn() {
    uiZoom = Math.min(170, uiZoom + 10);
    announce(`Zoom ${uiZoom}%`);
  }

  function zoomOut() {
    uiZoom = Math.max(70, uiZoom - 10);
    announce(`Zoom ${uiZoom}%`);
  }

  function resetZoom() {
    uiZoom = 100;
    announce('Zoom 100%');
  }
  
  // Sync scroll between editor and line numbers
  function handleEditorScroll() {
    if (lineNumbersEl && editorTextarea) {
      lineNumbersEl.scrollTop = editorTextarea.scrollTop;
    }
  }

  function handleSelectionChange() {
    if (!editorTextarea) return;
    const { selectionStart = 0, selectionEnd = 0 } = editorTextarea;
    selectionLength = Math.abs(selectionEnd - selectionStart);
    const pos = selectionStart;
    const text = currentDoc?.content ?? '';
    const before = text.slice(0, pos);
    const lines = before.split('\n');
    caretLine = lines.length;
    caretCol = lines[lines.length - 1]?.length + 1 || 1;
  }

  function handleEditorKeydown(e: KeyboardEvent) {
    if (!currentDoc || !editorTextarea) return;
    if (e.key !== 'Tab') return;
    e.preventDefault();

    const start = editorTextarea.selectionStart ?? 0;
    const end = editorTextarea.selectionEnd ?? 0;
    const text = currentDoc.content;
    const lineStart = text.lastIndexOf('\n', Math.max(0, start - 1)) + 1;
    const nextNewline = text.indexOf('\n', end);
    const lineEnd = nextNewline === -1 ? text.length : nextNewline;

    // Single caret: insert tab / remove one indent level.
    if (start === end) {
      if (e.shiftKey) {
        const beforeLine = text.slice(lineStart, start);
        if (beforeLine.endsWith('\t')) {
          const updated = `${text.slice(0, start - 1)}${text.slice(end)}`;
          appStore.updateDocumentContent(currentDoc.id, updated);
          tick().then(() => {
            const pos = start - 1;
            editorTextarea.selectionStart = pos;
            editorTextarea.selectionEnd = pos;
            handleSelectionChange();
          });
          return;
        }
        if (beforeLine.endsWith('  ')) {
          const updated = `${text.slice(0, start - 2)}${text.slice(end)}`;
          appStore.updateDocumentContent(currentDoc.id, updated);
          tick().then(() => {
            const pos = start - 2;
            editorTextarea.selectionStart = pos;
            editorTextarea.selectionEnd = pos;
            handleSelectionChange();
          });
          return;
        }
        return;
      }
      const updated = `${text.slice(0, start)}\t${text.slice(end)}`;
      appStore.updateDocumentContent(currentDoc.id, updated);
      tick().then(() => {
        const pos = start + 1;
        editorTextarea.selectionStart = pos;
        editorTextarea.selectionEnd = pos;
        handleSelectionChange();
      });
      return;
    }

    // Multi-line selection: indent/outdent all selected lines.
    const block = text.slice(lineStart, lineEnd);
    const lines = block.split('\n');
    const updatedLines = e.shiftKey
      ? lines.map((line) => line.replace(/^\t|^ {1,2}/, ''))
      : lines.map((line) => `\t${line}`);
    const updatedBlock = updatedLines.join('\n');
    const updated = `${text.slice(0, lineStart)}${updatedBlock}${text.slice(lineEnd)}`;
    appStore.updateDocumentContent(currentDoc.id, updated);
    tick().then(() => {
      editorTextarea.selectionStart = lineStart;
      editorTextarea.selectionEnd = lineStart + updatedBlock.length;
      handleSelectionChange();
    });
  }

  function handleSearchKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') {
      e.preventDefault();
      showSearchPanel = false;
      return;
    }
    if (e.key === 'Enter') {
      e.preventDefault();
      const first = searchResults[0];
      if (first) jumpToSearchResult(first.id);
    }
  }
  
  // Resizer handlers
  function startResizing(e: MouseEvent) {
    isResizing = true;
    document.body.style.cursor = 'col-resize';
    document.body.style.userSelect = 'none';
  }
  
  function stopResizing() {
    isResizing = false;
    document.body.style.cursor = '';
    document.body.style.userSelect = '';
  }
  
  function handleResizing(e: MouseEvent) {
    const containerWidth = window.innerWidth;
    if (isResizing) {
      const newWidth = (e.clientX / containerWidth) * 100;
      if (newWidth >= 20 && newWidth <= 80) {
        sidebarWidth = newWidth;
        appStore.setSidebarWidth(newWidth);
      }
      return;
    }
  }

  function handleGlobalClick(e: MouseEvent) {
    const target = e.target as Node | null;
    if (!target) return;
    // Toolbar menus
    if ((showTableMenu || showEmojiMenu || showToolbarOverflowMenu || showImportMenu || showExportMenu) &&
        toolbarHostEl && !toolbarHostEl.contains(target)) {
      showTableMenu = false;
      showEmojiMenu = false;
      showToolbarOverflowMenu = false;
      showImportMenu = false;
      showExportMenu = false;
      tableHoverRows = 0;
      tableHoverCols = 0;
    }
    if (showTemplateMenu) {
      const templatePanel = document.querySelector('.template-float-panel');
      const templateBtn = document.querySelector('.template-tab-btn');
      if (
        templatePanel && !templatePanel.contains(target) &&
        (!templateBtn || !templateBtn.contains(target))
      ) {
        showTemplateMenu = false;
      }
    }
  }
  
  // Toggle fullscreen mode
  function toggleFullscreen() {
    if (!document.fullscreenElement) {
      document.documentElement.requestFullscreen().then(() => {
        isFullscreen = true;
      }).catch((err) => {
        console.error('Error attempting to enable fullscreen:', err);
      });
    } else {
      document.exitFullscreen().then(() => {
        isFullscreen = false;
      }).catch((err) => {
        console.error('Error attempting to exit fullscreen:', err);
      });
    }
  }
  
  // Keyboard shortcuts
  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') {
      if (showTableMenu || showEmojiMenu || showToolbarOverflowMenu || showImportMenu || showExportMenu ||
          showTemplateMenu || showLinkModal || showImageModal) {
        closeAllToolbarMenus();
        showTemplateMenu = false;
        e.preventDefault();
        return;
      }
      if (showSearchPanel) {
        showSearchPanel = false;
        e.preventDefault();
        return;
      }
    }

    // Ctrl/Cmd + N: New document
    if ((e.ctrlKey || e.metaKey) && e.key === 'n') {
      e.preventDefault();
      handleNewDocument();
    }

    // Ctrl/Cmd + Z: Undo
    if ((e.ctrlKey || e.metaKey) && !e.shiftKey && e.key.toLowerCase() === 'z') {
      e.preventDefault();
      handleUndo();
    }

    // Ctrl/Cmd + Shift + Z: Redo
    if ((e.ctrlKey || e.metaKey) && e.shiftKey && e.key.toLowerCase() === 'z') {
      e.preventDefault();
      handleRedo();
    }
    
    // Ctrl/Cmd + S: Download (since it's auto-saved)
    if ((e.ctrlKey || e.metaKey) && e.key === 's') {
      e.preventDefault();
      handleDownload();
    }
    
    // Ctrl/Cmd + Shift + L: Toggle theme
    if ((e.ctrlKey || e.metaKey) && e.shiftKey && e.key.toLowerCase() === 'l') {
      e.preventDefault();
      appStore.toggleTheme();
    }
    
    // Alt + Z: Toggle word wrap (avoids browser Ctrl+W close-tab)
    if (e.altKey && e.key.toLowerCase() === 'z') {
      e.preventDefault();
      appStore.toggleWordWrap();
    }

    // Ctrl/Cmd + K: Command palette
    if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === 'k') {
      e.preventDefault();
      if (showPalette) {
        closePalette();
      } else {
        openPalette();
      }
    }

    if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === 'f') {
      e.preventDefault();
      showSearchPanel = !showSearchPanel;
    }

    if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === 'b') {
      e.preventDefault();
      wrapSelection('**');
    }

    if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === 'i') {
      e.preventDefault();
      wrapSelection('*');
    }

    if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === 'u') {
      e.preventDefault();
      wrapSelection('<u>', '</u>', 'text');
    }

    if ((e.ctrlKey || e.metaKey) && e.shiftKey && e.key.toLowerCase() === 'x') {
      e.preventDefault();
      wrapSelection('~~', '~~', 'text');
    }

    if ((e.ctrlKey || e.metaKey) && e.shiftKey && e.key === '7') {
      e.preventDefault();
      applyOrderedList();
    }

    if ((e.ctrlKey || e.metaKey) && e.shiftKey && e.key === '8') {
      e.preventDefault();
      applyPrefixToSelectedLines('- ');
    }

    if ((e.ctrlKey || e.metaKey) && e.altKey && e.key === '1') {
      e.preventDefault();
      insertHeading(1);
    }

    if ((e.ctrlKey || e.metaKey) && e.altKey && e.key === '2') {
      e.preventDefault();
      insertHeading(2);
    }

    if ((e.ctrlKey || e.metaKey) && e.altKey && e.key === '3') {
      e.preventDefault();
      insertHeading(3);
    }

    if ((e.ctrlKey || e.metaKey) && (e.key === '=' || e.key === '+')) {
      e.preventDefault();
      zoomIn();
    }

    if ((e.ctrlKey || e.metaKey) && e.key === '-') {
      e.preventDefault();
      zoomOut();
    }

    if ((e.ctrlKey || e.metaKey) && e.key === '0') {
      e.preventDefault();
      resetZoom();
    }
  }
  
  onMount(() => {
    document.title = 'FireNotes — Write';
    appStore.loadDocuments();
    showMobileDevNotice = window.matchMedia('(max-width: 900px)').matches;
    window.addEventListener('mousemove', handleResizing);
    window.addEventListener('mouseup', stopResizing);
    window.addEventListener('mousedown', handleGlobalClick);
    window.addEventListener('keydown', handleKeydown);
    
    // Listen for fullscreen changes
    const handleFullscreenChange = () => {
      isFullscreen = !!document.fullscreenElement;
    };
    document.addEventListener('fullscreenchange', handleFullscreenChange);
    
    return () => {
      window.removeEventListener('mousemove', handleResizing);
      window.removeEventListener('mouseup', stopResizing);
      window.removeEventListener('mousedown', handleGlobalClick);
      window.removeEventListener('keydown', handleKeydown);
      document.removeEventListener('fullscreenchange', handleFullscreenChange);
    };
  });
</script>

<main class="app-container">
  <div class="sr-only" aria-live="polite">{liveMessage}</div>
  <!-- Header / Tab Bar -->
  <header class="header">
    <div class="app-brand">FireNotes</div>
    <div class="tabs-container">
      <div class="tabs-scroll">
        {#each visibleDocuments as doc (doc.id)}
          <div 
            class="tab-wrapper animate-slide-in"
            class:active={doc.id === $appStore.activeDocumentId}
          >
            {#if editingNameId === doc.id}
              <input
                id="rename-input-{doc.id}"
                type="text"
                class="rename-input"
                bind:value={editingNameValue}
                on:blur={finishEditingName}
                on:keydown={handleRenameKeydown}
              />
            {:else}
              <button 
                class="tab animate-fade-in"
                class:active={doc.id === $appStore.activeDocumentId}
                on:click={() => handleTabClick(doc.id)}
                on:dblclick={() => startEditingName(doc)}
                title={doc.name}
              >
                <FileText size={14} />
                <span class="tab-name">{doc.name}</span>
                <button 
                  class="tab-close"
                  on:click={(e) => handleCloseDocument(e, doc.id)}
                  title="Close tab"
                >
                  <X size={12} />
                </button>
              </button>
            {/if}
          </div>
        {/each}
        
        <button
          class="add-tab-btn template-tab-btn animate-fade-in"
          class:is-active={showTemplateMenu}
          on:click={toggleTemplateMenu}
          title="Templates"
        >
          <LayoutTemplate size={15} />
        </button>
        <button
          class="add-tab-btn animate-fade-in"
          class:is-disabled={!canCreateMoreTabs}
          on:click={handleNewDocument}
          title={canCreateMoreTabs ? 'New document (Ctrl+N)' : `Maximum ${MAX_TABS_PER_WORKSPACE} tabs`}
          disabled={!canCreateMoreTabs}
        >
          <Plus size={16} />
        </button>
      </div>
    </div>
  </header>

  {#if showMobileDevNotice}
    <div class="mobile-dev-backdrop">
      <div class="mobile-dev-modal">
        <h3>Mobile Support In Progress</h3>
        <p>The mobile experience is still being developed and may have layout issues.</p>
        <button class="btn btn-primary" on:click={() => (showMobileDevNotice = false)}>Continue Anyway</button>
      </div>
    </div>
  {/if}

  {#if currentDoc}
    <div class="toolbar-host" bind:this={toolbarHostEl}>
      <div class="format-toolbar unified-toolbar">
        <div class="toolbar-main desktop-toolbar">
          <!-- Left side: Formatting controls -->
          <div class="toolbar-left">
            <div class="toolbar-group">
              <button class="toolbar-icon-btn" class:disabled={!undoBtnActive} on:click={handleUndo} title="Undo (Ctrl+Z)"><Undo2 size={16} /></button>
              <button class="toolbar-icon-btn" class:disabled={!redoBtnActive} on:click={handleRedo} title="Redo (Ctrl+Shift+Z)"><Redo2 size={16} /></button>
            </div>
            <span class="toolbar-divider" aria-hidden="true"></span>
            <div class="toolbar-group">
              <button class="toolbar-icon-btn" on:click={() => wrapSelection('**')} title="Bold"><Bold size={16} /></button>
              <button class="toolbar-icon-btn" on:click={() => wrapSelection('*')} title="Italic"><Italic size={16} /></button>
              <button class="toolbar-icon-btn" on:click={() => wrapSelection('~~', '~~', 'text')} title="Strikethrough"><Strikethrough size={16} /></button>
              <button class="toolbar-icon-btn" on:click={() => wrapSelection('<u>', '</u>', 'text')} title="Underline"><Underline size={16} /></button>
            </div>
            <span class="toolbar-divider" aria-hidden="true"></span>

            <div class="toolbar-group">
              <button class="toolbar-icon-btn" on:click={() => insertHeading(1)} title="Header 1"><Heading1 size={16} /></button>
              <button class="toolbar-icon-btn" on:click={() => insertHeading(2)} title="Header 2"><Heading2 size={16} /></button>
              <button class="toolbar-icon-btn" on:click={() => insertHeading(3)} title="Header 3"><Heading3 size={16} /></button>
            </div>
            <span class="toolbar-divider" aria-hidden="true"></span>
            <div class="toolbar-group">
              <button class="toolbar-icon-btn" on:click={() => wrapSelection('<sub>', '</sub>', 'sub')} title="Subtext"><Subscript size={16} /></button>
              <button class="toolbar-icon-btn" on:click={() => wrapSelection('<sup>', '</sup>', 'sup')} title="Supertext"><Superscript size={16} /></button>
            </div>
            <span class="toolbar-divider" aria-hidden="true"></span>

            <div class="toolbar-group">
              <button class="toolbar-icon-btn" class:active-toolbar={showTableMenu} on:click={openTableMenu} title="Create table"><Table2 size={16} /></button>
              <button class="toolbar-icon-btn" on:click={insertImageSnippet} title="Insert image"><ImagePlus size={16} /></button>
              <button class="toolbar-icon-btn" on:click={insertLinkSnippet} title="Insert link"><Link2 size={16} /></button>
            </div>
            <span class="toolbar-divider" aria-hidden="true"></span>

            <div class="toolbar-group">
              <button class="toolbar-icon-btn" on:click={applyOrderedList} title="Ordered list"><ListOrdered size={16} /></button>
              <button class="toolbar-icon-btn" on:click={() => applyPrefixToSelectedLines('- ')} title="Unordered list"><List size={16} /></button>
              <button class="toolbar-icon-btn" on:click={() => applyPrefixToSelectedLines('- [ ] ')} title="Task list"><ListTodo size={16} /></button>
            </div>
            <span class="toolbar-divider" aria-hidden="true"></span>

            <div class="toolbar-group">
              <button class="toolbar-icon-btn" on:click={insertCodeBlock} title="Code block"><SquareCode size={16} /></button>
              <button class="toolbar-icon-btn" on:click={insertInlineCode} title="Inline code"><Code size={16} /></button>
            </div>
            <span class="toolbar-divider" aria-hidden="true"></span>
            <div class="toolbar-group">
              <button class="toolbar-icon-btn" on:click={() => applyPrefixToSelectedLines('> ')} title="Quote"><Quote size={16} /></button>
              <button class="toolbar-icon-btn" on:click={insertHorizontalRule} title="Horizontal rule"><Minus size={16} /></button>
            </div>
            <span class="toolbar-divider" aria-hidden="true"></span>

            <div class="toolbar-group">
              <button class="toolbar-icon-btn" class:active-toolbar={showEmojiMenu} on:click={openEmojiMenu} title="Insert emoji"><Smile size={16} /></button>
            </div>
          </div>

          <!-- Spacer -->
          <div class="toolbar-spacer"></div>

          <!-- Right side: View/Preview controls -->
          <div class="toolbar-right">
            <div class="toolbar-group">
              <button class="preview-icon-btn animate-fade-in" class:active-accent={$appStore.wordWrap} on:click={() => appStore.toggleWordWrap()} title="Toggle word wrap (Alt+Z)">
                <WrapText size={17} />
              </button>

            </div>
            <span class="toolbar-divider" aria-hidden="true"></span>
            <div class="toolbar-group">
              <button class="preview-icon-btn animate-fade-in" on:click={toggleImportMenu} title="Imports">
                <Upload size={17} />
              </button>
              <button class="preview-icon-btn animate-fade-in" on:click={toggleExportMenu} title="Exports">
                <Download size={17} />
              </button>
            </div>
            <span class="toolbar-divider" aria-hidden="true"></span>
            <div class="toolbar-group">
              <button class="preview-icon-btn animate-fade-in" class:active-accent={showToc} on:click={() => (showToc = !showToc)} title="Table of Contents">
                <ListCollapse size={17} />
              </button>
            </div>
            <span class="toolbar-divider" aria-hidden="true"></span>
            <div class="toolbar-group">
              <button class="preview-icon-btn animate-fade-in" on:click={() => appStore.toggleTheme()} title="Toggle theme (Ctrl+Shift+L)">
                {#if $appStore.theme === 'dark'}
                  <Sun size={17} />
                {:else}
                  <Moon size={17} />
                {/if}
              </button>
              <button class="preview-icon-btn animate-fade-in" on:click={toggleFullscreen} title="Toggle fullscreen">
                {#if isFullscreen}
                  <Minimize size={17} />
                {:else}
                  <Maximize size={17} />
                {/if}
              </button>
            </div>
          </div>
        </div>

        <div class="toolbar-main mobile-toolbar">
          <button class="toolbar-icon-btn" on:click={() => wrapSelection('**')} title="Bold"><Bold size={16} /></button>
          <button class="toolbar-icon-btn" on:click={() => wrapSelection('*')} title="Italic"><Italic size={16} /></button>
          <button class="toolbar-icon-btn" on:click={insertLinkSnippet} title="Insert link"><Link2 size={16} /></button>
          <button class="toolbar-icon-btn" on:click={insertCodeBlock} title="Code block"><SquareCode size={16} /></button>
          <button class="toolbar-icon-btn" class:active-toolbar={showToolbarOverflowMenu} on:click={toggleToolbarOverflow} title="More"><Ellipsis size={16} /></button>
        </div>

        <input bind:this={importInputEl} type="file" accept=".md,text/markdown" multiple hidden on:change={handleImportFiles} />
      </div>
    </div>
  {:else}
    <div class="toolbar-host empty-toolbars">
      <div class="empty-toolbar-state">
        <span>No document selected</span>
        <button class="btn btn-primary" on:click={handleNewDocument}>
          <Plus size={14} />
          Create document
        </button>
      </div>
    </div>
  {/if}
  
  <!-- Workspace -->
  <div class="workspace">
    <!-- Table of Contents Sidebar -->
    {#if showToc && tocEntries.length > 0}
      <aside class="toc-sidebar">
        <div class="toc-header">
          <span>Contents</span>
          <button class="status-icon-btn" on:click={() => (showToc = false)} title="Close">×</button>
        </div>
        <nav class="toc-list">
          {#each tocEntries as entry}
            <button
              class="toc-entry"
              class:toc-h1={entry.level === 1}
              class:toc-h2={entry.level === 2}
              class:toc-h3={entry.level >= 3}
              style="padding-left: {(entry.level - 1) * 12 + 8}px"
              on:click={() => scrollToHeading(entry)}
              title={entry.text}
            >
              {entry.text}
            </button>
          {/each}
        </nav>
      </aside>
    {/if}

    <!-- Editor Pane -->
    <!-- svelte-ignore a11y-no-static-element-interactions -->
    <section class="editor-pane" class:hidden-pane={viewMode === 'preview'} style="width: {viewMode === 'write' || viewMode === 'live' ? '100%' : `${sidebarWidth}%`}">
      {#if viewMode === 'live'}
        <!-- Live Editor -->
        <!-- svelte-ignore a11y-no-static-element-interactions -->
        <!-- svelte-ignore a11y-click-events-have-key-events -->
        <div class="live-editor" style="font-size: {uiZoom}%">
          {#each liveBlocks as block, i (i)}
            {#if liveActiveBlock === i}
              <!-- svelte-ignore a11y-autofocus -->
              <textarea
                bind:this={liveBlockTextarea}
                class="live-block-raw"
                bind:value={liveBlockDraft}
                on:blur={commitLiveBlock}
                on:keydown={handleLiveBlockKeydown}
                on:input={(e) => autoResizeLiveTextarea(e.currentTarget)}
                rows={1}
                spellcheck="false"
                placeholder="Write markdown..."
              ></textarea>
            {:else}
              <div
                class="live-block-rendered"
                class:live-block-empty={!block.trim()}
                on:click={() => activateLiveBlock(i)}
                role="button"
                tabindex={0}
                on:keydown={(e) => { if (e.key === 'Enter') activateLiveBlock(i); }}
              >
                {#if block.trim()}
                  {@html marked.parse(block)}
                {:else}
                  <span class="live-block-placeholder">Click to edit...</span>
                {/if}
              </div>
            {/if}
          {/each}
          <!-- Add new block area -->
          <div class="live-add-block" on:click={addLiveBlock} role="button" tabindex={0} on:keydown={(e) => { if (e.key === 'Enter') addLiveBlock(); }}>
            <span>+</span> New block
          </div>
        </div>
      {:else}
        <div class="editor-container">
          <!-- Line Numbers -->
          <div class="line-numbers" bind:this={lineNumbersEl} style="font-size: {Math.round((14 * uiZoom) / 100)}px">
            {#each Array(lineCount) as _, i}
              <div class="line-number">{i + 1}</div>
            {/each}
          </div>
          
          <!-- Editor -->
          <textarea
            bind:this={editorTextarea}
            class="editor-textarea"
            style="font-size: {Math.round((14 * uiZoom) / 100)}px"
            class:word-wrap-enabled={$appStore.wordWrap}
            aria-label="Markdown editor"
            value={currentDoc?.content || ''}
            on:input={handleContentChange}
            on:paste={handlePaste}
            on:scroll={handleEditorScroll}
            on:click={handleSelectionChange}
            on:keyup={handleSelectionChange}
            on:select={handleSelectionChange}
            on:keydown={handleEditorKeydown}
            on:dragover={(e) => {
              e.preventDefault();
              isEditorDragOver = true;
            }}
            on:dragleave={() => (isEditorDragOver = false)}
            on:drop={handleDropImport}
            class:drag-over={isEditorDragOver}
            placeholder="Start writing..."
            spellcheck="false"
          ></textarea>
        </div>
      {/if}
    </section>
    
    <!-- Resizer -->
    <!-- svelte-ignore a11y-no-static-element-interactions -->
    <div 
      class="resizer"
      class:hidden-pane={viewMode !== 'split'}
      class:active={isResizing}
      on:mousedown={startResizing}
    ></div>
    
    <!-- Preview Pane -->
    <section class="preview-pane" class:hidden-pane={viewMode === 'write' || viewMode === 'live'}>
      <div class="preview-container" bind:this={previewContainer} class:word-wrap-enabled={$appStore.wordWrap} class:word-wrap-disabled={!$appStore.wordWrap}>
        {#if currentDoc}
          <article class="markdown-preview animate-fade-up" style="font-size: {uiZoom}%">
            {@html renderedMarkdown}
          </article>
        {:else}
          <div class="empty-state">
            <div class="empty-icon">
              <FileText size={48} />
            </div>
            <p>No document selected</p>
            <button class="btn btn-primary" on:click={handleNewDocument}>
              <Plus size={16} />
              Create New Document
            </button>
          </div>
        {/if}
      </div>
    </section>

  </div>


  
  <!-- Status Bar -->
  <footer class="status-bar">
    <div class="status-left">
      <button class="status-icon-btn" on:click={() => (showSearchPanel = !showSearchPanel)} title="Global search">
        <Search size={14} />
      </button>
      <span class="status-pill subtle">
        {$appStore.documents.length} {$appStore.documents.length === 1 ? 'document' : 'documents'}
      </span>
      {#if currentDoc}
        <span class="status-pill">
          {wordCount.toLocaleString()} words
        </span>
        <span class="status-pill">
          {currentDoc.content.length.toLocaleString()} chars
        </span>
        <span class="status-pill">
          {lineCount.toLocaleString()} lines
        </span>
        <span class="status-pill">
          Line {caretLine}, Col {caretCol}
        </span>
        {#if selectionLength > 0}
          <span class="status-pill accent">{selectionLength} selected</span>
        {/if}
      {/if}
    </div>
    <div class="status-right">
      <div class="status-group">
        <button class="status-icon-btn" class:active-accent={viewMode === 'write'} on:click={() => viewMode = 'write'} title="Write Mode">
          <PenLine size={14} />
        </button>
        <button class="status-icon-btn" class:active-accent={viewMode === 'split'} on:click={() => viewMode = 'split'} title="Split Mode">
          <Columns size={14} />
        </button>
        <button class="status-icon-btn" class:active-accent={viewMode === 'preview'} on:click={() => viewMode = 'preview'} title="Preview Mode">
          <Eye size={14} />
        </button>
        <button class="status-icon-btn status-live-btn" class:active-accent={viewMode === 'live'} on:click={() => viewMode = 'live'} title="Live Mode">
          <Zap size={14} />
        </button>
      </div>
      <span class="status-divider"></span>
      <button class="status-icon-btn" on:click={zoomOut} title="Zoom out (Ctrl+-)">-</button>
      <button class="status-icon-readout" on:click={resetZoom} title="Reset zoom (Ctrl+0)">{uiZoom}%</button>
      <button class="status-icon-btn" on:click={zoomIn} title="Zoom in (Ctrl/Cmd +)">+</button>
    </div>
  </footer>
</main>

<CommandPalette
  show={showPalette}
  commands={paletteCommands}
  on:close={() => (showPalette = false)}
  on:run={(e) => { showPalette = false; e.detail.run(); }}
/>

<SearchPanel
  show={showSearchPanel}
  documents={$appStore.documents}
  on:close={() => (showSearchPanel = false)}
  on:jumpTo={(e) => { appStore.setActiveDocument(e.detail); showSearchPanel = false; }}
  on:replaceAll={(e) => {
    const regex = new RegExp(escapeRegExp(e.detail.query), 'gi');
    for (const doc of $appStore.documents) {
      if (!doc.content.match(regex)) continue;
      appStore.updateDocumentContent(doc.id, doc.content.replace(regex, e.detail.replacement));
    }
    announce('Replace applied across documents');
  }}
/>

{#if showImportMenu}
  <div class="palette-backdrop" role="presentation" on:click={() => (showImportMenu = false)}></div>
  <div class="floating-panel">
    <div class="search-panel-header">
      <h3>Import</h3>
      <button class="status-icon-btn" on:click={() => (showImportMenu = false)} title="Close">×</button>
    </div>
    <div class="search-list" style="margin-top: 8px;">
      <button class="palette-item" style="text-align: left;" on:click={() => { importInputEl?.click(); showImportMenu = false; }}>
        <div class="palette-title" style="display: flex; align-items: center; gap: 8px;">
          <Upload size={14} /> Import Markdown
        </div>
        <div class="palette-desc">Load .md files from your computer</div>
      </button>
    </div>
  </div>
{/if}

{#if showEmojiMenu}
  <div class="palette-backdrop" role="presentation" on:click={() => (showEmojiMenu = false)}></div>
  <div class="floating-panel emoji-float-panel">
    <div class="search-panel-header">
      <h3>Insert Emoji</h3>
      <button class="status-icon-btn" on:click={() => (showEmojiMenu = false)} title="Close">×</button>
    </div>
    <EmojiPicker on:pick={handleEmojiPicked} />
  </div>
{/if}

{#if showExportMenu}
  <div class="palette-backdrop" role="presentation" on:click={() => (showExportMenu = false)}></div>
  <div class="floating-panel">
    <div class="search-panel-header">
      <h3>Export</h3>
      <button class="status-icon-btn" on:click={() => (showExportMenu = false)} title="Close">×</button>
    </div>
    <div class="search-list" style="margin-top: 8px;">
      <button class="palette-item" style="text-align: left;" on:click={() => { handleDownload(); showExportMenu = false; }}>
        <div class="palette-title" style="display: flex; align-items: center; gap: 8px;">
          <Download size={14} /> Download .md
        </div>
        <div class="palette-desc">Save the raw markdown file</div>
      </button>
      <button class="palette-item" style="text-align: left;" on:click={() => { handleExportHtml(); showExportMenu = false; }}>
        <div class="palette-title" style="display: flex; align-items: center; gap: 8px;">
          <FileDown size={14} /> Export HTML
        </div>
        <div class="palette-desc">Save rendered document as standalone HTML</div>
      </button>
      <button class="palette-item" style="text-align: left;" on:click={() => { handleExportPdf(); showExportMenu = false; }}>
        <div class="palette-title" style="display: flex; align-items: center; gap: 8px;">
          <FileUp size={14} /> Export PDF
        </div>
        <div class="palette-desc">Print rendered document to PDF</div>
      </button>
    </div>
  </div>
{/if}

{#if showTemplateMenu}
  <div class="palette-backdrop" role="presentation" on:click={() => (showTemplateMenu = false)}></div>
  <div class="floating-panel template-float-panel">
    <div class="search-panel-header">
      <h3>Templates</h3>
      <button class="status-icon-btn" on:click={() => (showTemplateMenu = false)} title="Close">×</button>
    </div>
    <p class="template-panel-subtitle">Pick a template to open a pre-filled document in a new tab.</p>
    <div class="template-grid">
      {#each TEMPLATES as tpl}
        <button class="template-card" on:click={() => applyTemplate(tpl)}>
          <span class="template-card-emoji">{tpl.emoji}</span>
          <div class="template-card-body">
            <div class="template-card-name">{tpl.name}</div>
            <div class="template-card-desc">{tpl.description}</div>
          </div>
        </button>
      {/each}
    </div>
  </div>
{/if}

{#if showTableMenu}
  <div class="palette-backdrop" role="presentation" on:click={() => (showTableMenu = false)}></div>
  <div class="floating-panel table-float-panel">
    <div class="search-panel-header">
      <h3>Insert Table</h3>
      <button class="status-icon-btn" on:click={() => (showTableMenu = false)} title="Close">×</button>
    </div>
    <p class="template-panel-subtitle">Drag over the grid to set dimensions, then click to insert.</p>
    <div class="table-grid-modal" role="grid">
      {#each Array(8) as _, row}
        <div class="table-grid-row-modal">
          {#each Array(10) as _, col}
            <button
              class="table-cell-modal"
              class:active-cell={row < tableHoverRows && col < tableHoverCols}
              on:mouseenter={() => { tableHoverRows = row + 1; tableHoverCols = col + 1; }}
              on:focus={() => { tableHoverRows = row + 1; tableHoverCols = col + 1; }}
              on:mouseleave={() => { if (tableHoverRows === row + 1 && tableHoverCols === col + 1) { tableHoverRows = 0; tableHoverCols = 0; } }}
              on:click={() => insertTable(row + 1, col + 1)}
              title={`${row + 1} × ${col + 1}`}
            ></button>
          {/each}
        </div>
      {/each}
    </div>
    <div class="table-grid-label-modal">
      {#if tableHoverRows > 0 && tableHoverCols > 0}
        {tableHoverRows} × {tableHoverCols} table
      {:else}
        Hover to preview size
      {/if}
    </div>
  </div>
{/if}

{#if showToolbarOverflowMenu}
  <div class="palette-backdrop" role="presentation" on:click={() => (showToolbarOverflowMenu = false)}></div>
  <div class="floating-panel overflow-float-panel">
    <div class="search-panel-header">
      <h3>Format</h3>
      <button class="status-icon-btn" on:click={() => (showToolbarOverflowMenu = false)} title="Close">×</button>
    </div>
    <div class="overflow-modal-grid">
      <button class="toolbar-icon-btn" on:click={() => { wrapSelection('**'); showToolbarOverflowMenu = false; }} title="Bold"><Bold size={16} /></button>
      <button class="toolbar-icon-btn" on:click={() => { wrapSelection('*'); showToolbarOverflowMenu = false; }} title="Italic"><Italic size={16} /></button>
      <button class="toolbar-icon-btn" on:click={() => { wrapSelection('~~', '~~', 'text'); showToolbarOverflowMenu = false; }} title="Strikethrough"><Strikethrough size={16} /></button>
      <button class="toolbar-icon-btn" on:click={() => { wrapSelection('<u>', '</u>', 'text'); showToolbarOverflowMenu = false; }} title="Underline"><Underline size={16} /></button>
      <button class="toolbar-icon-btn" on:click={() => { insertHeading(1); showToolbarOverflowMenu = false; }} title="Header 1"><Heading1 size={16} /></button>
      <button class="toolbar-icon-btn" on:click={() => { insertHeading(2); showToolbarOverflowMenu = false; }} title="Header 2"><Heading2 size={16} /></button>
      <button class="toolbar-icon-btn" on:click={() => { insertHeading(3); showToolbarOverflowMenu = false; }} title="Header 3"><Heading3 size={16} /></button>
      <button class="toolbar-icon-btn" on:click={() => { openTableMenu(); showToolbarOverflowMenu = false; }} title="Create table"><Table2 size={16} /></button>
      <button class="toolbar-icon-btn" on:click={() => { insertImageSnippet(); showToolbarOverflowMenu = false; }} title="Insert image"><ImagePlus size={16} /></button>
      <button class="toolbar-icon-btn" on:click={() => { insertLinkSnippet(); showToolbarOverflowMenu = false; }} title="Insert link"><Link2 size={16} /></button>
      <button class="toolbar-icon-btn" on:click={() => { formatUrlAction(); showToolbarOverflowMenu = false; }} title="Format URL"><ScanText size={16} /></button>
      <button class="toolbar-icon-btn" on:click={() => { applyOrderedList(); showToolbarOverflowMenu = false; }} title="Ordered list"><ListOrdered size={16} /></button>
      <button class="toolbar-icon-btn" on:click={() => { applyPrefixToSelectedLines('- '); showToolbarOverflowMenu = false; }} title="Unordered list"><List size={16} /></button>
      <button class="toolbar-icon-btn" on:click={() => { applyPrefixToSelectedLines('- [ ] '); showToolbarOverflowMenu = false; }} title="Task list"><ListTodo size={16} /></button>
      <button class="toolbar-icon-btn" on:click={() => { insertInlineCode(); showToolbarOverflowMenu = false; }} title="Inline code"><Code size={16} /></button>
      <button class="toolbar-icon-btn" on:click={() => { insertCodeBlock(); showToolbarOverflowMenu = false; }} title="Code block"><SquareCode size={16} /></button>
      <button class="toolbar-icon-btn" on:click={() => { applyPrefixToSelectedLines('> '); showToolbarOverflowMenu = false; }} title="Quote"><Quote size={16} /></button>
      <button class="toolbar-icon-btn" on:click={() => { insertHorizontalRule(); showToolbarOverflowMenu = false; }} title="Horizontal rule"><Minus size={16} /></button>
      <button class="toolbar-icon-btn" on:click={() => { openEmojiMenu(); showToolbarOverflowMenu = false; }} title="Insert emoji"><Smile size={16} /></button>
      <button class="toolbar-icon-btn" on:click={() => { appStore.toggleWordWrap(); showToolbarOverflowMenu = false; }} title="Toggle word wrap"><WrapText size={16} /></button>
      <button class="toolbar-icon-btn" on:click={() => { toggleImportMenu(); showToolbarOverflowMenu = false; }} title="Imports"><Upload size={16} /></button>
      <button class="toolbar-icon-btn" on:click={() => { toggleExportMenu(); showToolbarOverflowMenu = false; }} title="Exports"><Download size={16} /></button>
      <button class="toolbar-icon-btn" on:click={() => { appStore.toggleTheme(); showToolbarOverflowMenu = false; }} title="Toggle theme">
        {#if $appStore.theme === 'dark'}<Sun size={16} />{:else}<Moon size={16} />{/if}
      </button>
      <button class="toolbar-icon-btn" on:click={() => { toggleFullscreen(); showToolbarOverflowMenu = false; }} title="Toggle fullscreen">
        {#if isFullscreen}<Minimize size={16} />{:else}<Maximize size={16} />{/if}
      </button>
    </div>
  </div>
{/if}

{#if showLinkModal}
  <div class="palette-backdrop" role="presentation" on:click={() => (showLinkModal = false)}></div>
  <div class="floating-panel link-modal-panel">
    <div class="search-panel-header">
      <h3>Insert Link</h3>
      <button class="status-icon-btn" on:click={() => (showLinkModal = false)} title="Close">×</button>
    </div>
    <div class="link-modal-form">
      <label class="link-modal-label" for="link-modal-text">Link text</label>
      <input
        id="link-modal-text"
        class="panel-input"
        type="text"
        placeholder="e.g. Visit our site"
        bind:value={linkModalText}
        on:keydown={(e) => { if (e.key === 'Enter') confirmLinkInsert(); if (e.key === 'Escape') showLinkModal = false; }}
      />
      <label class="link-modal-label" for="link-modal-url">URL</label>
      <input
        id="link-modal-url"
        class="panel-input"
        type="url"
        placeholder="https://"
        bind:value={linkModalUrl}
        on:keydown={(e) => { if (e.key === 'Enter') confirmLinkInsert(); if (e.key === 'Escape') showLinkModal = false; }}
      />
    </div>
    <div class="link-modal-actions">
      <button class="btn btn-secondary" on:click={() => (showLinkModal = false)}>Cancel</button>
      <button class="btn btn-primary" on:click={confirmLinkInsert} disabled={!linkModalUrl.trim()}>Insert</button>
    </div>
  </div>
{/if}

{#if showImageModal}
  <div class="palette-backdrop" role="presentation" on:click={() => (showImageModal = false)}></div>
  <div class="floating-panel link-modal-panel">
    <div class="search-panel-header">
      <h3>Insert Image</h3>
      <button class="status-icon-btn" on:click={() => (showImageModal = false)} title="Close">×</button>
    </div>
    <div class="link-modal-form">
      <label class="link-modal-label" for="image-modal-alt">Alt text</label>
      <input
        id="image-modal-alt"
        class="panel-input"
        type="text"
        placeholder="e.g. Screenshot of dashboard"
        bind:value={imageModalAlt}
        on:keydown={(e) => { if (e.key === 'Enter') confirmImageInsert(); if (e.key === 'Escape') showImageModal = false; }}
      />
      <label class="link-modal-label" for="image-modal-url">Image URL</label>
      <input
        id="image-modal-url"
        class="panel-input"
        type="url"
        placeholder="https://"
        bind:value={imageModalUrl}
        on:keydown={(e) => { if (e.key === 'Enter') confirmImageInsert(); if (e.key === 'Escape') showImageModal = false; }}
      />
    </div>
    <div class="link-modal-actions">
      <button class="btn btn-secondary" on:click={() => (showImageModal = false)}>Cancel</button>
      <button class="btn btn-primary" on:click={confirmImageInsert} disabled={!imageModalUrl.trim()}>Insert</button>
    </div>
  </div>
{/if}

<style>
  @import url('https://fonts.googleapis.com/css2?family=Questrial&display=swap');

  .app-container {
    display: flex;
    flex-direction: column;
    height: 100vh;
    width: 100vw;
    overflow: hidden;
    background: transparent;
    padding: 10px;
    gap: 10px;
  }

  .header {
    display: flex;
    align-items: center;
    gap: 8px;
    min-height: 40px;
    padding: 4px 6px;
    border: 1px solid var(--border-subtle);
    background: color-mix(in srgb, var(--bg-surface) 88%, transparent);
    box-shadow: var(--shadow-sm);
    backdrop-filter: blur(8px);
  }

  .app-brand {
    flex-shrink: 0;
    font-size: 0.82rem;
    font-weight: 800;
    letter-spacing: 0.05em;
    text-transform: uppercase;
    color: var(--text-primary);
    padding: 0 4px;
    font-family: 'Questrial', sans-serif;
  }

  .tabs-container {
    flex: 1;
    min-width: 0;
    overflow: hidden;
  }

  .tabs-scroll {
    display: flex;
    align-items: center;
    gap: 6px;
    overflow-x: auto;
    overflow-y: hidden;
    max-width: 100%;
    padding: 2px 2px 6px;
    scrollbar-width: thin;
    scrollbar-color: color-mix(in srgb, var(--text-secondary) 55%, transparent) transparent;
  }

  .tabs-scroll::-webkit-scrollbar {
    height: 6px;
  }

  .tabs-scroll::-webkit-scrollbar-track {
    background: transparent;
  }

  .tabs-scroll::-webkit-scrollbar-thumb {
    background: color-mix(in srgb, var(--text-secondary) 48%, transparent);
  }

  .tabs-scroll::-webkit-scrollbar-thumb:hover {
    background: color-mix(in srgb, var(--text-primary) 62%, transparent);
  }

  .tab-wrapper {
    display: flex;
    align-items: center;
    flex-shrink: 0;
  }

  .tab-wrapper.active {
    z-index: 2;
  }

  .tab {
    display: inline-flex;
    align-items: center;
    gap: 8px;
    max-width: 220px;
    border: 1px solid transparent;
    border-radius: 6px;
    background: transparent;
    color: var(--text-secondary);
    padding: 6px 8px;
    font-size: 0.8rem;
    font-weight: 600;
    cursor: pointer;
    transition: all var(--transition-fast);
  }

  .tab:hover {
    background: var(--bg-hover);
    color: var(--text-primary);
  }

  .tab.active {
    background: var(--bg-surface);
    border-color: var(--border-subtle);
    color: var(--text-primary);
    box-shadow: var(--shadow-sm);
  }

  .tab-name {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    max-width: 124px;
  }

  .tab-close {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 18px;
    height: 18px;
    margin-left: 2px;
    border: none;
    border-radius: 4px;
    background: transparent;
    color: inherit;
    opacity: 0;
    transform: scale(0.88);
    transition: all var(--transition-fast);
    cursor: pointer;
  }

  .tab:hover .tab-close,
  .tab.active .tab-close {
    opacity: 1;
    transform: scale(1);
  }

  .tab-close:hover {
    background: var(--bg-hover);
    color: var(--accent-primary);
  }

  .rename-input {
    font-family: inherit;
    width: 160px;
    border: 1px solid var(--accent-primary);
    border-radius: 6px;
    background: var(--bg-surface);
    color: var(--text-primary);
    padding: 6px 8px;
    font-size: 0.8rem;
    font-weight: 600;
    outline: none;
    box-shadow: 0 0 0 3px color-mix(in srgb, var(--accent-primary) 16%, transparent);
  }

  .add-tab-btn {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 24px;
    height: 24px;
    border: none;
    background: transparent;
    color: var(--text-secondary);
    cursor: pointer;
    transition: all var(--transition-fast);
  }

  .add-tab-btn:hover {
    background: color-mix(in srgb, var(--accent-primary) 12%, transparent);
    color: var(--accent-primary);
  }

  .add-tab-btn:disabled,
  .add-tab-btn.is-disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .add-tab-btn:disabled:hover,
  .add-tab-btn.is-disabled:hover {
    background: transparent;
    color: var(--text-secondary);
  }

  .add-tab-btn.is-active {
    background: color-mix(in srgb, var(--accent-primary) 14%, transparent);
    color: var(--accent-primary);
  }

  /* ── Template Panel ─────────────────────────────────── */
  .template-float-panel {
    width: min(540px, 96vw);
    max-height: 80vh;
    overflow-y: auto;
  }

  .template-panel-subtitle {
    font-size: 0.78rem;
    color: var(--text-secondary);
    margin: 0 0 14px;
    padding: 0 2px;
  }

  .template-grid {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .template-card {
    display: flex;
    align-items: flex-start;
    gap: 14px;
    width: 100%;
    padding: 12px 14px;
    border: 1px solid var(--border-subtle);
    border-radius: 10px;
    background: transparent;
    color: var(--text-primary);
    cursor: pointer;
    text-align: left;
    transition: background var(--transition-fast), border-color var(--transition-fast), transform var(--transition-fast);
  }

  .template-card:hover {
    background: var(--bg-hover);
    border-color: color-mix(in srgb, var(--accent-primary) 40%, transparent);
    transform: translateX(3px);
  }

  .template-card:active {
    transform: scale(0.99);
  }

  .template-card-emoji {
    font-size: 1.55rem;
    line-height: 1;
    flex-shrink: 0;
    margin-top: 2px;
  }

  .template-card-body {
    display: flex;
    flex-direction: column;
    gap: 3px;
    min-width: 0;
  }

  .template-card-name {
    font-size: 0.875rem;
    font-weight: 700;
    color: var(--text-primary);
    letter-spacing: -0.01em;
  }

  .template-card-desc {
    font-size: 0.76rem;
    color: var(--text-secondary);
    line-height: 1.45;
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

  .workspace {
    display: flex;
    flex: 1;
    overflow: hidden;
    min-height: 0;
    border-radius: 8px;
    border: 1px solid var(--border-subtle);
    background: color-mix(in srgb, var(--bg-surface) 86%, transparent);
    box-shadow: var(--shadow-md);
  }

  .toolbar-host {
    border: 1px solid var(--border-subtle);
    border-top: none;
    background: color-mix(in srgb, var(--bg-surface) 90%, transparent);
  }

  .empty-toolbars {
    min-height: 40px;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .empty-toolbar-state {
    display: inline-flex;
    align-items: center;
    gap: 10px;
    font-size: 0.8rem;
    color: var(--text-secondary);
  }

  .editor-pane,
  .preview-pane {
    display: flex;
    flex-direction: column;
    min-width: 220px;
    min-height: 0;
    background: var(--bg-surface);
  }

  .editor-pane {
    border-right: 1px solid var(--border-subtle);
    overflow: hidden;
    min-height: 0;
  }

  .preview-pane {
    flex: 1;
    overflow: hidden;
    min-height: 0;
  }

  .format-toolbar {
    position: relative;
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 6px;
    border-bottom: 1px solid var(--border-subtle);
    background: color-mix(in srgb, var(--bg-sidebar) 70%, var(--bg-surface));
    overflow-x: auto;
    overflow-y: visible;
    width: 100%;
  }

  .unified-toolbar {
    justify-content: flex-start;
    width: 100%;
  }

  .toolbar-main {
    display: flex;
    align-items: center;
    gap: 10px;
    width: 100%;
    flex-wrap: nowrap;
  }

  .toolbar-group {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    flex-wrap: nowrap;
  }

  .toolbar-divider {
    width: 1px;
    height: 16px;
    margin: 0 8px;
    background: color-mix(in srgb, var(--text-secondary) 42%, transparent);
    opacity: 0.7;
    flex-shrink: 0;
  }

  .toolbar-left,
  .toolbar-right {
    display: flex;
    align-items: center;
    gap: 0;
    flex-wrap: nowrap;
  }

  .toolbar-spacer {
    flex: 1 1 auto;
    min-width: 40px;
  }

  .toolbar-icon-btn {
    width: 28px;
    height: 28px;
    min-width: 28px;
    border: none;
    background: transparent;
    color: var(--text-secondary);
    display: inline-flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
    transition: all var(--transition-fast);
  }

  .toolbar-icon-btn:hover,
  .toolbar-icon-btn.active-toolbar {
    color: var(--text-primary);
    background: color-mix(in srgb, var(--accent-primary) 12%, transparent);
  }

  .preview-icon-btn {
    width: 28px;
    height: 28px;
    min-width: 28px;
    border: none;
    background: transparent;
    color: var(--text-secondary);
    display: inline-flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
    transition: all var(--transition-fast);
  }

  .preview-icon-btn:hover,
  .preview-icon-btn.active-accent {
    color: var(--text-primary);
    background: color-mix(in srgb, var(--accent-primary) 12%, transparent);
  }

  .mobile-toolbar {
    display: none;
  }

  /* ── Unified Table Modal ───────────────────────────── */
  .table-float-panel {
    width: min(340px, 96vw);
  }

  .table-grid-modal {
    display: flex;
    flex-direction: column;
    gap: 3px;
    margin-top: 4px;
  }

  .table-grid-row-modal {
    display: flex;
    gap: 3px;
  }

  .table-cell-modal {
    width: 24px;
    height: 24px;
    border: 1px solid var(--border-subtle);
    border-radius: 3px;
    background: transparent;
    cursor: pointer;
    padding: 0;
    transition: background var(--transition-fast), border-color var(--transition-fast);
  }

  .table-cell-modal:hover,
  .table-cell-modal.active-cell {
    background: color-mix(in srgb, var(--accent-primary) 22%, transparent);
    border-color: var(--accent-primary);
  }

  .table-grid-label-modal {
    margin-top: 10px;
    font-size: 0.75rem;
    color: var(--text-secondary);
    text-align: center;
    min-height: 1.2em;
  }

  /* ── Unified Overflow (Format) Modal ───────────────── */
  .overflow-float-panel {
    width: min(360px, 96vw);
  }

  .overflow-modal-grid {
    display: grid;
    grid-template-columns: repeat(7, 1fr);
    gap: 6px;
    margin-top: 8px;
  }

  /* ── Link / Image Modal ────────────────────────────── */
  .link-modal-panel {
    width: min(380px, 96vw);
  }

  .link-modal-form {
    display: flex;
    flex-direction: column;
    gap: 6px;
    margin-top: 8px;
  }

  .link-modal-label {
    font-size: 0.72rem;
    font-weight: 600;
    letter-spacing: 0.04em;
    text-transform: uppercase;
    color: var(--text-secondary);
    margin-bottom: 1px;
  }

  .link-modal-actions {
    display: flex;
    justify-content: flex-end;
    gap: 8px;
    margin-top: 14px;
  }

  /* ── Live Mode Editor ─────────────────────────────── */
  .live-editor {
    flex: 1;
    overflow-y: auto;
    overflow-x: hidden;
    padding: 2.4rem max(2.4rem, calc(50% - 360px));
    background: var(--bg-surface);
    display: flex;
    flex-direction: column;
    gap: 2px;
    height: 100%;
  }

  .live-block-rendered,
  .live-block-raw {
    width: 100%;
    border-radius: 6px;
    padding: 6px 10px;
    transition: background 0.15s ease, opacity 0.18s ease, transform 0.18s ease;
    cursor: text;
    min-height: 1.8em;
    position: relative;
    animation: live-block-appear 0.18s ease forwards;
  }

  @keyframes live-block-appear {
    from { opacity: 0; transform: translateY(4px); }
    to { opacity: 1; transform: translateY(0); }
  }

  .live-block-rendered {
    color: var(--text-primary);
    line-height: 1.75;
  }

  .live-block-rendered:hover {
    background: color-mix(in srgb, var(--accent-primary) 5%, transparent);
  }

  /* Inherit all markdown-preview typography inside live blocks */
  .live-block-rendered :global(h1),
  .live-block-rendered :global(h2),
  .live-block-rendered :global(h3),
  .live-block-rendered :global(h4) {
    margin: 0;
    line-height: 1.3;
    font-weight: 700;
    color: var(--text-primary);
  }
  .live-block-rendered :global(h1) { font-size: 1.9em; }
  .live-block-rendered :global(h2) { font-size: 1.45em; }
  .live-block-rendered :global(h3) { font-size: 1.18em; }

  .live-block-rendered :global(p) { margin: 0; }
  .live-block-rendered :global(ul),
  .live-block-rendered :global(ol) { margin: 0; padding-left: 1.4em; }
  .live-block-rendered :global(blockquote) {
    margin: 0;
    padding: 0.3em 1em;
    border-left: 3px solid var(--accent-primary);
    color: var(--text-secondary);
    background: color-mix(in srgb, var(--accent-primary) 6%, transparent);
    border-radius: 0 4px 4px 0;
  }
  .live-block-rendered :global(code) {
    font-family: 'JetBrains Mono', 'Fira Code', monospace;
    font-size: 0.88em;
    background: var(--bg-hover);
    padding: 0.1em 0.35em;
    border-radius: 3px;
  }
  .live-block-rendered :global(pre) {
    background: var(--bg-hover);
    padding: 1em;
    border-radius: 6px;
    overflow-x: auto;
    margin: 0;
  }
  .live-block-rendered :global(pre code) {
    background: transparent;
    padding: 0;
  }
  .live-block-rendered :global(table) {
    border-collapse: collapse;
    width: 100%;
  }
  .live-block-rendered :global(th),
  .live-block-rendered :global(td) {
    border: 1px solid var(--border-subtle);
    padding: 0.35em 0.6em;
    text-align: left;
  }
  .live-block-rendered :global(hr) {
    border: none;
    border-top: 1px solid var(--border-subtle);
    margin: 0;
  }

  .live-block-empty { opacity: 0.35; }

  .live-block-placeholder {
    font-style: italic;
    color: var(--text-tertiary);
    font-size: 0.88em;
  }

  .live-block-raw {
    display: block;
    resize: none;
    border: 1px solid color-mix(in srgb, var(--accent-primary) 40%, transparent);
    background: color-mix(in srgb, var(--accent-primary) 4%, var(--bg-surface));
    box-shadow: 0 0 0 3px color-mix(in srgb, var(--accent-primary) 12%, transparent);
    color: var(--text-primary);
    font-family: 'JetBrains Mono', 'Fira Code', monospace;
    font-size: 0.93em;
    line-height: 1.7;
    outline: none;
    overflow: hidden;
    min-height: 2.5em;
  }

  .live-add-block {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 8px 10px;
    color: var(--text-tertiary);
    font-size: 0.78rem;
    cursor: pointer;
    border-radius: 6px;
    transition: color 0.15s ease, background 0.15s ease;
    user-select: none;
    margin-top: 4px;
  }

  .live-add-block:hover {
    color: var(--accent-primary);
    background: color-mix(in srgb, var(--accent-primary) 6%, transparent);
  }

  .live-add-block span {
    font-size: 1rem;
    line-height: 1;
    font-weight: 700;
  }

  .status-live-btn {
    color: var(--text-secondary);
  }

  .status-live-btn:hover,
  .status-live-btn.active-accent {
    color: var(--accent-primary);
  }

  .emoji-float-panel {
    width: min(380px, calc(100vw - 28px));
    padding: 12px;
    overflow: visible;
  }

  :global(.emoji-float-panel emoji-picker) {
    --border-size: 0;
    --background: transparent;
    --input-background-color: var(--bg-hover);
    --input-border-color: var(--border-subtle);
    --input-font-color: var(--text-primary);
    --input-placeholder-color: var(--text-tertiary);
    --category-font-color: var(--text-secondary);
    --category-font-color-hover: var(--text-primary);
    --indicator-color: var(--accent-primary);
    --button-hover-background: color-mix(in srgb, var(--accent-primary) 14%, transparent);
    --outline-color: var(--accent-primary);
    --emoji-size: 1.5rem;
    --emoji-padding: 0.3rem;
    --emoji-font-family: "Apple Color Emoji", "Noto Color Emoji", "Segoe UI Emoji", "Twemoji Mozilla", sans-serif;
    --num-columns: 8;
    width: 100%;
    height: 380px;
  }

  .editor-container {
    display: flex;
    flex: 1;
    min-height: 0;
    overflow: hidden;
  }

  .line-numbers {
    height: 100%;
    overflow-y: auto;
    overflow-x: hidden;
    min-height: 0;
    user-select: none;
    border-right: 1px solid var(--border-subtle);
    background: color-mix(in srgb, var(--bg-sidebar) 55%, transparent);
    pointer-events: none;
  }

  .line-number {
    height: 1.7em;
    line-height: 1.7;
  }

  .line-numbers::-webkit-scrollbar {
    width: 6px;
  }

  .line-numbers::-webkit-scrollbar-track {
    background: transparent;
  }

  .line-numbers::-webkit-scrollbar-thumb {
    background-color: color-mix(in srgb, var(--text-secondary) 30%, transparent);
    border-radius: 3px;
  }

  .line-numbers::-webkit-scrollbar-thumb:hover {
    background-color: color-mix(in srgb, var(--text-secondary) 50%, transparent);
  }

  .editor-textarea {
    flex: 1;
    border: none;
    outline: none;
    resize: none;
    font-family: 'JetBrains Mono', 'Fira Code', monospace;
    font-size: 14px;
    line-height: 1.7;
    background: var(--bg-surface);
    color: var(--text-primary);
    padding: 1.4em;
    tab-size: 2;
    -moz-tab-size: 2;
    white-space: pre;
    overflow-wrap: normal;
    overflow-x: auto;
    overflow-y: auto;
  }

  .editor-textarea::placeholder {
    color: var(--text-tertiary);
  }

  .editor-textarea.drag-over {
    box-shadow: inset 0 0 0 2px var(--accent-primary);
    background: color-mix(in srgb, var(--accent-primary) 6%, var(--bg-surface));
  }

  .editor-textarea.word-wrap-enabled {
    white-space: pre-wrap;
    overflow-wrap: break-word;
  }

  .preview-container {
    flex: 1;
    overflow: auto;
    padding: 2.1rem 2.3rem;
    min-height: 0;
    height: 100%;
    background: var(--bg-surface);
  }

  .preview-container.word-wrap-enabled .markdown-preview {
    white-space: normal;
    overflow-wrap: anywhere;
    word-break: break-word;
  }

  .preview-container.word-wrap-disabled .markdown-preview {
    white-space: pre;
    overflow-x: auto;
  }

  .markdown-preview {
    max-width: 820px;
    margin: 0 auto;
  }

  .resizer {
    width: 8px;
    background: transparent;
    cursor: col-resize;
    position: relative;
    flex-shrink: 0;
  }

  .resizer::after {
    content: '';
    position: absolute;
    top: 50%;
    left: 50%;
    width: 2px;
    height: 50px;
    border-radius: 999px;
    background: var(--border-subtle);
    transform: translate(-50%, -50%);
    transition: background var(--transition-fast), box-shadow var(--transition-fast);
  }

  .resizer:hover::after,
  .resizer.active::after {
    background: var(--accent-primary);
    box-shadow: 0 0 0 4px color-mix(in srgb, var(--accent-primary) 16%, transparent);
  }

  .sr-only {
    position: absolute;
    width: 1px;
    height: 1px;
    padding: 0;
    margin: -1px;
    overflow: hidden;
    clip: rect(0, 0, 0, 0);
    white-space: nowrap;
    border: 0;
  }

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

  .search-list {
    max-height: 320px;
    overflow: auto;
    display: flex;
    flex-direction: column;
    gap: 3px;
  }

  .status-bar {
    min-height: 36px;
    flex-shrink: 0;
    border: 1px solid var(--border-subtle);
    border-radius: 6px;
    background: color-mix(in srgb, var(--bg-surface) 88%, transparent);
    box-shadow: var(--shadow-sm);
    padding: 6px 10px;
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 10px;
    font-size: 0.75rem;
  }

  .status-left,
  .status-right {
    display: flex;
    align-items: center;
    gap: 6px;
    flex-wrap: wrap;
  }

  .status-pill {
    display: inline-flex;
    align-items: center;
    gap: 5px;
    padding: 0.2rem 0.35rem;
    border-radius: 999px;
    border: none;
    background: transparent;
    color: var(--text-secondary);
    line-height: 1.2;
    font-weight: 600;
  }

  .status-pill.subtle {
    background: transparent;
  }

  .status-pill.accent {
    background: color-mix(in srgb, var(--accent-primary) 14%, transparent);
    border-color: color-mix(in srgb, var(--accent-primary) 34%, transparent);
    color: var(--accent-primary);
  }

  .status-icon-btn,
  .status-icon-readout {
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

  .status-icon-btn:hover,
  .status-icon-readout:hover {
    color: var(--text-primary);
    background: transparent;
  }

  .status-icon-btn.active-accent {
    color: var(--accent-primary);
    background: color-mix(in srgb, var(--accent-primary) 12%, transparent);
    border-radius: 4px;
  }

  .status-group {
    display: flex;
    align-items: center;
    gap: 2px;
    background: color-mix(in srgb, var(--bg-hover) 30%, transparent);
    padding: 2px;
    border-radius: 6px;
  }

  .status-group .status-icon-btn {
    border-radius: 4px;
  }

  .status-divider {
    width: 1px;
    height: 14px;
    background: var(--border-subtle);
    margin: 0 4px;
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

  .btn-primary {
    background: var(--accent-primary);
    color: #fff;
    border-color: var(--accent-primary);
  }

  .btn-primary:hover {
    background: var(--accent-hover);
    border-color: var(--accent-hover);
    color: #fff;
  }

  .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    min-height: 100%;
    gap: 10px;
    color: var(--text-secondary);
  }

  .empty-icon {
    color: var(--text-tertiary);
  }

  :global(.markdown-preview pre.code-block) {
    position: relative;
    padding-top: 2.7rem;
    border: 1px solid var(--border-subtle);
  }

  :global(.markdown-preview .code-toolbar) {
    position: absolute;
    top: 8px;
    right: 8px;
    display: flex;
    gap: 6px;
    opacity: 0;
    transform: translateY(-4px);
    transition: opacity var(--transition-fast), transform var(--transition-fast);
  }

  :global(.markdown-preview pre.code-block:hover .code-toolbar) {
    opacity: 1;
    transform: translateY(0);
  }

  :global(.markdown-preview .code-action) {
    border: 1px solid var(--border-subtle);
    border-radius: 6px;
    background: var(--bg-surface);
    color: var(--text-secondary);
    font-size: 0.74rem;
    padding: 0.22rem 0.52rem;
    cursor: pointer;
    transition: all var(--transition-fast);
  }

  :global(.markdown-preview .code-action:hover) {
    background: var(--bg-hover);
    color: var(--text-primary);
  }



  :global(.markdown-preview .callout) {
    border: 1px solid var(--border-subtle);
    border-left-width: 4px;
    border-radius: 6px;
    padding: 0.75rem 0.9rem;
    margin: 1rem 0;
    background: color-mix(in srgb, var(--bg-hover) 68%, var(--bg-surface));
  }

  :global(.markdown-preview .callout-title) {
    font-weight: 700;
    margin-bottom: 0.3rem;
    text-transform: uppercase;
    font-size: 0.78rem;
    letter-spacing: 0.02em;
  }

  :global(.markdown-preview .callout.info) { border-left-color: #3b82f6; }
  :global(.markdown-preview .callout.warn),
  :global(.markdown-preview .callout.warning) { border-left-color: #f59e0b; }
  :global(.markdown-preview .callout.error) { border-left-color: #ef4444; }
  :global(.markdown-preview .callout.success) { border-left-color: #22c55e; }
  :global(.markdown-preview .callout.tip) { border-left-color: #a855f7; }

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
    .app-container {
      padding: 8px;
      gap: 8px;
    }

    .header {
      min-height: auto;
      padding: 8px;
    }

    .workspace {
      flex-direction: column;
    }

    .toc-sidebar {
      display: none;
    }

    .editor-pane,
    .preview-pane {
      width: 100% !important;
      height: 45%;
      min-width: 0;
    }

    .editor-pane {
      border-right: none;
      border-bottom: 1px solid var(--border-subtle);
    }

    .resizer {
      width: 100%;
      height: 8px;
      cursor: row-resize;
    }

    .resizer::after {
      width: 56px;
      height: 2px;
    }

    .preview-container {
      padding: 1.2rem;
    }

    .desktop-toolbar {
      display: none;
    }

    .mobile-toolbar {
      display: inline-flex;
    }

    .floating-panel {
      top: auto;
      bottom: 54px;
      left: 8px;
      right: 8px;
      transform: none;
      width: auto;
      max-height: 52vh;
    }

    .status-bar {
      padding: 6px 8px;
      gap: 6px;
    }
  }

  @media (max-width: 640px) {
    .app-container {
      padding: 6px;
      gap: 6px;
    }

    .tabs-scroll {
      gap: 4px;
    }

    .tab {
      max-width: 160px;
      padding: 5px 7px;
    }

    .tab-name {
      max-width: 88px;
    }

    .status-left .status-pill:nth-child(n + 3) {
      display: none;
    }
  }

  /* Flat + compact mode requested */
  .app-container {
    padding: 0 !important;
    gap: 0 !important;
  }

  .header,
  .workspace,
  .status-bar,
  .tab,
  .tab-close,
  .add-tab-btn,
  .rename-input,
  .panel-input,
  .toolbar-icon-btn,
  .preview-icon-btn,
  .btn,
  .floating-panel,
  .status-pill,
  .toolbar-host,
  :global(.markdown-preview pre),
  :global(.markdown-preview code),
  :global(.markdown-preview img),
  :global(.markdown-preview .code-action),
  :global(.markdown-preview .callout) {
    border-radius: 0 !important;
    box-shadow: none !important;
  }

  .header {
    padding: 2px 4px !important;
    gap: 4px !important;
    backdrop-filter: none !important;
  }

  .tabs-scroll,
  .format-toolbar {
    gap: 2px !important;
    padding: 2px !important;
  }

  .tab {
    padding: 3px 6px !important;
  }

  .panel-input,
  .btn,
  .status-icon-btn,
  .status-icon-readout {
    min-height: 24px !important;
    height: 24px !important;
    padding: 0 6px !important;
  }

  .editor-textarea {
    padding: 0.6em !important;
  }

  .preview-container {
    padding: 0.7rem !important;
  }

  .status-bar {
    padding: 2px 6px !important;
    min-height: 26px !important;
  }

  .mobile-dev-backdrop {
    position: fixed;
    inset: 0;
    z-index: 100;
    background: rgba(0, 0, 0, 0.7);
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 16px;
  }

  .mobile-dev-modal {
    width: min(420px, 100%);
    border: 1px solid var(--border-subtle);
    background: var(--bg-surface);
    padding: 14px;
    display: flex;
    flex-direction: column;
    gap: 10px;
  }

  .mobile-dev-modal h3 {
    font-size: 0.95rem;
  }

  .mobile-dev-modal p {
    font-size: 0.85rem;
    color: var(--text-secondary);
  }

  /* ── Undo/Redo disabled state ──────────────────────── */
  .toolbar-icon-btn.disabled {
    opacity: 0.35;
    cursor: default;
    pointer-events: none;
  }

  /* ── Table of Contents Sidebar ─────────────────────── */
  .toc-sidebar {
    width: 220px;
    min-width: 180px;
    max-width: 300px;
    flex-shrink: 0;
    display: flex;
    flex-direction: column;
    border-right: 1px solid var(--border-subtle);
    background: color-mix(in srgb, var(--bg-sidebar) 60%, var(--bg-surface));
    overflow: hidden;
    animation: tocSlideIn 180ms ease-out;
  }

  @keyframes tocSlideIn {
    from { opacity: 0; transform: translateX(-8px); }
    to { opacity: 1; transform: translateX(0); }
  }

  .toc-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 10px 12px;
    font-size: 0.72rem;
    font-weight: 700;
    text-transform: uppercase;
    letter-spacing: 0.06em;
    color: var(--text-secondary);
    border-bottom: 1px solid var(--border-subtle);
    flex-shrink: 0;
  }

  .toc-list {
    flex: 1;
    overflow-y: auto;
    padding: 6px 0;
  }

  .toc-entry {
    display: block;
    width: 100%;
    text-align: left;
    border: none;
    background: transparent;
    color: var(--text-secondary);
    font-size: 0.78rem;
    padding: 5px 12px;
    cursor: pointer;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    transition: background var(--transition-fast), color var(--transition-fast);
    line-height: 1.4;
  }

  .toc-entry:hover {
    background: var(--bg-hover);
    color: var(--text-primary);
  }

  .toc-h1 {
    font-weight: 700;
    color: var(--text-primary);
    font-size: 0.82rem;
  }

  .toc-h2 {
    font-weight: 600;
  }

  .toc-h3 {
    font-size: 0.74rem;
    color: var(--text-tertiary);
  }
</style>
