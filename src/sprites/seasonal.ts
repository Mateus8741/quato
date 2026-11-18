// Acessórios sazonais: lógica pura (data -> acessório), sem tocar no relógio nem na tela.

/** Espaço de cada acessório. Espelho de `ACCESSORIES` em src-tauri/src/look.rs (há um teste que confere). */
export const ACCESSORY_SLOTS: Readonly<Record<string, string>> = {
  'peruca-ruiva': 'cabeca',
  'peruca-curta': 'cabeca',
  'chapeu-festa': 'cabeca',
  'coroa': 'cabeca',
  'cartola': 'cabeca',
  'chapeu-seletor': 'cabeca',
  'chapeu-capivara': 'cabeca',
  'gorro-natal': 'cabeca',
  'chapeu-bruxa': 'cabeca',
  'chapeu-pirata': 'cabeca',
  'oculos': 'rosto',
  'oculos-escuros': 'rosto',
  'tapa-olho': 'rosto',
  'fones': 'orelha',
  'laco': 'pescoco',
  'capa': 'pescoco',
  'mochila': 'costas',
  'flor': 'enfeite',
  'varinha': 'mao',
  'faca': 'mao',
  'faca-sangue': 'mao',
  'arminha': 'mao',
};

/** Acessório da data (usa o dia/mês locais de `date`), ou null fora das datas especiais.
 *  Halloween 25-31/out, Natal 15-26/dez, Ano Novo 31/dez-2/jan (chapéu de festa). */
export function seasonalAccessory(date: Date): string | null {
  const m = date.getMonth() + 1;
  const d = date.getDate();
  if (m === 10 && d >= 25) return 'chapeu-bruxa';
  if (m === 12 && d >= 15 && d <= 26) return 'gorro-natal';
  if ((m === 12 && d === 31) || (m === 1 && d <= 2)) return 'chapeu-festa';
  return null;
}

/** Lista de acessórios a desenhar: com `enabled` e numa data especial, acrescenta o da data
 *  se a pessoa não escolheu nada no mesmo espaço (a escolha dela sempre vence). */
export function withSeasonal(accessories: readonly string[], enabled: boolean, date: Date): string[] {
  const out = [...accessories];
  if (!enabled) return out;
  const id = seasonalAccessory(date);
  if (!id || out.includes(id)) return out;
  const slot = ACCESSORY_SLOTS[id];
  if (out.some((a) => ACCESSORY_SLOTS[a] === slot)) return out;
  out.push(id);
  return out;
}
