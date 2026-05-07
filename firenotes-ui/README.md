# FireNotes UI

Modern markdown editor UI built with Svelte, TypeScript, and Tailwind CSS.

## Features

- 📝 Clean, distraction-free markdown editor
- 👁️ Real-time preview with syntax highlighting
- 📑 Multi-document support
- 🌙 Dark mode
- 💾 Auto-save
- 🔀 Standalone mode (localStorage) or Core mode (API integration)

## Development

```bash
# Install dependencies (⚠️ SEMPRE usar bun)
bun install

# Start development server
bun run dev

# Build for production
bun run build

# Preview production build
bun run preview
```

⚠️ **IMPORTANTE**: Usar sempre `bun` para gerenciar dependências. Nunca usar `npm` ou `pnpm`.

## Modes

### Standalone Mode
- Uses localStorage for document storage
- No backend required
- Perfect for Vercel deployment
- Enable with `?mode=standalone` URL param (default)

### Core Mode
- Uses FireNotes Core Rust backend via API
- More powerful parsing and storage
- Enable with `?mode=core&api=http://localhost:3000/api` URL params

## Tech Stack

- **Framework**: Svelte 4
- **Language**: TypeScript 5
- **Styling**: Tailwind CSS 3
- **Bundler**: Vite 5
- **Markdown**: marked + highlight.js
- **Icons**: lucide-svelte