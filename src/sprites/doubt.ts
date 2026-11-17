// "Dúvida aleatória": em qualquer humor (menos 'duvida', que sempre mostra o '?'), de tempos em tempos
// (intervalo sorteado entre 20 e 60 s) o '?' aparece sobre a cabeça por ~3 s. Função pura, como scream.ts.
import { hash01 } from './scream.ts';

export const DOUBT_MIN_GAP_MS = 20_000;
export const DOUBT_MAX_GAP_MS = 60_000;
export const DOUBT_DURATION_MS = 3_000;

/** Espera antes da dúvida número i. */
export function doubtGap(seed: number, i: number): number {
  return DOUBT_MIN_GAP_MS + Math.floor(hash01(seed + 101, i) * (DOUBT_MAX_GAP_MS - DOUBT_MIN_GAP_MS + 1));
}

export interface DoubtState {
  doubting: boolean;
  /** Instante (mesma base de `now`) em que o estado muda. */
  until: number;
}

/** `start`: quando o humor atual começou. */
export function doubtState(now: number, start: number, seed: number): DoubtState {
  let t = start;
  for (let i = 0; ; i++) {
    t += doubtGap(seed, i);
    if (now < t) return { doubting: false, until: t };
    if (now < t + DOUBT_DURATION_MS) return { doubting: true, until: t + DOUBT_DURATION_MS };
    t += DOUBT_DURATION_MS;
  }
}
