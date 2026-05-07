#!/bin/bash
# Mata todos os processos nas portas 3000 e 8080

echo "🔪 Matando processos na porta 3000..."
lsof -ti:3000 | xargs kill -9 2>/dev/null || echo "Nenhum processo na 3000"

echo "🔪 Matando processos na porta 8080..."
lsof -ti:8080 | xargs kill -9 2>/dev/null || echo "Nenhum processo na 8080"

echo "✅ Limpo!"