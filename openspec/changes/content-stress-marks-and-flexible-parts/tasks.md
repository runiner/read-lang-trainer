# Tasks

## 1. Алфавит: ударные знаки

- [x] 1.1 В `crates/core/src/session.rs` расширить `is_allowed_char` комбинирующими `\u{300}`/`\u{301}`; добавить unit-тест «слово с ударением допустимо» (пример вида `"сло́во"` проходит проверку, `"сло\u{302}во"` — нет); `cargo test -p rus-core` зелёные на текущем контенте пользователя (с ударениями и Вороной 26 частей)

## 2. Тесты без привязки к данным

- [x] 2.1 Вынести Fisher-Yates-генерацию плана в приватную функцию поверх произвольного пула + `#[cfg(test)] plan_from_pool(&[&str], size, seed)`; перевести `plan_repeats_when_size_exceeds_pool` и `speed_is_chars_per_minute`/`zero_time_yields_no_speed` на синтетические пулы (без литералов «10 уникальных» из реального levels.json); поведение прод-пути не меняется — остальные тесты уровня проходят как раньше
- [x] 2.2 Удалить `part_counts_are_recorded`; убедиться grep'ом, что в тестах core не осталось литеральных количеств частей/примеров из реального контента; инвариант соответствия UI остаётся: тест `story_part_count(id) == story.parts.len()` для всех произведений (или существующий accessor-тест покрывает это)

## 3. Web-регрессия

- [x] 3.1 В `web/src/story.test.ts` заменить литерал «12» (частей martyshka) на значение из `listStories()` того же id (ожидание вычисляется из данных); vitest зелёные при текущем stories.json; `npx tsc --noEmit` чистый

## 4. Интеграция

- [x] 4.1 Доказать независимость от данных экспериментом в рабочей копии: временно добавить слово в `vowels` и ещё одно ударение в басню → `cargo test -p rus-core` и `npm --prefix web run test` остаются зелёными; откатить эксперимент; финально: `cargo test --workspace`, vitest, `npm run build`, `npx @fission-ai/openspec validate content-stress-marks-and-flexible-parts` зелёные
