//! Overlay `fx`: janela transparente, sem cliques, que cobre o monitor onde o pato está e só
//! desenha os efeitos da arminha (rastro, marcas de tiro, estojos). Criada sob demanda,
//! escondida quando não há mais nada para mostrar e fechada depois de um tempo sem uso.
//! Rust converte tudo para pixels CSS relativos ao canto da janela; o JS só desenha.
use crate::fire::Shot;
use crate::platform::{self, MonitorSim};
use crate::sim::vec2::{Rect, Vec2};
use serde::Serialize;
use std::sync::atomic::{AtomicBool, Ordering};
use tauri::{AppHandle, Emitter, Listener, Manager};

pub const EV_FX_SHOT: &str = "fx:shot";
pub const EV_FX_CLEAR: &str = "fx:clear";
/// O JS avisa que já escuta os eventos.
pub const EV_FX_READY: &str = "fx:ready";
pub const LABEL: &str = "fx";

/// As marcas somem em ~20 s e os estojos em poucos segundos: depois disso a janela se esconde.
const VISIBLE_SECS: f64 = 22.0;
/// Sem ninguém pedindo (sem arminha ou sem raiva) por tanto tempo: fecha a janela de vez.
const CLOSE_AFTER_SECS: f64 = 90.0;
const MAX_PENDING: usize = 6;

static READY: AtomicBool = AtomicBool::new(false);

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub struct P {
    pub x: f32,
    pub y: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FxShot {
    pub from: P,
    pub to: P,
    pub eject: P,
    /// +1 / -1: lado dos estojos.
    pub eject_dir: f32,
    /// y do chão (pés do pato), em px CSS da janela fx.
    pub floor: f32,
}

/// Espaço da simulação -> pixels CSS relativos ao canto da janela fx.
pub fn to_css(p: Vec2, origin: Vec2, css_per_sim: f32) -> P {
    P { x: (p.x - origin.x) * css_per_sim, y: (p.y - origin.y) * css_per_sim }
}

pub fn shot_to_css(s: &Shot, area: Rect, css_per_sim: f32) -> FxShot {
    let o = Vec2::new(area.x, area.y);
    FxShot { from: to_css(s.muzzle, o, css_per_sim), to: to_css(s.hit, o, css_per_sim), eject: to_css(s.eject, o, css_per_sim), eject_dir: s.eject_dir, floor: (s.floor_y - area.y) * css_per_sim }
}

pub struct Fx {
    failed: bool,
    area: Option<Rect>,
    shown: bool,
    visible_until: f64,
    last_wanted: f64,
    listening: bool,
    pending: Vec<FxShot>,
}

impl Fx {
    pub fn new() -> Self {
        Self { failed: false, area: None, shown: false, visible_until: 0.0, last_wanted: 0.0, listening: false, pending: vec![] }
    }

    /// Tem janela, algo escondido para limpar ou envio pendente: vale continuar olhando a cada tick.
    pub fn busy(&self, app: &AppHandle) -> bool {
        !self.failed && (self.shown || app.get_webview_window(LABEL).is_some())
    }

    /// Chamado enquanto o pato está apto a atirar: cria a janela (escondida) se ainda não existe.
    pub fn want(&mut self, app: &AppHandle, now: f64) {
        if self.failed {
            return;
        }
        self.last_wanted = now;
        if app.get_webview_window(LABEL).is_some() {
            return;
        }
        if !self.listening {
            self.listening = true;
            app.listen_any(EV_FX_READY, |_| READY.store(true, Ordering::SeqCst));
        }
        READY.store(false, Ordering::SeqCst);
        if let Err(e) = crate::windows::create_fx(app) {
            eprintln!("[quato] não consegui criar a janela dos tiros ({e}); a arminha fica só de enfeite");
            self.failed = true;
        }
    }

    /// Manda um tiro. `mon` é o monitor onde o pato está (a janela cobre ele inteiro).
    pub fn shoot(&mut self, app: &AppHandle, now: f64, mon: &MonitorSim, shot: &Shot) {
        let Some(win) = app.get_webview_window(LABEL) else { return };
        let area = mon.rect;
        let k = platform::css_per_sim(mon);
        let payload = shot_to_css(shot, area, k);
        if !READY.load(Ordering::SeqCst) {
            // A página ainda está carregando: guarda uns poucos tiros (com a área atual).
            self.area = Some(area);
            if self.pending.len() < MAX_PENDING {
                self.pending.push(payload);
            }
            return;
        }
        if self.area != Some(area) {
            // Mudou de monitor: limpa o que sobrou do outro e reposiciona.
            if self.area.is_some() {
                let _ = app.emit_to(LABEL, EV_FX_CLEAR, ());
            }
            platform::cover_rect(&win, area);
            self.area = Some(area);
            // O tiro foi calculado para o monitor novo, mas os pendentes eram da área antiga.
            self.pending.clear();
        }
        if !self.shown {
            platform::cover_rect(&win, area);
            let _ = win.set_ignore_cursor_events(true);
            let _ = win.show();
            self.shown = true;
        }
        for p in self.pending.drain(..) {
            let _ = app.emit_to(LABEL, EV_FX_SHOT, p);
        }
        let _ = app.emit_to(LABEL, EV_FX_SHOT, payload);
        self.visible_until = now + VISIBLE_SECS;
    }

    /// Todo tick: esconde a janela quando não sobrou nada e fecha se ficou muito tempo sem uso.
    pub fn housekeeping(&mut self, app: &AppHandle, now: f64) {
        let Some(win) = app.get_webview_window(LABEL) else {
            self.shown = false;
            return;
        };
        if self.shown && now >= self.visible_until {
            let _ = win.hide();
            let _ = app.emit_to(LABEL, EV_FX_CLEAR, ());
            self.shown = false;
            self.area = None;
        }
        if !self.shown && now - self.last_wanted > CLOSE_AFTER_SECS {
            let _ = win.close();
            READY.store(false, Ordering::SeqCst);
            self.pending.clear();
            self.area = None;
        }
    }

    /// Para o tick saber se precisa acordar de novo por causa do overlay.
    pub fn next_check(&self, now: f64) -> Option<f64> {
        if self.shown {
            Some((self.visible_until - now).max(0.05))
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn css_is_relative_to_the_window_corner() {
        let area = Rect::new(-1000.0, 100.0, 800.0, 600.0);
        let shot = Shot { muzzle: Vec2::new(-900.0, 300.0), hit: Vec2::new(-500.0, 400.0), eject: Vec2::new(-910.0, 290.0), eject_dir: -1.0, floor_y: 500.0 };
        let c = shot_to_css(&shot, area, 1.0);
        assert_eq!(c.from, P { x: 100.0, y: 200.0 });
        assert_eq!(c.to, P { x: 500.0, y: 300.0 });
        assert_eq!(c.floor, 400.0);
        // Windows em 150%: 1 px físico = 1/1,5 px CSS
        let w = shot_to_css(&shot, area, 1.0 / 1.5);
        assert!((w.from.x - 66.6667).abs() < 0.01 && (w.floor - 266.6667).abs() < 0.01);
    }

    #[test]
    fn payload_uses_camel_case() {
        let s = FxShot { from: P { x: 1.0, y: 2.0 }, to: P { x: 3.0, y: 4.0 }, eject: P { x: 5.0, y: 6.0 }, eject_dir: -1.0, floor: 9.0 };
        let j = serde_json::to_string(&s).unwrap();
        assert!(j.contains("\"ejectDir\":-1.0") && j.contains("\"floor\":9.0"), "{j}");
    }
}
