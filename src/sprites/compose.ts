import type { Mood } from '../shared/types.ts';
import { BASE_PALETTE, colorById } from './colors.ts';
import { accessoryById } from './accessories.ts';
import { ANIMS } from './base.ts';
import { GESTURE_EYES, gestureGrid } from './gesture.ts';
import { LIZARD_ANIMS, LIZARD_PALETTE, LIZARD_SCREAM_ANIMS } from './lizard.ts';
import { SIZE, type AnimName, type Character, type Frame, type Look, type MoodSkin, type Palette, type PixelGrid } from './types.ts';

export const DEFAULT_LOOK: Look = { color: 'amarelo', accessories: [] };

/** O humor 'lagarto' troca o pato pelo lagartinho. */
export function characterFor(mood: Mood): Character {
  return mood === 'lagarto' ? 'lizard' : 'duck';
}

/** Quadros da animação para o humor (pato ou lagartinho; `screaming` = boca escancarada). */
export function framesFor(mood: Mood, anim: AnimName, screaming = false): readonly Frame[] {
  if (characterFor(mood) === 'lizard') return (screaming ? LIZARD_SCREAM_ANIMS : LIZARD_ANIMS)[anim];
  return ANIMS[anim];
}

function hexToRgba(hex: string): [number, number, number, number] {
  const h = hex.replace('#', '');
  const n = (i: number) => parseInt(h.slice(i, i + 2), 16);
  return [n(0), n(2), n(4), h.length >= 8 ? n(6) : 255];
}

function stamp(buf: Uint8ClampedArray, grid: PixelGrid, ox: number, oy: number, palette: Palette): void {
  grid.forEach((row, j) => {
    for (let i = 0; i < row.length; i++) {
      const ch = row[i]!;
      if (ch === '.') continue;
      const color = palette[ch];
      const x = ox + i;
      const y = oy + j;
      if (!color || x < 0 || y < 0 || x >= SIZE || y >= SIZE) continue;
      const [r, g, b, a] = hexToRgba(color);
      const k = (y * SIZE + x) * 4;
      buf[k] = r; buf[k + 1] = g; buf[k + 2] = b; buf[k + 3] = a;
    }
  });
}

/** Paleta efetiva: base + cor escolhida + bochecha do humor. O lagartinho tem paleta fixa. */
export function paletteFor(skin: MoodSkin, look: Look): Palette {
  if (characterFor(skin.mood) === 'lizard') return { ...BASE_PALETTE, ...LIZARD_PALETTE, C: skin.cheekColor };
  return { ...BASE_PALETTE, ...colorById(look.color).palette, C: skin.cheekColor };
}

export interface ComposeExtra {
  /** "Pena do meio" levantada (e olhos semicerrados). */
  gesture?: boolean;
}

/** Junta corpo + acessórios de trás + expressão + acessórios da frente, em RGBA 32x32 olhando para a direita. */
export function composeRGBA(
  skin: MoodSkin,
  frame: Frame,
  eyesClosed: boolean,
  look: Look = DEFAULT_LOOK,
  extra: ComposeExtra = {},
): Uint8ClampedArray {
  const buf = new Uint8ClampedArray(SIZE * SIZE * 4);
  const pal = paletteFor(skin, look);
  const worn = look.accessories.map(accessoryById).filter((a) => a !== undefined);
  stamp(buf, frame.grid, 0, 0, pal);
  for (const a of worn) if (a.layer === 'back') stamp(buf, a.grid, frame.eye.x + a.dx, frame.eye.y + a.dy, a.palette);
  if (skin.cheek) stamp(buf, ['CC', 'CC'], frame.eye.x - 3, frame.eye.y + 4, pal);
  if (extra.gesture) {
    stamp(buf, GESTURE_EYES, frame.eye.x, frame.eye.y, pal);
  } else {
    stamp(buf, eyesClosed ? skin.eyesClosed : skin.eyes, frame.eye.x, frame.eye.y, pal);
    if (skin.brow && !eyesClosed) stamp(buf, skin.brow.grid, frame.eye.x + skin.brow.dx, frame.eye.y + skin.brow.dy, pal);
  }
  for (const a of worn) if (a.layer === 'front') stamp(buf, a.grid, frame.eye.x + a.dx, frame.eye.y + a.dy, a.palette);
  for (const a of worn) if (a.layer === 'hand') stamp(buf, a.grid, frame.grip.x + a.dx, frame.grip.y + a.dy, a.palette);
  if (extra.gesture) stamp(buf, gestureGrid(characterFor(skin.mood)), frame.hand.x, frame.hand.y, pal);
  return buf;
}
