#!/bin/bash
# FireNotes UI Core Mode (Backend + Frontend)
set -e

# Ir para raiz do projeto (scripts/ está na raiz)
cd "$(dirname "$0")/.."

# Build Rust se necessário
echo "🔨 Building Rust backend..."
cargo build --release

# Iniciar backend em background
echo "🚀 Starting Rust API server on port 8080..."
./target/release/firenotes ui --port 8080 &
BACKEND_PID=$!

# Esperar backend iniciar
sleep 3

# Iniciar frontend
echo "🎨 Starting UI dev server..."
cd firenotes-ui
bun install
bun run dev &
FRONTEND_PID=$!

echo ""
echo "✅ FireNotes Core Mode iniciado!"
echo "   Backend: http://localhost:8080/api"
echo "   Frontend: http://localhost:3000?mode=core&api=http://localhost:8080/api"
echo ""
echo "⚠️  IMPORTANTE: Acesse com os parâmetros na URL acima!"
echo "   Sem os parâmetros, a UI usa modo standalone (localStorage)."
echo ""
echo "Pressione Ctrl+C para parar ambos"

# Trap para matar ambos os processos
trap "kill $BACKEND_PID $FRONTEND_PID 2>/dev/null || true" EXIT

wait