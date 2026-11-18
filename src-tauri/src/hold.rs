//! Ctrl + arrastar: segurando Ctrl com o cursor em cima do pato, ele deixa de ser
//! "transparente" para cliques e pode ser carregado para outro lugar da tela.
//!
//! Pena do meio: com o cursor em cima de uma patinha (sem Ctrl), a janela também passa
//! a receber cliques; clicar ali faz o pato levantar a asa (`duck:gesture`). O clique é
//! engolido só nas patinhas; no resto do corpo, sem Ctrl, os cliques continuam passando.
//!
//! `HoldTracker` é a lógica pura (testável); `spawn` liga ela ao teclado/mouse reais.
use crate::events::{DuckGesture, GestureKind, EV_DUCK_GESTURE, FEATHER_MS};
use crate::platform_input::{self, InputState};
use crate::sim::model::SimCommand;
use crate::sim::vec2::{Rect, Vec2};
use crate::state::AppState;
use std::time::{Duration, Instant};
use tauri::{AppHandle, Emitter, Manager};

/// Área clicável do pato em relação aos pés (âncora), em pixels de design: (x0, y0, x1, y1).
const HIT: (f32, f32, f32, f32) = (-48.0, -100.0, 44.0, 4.0);
/// Patinhas (clique da "pena do meio"), em relação à âncora.
const PAWS: [(f32, f32, f32, f32); 2] = [(-30.0, -6.0, -6.0, 8.0), (2.0, -6.0, 30.0, 8.0)];
/// Intervalo mínimo entre duas penas do meio.
const FEATHER_COOLDOWN: f64 = 0.6;

fn rect_from(pos: Vec2, scale: f32, r: (f32, f32, f32, f32)) -> Rect {
    Rect::new(pos.x + r.0 * scale, pos.y + r.1 * scale, (r.2 - r.0) * scale, (r.3 - r.1) * scale)
}

pub fn hit_rect(pos: Vec2, scale: f32) -> Rect {
    rect_from(pos, scale, HIT)
}

/// Onde dá para mexer no pato agora (espaço da simulação).
#[derive(Clone, Copy, Debug)]
pub struct HitAreas {
    pub body: Rect,
    pub paws: [Rect; 2],
}

pub fn hit_areas(pos: Vec2, scale: f32) -> HitAreas {
    HitAreas { body: hit_rect(pos, scale), paws: PAWS.map(|r| rect_from(pos, scale, r)) }
}

fn inside(r: &Rect, p: Vec2) -> bool {
    p.x >= r.x && p.x <= r.right() && p.y >= r.y && p.y <= r.bottom()
}

#[derive(Debug, PartialEq)]
pub struct HoldOut {
    /// A janela do pato deve receber cliques (em vez de deixá-los passar).
    pub capture: bool,
    pub command: Option<SimCommand>,
    /// Clicou numa patinha: mostrar a pena do meio.
    pub feather: bool,
}

#[derive(Default)]
pub struct HoldTracker {
    dragging: bool,
    prev_button: bool,
    feather_ready_at: f64,
}

impl HoldTracker {
    /// `now` em segundos (qualquer relógio crescente).
    pub fn update(&mut self, st: &InputState, areas: &HitAreas, now: f64) -> HoldOut {
        let over = |r: &Rect| st.cursor.map(|c| inside(r, c)).unwrap_or(false);
        let over_paw = areas.paws.iter().any(over);
        let over_body = over(&areas.body) || over_paw;
        let ctrl_hover = st.ctrl && over_body;
        let click = st.button && !self.prev_button;
        let mut command = None;
        let mut feather = false;
        // Só começa a carregar no instante do clique (não vale já chegar com o botão apertado).
        if !self.dragging && ctrl_hover && click {
            if let Some(c) = st.cursor {
                self.dragging = true;
                command = Some(SimCommand::UserGrab(c));
            }
        } else if self.dragging && !st.button {
            self.dragging = false;
            command = Some(SimCommand::UserDrop);
        } else if !self.dragging && !st.ctrl && over_paw && click && now >= self.feather_ready_at {
            feather = true;
            self.feather_ready_at = now + FEATHER_COOLDOWN;
        }
        self.prev_button = st.button;
        HoldOut { capture: ctrl_hover || self.dragging || (over_paw && !st.ctrl), command, feather }
    }
}

pub fn spawn(app: AppHandle) {
    std::thread::Builder::new()
        .name("quato-hold".into())
        .spawn(move || {
            let start = Instant::now();
            let mut tracker = HoldTracker::default();
            let mut capturing = false;
            loop {
                std::thread::sleep(Duration::from_millis(25));
                let geom = *app.state::<AppState>().duck_geom.lock().unwrap();
                if !geom.shown || crate::dnd::is_hidden() {
                    if capturing {
                        if let Some(w) = app.get_webview_window("duck") {
                            let _ = w.set_ignore_cursor_events(true);
                        }
                        capturing = false;
                    }
                    continue;
                }
                let out = tracker.update(&platform_input::read(), &hit_areas(geom.pos, geom.scale), start.elapsed().as_secs_f64());
                if out.capture != capturing {
                    if let Some(w) = app.get_webview_window("duck") {
                        let _ = w.set_ignore_cursor_events(!out.capture);
                    }
                    capturing = out.capture;
                }
                if let Some(cmd) = out.command {
                    crate::actions::send(&app, cmd);
                }
                if out.feather {
                    let _ = app.emit(EV_DUCK_GESTURE, DuckGesture { kind: GestureKind::Feather, duration_ms: FEATHER_MS });
                }
            }
        })
        .expect("não consegui iniciar o vigia do Ctrl + arrastar");
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Pato com os pés em (145, 196), escala 1: corpo x 97..189, y 96..200;
    /// patinhas x 115..139 e 147..175, y 190..204.
    fn rect() -> HitAreas {
        hit_areas(Vec2::new(145.0, 196.0), 1.0)
    }
    fn st(ctrl: bool, button: bool, x: f32, y: f32) -> InputState {
        InputState { ctrl, button, cursor: Some(Vec2::new(x, y)) }
    }
    const PAW: (f32, f32) = (125.0, 200.0);
    const PAW_R: (f32, f32) = (160.0, 202.0);

    #[test]
    fn without_ctrl_clicks_pass_through() {
        let mut t = HoldTracker::default();
        let o = t.update(&st(false, false, 120.0, 120.0), &rect(), 0.0);
        assert_eq!(o, HoldOut { capture: false, command: None, feather: false });
        let o = t.update(&st(false, true, 120.0, 120.0), &rect(), 0.0);
        assert_eq!(o, HoldOut { capture: false, command: None, feather: false });
    }

    #[test]
    fn ctrl_over_duck_captures_clicks_but_ctrl_elsewhere_does_not() {
        let mut t = HoldTracker::default();
        assert!(t.update(&st(true, false, 120.0, 120.0), &rect(), 0.0).capture);
        assert!(!t.update(&st(true, false, 500.0, 500.0), &rect(), 0.0).capture);
    }

    #[test]
    fn click_starts_carry_and_release_drops() {
        let mut t = HoldTracker::default();
        t.update(&st(true, false, 120.0, 120.0), &rect(), 0.0);
        let o = t.update(&st(true, true, 120.0, 120.0), &rect(), 0.0);
        assert!(matches!(o.command, Some(SimCommand::UserGrab(_))));
        assert!(o.capture);
        // arrastando para longe do retângulo velho: continua carregando
        let o = t.update(&st(true, true, 600.0, 600.0), &rect(), 0.0);
        assert_eq!(o, HoldOut { capture: true, command: None, feather: false });
        let o = t.update(&st(true, false, 600.0, 600.0), &rect(), 0.0);
        assert!(matches!(o.command, Some(SimCommand::UserDrop)));
        assert!(!o.capture);
    }

    #[test]
    fn releasing_ctrl_mid_drag_keeps_carrying_until_button_up() {
        let mut t = HoldTracker::default();
        t.update(&st(true, false, 120.0, 120.0), &rect(), 0.0);
        t.update(&st(true, true, 120.0, 120.0), &rect(), 0.0);
        let o = t.update(&st(false, true, 130.0, 130.0), &rect(), 0.0);
        assert_eq!(o, HoldOut { capture: true, command: None, feather: false });
        let o = t.update(&st(false, false, 130.0, 130.0), &rect(), 0.0);
        assert!(matches!(o.command, Some(SimCommand::UserDrop)));
    }

    #[test]
    fn arriving_with_button_already_down_does_not_grab() {
        let mut t = HoldTracker::default();
        t.update(&st(true, true, 500.0, 500.0), &rect(), 0.0);
        let o = t.update(&st(true, true, 120.0, 120.0), &rect(), 0.0);
        assert_eq!(o.command, None);
    }

    #[test]
    fn hovering_a_paw_captures_clicks_without_ctrl() {
        let mut t = HoldTracker::default();
        assert!(t.update(&st(false, false, PAW.0, PAW.1), &rect(), 0.0).capture);
        assert!(t.update(&st(false, false, PAW_R.0, PAW_R.1), &rect(), 0.0).capture);
        // entre as patinhas e no corpo, sem Ctrl, os cliques passam
        assert!(!t.update(&st(false, false, 143.0, 200.0), &rect(), 0.0).capture);
        assert!(!t.update(&st(false, false, 140.0, 140.0), &rect(), 0.0).capture);
    }

    #[test]
    fn clicking_a_paw_shows_the_feather_with_cooldown() {
        let mut t = HoldTracker::default();
        t.update(&st(false, false, PAW.0, PAW.1), &rect(), 0.0);
        let o = t.update(&st(false, true, PAW.0, PAW.1), &rect(), 0.1);
        assert!(o.feather && o.capture && o.command.is_none());
        // segurar o botão não repete
        assert!(!t.update(&st(false, true, PAW.0, PAW.1), &rect(), 0.2).feather);
        t.update(&st(false, false, PAW.0, PAW.1), &rect(), 0.3);
        // clique de novo antes de 600 ms: nada
        assert!(!t.update(&st(false, true, PAW.0, PAW.1), &rect(), 0.4).feather);
        t.update(&st(false, false, PAW.0, PAW.1), &rect(), 0.5);
        assert!(t.update(&st(false, true, PAW_R.0, PAW_R.1), &rect(), 0.8).feather);
    }

    #[test]
    fn clicking_outside_the_paws_does_nothing() {
        let mut t = HoldTracker::default();
        for (x, y) in [(140.0, 140.0), (143.0, 200.0), (500.0, 500.0), (125.0, 210.0)] {
            t.update(&st(false, false, x, y), &rect(), 0.0);
            let o = t.update(&st(false, true, x, y), &rect(), 10.0);
            assert_eq!(o, HoldOut { capture: false, command: None, feather: false }, "em ({x}, {y})");
            t.update(&st(false, false, x, y), &rect(), 10.1);
        }
    }

    #[test]
    fn ctrl_click_still_carries_even_on_a_paw() {
        for (x, y) in [(140.0, 140.0), PAW] {
            let mut t = HoldTracker::default();
            t.update(&st(true, false, x, y), &rect(), 0.0);
            let o = t.update(&st(true, true, x, y), &rect(), 0.1);
            assert!(matches!(o.command, Some(SimCommand::UserGrab(_))));
            assert!(!o.feather);
        }
    }

    #[test]
    fn no_feather_while_carrying() {
        let mut t = HoldTracker::default();
        t.update(&st(true, false, 140.0, 140.0), &rect(), 0.0);
        t.update(&st(true, true, 140.0, 140.0), &rect(), 0.1);
        // arrastando por cima de uma patinha, soltando o Ctrl
        let o = t.update(&st(false, true, PAW.0, PAW.1), &rect(), 0.2);
        assert!(!o.feather && o.capture);
        let o = t.update(&st(false, false, PAW.0, PAW.1), &rect(), 0.3);
        assert!(matches!(o.command, Some(SimCommand::UserDrop)) && !o.feather);
    }

    #[test]
    fn paws_follow_anchor_and_scale() {
        let a = hit_areas(Vec2::new(1000.0, 800.0), 2.0);
        assert_eq!((a.paws[0].x, a.paws[0].y, a.paws[0].w, a.paws[0].h), (940.0, 788.0, 48.0, 28.0));
        assert_eq!((a.paws[1].x, a.paws[1].y, a.paws[1].w, a.paws[1].h), (1004.0, 788.0, 56.0, 28.0));
    }

    #[test]
    fn hit_rect_follows_scale() {
        let r = hit_rect(Vec2::new(1000.0, 800.0), 2.0);
        assert_eq!((r.x, r.y, r.w, r.h), (904.0, 600.0, 184.0, 208.0));
    }

    #[test]
    fn global_input_reads_real_cursor() {
        let s = crate::platform_input::read();
        assert!(s.cursor.is_some());
    }
}
