// Заводная музыка празднования: WebAudio-секвенсор «robot dance»-вайба.
// Ошибки WebAudio проглатываются — спеке достаточно тишины (визуальный эквивалент есть).

export interface CheerMusic {
  stop(): void;
}

const BPM = 120;
const STEP = 60 / BPM / 2; // восьмушки
// Лас-мажор, квадратные волны: бас + арпеджио.
const BASS = [57, 57, 64, 64, 60, 60, 65, 65]; // A2 C3 C3... тональность не критична
const LEAD = [69, 73, 76, 81, 76, 73, 69, 73, 72, 76, 79, 84, 79, 76, 72, 76];

const midiHz = (m: number) => 440 * Math.pow(2, (m - 69) / 12);

export function playCheerMusic(totalMs = 12_000): CheerMusic {
  let ctx: AudioContext | null = null;
  let master: GainNode | null = null;
  try {
    ctx = new AudioContext();
    void ctx.resume?.().catch(() => {});
    const now = ctx.currentTime;
    master = ctx.createGain();
    master.connect(ctx.destination);
    // fade-in 300ms
    master.gain.setValueAtTime(0.0001, now);
    master.gain.linearRampToValueAtTime(0.22, now + 0.3);
    // fade-out последние 1.5s
    master.gain.setValueAtTime(0.22, now + totalMs / 1000 - 1.5);
    master.gain.linearRampToValueAtTime(0.0001, now + totalMs / 1000);

    // Весь паттерн планируется заранее: setInterval-такты ненадёжны (тормозят в фоновых
    // вкладках), а при рассинхронизации с currentTime ноты просто выпадали — так и пропала музыка.
    const t0 = now + 0.05;
    const endAt = now + totalMs / 1000 - 1.4; // до начала fade-out
    for (let step = 0; t0 + step * STEP < endAt; step++) {
      const at = t0 + step * STEP;
      voice(ctx, master, 'square', midiHz(BASS[step % BASS.length]), at, STEP * 0.9, 0.5);
      voice(ctx, master, 'sawtooth', midiHz(LEAD[step % LEAD.length] + 12), at, STEP * 0.45, 0.18);
    }
  } catch (e) {
    console.error('cheer music: WebAudio недоступен', e);
    ctx = null;
  }
  return {
    stop() {
      try {
        master?.disconnect();
        void ctx?.close();
      } catch {
        /* уже закрыт */
      }
      ctx = null;
      master = null;
    },
  };
}

function voice(ctx: AudioContext, dest: GainNode, type: OscillatorType, freq: number, at: number, dur: number, gain: number) {
  const osc = ctx.createOscillator();
  const g = ctx.createGain();
  osc.type = type;
  osc.frequency.value = freq;
  g.gain.setValueAtTime(gain, at);
  g.gain.linearRampToValueAtTime(0.0001, at + dur);
  osc.connect(g).connect(dest);
  osc.start(at);
  osc.stop(at + dur + 0.02);
}
