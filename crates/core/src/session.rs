use crate::level::{Example, Level};
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
pub fn session_plan(level: Level, size: usize, seed: u64) -> Vec<Example> {
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
            text: pool[i],
            level,
        })
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
    cursor: usize,
    total_chars: u64,
    total_elapsed_ms: u64,
}

impl Session {
    pub fn new(level: Level, size: usize, seed: u64) -> Self {
        let plan = session_plan(level, size, seed);
        let total_chars = plan.iter().map(|e| e.text.chars().count() as u64).sum();
        Session {
            plan,
            cursor: 0,
            total_chars,
            total_elapsed_ms: 0,
        }
    }

    /// Текущий упражнение (1-based позиция, текст). После конца — None.
    pub fn current(&self) -> Option<(usize, &'static str)> {
        self.plan
            .get(self.cursor)
            .map(|e| (self.cursor + 1, e.text))
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
        let a: Vec<_> = session_plan(Level::Words, 12, 7).iter().map(|e| e.text).collect();
        let b: Vec<_> = session_plan(Level::Words, 12, 7).iter().map(|e| e.text).collect();
        assert_eq!(a, b);
    }

    #[test]
    fn plan_repeats_when_size_exceeds_pool() {
        // VOWELS всего 10 уникальных.
        let plan = session_plan(Level::Vowels, 50, 99);
        assert_eq!(plan.len(), 50);
        let uniq: std::collections::HashSet<_> = plan.iter().map(|e| e.text).collect();
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
}
