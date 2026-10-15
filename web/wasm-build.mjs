// Пересборка wasm-пакета с гарантией свежего pkg даже при сбое сборки.
// npm run dev/test/build вызывают этот скрипт (pretest/prebuild), поэтому pkg в web/pkg
// всегда либо пересобран из текущего контента, либо сборка падает явной ошибкой.
import { spawnSync } from 'node:child_process';
import { existsSync, mkdirSync, renameSync, rmSync } from 'node:fs';
import { fileURLToPath } from 'node:url';

const root = fileURLToPath(new URL('..', import.meta.url));
const out = `${root}/web/pkg`;
const tmp = `${root}/crates/wasm/pkg`;

if (!existsSync(`${root}/tools/wasm-pack`)) {
  console.error('Не найден ./tools/wasm-pack — см. README «Сборка».');
  process.exit(1);
}
const r = spawnSync('./tools/wasm-pack', ['build', 'crates/wasm', '-t', 'web'], { cwd: root, stdio: 'inherit' });
if (r.status !== 0) {
  // старый pkg остался на месте — проверка свежести (freshness.test.ts) поймёт это по версии контента
  console.error('wasm-pack build завершился с ошибкой — web/pkg не обновлён.');
  process.exit(r.status ?? 1);
}
rmSync(out, { recursive: true, force: true });
mkdirSync(`${root}/crates/wasm`, { recursive: true });
renameSync(tmp, out);
console.log(`web/pkg обновлён из ${tmp}`);
