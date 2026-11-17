import { WATER_MAX_INTERVAL, WATER_MIN_INTERVAL } from '../shared/types.ts';

/** Atalhos de intervalo (minutos) da seção "Lembrete de água". */
export const WATER_PRESETS = [15, 30, 45, 60, 90, 120] as const;

/** Lê o campo de minutos: inteiro entre 1 e 720 (ajusta para o limite); null se não for número. */
export function parseInterval(text: string): number | null {
  const t = text.trim();
  if (!/^\d+([.,]\d+)?$/.test(t)) return null;
  const n = Math.round(Number(t.replace(',', '.')));
  if (!Number.isFinite(n)) return null;
  return Math.min(WATER_MAX_INTERVAL, Math.max(WATER_MIN_INTERVAL, n));
}

/** "1 h 30 min", "45 min", "2 h". */
export function formatMinutes(min: number): string {
  const h = Math.floor(min / 60);
  const m = min % 60;
  if (h === 0) return `${m} min`;
  return m === 0 ? `${h} h` : `${h} h ${m} min`;
}

/** Texto do próximo lembrete a partir dos segundos restantes; vazio se desligado. */
export function nextReminderText(sec: number | null): string {
  if (sec === null) return '';
  if (sec < 60) return 'Próximo lembrete em menos de 1 min.';
  return `Próximo lembrete em ~${formatMinutes(Math.ceil(sec / 60))}.`;
}
