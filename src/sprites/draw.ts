// Primitivas de desenho em grade de caracteres (32x32), usadas pelo pato e pelo lagartinho.
import { SIZE, type PixelGrid } from './types.ts';

export type Buf = string[][];

export function blank(): Buf {
  return Array.from({ length: SIZE }, () => Array<string>(SIZE).fill('.'));
}

export function set(g: Buf, x: number, y: number, ch: string): void {
  if (x >= 0 && x < SIZE && y >= 0 && y < SIZE) g[y]![x] = ch;
}

export function get(g: Buf, x: number, y: number): string | undefined {
  return g[y]?.[x];
}

export function ellipse(g: Buf, cx: number, cy: number, rx: number, ry: number, ch: string): void {
  for (let y = Math.floor(cy - ry); y <= Math.ceil(cy + ry); y++) {
    for (let x = Math.floor(cx - rx); x <= Math.ceil(cx + rx); x++) {
      const dx = (x + 0.5 - cx) / rx;
      const dy = (y + 0.5 - cy) / ry;
      if (dx * dx + dy * dy <= 1) set(g, x, y, ch);
    }
  }
}

export function rect(g: Buf, x: number, y: number, w: number, h: number, ch: string): void {
  for (let j = 0; j < h; j++) for (let i = 0; i < w; i++) set(g, x + i, y + j, ch);
}

/** Troca `from` por `to` dentro de um retângulo (para sombrear sem sair do corpo). */
export function recolor(g: Buf, x: number, y: number, w: number, h: number, from: string, to: string): void {
  for (let j = 0; j < h; j++) for (let i = 0; i < w; i++) if (get(g, x + i, y + j) === from) set(g, x + i, y + j, to);
}

/** Contorno de 1 pixel ('O') em volta de tudo que não é transparente. */
export function outline(g: Buf): void {
  const copy = g.map((r) => r.slice());
  for (let y = 0; y < SIZE; y++) {
    for (let x = 0; x < SIZE; x++) {
      if (copy[y]![x] !== '.') continue;
      const near = [copy[y - 1]?.[x], copy[y + 1]?.[x], copy[y]?.[x - 1], copy[y]?.[x + 1]];
      if (near.some((c) => c !== undefined && c !== '.' && c !== 'O')) g[y]![x] = 'O';
    }
  }
}

export function toGrid(g: Buf): PixelGrid {
  return g.map((r) => r.join(''));
}
