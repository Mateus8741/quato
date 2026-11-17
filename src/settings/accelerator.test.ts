import { describe, expect, it } from 'vitest';
import { eventToAccelerator, formatAccelerator } from './accelerator.ts';

const ev = (code: string, o: Partial<{ metaKey: boolean; ctrlKey: boolean; altKey: boolean; shiftKey: boolean }> = {}) => ({
  code, metaKey: false, ctrlKey: false, altKey: false, shiftKey: false, ...o,
});

describe('eventToAccelerator', () => {
  it('mac: cmd+shift+D', () => {
    expect(eventToAccelerator(ev('KeyD', { metaKey: true, shiftKey: true }), 'macos')).toEqual({ ok: 'CommandOrControl+Shift+D' });
  });
  it('windows: ctrl+shift+D', () => {
    expect(eventToAccelerator(ev('KeyD', { ctrlKey: true, shiftKey: true }), 'windows')).toEqual({ ok: 'CommandOrControl+Shift+D' });
  });
  it('windows: win+alt+1', () => {
    expect(eventToAccelerator(ev('Digit1', { metaKey: true, altKey: true }), 'windows')).toEqual({ ok: 'Super+Alt+1' });
  });
  it('mac: ctrl+alt+seta', () => {
    expect(eventToAccelerator(ev('ArrowUp', { ctrlKey: true, altKey: true }), 'macos')).toEqual({ ok: 'Control+Alt+ArrowUp' });
  });
  it('só modificador', () => {
    expect(eventToAccelerator(ev('ShiftLeft', { shiftKey: true }), 'macos')).toEqual({ error: 'modifierOnly' });
  });
  it('só shift não vale', () => {
    expect(eventToAccelerator(ev('KeyD', { shiftKey: true }), 'macos')).toEqual({ error: 'needsModifier' });
  });
  it('tecla não suportada', () => {
    expect(eventToAccelerator(ev('Tab', { metaKey: true }), 'macos')).toEqual({ error: 'unsupportedKey' });
  });
  it('F1..F12 sim, F13 não', () => {
    expect(eventToAccelerator(ev('F5', { altKey: true }), 'windows')).toEqual({ ok: 'Alt+F5' });
    expect(eventToAccelerator(ev('F13', { altKey: true }), 'windows')).toEqual({ error: 'unsupportedKey' });
  });
});

describe('formatAccelerator (mesmos casos do Rust)', () => {
  it('formata', () => {
    expect(formatAccelerator('CommandOrControl+Shift+D', 'macos')).toBe('⇧⌘D');
    expect(formatAccelerator('CommandOrControl+Shift+D', 'windows')).toBe('Ctrl+Shift+D');
    expect(formatAccelerator('Control+Alt+ArrowUp', 'macos')).toBe('⌃⌥↑');
  });
});
