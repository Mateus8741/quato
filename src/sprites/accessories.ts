import type { Accessory } from './types.ts';

// Ids e espaços iguais aos de `ACCESSORIES` em src-tauri/src/look.rs (há um teste que confere).
// As coordenadas são relativas ao canto do olho (x,y): a cabeça fica de x-8 a x+4 e de y-3 a y+8.
// Exceção: na layer 'hand' (faca, arminha, varinha) são relativas a `frame.grip`, a mão do bicho.

/** Cerca a grade com 1 pixel de contorno `ch` (a grade cresce 1 em cada lado; ajuste dx/dy em -1). */
function outlined(rows: readonly string[], ch: string): string[] {
  const w = rows[0]!.length + 2;
  const pad = ['.'.repeat(w), ...rows.map((r) => `.${r}.`), '.'.repeat(w)];
  return pad.map((row, y) => [...row].map((c, x) => {
    if (c !== '.') return c;
    for (const [i, j] of [[1, 0], [-1, 0], [0, 1], [0, -1], [1, 1], [-1, -1], [1, -1], [-1, 1]] as const) {
      const n = pad[y + j]?.[x + i];
      if (n !== undefined && n !== '.') return ch;
    }
    return '.';
  }).join(''));
}
export const ACCESSORIES: readonly Accessory[] = [
  {
    id: 'peruca-ruiva', label: 'Peruca ruiva', layer: 'back', dx: -10, dy: -5,
    palette: { R: '#d9531e', r: '#b03a10', h: '#f08040' },
    grid: [
      '.....RRRR....',
      '...RRhhRRRR..',
      '..RRRRhRRRRRR',
      '.RRRRRRRRRRRR',
      '.RRRrRRRRRR..',
      'RRRRrRRR.....',
      'RRRrRRRR.....',
      'RRRrRRR......',
      'RRrRRRR......',
      'RRrRRR.......',
      'RRRrRR.......',
      'RRrRRR.......',
      '.RRrRR.......',
      '.RRRRR.......',
      '.rRRRR.......',
      '..RRR........',
    ],
  },
  {
    id: 'peruca-curta', label: 'Peruca ruiva curta', layer: 'back', dx: -10, dy: -5,
    palette: { R: '#d9531e', r: '#b03a10', h: '#f08040' },
    grid: [
      '.....RRRR....',
      '...RRhhRRRR..',
      '..RRRRhRRRRRR',
      '.RRRRRRRRRRRR',
      '.RRRrRRRRRR..',
      'RRRRrRRR.....',
      'RRRrRRRR.....',
      '.RRRRRR......',
      '..rRRr.......',
    ],
  },
  {
    id: 'chapeu-festa', label: 'Chapéu de festa', layer: 'back', dx: -5, dy: -9,
    palette: { P: '#ff5c8a', Y: '#ffe14a', W: '#ffffff' },
    grid: [
      '...W...',
      '...P...',
      '..PYP..',
      '..PPP..',
      '.PYPPP.',
      '.PPPPY.',
      'YYYYYYY',
    ],
  },
  {
    id: 'coroa', label: 'Coroa', layer: 'back', dx: -6, dy: -7,
    palette: { G: '#ffd23f', g: '#d9a000', J: '#e63946' },
    grid: [
      'G...G...G',
      'GG.GGG.GG',
      'GGGGGGGGG',
      'gJgGgJgGg',
      'ggggggggg',
    ],
  },
  {
    id: 'cartola', label: 'Cartola', layer: 'back', dx: -7, dy: -8,
    palette: { H: '#2a2a33', h: '#4a4a58', B: '#c0392b' },
    grid: [
      '..hHHHHH..',
      '..hHHHHH..',
      '..hHHHHH..',
      '..BBBBBB..',
      '..hHHHHH..',
      'HHHHHHHHHH',
    ],
  },
  {
    id: 'chapeu-seletor', label: 'Chapéu seletor', layer: 'back', dx: -9, dy: -9,
    palette: { B: '#8a5a2e', b: '#6a4120', L: '#b07c45', D: '#3d2410' },
    grid: [
      '..BBB.........',
      '.BBBBB........',
      '..BBLBBB......',
      '...BBBDBBB....',
      '...BBLBBBBB...',
      '..BBBBDBBBBB..',
      '.bBBBBBBBBBBb.',
      'bbbbbbbbbbbbbb',
    ],
  },
  {
    // Uma capivarinha de lado agarrada na cabeça (olhando para onde o bicho olha): corpo baixo e largo,
    // duas orelhinhas, focinho quadrado escuro com narinas claras, olhinho bravo e patinhas curtas.
    // As linhas 7-8 (colunas 7-9) são escuras de propósito: a sobrancelha da raiva cai ali e se funde.
    id: 'chapeu-capivara', label: 'Chapéu de capivara', layer: 'back', dx: -8, dy: -9,
    palette: { D: '#4a2a14', B: '#c9812f', b: '#a0621f', M: '#5a3320', n: '#e8c8a0', E: '#14100d', K: '#3b2412' },
    grid: [
      '.......DD.DD..',
      '......DbbDbbD.',
      '...DDDDBBBBBBD',
      '..DBBBBBBKKBBD',
      '.DBBBBBBBBEBBD',
      '.DBBBBBBBMnMnD',
      '.DBBBBBBBMMMMD',
      '.DbbBBBDDDMMDD',
      '.DbbDbbDDDDD..',
      '..DDD.DD......',
    ],
  },
  {
    id: 'oculos', label: 'Óculos', layer: 'front', dx: -8, dy: -1,
    palette: { F: '#2b2b36' },
    grid: [
      '.......FFFF',
      'FFFFFFFF..F',
      '.......F..F',
      '.......FFFF',
    ],
  },
  {
    id: 'oculos-escuros', label: 'Óculos escuros', layer: 'front', dx: -8, dy: -1,
    palette: { F: '#15151c', W: '#7a7f99' },
    grid: [
      '.......FFFF',
      'FFFFFFFFWFF',
      '.......FFFF',
      '.......FFF.',
    ],
  },
  {
    id: 'fones', label: 'Fones de ouvido', layer: 'front', dx: -6, dy: -4,
    palette: { D: '#2c2f3a', d: '#4a4e5e', A: '#ff4d6d' },
    grid: [
      '..DDDDDD',
      '..D.....',
      '..D.....',
      '..D.....',
      '.dDDd...',
      '.DAAD...',
      '.DAAD...',
      '.DAAD...',
      '.DDDD...',
      '..dd....',
    ],
  },
  {
    id: 'laco', label: 'Laço', layer: 'front', dx: -3, dy: 6,
    palette: { B: '#ff6f9c', b: '#d94a78', K: '#ffb3cc' },
    grid: [
      'BB...BB',
      'bBBKBBb',
      'BB...BB',
    ],
  },
  {
    id: 'flor', label: 'Flor', layer: 'front', dx: -9, dy: -6,
    palette: { O: '#4a3320', P: '#ff7eb6', Y: '#ffd23f' },
    grid: [
      '..OOO..',
      '.OPPPO.',
      'OPPYPPO',
      '.OPPPO.',
      '..OOO..',
    ],
  },
  // ---- espaço "mão" (layer 'hand'): dx/dy em relação a frame.grip, de frente para a direita ----
  {
    // Lâmina de 5-6 px saindo do cabo; a ponta fica em grip + (7,0), de onde pinga a gota do lagartinho.
    id: 'faca', label: 'Faquinha', layer: 'hand', dx: -2, dy: -3,
    palette: { H: '#8a5527', L: '#eef2f7', l: '#a9b3c4', X: '#2a2a3a' },
    grid: outlined([
      'H..LLLLL.',
      'HHHllllll',
      'H........',
    ], 'X'),
  },
  {
    id: 'faca-sangue', label: 'Faquinha com sangue', layer: 'hand', dx: -2, dy: -3,
    palette: { H: '#8a5527', L: '#eef2f7', l: '#a9b3c4', R: '#e3222c', r: '#a3121b', X: '#2a2a3a' },
    grid: outlined([
      'H..LLLRR.',
      'HHHllRRRR',
      'H....R.r.',
      '.....r...',
    ], 'X'),
  },
  {
    // Pistolinha de cartum; o cano termina perto de (29,20) com o bicho olhando para a direita.
    id: 'arminha', label: 'Arminha', layer: 'hand', dx: -3, dy: -3,
    palette: { G: '#4a4e5e', g: '#8a90a6', H: '#8a5527', h: '#5e3a18', X: '#20222c' },
    grid: outlined([
      '.ggggggggg',
      'GGGGGGGGGG',
      '..HHGGG...',
      '..HH.G....',
      '...HH.....',
      '...hh.....',
    ], 'X'),
  },
  {
    // Na mão, apontando para cima e para a frente (a estrelinha fica logo abaixo do bico).
    id: 'varinha', label: 'Varinha de bruxo', layer: 'hand', dx: 0, dy: -6,
    palette: { H: '#4a2e16', h: '#d9a05b', Y: '#ffe680', W: '#ffffff' },
    grid: [
      '......Y.',
      '.....YWY',
      '....HhY.',
      '...Hh...',
      '..Hh....',
      '.Hh.....',
      'Hh......',
    ],
  },
];

export function accessoryById(id: string): Accessory | undefined {
  return ACCESSORIES.find((a) => a.id === id);
}
