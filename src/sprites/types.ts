import type { Mood } from '../shared/types.ts';

/** 32 strings de 32 caracteres. '.' = transparente. */
export type PixelGrid = readonly string[];
/** Um caractere -> cor "#rrggbb" ou "#rrggbbaa". */
export type Palette = Readonly<Record<string, string>>;
export type AnimName = 'idle' | 'walk' | 'run' | 'grab' | 'sleep' | 'dance' | 'eat';
export const ANIM_NAMES: readonly AnimName[] = ['idle', 'walk', 'run', 'grab', 'sleep', 'dance', 'eat'];
export type EffectName = 'sparkle' | 'steam' | 'tear' | 'dots' | 'zzz' | 'notes' | 'drip' | 'question';
/** Quem aparece na janela: o pato, ou o lagartinho no humor 'lagarto'. */
export type Character = 'duck' | 'lizard';

export const SIZE = 32;

export interface Point {
  x: number;
  y: number;
}

export interface Frame {
  grid: PixelGrid;
  /** Canto superior esquerdo do olho, para desenhar a expressão do humor por cima. */
  eye: Point;
  /** Canto superior esquerdo da asa/mão levantada no gesto da "pena do meio". */
  hand: Point;
  /** Onde a mão segura o que está no espaço "mão" (faca, arminha, varinha): o canto da empunhadura. */
  grip: Point;
}

export type { Look } from '../shared/types.ts';

export interface ColorDef {
  id: string;
  label: string;
  /** Sobrescreve a paleta base (O contorno, B corpo, b sombra, K/k bico, F pés, E olho, W brilho). */
  palette: Palette;
}

export type AccessoryLayer = 'back' | 'front' | 'hand';

export interface Accessory {
  id: string;
  label: string;
  /** 'back' fica atrás dos olhos (perucas, chapéus); 'front' por cima (óculos, fones);
   *  'hand' é desenhado por último, por cima de tudo (menos o gesto da pena), em relação à mão. */
  layer: AccessoryLayer;
  /** Posição do canto superior esquerdo em relação ao canto do olho (que acompanha a cabeça)
   *  ou, na layer 'hand', em relação a `frame.grip`. */
  dx: number;
  dy: number;
  grid: PixelGrid;
  palette: Palette;
}

export interface MoodSkin {
  mood: Mood;
  label: string;
  /** Cor das bochechas do humor. */
  cheekColor: string;
  /** Olhos abertos, fechados, e (opcional) sobrancelha + deslocamento dela em relação ao olho. */
  eyes: PixelGrid;
  eyesClosed: PixelGrid;
  brow?: { grid: PixelGrid; dx: number; dy: number };
  cheek: boolean;
  effect: EffectName;
  /** Quadros por segundo de cada animação. */
  fps: Readonly<Record<AnimName, number>>;
}
