// Свежесть собранного контента: pkg ↔ content/*.json (см. openspec exercise-content).
import { beforeAll, beforeEach, describe, expect, it, vi } from 'vitest';
import crypto from 'node:crypto';
import fs from 'node:fs';
import path from 'node:path';

const root = path.resolve(process.cwd(), '..');

/** Тот же алгоритм, что в rus_core::content_version: sha256(levels + "\n---\n" + stories). */
function expectedVersion(): string {
  const h = crypto.createHash('sha256');
  h.update(fs.readFileSync(path.join(root, 'content/levels.json')));
  h.update('\n---\n');
  h.update(fs.readFileSync(path.join(root, 'content/stories.json')));
  return h.digest('hex');
}

const wasmBytes = fs.readFileSync(path.resolve(process.cwd(), 'pkg/rus_wasm_bg.wasm'));

beforeAll(() => {
  vi.doMock('../pkg/rus_wasm.js', async () => {
    const real = await vi.importActual<typeof import('../pkg/rus_wasm.js')>('../pkg/rus_wasm.js');
    return { ...real, default: async () => real.initSync({ module: new Uint8Array(wasmBytes.buffer.slice(0)) }) };
  });
});

beforeEach(async () => {
  // инициализация одна на модуль pkg — гарантируем её до вызова экспортов
  const mod = await import('../pkg/rus_wasm.js');
  mod.initSync({ module: new Uint8Array(wasmBytes.buffer.slice(0)) });
});

describe('wasm artifact freshness', () => {
  it('pkg content version matches content documents on disk', async () => {
    const { contentVersion } = await import('./wasm.js');
    const inPkg = contentVersion();
    const onDisk = expectedVersion();
    expect(inPkg).toBe(onDisk); // при расхождении vitest печатает обе версии
    if (inPkg !== onDisk) {
      throw new Error(`pkg=${inPkg} files=${onDisk} — выполните npm --prefix web run build:wasm`);
    }
  });
});
