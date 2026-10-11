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

/// Границы фрагментов: абзацы режем по `\n`, внутри абзаца — после `. , : ; ? ! …`
/// (закрывающая кавычка/скобка `» ' " ) ’` приклеивается к фрагменту; прямая речь
/// «...», стоящая сразу за границей, дописывается в тот же фрагмент).
pub fn split_sentences(text: &str) -> Vec<String> {
    const TERMS: [char; 7] = ['.', ',', ':', ';', '!', '?', '\u{2026}'];
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
            if chars[i].is_whitespace() {
                i += 1;
                continue;
            }
            // идём до ближайшего терминатора
            let mut term = None;
            let mut j = i;
            while j < chars.len() {
                if TERMS.contains(&chars[j]) {
                    term = Some(j);
                    break;
                }
                j += 1;
            }
            match term {
                Some(t) => {
                    let mut e = t + 1;
                    while e < chars.len() && matches!(chars[e], '\u{00BB}' | '\'' | '"' | ')' | '\u{2019}') {
                        e += 1;
                    }
                    // «...» сразу за границей — продолжение этого фрагмента
                    if e < chars.len() && chars[e] == '\u{00AB}' {
                        if let Some(close) = chars[e..].iter().position(|&c| c == '\u{00BB}') {
                            e += close + 1;
                        }
                    }
                    let piece: String = chars[start..e.min(chars.len())].iter().collect();
                    let piece = piece.trim();
                    if !piece.is_empty() {
                        out.push(piece.to_string());
                    }
                    start = e;
                    i = e;
                }
                None => break, // хвост без терминатора — добавим ниже
            }
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

/// Фрагменты для разбиения: границы по всем знакам препинания плюс склейка
/// микро-обрывков (<4 символов) с предыдущим. Единственный источник числа частей.
/// Грубые границы по всем знакам препинания, без склейки.
pub fn split_sentences_raw(text: &str) -> Vec<String> {
    split_sentences(text)
}

/// Границы + склейка микро-обрывков (<4 символов) с предыдущим.
pub fn story_fragments(text: &str) -> Vec<String> {
    glue_small(split_sentences(text), 4)
}

/// Индекс end ∈ [lo..=hi], где кумулятивная длина cum[end+?] ближе всего к goal.
/// cum[i] — длина префикса sentences[0..=i]; граница после end <=> cum[end].
/// Граница end ∈ [lo..=hi]: префикс sentences[0..=end] ближе всего к goal.
fn nearest_cut(cum: &[usize], lens: &[usize], goal: usize, lo: usize, hi: usize) -> usize {
    // длина префикса до end включительно = sum(lens[..=end]) (без разделителей)
    let pref_end = |e: usize| -> usize { lens[..=e].iter().sum() };
    let mut best = lo;
    let mut bd = pref_end(lo).abs_diff(goal);
    for j in lo + 1..=hi {
        let d = pref_end(j).abs_diff(goal);
        if d < bd {
            bd = d;
            best = j;
        }
    }
    let _ = cum;
    best
}

fn sum_lens(lens: &[usize], from: usize, to: usize) -> usize {
    lens[from..=to].iter().sum()
}

fn glue_small(items: Vec<String>, min_len: usize) -> Vec<String> {
    let mut v = items;
    let mut k = 1usize;
    while k < v.len() {
        if v[k].chars().count() < min_len {
            let tail = v.remove(k);
            let prev = v.remove(k - 1);
            v.insert(k - 1, format!("{prev} {tail}"));
        } else {
            k += 1;
        }
    }
    v
}

/// Сколько фрагментов в произведении — максимум упражнений уровня Text.
pub fn story_sentence_count(id: &str) -> usize {
    story_by_id(id).map(|s| story_fragments(s.text).len()).unwrap_or(0)
}

/// Разбить произведение ровно на `n` упорядоченных частей по границам предложений.
/// Жадная раздача: цель части ≈ total_chars/n; последняя часть забирает остаток.
pub fn story_parts(story: &Story, n: usize) -> Vec<String> {
    let sentences = story_fragments(story.text);
    let m = sentences.len();
    let n = n.clamp(1, m);
    if n >= m {
        return sentences; // по одному фрагменту на часть
    }
    let lens: Vec<usize> = sentences.iter().map(|x| x.chars().count()).collect();
    // pref[i] = длина join(sentences[0..i]) с пробелами
    let mut pref = vec![0usize; m + 1];
    for i in 0..m {
        pref[i + 1] = pref[i] + lens[i] + if i == 0 { 0 } else { 1 };
    }
    let total = pref[m];
    // DP: dp[k][j] = мин. сумма квадратов отклонений частей от ideal при разбивке
    // первых j фрагментов на k частей. Идеал тот же для всех частей (равномерно).
    let ideal = total as f64 / n as f64;
    let mut dp = vec![vec![f64::INFINITY; m + 1]; n + 1];
    let mut cut = vec![vec![0usize; m + 1]; n + 1];
    dp[0][0] = 0.0;
    for k in 1..=n {
        for j in k..=m - (n - k) {
            // предыдущая граница i: у части [i..j) хотя бы 1 фрагмент, и до i хватает для k-1
            let lo = k - 1;
            let hi = j - 1;
            for i in lo..=hi {
                if !dp[k - 1][i].is_finite() {
                    continue;
                }
                let len = pref[j] - pref[i];
                let dev = len as f64 - ideal;
                let val = dp[k - 1][i] + dev * dev;
                if val < dp[k][j] {
                    dp[k][j] = val;
                    cut[k][j] = i;
                }
            }
        }
    }
    // восстановление границ
    let mut bounds = Vec::with_capacity(n + 1);
    let (mut k, mut j) = (n, m);
    bounds.push(j);
    while k > 0 {
        j = cut[k][j];
        bounds.push(j);
        k -= 1;
    }
    bounds.reverse();
    let mut parts: Vec<String> = Vec::with_capacity(n);
    for w in bounds.windows(2) {
        parts.push(sentences[w[0]..w[1]].join(" "));
    }
    debug_assert_eq!(parts.len(), n);
    parts
}

/// План сессии уровня Text: части произведения как примеры.
/// Размер плана = min(size, число предложений) — см. спеку «частей больше, чем предложений».
pub fn text_plan(story_id: &str, size: usize) -> Vec<Example> {
    let story = story_by_id(story_id).unwrap_or(&STORIES[0]);
    let n = size.min(story_fragments(story.text).len()).max(1);
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
            let n = size.min(story_fragments(story.text).len()).max(1);
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
    fn boundaries_after_every_punctuation_mark() {
        let text = "Мама мыла, а папа читал; бабушка вязала. Дед пилил: и! да?";
        let got = split_sentences(text);
        assert_eq!(
            got,
            vec![
                "Мама мыла,",
                "а папа читал;",
                "бабушка вязала.",
                "Дед пилил:",
                "и!",
                "да?"
            ]
        );
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
            let max = story_fragments(st.text).len(); // тот же источник, что и story_parts
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
        assert_eq!(s.total(), max, "нет упражнений сверх фрагментов");
        assert!(max > 0);
    }

    #[test]
    fn unknown_story_id_falls_back_to_first() {
        let s = Session::with_story(Level::Text, 5, 1, "");
        assert!(s.current().unwrap().1.starts_with("Уж сколько раз"));
    }
}
