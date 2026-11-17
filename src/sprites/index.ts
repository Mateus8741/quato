import { MOODS, type Mood } from '../shared/types.ts';
import { ANIMS } from './base.ts';
import { SKINS } from './moods.ts';
import { COLORS } from './colors.ts';
import { ACCESSORIES } from './accessories.ts';
import { LIZARD_ANIMS, LIZARD_SCREAM_ANIMS, LIZARD_PALETTE } from './lizard.ts';
import { SIZE, ANIM_NAMES, type AnimName, type Character, type Frame, type MoodSkin, type PixelGrid } from './types.ts';

export { SKINS, ANIMS, SIZE, COLORS, ACCESSORIES, ANIM_NAMES, LIZARD_ANIMS, LIZARD_SCREAM_ANIMS, LIZARD_PALETTE };
export { MOODS };
export { characterFor, framesFor } from './compose.ts';
export type { AnimName, Character, Frame, MoodSkin, PixelGrid, Mood };
export type { Look, Accessory, ColorDef } from './types.ts';

export function validateGrid(grid: PixelGrid, palette: Readonly<Record<string, string>>): string[] {
  const errors: string[] = [];
  if (grid.length !== SIZE) errors.push(`altura ${grid.length} (esperado ${SIZE})`);
  grid.forEach((row, y) => {
    if (row.length !== SIZE) errors.push(`linha ${y} tem ${row.length} colunas`);
    for (const ch of row) if (ch !== '.' && !(ch in palette)) errors.push(`char "${ch}" sem cor na linha ${y}`);
  });
  return errors;
}
