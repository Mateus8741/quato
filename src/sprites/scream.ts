// "Ataque de grito" do lagartinho: de tempos em tempos (intervalo sorteado entre 12 e 35 s) ele grita
// por ~3,5 s. Função pura: dado o início do humor, a semente e o instante, diz se está gritando e
// quando o estado muda (para agendar um único setTimeout, sem ficar consultando).

export const SCREAM_MIN_GAP_MS = 12_000;
export const SCREAM_MAX_GAP_MS = 35_000;
export const SCREAM_DURATION_MS = 3_500;
export const SCREAM_TEXT = 'AAAAAAAAAAAAAAAAAAAAAAAA';

/** Número pseudoaleatório em [0, 1) para (semente, índice). */
export function hash01(seed: number, i: number): number {
  let h = (Math.imul(seed | 0, 0x9e3779b1) ^ Math.imul(i + 1, 0x85ebca77)) >>> 0;
  h = Math.imul(h ^ (h >>> 16), 0x7feb352d) >>> 0;
  h = Math.imul(h ^ (h >>> 15), 0x846ca68b) >>> 0;
  h = (h ^ (h >>> 16)) >>> 0;
  return h / 4294967296;
}

/** Espera antes do grito número i. */
export function screamGap(seed: number, i: number): number {
  return SCREAM_MIN_GAP_MS + Math.floor(hash01(seed, i) * (SCREAM_MAX_GAP_MS - SCREAM_MIN_GAP_MS + 1));
}

export interface ScreamState {
  screaming: boolean;
  /** Instante (mesma base de `now`) em que o estado muda. */
  until: number;
}

/** `start`: quando o humor 'lagarto' começou. */
export function screamState(now: number, start: number, seed: number): ScreamState {
  let t = start;
  for (let i = 0; ; i++) {
    t += screamGap(seed, i);
    if (now < t) return { screaming: false, until: t };
    if (now < t + SCREAM_DURATION_MS) return { screaming: true, until: t + SCREAM_DURATION_MS };
    t += SCREAM_DURATION_MS;
  }
}

/** Deslocamento da tremedeira (em pixels de arte, -1..1) para o tique dado. */
export function shakeOffset(tick: number, seed = 0): { x: number; y: number } {
  const a = hash01(seed + 7, tick);
  const b = hash01(seed + 13, tick);
  return { x: Math.floor(a * 3) - 1, y: Math.floor(b * 3) - 1 };
}
