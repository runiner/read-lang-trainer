import init, { content_version, create_session, stories_json, WasmSession } from '../pkg/rus_wasm.js';

export type LevelId = 'vowels' | 'simple-syllables' | 'syllables' | 'words' | 'phrases' | 'text';

export interface StoryInfo {
  id: string;
  title: string;
  author: string;
  /** Размер сессии уровня «текст» задаёт разметка, не настройка. */
  parts: number;
}

export interface ExerciseView {
  position: number;
  text: string;
  total: number;
  story?: { title: string; author: string } | null;
}

export interface SessionRecord {
  date_ms: number;
  level: LevelId;
  count: number;
  duration_ms: number;
  /** Отсутствует в записях старого формата (символы/мин) — показываем прочерк. */
  words_per_min?: number | null;
}

let initialized = false;

/** Типизированный фасад единственной границы wasm/JS. */
export async function initWasm(): Promise<void> {
  if (!initialized) {
    await init();
    initialized = true;
  }
}

export function startSession(level: LevelId, size: number, seed: number, storyId = ''): SessionHandle {
  const raw = create_session(level, size, seed, storyId);
  return new SessionHandle(raw);
}

/** Произведения уровня «текст» с числом частей из разметки. */
export function listStories(): StoryInfo[] {
  return JSON.parse(stories_json()) as StoryInfo[];
}

/** Версия контента внутри собранного wasm-артефакта (hex SHA-256). */
export function contentVersion(): string {
  return content_version();
}

export class SessionHandle {
  private raw: WasmSession;
  constructor(raw: WasmSession) {
    this.raw = raw;
  }

  current(): ExerciseView | null {
    const json = this.raw.current_json();
    return json === undefined || json === null ? null : (JSON.parse(json) as ExerciseView);
  }

  next(elapsedMs: number): boolean {
    return this.raw.next(Math.max(0, Math.round(elapsedMs)));
  }

  finished(): boolean {
    return this.raw.finished();
  }

  /** Итог сессии для экрана статистики и localStorage. */
  finish(): SessionRecord {
    return JSON.parse(this.raw.finish(Date.now())) as SessionRecord;
  }

  free(): void {
    this.raw.free();
  }
}
