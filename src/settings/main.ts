import { api, EV, on } from '../shared/ipc.ts';
import type { AccessibilityStatus, DragError, Look, Mood, SettingsView, ShortcutField } from '../shared/types.ts';
import { MOODS } from '../shared/types.ts';
import { composeRGBA, framesFor } from '../sprites/compose.ts';
import { ACCESSORIES, COLORS, SKINS } from '../sprites/index.ts';
import { formatAccelerator } from './accelerator.ts';
import { recordShortcut } from './shortcut-recorder.ts';
import { nextReminderText, parseInterval, WATER_PRESETS } from './water.ts';

const app = document.getElementById('app')!;
let view: SettingsView | null = null;
let recordingField: ShortcutField | null = null;
let shortcutMsg: Partial<Record<ShortcutField, string>> = {};
let dragMsg = '';
let accessNote = '';
let pollTimer: number | undefined;
let waterMsg = '';
let updateMsg = '';
let updateBusy = false;
/** Segundos até o próximo lembrete, e quando isso foi lido (para a contagem local). */
let waterNext: { sec: number | null; at: number } = { sec: null, at: 0 };
/** Chegou uma atualização enquanto a pessoa digitava os minutos: redesenha ao sair do campo. */
let pendingRender = false;

const FIELDS: { field: ShortcutField; label: string; hint: string }[] = [
  { field: 'toggleChase', label: 'Perseguir o cursor', hint: 'Liga e desliga a perseguição' },
  { field: 'cycleMood', label: 'Trocar de humor', hint: 'Passa para o próximo humor' },
];

const esc = (s: string) => s.replace(/[&<>"]/g, (c) => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;' })[c]!);

const previews = new Map<string, string>();

/** Imagem do personagem do humor (pato ou lagartinho; parado, olhando para a direita) com o humor e a aparência dados. */
function preview(mood: Mood, look: Look): string {
  const key = `${mood}|${look.color}|${look.accessories.join(',')}`;
  const hit = previews.get(key);
  if (hit) return hit;
  const c = document.createElement('canvas');
  c.width = 32; c.height = 32;
  const px = composeRGBA(SKINS[mood], framesFor(mood, 'idle')[0]!, false, look);
  c.getContext('2d')!.putImageData(new ImageData(px as unknown as Uint8ClampedArray<ArrayBuffer>, 32, 32), 0, 0);
  const url = c.toDataURL();
  previews.set(key, url);
  return url;
}

function option(attr: string, value: string, label: string, img: string, on: boolean): string {
  return `<button class="mood ${on ? 'on' : ''}" ${attr}="${value}" aria-pressed="${on}">
    <img src="${img}" alt="" width="64" height="64" /><span>${esc(label)}</span></button>`;
}

function render(): void {
  const v = view;
  if (!v) return;
  const platform = v.platform;
  const look: Look = { color: v.config.color, accessories: v.config.accessories };
  const moodHtml = MOODS.map((m) => option('data-mood', m, SKINS[m].label, preview(m, look), v.config.mood === m)).join('');
  const colorHtml = COLORS.map((c) =>
    option('data-color', c.id, c.label, preview('feliz', { color: c.id, accessories: [] }), look.color === c.id),
  ).join('');
  const lizard = v.config.mood === 'lagarto';
  const accHtml2 = ACCESSORIES.map((a) =>
    option('data-acc', a.id, a.label, preview(v.config.mood, { color: look.color, accessories: [a.id] }), look.accessories.includes(a.id)),
  ).join('');

  const shortcutHtml = FIELDS.map(({ field, label, hint }) => {
    const rec = recordingField === field;
    const status = v.shortcutStatus[field];
    const msg = shortcutMsg[field] ?? status ?? '';
    return `<div class="row">
      <div class="grow"><strong>${label}</strong><small>${hint}</small>${msg ? `<small class="err">${esc(msg)}</small>` : ''}</div>
      <kbd class="${rec ? 'rec' : ''}">${rec ? 'Pressione… (Esc cancela)' : esc(formatAccelerator(v.config.shortcuts[field], platform))}</kbd>
      <button data-record="${field}" ${recordingField ? 'disabled' : ''}>Alterar</button></div>`;
  }).join('');

  const water = v.config.water;
  const presetHtml = WATER_PRESETS.map((m) =>
    `<button class="preset ${water.intervalMin === m ? 'on' : ''}" data-water-preset="${m}" aria-pressed="${water.intervalMin === m}">${m} min</button>`,
  ).join('');
  const waterHtml = `<section id="water"><h2>Lembrete de água</h2>
      <label class="check"><input type="checkbox" data-water ${water.enabled ? 'checked' : ''} /> Lembrar de beber água</label>
      <div class="water-row">
        <label for="water-min">A cada</label>
        <input id="water-min" type="number" inputmode="numeric" min="1" max="720" step="1" value="${water.intervalMin}" data-water-min aria-describedby="water-help" />
        <span>minutos</span>
      </div>
      <div class="presets" role="group" aria-label="Intervalos prontos">${presetHtml}</div>
      ${waterMsg ? `<p class="err" role="alert">${esc(waterMsg)}</p>` : ''}
      <p class="note" id="water-help">De 1 a 720 minutos (12 h). Na hora, o pato mostra uma plaquinha por 15 segundos. “Já bebi água” (aqui ou no menu) esconde a plaquinha e recomeça a contagem.</p>
      <p class="note" id="water-next" aria-live="polite">${esc(waterNextText())}</p>
      <div class="buttons"><button data-water-test>Testar plaquinha</button>
      <button data-water-drank ${water.enabled ? '' : 'disabled'}>Já bebi água</button></div></section>`;

  const acc = v.accessibility;
  const accHtml = acc.required
    ? `<section id="accessibility"><h2>Permissão de Acessibilidade</h2>
      <p>${acc.trusted ? '✅ Permissão concedida.' : 'Para mexer no mouse de verdade, o macOS pede a permissão de <b>Acessibilidade</b>. O Quato só usa isso para mover o cursor, nunca para clicar nem ler o que você digita.'}</p>
      ${acc.trusted ? '' : '<button data-access>Abrir Ajustes do Sistema</button>'}
      ${accessNote ? `<p class="ok">${esc(accessNote)}</p>` : ''}</section>`
    : '';

  const up = v.update;
  const bannerHtml = up
    ? `<section class="update-banner" role="status"><div class="grow"><strong>Nova versão ${esc(up.version)} disponível</strong>
      ${up.notes ? `<small>${esc(up.notes)}</small>` : ''}</div><button class="primary" data-update-open>Baixar agora</button></section>`
    : '';
  const footerHtml = `<footer class="about"><span>Quato v${esc(v.appVersion)}</span>
      <button data-update-check ${updateBusy ? 'disabled' : ''}>Procurar atualizações</button>
      <span class="note" id="update-result" role="status" aria-live="polite">${esc(updateMsg)}</span></footer>`;

  app.innerHTML = `
    ${bannerHtml}
    <h1>Quato</h1>
    <section><h2>Humor</h2><div class="moods mood-grid">${moodHtml}</div>
      <p class="note">O humor muda a cara, o jeito de andar e os efeitos. A cor e os acessórios você escolhe à parte.</p></section>
    <section id="color" class="${lizard ? 'dim' : ''}"><h2>Cor</h2><div class="moods">${colorHtml}</div>
      ${lizard ? '<p class="note">O lagartinho é sempre verde. A cor vale para o pato.</p>' : ''}</section>
    <section><h2>Acessórios</h2><div class="moods">${accHtml2}</div>
      <p class="note">Dá para combinar, um por lugar: um item de cabeça (peruca, chapéu, gorro, coroa), um de rosto (óculos, tapa-olho), fones, laço ou capa, mochila e flor.</p>
      <label class="check"><input type="checkbox" data-seasonal ${v.config.seasonalAccessories ? 'checked' : ''} /> Acessórios sazonais</label>
      <p class="note">Ligado, em datas especiais (gorro de Natal de 15 a 26/dez, chapéu de bruxa de 25 a 31/out, chapéu de festa no Ano Novo, de 31/dez a 2/jan) o pato usa o acessório da data, só se você não escolheu nenhum item de cabeça.</p>
      <button data-acc-clear ${look.accessories.length ? '' : 'disabled'}>Tirar todos os acessórios</button></section>
    <section><h2>Passeio</h2>
      <label class="check"><input type="checkbox" data-roam ${v.config.freeRoam ? 'checked' : ''} /> Passear pela tela toda</label>
      <p class="note">Desligado, ele anda só pertinho do lugar dele. Ligado, vai para qualquer canto da tela.</p>
      <p class="note"><b>Mudar o lugar dele:</b> segure <kbd class="mini">${platform === 'macos' ? '⌃ Ctrl' : 'Ctrl'}</kbd> e arraste o pato. Ele passa a andar por onde você soltar.</p>
      <button data-home-reset ${v.config.home ? '' : 'disabled'}>Voltar para o canto</button></section>
    ${waterHtml}
    <section><h2>Dança</h2>
      <label class="check"><input type="checkbox" data-dance-music ${v.config.danceOnMusic ? 'checked' : ''} /> Dançar quando tiver som tocando</label>
      <p class="note">Ele percebe quando tem <b>som saindo do computador</b>, seja música, vídeo, chamada ou um aviso, e dança enquanto durar (para uns segundos depois que o som acaba). Não ouve nem grava nada: só pergunta ao sistema se a saída de som está tocando. ${platform === 'macos' ? 'No Mac, alguns apps deixam o som “aberto” mesmo em silêncio; aí ele pode dançar sem música.' : ''}</p>
      <p class="note">Perseguindo o cursor, carregado ou dormindo, ele não dança. O humor “Dançando” faz ele dançar o tempo todo.</p></section>
    <section><h2>Atalhos</h2>${shortcutHtml}
      <p class="note">Se o atalho não responder, outro app pode estar usando a mesma combinação.</p>
      <button data-reset>Restaurar padrões</button></section>
    <section><h2>Arrastar o mouse</h2>
      <label class="check"><input type="checkbox" data-drag ${v.dragEnabled ? 'checked' : ''} /> Deixar o Quato arrastar o mouse de verdade</label>
      <p class="note">Ele pega o cursor por no máximo 10 segundos. Para soltar na hora: aperte o atalho de perseguir, mexa o mouse com força ou use “Soltar o mouse agora” no menu. Só move o cursor, nunca clica.</p>
      ${dragMsg ? `<p class="err">${esc(dragMsg)}</p>` : ''}</section>
    ${accHtml}
    ${footerHtml}`;
}

function setUpdateMsg(msg: string): void {
  updateMsg = msg;
  const el = document.getElementById('update-result');
  if (el) el.textContent = msg;
}

async function checkForUpdate(): Promise<void> {
  if (updateBusy) return;
  updateBusy = true;
  document.querySelector<HTMLButtonElement>('[data-update-check]')?.setAttribute('disabled', '');
  setUpdateMsg('Procurando…');
  try {
    const r = await api.checkUpdate();
    setUpdateMsg(
      r.status === 'available' && r.update ? `Nova versão ${r.update.version} disponível.`
        : r.status === 'upToDate' ? 'Você está na versão mais recente.'
        : 'Não consegui verificar agora.',
    );
  } catch {
    setUpdateMsg('Não consegui verificar agora.');
  }
  updateBusy = false;
  document.querySelector<HTMLButtonElement>('[data-update-check]')?.removeAttribute('disabled');
}

function waterNextText(): string {
  if (waterNext.sec === null) return '';
  const elapsed = (performance.now() - waterNext.at) / 1000;
  return nextReminderText(Math.max(0, waterNext.sec - elapsed));
}

function setView(v: SettingsView): void {
  view = v;
  waterNext = { sec: v.waterNextInSec, at: performance.now() };
  // Não apaga o que a pessoa está digitando no campo de minutos.
  if ((document.activeElement as HTMLElement | null)?.matches?.('[data-water-min]')) {
    pendingRender = true;
    return;
  }
  pendingRender = false;
  render();
  watchAccessibility();
}

async function applyWaterInterval(input: HTMLInputElement): Promise<void> {
  if (!view) return;
  const n = parseInterval(input.value);
  if (n === null) {
    waterMsg = 'Digite um número de minutos entre 1 e 720.';
    input.setAttribute('aria-invalid', 'true');
    render();
    return;
  }
  waterMsg = '';
  input.value = String(n);
  input.removeAttribute('aria-invalid');
  setView(await api.setWater(view.config.water.enabled, n));
}

// Atualiza só o texto do próximo lembrete (sem redesenhar a tela toda).
window.setInterval(() => {
  const el = document.getElementById('water-next');
  if (el) el.textContent = waterNextText();
}, 20_000);

function watchAccessibility(): void {
  window.clearInterval(pollTimer);
  if (!view?.accessibility.required || view.accessibility.trusted) return;
  pollTimer = window.setInterval(async () => {
    const s: AccessibilityStatus = await api.accessibilityStatus();
    if (s.trusted && view) {
      accessNote = 'Pronto! Se o arrasto não funcionar, saia e abra o Quato de novo.';
      setView(await api.getSettingsView());
    }
  }, 1500);
}

function scrollToSection(name: string | null): void {
  if (name === 'accessibility') {
    const el = document.getElementById('accessibility');
    el?.scrollIntoView({ behavior: 'smooth' });
    el?.classList.add('flash');
    window.setTimeout(() => el?.classList.remove('flash'), 1600);
  }
}

app.addEventListener('click', async (e) => {
  const t = (e.target as HTMLElement).closest<HTMLElement>('[data-mood],[data-color],[data-acc],[data-acc-clear],[data-home-reset],[data-record],[data-reset],[data-access],[data-water-preset],[data-water-test],[data-water-drank],[data-update-open],[data-update-check]');
  if (!t || !view) return;
  if (t.dataset.updateOpen !== undefined) await api.openUpdate();
  else if (t.dataset.updateCheck !== undefined) await checkForUpdate();
  else if (t.dataset.waterPreset) {
    waterMsg = '';
    setView(await api.setWater(view.config.water.enabled, Number(t.dataset.waterPreset)));
  } else if (t.dataset.waterTest !== undefined) await api.waterTest();
  else if (t.dataset.waterDrank !== undefined) await api.waterDrank();
  else if (t.dataset.mood) await api.setMood(t.dataset.mood as Mood);
  else if (t.dataset.color) await api.setColor(t.dataset.color);
  else if (t.dataset.acc) {
    const cur = view.config.accessories;
    const id = t.dataset.acc;
    await api.setAccessories(cur.includes(id) ? cur.filter((a) => a !== id) : [...cur, id]);
  } else if (t.dataset.accClear !== undefined) await api.setAccessories([]);
  else if (t.dataset.homeReset !== undefined) await api.resetHome();
  else if (t.dataset.reset !== undefined) {
    shortcutMsg = {};
    try { setView(await api.resetShortcuts()); } catch (err) { shortcutMsg = { toggleChase: (err as { message?: string }).message ?? String(err) }; render(); }
  } else if (t.dataset.access !== undefined) {
    await api.requestAccessibility();
  } else if (t.dataset.record) {
    shortcutMsg = {};
    await recordShortcut(t.dataset.record as ShortcutField, {
      onStart: (f) => { recordingField = f; render(); },
      onEnd: async () => { recordingField = null; setView(await api.getSettingsView()); },
      onError: (f, m) => { shortcutMsg = { ...shortcutMsg, [f]: m }; render(); },
      current: () => view!.config.shortcuts,
      platform: () => view!.platform,
    });
  }
});

app.addEventListener('change', async (e) => {
  const t = e.target as HTMLInputElement;
  if (t.matches('[data-roam]')) {
    await api.setFreeRoam(t.checked);
    return;
  }
  if (t.matches('[data-water]')) {
    setView(await api.setWater(t.checked, view?.config.water.intervalMin ?? 45));
    return;
  }
  if (t.matches('[data-water-min]')) {
    await applyWaterInterval(t);
    return;
  }
  if (t.matches('[data-seasonal]')) {
    await api.setSeasonalAccessories(t.checked);
    return;
  }
  if (t.matches('[data-dance-music]')) {
    await api.setDanceOnMusic(t.checked);
    return;
  }
  if (!t.matches('[data-drag]')) return;
  dragMsg = '';
  try {
    await api.setDragEnabled(t.checked);
  } catch (err) {
    dragMsg = (err as DragError).message ?? String(err);
    t.checked = false;
    render();
  }
});

async function start(): Promise<void> {
  setView(await api.getSettingsView());
  await on<SettingsView>(EV.settingsChanged, (v) => { if (!recordingField) setView(v); else view = v; });
  await on<string>(EV.settingsGoto, (s) => scrollToSection(s));
  scrollToSection(new URLSearchParams(location.search).get('section'));
}

app.addEventListener('focusout', (e) => {
  if ((e.target as HTMLElement).matches('[data-water-min]') && pendingRender) {
    // Espera o 'change' (que vem logo depois) antes de redesenhar.
    window.setTimeout(() => { if (pendingRender && view) setView(view); }, 0);
  }
});

app.addEventListener('keydown', (e) => {
  const t = e.target as HTMLInputElement;
  if (e.key === 'Enter' && t.matches('[data-water-min]')) t.blur();
});

start().catch((e) => { app.textContent = `Erro ao carregar: ${String(e)}`; });
