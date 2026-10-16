// Структурные контракты празднования (движение/WebAudio в jsdom не проверяются).
import { beforeEach, describe, expect, it, vi } from 'vitest';
import { startCelebration } from './celebration.js';

beforeEach(() => {
  document.body.innerHTML = '';
  localStorage.clear();
  vi.stubGlobal('AudioContext', undefined); // музыка молчит, ветка тишины
});

const reduced = (matches: boolean) =>
  vi.stubGlobal('matchMedia', vi.fn().mockImplementation((q: string) => ({ matches: q.includes('reduced-motion') && matches, addEventListener: () => {}, removeEventListener: () => {} })));

describe('celebration overlay', () => {
  it('creates overlay with puppy and praise, auto-closes after duration', () => {
    reduced(false);
    let done: (() => void) | null = null;
    const h = startCelebration(document.body, { durationMs: 12_000, schedule: (fn) => { done = fn; return 1; } });
    const o = document.getElementById('celebration');
    expect(o).not.toBeNull();
    expect(o!.querySelector('#puppy')).not.toBeNull();
    expect(o!.textContent).toContain('Молодец!');
    done!();
    expect(document.getElementById('celebration')).toBeNull();
    h.stop(); // идемпотентно
  });

  it('any pointerdown interrupts immediately', () => {
    reduced(false);
    startCelebration(document.body, { schedule: () => 1 });
    window.dispatchEvent(new PointerEvent('pointerdown'));
    expect(document.getElementById('celebration')).toBeNull();
  });

  it('keydown interrupts too', () => {
    reduced(false);
    startCelebration(document.body, { schedule: () => 1 });
    window.dispatchEvent(new KeyboardEvent('keydown'));
    expect(document.getElementById('celebration')).toBeNull();
  });

  it('restart works after stop', () => {
    reduced(false);
    const h = startCelebration(document.body, { schedule: () => 1 });
    h.stop();
    const h2 = startCelebration(document.body, { schedule: () => 1 });
    expect(document.querySelectorAll('#celebration').length).toBe(1);
    h2.stop();
  });

  it('reduced motion uses static puppy, no canvas animation class', () => {
    reduced(true);
    const h = startCelebration(document.body, { schedule: () => 1 });
    const o = document.getElementById('celebration')!;
    expect(o.classList.contains('static')).toBe(true);
    const src = (o.querySelector('#puppy') as HTMLImageElement).getAttribute('src')!;
    expect(src.startsWith('./')).toBe(true); // не '/puppy/…': иначе 404 на GitHub Pages в подпути
    expect(src).toContain('static.svg');
    h.stop();
  });
});
