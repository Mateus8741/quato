import { describe, expect, it } from 'vitest';
import type { DuckView } from '../shared/types.ts';
import { animFor } from './animator.ts';

const view = (o: Partial<DuckView>): DuckView => ({ mode: 'idle', mood: 'feliz', facing: 'right', pace: 'still', chasing: false, dragEnabled: false, ...o });

describe('animFor', () => {
  it('escolhe a animação pelo estado', () => {
    expect(animFor(view({}))).toBe('idle');
    expect(animFor(view({ mode: 'walk', pace: 'slow' }))).toBe('walk');
    expect(animFor(view({ mode: 'chase', pace: 'fast' }))).toBe('run');
    expect(animFor(view({ mode: 'sleep' }))).toBe('sleep');
    expect(animFor(view({ mode: 'grab' }))).toBe('grab');
    expect(animFor(view({ mode: 'dance' }))).toBe('dance');
    expect(animFor(view({ mode: 'dance', mood: 'lagarto' }))).toBe('dance');
  });
  it('segurado pela pessoa usa a animação de bico aberto', () => {
    expect(animFor(view({ mode: 'held' }))).toBe('grab');
  });
});
