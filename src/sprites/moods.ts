import type { Mood } from '../shared/types.ts';
import type { MoodSkin } from './types.ts';

// O humor muda só a expressão, o efeito e o ritmo. A cor do corpo vem de colors.ts.
const fpsDefault = { idle: 2, walk: 6, run: 10, grab: 8, sleep: 2, dance: 8 } as const;

export const SKINS: Readonly<Record<Mood, MoodSkin>> = {
  feliz: {
    mood: 'feliz', label: 'Feliz', cheekColor: '#ff9bb0',
    eyes: ['WE', 'EE'], eyesClosed: ['.E.', 'E.E'],
    cheek: true, effect: 'sparkle', fps: fpsDefault,
  },
  raiva: {
    mood: 'raiva', label: 'Com raiva', cheekColor: '#e8423f',
    eyes: ['EE', 'EE'], eyesClosed: ['EE'],
    brow: { grid: ['SS.', '.SS'], dx: -1, dy: -2 },
    cheek: true, effect: 'steam', fps: { ...fpsDefault, idle: 4, run: 14 },
  },
  triste: {
    mood: 'triste', label: 'Triste', cheekColor: '#b9c7e6',
    eyes: ['EE', 'EW', '.E'], eyesClosed: ['EE'],
    brow: { grid: ['.SS', 'SS.'], dx: -1, dy: -2 },
    cheek: false, effect: 'tear', fps: { ...fpsDefault, walk: 4, run: 6 },
  },
  sozinho: {
    mood: 'sozinho', label: 'Sozinho', cheekColor: '#d9b6a6',
    eyes: ['EW', 'EE'], eyesClosed: ['EE'],
    cheek: false, effect: 'dots', fps: { ...fpsDefault, walk: 5 },
  },
  sono: {
    mood: 'sono', label: 'Com sono', cheekColor: '#e9b9a0',
    eyes: ['EE', '..'], eyesClosed: ['EE'],
    cheek: true, effect: 'zzz', fps: { ...fpsDefault, walk: 4, run: 6, sleep: 1.5 },
  },
  // Dançando: olhinhos fechados de felicidade ("^ ^"), bochecha rosada e notas musicais.
  danca: {
    mood: 'danca', label: 'Dançando', cheekColor: '#ff7fa8',
    eyes: ['.E.', 'E.E'], eyesClosed: ['EEE'],
    cheek: true, effect: 'notes', fps: { ...fpsDefault, walk: 7, dance: 8 },
  },
  // Lagartinho: no lugar do pato aparece o lagarto (src/sprites/lizard.ts), olho arregalado e a gota
  // pingando da faquinha. O "ataque de grito" fica em src/sprites/scream.ts.
  lagarto: {
    mood: 'lagarto', label: 'Lagartinho', cheekColor: '#ff9bb0',
    eyes: ['WE', 'EE'], eyesClosed: ['SS'],
    cheek: false, effect: 'drip', fps: { ...fpsDefault, walk: 7, run: 12 },
  },
  // Duvidoso: olha para cima de lado, uma sobrancelha arqueada e o '?' pairando sobre a cabeça.
  duvida: {
    mood: 'duvida', label: 'Duvidoso', cheekColor: '#ffb39b',
    eyes: ['WE', 'EE'], eyesClosed: ['EE'],
    brow: { grid: ['.SS', 'S..'], dx: -1, dy: -2 },
    cheek: false, effect: 'question', fps: fpsDefault,
  },
};
