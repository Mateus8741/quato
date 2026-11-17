//! Ações de alto nível chamadas por tray, atalhos e comandos. Só enviam mensagens
//! para a simulação e atualizam estado: nada aqui segura lock chamando o AppHandle.
use crate::events::*;
use crate::sim::model::SimCommand;
use crate::sim::mood::Mood;
use crate::state::AppState;
use std::sync::atomic::Ordering;
use tauri::{AppHandle, Emitter, Manager};

pub fn send(app: &AppHandle, cmd: SimCommand) {
    let _ = app.state::<AppState>().tx.send(cmd);
}

pub fn current_view(app: &AppHandle) -> DuckView {
    *app.state::<AppState>().view.lock().unwrap()
}

pub fn toggle_chase(app: &AppHandle) {
    let on = current_view(app).chasing;
    send(app, SimCommand::SetChasing(!on));
}

pub fn set_chasing(app: &AppHandle, on: bool) {
    send(app, SimCommand::SetChasing(on));
}

pub fn set_mood(app: &AppHandle, mood: Mood) {
    {
        let st = app.state::<AppState>();
        let mut cfg = st.config.lock().unwrap();
        cfg.mood = mood;
        crate::config::save(app, &cfg);
    }
    send(app, SimCommand::SetMood(mood));
    emit_settings(app);
    crate::tray::sync(app);
}

pub fn current_look(app: &AppHandle) -> Look {
    let st = app.state::<AppState>();
    let cfg = st.config.lock().unwrap();
    Look { color: cfg.color.clone(), accessories: cfg.accessories.clone() }
}

fn look_changed(app: &AppHandle) {
    let _ = app.emit(EV_DUCK_LOOK, current_look(app));
    emit_settings(app);
}

pub fn set_color(app: &AppHandle, color: &str) {
    {
        let st = app.state::<AppState>();
        let mut cfg = st.config.lock().unwrap();
        cfg.color = crate::look::sanitize_color(color);
        crate::config::save(app, &cfg);
    }
    look_changed(app);
}

pub fn set_accessories(app: &AppHandle, list: &[String]) {
    {
        let st = app.state::<AppState>();
        let mut cfg = st.config.lock().unwrap();
        cfg.accessories = crate::look::sanitize_accessories(list);
        crate::config::save(app, &cfg);
    }
    look_changed(app);
}

pub fn set_free_roam(app: &AppHandle, on: bool) {
    {
        let st = app.state::<AppState>();
        let mut cfg = st.config.lock().unwrap();
        cfg.free_roam = on;
        crate::config::save(app, &cfg);
    }
    send(app, SimCommand::SetFreeRoam(on));
    emit_settings(app);
    crate::tray::sync(app);
}

/// A pessoa soltou o pato num lugar novo (chamado pela thread da simulação).
pub fn home_changed(app: &AppHandle, pos: crate::sim::vec2::Vec2) {
    {
        let st = app.state::<AppState>();
        let mut cfg = st.config.lock().unwrap();
        cfg.home = Some([pos.x, pos.y]);
        crate::config::save(app, &cfg);
    }
    emit_settings(app);
}

/// Volta para o canto padrão.
pub fn reset_home(app: &AppHandle) {
    {
        let st = app.state::<AppState>();
        let mut cfg = st.config.lock().unwrap();
        cfg.home = None;
        crate::config::save(app, &cfg);
    }
    send(app, SimCommand::SetHome(None));
    emit_settings(app);
}

pub fn set_dance_on_music(app: &AppHandle, on: bool) {
    {
        let st = app.state::<AppState>();
        let mut cfg = st.config.lock().unwrap();
        cfg.dance_on_music = on;
        crate::config::save(app, &cfg);
    }
    send(app, SimCommand::SetDanceOnMusic(on));
    emit_settings(app);
}

pub fn cycle_mood(app: &AppHandle) {
    let next = app.state::<AppState>().config.lock().unwrap().mood.next();
    set_mood(app, next);
}

pub fn release_mouse(app: &AppHandle) {
    send(app, SimCommand::ReleaseMouse);
}

pub fn set_drag(app: &AppHandle, enabled: bool) -> Result<bool, DragError> {
    if enabled && crate::accessibility::required() && !crate::accessibility::is_trusted() {
        {
            let st = app.state::<AppState>();
            let mut cfg = st.config.lock().unwrap();
            if !cfg.accessibility_explained {
                cfg.accessibility_explained = true;
                crate::config::save(app, &cfg);
            }
        }
        crate::windows::open_settings(app, Some("accessibility"));
        let _ = app.emit(EV_ACCESSIBILITY_CHANGED, crate::accessibility::status());
        crate::tray::sync(app);
        return Err(DragError {
            kind: DragErrorKind::AccessibilityRequired,
            message: "Para mexer no mouse de verdade, o Quato precisa da permissão de Acessibilidade.".into(),
        });
    }
    send(app, SimCommand::SetDragEnabled(enabled));
    Ok(enabled)
}

pub fn settings_view(app: &AppHandle) -> SettingsView {
    let water_next_in_sec = crate::water::next_in_sec(app);
    let st = app.state::<AppState>();
    let view = *st.view.lock().unwrap();
    let config = st.config.lock().unwrap().clone();
    let shortcut_status = st.shortcut_status.lock().unwrap().clone();
    SettingsView {
        config,
        drag_enabled: view.drag_enabled,
        shortcut_status,
        shortcuts_paused: st.paused.load(Ordering::SeqCst),
        platform: current_platform(),
        accessibility: crate::accessibility::status(),
        defaults: ShortcutsConfig::default(),
        water_next_in_sec,
        app_version: crate::update::CURRENT_VERSION.to_string(),
        update: crate::update::current(app),
    }
}

pub fn emit_settings(app: &AppHandle) {
    let _ = app.emit(EV_SETTINGS_CHANGED, settings_view(app));
}

/// Chamado pela thread da simulação quando o DuckView muda.
pub fn on_view_changed(app: &AppHandle, view: DuckView) {
    *app.state::<AppState>().view.lock().unwrap() = view;
    let _ = app.emit(EV_DUCK_VIEW, view);
    emit_settings(app);
    crate::tray::sync(app);
}
