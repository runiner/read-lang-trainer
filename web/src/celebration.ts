// Празднование по оценке взрослого (openspec: capability celebration).
// Единственное место приложения с движением; уважает prefers-reduced-motion.

import { playCheerMusic, type CheerMusic } from './music.js';

export interface CelebrationOptions {
  durationMs?: number;
  /** Инъекция таймера автозавершения ради тестов с fake timers. */
  schedule?: (fn: () => void, ms: number) => unknown;
}

export interface CelebrationHandle {
  stop(): void;
}

const PARTICLES = 80;
const FRAME_MS = 150;

export function startCelebration(root: HTMLElement, opts: CelebrationOptions = {}): CelebrationHandle {
  const durationMs = opts.durationMs ?? 12_000;
  const schedule = opts.schedule ?? ((fn, ms) => setTimeout(fn, ms));
  const reduced = window.matchMedia?.('(prefers-reduced-motion: reduce)').matches ?? false;

  const overlay = document.createElement('div');
  overlay.id = 'celebration';
  overlay.className = `celebration-overlay${reduced ? ' static' : ''}`;
  overlay.setAttribute('role', 'dialog');
  overlay.setAttribute('aria-label', 'Молодец!');

  const canvas = document.createElement('canvas');
  canvas.className = 'confetti';
  canvas.width = window.innerWidth;
  canvas.height = window.innerHeight;
  const puppy = document.createElement('img');
  puppy.id = 'puppy';
  puppy.alt = '';
  puppy.src = reduced ? '/puppy/static.svg' : '/puppy/frame-1.svg';
  const praise = document.createElement('p');
  praise.className = 'praise';
  praise.textContent = 'Молодец!';
  overlay.append(canvas, puppy, praise);
  root.appendChild(overlay);

  let raf = 0;
  let frameTimer: unknown = null;
  let music: CheerMusic | null = null;
  try {
    music = playCheerMusic(durationMs);
  } catch {
    music = null;
  }

  if (!reduced) {
    // конфетти
    const ctx2d = canvas.getContext('2d');
    const colors = ['#c9531e', '#f6c445', '#2e7d32', '#1565c0', '#ad1457'];
    const bits = Array.from({ length: PARTICLES }, (_, i) => ({
      x: Math.random() * canvas.width,
      y: -20 - Math.random() * canvas.height * 0.5,
      r: 4 + Math.random() * 5,
      vy: 1.5 + Math.random() * 2.5,
      vx: -0.7 + Math.random() * 1.4,
      rot: Math.random() * Math.PI,
      color: colors[i % colors.length],
    }));
    const tick = () => {
      if (!ctx2d) return;
      ctx2d.clearRect(0, 0, canvas.width, canvas.height);
      for (const b of bits) {
        b.y += b.vy;
        b.x += b.vx + Math.sin(b.y / 40);
        b.rot += 0.05;
        if (b.y > canvas.height + 20) b.y = -20;
        ctx2d.save();
        ctx2d.translate(b.x, b.y);
        ctx2d.rotate(b.rot);
        ctx2d.fillStyle = b.color;
        ctx2d.fillRect(-b.r / 2, -b.r / 2, b.r, b.r * 1.6);
        ctx2d.restore();
      }
      raf = requestAnimationFrame(tick);
    };
    raf = requestAnimationFrame(tick);
    // пляска: смена кадров
    let frame = 1;
    frameTimer = setInterval(() => {
      frame = (frame % 3) + 1;
      puppy.src = `/puppy/frame-${frame}.svg`;
    }, FRAME_MS);
  }

  let stopped = false;
  const stop = () => {
    if (stopped) return;
    stopped = true;
    cancelAnimationFrame(raf);
    if (frameTimer !== null) clearInterval(frameTimer as ReturnType<typeof setInterval>);
    music?.stop();
    window.removeEventListener('pointerdown', onInput);
    window.removeEventListener('keydown', onInput);
    overlay.remove();
  };
  const onInput = () => stop();
  window.addEventListener('pointerdown', onInput);
  window.addEventListener('keydown', onInput);
  schedule(stop, durationMs);
  return { stop };
}
