use rus_core::{story_sentence_count, Level, Session, STORIES};
use wasm_bindgen::prelude::*;

/// Обёртка сессии для JS: одно упражнение = один объект.
#[wasm_bindgen]
pub struct WasmSession(Session);

fn level_from_str(s: &str) -> Level {
    match s {
        "vowels" => Level::Vowels,
        "simple-syllables" => Level::SimpleSyllables,
        "syllables" => Level::Syllables,
        "words" => Level::Words,
        "phrases" => Level::Phrases,
        "text" => Level::Text,
        _ => Level::SimpleSyllables,
    }
}

#[wasm_bindgen]
pub fn create_session(level: &str, size: u32, seed: f64, story_id: &str) -> WasmSession {
    WasmSession(Session::with_story(
        level_from_str(level),
        size as usize,
        seed as u64,
        story_id,
    ))
}

/// Список произведений уровня «текст»: JSON [{"id","title","author","maxParts"}].
#[wasm_bindgen]
pub fn stories_json() -> String {
    let items: Vec<String> = STORIES
        .iter()
        .map(|st| {
            format!(
                "{{\"id\":\"{}\",\"title\":\"{}\",\"author\":\"{}\",\"maxParts\":{}}}",
                st.id,
                st.title,
                st.author,
                story_sentence_count(st.id)
            )
        })
        .collect();
    format!("[{}]", items.join(","))
}

#[wasm_bindgen]
impl WasmSession {
    /// JSON: {"position":i,"text":"...","total":N,"story":{"title","author"}?} или null.
    pub fn current_json(&self) -> Option<String> {
        let (pos, text) = self.0.current()?;
        #[derive(serde::Serialize)]
        struct Cur<'a> {
            position: usize,
            text: &'a str,
            total: usize,
            story: Option<StoryRef>,
        }
        #[derive(serde::Serialize)]
        struct StoryRef {
            title: &'static str,
            author: &'static str,
        }
        let cur = Cur {
            position: pos,
            text,
            total: self.0.total(),
            story: self.0.story_title().map(|(title, author)| StoryRef { title, author }),
        };
        serde_json::to_string(&cur).ok()
    }

    pub fn next(&mut self, elapsed_ms: u32) -> bool {
        self.0.next(elapsed_ms as u64)
    }

    pub fn finished(&self) -> bool {
        self.0.finished()
    }

    /// JSON SessionRecord для localStorage.
    pub fn finish(&self, date_ms: f64) -> String {
        serde_json::to_string(&self.0.finish(date_ms as u64)).expect("record serializes")
    }
}

#[wasm_bindgen]
pub fn levels() -> Vec<String> {
    Level::ALL.iter().map(|l| l.title().to_string()).collect()
}
