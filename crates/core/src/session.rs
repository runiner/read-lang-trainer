use crate::level::{Example, Level};
use crate::stories::{story_by_id, Story, STORIES};
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
    let pool = level.examples();
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

/// Части произведения заданы разметкой: одна непустая строка текста = одна часть.
pub fn story_parts(story: &Story) -> Vec<&'static str> {
    story
        .text
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .collect()
}

/// Сколько частей в произведении — это же размер сессии уровня Text.
pub fn story_part_count(id: &str) -> usize {
    story_by_id(id).map(|s| story_parts(s).len()).unwrap_or(0)
}

/// План сессии уровня Text: все части произведения по порядку (size/seed не влияют).
pub fn text_plan(story_id: &str) -> Vec<Example> {
    let story = story_by_id(story_id).unwrap_or(&STORIES[0]);
    story_parts(story)
        .into_iter()
        .map(|text| Example { text: text.to_string(), level: Level::Text })
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
    story: Option<&'static Story>,
    cursor: usize,
    total_chars: u64,
    total_elapsed_ms: u64,
}

impl Session {
    pub fn new(level: Level, size: usize, seed: u64) -> Self {
        Self::with_story(level, size, seed, "")
    }

    /// Заголовок-подпись для экрана упражнения: Some((название, автор)) у уровня Text.
    pub fn story_title(&self) -> Option<(&'static str, &'static str)> {
        self.story.as_ref().map(|st| (st.title, st.author))
    }

    /// `story_id` значим только для Level::Text; пустой id → первая история (страховка).
    pub fn with_story(level: Level, size: usize, seed: u64, story_id: &str) -> Self {
        let (plan, story) = if level == Level::Text {
            // Размер задаёт разметка: size игнорируется.
            (text_plan(story_id), Some(story_by_id(story_id).unwrap_or(&STORIES[0])))
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
mod tests {
    use super::*;

    #[test]
    fn every_level_has_examples_and_all_cyrillic() {
        for level in Level::ALL {
            if level == Level::Text {
                continue; // тексты живут в stories.rs, проверяются отдельными тестами
            }
            let pool = level.examples();
            assert!(!pool.is_empty(), "{level:?} пустой");
            for text in pool {
                assert!(
                    !text.is_empty() && text.chars().all(|c| ('\u{410}'..='\u{44F}').contains(&c) || c == '\u{401}' || c == '\u{451}' || c == ' '),
                    "не кириллица: {text}"
                );
            }
        }
    }

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
        // VOWELS всего 10 уникальных.
        let plan = session_plan(Level::Vowels, 50, 99);
        assert_eq!(plan.len(), 50);
        let uniq: std::collections::HashSet<_> = plan.iter().map(|e| e.text.as_str()).collect();
        assert_eq!(uniq.len(), 10);
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
        // 10 односимвольных примеров за 60 секунд → 10 симв/мин.
        let mut s = Session::new(Level::Vowels, 10, 3);
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
        let mut s = Session::new(Level::Vowels, 2, 3);
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
    #[test]
    fn stories_are_valid_and_cyrillic() {
        assert!(STORIES.len() >= 2);
        let ids: std::collections::HashSet<_> = STORIES.iter().map(|s| s.id).collect();
        assert_eq!(ids.len(), STORIES.len(), "id уникальны");
        for st in STORIES {
            assert!(!st.title.is_empty() && !st.author.is_empty());
            let bad: Vec<_> = st.text
                .chars()
                .filter(|c| {
                    !('\u{410}'..='\u{44F}').contains(c)
                        && *c != '\u{401}' && *c != '\u{451}'
                        && !c.is_whitespace()
                        && !matches!(c, '.' | ',' | '!' | '?' | ';' | ':' | '-' | '(' | ')' | '\u{00AB}' | '\u{00BB}' | '\u{2014}' | '\u{2026}' | '0'..='9')
                })
                .collect();
            assert!(bad.is_empty(), "недопустимые символы в {}: {:?}", st.id, bad);
        }
    }

    #[test]
    fn stories_have_manual_parts() {
        for st in STORIES {
            let parts = story_parts(st);
            assert!(parts.len() >= 2, "{}: мало частей", st.id);
            assert!(parts.iter().all(|p| !p.trim().is_empty()));
            // склейка частей через \n воспроизводит текст без потери содержимого
            let strip = |t: &str| t.chars().filter(|c| !c.is_whitespace()).collect::<String>();
            assert_eq!(strip(&parts.join("\n")), strip(st.text), "{}: склейка != источник", st.id);
        }
    }

    #[test]
    fn part_counts_are_recorded() {
        // «записать итог сравнения» из задачи 4.1 — числа фиксируются тестом
        let counts: Vec<(&str, usize)> = STORIES
            .iter()
            .map(|st| (st.id, story_part_count(st.id)))
            .collect();
        for (id, c) in &counts {
            assert!((6..=30).contains(c), "{id}: {c} частей вне разумного диапазона");
        }
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
        let s = Session::with_story(Level::Text, 5, 1, "");
        assert!(s.current().unwrap().1.starts_with("Уж сколько раз"));
    }
}
