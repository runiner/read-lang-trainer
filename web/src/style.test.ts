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

  it('interface is static: no transitions or animations', () => {
    expect(css).not.toMatch(/transition\s*:/);
    expect(css).not.toMatch(/animation\s*:/);
    expect(css).not.toContain('@keyframes');
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
