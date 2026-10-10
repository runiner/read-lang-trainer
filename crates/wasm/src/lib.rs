use rus_core::{Level, Session};
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
        _ => Level::SimpleSyllables,
    }
}

#[wasm_bindgen]
pub fn create_session(level: &str, size: u32, seed: f64) -> WasmSession { WasmSession(Session::new(level_from_str(level), size as usize, seed as u64)) }

#[allow(dead_code)]
fn _unused_create_session(level: &str, size: usize, seed: u64) -> WasmSession {
    WasmSession(Session::new(level_from_str(level), size, seed))
}

#[wasm_bindgen]
impl WasmSession {
    /// JSON: {"position":i,"text":"...","total":N} или null после конца.
    pub fn current_json(&self) -> Option<String> {
        let (pos, text) = self.0.current()?;
        Some(format!(
            "{{\"position\":{},\"text\":\"{}\",\"total\":{}}}",
            pos,
            text,
            self.0.total()
        ))
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
