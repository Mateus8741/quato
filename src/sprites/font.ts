// Fonte pixel própria (letras de 5 de altura, largura variável 1..5), usada na plaquinha e no balão.
// Cada célula tem 8 linhas: 2 para o acento, 5 da letra e 1 para a parte de baixo (vírgula, cedilha).
// Minúsculas usam o desenho das maiúsculas.

/** Letras: 5 linhas ('#' = tinta); uma 6ª linha opcional desce abaixo da base. */
export const GLYPHS: Readonly<Record<string, readonly string[]>> = {
  A: ['.##.', '#..#', '####', '#..#', '#..#'],
  B: ['###.', '#..#', '###.', '#..#', '###.'],
  C: ['.###', '#...', '#...', '#...', '.###'],
  D: ['###.', '#..#', '#..#', '#..#', '###.'],
  E: ['####', '#...', '###.', '#...', '####'],
  F: ['####', '#...', '###.', '#...', '#...'],
  G: ['.###', '#...', '#.##', '#..#', '.###'],
  H: ['#..#', '#..#', '####', '#..#', '#..#'],
  I: ['###', '.#.', '.#.', '.#.', '###'],
  J: ['..##', '...#', '...#', '#..#', '.##.'],
  K: ['#..#', '#.#.', '##..', '#.#.', '#..#'],
  L: ['#...', '#...', '#...', '#...', '####'],
  M: ['#...#', '##.##', '#.#.#', '#...#', '#...#'],
  N: ['#..#', '##.#', '#.##', '#..#', '#..#'],
  O: ['.##.', '#..#', '#..#', '#..#', '.##.'],
  P: ['###.', '#..#', '###.', '#...', '#...'],
  Q: ['.##.', '#..#', '#..#', '#.##', '.###'],
  R: ['###.', '#..#', '###.', '#.#.', '#..#'],
  S: ['.###', '#...', '.##.', '...#', '###.'],
  T: ['###', '.#.', '.#.', '.#.', '.#.'],
  U: ['#..#', '#..#', '#..#', '#..#', '.##.'],
  V: ['#.#', '#.#', '#.#', '#.#', '.#.'],
  W: ['#...#', '#...#', '#.#.#', '##.##', '#...#'],
  X: ['#..#', '#..#', '.##.', '#..#', '#..#'],
  Y: ['#.#', '#.#', '.#.', '.#.', '.#.'],
  Z: ['####', '...#', '.##.', '#...', '####'],
  '0': ['.##.', '#.##', '##.#', '#..#', '.##.'],
  '1': ['.#.', '##.', '.#.', '.#.', '###'],
  '2': ['###.', '...#', '.##.', '#...', '####'],
  '3': ['###.', '...#', '.##.', '...#', '###.'],
  '4': ['#..#', '#..#', '####', '...#', '...#'],
  '5': ['####', '#...', '###.', '...#', '###.'],
  '6': ['.##.', '#...', '###.', '#..#', '.##.'],
  '7': ['####', '...#', '..#.', '.#..', '.#..'],
  '8': ['.##.', '#..#', '.##.', '#..#', '.##.'],
  '9': ['.##.', '#..#', '.###', '...#', '.##.'],
  '!': ['#', '#', '#', '.', '#'],
  '?': ['###.', '...#', '.##.', '....', '.#..'],
  '.': ['.', '.', '.', '.', '#'],
  ',': ['.', '.', '.', '.', '#', '#'],
  '-': ['...', '...', '###', '...', '...'],
  ':': ['.', '#', '.', '#', '.'],
  "'": ['#', '#', '.', '.', '.'],
  ' ': ['..', '..', '..', '..', '..'],
};

type Mark = 'acute' | 'grave' | 'circ' | 'tilde' | 'cedilla';

/** Marcas de 3 de largura e 2 de altura (centralizadas em cima da letra; a cedilha vai embaixo). */
const MARKS: Readonly<Record<Mark, readonly string[]>> = {
  acute: ['..#', '.#.'],
  grave: ['#..', '.#.'],
  circ: ['.#.', '#.#'],
  tilde: ['.##', '#..'],
  cedilla: ['.#.'],
};

const ACCENTED: Readonly<Record<string, readonly [string, Mark]>> = {
  'Á': ['A', 'acute'], 'À': ['A', 'grave'], 'Â': ['A', 'circ'], 'Ã': ['A', 'tilde'],
  'É': ['E', 'acute'], 'Ê': ['E', 'circ'], 'Í': ['I', 'acute'],
  'Ó': ['O', 'acute'], 'Ô': ['O', 'circ'], 'Õ': ['O', 'tilde'], 'Ú': ['U', 'acute'], 'Ç': ['C', 'cedilla'],
};

export const CELL_H = 8;
export const ACCENT_ROWS = 2;
export const LINE_H = CELL_H + 1;
export const LETTER_GAP = 1;

export interface Glyph {
  width: number;
  /** CELL_H linhas de `width` caracteres. */
  rows: readonly string[];
}

/** Maiúscula, com acentos mantidos ('á' -> 'Á'). */
export function normalizeChar(ch: string): string {
  return ch.toLocaleUpperCase('pt-BR');
}

export function hasGlyph(ch: string): boolean {
  const c = normalizeChar(ch);
  return c in GLYPHS || c in ACCENTED;
}

const cache = new Map<string, Glyph>();

export function glyph(ch: string): Glyph | undefined {
  const c = normalizeChar(ch);
  const hit = cache.get(c);
  if (hit) return hit;
  const acc = ACCENTED[c];
  const base = GLYPHS[acc ? acc[0] : c];
  if (!base) return undefined;
  const width = base[0]!.length;
  const rows = Array.from({ length: CELL_H }, () => Array<string>(width).fill('.'));
  base.forEach((r, j) => [...r].forEach((p, i) => { if (p === '#') rows[ACCENT_ROWS + j]![i] = '#'; }));
  if (acc) {
    const mark = MARKS[acc[1]];
    const ox = Math.floor((width - 3) / 2);
    const oy = acc[1] === 'cedilla' ? ACCENT_ROWS + 5 : 0;
    mark.forEach((r, j) => [...r].forEach((p, i) => {
      const x = ox + i;
      if (p === '#' && x >= 0 && x < width) rows[oy + j]![x] = '#';
    }));
  }
  const g = { width, rows: rows.map((r) => r.join('')) };
  cache.set(c, g);
  return g;
}

/** Caracteres do texto que a fonte não sabe desenhar. */
export function unknownChars(text: string): string[] {
  return [...new Set([...text].filter((c) => !hasGlyph(c)))];
}

export function textWidth(text: string): number {
  let w = 0;
  for (const ch of text) w += (glyph(ch)?.width ?? 0) + LETTER_GAP;
  return Math.max(0, w - LETTER_GAP);
}

/** Quebra por palavras para caber em `maxWidth` (palavras longas demais são cortadas por letra). */
export function wrapText(text: string, maxWidth: number): string[] {
  const lines: string[] = [];
  let cur = '';
  const push = () => { if (cur) lines.push(cur); cur = ''; };
  for (const word of text.trim().split(/\s+/)) {
    const tryLine = cur ? `${cur} ${word}` : word;
    if (textWidth(tryLine) <= maxWidth) { cur = tryLine; continue; }
    push();
    if (textWidth(word) <= maxWidth) { cur = word; continue; }
    for (const ch of word) {
      if (textWidth(cur + ch) > maxWidth) push();
      cur += ch;
    }
  }
  push();
  return lines;
}

/** Desenha uma linha de texto chamando `plot` para cada pixel de tinta; `jitter(i)` desloca a letra i em y. */
export function drawText(
  text: string,
  x: number,
  y: number,
  plot: (x: number, y: number) => void,
  jitter?: (i: number) => number,
): void {
  let cx = x;
  [...text].forEach((ch, i) => {
    const g = glyph(ch);
    if (!g) return;
    const dy = jitter ? jitter(i) : 0;
    g.rows.forEach((row, j) => [...row].forEach((p, k) => { if (p === '#') plot(cx + k, y + j + dy); }));
    cx += g.width + LETTER_GAP;
  });
}

/**
 * Como `wrapText`, mas, se precisar de 2 linhas e houver espaços, escolhe a quebra que deixa as
 * linhas mais parecidas ("HORA DE / BEBER ÁGUA!" em vez de "HORA DE BEBER / ÁGUA!").
 */
export function wrapBalanced(text: string, maxWidth: number, maxLines = 2): string[] {
  const greedy = wrapText(text, maxWidth);
  if (greedy.length !== 2 || maxLines < 2) return greedy.slice(0, maxLines);
  const words = text.trim().split(/\s+/);
  let best = greedy;
  let bestW = Math.max(...greedy.map(textWidth));
  for (let i = 1; i < words.length; i++) {
    const a = words.slice(0, i).join(' '), b = words.slice(i).join(' ');
    const w = Math.max(textWidth(a), textWidth(b));
    if (w <= maxWidth && w < bestW) { best = [a, b]; bestW = w; }
  }
  return best;
}
