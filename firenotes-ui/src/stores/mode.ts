import { writable } from 'svelte/store';

export type Mode = 'standalone' | 'core';

interface ModeState {
  mode: Mode;
  apiEndpoint?: string;
}

function createModeStore() {
  const { subscribe, set, update } = writable<ModeState>({
    mode: 'standalone',
    apiEndpoint: undefined
  });

  return {
    subscribe,
    setMode: (mode: Mode, apiEndpoint?: string) => {
      update(state => ({ ...state, mode, apiEndpoint }));
    },
    isStandalone: () => {
      let isStandalone = false;
      subscribe(state => { isStandalone = state.mode === 'standalone'; })();
      return isStandalone;
    }
  };
}

export const modeStore = createModeStore();