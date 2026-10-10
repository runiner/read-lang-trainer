// Настройки отображения (шрифт, регистр, методика) — переживают перезапуск.
export type LetterCase = 'lower' | 'upper';

export interface Preferences {
  fontId: string;
  letterCase: LetterCase;
  methodologySeen: boolean;
}

export const FONTS: [string, string][] = [
  ['system', 'Системный (по умолчанию)'],
  ['serif', 'Georgia, с засечками'],
  ['round', 'Округлый рубленый'],
];

export const DEFAULT_PREFERENCES: Preferences = {
  fontId: 'system',
  letterCase: 'lower',
  methodologySeen: false,
};

const KEY = 'rus-trainer.preferences';

export interface PreferencesStore {
  load(): Preferences;
  save(prefs: Preferences): boolean; // false, если хранилище недоступно
}

/** Повреждённые/неизвестные поля молча возвращают дефолты (см. design). */
export function parsePreferences(raw: string | null): Preferences {
  if (!raw) return { ...DEFAULT_PREFERENCES };
  try {
    const data = JSON.parse(raw) as Partial<Preferences>;
    return {
      fontId: FONTS.some(([id]) => id === data.fontId) ? data.fontId! : DEFAULT_PREFERENCES.fontId,
      letterCase: data.letterCase === 'upper' ? 'upper' : 'lower',
      methodologySeen: data.methodologySeen === true,
    };
  } catch {
    return { ...DEFAULT_PREFERENCES };
  }
}

export function createPreferencesStore(storage: Storage | null): PreferencesStore {
  return {
    load() {
      if (!storage) return { ...DEFAULT_PREFERENCES };
      try {
        return parsePreferences(storage.getItem(KEY));
      } catch {
        return { ...DEFAULT_PREFERENCES };
      }
    },
    save(prefs) {
      if (!storage) return false;
      try {
        storage.setItem(KEY, JSON.stringify(prefs));
        return true;
      } catch {
        return false;
      }
    },
  };
}
