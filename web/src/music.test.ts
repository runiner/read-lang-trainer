import { describe, expect, it, vi } from 'vitest';
import { playCheerMusic } from './music.js';

describe('cheer music', () => {
  it('starts oscillators and stops cleanly', () => {
    const start = vi.fn();
    const stop = vi.fn();
    const close = vi.fn();
    const node = () => ({ connect: () => node(), disconnect: vi.fn(), gain: { value: 0, setValueAtTime: vi.fn(), linearRampToValueAtTime: vi.fn(), exponentialRampToValueAtTime: vi.fn() } });
    class FakeCtx {
      currentTime = 0;
      destination = {};
      createGain() { return node(); }
      createOscillator() { return { ...node(), type: '', frequency: { value: 0 }, start, stop }; }
      close() { close(); return Promise.resolve(); }
    }
    vi.stubGlobal('AudioContext', FakeCtx);
    vi.useFakeTimers();
    const m = playCheerMusic(12_000);
    vi.advanceTimersByTime(1000);
    expect(start).toHaveBeenCalled();
    m.stop();
    expect(stop).toHaveBeenCalled();
    expect(close).toHaveBeenCalled();
    m.stop(); // идемпотентно
    vi.useRealTimers();
    vi.unstubAllGlobals();
  });

  it('silently survives missing WebAudio', () => {
    vi.stubGlobal('AudioContext', undefined);
    const m = playCheerMusic();
    expect(() => m.stop()).not.toThrow();
    vi.unstubAllGlobals();
  });
});
