import type { ColorDef, Palette } from './types.ts';

// Ids iguais aos de `COLORS` em src-tauri/src/look.rs (há um teste que confere).
export const BASE_PALETTE: Palette = {
  O: '#4a3320', B: '#ffd84d', b: '#f2b92b', K: '#ff9d1c', k: '#e07a10', F: '#ff9d1c',
  E: '#1d1612', W: '#ffffff', C: '#ff9bb0', S: '#4a3320',
};

export const COLORS: readonly ColorDef[] = [
  { id: 'amarelo', label: 'Amarelo', palette: { B: '#ffd84d', b: '#f2b92b' } },
  { id: 'laranja', label: 'Laranja', palette: { B: '#ffb347', b: '#e0892a', K: '#e8632a', k: '#c24a14', F: '#e8632a' } },
  { id: 'branco', label: 'Branco', palette: { B: '#fbfbf8', b: '#dcdcd6' } },
  { id: 'cinza', label: 'Cinza', palette: { B: '#cfd8e3', b: '#a9b6c6', K: '#e0a04a', k: '#c18530', F: '#e0a04a' } },
  { id: 'creme', label: 'Creme', palette: { B: '#e9e1b5', b: '#c9bf8c' } },
  { id: 'rosa', label: 'Rosa', palette: { B: '#ffb6cf', b: '#f08fb0' } },
  { id: 'menta', label: 'Menta', palette: { B: '#a8e6c8', b: '#7fcaa5' } },
  { id: 'azul', label: 'Azul', palette: { B: '#86cdfb', b: '#5aaee8' } },
  { id: 'marrom', label: 'Marrom', palette: { B: '#b98760', b: '#9a6b45', O: '#3a2412', S: '#3a2412' } },
  { id: 'lavanda', label: 'Lavanda', palette: { B: '#cbb6ff', b: '#a98ee8' } },
  { id: 'preto', label: 'Preto', palette: { B: '#2e2e38', b: '#1f1f27', O: '#0e0e13', E: '#f5f5f5', W: '#1a1a1f', S: '#d9dbe8' } },
  { id: 'vermelho', label: 'Vermelho', palette: { B: '#ff7b72', b: '#e0564e' } },
  { id: 'verde', label: 'Verde', palette: { B: '#8fd36a', b: '#6bb847' } },
  { id: 'roxo', label: 'Roxo', palette: { B: '#a77be0', b: '#8657c4' } },
  { id: 'turquesa', label: 'Turquesa', palette: { B: '#5fd9d0', b: '#3cbab1' } },
];

export const DEFAULT_COLOR = 'amarelo';

export function colorById(id: string): ColorDef {
  return COLORS.find((c) => c.id === id) ?? COLORS[0]!;
}
