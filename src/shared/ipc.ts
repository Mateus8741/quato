import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import type { AccessibilityStatus, DuckView, Look, Mood, SettingsView, ShortcutField, ShortcutsConfig, UpdateCheckResult } from './types.ts';

export const EV = {
  duckView: 'duck:view',
  settingsChanged: 'settings:changed',
  accessibilityChanged: 'accessibility:changed',
  settingsGoto: 'settings:goto',
  duckLook: 'duck:look',
  duckSign: 'duck:sign',
  duckGesture: 'duck:gesture',
  fxShot: 'fx:shot',
  fxClear: 'fx:clear',
  fxReady: 'fx:ready',
} as const;

export const api = {
  duckReady: () => invoke<void>('duck_ready'),
  getDuckView: () => invoke<DuckView>('get_duck_view'),
  getSettingsView: () => invoke<SettingsView>('get_settings_view'),
  getLook: () => invoke<Look>('get_look'),
  setColor: (color: string) => invoke<void>('set_color', { color }),
  setAccessories: (accessories: string[]) => invoke<void>('set_accessories', { accessories }),
  setFreeRoam: (enabled: boolean) => invoke<void>('set_free_roam', { enabled }),
  resetHome: () => invoke<void>('reset_home'),
  setWater: (enabled: boolean, intervalMin: number) => invoke<SettingsView>('set_water', { enabled, intervalMin }),
  waterDrank: () => invoke<void>('water_drank'),
  waterTest: () => invoke<void>('water_test'),
  setDanceOnMusic: (enabled: boolean) => invoke<void>('set_dance_on_music', { enabled }),
  checkUpdate: () => invoke<UpdateCheckResult>('check_update'),
  openUpdate: () => invoke<void>('open_update'),
  setMood: (mood: Mood) => invoke<void>('set_mood', { mood }),
  setChasing: (enabled: boolean) => invoke<void>('set_chasing', { enabled }),
  setDragEnabled: (enabled: boolean) => invoke<boolean>('set_drag_enabled', { enabled }),
  releaseMouse: () => invoke<void>('release_mouse'),
  updateShortcuts: (shortcuts: ShortcutsConfig) => invoke<SettingsView>('update_shortcuts', { shortcuts }),
  resetShortcuts: () => invoke<SettingsView>('reset_shortcuts'),
  validateShortcut: (field: ShortcutField, accelerator: string) => invoke<string>('validate_shortcut', { field, accelerator }),
  pauseShortcuts: (paused: boolean) => invoke<void>('pause_shortcuts', { paused }),
  accessibilityStatus: () => invoke<AccessibilityStatus>('accessibility_status'),
  requestAccessibility: () => invoke<AccessibilityStatus>('request_accessibility'),
};

export function on<T>(event: string, cb: (payload: T) => void): Promise<UnlistenFn> {
  return listen<T>(event, (e) => cb(e.payload));
}
