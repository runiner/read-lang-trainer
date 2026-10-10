// Тесты предпочтений отображения: сохранение, восстановление, битые данные → дефолты.
import { describe, expect, it } from 'vitest';
import { createPreferencesStore, parsePreferences, DEFAULT_PREFERENCES } from './preferences.js';

class FakeStorage implements Storage {
  private map = new Map<string, string>();
  get length() { return this.map.size; }
  clear(): void { this.map.clear(); }
  getItem(k: string) { return this.map.get(k) ?? null; }
  key(i: number) { return [...this.map.keys()][i] ?? null; }
  removeItem(k: string) { this.map.delete(k); }
  setItem(k: string, v: string) { this.map.set(k, String(v)); }
}

describe('preferences', () => {
  it('defaults when nothing stored', () => {
    const s = createPreferencesStore(new FakeStorage());
    expect(s.load()).toEqual(DEFAULT_PREFERENCES);
  });

  it('saves and restores across reload (same storage)', () => {
    const st = new FakeStorage();
    createPreferencesStore(st).save({ fontId: 'serif', letterCase: 'upper', methodologySeen: true });
    expect(createPreferencesStore(st).load()).toEqual({
      fontId: 'serif', letterCase: 'upper', methodologySeen: true,
    });
  });

  it('unknown font or garbage falls back to defaults', () => {
    expect(parsePreferences('{"fontId":"comic","letterCase":"weird"}').fontId).toBe('system');
    expect(parsePreferences('{"fontId":"comic","letterCase":"weird"}').letterCase).toBe('lower');
    expect(parsePreferences('не json').fontId).toBe('system');
    expect(parsePreferences(null).methodologySeen).toBe(false);
  });

  it('reports failure when storage unavailable', () => {
    expect(createPreferencesStore(null).save(DEFAULT_PREFERENCES)).toBe(false);
  });
});
