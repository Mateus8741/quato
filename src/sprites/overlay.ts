// Faixa de cima da janela (192x64 na tela = 96x32 pixels de arte, ampliada x2):
// a plaquinha de madeira do lembrete de água e o balão do grito do lagartinho.
// Funções puras que devolvem RGBA 96x32 (o renderer só copia para o canvas e anima a escala).
import { LINE_H, drawText, textWidth, wrapBalanced, wrapText } from './font.ts';
import { hash01, shakeOffset } from './scream.ts';

export const TOP_W = 96;
export const TOP_H = 32;
/** Ampliação da faixa de cima na tela. */
export const TOP_SCALE = 2;

export interface Pix {
  w: number;
  h: number;
  data: Uint8ClampedArray;
}

export function newPix(w = TOP_W, h = TOP_H): Pix {
  return { w, h, data: new Uint8ClampedArray(w * h * 4) };
}

function rgb(hex: string): [number, number, number] {
  const n = parseInt(hex.slice(1), 16);
  return [(n >> 16) & 255, (n >> 8) & 255, n & 255];
}

export function plot(p: Pix, x: number, y: number, hex: string): void {
  if (x < 0 || y < 0 || x >= p.w || y >= p.h) return;
  const [r, g, b] = rgb(hex);
  const k = (y * p.w + x) * 4;
  p.data[k] = r; p.data[k + 1] = g; p.data[k + 2] = b; p.data[k + 3] = 255;
}

function fill(p: Pix, x: number, y: number, w: number, h: number, hex: string): void {
  for (let j = 0; j < h; j++) for (let i = 0; i < w; i++) plot(p, x + i, y + j, hex);
}

/** Retângulo com cantos arredondados (1 px) e contorno. */
function roundBox(p: Pix, x: number, y: number, w: number, h: number, fillHex: string, lineHex: string): void {
  fill(p, x + 1, y, w - 2, h, lineHex);
  fill(p, x, y + 1, w, h - 2, lineHex);
  fill(p, x + 1, y + 1, w - 2, h - 2, fillHex);
}

function stampGrid(p: Pix, grid: readonly string[], x: number, y: number, pal: Readonly<Record<string, string>>): void {
  grid.forEach((row, j) => [...row].forEach((ch, i) => { const c = pal[ch]; if (c) plot(p, x + i, y + j, c); }));
}

const DROP = [
  '...o...',
  '..owo..',
  '..owo..',
  '.owwwo.',
  'owlwwwo',
  'owlwwwo',
  'owwwwwo',
  '.owwwo.',
  '..ooo..',
];
const DROP_PAL = { o: '#1d4f8c', w: '#4fb0ff', l: '#d6f0ff' };

// Ícone da plaquinha de atualização: seta verde para baixo com estrelinha amarela.
const UPDATE = [
  '.y...y.',
  '..ggg..',
  '..gLg..',
  '..gLg..',
  'ggggggg',
  '.gLLLg.',
  '..gLg..',
  '...g...',
  'yyyyyyy',
];
const UPDATE_PAL = { g: '#1f7a3a', L: '#7be08f', y: '#ffd23f' };
export type SignKind = 'water' | 'update';

const WOOD = { line: '#4a2a12', board: '#d9a066', light: '#f0c48c', grain: '#bf844c', nail: '#6b4423', post: '#a8693a', ink: '#4a2a12' };

export interface SignLayout {
  lines: string[];
  /** caixa da placa e ponto de apoio (embaixo, no meio do pauzinho) */
  x: number;
  y: number;
  w: number;
  h: number;
  anchorX: number;
  anchorY: number;
}

export function signLayout(text: string): SignLayout {
  const lines = wrapBalanced(text, 62, 2);
  const tw = Math.max(...lines.map(textWidth), 1);
  const w = 3 + DROP[0]!.length + 3 + tw + 4;
  const textH = lines.length * LINE_H - 1 - 2; // tira as 2 linhas de acento da primeira linha
  const h = Math.max(textH + 6, DROP.length + 6);
  const x = Math.round((TOP_W - w) / 2);
  return { lines, x, y: 1, w, h, anchorX: TOP_W / 2, anchorY: TOP_H };
}

/** Plaquinha de madeira com gotinha azul e o texto (até 2 linhas), num pauzinho que desce até o pato. */
export function renderSign(text: string, kind: SignKind = 'water'): Pix {
  const p = newPix();
  const L = signLayout(text);
  const icon = kind === 'update' ? UPDATE : DROP;
  const iconPal = kind === 'update' ? UPDATE_PAL : DROP_PAL;
  // pauzinho
  fill(p, L.anchorX - 2, L.y + L.h - 1, 4, TOP_H - (L.y + L.h - 1), WOOD.line);
  fill(p, L.anchorX - 1, L.y + L.h - 1, 2, TOP_H - (L.y + L.h - 1), WOOD.post);
  // tábua
  roundBox(p, L.x, L.y, L.w, L.h, WOOD.board, WOOD.line);
  fill(p, L.x + 2, L.y + 1, L.w - 4, 1, WOOD.light);
  fill(p, L.x + 1, L.y + L.h - 2, L.w - 2, 1, WOOD.grain);
  // veios da madeira (curtinhos, fora do texto)
  for (const [gx, gy] of [[L.x + 2, L.y + 4], [L.x + L.w - 6, L.y + L.h - 4], [L.x + 4, L.y + L.h - 4]] as const) {
    fill(p, gx, gy, 3, 1, WOOD.grain);
  }
  // pregos
  for (const nx of [L.x + 2, L.x + L.w - 3]) {
    plot(p, nx, L.y + 2, WOOD.nail);
    plot(p, nx, L.y + L.h - 3, WOOD.nail);
  }
  // gotinha
  stampGrid(p, icon, L.x + 3, L.y + Math.floor((L.h - icon.length) / 2), iconPal);
  // texto (centralizado na área à direita da gota)
  const tx0 = L.x + 3 + DROP[0]!.length + 3;
  const areaW = L.w - (tx0 - L.x) - 4;
  const textTop = L.y + Math.round((L.h - (L.lines.length * LINE_H - 1 - 2)) / 2) - 2;
  L.lines.forEach((line, i) => {
    const tx = tx0 + Math.floor((areaW - textWidth(line)) / 2);
    drawText(line, tx, textTop + i * LINE_H, (x, y) => plot(p, x, y, WOOD.ink));
  });
  return p;
}

const BALLOON = { fill: '#ffffff', line: '#2a1a10', ink: '#d8262b' };

/**
 * Balão de fala do grito. `tailX` = x (em pixels da faixa) da boca do lagartinho;
 * `tick` muda a tremedeira (balão inteiro e letras).
 */
export function renderBalloon(text: string, tailX: number, tick: number): Pix {
  const p = newPix();
  const lines = wrapText(text, 62).slice(0, 2);
  const tw = Math.max(...lines.map(textWidth), 1);
  const w = tw + 8;
  const h = lines.length * LINE_H - 1 - 2 + 6;
  const s = shakeOffset(tick, 3);
  const x = Math.round((TOP_W - w) / 2) + s.x;
  const y = 1 + Math.max(0, s.y);
  // rabinho apontando para a boca
  const tx = Math.min(Math.max(tailX, x + 6), x + w - 7);
  const tailRows = Math.min(8, TOP_H - (y + h) + 1);
  const dir = tailX > TOP_W / 2 ? 1 : -1;
  for (let j = 0; j < tailRows; j++) {
    const half = Math.max(0, 3 - Math.floor(j / 2));
    const cx = tx + Math.round(dir * j * 0.6);
    const ty = y + h - 1 + j;
    if (half === 0 || j === tailRows - 1) { plot(p, cx, ty, BALLOON.line); continue; }
    fill(p, cx - half, ty, half * 2 + 1, 1, BALLOON.fill);
    plot(p, cx - half - 1, ty, BALLOON.line);
    plot(p, cx + half + 1, ty, BALLOON.line);
  }
  roundBox(p, x, y, w, h, BALLOON.fill, BALLOON.line);
  // reabre a junção do rabinho com o balão
  const t0 = tx - 2;
  fill(p, t0, y + h - 1, 5, 1, BALLOON.fill);
  // linhas de "tremor" nos cantos
  for (const [ox, oy] of [[x - 3, y + 2], [x - 3, y + 5], [x + w + 1, y + 2], [x + w + 1, y + 5]] as const) {
    if (hash01(tick, ox + oy) < 0.7) fill(p, ox, oy, 2, 1, BALLOON.line);
  }
  const textTop = y + 3 - 2;
  lines.forEach((line, i) => {
    const lx = x + Math.floor((w - textWidth(line)) / 2);
    drawText(line, lx, textTop + i * LINE_H, (px, py) => plot(p, px, py, BALLOON.ink), (k) => (hash01(tick * 31 + i, k) < 0.3 ? -1 : 0));
  });
  return p;
}

/**
 * Onde está a boca do personagem na faixa de cima (x em pixels de 96), a partir do x da boca no quadro
 * 32x32 olhando para a direita. Canvas do pato: left 32px na janela, ampliado x4.
 */
export function mouthToTopX(mouthX: number, facing: 'left' | 'right'): number {
  const artX = facing === 'left' ? 31 - mouthX : mouthX;
  return Math.round((32 + artX * 4 + 2) / TOP_SCALE);
}
