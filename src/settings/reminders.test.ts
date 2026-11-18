import { describe, expect, it } from 'vitest';
import type { RemindersStatus } from '../shared/types.ts';
import { focusStatusText, parseMinutes, textError } from './reminders.ts';

const status = (over: Partial<RemindersStatus>): RemindersStatus => ({
  postureNextInSec: null,
  stretchNextInSec: null,
  eyesNextInSec: null,
  focusPhase: 'off',
  focusRemainingSec: null,
  ...over,
});

describe('parseMinutes', () => {
  it('aceita inteiros e ajusta ao limite', () => {
    expect(parseMinutes('25', 180)).toBe(25);
    expect(parseMinutes('0', 180)).toBe(1);
    expect(parseMinutes('999', 60)).toBe(60);
    expect(parseMinutes('4,6', 60)).toBe(5);
  });
  it('recusa o que não é número', () => {
    for (const t of ['', 'abc', '-5', '1e3']) expect(parseMinutes(t, 60)).toBeNull();
  });
});

describe('textError', () => {
  it('aceita texto vazio (volta ao padrão) e textos curtos', () => {
    expect(textError('')).toBeNull();
    expect(textError('Olhe a 6 m por 20 s')).toBeNull();
  });
  it('recusa texto longo ou com letras que a fonte não tem', () => {
    expect(textError('a'.repeat(29))).toMatch(/28/);
    expect(textError('oi {}')).toMatch(/não sabe desenhar/);
  });
});

describe('focusStatusText', () => {
  it('vazio com o foco desligado', () => {
    expect(focusStatusText(status({}))).toBe('');
  });
  it('mostra fase e tempo restante', () => {
    expect(focusStatusText(status({ focusPhase: 'focus', focusRemainingSec: 1500 }))).toBe('Foco: faltam ~25 min.');
    expect(focusStatusText(status({ focusPhase: 'break', focusRemainingSec: 300 }), 250)).toBe('Pausa: falta menos de 1 min.');
  });
});
