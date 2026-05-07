# FireNotes

Modern markdown editor for the FireflyLabs ecosystem.

## Overview

FireNotes is a versatile markdown editor with two modes of operation:
- **Standalone Mode**: Browser-based with localStorage (perfect for Vercel deployment)
- **Core Mode**: Full-featured with Rust backend for advanced parsing and storage

## Features

- 📝 Clean, distraction-free markdown editor
- 👁️ Real-time preview with syntax highlighting
- 📑 Multi-document support
- 🌙 Dark mode
- 💾 Auto-save
- 🔀 Dual-mode operation (standalone/core)
- 🔍 Full-text search
- 📤 Export to multiple formats (Markdown, HTML, Plain text, JSON)

## Architecture

```
firenotes/
├── firenotes-core/     # Rust library (document, parser, storage, export)
├── firenotes-cli/      # Rust CLI + HTTP API server
└── firenotes-ui/       # Svelte + TypeScript + Tailwind UI
```

## Quick Start

### CLI (Core Mode)
```bash
# Build
cargo build --release

# Install
cargo install --path .

# Start API server
firenotes ui --port 3000

# CLI commands
firenotes new "My Document"
firenotes list
firenotes show <uuid>
```

### UI (Standalone Mode)
```bash
# Script conveniente
./scripts/run-ui-standalone.sh

# Ou manualmente
cd firenotes-ui
bun install      # ⚠️ SEMPRE usar bun
bun run dev
# Access http://localhost:3000
```

### UI (Core Mode)
```bash
# ⚠️ IMPORTANTE: Modo core REQUER parâmetros na URL!

# Script conveniente (backend + frontend)
./scripts/run-ui-core.sh
# Acessar: http://localhost:3000?mode=core&api=http://localhost:8080/api
#
# ⚠️ SEMPRE use os parâmetros ?mode=core&api=... na URL!
# Sem os parâmetros, a UI usa modo standalone (localStorage).

# Ou manualmente (2 terminais)
# Terminal 1: Start backend
firenotes ui --port 8080

# Terminal 2: Start UI
cd firenotes-ui
bun install      # ⚠️ SEMPRE usar bun
bun run dev
# Acessar: http://localhost:3000?mode=core&api=http://localhost:8080/api
```

## Modes

### Standalone Mode
- Uses localStorage for persistence
- No backend required
- Perfect for Vercel/Netlify deployment
- Markdown parsing with marked.js

### Core Mode
- Uses Rust backend via HTTP API
- Advanced parsing with comrak
- Filesystem storage following FireflyLabs patterns
- Better performance and features

## Development

See [AGENTS.md](./AGENTS.md) for detailed development rules and guidelines.

## License

MIT