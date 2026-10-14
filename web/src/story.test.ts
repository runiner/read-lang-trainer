// Уровень «текст»: выбор произведения, readonly-размер из разметки, порядок частей, заголовок.
import { beforeAll, beforeEach, describe, expect, it, vi } from 'vitest';
import fs from 'node:fs';
import path from 'node:path';
// listStories читает уже инициализированный wasm — берём из модуля после boot.

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

  it('size field becomes readonly with part count; parts walk in order under story title', async () => {
    await boot();
    const level = q<HTMLSelectElement>('#level');
    level.value = 'text';
    level.dispatchEvent(new Event('change', { bubbles: true }));
    expect(q('#story-row').hasAttribute('hidden')).toBe(false);
    expect([...q('#story').options].length).toBeGreaterThanOrEqual(2);

    // поле размера неактивно и равно числу частей выбранного произведения
    const storySel = q<HTMLSelectElement>('#story');
    storySel.value = 'martyshka-i-ochki';
    storySel.dispatchEvent(new Event('change', { bubbles: true }));
    const size = q<HTMLInputElement>('#size');
    expect(size.getAttribute('readonly')).not.toBeNull();
    const expectedParts = Number(size.value); // поле заполнено из stories.json через фасад
    expect(expectedParts).toBeGreaterThanOrEqual(2); // инвариант документа, конкретное число не фиксируем

    // попытка задать своё число игнорируется: сессия из всех частей по порядку
    size.value = '50';
    q('#start').click();
    expect(q('.story-title').textContent).toContain('Мартышка и очки');
    expect(q('.progress').textContent).toBe(`1 / ${expectedParts}`);
    const first = q('.exercise .text').textContent as string;
    expect(first.startsWith('Мартышка к старости')).toBe(true);
    q('#exercise-area').click();
    expect(q('.progress').textContent).toBe(`2 / ${expectedParts}`);
    expect(q('.exercise .text').textContent).not.toBe(first);
  });

  it('start without chosen story is blocked with hint', async () => {
    await boot();
    const level = q<HTMLSelectElement>('#level');
    level.value = 'text';
    level.dispatchEvent(new Event('change', { bubbles: true }));
    const storySel = q<HTMLSelectElement>('#story');
    storySel.value = '';
    q('#start').click();
    expect(document.querySelector('.exercise')).toBeNull();
    expect(q('#story-hint').textContent).toContain('Выберите произведение');
  });

  it('stats record keeps level text and manual part count', async () => {
    await boot();
    const level = q<HTMLSelectElement>('#level');
    level.value = 'text';
    level.dispatchEvent(new Event('change', { bubbles: true }));
    const storySel = q<HTMLSelectElement>('#story');
    storySel.value = 'martyshka-i-ochki';
    storySel.dispatchEvent(new Event('change', { bubbles: true }));
    const size2 = q<HTMLInputElement>('#size');
    const total = Number(size2.value); // размер сессии = число частей из stories.json
    expect(total).toBeGreaterThanOrEqual(2);
    q('#start').click();
    for (let i = 0; i < total; i++) {
      await vi.waitFor(() => expect(q('#exercise-area')).toBeTruthy());
      q('#exercise-area').click(); // последнее открывает статистику
    }
    await vi.waitFor(() => expect(document.querySelector('.stats')).toBeTruthy());
    const raw = JSON.parse(localStorage.getItem('rus-trainer.sessions')!);
    expect(raw[0].level).toBe('text');
    expect(raw[0].count).toBe(total);
  });
});
