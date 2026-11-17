import { describe, expect, it } from 'vitest';
import { formatMinutes, nextReminderText, parseInterval, WATER_PRESETS } from './water.ts';

describe('parseInterval', () => {
  it('aceita inteiros dentro do limite', () => {
    expect(parseInterval('45')).toBe(45);
    expect(parseInterval(' 90 ')).toBe(90);
  });
  it('ajusta para 1..720', () => {
    expect(parseInterval('0')).toBe(1);
    expect(parseInterval('9999')).toBe(720);
  });
  it('arredonda decimais (com vírgula ou ponto)', () => {
    expect(parseInterval('12,6')).toBe(13);
    expect(parseInterval('12.4')).toBe(12);
  });
  it('recusa o que não é número', () => {
    for (const t of ['', 'abc', '-5', '1e3', '10 min']) expect(parseInterval(t)).toBeNull();
  });
  it('atalhos estão dentro do limite', () => {
    for (const p of WATER_PRESETS) expect(parseInterval(String(p))).toBe(p);
  });
});

describe('textos', () => {
  it('formata minutos', () => {
    expect(formatMinutes(45)).toBe('45 min');
    expect(formatMinutes(60)).toBe('1 h');
    expect(formatMinutes(90)).toBe('1 h 30 min');
  });
  it('próximo lembrete', () => {
    expect(nextReminderText(null)).toBe('');
    expect(nextReminderText(30)).toBe('Próximo lembrete em menos de 1 min.');
    expect(nextReminderText(61)).toBe('Próximo lembrete em ~2 min.');
    expect(nextReminderText(45 * 60)).toBe('Próximo lembrete em ~45 min.');
  });
});
