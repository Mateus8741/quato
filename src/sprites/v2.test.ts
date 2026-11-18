import { describe, expect, it } from 'vitest';
import { CELL_H, GLYPHS, glyph, hasGlyph, textWidth, unknownChars, wrapBalanced, wrapText } from './font.ts';
import { signIcon, TOP_H, TOP_W, mouthToTopX, renderBalloon, renderSign, signLayout } from './overlay.ts';
import { DOUBT_DURATION_MS, DOUBT_MAX_GAP_MS, DOUBT_MIN_GAP_MS, doubtGap, doubtState } from './doubt.ts';
import { SCREAM_DURATION_MS, SCREAM_MAX_GAP_MS, SCREAM_MIN_GAP_MS, SCREAM_TEXT, screamGap, screamState, shakeOffset } from './scream.ts';

const WATER = 'Hora de beber água!';

describe('fonte pixel', () => {
  it('tem A–Z, 0–9, pontuação e acentos (maiúsculas e minúsculas)', () => {
    const must = 'ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789!?.,-:\'ÁÀÂÃÉÊÍÓÔÕÚÇ ';
    for (const ch of must) expect(hasGlyph(ch), ch).toBe(true);
    for (const ch of must.toLowerCase()) expect(hasGlyph(ch), ch).toBe(true);
  });

  it('todo caractere da plaquinha e do balão tem glifo', () => {
    expect(unknownChars(WATER)).toEqual([]);
    expect(unknownChars(SCREAM_TEXT)).toEqual([]);
    expect(SCREAM_TEXT).toBe('A'.repeat(24));
  });

  it('glifos têm altura fixa e linhas da mesma largura', () => {
    for (const ch of [...Object.keys(GLYPHS), 'Á', 'Ç', 'õ']) {
      const g = glyph(ch)!;
      expect(g.rows.length, ch).toBe(CELL_H);
      for (const r of g.rows) expect(r.length, ch).toBe(g.width);
    }
    expect(glyph('á')).toEqual(glyph('Á'));
    expect(glyph('Á')).not.toEqual(glyph('A'));
    expect(glyph('€')).toBeUndefined();
  });

  it('quebra de linha: palavras e palavras longas demais', () => {
    expect(wrapBalanced(WATER, 62, 2)).toEqual(['Hora de', 'beber água!']);
    const lines = wrapText(SCREAM_TEXT, 62);
    expect(lines).toHaveLength(2);
    expect(lines.join('')).toBe(SCREAM_TEXT);
    for (const l of lines) expect(textWidth(l)).toBeLessThanOrEqual(62);
  });
});

describe('plaquinha e balão', () => {
  it('a plaquinha cabe na faixa de cima e tem pixels', () => {
    const L = signLayout(WATER);
    expect(L.lines).toHaveLength(2);
    expect(L.x).toBeGreaterThanOrEqual(0);
    expect(L.x + L.w).toBeLessThanOrEqual(TOP_W);
    expect(L.y + L.h).toBeLessThanOrEqual(TOP_H);
    const p = renderSign(WATER);
    expect(p.w).toBe(TOP_W);
    expect(p.h).toBe(TOP_H);
    expect(p.data.some((v, i) => i % 4 === 3 && v > 0)).toBe(true);
  });

  it('o balão treme (muda com o tique) e aponta para a boca dos dois lados', () => {
    const a = renderBalloon(SCREAM_TEXT, mouthToTopX(26, 'right'), 1);
    const b = renderBalloon(SCREAM_TEXT, mouthToTopX(26, 'right'), 2);
    expect(Buffer.compare(Buffer.from(a.data), Buffer.from(b.data))).not.toBe(0);
    expect(mouthToTopX(26, 'right')).toBeGreaterThan(TOP_W / 2);
    expect(mouthToTopX(26, 'left')).toBeLessThan(TOP_W / 2);
  });
});

describe('ataque de grito (função pura)', () => {
  it('intervalos entre 12 e 35 s', () => {
    for (let s = 1; s < 40; s++) for (let i = 0; i < 20; i++) {
      const g = screamGap(s, i);
      expect(g).toBeGreaterThanOrEqual(SCREAM_MIN_GAP_MS);
      expect(g).toBeLessThanOrEqual(SCREAM_MAX_GAP_MS);
    }
  });

  it('começa calado, grita por 3,5 s e volta a ficar calado', () => {
    const start = 1_000_000, seed = 42;
    const s0 = screamState(start, start, seed);
    expect(s0.screaming).toBe(false);
    const g = screamGap(seed, 0);
    expect(s0.until).toBe(start + g);
    expect(screamState(start + g - 1, start, seed).screaming).toBe(false);
    const on = screamState(start + g, start, seed);
    expect(on).toEqual({ screaming: true, until: start + g + SCREAM_DURATION_MS });
    expect(screamState(start + g + SCREAM_DURATION_MS, start, seed).screaming).toBe(false);
  });

  it('é determinística e a fração do tempo gritando é pequena', () => {
    const start = 0, seed = 7;
    let screaming = 0;
    const N = 3600;
    for (let t = 0; t < N; t++) if (screamState(t * 1000, start, seed).screaming) screaming++;
    expect(screaming).toBeGreaterThan(N * 0.05);
    expect(screaming).toBeLessThan(N * 0.3);
    expect(screamState(123_456, start, seed)).toEqual(screamState(123_456, start, seed));
  });

  it('tremedeira fica entre -1 e 1 e varia', () => {
    const seen = new Set<string>();
    for (let t = 0; t < 50; t++) {
      const o = shakeOffset(t);
      expect(Math.abs(o.x)).toBeLessThanOrEqual(1);
      expect(Math.abs(o.y)).toBeLessThanOrEqual(1);
      seen.add(`${o.x},${o.y}`);
    }
    expect(seen.size).toBeGreaterThan(3);
  });
});

describe('dúvida aleatória e plaquinha de update', () => {
  it('doubtGap fica entre 20 e 60 s e é determinístico', () => {
    for (let i = 0; i < 200; i++) {
      const g = doubtGap(7, i);
      expect(g).toBeGreaterThanOrEqual(DOUBT_MIN_GAP_MS);
      expect(g).toBeLessThanOrEqual(DOUBT_MAX_GAP_MS);
      expect(doubtGap(7, i)).toBe(g);
    }
  });

  it('doubtState alterna espera e ~3 s de dúvida', () => {
    const seed = 3, start = 1000;
    const a = doubtState(start, start, seed);
    expect(a.doubting).toBe(false);
    expect(a.until - start).toBe(doubtGap(seed, 0));
    const b = doubtState(a.until, start, seed);
    expect(b.doubting).toBe(true);
    expect(b.until - a.until).toBe(DOUBT_DURATION_MS);
    const c = doubtState(b.until, start, seed);
    expect(c.doubting).toBe(false);
    expect(c.until - b.until).toBe(doubtGap(seed, 1));
    expect(doubtState(a.until - 1, start, seed).doubting).toBe(false);
  });

  it('o efeito question existe e o humor duvida o usa', async () => {
    const { EFFECTS } = await import('./effects.ts');
    const { SKINS } = await import('./index.ts');
    expect(EFFECTS.question.frames.length).toBeGreaterThanOrEqual(3);
    expect(SKINS.duvida.effect).toBe('question');
  });

  it('a plaquinha de update difere da de água e cabe na faixa', () => {
    const w = renderSign('Nova versão disponível!', 'water');
    const u = renderSign('Nova versão disponível!', 'update');
    expect(u.w).toBe(TOP_W);
    expect(u.h).toBe(TOP_H);
    expect(Buffer.compare(Buffer.from(w.data), Buffer.from(u.data))).not.toBe(0);
    expect(u.data.some((v, i) => i % 4 === 3 && v > 0)).toBe(true);
  });

  it('postura, alongar e vista têm ícone próprio de 7 px, diferente da gotinha e entre si', () => {
    const kinds = ['water', 'posture', 'stretch', 'eyes'] as const;
    for (const k of kinds) {
      const { grid, pal } = signIcon(k);
      for (const row of grid) {
        expect(row.length, k).toBe(7);
        for (const ch of row) if (ch !== '.') expect(pal[ch], `${k} cor ${ch}`).toBeDefined();
      }
    }
    const sigs = kinds.map((k) => Buffer.from(renderSign('Ajeite a postura!', k).data).toString('base64'));
    expect(new Set(sigs).size).toBe(kinds.length);
  });
});
