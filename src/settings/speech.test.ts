import { describe, expect, it } from 'vitest';
import { renderSign, renderSpeech, TOP_H, TOP_W } from '../sprites/overlay.ts';
import { SPEECH_FREQUENCIES, SPEECH_MAX_CHARS, SPEECH_MAX_EXTRA } from '../shared/types.ts';
import { addPhrase, normalizePhrase, removePhrase, SPEECH_HINTS, SPEECH_LABELS } from './speech.ts';

describe('frases extras', () => {
  it('toda frequência tem rótulo e dica', () => {
    for (const f of SPEECH_FREQUENCIES) {
      expect(SPEECH_LABELS[f]).toBeTruthy();
      expect(SPEECH_HINTS[f]).toBeTruthy();
    }
  });

  it('arruma espaços e recusa vazio', () => {
    expect(normalizePhrase('  oi   pato  ')).toEqual({ ok: true, text: 'oi pato' });
    expect(normalizePhrase('   ').ok).toBe(false);
  });

  it('recusa frase longa demais', () => {
    expect(normalizePhrase('a'.repeat(SPEECH_MAX_CHARS)).ok).toBe(true);
    expect(normalizePhrase('a'.repeat(SPEECH_MAX_CHARS + 1)).ok).toBe(false);
  });

  it('recusa o que a fonte do balão não desenha, mas aceita acentos e pontuação', () => {
    expect(normalizePhrase('Vamos dançar, né? Quá!').ok).toBe(true);
    const r = normalizePhrase('oi 🦆');
    expect(r.ok).toBe(false);
  });

  it('adiciona sem repetir (ignorando maiúsculas) e respeita o limite', () => {
    const a = addPhrase([], 'Oi');
    expect(a).toEqual({ ok: true, list: ['Oi'] });
    expect(addPhrase(['Oi'], 'oi').ok).toBe(false);
    const full = Array.from({ length: SPEECH_MAX_EXTRA }, (_, i) => `frase ${i}`);
    expect(addPhrase(full, 'mais uma').ok).toBe(false);
  });

  it('remove pelo índice sem mexer na lista original', () => {
    const l = ['a', 'b', 'c'];
    expect(removePhrase(l, 1)).toEqual(['a', 'c']);
    expect(l).toEqual(['a', 'b', 'c']);
  });
});

describe('balão de fala', () => {
  it('desenha pixels, cabe na faixa e difere da plaquinha', () => {
    const s = renderSpeech('Será que deixei o fogão ligado?');
    expect([s.w, s.h]).toEqual([TOP_W, TOP_H]);
    let ink = 0;
    for (let i = 3; i < s.data.length; i += 4) if (s.data[i]! > 0) ink++;
    expect(ink).toBeGreaterThan(100);
    expect(renderSign('Oi', 'speech').data).toEqual(renderSpeech('Oi').data);
    expect(renderSign('Oi', 'water').data).not.toEqual(renderSpeech('Oi').data);
  });
});
