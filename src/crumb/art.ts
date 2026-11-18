// Migalha: arte e animação de "comer" (funções puras, sem DOM). Tempos em milissegundos,
// posições em pixels de arte (a página amplia por um número inteiro).

/** Migalha de pão 8x6: 'o' contorno, 'b' miolo, 'l' brilho. */
export const CRUMB_GRID: readonly string[] = [
  '..ooo...',
  '.oblloo.',
  'oblbbbbo',
  'obbbbbbo',
  '.obbbbo.',
  '..oooo..',
];
export const CRUMB_PALETTE: Readonly<Record<string, string>> = { o: '#7a4a1e', b: '#e8b86a', l: '#f9e2a8' };
export const CRUMB_W = 8;
export const CRUMB_H = 6;

/** Duração da animação de comer (a janela fecha logo depois, em crumb.rs). */
export const EAT_MS = 800;
/** A migalha some em 3 mordidas: escala restante em cada fatia do tempo. */
const BITES = [1, 0.7, 0.4, 0.15] as const;
const BITE_END = 0.6;

/** Quanto da migalha ainda existe (0..1) `t` ms depois de começar a comer. */
export function crumbScale(t: number): number {
  if (t <= 0) return 1;
  const k = Math.min(1, t / (EAT_MS * BITE_END));
  if (k >= 1) return 0;
  return BITES[Math.min(BITES.length - 1, Math.floor(k * BITES.length))]!;
}

export interface Particle {
  x: number;
  y: number;
  vx: number;
  vy: number;
  color: string;
}

/** Farelinhos que pulam para cima e para os lados. `rnd` devolve [0, 1). */
export function burst(rnd: () => number, n = 9): Particle[] {
  const colors = [CRUMB_PALETTE.b!, CRUMB_PALETTE.l!, CRUMB_PALETTE.o!];
  return Array.from({ length: n }, (_, i) => ({
    x: CRUMB_W / 2,
    y: CRUMB_H / 2,
    vx: (rnd() * 2 - 1) * 14,
    vy: -(6 + rnd() * 10),
    color: colors[i % colors.length]!,
  }));
}

export const GRAVITY = 40;

/** Posição e transparência do farelo `t` ms depois do início (queda com gravidade, some no fim). */
export function particleAt(p: Particle, t: number): { x: number; y: number; alpha: number } {
  const s = Math.max(0, t) / 1000;
  const alpha = Math.max(0, 1 - Math.max(0, t) / EAT_MS);
  return { x: p.x + p.vx * s, y: p.y + p.vy * s + 0.5 * GRAVITY * s * s, alpha };
}
