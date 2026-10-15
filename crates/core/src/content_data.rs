//! Контент упражнений живёт в JSON-документах репозитория (каталог `content/`).
//! Единственные места, где известен путь к файлам, — include_str! ниже.

use serde::Deserialize;
use std::collections::HashMap;
use std::sync::LazyLock;

const LEVELS_JSON: &str = include_str!("../../../content/levels.json");
const STORIES_JSON: &str = include_str!("../../../content/stories.json");
const LEVELS_BYTES: &[u8] = include_bytes!("../../../content/levels.json");
const STORIES_BYTES: &[u8] = include_bytes!("../../../content/stories.json");

/// Идентификатор версии контента: hex SHA-256 от байтов контентных документов
/// в фиксированном порядке с фиксированным разделителем. Меняется только вместе с файлами.
pub fn content_version() -> &'static str {
    static VERSION: LazyLock<String> = LazyLock::new(|| {
        use sha2::{Digest, Sha256};
        let mut h = Sha256::new();
        h.update(LEVELS_BYTES);
        h.update(b"\n---\n");
        h.update(STORIES_BYTES);
        h.finalize().iter().map(|b| format!("{b:02x}")).collect()
    });
    &VERSION
}

#[derive(Debug, Deserialize)]
pub struct StoryDoc {
    pub id: String,
    pub title: String,
    pub author: String,
    pub parts: Vec<String>,
}

pub static LEVELS: LazyLock<HashMap<String, Vec<String>>> = LazyLock::new(|| {
    serde_json::from_str(LEVELS_JSON).expect("content/levels.json: битый JSON или неверная схема")
});

pub static STORIES: LazyLock<Vec<StoryDoc>> = LazyLock::new(|| {
    serde_json::from_str(STORIES_JSON).expect("content/stories.json: битый JSON или неверная схема")
});

/// Примеры короткого уровня по id (`vowels`, `simple-syllables`, ...).
pub fn level_examples(id: &str) -> &'static [String] {
    let map: &'static HashMap<String, Vec<String>> = &LEVELS;
    map.get(id)
        .map(|v| v.as_slice())
        .unwrap_or_else(|| panic!("content/levels.json: нет уровня {id}"))
}

pub fn story_docs() -> &'static [StoryDoc] {
    &STORIES
}
