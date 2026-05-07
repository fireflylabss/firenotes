#!/bin/bash
# FireNotes UI Standalone Mode
cd "$(dirname "$0")/../firenotes-ui"
bun install
bun run dev