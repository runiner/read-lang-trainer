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

/// Границы предложений: абзацы режем по `\n`, внутри абзаца — после `.!?…`
/// (закрывающие » ' " ) ’ приклеиваются; прямая речь «...» сразу за терминатором
/// или после двоеточия входит в то же предложение).
pub fn split_sentences(text: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for para in text.split('\n') {
        let p = para.trim();
        if p.is_empty() {
            continue;
        }
        let chars: Vec<char> = p.chars().collect();
        let mut start = 0usize;
        let mut i = 0usize;
        while i < chars.len() {
            if matches!(chars[i], '.' | '!' | '?' | '\u{2026}') {
                let mut j = i + 1;
                while j < chars.len() && matches!(chars[j], '\u{00BB}' | '\'' | '"' | ')' | '\u{2019}') {
                    j += 1;
                }
                // прямая речь сразу за границей («...») дописывается к предложению
                if j < chars.len() && chars[j] == '\u{00AB}' {
                    if let Some(close) = chars[j..].iter().position(|&c| c == '\u{00BB}') {
                        j += close + 1;
                    }
                }
                let piece: String = chars[start..j.min(chars.len())].iter().collect();
                let piece = piece.trim();
                if !piece.is_empty() {
                    out.push(piece.to_string());
                }
                start = j;
                i = j;
                continue;
            }
            i += 1;
        }
        if start < chars.len() {
            let tail: String = chars[start..].iter().collect();
            let tail = tail.trim();
            if !tail.is_empty() {
                out.push(tail.to_string());
            }
        }
    }
    out
}

/// Сколько предложений в произведении — максимум упражнений уровня Text.
pub fn story_sentence_count(id: &str) -> usize {
    story_by_id(id).map(|s| split_sentences(s.text).len()).unwrap_or(0)
}

/// Разбить произведение ровно на `n` упорядоченных частей по границам предложений.
/// Жадная раздача: цель части ≈ total_chars/n; последняя часть забирает остаток.
pub fn story_parts(story: &Story, n: usize) -> Vec<String> {
    let sentences = split_sentences(story.text);
    assert!(!sentences.is_empty(), "пустое произведение");
    let n = n.max(1);
    let lens: Vec<usize> = sentences.iter().map(|x| x.chars().count()).collect();
    let total: usize = lens.iter().sum();
    let target = (total / n).max(1);
    // Если предложений меньше n, частей физически не может быть больше — тесты и UI
    // сверяются через story_sentence_count; здесь clamp уже сделал своё.
    let n = n.min(sentences.len());
    // Границы: индекс последнего предложения k-й части (равномерно по символам,
    // всегда оставляем по одному предложению на каждую оставшуюся часть).
    let mut cuts: Vec<usize> = Vec::with_capacity(n.saturating_sub(1));
    let last_valid = sentences.len() - 1;
    for k in 1..n {
        let goal = total * k / n;
        let prev = cuts.last().copied();
        let min_j = prev.map(|p| p + 1).unwrap_or(0); // строго растём
        let max_j = last_valid - (n - k); // резерв под остаток частей
        let mut j = min_j;
        let mut acc: usize = (0..=j.min(last_valid)).map(|x| lens[x]).sum();
        while acc < goal && j < max_j {
            j += 1;
            acc += lens[j];
        }
        cuts.push(j);
    }
    let mut parts: Vec<String> = Vec::with_capacity(n);
    let mut prev_end = 0usize;
    for &cut in &cuts {
        let piece = sentences[prev_end..=cut].join(" ");
        parts.push(piece);
        prev_end = cut + 1;
    }
    if prev_end < sentences.len() {
        parts.push(sentences[prev_end..].join(" "));
    }
    debug_assert_eq!(parts.len(), n);
    parts
}

/// План сессии уровня Text: части произведения как примеры.
/// Размер плана = min(size, число предложений) — см. спеку «частей больше, чем предложений».
pub fn text_plan(story_id: &str, size: usize) -> Vec<Example> {
    let story = story_by_id(story_id).unwrap_or(&STORIES[0]);
    let n = size.min(split_sentences(story.text).len()).max(1);
    story_parts(story, n)
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
        let size = size.max(1);
        let (plan, story) = if level == Level::Text {
            let story = story_by_id(story_id).unwrap_or(&STORIES[0]);
            let n = size.min(split_sentences(story.text).len()).max(1);
            (
                story_parts(story, n)
                    .into_iter()
                    .map(|text| Example { text, level })
                    .collect::<Vec<_>>(),
                Some(story),
            )
        } else {
            (session_plan(level, size, seed), None)
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
    fn parts_reassemble_into_source() {
        for st in STORIES {
            let n = 5;
            let parts = story_parts(st, n);
            assert_eq!(parts.len(), n.min(split_sentences(st.text).len()));
            let joined: String = parts.join("");
            let strip = |t: &str| t.chars().filter(|c| !c.is_whitespace()).collect::<String>();
            assert_eq!(strip(&joined), strip(st.text), "склейка частей != источник в {}", st.id);
        }
    }

    #[test]
    fn parts_ordered_nonempty_for_many_sizes() {
        for st in STORIES {
            let max = split_sentences(st.text).len();
            for n in [1usize, 5, 12, max] {
                let parts = story_parts(st, n);
                assert!(parts.iter().all(|p| !p.trim().is_empty()), "пустая часть при n={n}");
                assert_eq!(parts.len(), n.min(max));
            }
        }
    }

    #[test]
    fn text_plan_is_deterministic_without_seed() {
        let a: Vec<_> = text_plan("vorona-i-lisitsa", 6).iter().map(|e| e.text.clone()).collect();
        let b: Vec<_> = text_plan("vorona-i-lisitsa", 6).iter().map(|e| e.text.clone()).collect();
        assert_eq!(a, b);
        assert_eq!(a.len(), 6);
    }

    #[test]
    fn session_text_walks_story_in_order() {
        let mut s = Session::with_story(Level::Text, 5, 999, "martyshka-i-ochki");
        assert_eq!(s.total(), 5);
        let first = s.current().unwrap().1.to_string();
        assert!(first.starts_with("Мартышка к старости"));
        while s.next(100) {}
        assert!(s.finished());
        assert_eq!(s.finish(0).level, Level::Text);
    }

    #[test]
    fn session_text_caps_at_sentence_count() {
        let max = story_sentence_count("strekoza-i-muravey");
        let s = Session::with_story(Level::Text, 50, 1, "strekoza-i-muravey");
        assert_eq!(s.total(), max, "нет упражнений сверх предложений");
    }

    #[test]
    fn unknown_story_id_falls_back_to_first() {
        let s = Session::with_story(Level::Text, 5, 1, "");
        assert!(s.current().unwrap().1.starts_with("Уж сколько раз"));
    }
}
