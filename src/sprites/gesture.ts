// "Pena do meio": o pato levanta a asa com 3 penas, a do meio mais alta (e o lagartinho, a mãozinha
// com 3 dedinhos, o do meio maior). Desenhado com as chaves da paleta do corpo (O, B, b), por isso
// acompanha a cor escolhida. Fica na camada da frente, no ponto `frame.hand`.
import type { Character, PixelGrid } from './types.ts';

export const FEATHER_DUCK: PixelGrid = [
  '....OO....',
  '...ObBO...',
  '...ObBO...',
  '...ObBO...',
  '...ObBO...',
  '.OOObBOOO.',
  'ObBObBObBO',
  'ObBObBObBO',
  'ObBObBObBO',
  'ObbbBBbbbO',
  'ObBBBBBBBO',
  '.ObBBBBBO.',
  '..ObBBBO..',
  '...OOOO...',
];

export const FEATHER_LIZARD: PixelGrid = [
  '...OO...',
  '..ObBO..',
  '..ObBO..',
  'OO.ObBO.OO',
  'ObOObBOObO',
  'ObBObBObBO',
  'ObBbBBbBBO',
  '.ObBBBBBO.',
  '..ObBBBO..',
  '..ObBBO...',
  '..ObBBO...',
  '..ObBBO...',
].map((r) => r.padEnd(10, '.'));

export function gestureGrid(c: Character): PixelGrid {
  return c === 'lizard' ? FEATHER_LIZARD : FEATHER_DUCK;
}

/** Olhos semicerrados ("sério/zoando") durante o gesto: pálpebra reta em cima, pupila embaixo. */
export const GESTURE_EYES: PixelGrid = ['SSS', 'EE.'];
