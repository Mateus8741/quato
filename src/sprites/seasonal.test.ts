import { readFileSync } from 'node:fs';
import { describe, expect, it } from 'vitest';
import { ACCESSORIES } from './index.ts';
import { ACCESSORY_SLOTS, seasonalAccessory, withSeasonal } from './seasonal.ts';

const day = (m: number, d: number) => new Date(2026, m - 1, d, 12);

describe('seasonalAccessory', () => {
  it('Natal: 15 a 26 de dezembro', () => {
    expect(seasonalAccessory(day(12, 14))).toBeNull();
    expect(seasonalAccessory(day(12, 15))).toBe('gorro-natal');
    expect(seasonalAccessory(day(12, 25))).toBe('gorro-natal');
    expect(seasonalAccessory(day(12, 26))).toBe('gorro-natal');
    expect(seasonalAccessory(day(12, 27))).toBeNull();
  });
  it('Halloween: 25 a 31 de outubro', () => {
    expect(seasonalAccessory(day(10, 24))).toBeNull();
    expect(seasonalAccessory(day(10, 25))).toBe('chapeu-bruxa');
    expect(seasonalAccessory(day(10, 31))).toBe('chapeu-bruxa');
    expect(seasonalAccessory(day(11, 1))).toBeNull();
  });
  it('Ano Novo: 31/dez a 2/jan', () => {
    expect(seasonalAccessory(day(12, 30))).toBeNull();
    expect(seasonalAccessory(day(12, 31))).toBe('chapeu-festa');
    expect(seasonalAccessory(day(1, 1))).toBe('chapeu-festa');
    expect(seasonalAccessory(day(1, 2))).toBe('chapeu-festa');
    expect(seasonalAccessory(day(1, 3))).toBeNull();
  });
  it('dia comum não tem acessório', () => {
    expect(seasonalAccessory(day(6, 15))).toBeNull();
  });
});

describe('withSeasonal', () => {
  it('desligado não muda nada', () => {
    expect(withSeasonal([], false, day(12, 25))).toEqual([]);
  });
  it('ligado e na data: acrescenta o da data', () => {
    expect(withSeasonal([], true, day(12, 25))).toEqual(['gorro-natal']);
    expect(withSeasonal(['oculos'], true, day(10, 28))).toEqual(['oculos', 'chapeu-bruxa']);
  });
  it('fora da data não acrescenta', () => {
    expect(withSeasonal(['oculos'], true, day(6, 1))).toEqual(['oculos']);
  });
  it('a escolha da pessoa no mesmo espaço vence', () => {
    expect(withSeasonal(['coroa'], true, day(12, 25))).toEqual(['coroa']);
    expect(withSeasonal(['coroa', 'oculos'], true, day(10, 30))).toEqual(['coroa', 'oculos']);
  });
  it('não duplica nem altera a lista original', () => {
    const base = ['gorro-natal'];
    expect(withSeasonal(base, true, day(12, 25))).toEqual(['gorro-natal']);
    withSeasonal(base, true, day(12, 25));
    expect(base).toEqual(['gorro-natal']);
  });
});

describe('espelho dos espaços com o Rust', () => {
  const rust = readFileSync(new URL('../../src-tauri/src/look.rs', import.meta.url), 'utf8');
  const block = /pub const ACCESSORIES[^=]*= \[([\s\S]*?)\];/.exec(rust)![1]!;
  const pairs = [...block.matchAll(/\("([^"]+)",\s*"([^"]+)"\)/g)].map((x) => [x[1]!, x[2]!] as const);

  it('ACCESSORY_SLOTS bate com look.rs', () => {
    expect(Object.fromEntries(pairs)).toEqual({ ...ACCESSORY_SLOTS });
  });
  it('todo acessório desenhado tem espaço', () => {
    for (const a of ACCESSORIES) expect(ACCESSORY_SLOTS[a.id], a.id).toBeDefined();
  });
});
