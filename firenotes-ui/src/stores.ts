import { writable, derived } from 'svelte/store';
import type { Readable } from 'svelte/store';

export interface Document {
  id: string;
  name: string;
  content: string;
  createdAt: number;
  updatedAt: number;
  tags?: string[];
}

export interface HistorySnapshot {
  content: string;
  timestamp: number;
}

export interface AppState {
  documents: Document[];
  activeDocumentId: string | null;
  theme: 'light' | 'dark';
  sidebarWidth: number;
  wordWrap: boolean;
  historyByDoc: Record<string, HistorySnapshot[]>;
  mode: 'standalone' | 'core';
  apiEndpoint: string;
  isLoading: boolean;
  error: string | null;
}

const STORAGE_KEY = 'firenotes-aire-v2';
const LEGACY_STORAGE_KEY = 'aire-v2';
const MAX_HISTORY_ENTRIES = 20;
const MAX_TABS = 32;
const HISTORY_DEBOUNCE_MS = 2000;

type LegacyDocument = Partial<Document> & { title?: string; created_at?: string; updated_at?: string; tags?: string[] };
type StoredStateLike = Partial<AppState> & {
  documents?: LegacyDocument[];
  activeWorkspaceId?: string;
  workspaces?: unknown[];
};
type ApiDocument = {
  id: string;
  title: string;
  content: string;
  created_at: string;
  updated_at: string;
  tags?: string[];
  history?: HistorySnapshot[];
};

const now = () => Date.now();

function getInitialMode() {
  if (typeof window === 'undefined') {
    return { mode: 'standalone' as const, apiEndpoint: 'http://localhost:8080/api' };
  }

  const params = new URLSearchParams(window.location.search);
  const mode = params.get('mode') === 'core' ? 'core' : 'standalone';
  const apiEndpoint = params.get('api') || 'http://localhost:8080/api';
  return { mode, apiEndpoint } as const;
}

const initialRuntime = getInitialMode();

const createDefaultDocument = (): Document => ({
  id: crypto.randomUUID(),
  name: 'Untitled',
  content: `# Welcome to FireNotes

This is a **modern**, _fluid_ markdown editor with full GitHub Flavored Markdown support.

## Features

- ✅ Full GFM support (tables, task lists, strikethrough)
- ✅ Syntax highlighting for code blocks
- ✅ Multiple tabs
- ✅ Light & Dark themes
- ✅ Everything auto-saved

## Try It Out

### Code Block

\`\`\`typescript
function greet(name: string): string {
  return \`Hello, \${name}!\`;
}

console.log(greet('World'));
\`\`\`

### Table

| Feature | Status | Notes |
|---------|--------|-------|
| GFM Support | ✅ | Complete |
| Syntax Highlight | ✅ | 100+ languages |
| Themes | ✅ | Light & Dark |

### Task List

- [x] Create awesome editor
- [x] Add smooth animations
- [ ] Write documentation
- [ ] Share with friends

> "Simplicity is the ultimate sophistication." - Leonardo da Vinci

Enjoy writing! 🚀
`,
  createdAt: now(),
  updatedAt: now(),
});

function safeParse(raw: string | null): StoredStateLike | null {
  if (!raw) return null;
  try {
    return JSON.parse(raw) as StoredStateLike;
  } catch (e) {
    console.error('Failed to parse localStorage state:', e);
    return null;
  }
}

function parseTimestamp(value: unknown, fallback = now()): number {
  if (typeof value === 'number' && Number.isFinite(value)) return value;
  if (typeof value === 'string') {
    const parsed = Date.parse(value);
    if (Number.isFinite(parsed)) return parsed;
  }
  return fallback;
}

function normalizeDocuments(input: LegacyDocument[] | undefined): Document[] {
  if (!Array.isArray(input) || input.length === 0) return [createDefaultDocument()];
  const ts = now();
  const docs = input.map((doc, index) => ({
    id: typeof doc.id === 'string' && doc.id.trim() ? doc.id : crypto.randomUUID(),
    name:
      typeof doc.name === 'string' && doc.name.trim()
        ? doc.name.trim()
        : typeof doc.title === 'string' && doc.title.trim()
          ? doc.title.trim()
          : index === 0
            ? 'Untitled'
            : `Untitled ${index + 1}`,
    content: typeof doc.content === 'string' ? doc.content : '',
    createdAt: parseTimestamp(doc.createdAt ?? doc.created_at, ts),
    updatedAt: parseTimestamp(doc.updatedAt ?? doc.updated_at, ts),
    tags: Array.isArray(doc.tags) ? doc.tags : [],
  }));
  return docs.slice(0, MAX_TABS);
}

function normalizeHistory(historyByDoc: unknown): Record<string, HistorySnapshot[]> {
  if (!historyByDoc || typeof historyByDoc !== 'object') return {};
  const entries = Object.entries(historyByDoc as Record<string, unknown>).map(([docId, snapshots]) => {
    if (!Array.isArray(snapshots)) return [docId, [] as HistorySnapshot[]] as const;
    const normalized = snapshots
      .filter((entry): entry is { content: unknown; timestamp: unknown } => !!entry && typeof entry === 'object')
      .map((entry) => ({
        content: typeof entry.content === 'string' ? entry.content : '',
        timestamp: parseTimestamp(entry.timestamp),
      }))
      .slice(-MAX_HISTORY_ENTRIES);
    return [docId, normalized] as const;
  });
  return Object.fromEntries(entries);
}

function apiToDocument(doc: ApiDocument): Document {
  return {
    id: doc.id,
    name: doc.title,
    content: doc.content,
    createdAt: parseTimestamp(doc.created_at),
    updatedAt: parseTimestamp(doc.updated_at),
    tags: doc.tags || [],
  };
}

function documentToApi(doc: Document) {
  return {
    id: doc.id,
    title: doc.name,
    content: doc.content,
    tags: doc.tags || [],
  };
}

function loadFromStorage(): AppState {
  const rawV2 = safeParse(localStorage.getItem(STORAGE_KEY));
  const rawLegacy = rawV2 ? null : safeParse(localStorage.getItem(LEGACY_STORAGE_KEY));
  const stored = rawV2 || rawLegacy;

  const documents = initialRuntime.mode === 'core'
    ? [createDefaultDocument()]
    : normalizeDocuments(stored?.documents);
  const activeDocumentId =
    typeof stored?.activeDocumentId === 'string' && documents.some((d) => d.id === stored.activeDocumentId)
      ? stored.activeDocumentId
      : documents[0]?.id || null;

  return {
    documents,
    activeDocumentId,
    theme: stored?.theme === 'light' ? 'light' : 'dark',
    sidebarWidth: typeof stored?.sidebarWidth === 'number' ? stored.sidebarWidth : 50,
    wordWrap: stored?.wordWrap ?? true,
    historyByDoc: normalizeHistory(stored?.historyByDoc),
    mode: initialRuntime.mode,
    apiEndpoint: initialRuntime.apiEndpoint,
    isLoading: false,
    error: null,
  };
}

function saveToStorage(state: AppState): void {
  try {
    const persisted = state.mode === 'core'
      ? {
          theme: state.theme,
          sidebarWidth: state.sidebarWidth,
          wordWrap: state.wordWrap,
          activeDocumentId: state.activeDocumentId,
          mode: state.mode,
          apiEndpoint: state.apiEndpoint,
          historyByDoc: state.historyByDoc,
        }
      : state;
    localStorage.setItem(STORAGE_KEY, JSON.stringify(persisted));
  } catch (e) {
    console.error('Failed to save to localStorage:', e);
  }
}

function createDocumentStore() {
  const initialState: AppState = loadFromStorage();
  const { subscribe, set, update } = writable<AppState>(initialState);
  let currentState = initialState;

  subscribe((state) => {
    currentState = state;
    saveToStorage(state);
  });

  async function apiRequest(path: string, init?: RequestInit) {
    const response = await fetch(`${currentState.apiEndpoint}${path}`, {
      headers: {
        'Content-Type': 'application/json',
        ...(init?.headers || {}),
      },
      ...init,
    });
    if (!response.ok) {
      const body = await response.text();
      throw new Error(body || response.statusText);
    }
    if (response.status === 204) return null;
    return response.json();
  }

  function syncCreateDocument(doc: Document) {
    if (currentState.mode !== 'core') return;
    apiRequest('/documents', {
      method: 'POST',
      body: JSON.stringify(documentToApi(doc)),
    }).catch((e) => {
      update((state) => ({ ...state, error: e instanceof Error ? e.message : 'Failed to create document' }));
    });
  }

  function syncUpdateDocument(id: string, updates: Partial<Document>) {
    if (currentState.mode !== 'core') return;
    const body: Record<string, unknown> = {};
    if (typeof updates.name === 'string') body.title = updates.name;
    if (typeof updates.content === 'string') body.content = updates.content;
    if (updates.tags) body.tags = updates.tags;
    apiRequest(`/documents/${id}`, {
      method: 'PUT',
      body: JSON.stringify(body),
    }).catch((e) => {
      update((state) => ({ ...state, error: e instanceof Error ? e.message : 'Failed to update document' }));
    });
  }

  return {
    subscribe,

    loadDocuments: async () => {
      if (currentState.mode !== 'core') return;
      update((state) => ({ ...state, isLoading: true, error: null }));
      try {
        const remoteDocs = (await apiRequest('/documents')) as ApiDocument[];
        const documents = remoteDocs.map(apiToDocument);
        const fallback = documents.length > 0 ? null : createDefaultDocument();
        if (fallback) syncCreateDocument(fallback);
        update((state) => ({
          ...state,
          documents: documents.length > 0 ? documents : [fallback!],
          activeDocumentId:
            documents.some((doc) => doc.id === state.activeDocumentId)
              ? state.activeDocumentId
              : documents[0]?.id || null,
          historyByDoc: {
            ...state.historyByDoc,
            ...Object.fromEntries(remoteDocs.map((doc) => [doc.id, doc.history || []])),
          },
          isLoading: false,
        }));
      } catch (e) {
        update((state) => ({
          ...state,
          isLoading: false,
          error: e instanceof Error ? e.message : 'Failed to load documents',
        }));
      }
    },

    createDocument: (name?: string, content?: string) => {
      const newDoc: Document = {
        id: crypto.randomUUID(),
        name: name || 'Untitled',
        content: content || '',
        createdAt: now(),
        updatedAt: now(),
        tags: [],
      };
      let createdId: string | null = null;

      update((state) => {
        if (state.documents.length >= MAX_TABS) return state;
        createdId = newDoc.id;
        return {
          ...state,
          documents: [...state.documents, newDoc],
          activeDocumentId: newDoc.id,
        };
      });

      if (createdId) syncCreateDocument(newDoc);
      return createdId;
    },

    updateDocument: (id: string, updates: Partial<Document>) => {
      update((state) => ({
        ...state,
        documents: state.documents.map((doc) =>
          doc.id === id
            ? { ...doc, ...updates, updatedAt: now() }
            : doc
        ),
      }));
      syncUpdateDocument(id, updates);
    },

    updateDocumentContent: (id: string, content: string) => {
      let shouldSync = false;
      update((state) => {
        const timestamp = now();
        const previousDoc = state.documents.find((doc) => doc.id === id);
        const documents = state.documents.map((doc) =>
          doc.id === id
            ? { ...doc, content, updatedAt: timestamp }
            : doc
        );
        const previousHistory = state.historyByDoc[id] || [];
        const lastSnapshot = previousHistory[previousHistory.length - 1];
        const timeSinceLast = lastSnapshot ? timestamp - lastSnapshot.timestamp : Infinity;
        const shouldAppend = previousDoc && previousDoc.content !== content && timeSinceLast >= HISTORY_DEBOUNCE_MS;
        shouldSync = !!previousDoc && previousDoc.content !== content;
        const historyByDoc = shouldAppend
          ? {
              ...state.historyByDoc,
              [id]: [
                ...previousHistory,
                { content: previousDoc.content, timestamp },
              ].slice(-MAX_HISTORY_ENTRIES),
            }
          : state.historyByDoc;

        return {
          ...state,
          documents,
          historyByDoc,
        };
      });
      if (shouldSync) syncUpdateDocument(id, { content });
    },

    restoreDocumentVersion: (id: string, historyIndex: number) => {
      let restoredContent: string | null = null;
      update((state) => {
        const snapshots = state.historyByDoc[id] || [];
        const snapshot = snapshots[historyIndex];
        if (!snapshot) return state;
        restoredContent = snapshot.content;
        return {
          ...state,
          documents: state.documents.map((doc) =>
            doc.id === id
              ? { ...doc, content: snapshot.content, updatedAt: now() }
              : doc
          ),
        };
      });
      if (restoredContent !== null) syncUpdateDocument(id, { content: restoredContent });
    },

    renameDocument: (id: string, name: string) => {
      update((state) => ({
        ...state,
        documents: state.documents.map((doc) =>
          doc.id === id
            ? { ...doc, name, updatedAt: now() }
            : doc
        ),
      }));
      syncUpdateDocument(id, { name });
    },

    deleteDocument: (id: string) => {
      update((state) => {
        const documents = state.documents.filter((doc) => doc.id !== id);
        if (documents.length === 0) {
          const fallback = createDefaultDocument();
          const { [id]: _removed, ...remainingHistory } = state.historyByDoc;
          if (state.mode === 'core') syncCreateDocument(fallback);
          return {
            ...state,
            documents: [fallback],
            activeDocumentId: fallback.id,
            historyByDoc: remainingHistory,
          };
        }

        const activeDocumentId = state.activeDocumentId === id ? documents[0].id : state.activeDocumentId;
        return {
          ...state,
          documents,
          activeDocumentId,
          historyByDoc: Object.fromEntries(Object.entries(state.historyByDoc).filter(([docId]) => docId !== id)),
        };
      });
      if (currentState.mode === 'core') {
        apiRequest(`/documents/${id}`, { method: 'DELETE' }).catch((e) => {
          update((state) => ({ ...state, error: e instanceof Error ? e.message : 'Failed to delete document' }));
        });
      }
    },

    setActiveDocument: (id: string) => {
      update((state) => ({
        ...state,
        activeDocumentId: id,
      }));
    },

    reorderDocuments: (newOrder: Document[]) => {
      update((state) => ({
        ...state,
        documents: newOrder,
      }));
    },

    toggleTheme: () => {
      update((state) => ({
        ...state,
        theme: state.theme === 'light' ? 'dark' : 'light',
      }));
    },

    setTheme: (theme: 'light' | 'dark') => {
      update((state) => ({
        ...state,
        theme,
      }));
    },

    setSidebarWidth: (width: number) => {
      update((state) => ({
        ...state,
        sidebarWidth: Math.max(20, Math.min(80, width)),
      }));
    },

    toggleWordWrap: () => {
      update((state) => ({
        ...state,
        wordWrap: !state.wordWrap,
      }));
    },

    setWordWrap: (enabled: boolean) => {
      update((state) => ({
        ...state,
        wordWrap: enabled,
      }));
    },

    reset: () => {
      const doc = createDefaultDocument();
      set({
        documents: [doc],
        activeDocumentId: doc.id,
        theme: 'dark',
        sidebarWidth: 50,
        wordWrap: true,
        historyByDoc: {},
        mode: currentState.mode,
        apiEndpoint: currentState.apiEndpoint,
        isLoading: false,
        error: null,
      });
      syncCreateDocument(doc);
    },
  };
}

export const appStore = createDocumentStore();

export const activeDocument: Readable<Document | null> = derived(
  appStore,
  ($appStore) =>
    $appStore.documents.find((doc) => doc.id === $appStore.activeDocumentId) || null
);

export const documentCount: Readable<number> = derived(
  appStore,
  ($appStore) => $appStore.documents.length
);

export const hasUnsavedChanges: Readable<boolean> = derived(
  appStore,
  () => false
);

const undoStacks = new Map<string, string[]>();
const redoStacks = new Map<string, string[]>();
const lastContentByDoc = new Map<string, string>();
let undoDebounceTimer: ReturnType<typeof setTimeout> | null = null;

export function initUndoForDoc(docId: string, content: string) {
  if (!undoStacks.has(docId)) undoStacks.set(docId, []);
  if (!redoStacks.has(docId)) redoStacks.set(docId, []);
  lastContentByDoc.set(docId, content);
}

export function pushUndoIfNeeded(docId: string, prevContent: string) {
  if (undoDebounceTimer) clearTimeout(undoDebounceTimer);
  undoDebounceTimer = setTimeout(() => {
    const stack = undoStacks.get(docId);
    if (!stack) return;
    const last = stack.length > 0 ? stack[stack.length - 1] : null;
    if (prevContent !== last) {
      stack.push(prevContent);
      if (stack.length > 50) stack.shift();
    }
    redoStacks.get(docId)?.splice(0);
  }, 800);
}

export function undo(docId: string, currentContent: string): string | null {
  const stack = undoStacks.get(docId);
  if (!stack || stack.length === 0) return null;
  const prev = stack.pop()!;
  redoStacks.get(docId)?.push(currentContent);
  lastContentByDoc.set(docId, prev);
  return prev;
}

export function redo(docId: string, currentContent: string): string | null {
  const stack = redoStacks.get(docId);
  if (!stack || stack.length === 0) return null;
  const next = stack.pop()!;
  undoStacks.get(docId)?.push(currentContent);
  lastContentByDoc.set(docId, next);
  return next;
}

export function canUndo(docId: string): boolean {
  return (undoStacks.get(docId)?.length ?? 0) > 0;
}

export function canRedo(docId: string): boolean {
  return (redoStacks.get(docId)?.length ?? 0) > 0;
}
