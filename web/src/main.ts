import './style.css';
import { initWasm, listStories, startSession, type LevelId, type SessionHandle, type StoryInfo } from './wasm.js';
import { createHistoryStore, tryLocalStorage } from './history.js';
import { createPreferencesStore, FONTS, type Preferences } from './preferences.js';
import { startCelebration } from './celebration.js';

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
  ['text', 'текст'],
];

let session: SessionHandle | null = null;
let shownAt = 0;
const stories: StoryInfo[] = [];

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
      <div class="card">
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
      <p class="hint" id="size-fixed-hint" hidden>Размер задаёт произведение.</p>
      <label for="level">Уровень сложности</label>
      <select id="level">
        ${LEVEL_OPTIONS.map(([id, t]) => `<option value="${id}"${id === 'simple-syllables' ? ' selected' : ''}>${t}</option>`).join('')}
      </select>
      <div id="story-row" hidden>
        <label for="story">Произведение</label>
        <select id="story"></select>
        <p class="hint" id="story-hint" hidden></p>
      </div>

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
                `<li>${new Date(r.date_ms).toLocaleDateString('ru-RU')} · ${LEVEL_OPTIONS.find(([id]) => id === r.level)?.[1] ?? r.level} · ${r.count} упр. · ${r.words_per_min ?? '—'} слов/мин</li>`,
            )
            .join('')}
        </ol>
      </details>
      </div>
    </section>`;
  const levelSel = app.querySelector<HTMLSelectElement>('#level')!;
  const storyRow = app.querySelector<HTMLElement>('#story-row')!;
  const storySel = app.querySelector<HTMLSelectElement>('#story')!;
  const storyHint = app.querySelector<HTMLElement>('#story-hint')!;
  // список строится при каждом рендере настроек: select пересоздаётся innerHTML'ом,
  // разовое наполнение под if теряло опции после возврата из сессии (баг).
  if (stories.length === 0) stories.push(...listStories());
  storySel.innerHTML =
    '<option value="" disabled selected hidden>— выберите —</option>' +
    stories.map((st) => `<option value="${st.id}">${st.title} — ${st.author}</option>`).join('');
  const sizeInput = app.querySelector<HTMLInputElement>('#size')!;
  const fixedHint = app.querySelector<HTMLElement>('#size-fixed-hint')!;
  function applySizeMode() {
    const isText = levelSel.value === 'text';
    storyRow.toggleAttribute('hidden', !isText);
    sizeInput.readOnly = isText;
    sizeInput.tabIndex = isText ? -1 : 0;
    sizeInput.setAttribute('aria-disabled', String(isText));
    fixedHint.toggleAttribute('hidden', !isText);
    if (isText) {
      const st = stories.find((x) => x.id === storySel.value);
      if (st) sizeInput.value = String(st.parts);
    }
  }
  levelSel.addEventListener('change', () => {
    applySizeMode();
    storyHint.toggleAttribute('hidden', true);
  });
  storySel.addEventListener('change', applySizeMode);
  applySizeMode();

  app.querySelector<HTMLButtonElement>('#start')!.addEventListener('click', () => {
    const sizeInput = app.querySelector<HTMLInputElement>('#size')!;
    const size = Number(sizeInput.value);
    if (!Number.isInteger(size) || size < 5 || size > 50) {
      app.querySelector('#size-hint')!.toggleAttribute('hidden', false);
      sizeInput.setAttribute('aria-invalid', 'true');
      return;
    }
    const level = levelSel.value as LevelId;
    if (level === 'text') {
      const st = stories.find((x) => x.id === storySel.value);
      if (!st) {
        storyHint.textContent = 'Выберите произведение.';
        storyHint.toggleAttribute('hidden', false);
        return;
      }
      // размер игнорируется core — там сколько частей в разметке
      startNewSession(level, st.parts, st.id);
      return;
    }
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

function startNewSession(level: LevelId, size: number, storyId = '') {
  const seed = Math.floor(Math.random() * 2 ** 32);
  session = startSession(level, size, seed, storyId);
  renderExercise();
}

function renderExercise() {
  const view = session!.current();
  if (!view) {
    renderStats();
    return;
  }
  shownAt = performance.now();
  const storyLine = view.story
    ? `<p class="story-title">«${view.story.title}» · ${view.story.author} · часть ${view.position} из ${view.total}</p>`
    : '';
  app.innerHTML = `
    <section class="screen exercise" id="exercise-area" tabindex="-1" aria-live="polite">
      ${storyLine}
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
      <div class="card">
      <ul>
        <li>Дата: ${new Date(record.date_ms).toLocaleString('ru-RU')}</li>
        <li>Уровень: ${LEVEL_OPTIONS.find(([id]) => id === record.level)?.[1] ?? record.level}</li>
        <li>Упражнений: ${record.count}</li>
        <li>Время: ${fmtDuration(record.duration_ms)}</li>
        <li>Скорость: ${record.words_per_min == null ? '—' : `${record.words_per_min} слов в минуту`}</li>
      </ul>
      ${saved ? '' : '<p class="warn">История не сохранена: хранилище недоступно.</p>'}
      <button id="again" class="primary big">В настройки</button>
      <button id="great" class="big great-btn">Отлично!</button>
      </div>
    </section>`;
  app.querySelector<HTMLButtonElement>('#again')!.addEventListener('click', renderSettings);
  app.querySelector<HTMLButtonElement>('#great')!.addEventListener('click', () => startCelebration(document.body));
}

await initWasm();
renderSettings();
