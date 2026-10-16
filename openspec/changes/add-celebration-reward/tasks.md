# Tasks

## 1. Ассеты щенка

- [x] 1.1 Нарисовать и положить `web/public/puppy/frame-{1,2,3}.svg` (покадровая «пляска»: наклон/подскок/поворот) и `static.svg` (для reduced-motion); оригинальный жёлтый пёс в красной каске, без узнаваемых элементов «Щенячьего патруля»; проверить: все файлы — валидный SVG, открываются в браузере по dev-URL

## 2. Музыка (WebAudio)

- [x] 2.1 Создать `web/src/music.ts`: `playCheerMusic(): { stop(): void }` — секвенсор ~120 BPM (бас + арпеджио, квадратные волны), fade-in 300ms, заложенный fade-out к концу; контекст создаётся лениво, ошибка WebAudio = тишина без падения приложения; unit-тест с мокным AudioContext: вызывается start, stop останавливает узлы и закрывает контекст

## 3. Оверлей празднования

- [x] 3.1 Создать `web/src/celebration.ts`: `startCelebration(root, { durationMs = 12_000, schedule, now })` — оверлей `#celebration` с canvas-конфетти (rAF, ~80 частиц), меняющим кадры щенком (setInterval 150ms → frame-1..3), надписью «Молодец!»; автозавершение по durationMs, прерывание pointerdown/keydown (single-shot слушатели на window), `stop()` идемпотентен; prefers-reduced-motion → статичный static.svg без rAF/смены кадров; vitest (fake timers + mock matchMedia): появление оверлея, закрытие по таймеру, закрытие по клику, повторный старт, ветка reduced-motion
- [x] 3.2 Стили оверлея в `web/src/style.css`: блок `.celebration-overlay …` (fixed, z-index поверх кнопок, pointer-events auto) и его keyframes/transition — единственное место анимаций; перенастроить `style.test.ts`: запрещённые `transition:/animation:/@keyframes` допускаются только внутри селекторов `.celebration-overlay`/`#celebration`, базовые экраны как прежде; тест зелёный

## 4. Кнопка и интеграция

- [x] 4.1 `renderStats()`: добавить `<button id="great" class="big great-btn">Отлично!</button>` под `#again` (вторичный стиль — акцент-обводка), обработчик запускает celebration + музыку, повторный клик после завершения работает; обновить jsdom-тесты экрана статистики (main/story): кнопка есть, клик создаёт/удаляет `#celebration`; vitest полностью зелёный
- [ ] 4.2 Ручной прогон в dev: полный цикл до статистики → «Отлично!» → 12 секунд конфетти/щенок/музыка → автовозврат; тап раньше завершает; повторный запуск; режим reduced-motion в OS-эмуляции Chrome; мобильная эмуляция 360px (оверлей не режет скролл); при необходимости — правки числа частиц/таймингов
- [x] 4.3 Финально: `cargo test --workspace` (не затронут), `npm --prefix web run test`, `npx tsc --noEmit`, `npm run build`, `npx @fission-ai/openspec validate add-celebration-reward` зелёные; README: раздел «Празднование» (кнопка, длительность, reduced-motion)
