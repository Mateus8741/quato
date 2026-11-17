import type { EffectName, PixelGrid } from './types.ts';

// Cada efeito é uma lista de quadros pequenos desenhados perto da cabeça (ou da ponta da faquinha).
// Chaves: Y brilho, R vapor, T lágrima, G ponto, Z letra z, N/Q notas musicais, D/d gota vermelha.
export const EFFECT_COLORS: Readonly<Record<string, string>> = {
  Y: '#fff7b0', R: '#ff5a4a', T: '#6ec1ff', G: '#8a8a99', Z: '#6a7fd6',
  N: '#7b5cff', Q: '#ff5fa2', D: '#e3222c', d: '#a3121b',
  P: '#7b5cff', p: '#b7a6ff', V: '#ffffff',
};

/** Onde a gota se forma, em relação a `frame.grip`: logo abaixo da ponta da lâmina da faquinha (a lâmina em accessories.ts termina em grip + 7). */
export const KNIFE_TIP = { dx: 7, dy: 0 } as const;

export interface EffectDef {
  frames: readonly PixelGrid[];
  /** posição em relação ao olho (ou à ponta da faquinha, se `anchor: 'tip'`), olhando para a direita */
  dx: number;
  dy: number;
  fps: number;
  anchor?: 'eye' | 'tip';
}

// ♪ (cabeça embaixo, haste e bandeirinha)
const NOTE = ['.X..', '.XX.', '.X.X', 'XX..', 'XX..'];

/** Duas notinhas subindo em zigue-zague, defasadas. */
function notesFrames(): PixelGrid[] {
  const W = 11, H = 12, n = 6;
  const out: PixelGrid[] = [];
  for (let f = 0; f < n; f++) {
    const g = Array.from({ length: H }, () => Array<string>(W).fill('.'));
    const put = (ox: number, oy: number, ch: string) => {
      NOTE.forEach((row, j) => [...row].forEach((c, i) => {
        const y = oy + j, x = ox + i;
        if (c === 'X' && y >= 0 && y < H && x >= 0 && x < W) g[y]![x] = ch;
      }));
    };
    // nota A sobe de y=7 até sumir; nota B começa no meio do caminho
    const a = f, b = (f + 3) % n;
    if (a < 5) put(1 + (a % 2), 7 - a * 2, 'N');
    if (b < 5) put(5 - (b % 2), 7 - b * 2, 'Q');
    out.push(g.map((r) => r.join('')));
  }
  return out;
}

/** Gota que se forma embaixo da ponta da faquinha suja de sangue, cai e some. */
const DRIP: PixelGrid[] = [
  ['.', '.', '.', '.', '.', '.'],
  ['D', '.', '.', '.', '.', '.'],
  ['D', 'D', '.', '.', '.', '.'],
  ['D', 'D', 'd', '.', '.', '.'],
  ['.', 'D', 'D', '.', '.', '.'],
  ['.', '.', 'D', 'd', '.', '.'],
  ['.', '.', '.', 'D', 'd', '.'],
  ['.', '.', '.', '.', '.', 'D'],
];

// '?' 5x7 roxo com contorno branco (balançando 1 px e quicando em 3 quadros).
const Q_GLYPH = ['.PPP.', 'P...P', '....P', '...P.', '..P..', '.....', '..P..'];

function questionFrame(ox: number, oy: number): PixelGrid {
  const W = 9, H = 10;
  const g = Array.from({ length: H }, () => Array<string>(W).fill('.'));
  for (let j = 0; j < 7; j++) for (let i = 0; i < 5; i++) {
    if (Q_GLYPH[j]![i] !== 'P') continue;
    for (let dy = -1; dy <= 1; dy++) for (let dx = -1; dx <= 1; dx++) {
      const y = oy + 1 + j + dy, x = ox + 1 + i + dx;
      if (g[y]![x] === '.') g[y]![x] = 'V';
    }
  }
  for (let j = 0; j < 7; j++) for (let i = 0; i < 5; i++) if (Q_GLYPH[j]![i] === 'P') g[oy + 1 + j]![ox + 1 + i] = 'P';
  return g.map((r) => r.join(''));
}

export const EFFECTS: Readonly<Record<EffectName, EffectDef>> = {
  sparkle: {
    frames: [['.Y.', 'YYY', '.Y.'], ['...', '.Y.', '...'], ['Y.Y', '.Y.', 'Y.Y']],
    dx: -9, dy: -7, fps: 3,
  },
  steam: {
    frames: [['R.R', '.R.', 'R.R'], ['.R.', 'R.R', '.R.']],
    dx: -2, dy: -8, fps: 6,
  },
  tear: {
    frames: [['T', '.'], ['T', 'T'], ['.', 'T'], ['.', '.']],
    dx: 2, dy: 3, fps: 2,
  },
  dots: {
    frames: [['G..', '...'], ['G.G', '...'], ['G.G', '..G'], ['...', '...']],
    dx: -4, dy: -7, fps: 2,
  },
  zzz: {
    frames: [['ZZZ', '.Z.', 'ZZZ'], ['..ZZ', '...Z', '..ZZ'], ['.....', '.....', '.....']],
    dx: 2, dy: -9, fps: 1.5,
  },
  notes: { frames: notesFrames(), dx: -18, dy: -9, fps: 4 },
  question: {
    frames: [questionFrame(1, 1), questionFrame(1, 0), questionFrame(2, 1), questionFrame(1, 0)],
    dx: -14, dy: -9, fps: 3,
  },
  drip: { frames: DRIP, dx: 0, dy: 0, fps: 3, anchor: 'tip' },
};

/** Pixels do efeito no quadro 32x32 já virado para `facing` (usado pelo renderer e pelas prévias). */
export function effectPixels(
  effect: EffectName,
  frame: { eye: { x: number; y: number }; grip: { x: number; y: number } },
  facing: 'left' | 'right',
  effectFrame: number,
  /** só pinga com a "faca-sangue" equipada; sem ela o efeito 'drip' não desenha nada */
  bloodyKnife = false,
): { x: number; y: number; color: string }[] {
  const def = EFFECTS[effect];
  if (def.anchor === 'tip' && !bloodyKnife) return [];
  const fx = def.frames[effectFrame % def.frames.length]!;
  const base = def.anchor === 'tip' ? { x: frame.grip.x + KNIFE_TIP.dx, y: frame.grip.y + KNIFE_TIP.dy } : frame.eye;
  const ox = base.x + def.dx;
  const oy = base.y + def.dy;
  const out: { x: number; y: number; color: string }[] = [];
  fx.forEach((row, j) => {
    for (let i = 0; i < row.length; i++) {
      const color = EFFECT_COLORS[row[i]!];
      if (!color) continue;
      out.push({ x: facing === 'left' ? 31 - (ox + i) : ox + i, y: oy + j, color });
    }
  });
  return out;
}
