// O pato é desenhado por geometria simples (elipses) + contorno automático.
// Assim o corpo fica sempre fechado e dá para ajustar o desenho mexendo em poucos números.
import { blank, ellipse, outline, rect, recolor, toGrid } from './draw.ts';
import type { AnimName, Frame, PixelGrid } from './types.ts';

interface Pose {
  bob?: number;
  headDx?: number;
  headDy?: number;
  beakOpen?: boolean;
  /** pé de trás e da frente levantados (0, 1 ou 2 pixels) */
  footBack?: number;
  footFront?: number;
  /** cabeça pendendo, para dormir */
  tuck?: boolean;
  /** asa: descansando (padrão), levantada para cima ou aberta para trás (dança) */
  wing?: 'rest' | 'up' | 'back';
}

function duck(p: Pose = {}): Frame {
  const g = blank();
  const bob = p.bob ?? 0;
  const hx = p.headDx ?? 0;
  const hy = (p.headDy ?? 0) + bob;
  const wing = p.wing ?? 'rest';
  // pés (ficam no chão, embaixo do corpo)
  const fb = p.footBack ?? 0;
  const ff = p.footFront ?? 0;
  rect(g, 9, 29 - fb, 5, 2, 'F');
  rect(g, 17 + (ff ? 1 : 0), 29 - ff, 5, 2, 'F');
  // corpo e cauda
  ellipse(g, 14, 21 + bob, 11, 7, 'B');
  ellipse(g, 4.5, 18 + bob, 3.5, 2.5, 'B');
  // pescoço e cabeça
  ellipse(g, 19 + hx, 16 + bob, 5, 5, 'B');
  ellipse(g, 20 + hx, 11.5 + hy - bob, 6.2, 6, 'B');
  // asa (sombra) e barriga
  if (wing === 'rest') ellipse(g, 11.5, 21 + bob, 5.5, 3.2, 'b');
  recolor(g, 6, 25 + bob, 16, 4, 'B', 'b');
  if (wing === 'up') {
    // asa erguida acima das costas, com a ponta em leque
    ellipse(g, 10, 14.5 + bob, 4, 6, 'O');
    ellipse(g, 10, 14.5 + bob, 3, 5, 'b');
    ellipse(g, 10.5, 15.5 + bob, 1.5, 3, 'B');
  } else if (wing === 'back') {
    // asa aberta para trás, por cima da cauda
    ellipse(g, 7.5, 16.5 + bob, 7, 3.6, 'O');
    ellipse(g, 7.5, 16.5 + bob, 6, 2.6, 'b');
    rect(g, 4, 16 + bob, 7, 1, 'B');
  }
  // bico
  const by = Math.round(12 + hy - bob);
  const bx = 25 + hx;
  if (p.beakOpen) {
    rect(g, bx, by - 1, 5, 2, 'K');
    rect(g, bx, by + 2, 4, 2, 'k');
  } else {
    rect(g, bx, by, 5, 2, 'K');
    rect(g, bx, by + 2, 4, 1, 'k');
  }
  outline(g);
  const eye = { x: 22 + hx, y: Math.round(9 + hy - bob) };
  if (p.tuck) eye.y += 1;
  // grip: a "asa" segura o que estiver na mão na frente da barriga
  return { grid: toGrid(g), eye, hand: { x: 5, y: 5 + bob }, grip: { x: 22, y: 21 + bob } };
}

export const ANIMS: Readonly<Record<AnimName, readonly Frame[]>> = {
  idle: [duck()],
  walk: [duck({ footBack: 1, footFront: 0 }), duck({ bob: -1, footBack: 0, footFront: 1 })],
  run: [
    duck({ headDx: 1, bob: -1, footBack: 2, footFront: 0 }),
    duck({ headDx: 1, bob: 0, footBack: 0, footFront: 2 }),
  ],
  grab: [
    duck({ headDx: 1, beakOpen: true, footBack: 1 }),
    duck({ headDx: 1, bob: -1, beakOpen: true, footFront: 1 }),
  ],
  sleep: [duck({ headDx: -4, headDy: 4, tuck: true }), duck({ headDx: -4, headDy: 5, bob: 1, tuck: true })],
  // dança: balança a cabeça de um lado para o outro, pula num pé e no outro e bate as asas
  // comendo a migalha: a cabeça desce até o chão e bica; entre uma bicada e outra sobe um pouco
  eat: [
    duck({ headDx: 2, headDy: 3, beakOpen: true }),
    duck({ headDx: 2, headDy: 6 }),
    duck({ headDx: 2, headDy: 4, beakOpen: true }),
  ],
  dance: [
    duck({ headDx: -1, wing: 'up', footBack: 1 }),
    duck({ headDx: 0, headDy: 1, bob: -1, wing: 'back', beakOpen: true, footBack: 2, footFront: 2 }),
    duck({ headDx: 1, wing: 'up', footFront: 1 }),
    duck({ headDx: 0, headDy: 1, bob: -1, wing: 'back', beakOpen: true, footBack: 2, footFront: 2 }),
  ],
};

export function framesFor(anim: AnimName): readonly Frame[] {
  return ANIMS[anim];
}

export function gridsOf(frames: readonly Frame[]): PixelGrid[] {
  return frames.map((f) => f.grid);
}
