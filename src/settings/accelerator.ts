import type { Platform } from '../shared/types.ts';

export interface KeyInfo {
  code: string;
  metaKey: boolean;
  ctrlKey: boolean;
  altKey: boolean;
  shiftKey: boolean;
}

export type AccelResult = { ok: string } | { error: 'modifierOnly' | 'needsModifier' | 'unsupportedKey' };

const MODIFIER_CODES = /^(Control|Shift|Alt|Meta|OS)(Left|Right)?$/;

function keyFromCode(code: string): string | null {
  let m = /^Key([A-Z])$/.exec(code);
  if (m) return m[1]!;
  m = /^Digit([0-9])$/.exec(code);
  if (m) return m[1]!;
  m = /^F([0-9]{1,2})$/.exec(code);
  if (m && Number(m[1]) >= 1 && Number(m[1]) <= 12) return `F${m[1]}`;
  if (code === 'Space' || /^Arrow(Up|Down|Left|Right)$/.test(code)) return code;
  return null;
}

/** Converte um keydown no formato salvo ("CommandOrControl+Shift+D"). */
export function eventToAccelerator(e: KeyInfo, platform: Platform): AccelResult {
  if (MODIFIER_CODES.test(e.code)) return { error: 'modifierOnly' };
  const key = keyFromCode(e.code);
  if (!key) return { error: 'unsupportedKey' };
  const mods: string[] = [];
  if (platform === 'macos') {
    if (e.metaKey) mods.push('CommandOrControl');
    if (e.ctrlKey) mods.push('Control');
  } else {
    if (e.ctrlKey) mods.push('CommandOrControl');
    if (e.metaKey) mods.push('Super');
  }
  if (e.altKey) mods.push('Alt');
  const real = mods.length > 0;
  if (e.shiftKey) mods.push('Shift');
  if (!real) return { error: 'needsModifier' };
  return { ok: [...mods, key].join('+') };
}

const ARROWS: Record<string, string> = { ArrowUp: '↑', ArrowDown: '↓', ArrowLeft: '←', ArrowRight: '→' };

/** Texto legível: ⇧⌘D no Mac, Ctrl+Shift+D no Windows. */
export function formatAccelerator(accel: string, platform: Platform): string {
  const parts = accel.split('+');
  const key = parts.pop() ?? '';
  const has = (...names: string[]) => parts.some((p) => names.includes(p));
  const k = ARROWS[key] ?? key;
  if (platform === 'macos') {
    return (
      (has('Control', 'Ctrl') ? '⌃' : '') +
      (has('Alt', 'Option') ? '⌥' : '') +
      (has('Shift') ? '⇧' : '') +
      (has('CommandOrControl', 'CmdOrCtrl', 'Command', 'Cmd', 'Super') ? '⌘' : '') +
      k
    );
  }
  const out: string[] = [];
  if (has('CommandOrControl', 'CmdOrCtrl', 'Control', 'Ctrl')) out.push('Ctrl');
  if (has('Alt', 'Option')) out.push('Alt');
  if (has('Shift')) out.push('Shift');
  if (has('Super', 'Command', 'Cmd')) out.push('Win');
  out.push(k);
  return out.join('+');
}
