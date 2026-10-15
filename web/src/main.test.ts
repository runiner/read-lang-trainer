// Интеграционный прогон полного цикла на jsdom: настройки → упражнения (пробел/тап) → статистика → история.
import { beforeAll, beforeEach, describe, expect, it, vi } from 'vitest';

beforeAll(async () => {
  const fs = await import('node:fs');
  // main.js сам инициализирует wasm через fetch(URL) — в jsdom это не работает,
  // поэтому подменяем модуль pkg на sync-инициализацию.
  const path = await import('node:path');
  const wasmPath = path.resolve(process.cwd(), 'pkg/rus_wasm_bg.wasm');
  const bytes = new Uint8Array(fs.readFileSync(wasmPath));
  vi.doMock('../pkg/rus_wasm.js', async () => {
    const real = await vi.importActual<typeof import('../pkg/rus_wasm.js')>('../pkg/rus_wasm.js');
    return { ...real, default: async () => real.initSync({ module: bytes.slice() }) };
  });
});

beforeEach(() => {
  document.body.innerHTML = '<div id="app"></div>';
  localStorage.clear();
});

const q = <T extends Element = HTMLElement>(sel: string) => document.querySelector<T>(sel) as any;

describe('session flow end-to-end', () => {
  it('settings -> exercises -> stats -> history after reload', async () => {
    await import('./main.js');

    // экран настроек с дефолтами
    expect(q('#size').value).toBe('10');
    expect(q<HTMLSelectElement>('#level').selectedOptions[0].textContent).toBe('простые слоги');

    // значение вне диапазона не принимается, сессия не стартует
    q('#size').value = '3';
    q('#start').click();
    expect(q('#size-hint').hasAttribute('hidden')).toBe(false);
    expect(document.querySelector('.exercise')).toBeNull();

    // корректные настройки стартуют сессию
    q('#size').value = '5';
    q('#level').value = 'vowels';
    q('#start').click();
    expect(q('.exercise .text').textContent).toBeTruthy();
    expect(q('.progress').textContent).toBe('1 / 5');

    // пробел и тап продвигают
    document.dispatchEvent(new KeyboardEvent('keydown', { code: 'Space', bubbles: true }));
    expect(q('.progress').textContent).toBe('2 / 5');
    q('#next').click(); // клик по кнопке внутри области — ровно один переход
    expect(q('.progress').textContent).toBe('3 / 5');
    q('#exercise-area').click();
    expect(q('.progress').textContent).toBe('4 / 5');

    // оставшиеся упражнения — последний переход открывает статистику
    q('#exercise-area').click();
    q('#exercise-area').click();
    expect(document.querySelector('.stats')).not.toBeNull();
    expect(document.body.textContent).toContain('слов в минуту');

    // запись упала в localStorage со скоростью
    const raw = JSON.parse(localStorage.getItem('rus-trainer.sessions')!);
    expect(raw.length).toBe(1);
    expect(raw[0].count).toBe(5);
    expect(raw[0].words_per_min).toBeGreaterThan(0);

    // «перезагрузка»: новый импорт main видит историю
    vi.resetModules();
    await import('./main.js');
    expect(q('#history-block').textContent).toContain('История (1)');
  });
});
