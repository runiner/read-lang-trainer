// Регистр и шрифт: трансформация только визуальная, статистика считает исходный текст.
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

it('style.css keeps the uppercase rule scoped to .case-upper', () => {
  const css = fs.readFileSync(path.resolve(process.cwd(), 'src/style.css'), 'utf8');
  expect(css).toMatch(/\.case-upper[\s\S]*?text-transform:\s*uppercase/);
});

describe('presentation settings', () => {
  it('defaults: first font, lowercase, methodology open on first run', async () => {
    await boot();
    expect(document.documentElement.dataset.font).toBe('system');
    expect((q('#methodology') as HTMLDetailsElement).open).toBe(true);
    expect(q('#preview .text').className).toBe('text case-lower');
  });

  it('switching to upper shows capital letters in preview and exercises', async () => {
    await boot();
    const upper = q<HTMLInputElement>('input[name="case"][value="upper"]');
    upper.checked = true;
    upper.dispatchEvent(new Event('change', { bubbles: true }));
    expect(q('#preview .text').className).toBe('text case-upper');

    // старт сессии — текст упражнения тоже прописной (в DOM остаётся исходным)
    q('#size').value = '5';
    q('#level').value = 'vowels';
    q('#start').click();
    expect(q('.exercise .text').className).toBe('text case-upper');
    expect(q('.exercise .text').textContent).toBe(q('.exercise .text').textContent!.toLowerCase());
  });

  it('stats count original chars regardless of case', async () => {
    await boot();
    const upper = q<HTMLInputElement>('input[name="case"][value="upper"]');
    upper.checked = true;
    upper.dispatchEvent(new Event('change', { bubbles: true }));
    q('#size').value = '5';
    q('#level').value = 'vowels';
    q('#start').click();
    await vi.waitFor(() => {
      for (let i = 0; i < 5; i++) q('#exercise-area')?.click(); // последнее открывает статистику
    });
    const raw = JSON.parse(localStorage.getItem('rus-trainer.sessions')!);
    expect(raw[0].count).toBe(5);
    expect(raw[0].chars_per_min).toBeGreaterThan(0);
  });

  it('font change applies immediately via data-font', async () => {
    await boot();
    const sel = q<HTMLSelectElement>('#font');
    sel.value = 'serif';
    sel.dispatchEvent(new Event('change', { bubbles: true }));
    expect(document.documentElement.dataset.font).toBe('serif');
  });

  it('preferences persist across reload; size/level reset to defaults', async () => {
    await boot();
    const sel = q<HTMLSelectElement>('#font');
    sel.value = 'round';
    sel.dispatchEvent(new Event('change', { bubbles: true }));
    const upper = q<HTMLInputElement>('input[name="case"][value="upper"]');
    upper.checked = true;
    upper.dispatchEvent(new Event('change', { bubbles: true }));
    q('#size').value = '23';
    q('#level').value = 'words';
    (q('#methodology') as HTMLDetailsElement).open = false; // взрослый прочитал — сворачивает
    q('#methodology').dispatchEvent(new Event('toggle'));

    await boot(); // «перезагрузка»
    expect(document.documentElement.dataset.font).toBe('round');
    expect(q('#preview .text').className).toBe('text case-upper');
    expect(q('#size').value).toBe('10');
    expect(q<HTMLSelectElement>('#level').value).toBe('simple-syllables');
    expect((q('#methodology') as HTMLDetailsElement).open).toBe(false);
  });
});
