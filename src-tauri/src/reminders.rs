//! Lembretes (postura, alongar, descansar a vista) e modo foco (Pomodoro).
//!
//! O lembrete de água continua em `water.rs` (mesma config, mesmos comandos); aqui entram os
//! outros três, que usam o mesmo agendador puro (`WaterTimer`), e o foco.
//!
//! Toda a lógica de tempo está em `Engine`/`FocusTimer`: funções puras que recebem o relógio
//! (segundos num relógio qualquer) e por isso são testáveis. O relógio de verdade é
//! `water::clock` (`Instant`: não anda enquanto o computador dorme). `spawn` liga tudo ao
//! relógio e emite `duck:sign`.
use crate::events::*;
use crate::state::AppState;
use crate::water::{clock, WaterTimer};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tauri::menu::{CheckMenuItem, IsMenuItem, MenuItem, Submenu};
use tauri::{AppHandle, Emitter, Manager, Wry};

/// Quanto tempo as plaquinhas dos lembretes novos ficam na tela.
pub const REMINDER_SIGN_MS: u32 = 15_000;
/// Plaquinhas do foco (início e pausa).
pub const FOCUS_SIGN_MS: u32 = 6_000;
/// A plaquinha só tem 2 linhas de ~13 letras: texto maior seria cortado.
pub const MAX_TEXT_CHARS: usize = 28;

pub const FOCUS_MIN_DEFAULT: u32 = 25;
pub const BREAK_MIN_DEFAULT: u32 = 5;
pub const FOCUS_MIN_MAX: u32 = 180;
pub const BREAK_MIN_MAX: u32 = 60;
pub const CYCLES_DEFAULT: u32 = 1;
pub const CYCLES_MAX: u32 = 12;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ReminderKind {
    Posture,
    Stretch,
    /// Regra 20-20-20: a cada 20 min, olhar 20 s para algo a 6 m.
    Eyes,
}

impl ReminderKind {
    pub const ALL: [ReminderKind; 3] = [ReminderKind::Posture, ReminderKind::Stretch, ReminderKind::Eyes];

    fn index(self) -> usize {
        match self {
            ReminderKind::Posture => 0,
            ReminderKind::Stretch => 1,
            ReminderKind::Eyes => 2,
        }
    }

    pub fn id(self) -> &'static str {
        match self {
            ReminderKind::Posture => "posture",
            ReminderKind::Stretch => "stretch",
            ReminderKind::Eyes => "eyes",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            ReminderKind::Posture => "Lembrete de postura",
            ReminderKind::Stretch => "Lembrete de alongar",
            ReminderKind::Eyes => "Descansar a vista (20-20-20)",
        }
    }

    pub fn default_text(self) -> &'static str {
        match self {
            ReminderKind::Posture => "Ajeite a postura!",
            ReminderKind::Stretch => "Hora de alongar!",
            ReminderKind::Eyes => "Olhe a 6 m por 20 s",
        }
    }

    pub fn default_interval(self) -> u32 {
        match self {
            ReminderKind::Posture => 30,
            ReminderKind::Stretch => 60,
            ReminderKind::Eyes => 20,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReminderConfig {
    pub enabled: bool,
    /// Minutos entre um lembrete e outro (1..=720).
    pub interval_min: u32,
    /// Texto da plaquinha (até `MAX_TEXT_CHARS` letras).
    pub text: String,
}

impl ReminderConfig {
    fn default_for(kind: ReminderKind) -> Self {
        Self { enabled: false, interval_min: kind.default_interval(), text: kind.default_text().into() }
    }

    fn sanitized(mut self, kind: ReminderKind) -> Self {
        self.interval_min = clamp_water_interval(self.interval_min);
        self.text = sanitize_text(&self.text, kind.default_text());
        self
    }
}

/// Os três lembretes novos. Campos que faltam na config salva ficam no padrão.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", from = "RawReminders")]
pub struct RemindersConfig {
    pub posture: ReminderConfig,
    pub stretch: ReminderConfig,
    pub eyes: ReminderConfig,
}

/// Leitura tolerante: cada campo que faltar assume o padrão DAQUELE lembrete.
#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
struct RawReminder {
    enabled: Option<bool>,
    interval_min: Option<u32>,
    text: Option<String>,
}

#[derive(Deserialize, Default)]
#[serde(default)]
struct RawReminders {
    posture: RawReminder,
    stretch: RawReminder,
    eyes: RawReminder,
}

impl RawReminder {
    fn into_config(self, kind: ReminderKind) -> ReminderConfig {
        let d = ReminderConfig::default_for(kind);
        ReminderConfig {
            enabled: self.enabled.unwrap_or(d.enabled),
            interval_min: self.interval_min.unwrap_or(d.interval_min),
            text: self.text.unwrap_or(d.text),
        }
    }
}

impl From<RawReminders> for RemindersConfig {
    fn from(r: RawReminders) -> Self {
        Self {
            posture: r.posture.into_config(ReminderKind::Posture),
            stretch: r.stretch.into_config(ReminderKind::Stretch),
            eyes: r.eyes.into_config(ReminderKind::Eyes),
        }
    }
}

impl Default for RemindersConfig {
    fn default() -> Self {
        Self {
            posture: ReminderConfig::default_for(ReminderKind::Posture),
            stretch: ReminderConfig::default_for(ReminderKind::Stretch),
            eyes: ReminderConfig::default_for(ReminderKind::Eyes),
        }
    }
}

impl RemindersConfig {
    pub fn get(&self, kind: ReminderKind) -> &ReminderConfig {
        match kind {
            ReminderKind::Posture => &self.posture,
            ReminderKind::Stretch => &self.stretch,
            ReminderKind::Eyes => &self.eyes,
        }
    }

    fn get_mut(&mut self, kind: ReminderKind) -> &mut ReminderConfig {
        match kind {
            ReminderKind::Posture => &mut self.posture,
            ReminderKind::Stretch => &mut self.stretch,
            ReminderKind::Eyes => &mut self.eyes,
        }
    }

    pub fn sanitized(self) -> Self {
        Self {
            posture: self.posture.sanitized(ReminderKind::Posture),
            stretch: self.stretch.sanitized(ReminderKind::Stretch),
            eyes: self.eyes.sanitized(ReminderKind::Eyes),
        }
    }
}

/// Durações do modo foco, em minutos.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct FocusConfig {
    pub focus_min: u32,
    pub break_min: u32,
    /// Quantos ciclos foco -> pausa seguidos (1 a 12; 1 = um só, como sempre foi).
    pub cycles: u32,
}

impl Default for FocusConfig {
    fn default() -> Self {
        Self { focus_min: FOCUS_MIN_DEFAULT, break_min: BREAK_MIN_DEFAULT, cycles: CYCLES_DEFAULT }
    }
}

impl FocusConfig {
    pub fn sanitized(self) -> Self {
        Self { focus_min: self.focus_min.clamp(1, FOCUS_MIN_MAX), break_min: self.break_min.clamp(1, BREAK_MIN_MAX), cycles: self.cycles.clamp(1, CYCLES_MAX) }
    }
}

/// Foco em andamento como vai para a config: o fim é um instante de relógio de parede
/// (segundos desde 1970), para valer depois de fechar e reabrir o app.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FocusRun {
    pub phase: FocusPhase,
    /// Quando a fase atual termina (segundos desde 1970).
    pub ends_at: f64,
    /// Ciclos que ainda faltam, contando o atual (foco e a pausa que o segue).
    pub cycles_left: u32,
    pub cycles_total: u32,
}

/// Chamado por `config::sanitize`.
pub fn sanitize_config(reminders: RemindersConfig, focus: FocusConfig) -> (RemindersConfig, FocusConfig) {
    (reminders.sanitized(), focus.sanitized())
}

/// Tira espaços sobrando, corta em `MAX_TEXT_CHARS` letras; vazio vira o texto padrão.
pub fn sanitize_text(text: &str, default: &str) -> String {
    let clean: String = text.split_whitespace().collect::<Vec<_>>().join(" ");
    let cut: String = clean.chars().take(MAX_TEXT_CHARS).collect::<String>().trim_end().to_string();
    if cut.is_empty() {
        default.to_string()
    } else {
        cut
    }
}

// ---------------------------------------------------------------- foco (puro)

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum FocusPhase {
    Off,
    Focus,
    Break,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FocusEvent {
    /// Acabou o foco: começa a pausa.
    BreakStarted,
    /// Acabou a pausa e ainda há ciclos: começa outro foco.
    FocusStarted,
    /// Acabou a pausa: volta ao normal.
    BreakEnded,
}

#[derive(Clone, Debug, PartialEq)]
pub struct FocusTimer {
    phase: FocusPhase,
    focus_sec: f64,
    break_sec: f64,
    due: f64,
    /// Ciclos que faltam, contando o atual (foco e a pausa que o segue).
    cycles_left: u32,
    cycles_total: u32,
}

impl FocusTimer {
    pub fn new() -> Self {
        Self { phase: FocusPhase::Off, focus_sec: 0.0, break_sec: 0.0, due: 0.0, cycles_left: 0, cycles_total: 0 }
    }

    /// Ciclo atual (1-based) e total; `(0, 0)` desligado.
    pub fn cycle(&self) -> (u32, u32) {
        if self.phase == FocusPhase::Off {
            return (0, 0);
        }
        // `cycles_left` conta o ciclo atual (foco e a pausa que o segue)
        (self.cycles_total - self.cycles_left + 1, self.cycles_total)
    }

    /// Estado para guardar na config; `wall_now` é o relógio de parede (segundos desde 1970).
    pub fn snapshot(&self, now: f64, wall_now: f64) -> Option<FocusRun> {
        let left = self.remaining(now)?;
        Some(FocusRun { phase: self.phase, ends_at: wall_now + left, cycles_left: self.cycles_left, cycles_total: self.cycles_total })
    }

    /// Retoma um foco guardado. `false` (e nada muda) se já venceu ou é inválido.
    pub fn restore(&mut self, run: FocusRun, cfg: FocusConfig, now: f64, wall_now: f64) -> bool {
        let left = run.ends_at - wall_now;
        if run.phase == FocusPhase::Off || !left.is_finite() || left <= 0.0 {
            return false;
        }
        let cfg = cfg.sanitized();
        self.phase = run.phase;
        self.focus_sec = cfg.focus_min as f64 * 60.0;
        self.break_sec = cfg.break_min as f64 * 60.0;
        self.cycles_total = run.cycles_total.clamp(1, CYCLES_MAX);
        self.cycles_left = run.cycles_left.clamp(1, self.cycles_total);
        // nunca mais que a fase inteira (relógio de parede mexido para trás)
        let cap = if run.phase == FocusPhase::Focus { self.focus_sec } else { self.break_sec };
        self.due = now + left.min(cap);
        true
    }

    pub fn phase(&self) -> FocusPhase {
        self.phase
    }

    pub fn start(&mut self, cfg: FocusConfig, now: f64) {
        let cfg = cfg.sanitized();
        self.phase = FocusPhase::Focus;
        self.focus_sec = cfg.focus_min as f64 * 60.0;
        self.break_sec = cfg.break_min as f64 * 60.0;
        self.cycles_left = cfg.cycles;
        self.cycles_total = cfg.cycles;
        self.due = now + self.focus_sec;
    }

    pub fn stop(&mut self) {
        self.phase = FocusPhase::Off;
    }

    pub fn poll(&mut self, now: f64) -> Option<FocusEvent> {
        match self.phase {
            FocusPhase::Off => None,
            FocusPhase::Focus if now >= self.due => {
                self.phase = FocusPhase::Break;
                self.due = now + self.break_sec;
                Some(FocusEvent::BreakStarted)
            }
            FocusPhase::Break if now >= self.due => {
                if self.cycles_left > 1 {
                    self.cycles_left -= 1;
                    self.phase = FocusPhase::Focus;
                    self.due = now + self.focus_sec;
                    Some(FocusEvent::FocusStarted)
                } else {
                    self.phase = FocusPhase::Off;
                    Some(FocusEvent::BreakEnded)
                }
            }
            _ => None,
        }
    }

    /// Segundos até o fim da fase atual (`None` = desligado).
    pub fn remaining(&self, now: f64) -> Option<f64> {
        (self.phase != FocusPhase::Off).then(|| (self.due - now).max(0.0))
    }
}

impl Default for FocusTimer {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------- motor (puro)

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Event {
    Remind(ReminderKind),
    Focus(FocusEvent),
}

/// Os três lembretes novos mais o foco, tudo contado no mesmo relógio.
#[derive(Clone, Debug)]
pub struct Engine {
    timers: [WaterTimer; 3],
    pub focus: FocusTimer,
}

impl Engine {
    pub fn new(cfg: &RemindersConfig, now: f64) -> Self {
        let t = |k: ReminderKind| {
            let c = cfg.get(k);
            WaterTimer::new(c.enabled, c.interval_min, now)
        };
        Self { timers: [t(ReminderKind::Posture), t(ReminderKind::Stretch), t(ReminderKind::Eyes)], focus: FocusTimer::new() }
    }

    /// Mudou a config de um lembrete: a contagem recomeça.
    pub fn configure(&mut self, kind: ReminderKind, enabled: bool, interval_min: u32, now: f64) {
        self.timers[kind.index()].configure(enabled, interval_min, now);
    }

    /// "Já fiz": recomeça a contagem desse lembrete.
    pub fn done(&mut self, kind: ReminderKind, now: f64) {
        self.timers[kind.index()].drank(now);
    }

    /// Durante o foco os lembretes ficam suspensos (inclusive o de água).
    pub fn suppressing(&self) -> bool {
        self.focus.phase() == FocusPhase::Focus
    }

    /// Avança o relógio e devolve o que aconteceu. No foco, a contagem dos lembretes fica
    /// recomeçando: quando o foco acaba, todos contam do zero.
    pub fn tick(&mut self, now: f64) -> Vec<Event> {
        let mut out = vec![];
        // Quem manda é a fase de antes: no instante em que o foco acaba a contagem recomeça.
        let suppressed = self.suppressing();
        if let Some(e) = self.focus.poll(now) {
            out.push(Event::Focus(e));
        }
        for kind in ReminderKind::ALL {
            let t = &mut self.timers[kind.index()];
            if suppressed {
                t.drank(now);
            } else if t.poll(now) {
                out.push(Event::Remind(kind));
            }
        }
        out
    }

    /// Segundos até o próximo lembrete (`None` = desligado).
    pub fn next_in(&self, kind: ReminderKind, now: f64) -> Option<u32> {
        self.timers[kind.index()].remaining(now).map(|r| r.ceil() as u32)
    }

    pub fn status(&self, now: f64) -> RemindersStatus {
        RemindersStatus {
            posture_next_in_sec: self.next_in(ReminderKind::Posture, now),
            stretch_next_in_sec: self.next_in(ReminderKind::Stretch, now),
            eyes_next_in_sec: self.next_in(ReminderKind::Eyes, now),
            focus_phase: self.focus.phase(),
            focus_remaining_sec: self.focus.remaining(now).map(|r| r.ceil() as u32),
            focus_cycle: self.focus.cycle().0,
            focus_cycles_total: self.focus.cycle().1,
        }
    }
}

/// Parte do `SettingsView`: contagens regressivas e fase do foco.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RemindersStatus {
    pub posture_next_in_sec: Option<u32>,
    pub stretch_next_in_sec: Option<u32>,
    pub eyes_next_in_sec: Option<u32>,
    pub focus_phase: FocusPhase,
    pub focus_remaining_sec: Option<u32>,
    /// Ciclo atual (1-based) e total; 0 com o foco desligado.
    pub focus_cycle: u32,
    pub focus_cycles_total: u32,
}

// ---------------------------------------------------------------- cola com o app

fn sign(app: &AppHandle, text: &str, duration_ms: u32) {
    let _ = app.emit(EV_DUCK_SIGN, DuckSign { kind: SignKind::Water, text: text.into(), duration_ms });
}

fn engine<R>(app: &AppHandle, f: impl FnOnce(&mut Engine) -> R) -> R {
    f(&mut app.state::<AppState>().reminders.lock().unwrap())
}

pub fn status(app: &AppHandle) -> RemindersStatus {
    let now = clock();
    engine(app, |e| e.status(now))
}

/// O foco está ligado? (usado pelo lembrete de água para ficar quieto)
pub fn suppressing(app: &AppHandle) -> bool {
    engine(app, |e| e.suppressing())
}

fn text_of(app: &AppHandle, kind: ReminderKind) -> String {
    app.state::<AppState>().config.lock().unwrap().reminders.get(kind).text.clone()
}

/// Liga/desliga, muda o intervalo e o texto de um lembrete (tudo validado aqui). Salva na config.
pub fn set(app: &AppHandle, kind: ReminderKind, enabled: bool, interval_min: u32, text: &str) {
    let interval_min = clamp_water_interval(interval_min);
    {
        let st = app.state::<AppState>();
        let mut cfg = st.config.lock().unwrap();
        *cfg.reminders.get_mut(kind) = ReminderConfig { enabled, interval_min, text: sanitize_text(text, kind.default_text()) };
        crate::config::save(app, &cfg);
    }
    engine(app, |e| e.configure(kind, enabled, interval_min, clock()));
    if !enabled {
        sign(app, "", 0);
    }
    crate::actions::emit_settings(app);
    crate::tray::sync(app);
}

pub fn toggle(app: &AppHandle, kind: ReminderKind) {
    let c = app.state::<AppState>().config.lock().unwrap().reminders.get(kind).clone();
    set(app, kind, !c.enabled, c.interval_min, &c.text);
}

/// "Já fiz": esconde a plaquinha e recomeça a contagem.
pub fn done(app: &AppHandle, kind: ReminderKind) {
    engine(app, |e| e.done(kind, clock()));
    sign(app, "", 0);
    crate::actions::emit_settings(app);
}

/// Mostra a plaquinha agora (botão "Testar plaquinha").
pub fn show_now(app: &AppHandle, kind: ReminderKind) {
    sign(app, &text_of(app, kind), REMINDER_SIGN_MS);
}

pub fn set_focus_config(app: &AppHandle, focus_min: u32, break_min: u32, cycles: u32) {
    let f = FocusConfig { focus_min, break_min, cycles }.sanitized();
    {
        let st = app.state::<AppState>();
        let mut cfg = st.config.lock().unwrap();
        cfg.focus = f;
        crate::config::save(app, &cfg);
    }
    crate::actions::emit_settings(app);
    crate::tray::sync(app);
}

/// Relógio de parede: segundos desde 1970.
fn wall_clock() -> f64 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs_f64()).unwrap_or(0.0)
}

/// Guarda (ou apaga) o foco em andamento na config.
fn persist_focus(app: &AppHandle) {
    let run = engine(app, |e| e.focus.snapshot(clock(), wall_clock()));
    let st = app.state::<AppState>();
    let mut cfg = st.config.lock().unwrap();
    if cfg.focus_run != run {
        cfg.focus_run = run;
        crate::config::save(app, &cfg);
    }
}

/// Ao abrir o app: retoma o foco guardado se ainda não venceu; se venceu, descarta.
fn resume_focus(app: &AppHandle) {
    let (run, fcfg) = {
        let st = app.state::<AppState>();
        let cfg = st.config.lock().unwrap();
        (cfg.focus_run, cfg.focus)
    };
    let Some(run) = run else { return };
    let ok = engine(app, |e| e.focus.restore(run, fcfg, clock(), wall_clock()));
    if ok && run.phase == FocusPhase::Focus {
        crate::actions::send(app, crate::sim::model::SimCommand::SetFocus(true));
    }
    persist_focus(app);
    crate::actions::emit_settings(app);
    crate::tray::sync(app);
}

fn focus_sign_text(app: &AppHandle) -> String {
    let (cur, total) = engine(app, |e| e.focus.cycle());
    let min = focus_cfg(app).focus_min;
    if total > 1 {
        format!("Foco {cur}/{total}: {min} min")
    } else {
        format!("Foco: {min} min")
    }
}

fn focus_cfg(app: &AppHandle) -> FocusConfig {
    app.state::<AppState>().config.lock().unwrap().focus
}

pub fn start_focus(app: &AppHandle) {
    let cfg = focus_cfg(app);
    engine(app, |e| e.focus.start(cfg, clock()));
    crate::actions::send(app, crate::sim::model::SimCommand::SetFocus(true));
    sign(app, "", 0);
    sign(app, &focus_sign_text(app), FOCUS_SIGN_MS);
    persist_focus(app);
    crate::actions::emit_settings(app);
    crate::tray::sync(app);
}

pub fn stop_focus(app: &AppHandle) {
    engine(app, |e| e.focus.stop());
    crate::actions::send(app, crate::sim::model::SimCommand::SetFocus(false));
    sign(app, "", 0);
    persist_focus(app);
    crate::actions::emit_settings(app);
    crate::tray::sync(app);
}

pub fn toggle_focus(app: &AppHandle) {
    if engine(app, |e| e.focus.phase()) == FocusPhase::Off {
        start_focus(app);
    } else {
        stop_focus(app);
    }
}

fn handle(app: &AppHandle, ev: Event) {
    match ev {
        Event::Remind(kind) => {
            sign(app, &text_of(app, kind), REMINDER_SIGN_MS);
            crate::actions::emit_settings(app);
        }
        Event::Focus(FocusEvent::BreakStarted) => {
            crate::actions::send(app, crate::sim::model::SimCommand::SetFocus(false));
            sign(app, &format!("Pausa: {} min", focus_cfg(app).break_min), FOCUS_SIGN_MS);
            persist_focus(app);
            crate::actions::emit_settings(app);
            crate::tray::sync(app);
        }
        Event::Focus(FocusEvent::FocusStarted) => {
            crate::actions::send(app, crate::sim::model::SimCommand::SetFocus(true));
            sign(app, &focus_sign_text(app), FOCUS_SIGN_MS);
            persist_focus(app);
            crate::actions::emit_settings(app);
            crate::tray::sync(app);
        }
        Event::Focus(FocusEvent::BreakEnded) => {
            persist_focus(app);
            crate::actions::emit_settings(app);
            crate::tray::sync(app);
        }
    }
}

pub fn spawn(app: AppHandle) {
    std::thread::Builder::new()
        .name("quato-reminders".into())
        .spawn(move || {
            resume_focus(&app);
            loop {
                let now = clock();
                let events = engine(&app, |e| e.tick(now));
                for ev in events {
                    handle(&app, ev);
                }
                std::thread::sleep(Duration::from_secs(1));
            }
        })
        .expect("não consegui iniciar os lembretes");
}

// ---------------------------------------------------------------- bandeja

/// Itens do menu da bandeja que são só dos lembretes novos e do foco.
pub struct TrayItems {
    pub checks: Vec<(ReminderKind, CheckMenuItem<Wry>)>,
    pub focus: MenuItem<Wry>,
}

pub fn tray_items(app: &AppHandle) -> tauri::Result<(Submenu<Wry>, MenuItem<Wry>)> {
    let mut checks: Vec<(ReminderKind, CheckMenuItem<Wry>)> = vec![];
    for k in ReminderKind::ALL {
        checks.push((k, CheckMenuItem::with_id(app, format!("rem:{}", k.id()), k.label(), true, false, None::<&str>)?));
    }
    let refs: Vec<&dyn IsMenuItem<Wry>> = checks.iter().map(|(_, i)| i as &dyn IsMenuItem<Wry>).collect();
    let sub = Submenu::with_items(app, "Mais lembretes", true, &refs)?;
    let focus = MenuItem::with_id(app, "focus", focus_label(FocusPhase::Off, FocusConfig::default()), true, None::<&str>)?;
    *app.state::<AppState>().rem_tray.lock().unwrap() = Some(TrayItems { checks, focus: focus.clone() });
    Ok((sub, focus))
}

fn focus_label(phase: FocusPhase, cfg: FocusConfig) -> String {
    match phase {
        FocusPhase::Off if cfg.cycles > 1 => format!("Iniciar foco ({} min × {})", cfg.focus_min, cfg.cycles),
        FocusPhase::Off => format!("Iniciar foco ({} min)", cfg.focus_min),
        FocusPhase::Focus => "Parar foco".into(),
        FocusPhase::Break => "Pular a pausa".into(),
    }
}

/// Chamado por `tray::sync`: atualiza checks e rótulo do foco. Não segura lock durante a main thread.
pub fn sync_tray(app: &AppHandle) {
    let st = app.state::<AppState>();
    let (rem, fcfg) = {
        let c = st.config.lock().unwrap();
        (c.reminders.clone(), c.focus)
    };
    let phase = engine(app, |e| e.focus.phase());
    let items = st.rem_tray.lock().unwrap().as_ref().map(|t| (t.checks.clone(), t.focus.clone()));
    let Some((checks, focus)) = items else { return };
    let _ = app.run_on_main_thread(move || {
        for (k, item) in &checks {
            let _ = item.set_checked(rem.get(*k).enabled);
        }
        let _ = focus.set_text(focus_label(phase, fcfg));
    });
}

/// Clique no menu: `true` se o id era daqui.
pub fn on_menu(app: &AppHandle, id: &str) -> bool {
    if id == "focus" {
        toggle_focus(app);
        return true;
    }
    if let Some(name) = id.strip_prefix("rem:") {
        if let Some(k) = ReminderKind::ALL.into_iter().find(|k| k.id() == name) {
            toggle(app, k);
        }
        return true;
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    fn on(min: u32) -> ReminderConfig {
        ReminderConfig { enabled: true, interval_min: min, text: "x".into() }
    }

    fn cfg_all_on() -> RemindersConfig {
        RemindersConfig { posture: on(30), stretch: on(60), eyes: on(20) }
    }

    fn reminds(events: &[Event]) -> Vec<ReminderKind> {
        events.iter().filter_map(|e| if let Event::Remind(k) = e { Some(*k) } else { None }).collect()
    }

    #[test]
    fn each_reminder_fires_on_its_own_interval() {
        let mut e = Engine::new(&cfg_all_on(), 0.0);
        assert!(e.tick(19.0 * 60.0).is_empty());
        assert_eq!(reminds(&e.tick(20.0 * 60.0)), vec![ReminderKind::Eyes]);
        assert_eq!(reminds(&e.tick(30.0 * 60.0)), vec![ReminderKind::Posture]);
        let at60 = reminds(&e.tick(60.0 * 60.0));
        assert!(at60.contains(&ReminderKind::Stretch) && at60.contains(&ReminderKind::Eyes) && at60.contains(&ReminderKind::Posture));
        // não repete logo em seguida
        assert!(e.tick(60.0 * 60.0 + 1.0).is_empty());
    }

    #[test]
    fn disabled_reminders_never_fire() {
        let mut e = Engine::new(&RemindersConfig::default(), 0.0);
        assert!(e.tick(1e7).is_empty());
        assert_eq!(e.next_in(ReminderKind::Eyes, 0.0), None);
    }

    #[test]
    fn configure_and_done_restart_the_count() {
        let mut e = Engine::new(&cfg_all_on(), 0.0);
        e.configure(ReminderKind::Eyes, true, 10, 100.0);
        assert!(e.tick(699.0).is_empty());
        assert_eq!(reminds(&e.tick(700.0)), vec![ReminderKind::Eyes]);
        e.done(ReminderKind::Posture, 1000.0);
        assert_eq!(e.next_in(ReminderKind::Posture, 1000.0), Some(1800));
        e.configure(ReminderKind::Stretch, false, 60, 1000.0);
        assert_eq!(e.next_in(ReminderKind::Stretch, 1000.0), None);
    }

    #[test]
    fn focus_runs_focus_then_break_then_off() {
        let mut f = FocusTimer::new();
        assert_eq!(f.poll(1e6), None);
        f.start(FocusConfig { focus_min: 25, break_min: 5, cycles: 1 }, 100.0);
        assert_eq!(f.phase(), FocusPhase::Focus);
        assert_eq!(f.remaining(100.0), Some(1500.0));
        assert_eq!(f.poll(100.0 + 1499.0), None);
        assert_eq!(f.poll(100.0 + 1500.0), Some(FocusEvent::BreakStarted));
        assert_eq!(f.phase(), FocusPhase::Break);
        assert_eq!(f.remaining(1600.0), Some(300.0));
        assert_eq!(f.poll(1600.0 + 299.0), None);
        assert_eq!(f.poll(1600.0 + 300.0), Some(FocusEvent::BreakEnded));
        assert_eq!(f.phase(), FocusPhase::Off);
        assert_eq!(f.remaining(1e6), None);
    }

    #[test]
    fn focus_can_be_stopped_and_restarted() {
        let mut f = FocusTimer::new();
        f.start(FocusConfig::default(), 0.0);
        f.stop();
        assert_eq!(f.phase(), FocusPhase::Off);
        assert_eq!(f.poll(1e6), None);
        f.start(FocusConfig { focus_min: 1, break_min: 1, cycles: 1 }, 10.0);
        assert_eq!(f.poll(70.0), Some(FocusEvent::BreakStarted));
    }

    #[test]
    fn focus_suspends_reminders_and_they_restart_after() {
        let mut e = Engine::new(&cfg_all_on(), 0.0);
        e.focus.start(FocusConfig { focus_min: 25, break_min: 5, cycles: 1 }, 600.0);
        assert!(e.suppressing());
        // o foco passa dos 20 min do "eyes", mas nada aparece
        for t in (600..2100).step_by(30) {
            assert!(reminds(&e.tick(t as f64)).is_empty(), "t={t}");
        }
        // fim do foco (2100 s): começa a pausa, e a contagem dos lembretes recomeça dali
        let ev = e.tick(2100.0);
        assert_eq!(ev, vec![Event::Focus(FocusEvent::BreakStarted)]);
        assert!(!e.suppressing());
        assert_eq!(e.next_in(ReminderKind::Eyes, 2100.0), Some(20 * 60));
        assert_eq!(reminds(&e.tick(2100.0 + 20.0 * 60.0)), vec![ReminderKind::Eyes]);
    }

    #[test]
    fn stopping_focus_resumes_reminders_from_zero() {
        let mut e = Engine::new(&cfg_all_on(), 0.0);
        e.focus.start(FocusConfig::default(), 0.0);
        e.tick(900.0);
        e.focus.stop();
        assert!(e.tick(901.0).is_empty());
        assert_eq!(e.next_in(ReminderKind::Eyes, 901.0), Some(1199));
    }

    #[test]
    fn status_reports_countdowns_and_phase() {
        let mut e = Engine::new(&cfg_all_on(), 0.0);
        e.focus.start(FocusConfig::default(), 0.0);
        let s = e.status(60.0);
        assert_eq!(s.focus_phase, FocusPhase::Focus);
        assert_eq!(s.focus_remaining_sec, Some(1440));
        assert_eq!(
            serde_json::to_string(&RemindersStatus {
                posture_next_in_sec: None,
                stretch_next_in_sec: Some(5),
                eyes_next_in_sec: None,
                focus_phase: FocusPhase::Off,
                focus_remaining_sec: None,
                focus_cycle: 0,
                focus_cycles_total: 0
            })
            .unwrap(),
            r#"{"postureNextInSec":null,"stretchNextInSec":5,"eyesNextInSec":null,"focusPhase":"off","focusRemainingSec":null,"focusCycle":0,"focusCyclesTotal":0}"#
        );
    }

    #[test]
    fn text_and_limits_are_sanitized() {
        assert_eq!(sanitize_text("  ", "padrão"), "padrão");
        assert_eq!(sanitize_text("  olá   mundo ", "padrão"), "olá mundo");
        assert_eq!(sanitize_text(&"a".repeat(100), "padrão").chars().count(), MAX_TEXT_CHARS);
        let f = FocusConfig { focus_min: 0, break_min: 9999, cycles: 1 }.sanitized();
        assert_eq!((f.focus_min, f.break_min, f.cycles), (1, 60, 1));
        assert_eq!(FocusConfig { cycles: 99, ..Default::default() }.sanitized().cycles, 12);
        assert_eq!(FocusConfig { cycles: 0, ..Default::default() }.sanitized().cycles, 1);
        let r = RemindersConfig { eyes: ReminderConfig { enabled: true, interval_min: 0, text: String::new() }, ..Default::default() }.sanitized();
        assert_eq!(r.eyes.interval_min, 1);
        assert_eq!(r.eyes.text, ReminderKind::Eyes.default_text());
    }

    #[test]
    fn old_configs_without_reminders_still_load() {
        let c = crate::config::parse(r#"{"water":{"enabled":true,"intervalMin":30},"mood":"raiva"}"#).unwrap();
        assert!(c.water.enabled);
        assert_eq!(c.reminders, RemindersConfig::default());
        assert_eq!(c.focus, FocusConfig::default());
        assert_eq!(c.reminders.eyes.interval_min, 20);
        assert_eq!(c.focus.focus_min, 25);
        // config parcial dentro de reminders completa o resto
        let c = crate::config::parse(r#"{"reminders":{"eyes":{"enabled":true}},"focus":{"breakMin":10}}"#).unwrap();
        assert!(c.reminders.eyes.enabled);
        assert_eq!(c.reminders.eyes.text, "Olhe a 6 m por 20 s");
        assert_eq!(c.reminders.posture.interval_min, 30);
        assert_eq!((c.focus.focus_min, c.focus.break_min, c.focus.cycles), (25, 10, 1));
        assert_eq!(c.focus_run, None);
    }

    fn cyc(n: u32) -> FocusConfig {
        FocusConfig { focus_min: 10, break_min: 2, cycles: n }
    }

    #[test]
    fn focus_repeats_cycles_then_ends() {
        let mut f = FocusTimer::new();
        f.start(cyc(3), 0.0);
        assert_eq!(f.cycle(), (1, 3));
        assert_eq!(f.poll(600.0), Some(FocusEvent::BreakStarted));
        assert_eq!(f.cycle(), (1, 3));
        assert_eq!(f.poll(720.0), Some(FocusEvent::FocusStarted));
        assert_eq!((f.phase(), f.cycle()), (FocusPhase::Focus, (2, 3)));
        assert_eq!(f.remaining(720.0), Some(600.0));
        assert_eq!(f.poll(1320.0), Some(FocusEvent::BreakStarted));
        assert_eq!(f.poll(1440.0), Some(FocusEvent::FocusStarted));
        assert_eq!(f.cycle(), (3, 3));
        assert_eq!(f.poll(2040.0), Some(FocusEvent::BreakStarted));
        assert_eq!(f.poll(2160.0), Some(FocusEvent::BreakEnded));
        assert_eq!((f.phase(), f.cycle()), (FocusPhase::Off, (0, 0)));
    }

    #[test]
    fn focus_keeps_reminders_suspended_across_cycles() {
        let mut e = Engine::new(&cfg_all_on(), 0.0);
        e.focus.start(cyc(2), 0.0);
        e.tick(600.0);
        assert!(!e.suppressing(), "na pausa os lembretes voltam");
        e.tick(720.0);
        assert!(e.suppressing(), "novo foco suspende de novo");
    }

    #[test]
    fn focus_run_survives_restart_by_wall_clock() {
        let mut f = FocusTimer::new();
        f.start(cyc(4), 1000.0);
        f.poll(1600.0); // pausa
        f.poll(1720.0); // foco 2
        let run = f.snapshot(1800.0, 5_000_000.0).unwrap(); // faltam 520 s do foco 2
        assert_eq!((run.phase, run.cycles_left, run.cycles_total), (FocusPhase::Focus, 3, 4));
        assert_eq!(run.ends_at, 5_000_520.0);
        // JSON da config
        let back: FocusRun = serde_json::from_str(&serde_json::to_string(&run).unwrap()).unwrap();
        assert_eq!(back, run);
        // reabre 100 s depois, com outro relógio monotônico
        let mut g = FocusTimer::new();
        assert!(g.restore(run, cyc(4), 7.0, 5_000_100.0));
        assert_eq!((g.phase(), g.cycle()), (FocusPhase::Focus, (2, 4)));
        assert_eq!(g.remaining(7.0), Some(420.0));
        assert_eq!(g.poll(7.0 + 420.0), Some(FocusEvent::BreakStarted));
    }

    #[test]
    fn expired_or_invalid_focus_run_is_discarded() {
        let run = FocusRun { phase: FocusPhase::Break, ends_at: 100.0, cycles_left: 1, cycles_total: 1 };
        let mut g = FocusTimer::new();
        assert!(!g.restore(run, cyc(1), 0.0, 100.0));
        assert!(!g.restore(run, cyc(1), 0.0, 500.0));
        assert_eq!(g.phase(), FocusPhase::Off);
        let off = FocusRun { phase: FocusPhase::Off, ends_at: 1e12, cycles_left: 1, cycles_total: 1 };
        assert!(!g.restore(off, cyc(1), 0.0, 0.0));
        assert_eq!(FocusTimer::new().snapshot(0.0, 0.0), None);
        // fim absurdamente longe (relógio mexido): limitado à fase inteira
        let far = FocusRun { phase: FocusPhase::Focus, ends_at: 1e12, cycles_left: 99, cycles_total: 99 };
        assert!(g.restore(far, cyc(1), 0.0, 0.0));
        assert_eq!(g.remaining(0.0), Some(600.0));
        assert_eq!(g.cycle().1, 12);
    }

    #[test]
    fn old_config_without_focus_run_loads_and_run_roundtrips() {
        let c = crate::config::parse(r#"{"focus":{"focusMin":30}}"#).unwrap();
        assert_eq!((c.focus.focus_min, c.focus.cycles, c.focus_run), (30, 1, None));
        let mut c = crate::events::Config::default();
        c.focus_run = Some(FocusRun { phase: FocusPhase::Break, ends_at: 123.5, cycles_left: 2, cycles_total: 3 });
        let back = crate::config::parse(&crate::config::serialize(&c)).unwrap();
        assert_eq!(back.focus_run, c.focus_run);
    }

    #[test]
    fn kind_ids_match_typescript() {
        assert_eq!(serde_json::to_string(&ReminderKind::Eyes).unwrap(), "\"eyes\"");
        for k in ReminderKind::ALL {
            assert_eq!(serde_json::to_string(&k).unwrap(), format!("\"{}\"", k.id()));
        }
    }

    #[test]
    fn default_texts_fit_the_sign() {
        for k in ReminderKind::ALL {
            assert!(k.default_text().chars().count() <= MAX_TEXT_CHARS);
        }
    }
}
