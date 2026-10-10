import init, { create_session, WasmSession } from '../pkg/rus_wasm.js';

export type LevelId = 'vowels' | 'simple-syllables' | 'syllables' | 'words' | 'phrases';

export interface ExerciseView {
  position: number;
  text: string;
  total: number;
}

export interface SessionRecord {
  date_ms: number;
  level: LevelId;
  count: number;
  duration_ms: number;
  chars_per_min: number | null;
}

let initialized = false;

/** Типизированный фасад единственной границы wasm/JS. */
export async function initWasm(): Promise<void> {
  if (!initialized) {
    await init();
    initialized = true;
  }
}

export function startSession(level: LevelId, size: number, seed: number): SessionHandle {
  const raw = create_session(level, size, seed);
  return new SessionHandle(raw);
}

export class SessionHandle {
  constructor(private raw: WasmSession) {}

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
