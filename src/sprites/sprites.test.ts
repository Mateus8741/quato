import { readFileSync } from 'node:fs';
import { describe, expect, it } from 'vitest';
import { ACCESSORIES, ANIMS, ANIM_NAMES, COLORS, LIZARD_ANIMS, LIZARD_SCREAM_ANIMS, MOODS, SKINS, validateGrid } from './index.ts';
import { BASE_PALETTE } from './colors.ts';
import { composeRGBA, paletteFor } from './compose.ts';
import { EFFECTS, effectPixels } from './effects.ts';
import { FEATHER_DUCK, FEATHER_LIZARD } from './gesture.ts';
import type { Frame } from './types.ts';

/** Todos os quadros dos dois personagens (pato, lagartinho e lagartinho gritando). */
const ALL_SETS = { pato: ANIMS, lagarto: LIZARD_ANIMS, grito: LIZARD_SCREAM_ANIMS } as const;
const allFrames = (): [string, Frame][] =>
  Object.entries(ALL_SETS).flatMap(([set, anims]) => Object.entries(anims).flatMap(([n, fs]) => fs.map((f, i): [string, Frame] => [`${set}/${n}/${i}`, f])));

describe('sprites', () => {
  it('todo humor tem skin e efeito', () => {
    for (const m of MOODS) {
      expect(SKINS[m]).toBeDefined();
      expect(EFFECTS[SKINS[m].effect]).toBeDefined();
    }
  });

  it('todos os quadros são 32x32 e usam só cores da paleta, em todas as cores', () => {
    for (const c of COLORS) {
      for (const m of MOODS) {
        const pal = paletteFor(SKINS[m], { color: c.id, accessories: [] });
        const set = m === 'lagarto' ? { ...LIZARD_ANIMS, ...Object.fromEntries(Object.entries(LIZARD_SCREAM_ANIMS).map(([k, v]) => [`grito-${k}`, v])) } : ANIMS;
        for (const [name, frames] of Object.entries(set)) {
          frames.forEach((f, i) => expect(validateGrid(f.grid, pal), `${c.id}/${m}/${name}/${i}`).toEqual([]));
        }
      }
    }
  });

  it('o olho fica dentro do quadro', () => {
    for (const [, f] of allFrames()) {
      {
        expect(f.eye.x).toBeGreaterThan(0);
        expect(f.eye.x).toBeLessThan(30);
        expect(f.eye.y).toBeGreaterThan(0);
        expect(f.eye.y).toBeLessThan(30);
      }
    }
  });

  it('cada cor define as chaves do corpo e muda de fato o corpo', () => {
    for (const c of COLORS) {
      expect(c.palette.B, c.id).toMatch(/^#[0-9a-f]{6}$/);
      expect(c.palette.b, c.id).toMatch(/^#[0-9a-f]{6}$/);
    }
    const bodies = new Set(COLORS.map((c) => c.palette.B));
    expect(bodies.size).toBe(COLORS.length);
  });

  it('os olhos e sobrancelhas do humor só usam chaves existentes', () => {
    for (const m of MOODS) {
      const s = SKINS[m];
      const pal = { ...BASE_PALETTE, C: s.cheekColor };
      for (const g of [s.eyes, s.eyesClosed, s.brow?.grid ?? []]) {
        for (const row of g) for (const ch of row) expect(ch === '.' || ch in pal, `${m} "${ch}"`).toBe(true);
      }
    }
  });
});

describe('acessórios', () => {
  it('grades retangulares e com cores definidas', () => {
    for (const a of ACCESSORIES) {
      const w = a.grid[0]!.length;
      for (const row of a.grid) {
        expect(row.length, `${a.id} linha`).toBe(w);
        for (const ch of row) expect(ch === '.' || ch in a.palette, `${a.id} "${ch}"`).toBe(true);
      }
    }
  });

  it('cabem no quadro 32x32 em todas as animações', () => {
    for (const a of ACCESSORIES) {
      for (const [, f] of allFrames()) {
        {
          const o = a.layer === 'hand' ? f.grip : f.eye;
          const x0 = o.x + a.dx, y0 = o.y + a.dy;
          expect(x0, `${a.id} x0`).toBeGreaterThanOrEqual(0);
          expect(y0, `${a.id} y0`).toBeGreaterThanOrEqual(0);
          expect(x0 + a.grid[0]!.length, `${a.id} x1`).toBeLessThanOrEqual(32);
          expect(y0 + a.grid.length, `${a.id} y1`).toBeLessThanOrEqual(32);
        }
      }
    }
  });

  it('usar um acessório muda os pixels', () => {
    const f = ANIMS.idle[0]!;
    const base = composeRGBA(SKINS.feliz, f, false, { color: 'amarelo', accessories: [] });
    for (const a of ACCESSORIES) {
      const out = composeRGBA(SKINS.feliz, f, false, { color: 'amarelo', accessories: [a.id] });
      expect(Buffer.compare(Buffer.from(out), Buffer.from(base)), a.id).not.toBe(0);
    }
  });
});

describe('espelho com o Rust (src-tauri/src/look.rs)', () => {
  const rust = readFileSync(new URL('../../src-tauri/src/look.rs', import.meta.url), 'utf8');

  it('mesmas cores', () => {
    const m = /pub const COLORS: \[&str; \d+\] = \[([^\]]+)\]/.exec(rust)!;
    const ids = [...m[1]!.matchAll(/"([^"]+)"/g)].map((x) => x[1]);
    expect(ids).toEqual(COLORS.map((c) => c.id));
  });

  it('mesmos acessórios', () => {
    const block = /pub const ACCESSORIES[^=]*= \[([\s\S]*?)\];/.exec(rust)![1]!;
    const ids = [...block.matchAll(/\("([^"]+)",\s*"([^"]+)"\)/g)].map((x) => x[1]);
    expect(ids.sort()).toEqual(ACCESSORIES.map((a) => a.id).sort());
  });
});

describe('v2: dança, lagartinho, pena do meio', () => {
  it('toda animação existe para o pato e o lagartinho; dança tem >= 4 quadros', () => {
    for (const set of Object.values(ALL_SETS)) {
      for (const n of ANIM_NAMES) expect(set[n].length, n).toBeGreaterThan(0);
      expect(set.dance.length).toBeGreaterThanOrEqual(4);
    }
    for (const m of MOODS) for (const n of ANIM_NAMES) expect(SKINS[m].fps[n], `${m}/${n}`).toBeGreaterThan(0);
    expect(SKINS.danca.fps.dance).toBeGreaterThanOrEqual(6);
    expect(SKINS.danca.effect).toBe('notes');
    expect(SKINS.lagarto.effect).toBe('drip');
  });

  it('os quadros da dança são diferentes entre si (anima de verdade)', () => {
    expect(new Set(ANIMS.dance.map((f) => f.grid.join(''))).size).toBeGreaterThanOrEqual(3);
  });

  it('o lagartinho ignora a cor escolhida e usa a paleta verde', () => {
    const a = composeRGBA(SKINS.lagarto, LIZARD_ANIMS.idle[0]!, false, { color: 'rosa', accessories: [] });
    const b = composeRGBA(SKINS.lagarto, LIZARD_ANIMS.idle[0]!, false, { color: 'preto', accessories: [] });
    expect(Buffer.compare(Buffer.from(a), Buffer.from(b))).toBe(0);
    const duck = composeRGBA(SKINS.feliz, ANIMS.idle[0]!, false, { color: 'amarelo', accessories: [] });
    expect(Buffer.compare(Buffer.from(a), Buffer.from(duck))).not.toBe(0);
  });

  it('a faca saiu do corpo do lagartinho: sem acessório não há lâmina nem gota', () => {
    for (const [name, f] of allFrames()) {
      if (name.startsWith('pato')) continue;
      const all = f.grid.join('');
      expect(all, name).not.toContain('L');
      expect(all, name).not.toContain('R');
    }
    const f = LIZARD_ANIMS.idle[0]!;
    for (let k = 0; k < EFFECTS.drip.frames.length; k++) expect(effectPixels('drip', f, 'right', k)).toEqual([]);
  });

  it('só a faca-sangue faz o lagartinho pingar, e a gota sai da ponta da lâmina', () => {
    const f = LIZARD_ANIMS.idle[0]!;
    const all = EFFECTS.drip.frames.flatMap((_, k) => effectPixels('drip', f, 'right', k, true));
    expect(all.length).toBeGreaterThan(0);
    for (const p of all) expect(p.x).toBeGreaterThanOrEqual(f.grip.x + 7);
    const clean = composeRGBA(SKINS.lagarto, f, false, { color: 'amarelo', accessories: ['faca'] });
    const bloody = composeRGBA(SKINS.lagarto, f, false, { color: 'amarelo', accessories: ['faca-sangue'] });
    expect(Buffer.compare(Buffer.from(clean), Buffer.from(bloody))).not.toBe(0);
  });

  it('todo quadro tem grip dentro do quadro nos dois personagens', () => {
    for (const [name, f] of allFrames()) {
      expect(f.grip.x, name).toBeGreaterThan(0);
      expect(f.grip.x, name).toBeLessThan(31);
      expect(f.grip.y, name).toBeGreaterThan(0);
      expect(f.grip.y, name).toBeLessThan(31);
    }
  });

  it('arminha: a ponta do cano cai em (29,20) ±1 no pato e no lagartinho em repouso', () => {
    const gun = ACCESSORIES.find((a) => a.id === 'arminha')!;
    expect(gun.layer).toBe('hand');
    for (const f of [ANIMS.idle[0]!, LIZARD_ANIMS.idle[0]!]) {
      const buf = composeRGBA(SKINS.feliz, f, false, { color: 'amarelo', accessories: ['arminha'] });
      const base = composeRGBA(SKINS.feliz, f, false);
      // ponta = pixel mais à direita da cor do cano (#8a90a6/#4a4e5e) que a arma acrescentou na linha do cano
      let maxX = -1, maxY = -1;
      for (let y = 0; y < 32; y++) for (let x = 0; x < 32; x++) {
        const k = (y * 32 + x) * 4;
        const isMetal = (buf[k] === 0x4a && buf[k + 1] === 0x4e) || (buf[k] === 0x8a && buf[k + 1] === 0x90);
        if (isMetal && x > maxX) { maxX = x; maxY = y; }
      }
      void base;
      expect(Math.abs(maxX - 29)).toBeLessThanOrEqual(1);
      expect(Math.abs(maxY - 20)).toBeLessThanOrEqual(1);
    }
  });

  it('itens de mão ficam por cima do corpo e acompanham o grip', () => {
    for (const id of ['faca', 'faca-sangue', 'arminha', 'varinha']) {
      expect(ACCESSORIES.find((a) => a.id === id)!.layer, id).toBe('hand');
    }
    const a = LIZARD_ANIMS.run[0]!, b = LIZARD_ANIMS.run[1]!;
    expect(a.grip.y).not.toBe(b.grip.y);
  });

  it('grito abre a boca (cor M em todos os quadros do grito, menos dormindo)', () => {
    for (const [name, f] of allFrames()) if (name.startsWith('grito') && !name.includes('/sleep/')) expect(f.grid.join('').split('M').length - 1, name).toBeGreaterThan(20);
  });

  it('efeitos (inclusive a gota e as notas) ficam dentro do quadro', () => {
    for (const m of MOODS) {
      const sets = m === 'lagarto' ? [LIZARD_ANIMS, LIZARD_SCREAM_ANIMS] : [ANIMS];
      for (const set of sets) for (const frames of Object.values(set)) for (const f of frames) {
        for (let k = 0; k < EFFECTS[SKINS[m].effect].frames.length; k++) {
          for (const facing of ['left', 'right'] as const) {
            for (const p of effectPixels(SKINS[m].effect, f, facing, k)) {
              expect(p.x, `${m} x`).toBeGreaterThanOrEqual(0);
              expect(p.x, `${m} x`).toBeLessThan(32);
              expect(p.y, `${m} y`).toBeGreaterThanOrEqual(0);
              expect(p.y, `${m} y`).toBeLessThan(32);
            }
          }
        }
      }
    }
  });

  it('pena do meio: a do meio é a mais alta e o gesto cabe no quadro', () => {
    for (const g of [FEATHER_DUCK, FEATHER_LIZARD]) {
      const w = g[0]!.length;
      for (const row of g) expect(row.length).toBe(w);
      const top = (x0: number, x1: number) => g.findIndex((row) => row.slice(x0, x1).replace(/\./g, '') !== '');
      const third = Math.floor(w / 3);
      expect(top(third, w - third)).toBeLessThan(top(0, third));
      expect(top(third, w - third)).toBeLessThan(top(w - third, w));
    }
    for (const [name, f] of allFrames()) {
      const g = name.startsWith('pato') ? FEATHER_DUCK : FEATHER_LIZARD;
      expect(f.hand.x, name).toBeGreaterThanOrEqual(0);
      expect(f.hand.y, name).toBeGreaterThanOrEqual(0);
      expect(f.hand.x + g[0]!.length, name).toBeLessThanOrEqual(32);
      expect(f.hand.y + g.length, name).toBeLessThanOrEqual(32);
    }
  });

  it('o gesto muda os pixels nos dois personagens', () => {
    for (const [skin, f] of [[SKINS.feliz, ANIMS.idle[0]!], [SKINS.lagarto, LIZARD_ANIMS.idle[0]!]] as const) {
      const a = composeRGBA(skin, f, false);
      const b = composeRGBA(skin, f, false, undefined, { gesture: true });
      expect(Buffer.compare(Buffer.from(a), Buffer.from(b))).not.toBe(0);
    }
  });
});
