import type { SessionRecord } from './wasm.js';

const KEY = 'rus-trainer.sessions';
const CAP = 500;

export interface HistoryStore {
  save(record: SessionRecord): boolean; // false, если хранилище недоступно
  list(): SessionRecord[];
}

export function createHistoryStore(storage: Storage | null): HistoryStore {
  return {
    save(record) {
      if (!storage) return false;
      try {
        const all = this.list();
        all.unshift(record);
        storage.setItem(KEY, JSON.stringify(all.slice(0, CAP)));
        return true;
      } catch {
        return false;
      }
    },
    list() {
      if (!storage) return [];
      try {
        return JSON.parse(storage.getItem(KEY) ?? '[]') as SessionRecord[];
      } catch {
        return [];
      }
    },
  };
}

/** localStorage может бросать SecurityError — возвращаем null тогда. */
export function tryLocalStorage(): Storage | null {
  try {
    const probe = '__probe__';
    window.localStorage.setItem(probe, '1');
    window.localStorage.removeItem(probe);
    return window.localStorage;
  } catch {
    return null;
  }
}
