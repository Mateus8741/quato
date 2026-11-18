//! Lembrete de água: de tempos em tempos o pato mostra a plaquinha "Hora de beber água!".
//!
//! `WaterTimer` é o agendador PURO (segundos num relógio qualquer, testável);
//! `spawn` liga ele ao relógio de verdade e emite `duck:sign`.
use crate::events::*;
use crate::state::AppState;
use std::sync::OnceLock;
use std::time::{Duration, Instant};
use tauri::{AppHandle, Emitter, Manager};

#[derive(Clone, Debug, PartialEq)]
pub struct WaterTimer {
    enabled: bool,
    interval: f64,
    due: f64,
}

impl WaterTimer {
    pub fn new(enabled: bool, interval_min: u32, now: f64) -> Self {
        let mut t = Self { enabled: false, interval: 0.0, due: 0.0 };
        t.configure(enabled, interval_min, now);
        t
    }

    /// Mudou a config: a contagem recomeça do zero.
    pub fn configure(&mut self, enabled: bool, interval_min: u32, now: f64) {
        self.enabled = enabled;
        self.interval = clamp_water_interval(interval_min) as f64 * 60.0;
        self.due = now + self.interval;
    }

    /// A pessoa bebeu água: recomeça a contagem.
    pub fn drank(&mut self, now: f64) {
        self.due = now + self.interval;
    }

    /// `true` quando é hora de mostrar a plaquinha (e já agenda o próximo).
    pub fn poll(&mut self, now: f64) -> bool {
        if !self.enabled || now < self.due {
            return false;
        }
        self.due = now + self.interval;
        true
    }

    /// Segundos até o próximo lembrete (`None` = desligado).
    pub fn remaining(&self, now: f64) -> Option<f64> {
        self.enabled.then(|| (self.due - now).max(0.0))
    }
}

/// Relógio do lembrete: segundos desde que o app abriu.
pub fn clock() -> f64 {
    static START: OnceLock<Instant> = OnceLock::new();
    START.get_or_init(Instant::now).elapsed().as_secs_f64()
}

fn sign(app: &AppHandle, duration_ms: u32) {
    crate::speech::note_sign(duration_ms);
    let _ = app.emit(EV_DUCK_SIGN, DuckSign { kind: SignKind::Water, text: WATER_TEXT.into(), duration_ms });
}

pub fn next_in_sec(app: &AppHandle) -> Option<u32> {
    let now = clock();
    app.state::<AppState>().water.lock().unwrap().remaining(now).map(|r| r.ceil() as u32)
}

/// Liga/desliga e muda o intervalo (já validado aqui). Salva na config.
pub fn set(app: &AppHandle, enabled: bool, interval_min: u32) {
    let interval_min = clamp_water_interval(interval_min);
    {
        let st = app.state::<AppState>();
        let mut cfg = st.config.lock().unwrap();
        cfg.water = WaterConfig { enabled, interval_min };
        crate::config::save(app, &cfg);
    }
    app.state::<AppState>().water.lock().unwrap().configure(enabled, interval_min, clock());
    if !enabled {
        sign(app, 0);
    }
    crate::actions::emit_settings(app);
    crate::tray::sync(app);
}

pub fn toggle(app: &AppHandle) {
    let w = app.state::<AppState>().config.lock().unwrap().water;
    set(app, !w.enabled, w.interval_min);
}

/// "Já bebi água": esconde a plaquinha e recomeça a contagem.
pub fn drank(app: &AppHandle) {
    app.state::<AppState>().water.lock().unwrap().drank(clock());
    sign(app, 0);
    crate::actions::emit_settings(app);
}

/// Mostra a plaquinha agora (botão "Testar plaquinha").
pub fn show_now(app: &AppHandle) {
    sign(app, WATER_SIGN_MS);
}

pub fn spawn(app: AppHandle) {
    std::thread::Builder::new()
        .name("quato-water".into())
        .spawn(move || loop {
            let now = clock();
            let (fire, wait) = {
                let st = app.state::<AppState>();
                let mut t = st.water.lock().unwrap();
                let fire = t.poll(now);
                (fire, t.remaining(now).unwrap_or(1.0).clamp(0.05, 1.0))
            };
            if fire {
                sign(&app, WATER_SIGN_MS);
                crate::actions::emit_settings(&app);
            }
            std::thread::sleep(Duration::from_secs_f64(wait));
        })
        .expect("não consegui iniciar o lembrete de água");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fires_after_interval_and_reschedules() {
        let mut t = WaterTimer::new(true, 45, 0.0);
        assert!(!t.poll(0.0));
        assert!(!t.poll(45.0 * 60.0 - 1.0));
        assert!(t.poll(45.0 * 60.0));
        assert!(!t.poll(45.0 * 60.0 + 1.0), "não repete logo em seguida");
        assert!(t.poll(90.0 * 60.0));
    }

    #[test]
    fn disabled_never_fires() {
        let mut t = WaterTimer::new(false, 1, 0.0);
        assert!(!t.poll(1e6));
        assert_eq!(t.remaining(10.0), None);
    }

    #[test]
    fn drinking_restarts_the_count() {
        let mut t = WaterTimer::new(true, 10, 0.0);
        t.drank(500.0);
        assert!(!t.poll(600.0));
        assert!(!t.poll(1099.0));
        assert!(t.poll(1100.0));
    }

    #[test]
    fn changing_config_restarts_the_count() {
        let mut t = WaterTimer::new(true, 10, 0.0);
        t.configure(true, 30, 550.0);
        assert!(!t.poll(600.0));
        assert_eq!(t.remaining(600.0), Some(1750.0));
        assert!(t.poll(550.0 + 1800.0));
        // desligar e religar recomeça do zero
        t.configure(false, 30, 3000.0);
        assert!(!t.poll(9000.0));
        t.configure(true, 30, 9000.0);
        assert!(!t.poll(9001.0));
        assert!(t.poll(9000.0 + 1800.0));
    }

    #[test]
    fn interval_is_clamped() {
        let t = WaterTimer::new(true, 0, 0.0);
        assert_eq!(t.remaining(0.0), Some(60.0));
        let t = WaterTimer::new(true, 100_000, 0.0);
        assert_eq!(t.remaining(0.0), Some(720.0 * 60.0));
    }

    #[test]
    fn sign_payload_matches_typescript() {
        let s = DuckSign { kind: SignKind::Water, text: WATER_TEXT.into(), duration_ms: 15000 };
        assert_eq!(serde_json::to_string(&s).unwrap(), r#"{"kind":"water","text":"Hora de beber água!","durationMs":15000}"#);
        let g = DuckGesture { kind: GestureKind::Feather, duration_ms: 2500 };
        assert_eq!(serde_json::to_string(&g).unwrap(), r#"{"kind":"feather","durationMs":2500}"#);
    }
}
