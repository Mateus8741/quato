import { invoke } from '@tauri-apps/api/core';
import { unknownChars } from '../sprites/font.ts';
import {
  BREAK_MAX_MIN,
  FOCUS_MAX_MIN,
  REMINDER_TEXT_MAX,
  type ReminderKind,
  type RemindersStatus,
  type SettingsView,
} from '../shared/types.ts';
import { formatMinutes, nextReminderText, parseInterval } from './water.ts';

/** Seções "Lembretes" e "Modo foco" das Configurações (a água tem seção própria, em main.ts). */

export const REMINDER_KINDS: readonly ReminderKind[] = ['posture', 'stretch', 'eyes'];

const INFO: Record<ReminderKind, { title: string; check: string; help: string; done: string }> = {
  posture: {
    title: 'Postura',
    check: 'Lembrar de ajeitar a postura',
    help: 'Costas retas, ombros soltos, pés no chão.',
    done: 'Já ajeitei',
  },
  stretch: {
    title: 'Alongar',
    check: 'Lembrar de alongar',
    help: 'Levante, estique pescoço, ombros e pulsos.',
    done: 'Já alonguei',
  },
  eyes: {
    title: 'Descansar a vista (20-20-20)',
    check: 'Lembrar de descansar a vista',
    help: 'Regra 20-20-20: a cada 20 minutos, olhe por 20 segundos para algo a 6 metros (uns 20 pés).',
    done: 'Já descansei',
  },
};

/** Lê os minutos do foco/pausa: inteiro de 1 até `max` (ajusta ao limite); null se não for número. */
export function parseMinutes(text: string, max: number): number | null {
  const t = text.trim();
  if (!/^\d+([.,]\d+)?$/.test(t)) return null;
  const n = Math.round(Number(t.replace(',', '.')));
  return Number.isFinite(n) ? Math.min(max, Math.max(1, n)) : null;
}

/** Valida o texto da plaquinha: vazio vale (volta ao padrão); devolve o erro ou null. */
export function textError(text: string): string | null {
  const t = text.trim();
  if (t.length > REMINDER_TEXT_MAX) return `Use até ${REMINDER_TEXT_MAX} letras: a plaquinha tem só duas linhas.`;
  const bad = unknownChars(t);
  return bad.length ? `A plaquinha não sabe desenhar: ${bad.join(' ')}` : null;
}

/** "Foco: faltam ~12 min." / "Pausa: ..."; vazio com o foco desligado. */
export function focusStatusText(s: RemindersStatus, elapsedSec = 0): string {
  if (s.focusPhase === 'off' || s.focusRemainingSec === null) return '';
  const left = Math.max(0, s.focusRemainingSec - elapsedSec);
  const what = s.focusPhase === 'focus' ? 'Foco' : 'Pausa';
  return left < 60 ? `${what}: falta menos de 1 min.` : `${what}: faltam ~${formatMinutes(Math.ceil(left / 60))}.`;
}

function nextOf(s: RemindersStatus, k: ReminderKind): number | null {
  return k === 'posture' ? s.postureNextInSec : k === 'stretch' ? s.stretchNextInSec : s.eyesNextInSec;
}

const esc = (s: string): string => s.replace(/[&<>"']/g, (c) => `&#${c.charCodeAt(0)};`);

let snapshot: { status: RemindersStatus; at: number } | null = null;
const msgs: Partial<Record<ReminderKind | 'focus', string>> = {};

function elapsed(): number {
  return snapshot ? (performance.now() - snapshot.at) / 1000 : 0;
}

function nextText(k: ReminderKind): string {
  if (!snapshot) return '';
  const sec = nextOf(snapshot.status, k);
  return nextReminderText(sec === null ? null : Math.max(0, sec - elapsed()));
}

export function remindersHtml(v: SettingsView): string {
  snapshot = { status: v.reminders, at: performance.now() };
  const items = REMINDER_KINDS.map((k) => {
    const c = v.config.reminders[k];
    const i = INFO[k];
    return `<div class="rem" data-rem-box="${k}">
      <h3>${i.title}</h3>
      <label class="check"><input type="checkbox" data-rem-on="${k}" ${c.enabled ? 'checked' : ''} /> ${i.check}</label>
      <div class="water-row">
        <label for="rem-min-${k}">A cada</label>
        <input id="rem-min-${k}" type="number" inputmode="numeric" min="1" max="720" step="1" value="${c.intervalMin}" data-rem-min="${k}" />
        <span>minutos</span>
      </div>
      <div class="water-row">
        <label for="rem-text-${k}">Plaquinha</label>
        <input id="rem-text-${k}" type="text" maxlength="${REMINDER_TEXT_MAX}" value="${esc(c.text)}" data-rem-text="${k}" style="width:200px" />
      </div>
      ${msgs[k] ? `<p class="err" role="alert">${esc(msgs[k]!)}</p>` : ''}
      <p class="note">${i.help}</p>
      <p class="note" data-rem-next="${k}" aria-live="polite">${esc(nextText(k))}</p>
      <div class="buttons"><button data-rem-test="${k}">Testar plaquinha</button>
      <button data-rem-done="${k}" ${c.enabled ? '' : 'disabled'}>${i.done}</button></div>
    </div>`;
  }).join('');

  const f = v.config.focus;
  const phase = v.reminders.focusPhase;
  const btn = phase === 'off' ? 'Iniciar foco' : phase === 'focus' ? 'Parar foco' : 'Pular a pausa';
  const focusHtml = `<section id="focus"><h2>Modo foco (Pomodoro)</h2>
      <div class="water-row">
        <label for="focus-min">Foco</label>
        <input id="focus-min" type="number" inputmode="numeric" min="1" max="${FOCUS_MAX_MIN}" step="1" value="${f.focusMin}" data-focus-min />
        <span>min</span>
        <label for="break-min">Pausa</label>
        <input id="break-min" type="number" inputmode="numeric" min="1" max="${BREAK_MAX_MIN}" step="1" value="${f.breakMin}" data-break-min />
        <span>min</span>
      </div>
      ${msgs.focus ? `<p class="err" role="alert">${esc(msgs.focus)}</p>` : ''}
      <p class="note">Durante o foco o pato dorme e fica quieto, e todos os lembretes (inclusive o de água) ficam suspensos. No fim, uma plaquinha anuncia a pausa; depois dela tudo volta ao normal. Também dá para iniciar e parar pelo menu da barra.</p>
      <p class="note" data-focus-status aria-live="polite">${esc(focusStatusText(v.reminders))}</p>
      <div class="buttons"><button class="${phase === 'off' ? 'primary' : ''}" data-focus-toggle>${btn}</button></div></section>`;

  return `<section id="reminders"><h2>Lembretes</h2>
      <p class="note">Além da água: cada um liga e desliga sozinho, com intervalo e texto próprios. Contam só com o computador ligado.</p>
      ${items}</section>
    ${focusHtml}`;
}

/** Seletor dos campos que não devem ser apagados por um redesenho enquanto a pessoa digita. */
export const REMINDER_INPUTS = '[data-rem-min],[data-rem-text],[data-focus-min],[data-break-min]';

function field(root: HTMLElement, sel: string): HTMLInputElement {
  return root.querySelector<HTMLInputElement>(sel)!;
}

/** Liga os cliques e mudanças das duas seções (uma vez só; usa delegação no `app`). */
export function bindReminders(app: HTMLElement, getView: () => SettingsView | null, setView: (v: SettingsView) => void): void {
  async function saveReminder(k: ReminderKind): Promise<void> {
    const v = getView();
    if (!v) return;
    const enabled = field(app, `[data-rem-on="${k}"]`).checked;
    const minEl = field(app, `[data-rem-min="${k}"]`);
    const textEl = field(app, `[data-rem-text="${k}"]`);
    const n = parseInterval(minEl.value);
    const err = n === null ? 'Digite um número de minutos entre 1 e 720.' : textError(textEl.value);
    if (err || n === null) {
      msgs[k] = err ?? '';
      setView(v);
      return;
    }
    msgs[k] = undefined;
    setView(await invoke<SettingsView>('set_reminder', { kind: k, enabled, intervalMin: n, text: textEl.value }));
  }

  async function saveFocus(): Promise<void> {
    const v = getView();
    if (!v) return;
    const f = parseMinutes(field(app, '[data-focus-min]').value, FOCUS_MAX_MIN);
    const b = parseMinutes(field(app, '[data-break-min]').value, BREAK_MAX_MIN);
    if (f === null || b === null) {
      msgs.focus = `Digite os minutos: foco de 1 a ${FOCUS_MAX_MIN}, pausa de 1 a ${BREAK_MAX_MIN}.`;
      setView(v);
      return;
    }
    msgs.focus = undefined;
    setView(await invoke<SettingsView>('set_focus_config', { focusMin: f, breakMin: b }));
  }

  app.addEventListener('change', (e) => {
    const t = e.target as HTMLElement;
    const kind = (t.dataset.remOn ?? t.dataset.remMin ?? t.dataset.remText) as ReminderKind | undefined;
    if (kind) void saveReminder(kind);
    else if (t.matches('[data-focus-min],[data-break-min]')) void saveFocus();
  });

  app.addEventListener('click', (e) => {
    const t = (e.target as HTMLElement).closest<HTMLElement>('[data-rem-test],[data-rem-done],[data-focus-toggle]');
    if (!t) return;
    if (t.dataset.remTest) void invoke('reminder_test', { kind: t.dataset.remTest });
    else if (t.dataset.remDone) void invoke('reminder_done', { kind: t.dataset.remDone });
    else if (t.dataset.focusToggle !== undefined) void invoke<SettingsView>('focus_toggle').then(setView);
  });

  app.addEventListener('keydown', (e) => {
    const t = e.target as HTMLInputElement;
    if (e.key === 'Enter' && t.matches?.(REMINDER_INPUTS)) t.blur();
  });

  // Atualiza só os textos de contagem (sem redesenhar a tela toda).
  window.setInterval(() => {
    for (const k of REMINDER_KINDS) {
      const el = app.querySelector(`[data-rem-next="${k}"]`);
      if (el) el.textContent = nextText(k);
    }
    const st = app.querySelector('[data-focus-status]');
    if (st && snapshot) st.textContent = focusStatusText(snapshot.status, elapsed());
  }, 10_000);
}
