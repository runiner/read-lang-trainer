// Уровень «текст»: выбор произведения, валидация размера, порядок частей, заголовок.
import { beforeAll, beforeEach, describe, expect, it, vi } from 'vitest';
import fs from 'node:fs';
import path from 'node:path';

const wasmBytes = fs.readFileSync(path.resolve(process.cwd(), 'pkg/rus_wasm_bg.wasm'));

beforeAll(() => {
  vi.doMock('../pkg/rus_wasm.js', async () => {
    const real = await vi.importActual<typeof import('../pkg/rus_wasm.js')>('../pkg/rus_wasm.js');
    return { ...real, default: async () => real.initSync({ module: new Uint8Array(wasmBytes.buffer.slice(0)) }) };
  });
});

beforeEach(() => {
  document.body.innerHTML = '<div id="app"></div>';
  localStorage.clear();
  delete document.documentElement.dataset.font;
});

const q = <T extends Element = HTMLElement>(sel: string) => document.querySelector<T>(sel) as any;

async function boot() {
  vi.resetModules();
  await import('./main.js');
}

describe('story level', () => {
  it('row hidden for other levels, appears with six options incl. текст', async () => {
    await boot();
    expect(q('#story-row').hasAttribute('hidden')).toBe(true);
    const opts = [...q<HTMLSelectElement>('#level').options].map((o: HTMLOptionElement) => o.value);
    expect(opts).toEqual(['vowels', 'simple-syllables', 'syllables', 'words', 'phrases', 'text']);
  });

  it('start without story blocked; size>maxParts blocked with hint; valid start walks parts in order', async () => {
    await boot();
    const level = q<HTMLSelectElement>('#level');
    level.value = 'text';
    level.dispatchEvent(new Event('change', { bubbles: true }));
    expect(q('#story-row').hasAttribute('hidden')).toBe(false);
    expect([...q('#story').options].length).toBeGreaterThanOrEqual(2);

    // размер 50 у короткой истории → подсказка и старт блокируется
    q('#size').value = '50';
    q('#story').value = 'martyshka-i-ochki'; // maxParts = 5
    q('#story').dispatchEvent(new Event('change', { bubbles: true }));
    q('#start').click();
    expect(document.querySelector('.exercise')).toBeNull();
    expect(q('#story-hint').textContent).toContain('максимум 5');

    // корректный старт: части идут по порядку, заголовок с названием
    q('#size').value = '5';
    q('#size').dispatchEvent(new Event('input', { bubbles: true }));
    q('#start').click();
    expect(q('.story-title').textContent).toContain('Мартышка и очки');
    expect(q('.progress').textContent).toBe('1 / 5');
    const first = q('.exercise .text').textContent as string;
    expect(first.startsWith('Мартышка к старости')).toBe(true);
    q('#exercise-area').click();
    expect(q('.progress').textContent).toBe('2 / 5');
    expect(q('.exercise .text').textContent).not.toBe(first);
  });

  it('stats record keeps level text', async () => {
    await boot();
    const level = q<HTMLSelectElement>('#level');
    level.value = 'text';
    level.dispatchEvent(new Event('change', { bubbles: true }));
    q('#size').value = '5';
    q('#story').value = 'martyshka-i-ochki';
    q('#start').click();
    await vi.waitFor(() => {
      for (let i = 0; i < 5; i++) q('#exercise-area')?.click();
    });
    const raw = JSON.parse(localStorage.getItem('rus-trainer.sessions')!);
    expect(raw[0].level).toBe('text');
    expect(raw[0].count).toBe(5);
  });
});
