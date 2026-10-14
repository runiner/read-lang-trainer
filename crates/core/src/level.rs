use serde::Serialize;

/// Уровни сложности, в порядке от простого к сложному.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Level {
    Vowels,
    SimpleSyllables,
    Syllables,
    Words,
    Phrases,
    Text,
}

impl Level {
    pub const ALL: [Level; 6] = [
        Level::Vowels,
        Level::SimpleSyllables,
        Level::Syllables,
        Level::Words,
        Level::Phrases,
        Level::Text,
    ];

    /// Русское название уровня для UI.
    pub fn title(self) -> &'static str {
        match self {
            Level::Vowels => "гласные",
            Level::SimpleSyllables => "простые слоги",
            Level::Syllables => "слоги",
            Level::Words => "слова",
            Level::Phrases => "фразы",
            Level::Text => "текст",
        }
    }

    /// Ключ уровня в content/levels.json (для Text — None: источник — stories.json).
    pub fn json_key(self) -> Option<&'static str> {
        match self {
            Level::Vowels => Some("vowels"),
            Level::SimpleSyllables => Some("simple-syllables"),
            Level::Syllables => Some("syllables"),
            Level::Words => Some("words"),
            Level::Phrases => Some("phrases"),
            Level::Text => None,
        }
    }

    pub fn examples(self) -> &'static [String] {
        match self.json_key() {
            Some(key) => super::content_data::level_examples(key),
            None => &[],
        }
    }
}

/// Один пример для чтения вслух.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Example {
    pub text: String,
    pub level: Level,
}
