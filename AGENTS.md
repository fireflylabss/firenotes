# firenotes — Development Rules

## Regras Específicas

### ⚠️ REGRA CRÍTICA: Bun Obrigatório
- **OBRIGATÓRIO**: Usar **bun** para todos os comandos JavaScript/TypeScript
- **NUNCA** usar npm ou pnpm no firenotes-ui
- **Sempre**: `bun install`, `bun run dev`, `bun run build`
- **Motivo**: Bun é mais rápido, moderno e é o padrão do FireflyLabs

### ⚠️ REGRA CRÍTICA: Scripts em ./scripts/
- **OBRIGATÓRIO**: Todos os scripts shell/bash devem ficar em `./scripts/`
- **NUNCA** criar scripts na raiz do projeto
- **Sempre**: scripts executáveis em `scripts/` com shebang `#!/bin/bash`
- **Motivo**: Organização, descoberta fácil, padrão de projeto

### ⚠️ REGRA CRÍTICA: Modo Core Requer Parâmetros URL
- **OBRIGATÓRIO**: Modo core SEMPRE requer `?mode=core&api=<url>` na URL
- **NUNCA** acessar http://localhost:3000 direto para modo core
- **Sempre**: `http://localhost:3000?mode=core&api=http://localhost:8080/api`
- **Motivo**: UI usa localStorage por padrão se não houver parâmetros

### ⚠️ REGRA CRÍTICA: Interface 100% em Inglês
- **OBRIGATÓRIO**: Toda a interface do usuário (UI) deve ser 100% em inglês
- **NUNCA** traduzir botões, menus, labels, mensagens de erro da UI para português
- **Sempre**: Textos visíveis para o usuário em inglês (buttons, labels, placeholders, tooltips, error messages)
- **Motivo**: Padrão internacional, facilita contribuições, consistência com projetos open source
- **Exceção**: Documentação de projeto (README.md, AGENTS.md) pode estar em português

### Arquitetura
- **Workspace Rust**: `firenotes-core` (biblioteca) + `firenotes-cli` (binário)
- **UI**: Svelte + TypeScript + Tailwind CSS (separado em `firenotes-ui/`)
- **Modos de operação**: Standalone (localStorage) e Core (API Rust backend)

### Estrutura de Diretórios
```
firenotes/
├── Cargo.toml              # Workspace config
├── AGENTS.md               # Este arquivo
├── scripts/                # Scripts shell/bash
│   ├── run-ui-standalone.sh
│   ├── run-ui-core.sh
│   └── kill-dev.sh
├── firenotes-core/         # Biblioteca core (lógica)
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       ├── document.rs     # Estrutura de documento
│       ├── parser.rs       # Markdown parsing
│       ├── storage.rs      # Persistência
│       └── export.rs       # Exportação
├── firenotes-cli/          # CLI binário
│   ├── Cargo.toml
│   └── src/
│       ├── main.rs         # CLI entry point
│       └── server.rs       # HTTP API server
└── firenotes-ui/           # Interface web
    ├── package.json
    ├── vite.config.ts
    ├── tailwind.config.js
    ├── svelte.config.js
    └── src/
        ├── main.ts
        ├── App.svelte
        ├── components/
        │   ├── Editor.svelte
        │   ├── Sidebar.svelte
        │   └── Header.svelte
        ├── stores/
        │   ├── documents.ts
        │   └── mode.ts
        └── lib/
            └── markdown.ts
```

### Linguagens e Frameworks

#### Rust (Core + CLI)
- **Edition**: 2021
- **CLI**: clap (derive feature)
- **Async**: tokio (full features)
- **Web Server**: axum + tower-http (CORS)
- **Markdown**: comrak (parser)
- **JSON**: serde + serde_json
- **Erros**: anyhow::Result para CLI
- **Cores**: colored crate para output colorido

#### Svelte (UI)
- **Framework**: Svelte 4
- **Language**: TypeScript 5
- **Styling**: Tailwind CSS 3
- **Bundler**: Vite 5
- **Markdown**: marked + highlight.js
- **Icons**: lucide-svelte
- **State Management**: Svelte stores

### Convenções de Código

#### Rust
- Structs: PascalCase
- Functions: snake_case
- Constants: UPPER_SNAKE_CASE
- Modules: snake_case
- Usar `anyhow::Result` para todas as funções que podem falhar
- Usar `.context()` para adicionar contexto aos erros
- Nunca usar `unwrap()` ou `expect()` em código de produção
- Propagar erros com `?` operator

#### TypeScript/Svelte
- Components: PascalCase
- Functions/variables: camelCase
- Constants: UPPER_SNAKE_CASE
- Usar TypeScript strict mode
- Svelte components com `<script lang="ts">`
- Props com `export let`
- Stores com Svelte stores API

### Storage Pattern

#### Core (Rust)
- **Primary**: `/firefly/config/firenotes/` (se existir)
- **Fallback**: `~/.firenotes/` (development)
- **Format**: JSON files por documento (`{uuid}.json`)
- **Auto-save**: Implementado no CLI e UI

#### UI (Standalone)
- **Storage**: localStorage do navegador
- **Key**: `firenotes-documents`
- **Format**: Array de documentos em JSON
- **Auto-save**: A cada mudança com debounce (500ms)

### Modos de Operação

#### Standalone Mode
- **Flag**: `--standalone` no CLI ou `?mode=standalone` na UI
- **Storage**: localStorage (UI) ou filesystem local (CLI)
- **Parsing**: marked (UI) ou comrak (CLI)
- **Deploy**: Vercel, Netlify, GitHub Pages (só UI)
- **Use case**: Desenvolvimento rápido, deploy simples

#### Core Mode
- **Flag**: Sem flag no CLI ou `?mode=core&api=<url>` na UI
- **Storage**: `/firefly/config/firenotes/` via Storage Rust
- **Parsing**: comrak via API Rust
- **Backend**: Servidor HTTP axum em porta configurável
- **Use case**: Produção, features avançadas, integração FireflyLabs

### API REST (Core Mode)

#### Endpoints
```
GET    /api/documents          # Listar todos
POST   /api/documents          # Criar documento
GET    /api/documents/:id      # Obter documento
PUT    /api/documents/:id      # Atualizar documento
DELETE /api/documents/:id      # Deletar documento
POST   /api/documents/:id/parse # Parse markdown para HTML
GET    /api/search?q=query     # Buscar documentos
```

#### Request/Response Format
- **Content-Type**: application/json
- **Document**: `{ id, title, content, created_at, updated_at, tags }`
- **Parse**: `{ markdown: string }` → `{ html: string }`

### Build e Instalação

#### Rust (Core + CLI)
```bash
# Build debug
cargo build

# Build release
cargo build --release

# Executar CLI
cargo run -- firenotes --help

# Instalar localmente
cargo install --path .

# Testar API server
cargo run -- firenotes ui --port 3000
```

#### Svelte (UI) - ⚠️ SEMPRE usar bun
```bash
cd firenotes-ui

# Install dependencies (OBRIGATÓRIO usar bun)
bun install

# Development server
bun run dev

# Build for production
bun run build

# Preview production build
bun run preview

# Type checking
bun run check
```

#### ⚠️ IMPORTANTE
- **NUNCA** usar `npm install` ou `npm run` no firenotes-ui
- **SEMPRE** usar `bun install` e `bun run`
- Bun é o padrão obrigatório do FireflyLabs para JS/TS

### Filesystem Structure

#### Production
```
/firefly/
├── apps/
│   └── firenotes/          # App instalado
│       ├── firenotes-core/
│       ├── firenotes-cli/
│       └── firenotes-ui/
└── config/
    └── firenotes/          # Dados persistidos
        ├── {uuid}.json     # Documentos
        └── ...
```

#### Development
```
/home/gabriel/FireflyLabs/firenotes/
├── firenotes-core/
├── firenotes-cli/
└── firenotes-ui/
```

### Storage Pattern (FireflyLabs)

#### Priority
1. `/firefly/config/firenotes/` (produção)
2. `~/.firenotes/` (development fallback)

#### Implementation
- Verificar se `/firefly/` existe antes de usar
- Criar diretórios automaticamente se não existirem
- Usar `dirs` crate para cross-platform home directory

### Comandos CLI

#### Document Management
```bash
# Criar novo documento
firenotes new "My Document"

# Listar documentos
firenotes list

# Editar documento
firenotes edit <uuid>

# Mostrar documento
firenotes show <uuid>

# Deletar documento
firenotes delete <uuid>

# Buscar documentos
firenotes search "query"

# Exportar documento
firenotes export <uuid> --format markdown --output output.md
firenotes export <uuid> --format html
firenotes export <uuid> --format json
```

#### UI Commands
```bash
# ⚠️ MODO CORE REQUER PARÂMETROS NA URL!

# Modo standalone (sem backend) - script conveniente
./scripts/run-ui-standalone.sh
# Acessar: http://localhost:3000

# Modo core (com backend Rust) - script conveniente
./scripts/run-ui-core.sh
# Acessar: http://localhost:3000?mode=core&api=http://localhost:8080/api
# ⚠️ SEMPRE use ?mode=core&api=... na URL!

# Limpar portas de desenvolvimento
./scripts/kill-dev.sh

# Manual: Modo standalone
cd firenotes-ui && bun install && bun run dev
# Acessar: http://localhost:3000

# Manual: Modo core (2 terminais)
# Terminal 1: firenotes ui --port 8080
# Terminal 2: cd firenotes-ui && bun install && bun run dev
# Acessar: http://localhost:3000?mode=core&api=http://localhost:8080/api
```

### Segurança

#### Operações Destrutivas
- Sempre confirmar antes de deletar documentos
- Nunca executar operações em `/` sem verificação
- Validar UUIDs antes de operações

#### CORS (API)
- Permitir qualquer origin em desenvolvimento
- Configurar origins específicas em produção
- Usar tower-http CORS middleware

#### Secrets
- Nunca logar conteúdo de documentos
- Não incluir tokens em logs
- Validar todos os inputs da API

### Performance

#### Rust
- Operações I/O assíncronas com tokio
- Storage em memória para cache frequente
- Parsing lazy (só quando necessário)

#### Svelte
- Auto-save com debounce (500ms)
- Virtual scrolling para listas grandes
- Lazy loading de syntax highlighting

### Testing

#### Manual Testing
```bash
# Testar CLI commands
cargo run -- firenotes new "Test"
cargo run -- firenotes list
cargo run -- firenotes show <uuid>

# Testar API server
cargo run -- firenotes ui --port 3000
curl http://localhost:3000/api/documents

# Testar UI standalone
./scripts/run-ui-standalone.sh
# Acessar: http://localhost:3000

# Testar UI core mode
./scripts/run-ui-core.sh
# Acessar: http://localhost:3000?mode=core&api=http://localhost:8080/api
# ⚠️ IMPORTANTE: Use os parâmetros na URL!
```

### Dependencies

#### Core (firenotes-core)
- `anyhow` — Error handling
- `serde` + `serde_json` — Serialization
- `tokio` — Async runtime
- `chrono` — Date/time
- `uuid` — UUID generation
- `comrak` — Markdown parsing
- `dirs` — Cross-platform directories

#### CLI (firenotes-cli)
- `firenotes-core` — Core library
- `anyhow` — Error handling
- `clap` — CLI parsing
- `colored` — Colored output
- `tokio` — Async runtime
- `axum` — Web server
- `tower-http` — CORS
- `serde` + `serde_json` — Serialization

#### UI (firenotes-ui)
- `svelte` — Framework
- `typescript` — Type checking
- `vite` — Bundler
- `tailwindcss` — Styling
- `marked` — Markdown parsing
- `highlight.js` — Syntax highlighting
- `lucide-svelte` — Icons

### Integration com FireflyLabs

#### Workspace Path
- Desenvolvimento: `/home/gabriel/FireflyLabs/firenotes/`
- Produção: `/firefly/apps/firenotes/` (se instalado via fly)

#### Compatibilidade
- Seguir convenções do FireflyLabs (CONTEXT.md)
- Usar paths padrão do Firefly (`/firefly/`)
- Storage pattern consistente com outros apps
- Compatível com instalação via `fly`

### Debugging

#### Rust
```bash
# Build com debug symbols
cargo build

# Run com RUST_LOG
RUST_LOG=debug cargo run -- firenotes ui

# Backtrace em erros
RUST_BACKTRACE=1 cargo run -- firenotes <command>
```

#### Svelte
```bash
# Development server com verbose
npm run dev -- --debug

# Type checking
npm run check

# Build analysis
npm run build -- --mode development
```

### Common Issues

#### Rust
- **Build errors**: Verificar Rust version (`cargo --version`)
- **Dependency conflicts**: `cargo update`
- **Permission errors**: Verificar `/firefly/` permissions
- **Port already in use**: Mudar porta com `--port`

#### Svelte
- **Module not found**: `bun install`
- **Type errors**: `bun run check`
- **Build failures**: Limpar `node_modules/` e reinstalar com `bun install`
- **CORS errors**: Verificar modo (standalone vs core)

### Future Enhancements

#### Core (Rust)
- [ ] Document versioning
- [ ] Collaboration features
- [ ] Advanced search (full-text)
- [ ] Export to PDF
- [ ] Document templates
- [ ] Tags management

#### UI (Svelte)
- [ ] Drag and drop file import
- [ ] Keyboard shortcuts
- [ ] Split view editing
- [ ] Document history
- [ ] Collaborative editing
- [ ] Plugin system

#### CLI
- [ ] Interactive TUI mode
- [ ] Batch operations
- [ ] Sync with cloud storage
- [ ] Git integration
- [ ] Backup/restore

### Deployment

#### Vercel (Standalone UI)
```bash
cd firenotes-ui
bun install
bun run build
vercel deploy
```

#### Self-hosted (Core Mode)
```bash
# Build Rust
cargo build --release

# Build UI
cd firenotes-ui
npm run build

# Configure reverse proxy (nginx/caddy)
# Point to UI static files + API server
```

### Version Management

#### Version
- Definir em `Cargo.toml` (workspace) e `package.json`
- Usar formato semver (MAJOR.MINOR.PATCH)
- Atualizar em mudanças breaking

#### Changelog
- Manter `CHANGELOG.md` em cada crate
- Seguir Conventional Commits
- Documentar breaking changes