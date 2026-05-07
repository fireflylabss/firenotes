import { writable, derived } from 'svelte/store';
import { modeStore } from './mode';

export interface Document {
  id: string;
  title: string;
  content: string;
  created_at: string;
  updated_at: string;
  tags: string[];
}

interface DocumentState {
  documents: Document[];
  currentDocument: Document | null;
  isLoading: boolean;
  error: string | null;
}

function createDocumentStore() {
  const { subscribe, set, update } = writable<DocumentState>({
    documents: [],
    currentDocument: null,
    isLoading: false,
    error: null
  });

  // Load documents from localStorage (standalone mode)
  const loadFromLocalStorage = () => {
    try {
      const saved = localStorage.getItem('firenotes-documents');
      if (saved) {
        const docs = JSON.parse(saved);
        update(state => ({ ...state, documents: docs }));
      }
    } catch (e) {
      console.error('Failed to load from localStorage:', e);
    }
  };

  // Save documents to localStorage (standalone mode)
  const saveToLocalStorage = (documents: Document[]) => {
    try {
      localStorage.setItem('firenotes-documents', JSON.stringify(documents));
    } catch (e) {
      console.error('Failed to save to localStorage:', e);
    }
  };

  return {
    subscribe,
    loadDocuments: async () => {
      update(state => ({ ...state, isLoading: true, error: null }));
      
      let isStandalone = true;
      modeStore.subscribe(state => { isStandalone = state.mode === 'standalone'; })();

      if (isStandalone) {
        // Standalone mode: use localStorage
        loadFromLocalStorage();
        update(state => ({ ...state, isLoading: false }));
      } else {
        // Core mode: fetch from API
        try {
          let apiEndpoint = 'http://localhost:3000/api';
          modeStore.subscribe(state => { 
            if (state.apiEndpoint) apiEndpoint = state.apiEndpoint;
          })();

          const response = await fetch(`${apiEndpoint}/documents`);
          if (!response.ok) throw new Error('Failed to fetch documents');
          
          const documents = await response.json();
          update(state => ({ ...state, documents, isLoading: false }));
        } catch (e) {
          update(state => ({ 
            ...state, 
            isLoading: false, 
            error: e instanceof Error ? e.message : 'Unknown error'
          }));
        }
      }
    },
    createDocument: async (title: string, content: string = '') => {
      const newDoc: Document = {
        id: crypto.randomUUID(),
        title,
        content,
        created_at: new Date().toISOString(),
        updated_at: new Date().toISOString(),
        tags: []
      };

      update(state => ({ ...state, documents: [newDoc, ...state.documents] }));
      
      let isStandalone = true;
      modeStore.subscribe(state => { isStandalone = state.mode === 'standalone'; })();

      if (isStandalone) {
        saveToLocalStorage([...(await new Promise<Document[]>((resolve) => {
          let docs: Document[] = [];
          subscribe(state => { docs = state.documents; })();
          resolve(docs);
        }))]);
      } else {
        // TODO: Send to API
      }

      return newDoc;
    },
    updateDocument: async (id: string, updates: Partial<Document>) => {
      update(state => ({
        ...state,
        documents: state.documents.map(doc =>
          doc.id === id 
            ? { ...doc, ...updates, updated_at: new Date().toISOString() }
            : doc
        ),
        currentDocument: state.currentDocument?.id === id
          ? { ...state.currentDocument, ...updates, updated_at: new Date().toISOString() }
          : state.currentDocument
      }));

      let isStandalone = true;
      modeStore.subscribe(state => { isStandalone = state.mode === 'standalone'; })();

      if (isStandalone) {
        let docs: Document[] = [];
        subscribe(state => { docs = state.documents; })();
        saveToLocalStorage(docs);
      } else {
        // TODO: Send to API
      }
    },
    deleteDocument: async (id: string) => {
      update(state => ({
        ...state,
        documents: state.documents.filter(doc => doc.id !== id),
        currentDocument: state.currentDocument?.id === id ? null : state.currentDocument
      }));

      let isStandalone = true;
      modeStore.subscribe(state => { isStandalone = state.mode === 'standalone'; })();

      if (isStandalone) {
        let docs: Document[] = [];
        subscribe(state => { docs = state.documents; })();
        saveToLocalStorage(docs);
      } else {
        // TODO: Send to API
      }
    },
    setCurrentDocument: (document: Document | null) => {
      update(state => ({ ...state, currentDocument: document }));
    }
  };
}

export const documentStore = createDocumentStore();