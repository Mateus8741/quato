//! "Espaço da simulação": macOS = pontos lógicos globais; Windows = pixels físicos.
use crate::sim::vec2::{Rect, Vec2};
use tauri::{AppHandle, PhysicalPosition, Position, WebviewWindow};

/// Janela do pato: 192×192 lógicos. O canvas (128×128) fica em left 32 / top 64;
/// a faixa de cima (0..64) é a área da plaquinha/balão.
pub const WIN_SIZE: f32 = 192.0;
/// Meio dos pés em relação ao canto da janela.
pub const ANCHOR: Vec2 = Vec2 { x: 96.0, y: 184.0 };

#[derive(Clone, Copy, Debug)]
pub struct MonitorSim {
    pub rect: Rect,
    pub work: Rect,
    pub scale: f32,
}

/// Converte uma medida física para o espaço da simulação.
pub fn to_sim(mac: bool, physical: f32, scale: f32) -> f32 {
    if mac {
        physical / scale
    } else {
        physical
    }
}

const MAC: bool = cfg!(target_os = "macos");

pub fn monitors(app: &AppHandle) -> Vec<MonitorSim> {
    let Ok(list) = app.available_monitors() else { return vec![] };
    list.iter()
        .map(|m| {
            let sc = m.scale_factor() as f32;
            let p = m.position();
            let s = m.size();
            let w = m.work_area();
            MonitorSim {
                rect: Rect::new(to_sim(MAC, p.x as f32, sc), to_sim(MAC, p.y as f32, sc), to_sim(MAC, s.width as f32, sc), to_sim(MAC, s.height as f32, sc)),
                work: Rect::new(
                    to_sim(MAC, w.position.x as f32, sc),
                    to_sim(MAC, w.position.y as f32, sc),
                    to_sim(MAC, w.size.width as f32, sc),
                    to_sim(MAC, w.size.height as f32, sc),
                ),
                scale: sc,
            }
        })
        .collect()
}

/// Monitor primário (o que tem origem 0,0 no espaço global) ou o primeiro da lista.
pub fn primary(app: &AppHandle, all: &[MonitorSim]) -> Option<MonitorSim> {
    all.iter().copied().find(|m| m.rect.x.abs() < 1.0 && m.rect.y.abs() < 1.0).or_else(|| all.first().copied()).or_else(|| {
        let _ = app;
        None
    })
}

/// `primary_scale` vem do cache de monitores (evita uma chamada à main thread por tick).
pub fn cursor(app: &AppHandle, primary_scale: f32) -> Option<Vec2> {
    let p = app.cursor_position().ok()?;
    let sc = primary_scale;
    Some(Vec2::new(to_sim(MAC, p.x as f32, sc), to_sim(MAC, p.y as f32, sc)))
}

pub fn union_bounds(all: &[MonitorSim]) -> Option<Rect> {
    let mut it = all.iter();
    let first = it.next()?.rect;
    Some(it.fold(first, |acc, m| acc.union(&m.rect)))
}

/// União das áreas úteis (sem a barra de tarefas/dock/menu), onde o pato pode andar.
pub fn union_work(all: &[MonitorSim]) -> Option<Rect> {
    let mut it = all.iter();
    let first = it.next()?.work;
    Some(it.fold(first, |acc, m| acc.union(&m.work)))
}

/// Escala de "unidades de design" -> unidades da simulação, no monitor onde o pato está.
pub fn unit_scale(all: &[MonitorSim], pos: Vec2) -> f32 {
    if MAC {
        return 1.0;
    }
    all.iter().find(|m| pos.x >= m.rect.x && pos.x <= m.rect.right() && pos.y >= m.rect.y && pos.y <= m.rect.bottom()).map(|m| m.scale).unwrap_or(1.0)
}

/// Monitor que contém o ponto (espaço da simulação).
pub fn monitor_at(all: &[MonitorSim], p: Vec2) -> Option<MonitorSim> {
    all.iter().copied().find(|m| p.x >= m.rect.x && p.x <= m.rect.right() && p.y >= m.rect.y && p.y <= m.rect.bottom())
}

/// Quantos pixels CSS vale uma unidade da simulação nesse monitor (Mac: 1; Windows: 1 / escala).
pub fn css_per_sim(m: &MonitorSim) -> f32 {
    if MAC {
        1.0
    } else {
        1.0 / m.scale.max(0.1)
    }
}

/// Posiciona e dimensiona uma janela para cobrir o retângulo (espaço da simulação).
pub fn cover_rect(win: &WebviewWindow, r: Rect) {
    if MAC {
        let _ = win.set_size(tauri::LogicalSize::new(r.w as f64, r.h as f64));
        let _ = win.set_position(tauri::LogicalPosition::new(r.x as f64, r.y as f64));
    } else {
        let _ = win.set_size(tauri::PhysicalSize::new(r.w.round() as u32, r.h.round() as u32));
        let _ = win.set_position(Position::Physical(PhysicalPosition::new(r.x.round() as i32, r.y.round() as i32)));
    }
}

pub fn place_window(win: &WebviewWindow, pos: Vec2, unit_scale: f32) {
    let tl = pos - ANCHOR * unit_scale;
    if MAC {
        let _ = win.set_position(tauri::LogicalPosition::new(tl.x as f64, tl.y as f64));
    } else {
        let _ = win.set_position(Position::Physical(PhysicalPosition::new(tl.x.round() as i32, tl.y.round() as i32)));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn conversion() {
        assert_eq!(to_sim(true, 3024.0, 2.0), 1512.0);
        assert_eq!(to_sim(false, 3024.0, 2.0), 3024.0);
    }

    #[test]
    fn window_geometry_matches_contract() {
        assert_eq!(WIN_SIZE, 192.0);
        assert_eq!((ANCHOR.x, ANCHOR.y), (96.0, 184.0));
        // a âncora fica dentro da janela, com 8 px abaixo dos pés
        assert_eq!(WIN_SIZE - ANCHOR.y, 8.0);
        let conf = include_str!("../tauri.conf.json");
        assert!(conf.contains("\"width\": 192") && conf.contains("\"height\": 192"), "tauri.conf.json fora do contrato");
    }
}
