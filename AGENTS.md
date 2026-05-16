# FireNotes - Agent Guide

## Project Overview

FireNotes is a modern, distraction-free markdown editor built with Svelte, TypeScript, and Vite with an organized source structure.

**Tech Stack:**
- Frontend: Svelte 4.x, TypeScript 5.x
- Build Tool: Vite 5.x
- Markdown: Marked.js 11.x
- Syntax Highlighting: highlight.js 11.x
- Icons: lucide-svelte
- Math Rendering: KaTeX

## Development Commands

**Package Manager:** Use `bun` exclusively (not npm)

```bash
# Install dependencies
bun install

# Start development server
bun run dev

# Build for production
bun run build

# Preview production build
bun run preview

# Type checking
bun run check
```

## Project Structure

```
firenotes/
├── src/
│   ├── components/       # Svelte components
│   │   ├── CommandPalette.svelte
│   │   ├── EmojiPicker.svelte
│   │   └── SearchPanel.svelte
│   ├── stores/           # Svelte stores (global state)
│   │   └── stores.ts
│   ├── styles/           # Global styles
│   │   └── app.css
│   ├── assets/           # Static assets
│   │   └── svelte.svg
│   ├── lib/              # Library components
│   │   └── Counter.svelte
│   ├── App.svelte        # Root component
│   └── main.ts           # Entry point
├── public/               # Static files
├── index.html            # HTML template
├── package.json
├── svelte.config.js
├── vite.config.ts
└── tsconfig.json
```

## Key Files

### Core Application
- `src/App.svelte` - Main application component (3981 lines, contains most logic)
- `src/main.ts` - Application entry point
- `src/stores/stores.ts` - Global state management (documents, theme, undo/redo)

### Components
- `src/components/CommandPalette.svelte` - Command palette (Cmd+K)
- `src/components/SearchPanel.svelte` - Global search functionality
- `src/components/EmojiPicker.svelte` - Emoji picker component

### Configuration
- `vite.config.ts` - Vite configuration
- `svelte.config.js` - Svelte configuration
- `tsconfig.json` - TypeScript configuration

## State Management

The app uses Svelte stores for state management:

**Key Stores (in `src/stores/stores.ts`):**
- `appStore` - Main application state
  - `documents` - Array of document objects
  - `activeDocumentId` - Currently active document
  - `theme` - 'light' | 'dark'
  - `sidebarWidth` - Editor/preview split width
  - `wordWrap` - Word wrap setting

**Derived Stores:**
- `activeDocument` - Currently active document object
- `documentCount` - Number of open documents

**Undo/Redo System:**
- Per-document undo/redo stacks
- Functions: `undo()`, `redo()`, `canUndo()`, `canRedo()`
- Auto-tracks content changes

## Storage

- **Storage Key:** `firenotes-v1`
- **Legacy Key:** `firenotes-legacy`
- **Location:** localStorage
- **Data:** Documents, theme, settings, history

## Markdown Extensions

The app uses custom Marked.js extensions:
1. **Definition Lists** - Term + ": definition"
2. **Callouts** - `:::type Optional title\ncontent\n:::`
3. **Math (KaTeX)** - Inline `$...$` and block `$$...$$`
4. **GFM** - GitHub Flavored Markdown (tables, task lists, strikethrough)

## Features

- Multi-document tabs (max 32)
- Live markdown preview
- Syntax highlighting (100+ languages)
- Light/Dark themes
- Auto-save to localStorage
- Undo/redo per document
- Command palette (Cmd+K)
- Global search
- Emoji picker
- Math rendering (KaTeX)
- Custom markdown extensions
- Word wrap toggle
- Zoom controls
- Export to HTML/PDF
- Templates system

## Code Conventions

- Use TypeScript for type safety
- Follow existing code style in the project
- Components use `<script lang="ts">`
- State management via Svelte stores
- Keep imports organized (external libraries first, then local imports)
- Use lucide-svelte for icons

## Common Tasks

### Adding a New Component
1. Create component in `src/components/`
2. Import in parent component using relative path
3. Follow existing component patterns

### Modifying State
1. Use `appStore` methods from `src/stores/stores.ts`
2. Available methods: `createDocument()`, `updateDocument()`, `deleteDocument()`, etc.
3. Changes auto-save to localStorage

### Adding Markdown Extensions
1. Add extension in `src/App.svelte` (marked configuration section)
2. Follow existing extension pattern (tokenizer + renderer)

### Styling
- Global styles in `src/styles/app.css`
- Component-specific styles using `<style>` blocks in Svelte components
- Uses CSS variables for theming

## Testing

Currently no automated tests are set up. Manual testing:
1. Run `bun run dev`
2. Open http://localhost:5173
3. Test features interactively

## Build & Deployment

```bash
# Build
bun run build

# Preview build
bun run preview
```

Output goes to `dist/` directory. Can be deployed to any static hosting service (Vercel, Netlify, GitHub Pages, etc.).

## Important Notes

- **Always use bun** (not npm) for package management
- The app is fully client-side (no backend)
- All data stored in localStorage
- Maximum 32 documents per workspace
- Undo history limited to 50 entries per document
- Storage debounce: 2 seconds for history snapshots