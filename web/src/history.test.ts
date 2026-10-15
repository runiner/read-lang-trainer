import { describe, it, expect } from 'vitest';
import { createHistoryStore, type HistoryStore } from './history.js';
import type { SessionRecord } from './wasm.js';

class FakeStorage implements Storage {
  private map = new Map<string, string>();
  get length() { return this.map.size; }
  clear(): void { this.map.clear(); }
  getItem(k: string) { return this.map.get(k) ?? null; }
  key(i: number) { return [...this.map.keys()][i] ?? null; }
  removeItem(k: string) { this.map.delete(k); }
  setItem(k: string, v: string) { this.map.set(k, String(v)); }
}

const rec = (n: number): SessionRecord => ({
  date_ms: n, level: 'words', count: 10, duration_ms: 60_000, words_per_min: n,
});

describe('history store', () => {
  it('saves and lists newest first with speed', () => {
    const s: HistoryStore = createHistoryStore(new FakeStorage());
    s.save(rec(1));
    s.save(rec(2));
    const list = s.list();
    expect(list.map((r) => r.words_per_min)).toEqual([2, 1]);
  });

  it('caps history at 500 records', () => {
    const st = new FakeStorage();
    const s = createHistoryStore(st);
    for (let i = 0; i < 505; i++) s.save(rec(i));
    expect(s.list().length).toBe(500);
  });

  it('survives reload via same storage', () => {
    const st = new FakeStorage();
    createHistoryStore(st).save(rec(7));
    expect(createHistoryStore(st).list()[0].words_per_min).toBe(7);
  });

  it('reports failure when storage unavailable', () => {
    const s = createHistoryStore(null);
    expect(s.save(rec(1))).toBe(false);
    expect(s.list()).toEqual([]);
  });

  it('reports failure when storage throws', () => {
    const bad = new FakeStorage();
    bad.setItem = () => { throw new Error('quota'); };
    expect(createHistoryStore(bad).save(rec(1))).toBe(false);
  });
});
