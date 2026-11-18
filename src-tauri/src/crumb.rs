//! Migalha: o atalho joga uma migalha na posição do cursor; o pato vai até ela e "come".
//!
//! A decisão (para onde ele anda, se ignora, quando come) é da simulação (`sim/model.rs`).
//! Aqui só mora a janelinha transparente que desenha a migalha: sem foco, sem cliques, sem borda.
//! Nunca mexe no mouse: o cursor continua onde a pessoa deixou.
use crate::platform::{self, MonitorSim};
use crate::sim::model::CrumbEvent;
use crate::sim::vec2::{Rect, Vec2};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;
use tauri::{AppHandle, Emitter, Manager, WebviewUrl, WebviewWindowBuilder};

pub const LABEL: &str = "crumb";
/// A migalha foi jogada de novo (reinicia o desenho se a janela já existe).
pub const EV_CRUMB_SHOW: &str = "crumb:show";
/// O pato chegou e está comendo (a página faz as partículas).
pub const EV_CRUMB_EAT: &str = "crumb:eat";
/// Lado da janelinha, em unidades de design (multiplicado pela escala no Windows).
pub const SIZE: f32 = 48.0;
/// Depois de comer, a janela some quando a animação acaba.
const CLOSE_AFTER_EAT_MS: u64 = 1_100;

/// Muda a cada migalha nova: a thread que fecha a janela depois de comer só fecha a sua.
static GENERATION: AtomicU64 = AtomicU64::new(0);

/// Retângulo da janelinha (espaço da simulação) centrado na migalha.
pub fn window_rect(center: Vec2, scale: f32) -> Rect {
    let side = SIZE * scale;
    Rect::new(center.x - side / 2.0, center.y - side / 2.0, side, side)
}

fn create(app: &AppHandle, rect: Rect) -> tauri::Result<()> {
    let w = WebviewWindowBuilder::new(app, LABEL, WebviewUrl::App("crumb.html".into()))
        .title("Quato — migalha")
        .inner_size(SIZE as f64, SIZE as f64)
        .position(-10000.0, -10000.0)
        .resizable(false)
        .decorations(false)
        .transparent(true)
        .shadow(false)
        .always_on_top(true)
        .skip_taskbar(true)
        .visible_on_all_workspaces(true)
        .focused(false)
        .focusable(false)
        .visible(false)
        .build()?;
    platform::cover_rect(&w, rect);
    let _ = w.set_ignore_cursor_events(true);
    let _ = w.show();
    Ok(())
}

fn show(app: &AppHandle, pos: Vec2, all: &[MonitorSim]) {
    GENERATION.fetch_add(1, Ordering::SeqCst);
    let rect = window_rect(pos, platform::unit_scale(all, pos));
    if let Some(w) = app.get_webview_window(LABEL) {
        platform::cover_rect(&w, rect);
        let _ = w.show();
        let _ = app.emit_to(LABEL, EV_CRUMB_SHOW, ());
    } else if let Err(e) = create(app, rect) {
        eprintln!("[quato] não consegui mostrar a migalha ({e}); o pato come mesmo assim");
    }
}

fn eat(app: &AppHandle) {
    let _ = app.emit_to(LABEL, EV_CRUMB_EAT, ());
    let gen = GENERATION.load(Ordering::SeqCst);
    let app = app.clone();
    std::thread::spawn(move || {
        std::thread::sleep(Duration::from_millis(CLOSE_AFTER_EAT_MS));
        if GENERATION.load(Ordering::SeqCst) == gen {
            close(&app);
        }
    });
}

/// Fecha a janelinha (se existir).
pub fn close(app: &AppHandle) {
    GENERATION.fetch_add(1, Ordering::SeqCst);
    if let Some(w) = app.get_webview_window(LABEL) {
        let _ = w.close();
    }
}

/// Chamado pela thread da simulação com o que ela contou.
pub fn handle(app: &AppHandle, ev: CrumbEvent, all: &[MonitorSim]) {
    match ev {
        CrumbEvent::Dropped(p) => show(app, p, all),
        CrumbEvent::Eaten(_) => eat(app),
        CrumbEvent::Gone => close(app),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn window_is_centered_on_the_crumb() {
        let r = window_rect(Vec2::new(100.0, 200.0), 1.0);
        assert_eq!((r.x, r.y, r.w, r.h), (76.0, 176.0, 48.0, 48.0));
        // Windows a 150%: janela em pixels físicos, ainda centrada
        let r = window_rect(Vec2::new(300.0, 300.0), 1.5);
        assert_eq!((r.x, r.y, r.w, r.h), (264.0, 264.0, 72.0, 72.0));
    }

    #[test]
    fn page_and_capability_exist() {
        let caps = include_str!("../capabilities/default.json");
        assert!(caps.contains("\"crumb\""), "a janela da migalha precisa estar na capability");
    }
}
