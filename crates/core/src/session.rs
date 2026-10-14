use crate::level::{Example, Level};
use crate::content_data::{story_docs, StoryDoc};
use serde::Serialize;

/// Детерминированный ГПСЧ (xorshift64*), план сессии воспроизводится по seed.
struct Rng(u64);

impl Rng {
    fn next_u64(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.0 = x;
        x.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }
}

/// Порядок индексов примеров уровня для сессии размера `size`.
/// Fisher-Yates по уникальным примерам, затем цикл при нехватке.
/// Уровень Text строит план из частей произведения (упорядоченно, без RNG).
pub fn session_plan(level: Level, size: usize, seed: u64) -> Vec<Example> {
    debug_assert_ne!(level, Level::Text, "для текста используйте story_parts");
    plan_from(level, &level.examples().iter().map(String::as_str).collect::<Vec<_>>(), size, seed)
}

/// Общий алгоритм плана поверх произвольного пула (прод и тесты идут одну дорогу).
fn plan_from(level: Level, pool: &[&str], size: usize, seed: u64) -> Vec<Example> {
    assert!(!pool.is_empty() && size > 0, "пустая база или нулевой размер");
    let mut idx: Vec<usize> = (0..pool.len()).collect();
    let mut rng = Rng(seed | 1);
    for i in (1..idx.len()).rev() {
        let j = (rng.next_u64() % (i as u64 + 1)) as usize;
        idx.swap(i, j);
    }
    idx.iter()
        .cycle()
        .take(size)
        .map(|&i| Example {
            text: pool[i].to_string(),
            level,
        })
        .collect()
}

/// Тестовый конструктор: план на синтетическом пуле — проверки механики не зависят от контента.
#[cfg(test)]
fn plan_from_pool(pool: &[&str], size: usize, seed: u64) -> Vec<Example> {
    plan_from(Level::Vowels, pool, size, seed)
}

/// Найти произведение; пустой/неизвестный id → первое (страховка, UI обязан не допустить).
fn find_story(id: &str) -> &'static StoryDoc {
    story_docs().iter().find(|s| s.id == id).unwrap_or(&story_docs()[0])
}

/// Части произведения — из content/stories.json (один элемент массива = одна часть).
pub fn story_parts(story_id: &str) -> Vec<String> {
    find_story(story_id).parts.clone()
}

/// Сколько частей в произведении — это же размер сессии уровня Text.
pub fn story_part_count(id: &str) -> usize {
    find_story(id).parts.len()
}

/// План сессии уровня Text: все части произведения по порядку (size/seed не влияют).
pub fn text_plan(story_id: &str) -> Vec<Example> {
    story_parts(story_id)
        .into_iter()
        .map(|text| Example { text, level: Level::Text })
        .collect()
}

/// Итог завершённой сессии.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct SessionRecord {
    /// Unix-время завершения, мс.
    pub date_ms: u64,
    pub level: Level,
    pub count: usize,
    /// Суммарное время чтения всех упражнений, мс.
    pub duration_ms: u64,
    /// Символов в минуту; None, если суммарное время нулевое.
    pub chars_per_min: Option<u32>,
}

/// Проходимость сессии: курсор + накопители chars/elapsed.
pub struct Session {
    plan: Vec<Example>,
    story: Option<&'static StoryDoc>,
    cursor: usize,
    total_chars: u64,
    total_elapsed_ms: u64,
}

impl Session {
    /// Тестовый конструктор: готовый план без чтения контента.
    #[cfg(test)]
    pub fn from_plan(plan: Vec<Example>) -> Self {
        let total_chars = plan.iter().map(|e| e.text.chars().count() as u64).sum();
        Session { plan, story: None, cursor: 0, total_chars, total_elapsed_ms: 0 }
    }

    pub fn new(level: Level, size: usize, seed: u64) -> Self {
        Self::with_story(level, size, seed, "")
    }

    /// Заголовок-подпись для экрана упражнения: Some((название, автор)) у уровня Text.
    pub fn story_title(&self) -> Option<(&'static str, &'static str)> {
        self.story.map(|st| (st.title.as_str(), st.author.as_str()))
    }

    /// `story_id` значим только для Level::Text; пустой id → первая история (страховка).
    pub fn with_story(level: Level, size: usize, seed: u64, story_id: &str) -> Self {
        let (plan, story) = if level == Level::Text {
            // Размер задаёт разметка: size игнорируется.
            (text_plan(story_id), Some(find_story(story_id)))
        } else {
            (session_plan(level, size.max(1), seed), None)
        };
        let total_chars = plan.iter().map(|e| e.text.chars().count() as u64).sum();
        Session {
            plan,
            story,
            cursor: 0,
            total_chars,
            total_elapsed_ms: 0,
        }
    }

    /// Текущее упражнение (1-based позиция, текст). После конца — None.
    pub fn current(&self) -> Option<(usize, &str)> {
        self.plan
            .get(self.cursor)
            .map(|e| (self.cursor + 1, e.text.as_str()))
    }

    pub fn total(&self) -> usize {
        self.plan.len()
    }

    /// Переход к следующему упражнению; elapsed_ms — время на прочтение текущего.
    /// Возвращает false, если сессия уже завершена (лишних упражнений нет).
    pub fn next(&mut self, elapsed_ms: u64) -> bool {
        if self.finished() {
            return false;
        }
        self.total_elapsed_ms += elapsed_ms;
        self.cursor += 1;
        true
    }

    pub fn finished(&self) -> bool {
        self.cursor >= self.plan.len()
    }

    /// Записать итог. `date_ms` приходит из JS (Date.now), core не знает о часах.
    pub fn finish(&self, date_ms: u64) -> SessionRecord {
        let chars_per_min = if self.total_elapsed_ms == 0 {
            None
        } else {
            Some((self.total_chars * 60_000 / self.total_elapsed_ms) as u32)
        };
        SessionRecord {
            date_ms,
            level: self.plan[0].level,
            count: self.plan.len(),
            duration_ms: self.total_elapsed_ms,
            chars_per_min,
        }
    }
}

#[cfg(test)]
fn parse_levels(raw: &str) -> Result<std::collections::HashMap<String, Vec<String>>, serde_json::Error> {
    serde_json::from_str(raw)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plan_only_selected_level_and_size() {
        let plan = session_plan(Level::Vowels, 8, 42);
        assert_eq!(plan.len(), 8);
        assert!(plan.iter().all(|e| e.level == Level::Vowels));
    }

    #[test]
    fn plan_deterministic_by_seed() {
        let pa = session_plan(Level::Words, 12, 7);
        let pb = session_plan(Level::Words, 12, 7);
        let a: Vec<_> = pa.iter().map(|e| e.text.as_str()).collect();
        let b: Vec<_> = pb.iter().map(|e| e.text.as_str()).collect();
        assert_eq!(a, b);
    }

    #[test]
    fn plan_repeats_when_size_exceeds_pool() {
        // синтетический пул из 2 уникальных — механика повторов не зависит от levels.json
        let plan = plan_from_pool(&["а", "о"], 7, 99);
        assert_eq!(plan.len(), 7);
        let uniq: std::collections::HashSet<_> = plan.iter().map(|e| e.text.as_str()).collect();
        assert_eq!(uniq.len(), 2);
    }

    #[test]
    fn session_walks_to_end_and_stops() {
        let mut s = Session::new(Level::SimpleSyllables, 3, 5);
        assert_eq!(s.current(), Some((1, s.current().unwrap().1)));
        assert!(s.next(1000));
        assert!(s.next(1000));
        assert!(s.next(1000));
        assert!(s.finished());
        assert_eq!(s.current(), None);
        assert!(!s.next(1000), "нет упражнений сверх настройки");
    }

    #[test]
    fn speed_is_chars_per_minute() {
        // 10 односимвольных ПРИМЕРОВ из синтетики за 60 секунд → 10 симв/мин.
        let plan = plan_from_pool(&["а"], 10, 3);
        let mut s = Session::from_plan(plan);
        for _ in 0..10 {
            assert!(s.next(6_000));
        }
        let r = s.finish(0);
        assert_eq!(r.count, 10);
        assert_eq!(r.duration_ms, 60_000);
        assert_eq!(r.chars_per_min, Some(10));
    }

    #[test]
    fn zero_time_yields_no_speed() {
        let mut s = Session::from_plan(plan_from_pool(&["а", "б"], 2, 3));
        s.next(0);
        s.next(0);
        assert_eq!(s.finish(0).chars_per_min, None);
    }

    #[test]
    fn record_serializes_to_json() {
        let mut s = Session::new(Level::Words, 2, 3);
        s.next(500);
        s.next(500);
        let json = serde_json::to_string(&s.finish(1_700_000_000_000)).unwrap();
        assert!(json.contains("\"level\":\"words\""));
        assert!(json.contains("chars_per_min"));
    }
    fn is_allowed_char(c: char) -> bool {
        ('\u{410}'..='\u{44F}').contains(&c)
            || c == '\u{401}' || c == '\u{451}'
            || c.is_whitespace()
            || c == '\u{300}' || c == '\u{301}' // комбинирующие ударные знаки в словах
            || matches!(c, '.' | ',' | '!' | '?' | ';' | ':' | '-' | '(' | ')' | '\u{00AB}' | '\u{00BB}' | '\u{2014}' | '\u{2026}' | '0'..='9')
    }

    #[test]
    fn stress_marks_allowed_others_not() {
        assert!("\u{441}\u{43b}\u{43e}\u{301}\u{432}\u{43e}".chars().all(is_allowed_char), "сло́во допустимо");
        assert!("\u{43d}\u{430}\u{300}".chars().all(is_allowed_char), "на̀ допустимо");
        assert!(!"а\u{302}б".chars().all(is_allowed_char), "U+0302 не разрешён");
    }

    #[test]
    fn content_documents_are_valid() {
        // требование «Корректность контентных документов»: пустой уровень — ошибка данных
        for level in Level::ALL {
            if level == Level::Text {
                continue;
            }
            assert!(!level.examples().is_empty(), "{level:?} пустой в levels.json");
            for t in level.examples() {
                assert!(!t.is_empty() && t.chars().all(is_allowed_char), "не кириллица: {t}");
            }
        }
        let docs = story_docs();
        assert!(docs.len() >= 2);
        let ids: std::collections::HashSet<_> = docs.iter().map(|s| &s.id).collect();
        assert_eq!(ids.len(), docs.len(), "id произведений уникальны");
        for st in docs {
            assert!(!st.title.is_empty() && !st.author.is_empty());
            assert!(st.parts.len() >= 2, "{}: мало частей", st.id);
            for p in &st.parts {
                assert!(!p.trim().is_empty() && p.chars().all(is_allowed_char), "плохая часть в {}", st.id);
            }
        }
    }

    #[test]
    fn part_count_matches_document_length() {
        // инвариант UI-источника: count всегда = длина parts в stories.json
        for st in story_docs() {
            assert_eq!(story_part_count(&st.id), st.parts.len(), "{}", st.id);
        }
    }

    #[test]
    fn stories_have_manual_parts() {
        for st in story_docs() {
            let parts = story_parts(&st.id);
            assert_eq!(parts, st.parts, "{}: story_parts читает JSON", st.id);
        }
    }

    #[test]
    fn broken_json_is_a_parse_error_not_default() {
        // требование «Корректность контентных документов»: пустой уровень — ошибка данных: битый синтаксис — явная ошибка
        assert!(parse_levels(r#"{"vowels": [а, о],}"#).is_err());
        // пустой уровень ловится той же проверкой непустоты, что и в content_documents_are_valid
        let parsed = parse_levels(r#"{"vowels": [], "words": ["дом"]}"#).unwrap();
        assert!(parsed["vowels"].is_empty(), "пустой массив уровня должен быть виден как ошибка данных");
    }

    #[test]
    fn session_text_walks_story_in_order() {
        let expect = story_part_count("martyshka-i-ochki");
        let mut s = Session::with_story(Level::Text, 5, 999, "martyshka-i-ochki");
        assert_eq!(s.total(), expect, "размер задаёт разметка, не size");
        let first = s.current().unwrap().1.to_string();
        assert!(first.starts_with("Мартышка к старости"));
        while s.next(100) {}
        assert!(s.finished());
        assert_eq!(s.finish(0).level, Level::Text);
    }

    #[test]
    fn session_text_size_ignores_requested_size() {
        let parts = story_part_count("strekoza-i-muravey");
        for requested in [1usize, 7, 50] {
            let s = Session::with_story(Level::Text, requested, 1, "strekoza-i-muravey");
            assert_eq!(s.total(), parts, "частей ровно столько, сколько в разметке");
        }
    }

    #[test]
    fn unknown_story_id_falls_back_to_first() {
        let first = story_parts("").into_iter().next().unwrap();
        let s = Session::with_story(Level::Text, 5, 1, "");
        assert_eq!(s.current().unwrap().1, first, "fallback = первая история документа");
    }
}
