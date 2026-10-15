# Tasks

## 1. Версия контента в core

- [x] 1.1 Добавить в `crates/core` зависимость `sha2`; в `content_data.rs` реализовать `pub fn content_version() -> &'static str` — hex SHA-256 от конкатенации байтов `include_bytes!` levels.json и stories.json (фиксированный разделитель, порядок файлов зафиксирован); unit-тест: функция детерминированна и имеет форму 64 hex-символов; `cargo test -p rus-core` зелёные

## 2. Экспорт через фасад

- [x] 2.1 `crates/wasm`: `#[wasm_bindgen] pub fn content_version() -> String` (прокидывает core); пересобрать pkg (`npm --prefix web run build:wasm`), добавить функцию в `web/src/wasm.ts`; smoke в node: значение совпадает с посчитанным локально скриптом/one-liner'ом от файлов

## 3. Проверка свежести

- [x] 3.1 Создать `web/src/freshness.test.ts` (vitest): считает SHA-256 от `../content/levels.json` + `../content/stories.json` тем же алгоритмом через `node:crypto`, сравнивает с `contentVersion()` из собранного pkg; при расхождении сообщение содержит обе версии и подсказку «выполните npm --prefix web run build:wasm»; проверить негативный путь экспериментом (правка файла без пересборки → тест красный, откатить + пересобрать → зелёный)
- [x] 3.2 Убедиться, что CI покрывается существующим шагом: `npm run test` запускается после pretest-пересборки pkg в workflow (прочитать deploy.yml, при необходимости добавить комментарий-указатель на шаг сборки wasm); отдельный шаг не создавать

## 4. Документация и интеграция

- [x] 4.1 README («Как добавить упражнение»): одна строка про версию контента и то, что несоответствие pkg↔файлов ловит `npm --prefix web run test`; зафиксировать поведение reformat-чувствительности («изменение байтов = новая версия»)
- [x] 4.2 Финальный прогон: `cargo test --workspace`, `npm --prefix web run test` (включая freshness), `npm run build`, `npx @fission-ai/openspec validate wasm-artifact-freshness` зелёные
