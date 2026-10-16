// Защитники визуального слоя (openspec: ui-presentation).
import { describe, expect, it } from 'vitest';
import fs from 'node:fs';
import path from 'node:path';

const css = fs.readFileSync(path.resolve(process.cwd(), 'src/style.css'), 'utf8');

describe('design tokens layer', () => {
  it('defines the palette and scale tokens', () => {
    for (const t of ['--bg:', '--surface:', '--border:', '--text:', '--muted:', '--accent:', '--warn:', '--space-1:', '--radius:', '--tap-target:', '--fs-hero:']) {
      expect(css).toContain(t);
    }
  });

  it('no hex colors outside :root token block', () => {
    const rootEnd = css.indexOf('}', css.indexOf(':root'));
    const rest = css.slice(rootEnd);
    // белый текст на акцентной кнопке — единственное разрешённое исключение
    const offenders = rest.split('\n').filter((l) => /#[0-9a-fA-F]{3,8}\b/.test(l) && !l.includes('#fff;'));
    expect(offenders).toEqual([]);
  });

  it('base screens are static; motion only inside the celebration overlay', () => {
    // вырезаем блок празднования (единственное разрешённое исключение по ui-presentation)
    const cut = css.split('/* Празднование')[0];
    expect(cut).not.toMatch(/transition\s*:/);
    expect(cut).not.toMatch(/animation\s*:/);
    expect(cut).not.toContain('@keyframes');
    // а внутри праздника анимации привязаны только к оверлею/щенку/похвале
    const party = css.slice(css.indexOf('/* Празднование'));
    for (const line of party.split('\n')) {
      if (/animation:|@keyframes/.test(line)) {
        expect(line + ' ok').toMatch(/celebration-overlay|puppy-bounce|praise-pop|animation: none/);
      }
    }
  });

  it('keyboard focus is visibly styled', () => {
    expect(css).toMatch(/focus-visible\s*\{[^}]*outline/);
  });

  it('tap targets use the token scale', () => {
    expect(css).toMatch(/min-height:\s*var\(--tap-target\)/);
  });

  it('readonly field looks inactive but readable', () => {
    expect(css).toMatch(/\[readonly\]\s*\{[^}]*background:\s*var\(--bg\)/);
  });
});
