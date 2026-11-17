// O lagartinho do humor 'lagarto': em pé, cabeção redondo, com o bracinho estendido para segurar o
// que estiver no espaço "mão" (faquinha, arminha, varinha: veja accessories.ts). Mesmo esquema do
// pato: elipses + contorno automático.
// Olhando para a direita; o olho fica no mesmo lugar do olho do pato para os acessórios servirem.
import { blank, ellipse, outline, rect, recolor, set, toGrid } from './draw.ts';
import type { AnimName, Frame, Palette } from './types.ts';

/** Paleta fixa (ignora a cor escolhida). Chaves extras: Y barriga, M boca, P língua. */
export const LIZARD_PALETTE: Palette = {
  O: '#24361a', S: '#24361a', B: '#7bd64f', b: '#52ad35', F: '#52ad35', K: '#52ad35', k: '#3f8a28',
  Y: '#d8f59a', M: '#6e1620', P: '#ff7d93',
  E: '#1d1612', W: '#ffffff',
};

interface LPose {
  bob?: number;
  headDx?: number;
  headDy?: number;
  /** pernas levantadas (0..2) */
  legBack?: number;
  legFront?: number;
  /** boca: fechada (sorrisinho), entreaberta ou gritando */
  mouth?: 'smile' | 'open' | 'scream';
  /** quanto o braço (e o que ele segura) sobe (-2..2) */
  arm?: number;
  /** ponta da cauda balançando (0..2) */
  tail?: number;
  tuck?: boolean;
}

function lizard(p: LPose = {}): Frame {
  const g = blank();
  const bob = p.bob ?? 0;
  const hx = p.headDx ?? 0;
  const hy = p.headDy ?? 0;
  const arm = p.arm ?? 0;
  const tail = p.tail ?? 0;
  const lb = p.legBack ?? 0;
  const lf = p.legFront ?? 0;
  // cauda: sai de trás e enrola para cima na ponta
  ellipse(g, 6.5, 26.5 + bob, 5.5, 2.2, 'B');
  ellipse(g, 2.2, 23.5 + bob - tail, 1.8, 2.8, 'B');
  // pés e coxas
  rect(g, 8, 29 - lb, 5, 2, 'F');
  rect(g, 15 + (lf ? 1 : 0), 29 - lf, 5, 2, 'F');
  ellipse(g, 11, 26.5 + bob - lb / 2, 2.6, 2.6, 'B');
  ellipse(g, 17 + (lf ? 0.5 : 0), 26.5 + bob - lf / 2, 2.6, 2.6, 'B');
  // corpo em pé
  ellipse(g, 14, 21.5 + bob, 6.2, 6.8, 'B');
  // pescoço
  ellipse(g, 17.5 + hx / 2, 15.5 + bob, 4, 3.5, 'B');
  // cabeça grande + focinho arredondado + morrinho do olho
  // como no pato, a cabeça não acompanha o "bob" do corpo (os acessórios ficam sempre no quadro)
  const HY = hy;
  ellipse(g, 19.5 + hx, 11.5 + HY, 6.3, 5, 'B');
  ellipse(g, 25 + hx, 13 + HY, 4.5, 3, 'B');
  ellipse(g, 21.5 + hx, 8.5 + HY, 3, 3, 'B');
  // barriga clarinha e costas com pintinhas
  ellipse(g, 16.5, 22.5 + bob, 3.2, 5, 'Y');
  recolor(g, 8, 15 + bob, 4, 14, 'B', 'b');
  for (const [x, y] of [[12, 17], [10, 20], [13, 24], [16, 9], [14, 12]] as const) {
    set(g, x + (y < 14 ? hx : 0), y + (y < 14 ? HY : bob), 'b');
  }
  // boca
  const mx = 22 + hx;
  const my = Math.round(14 + HY);
  if (p.mouth === 'scream') {
    // boca BEM aberta: o queixo desce e aparece a língua
    ellipse(g, 25.5 + hx, 17 + HY, 4.5, 2.2, 'B');
    rect(g, mx, my - 1, 7, 4, 'M');
    rect(g, mx + 1, my + 3, 5, 1, 'M');
    rect(g, mx + 1, my + 2, 4, 2, 'P');
  } else if (p.mouth === 'open') {
    rect(g, mx + 1, my, 6, 2, 'M');
    rect(g, mx + 2, my + 1, 3, 1, 'P');
  } else {
    rect(g, mx + 1, my, 6, 1, 'k');
    set(g, mx + 7, my - 1, 'k');
  }
  // narina
  set(g, 28 + hx, Math.round(12 + HY), 'k');
  // braço da frente estendido; a mão fica em `grip` (a faca/arma é um acessório desenhado ali)
  const gy = 21 + bob - arm;
  ellipse(g, 19.5, 20.5 + bob - arm / 2, 2.4, 1.5, 'B');
  ellipse(g, 21.8, gy + 0.3, 1.6, 1.6, 'B');
  outline(g);
  const eye = { x: 21 + hx, y: 9 + HY };
  if (p.tuck) eye.y += 1;
  return { grid: toGrid(g), eye, hand: { x: 4, y: 9 + bob }, grip: { x: 22, y: gy } };
}

function anims(scream: boolean): Readonly<Record<AnimName, readonly Frame[]>> {
  const m = (base: LPose['mouth']): LPose['mouth'] => (scream ? 'scream' : base);
  return {
    idle: [lizard({ mouth: m('smile') }), lizard({ mouth: m('smile'), tail: 1 })],
    walk: [
      lizard({ mouth: m('smile'), legBack: 1, tail: 1 }),
      lizard({ mouth: m('smile'), bob: -1, legFront: 1 }),
    ],
    run: [
      lizard({ mouth: m('open'), headDx: 1, bob: -1, legBack: 2, tail: 2, arm: 1 }),
      lizard({ mouth: m('open'), headDx: 1, legFront: 2, arm: 0 }),
    ],
    grab: [
      lizard({ mouth: m('open'), headDx: 1, legBack: 1, arm: 1 }),
      lizard({ mouth: m('open'), headDx: 1, bob: -1, legFront: 1, arm: 2 }),
    ],
    sleep: [
      lizard({ mouth: 'smile', headDx: -1, headDy: 2, arm: -2, tuck: true }),
      lizard({ mouth: 'smile', headDx: -1, headDy: 3, arm: -2, tuck: true }),
    ],
    dance: [
      lizard({ mouth: m('smile'), headDx: -1, arm: 2, legBack: 1, tail: 0 }),
      lizard({ mouth: m('open'), headDy: 1, bob: -1, arm: 0, legBack: 2, legFront: 2, tail: 2 }),
      lizard({ mouth: m('smile'), headDx: 1, arm: 2, legFront: 1, tail: 1 }),
      lizard({ mouth: m('open'), headDy: 1, bob: -1, arm: 0, legBack: 2, legFront: 2, tail: 2 }),
    ],
  };
}

export const LIZARD_ANIMS = anims(false);
/** Mesmas poses com a boca escancarada, para o "ataque de grito". */
export const LIZARD_SCREAM_ANIMS = anims(true);
