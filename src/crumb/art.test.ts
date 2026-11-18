import { describe, expect, it } from 'vitest';
import { burst, crumbScale, CRUMB_GRID, CRUMB_H, CRUMB_PALETTE, CRUMB_W, EAT_MS, particleAt } from './art.ts';

describe('migalha', () => {
  it('a arte tem o tamanho certo e só usa cores da paleta', () => {
    expect(CRUMB_GRID).toHaveLength(CRUMB_H);
    for (const row of CRUMB_GRID) {
      expect(row).toHaveLength(CRUMB_W);
      for (const c of row) if (c !== '.') expect(CRUMB_PALETTE[c], c).toBeTruthy();
    }
  });

  it('a migalha encolhe em mordidas até sumir', () => {
    expect(crumbScale(-5)).toBe(1);
    expect(crumbScale(0)).toBe(1);
    let prev = 1;
    for (let t = 0; t <= EAT_MS; t += 20) {
      const s = crumbScale(t);
      expect(s).toBeLessThanOrEqual(prev);
      expect(s).toBeGreaterThanOrEqual(0);
      prev = s;
    }
    expect(crumbScale(EAT_MS)).toBe(0);
  });

  it('os farelos são determinísticos com o sorteio injetado, sobem e depois caem e somem', () => {
    const mk = () => {
      let n = 0;
      return () => ((n++ * 0.37) % 1);
    };
    const a = burst(mk());
    const b = burst(mk());
    expect(a).toEqual(b);
    expect(a.length).toBeGreaterThan(4);
    const p = a[0]!;
    const start = particleAt(p, 0);
    expect(start.alpha).toBe(1);
    expect(particleAt(p, 120).y).toBeLessThan(start.y);
    expect(particleAt(p, EAT_MS).alpha).toBe(0);
    expect(particleAt(p, EAT_MS - 1).y).toBeGreaterThan(particleAt(p, 120).y);
  });
});
