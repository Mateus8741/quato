import { describe, expect, it } from 'vitest';
import { flashDir } from './render.ts';
import { FxState, GRAVITY, MARK_FADE, MARK_HOLD, MAX_MARKS, MAX_SHELLS, SHELL_FADE, SHELL_REST, TRACER_LIFE, makeShell, markAlpha, shellAlpha, stepShell, type FxShot } from './state.ts';

const shot = (o: Partial<FxShot> = {}): FxShot => ({ from: { x: 100, y: 100 }, to: { x: 400, y: 300 }, eject: { x: 90, y: 95 }, ejectDir: -1, floor: 500, ...o });
const seeded = (seed: number) => {
  let a = seed;
  return () => {
    a = (a * 1664525 + 1013904223) >>> 0;
    return a / 0x100000000;
  };
};

describe('marcas', () => {
  it('ficam inteiras e somem devagar em ~20 s', () => {
    expect(markAlpha(0)).toBe(1);
    expect(markAlpha(MARK_HOLD)).toBe(1);
    expect(markAlpha(MARK_HOLD + MARK_FADE / 2)).toBeCloseTo(0.5);
    expect(markAlpha(MARK_HOLD + MARK_FADE)).toBe(0);
    expect(MARK_HOLD + MARK_FADE).toBe(20);
  });
  it('no máximo 60, as mais antigas saem primeiro', () => {
    const s = new FxState(seeded(1));
    for (let i = 0; i < 80; i++) s.addShot(shot({ to: { x: i, y: 0 } }), 0);
    expect(s.marks.length).toBe(MAX_MARKS);
    expect(s.marks[0]?.x).toBe(20);
    expect(s.marks.at(-1)?.x).toBe(79);
  });
  it('são removidas depois de sumir', () => {
    const s = new FxState(seeded(1));
    s.addShot(shot(), 0);
    s.step(0.016, 19);
    expect(s.marks.length).toBe(1);
    s.step(0.016, 21);
    expect(s.marks.length).toBe(0);
  });
});

describe('estojos', () => {
  it('saem para trás, caem, quicam 1 ou 2 vezes e param no chão', () => {
    for (let seed = 1; seed < 30; seed++) {
      const rnd = seeded(seed);
      const sh = makeShell(shot({ ejectDir: -1 }), 0, rnd);
      expect(sh.vx).toBeLessThan(0);
      expect(sh.vy).toBeLessThan(0);
      expect([1, 2]).toContain(sh.maxBounces);
      let t = 0;
      let ups = 0;
      let lastVy = sh.vy;
      while (sh.restAt === null && t < 10) {
        stepShell(sh, 1 / 60, t);
        if (lastVy > 0 && sh.vy < 0) ups++;
        lastVy = sh.vy;
        t += 1 / 60;
      }
      expect(sh.restAt).not.toBeNull();
      expect(sh.y).toBe(sh.floor);
      expect(sh.bounces).toBeLessThanOrEqual(2);
      expect(ups).toBeLessThanOrEqual(2);
    }
  });
  it('gravidade puxa para baixo', () => {
    const sh = makeShell(shot(), 0, seeded(2));
    sh.vy = 0;
    stepShell(sh, 0.1, 0);
    expect(sh.vy).toBeCloseTo(GRAVITY * 0.1);
  });
  it('ficam ~6,5 s parados e somem; no máximo 40', () => {
    const s = new FxState(seeded(3));
    s.addShot(shot(), 0);
    let t = 0;
    while (s.shells[0] && s.shells[0].restAt === null) {
      t += 1 / 60;
      s.step(1 / 60, t);
    }
    const rest = s.shells[0]!.restAt!;
    expect(shellAlpha(s.shells[0]!, rest + SHELL_REST - 0.1)).toBe(1);
    expect(shellAlpha(s.shells[0]!, rest + SHELL_REST + SHELL_FADE / 2)).toBeCloseTo(0.5);
    s.step(0.016, rest + SHELL_REST + SHELL_FADE + 0.1);
    expect(s.shells.length).toBe(0);
    for (let i = 0; i < 60; i++) s.addShot(shot(), 100);
    expect(s.shells.length).toBe(MAX_SHELLS);
  });
});

describe('estado geral', () => {
  it('rastro dura pouco e animating/empty refletem o que há na tela', () => {
    const s = new FxState(seeded(4));
    expect(s.empty()).toBe(true);
    expect(s.animating()).toBe(false);
    s.addShot(shot(), 0);
    expect(s.animating()).toBe(true);
    s.step(0.016, TRACER_LIFE + 0.01);
    expect(s.tracers.length).toBe(0);
    // sem estojos mexendo, só as marcas: não precisa de quadros rápidos
    s.shells = [];
    expect(s.animating()).toBe(false);
    expect(s.empty()).toBe(false);
    s.clear();
    expect(s.empty()).toBe(true);
  });
  it('o clarão do cano aponta para o alvo', () => {
    expect(flashDir(shot().from, shot().to)).toBe(1);
    expect(flashDir({ x: 500, y: 0 }, { x: 10, y: 0 })).toBe(-1);
  });
});
