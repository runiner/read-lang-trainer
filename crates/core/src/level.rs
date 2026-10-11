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

    pub fn examples(self) -> &'static [&'static str] {
        match self {
            Level::Vowels => &super::content::VOWELS,
            Level::SimpleSyllables => &super::content::SIMPLE_SYLLABLES,
            Level::Syllables => &super::content::SYLLABLES,
            Level::Words => &super::content::WORDS,
            Level::Phrases => &super::content::PHRASES,
            // «текст» планируется через stories.rs, плоского списка примеров нет.
            Level::Text => &[],
        }
    }
}

/// Один пример для чтения вслух.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Example {
    pub text: String,
    pub level: Level,
}
