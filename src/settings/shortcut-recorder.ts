import { api } from '../shared/ipc.ts';
import type { Platform, ShortcutError, ShortcutField, ShortcutsConfig } from '../shared/types.ts';
import { eventToAccelerator } from './accelerator.ts';

export interface RecorderHooks {
  onStart: (field: ShortcutField) => void;
  onEnd: () => void;
  onError: (field: ShortcutField, message: string) => void;
  /** Config atual, para montar o par completo ao salvar. */
  current: () => ShortcutsConfig;
  platform: () => Platform;
}

const KEY_ERRORS = {
  modifierOnly: null,
  needsModifier: 'Use pelo menos Ctrl, Alt ou ⌘ junto com a tecla.',
  unsupportedKey: 'Essa tecla não pode ser usada. Use letras, números, F1–F12, espaço ou setas.',
} as const;

let recording = false;

/** Grava uma combinação nova. Pausa os atalhos globais enquanto grava e sempre os retoma no fim. */
export async function recordShortcut(field: ShortcutField, hooks: RecorderHooks): Promise<void> {
  if (recording) return;
  recording = true;
  await api.pauseShortcuts(true);
  hooks.onStart(field);
  try {
    const accel = await waitForCombo(hooks.platform(), (msg) => hooks.onError(field, msg));
    if (accel) {
      const next: ShortcutsConfig = { ...hooks.current(), [field]: accel };
      try {
        await api.validateShortcut(field, accel);
        await api.updateShortcuts(next);
      } catch (e) {
        const err = e as ShortcutError;
        hooks.onError((err.field ?? field) as ShortcutField, err.message ?? String(e));
      }
    }
  } finally {
    recording = false;
    await api.pauseShortcuts(false);
    hooks.onEnd();
  }
}

function waitForCombo(platform: Platform, onHint: (m: string) => void): Promise<string | null> {
  return new Promise((resolve) => {
    const finish = (value: string | null) => {
      window.removeEventListener('keydown', onKey, true);
      window.removeEventListener('blur', onBlur);
      resolve(value);
    };
    const onBlur = () => finish(null);
    const onKey = (e: KeyboardEvent) => {
      e.preventDefault();
      e.stopPropagation();
      if (e.code === 'Escape') return finish(null);
      const r = eventToAccelerator(e, platform);
      if ('ok' in r) return finish(r.ok);
      const msg = KEY_ERRORS[r.error];
      if (msg) onHint(msg);
    };
    window.addEventListener('keydown', onKey, true);
    window.addEventListener('blur', onBlur);
  });
}
