// Espelho dos tipos de src-tauri/src/events.rs. Mantenha os dois lados iguais.
export type Mood = 'feliz' | 'raiva' | 'triste' | 'sozinho' | 'sono' | 'danca' | 'lagarto' | 'duvida';
export const MOODS: readonly Mood[] = ['feliz', 'raiva', 'triste', 'sozinho', 'sono', 'danca', 'lagarto', 'duvida'];
export type DuckMode = 'idle' | 'walk' | 'chase' | 'grab' | 'sleep' | 'held' | 'dance';
export type Pace = 'still' | 'slow' | 'fast';
export type Facing = 'left' | 'right';
export type Platform = 'macos' | 'windows';

export interface DuckView {
  mode: DuckMode;
  mood: Mood;
  facing: Facing;
  pace: Pace;
  chasing: boolean;
  dragEnabled: boolean;
}

export interface ShortcutsConfig {
  toggleChase: string;
  cycleMood: string;
  /** Joga uma migalha na posição do cursor. */
  dropCrumb: string;
}
export type ShortcutField = keyof ShortcutsConfig;

export interface Config {
  version: number;
  mood: Mood;
  shortcuts: ShortcutsConfig;
  accessibilityExplained: boolean;
  firstRunDone: boolean;
  color: string;
  accessories: string[];
  freeRoam: boolean;
  /** Lugar escolhido ao carregar o pato com Ctrl; null = canto padrão. */
  home: [number, number] | null;
  /** Lembrete de água (intervalo em minutos, 1..720). */
  water: WaterConfig;
  /** Lembretes de postura, alongar e vista. */
  reminders: RemindersConfig;
  /** Durações do modo foco. */
  focus: FocusConfig;
  /** Dança quando tem som saindo do computador. */
  danceOnMusic: boolean;
  /** Balão de fala do pato. */
  speech: SpeechConfig;
  /** Última versão nova já avisada com a plaquinha. */
  notifiedUpdate?: string | null;
  notifiedAt?: number;
  /** Acessórios sazonais (Natal, Halloween, Ano Novo) quando o espaço está livre. */
  seasonalAccessories?: boolean;
  /** Modo "não perturbe": esconde o pato com app em tela cheia no monitor dele. */
  dnd: boolean;
  /** Abrir ao ligar o computador. */
  launchAtLogin: boolean;
}

export interface WaterConfig {
  enabled: boolean;
  intervalMin: number;
}

/** Com que frequência o pato fala sozinho. Espelha `SpeechFrequency` do Rust. */
export type SpeechFrequency = 'never' | 'rare' | 'sometimes' | 'always';
export const SPEECH_FREQUENCIES: readonly SpeechFrequency[] = ['never', 'rare', 'sometimes', 'always'];

export interface SpeechConfig {
  frequency: SpeechFrequency;
  /** Frases extras da pessoa (valem para qualquer humor). */
  extra: string[];
}

/** Limites das frases extras (o balão tem 2 linhas); o Rust arruma de novo ao salvar. */
export const SPEECH_MAX_EXTRA = 30;
export const SPEECH_MAX_CHARS = 36;

export const WATER_MIN_INTERVAL = 1;
export const WATER_MAX_INTERVAL = 720;

/** Aparência do pato, separada do humor. Espelha `Look` do Rust. */
export interface Look {
  color: string;
  accessories: string[];
  /** Acessórios sazonais ligados (quem desenha aplica a data). */
  seasonal?: boolean;
}

export interface ShortcutStatus {
  toggleChase: string | null;
  cycleMood: string | null;
  dropCrumb: string | null;
}

export interface AccessibilityStatus {
  required: boolean;
  trusted: boolean;
}

export interface SettingsView {
  config: Config;
  dragEnabled: boolean;
  shortcutStatus: ShortcutStatus;
  shortcutsPaused: boolean;
  platform: Platform;
  accessibility: AccessibilityStatus;
  defaults: ShortcutsConfig;
  /** Segundos até o próximo lembrete de água; null = desligado. */
  waterNextInSec: number | null;
  /** Contagens dos outros lembretes e fase do modo foco. */
  reminders: RemindersStatus;
  /** Versão instalada (ex.: "0.3.1"). */
  appVersion: string;
  /** Versão mais nova disponível; null = em dia (ou ainda não verificou). */
  update: UpdateInfo | null;
}

export interface UpdateInfo {
  version: string;
  notes: string;
  /** Já validada pelo Rust: sempre https no site do Quato. */
  url: string;
}

export interface UpdateCheckResult {
  status: 'upToDate' | 'available' | 'error';
  update: UpdateInfo | null;
}

export interface ShortcutError {
  field: ShortcutField | null;
  kind: 'invalid' | 'noModifier' | 'duplicate' | 'reserved' | 'inUse';
  message: string;
}

export interface DragError {
  kind: 'accessibilityRequired' | 'pointerUnavailable';
  message: string;
}

/** Evento `duck:sign`: o pato mostra uma plaquinha (água, atualização) ou um balão de fala (`speech`). */
export interface DuckSign {
  kind: 'water' | 'update' | 'speech';
  text: string;
  /** Quanto tempo a plaquinha fica na tela; 0 = esconder agora. */
  durationMs: number;
}

/** Evento `duck:gesture`: o pato faz um gesto rápido (hoje só a "pena do meio"). */
export interface DuckGesture {
  kind: 'feather';
  durationMs: number;
}

/** Lembretes além da água (espelha `reminders.rs`). */
export type ReminderKind = 'posture' | 'stretch' | 'eyes';

export interface ReminderConfig {
  enabled: boolean;
  /** Minutos entre um lembrete e outro (1..720). */
  intervalMin: number;
  /** Texto da plaquinha (até 28 letras). */
  text: string;
}

export interface RemindersConfig {
  posture: ReminderConfig;
  stretch: ReminderConfig;
  eyes: ReminderConfig;
}

/** Durações do modo foco (Pomodoro), em minutos. */
export interface FocusConfig {
  focusMin: number;
  breakMin: number;
  /** Ciclos foco -> pausa seguidos (1 a 12; 1 = um só). */
  cycles: number;
}

export interface RemindersStatus {
  postureNextInSec: number | null;
  stretchNextInSec: number | null;
  eyesNextInSec: number | null;
  focusPhase: 'off' | 'focus' | 'break';
  /** Segundos até o fim da fase atual; null = foco desligado. */
  focusRemainingSec: number | null;
  /** Ciclo atual (a partir de 1) e total; 0 com o foco desligado. */
  focusCycle: number;
  focusCyclesTotal: number;
}

export const FOCUS_MAX_MIN = 180;
export const BREAK_MAX_MIN = 60;
export const FOCUS_CYCLES_MAX = 12;
export const REMINDER_TEXT_MAX = 28;
