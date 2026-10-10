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
}

impl Level {
    pub const ALL: [Level; 5] = [
        Level::Vowels,
        Level::SimpleSyllables,
        Level::Syllables,
        Level::Words,
        Level::Phrases,
    ];

    /// Русское название уровня для UI.
    pub fn title(self) -> &'static str {
        match self {
            Level::Vowels => "гласные",
            Level::SimpleSyllables => "простые слоги",
            Level::Syllables => "слоги",
            Level::Words => "слова",
            Level::Phrases => "фразы",
        }
    }

    pub fn examples(self) -> &'static [&'static str] {
        match self {
            Level::Vowels => &super::content::VOWELS,
            Level::SimpleSyllables => &super::content::SIMPLE_SYLLABLES,
            Level::Syllables => &super::content::SYLLABLES,
            Level::Words => &super::content::WORDS,
            Level::Phrases => &super::content::PHRASES,
        }
    }
}

/// Один пример для чтения вслух.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Example {
    pub text: &'static str,
    pub level: Level,
}
