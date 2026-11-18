import type { SpeechFrequency } from '../shared/types.ts';
import { SPEECH_MAX_CHARS, SPEECH_MAX_EXTRA } from '../shared/types.ts';
import { unknownChars } from '../sprites/font.ts';

/** Textos das opções de frequência da seção "Fala". */
export const SPEECH_LABELS: Readonly<Record<SpeechFrequency, string>> = {
  never: 'Nunca',
  rare: 'Raro',
  sometimes: 'Às vezes',
  always: 'Sempre',
};

export const SPEECH_HINTS: Readonly<Record<SpeechFrequency, string>> = {
  never: 'O pato fica calado.',
  rare: 'Uma frase a cada 6 a 12 minutos.',
  sometimes: 'Uma frase a cada 1,5 a 3 minutos.',
  always: 'Uma frase a cada 20 a 40 segundos.',
};

export type PhraseResult = { ok: true; text: string } | { ok: false; error: string };

/** Arruma a frase digitada (espaços) e confere se o balão consegue desenhar. */
export function normalizePhrase(raw: string): PhraseResult {
  const text = raw.replace(/\s+/g, ' ').trim();
  if (!text) return { ok: false, error: 'Digite uma frase.' };
  if ([...text].length > SPEECH_MAX_CHARS) return { ok: false, error: `Use no máximo ${SPEECH_MAX_CHARS} letras.` };
  const bad = unknownChars(text);
  if (bad.length) return { ok: false, error: `O balão não desenha: ${bad.join(' ')}. Use letras, números e ! ? . , - : '` };
  return { ok: true, text };
}

export type AddResult = { ok: true; list: string[] } | { ok: false; error: string };

/** Acrescenta uma frase à lista (sem repetir, sem passar do limite). */
export function addPhrase(list: readonly string[], raw: string): AddResult {
  const p = normalizePhrase(raw);
  if (!p.ok) return p;
  if (list.some((x) => x.toLocaleLowerCase('pt-BR') === p.text.toLocaleLowerCase('pt-BR'))) return { ok: false, error: 'Essa frase já está na lista.' };
  if (list.length >= SPEECH_MAX_EXTRA) return { ok: false, error: `No máximo ${SPEECH_MAX_EXTRA} frases.` };
  return { ok: true, list: [...list, p.text] };
}

export function removePhrase(list: readonly string[], index: number): string[] {
  return list.filter((_, i) => i !== index);
}
