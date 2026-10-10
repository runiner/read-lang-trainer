import './style.css';
import { initWasm, startSession, type LevelId, type SessionHandle } from './wasm.js';
import { createHistoryStore, tryLocalStorage } from './history.js';
import { createPreferencesStore, FONTS, type Preferences } from './preferences.js';

const app = document.querySelector<HTMLElement>('#app')!;
const storage = tryLocalStorage();
const store = createHistoryStore(storage);
const prefsStore = createPreferencesStore(storage);
let prefs: Preferences = prefsStore.load();

/** Применить предпочтения к документу: шрифт на <html>, регистр — классом на область текста. */
function applyPrefs() {
  document.documentElement.dataset.font = prefs.fontId;
}

const LEVEL_OPTIONS: [LevelId, string][] = [
  ['vowels', 'гласные'],
  ['simple-syllables', 'простые слоги'],
  ['syllables', 'слоги'],
  ['words', 'слова'],
  ['phrases', 'фразы'],
];

let session: SessionHandle | null = null;
let shownAt = 0;

function fmtDuration(ms: number): string {
  const s = Math.round(ms / 1000);
  return s < 60 ? `${s} с` : `${Math.floor(s / 60)} мин ${s % 60} с`;
}

function renderSettings() {
  const history = store.list();
  applyPrefs();
  app.innerHTML = `
    <section class="screen settings">
      <h1>Тренажёр чтения</h1>
      <details id="methodology"${prefs.methodologySeen ? '' : ' open'}>
        <summary>Методика</summary>
        <div class="methodology">
          <p>Программа показывает по одному тексту — от гласных до фраз. Ребёнок читает вслух; приложение не проверяет произношение.</p>
          <p><b>Ваша роль:</b> слушать чтение и подтверждать переход — пробелом, кнопкой «Далее» или тапом по экрану, когда прочитано.</p>
          <p><b>Уровень:</b> выбирайте самый простой, на котором ребёнок читает уверенно, и повышайте постепенно.</p>
        </div>
      </details>
      <label for="size">Упражнений в сессии (5–50)</label>
      <input id="size" type="number" min="5" max="50" step="1" value="10" />
      <p class="hint" id="size-hint" hidden>Допустимо целое число от 5 до 50.</p>
      <label for="level">Уровень сложности</label>
      <select id="level">
        ${LEVEL_OPTIONS.map(([id, t]) => `<option value="${id}"${id === 'simple-syllables' ? ' selected' : ''}>${t}</option>`).join('')}
      </select>
      <label for="font">Шрифт</label>
      <select id="font">
        ${FONTS.map(([id, t]) => `<option value="${id}"${id === prefs.fontId ? ' selected' : ''}>${t}</option>`).join('')}
      </select>
      <fieldset class="case-toggle">
        <legend>Буквы</legend>
        <label><input type="radio" name="case" value="lower"${prefs.letterCase === 'lower' ? ' checked' : ''} /> строчные</label>
        <label><input type="radio" name="case" value="upper"${prefs.letterCase === 'upper' ? ' checked' : ''} /> прописные</label>
      </fieldset>
      <p class="preview exercise-sample" id="preview"><span class="text case-${prefs.letterCase}">мама мыла раму</span></p>
      <button id="start" class="primary big">Начать</button>
      <details id="history-block">
        <summary>История (${history.length})</summary>
        <ol id="history-list">
          ${history
            .map(
              (r) =>
                `<li>${new Date(r.date_ms).toLocaleDateString('ru-RU')} · ${LEVEL_OPTIONS.find(([id]) => id === r.level)?.[1] ?? r.level} · ${r.count} упр. · ${r.chars_per_min ?? '—'} симв/мин</li>`,
            )
            .join('')}
        </ol>
      </details>
    </section>`;
  app.querySelector<HTMLButtonElement>('#start')!.addEventListener('click', () => {
    const sizeInput = app.querySelector<HTMLInputElement>('#size')!;
    const size = Number(sizeInput.value);
    if (!Number.isInteger(size) || size < 5 || size > 50) {
      app.querySelector('#size-hint')!.toggleAttribute('hidden', false);
      sizeInput.setAttribute('aria-invalid', 'true');
      return;
    }
    const level = app.querySelector<HTMLSelectElement>('#level')!.value as LevelId;
    startNewSession(level, size);
  });

  app.querySelector<HTMLSelectElement>('#font')!.addEventListener('change', (e) => {
    prefs.fontId = (e.target as HTMLSelectElement).value;
    prefsStore.save(prefs);
    applyPrefs();
  });
  app.querySelectorAll<HTMLInputElement>('input[name="case"]').forEach((radio) => {
    radio.addEventListener('change', () => {
      prefs.letterCase = radio.value as Preferences['letterCase'];
      prefsStore.save(prefs);
      const sample = app.querySelector('.exercise-sample')!;
      sample.className = 'preview exercise-sample';
      sample.querySelector('.text')!.className = `text case-${prefs.letterCase}`;
    });
  });
  const methodology = app.querySelector<HTMLDetailsElement>('#methodology')!;
  methodology.addEventListener('toggle', () => {
    if (!methodology.open && !prefs.methodologySeen) {
      prefs.methodologySeen = true;
      prefsStore.save(prefs);
    }
  });
}

function startNewSession(level: LevelId, size: number) {
  const seed = Math.floor(Math.random() * 2 ** 32);
  session = startSession(level, size, seed);
  renderExercise();
}

function renderExercise() {
  const view = session!.current();
  if (!view) {
    renderStats();
    return;
  }
  shownAt = performance.now();
  app.innerHTML = `
    <section class="screen exercise" id="exercise-area" tabindex="-1" aria-live="polite">
      <p class="progress">${view.position} / ${view.total}</p>
      <p class="text case-${prefs.letterCase}">${view.text}</p>
      <button id="next" class="primary big">Далее</button>
    </section>`;
  // Один обработчик на область: клик по кнопке всплывает сюда же — двойного перехода нет.
  app.querySelector('#exercise-area')!.addEventListener('click', advance);
}

function advance() {
  if (!session) return;
  const elapsed = Math.round(performance.now() - shownAt);
  if (!session.next(elapsed)) {
    renderStats();
    return;
  }
  renderExercise();
}

document.addEventListener('keydown', (e) => {
  if (e.code !== 'Space' || !session || session.finished()) return;
  const el = document.activeElement;
  if (el && (el.tagName === 'INPUT' || el.tagName === 'SELECT' || el.tagName === 'TEXTAREA')) return;
  e.preventDefault();
  advance();
});

function renderStats() {
  const record = session!.finish();
  const saved = store.save(record);
  session!.free();
  session = null;
  app.innerHTML = `
    <section class="screen stats">
      <h1>Готово!</h1>
      <ul>
        <li>Дата: ${new Date(record.date_ms).toLocaleString('ru-RU')}</li>
        <li>Уровень: ${LEVEL_OPTIONS.find(([id]) => id === record.level)?.[1] ?? record.level}</li>
        <li>Упражнений: ${record.count}</li>
        <li>Время: ${fmtDuration(record.duration_ms)}</li>
        <li>Скорость: ${record.chars_per_min === null ? '—' : `${record.chars_per_min} символов в минуту`}</li>
      </ul>
      ${saved ? '' : '<p class="warn">История не сохранена: хранилище недоступно.</p>'}
      <button id="again" class="primary big">В настройки</button>
    </section>`;
  app.querySelector<HTMLButtonElement>('#again')!.addEventListener('click', renderSettings);
}

await initWasm();
renderSettings();
