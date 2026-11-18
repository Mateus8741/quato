//! "Abrir ao ligar o computador": usa tauri-plugin-autostart (macOS: LaunchAgent; Windows: chave
//! Run do usuário no registro). Padrão desligado. A fonte da verdade é o sistema (a pessoa pode
//! desligar pelos Ajustes do sistema): na abertura a config é alinhada com ele.
use crate::state::AppState;
use std::sync::OnceLock;
use tauri::menu::{CheckMenuItem, Menu};
use tauri::{AppHandle, Manager, Wry};
use tauri_plugin_autostart::ManagerExt;

pub const MENU_ID: &str = "autostart";
const LABEL: &str = "Abrir ao ligar o computador";

static ITEM: OnceLock<CheckMenuItem<Wry>> = OnceLock::new();

pub fn plugin() -> tauri::plugin::TauriPlugin<Wry> {
    tauri_plugin_autostart::init(tauri_plugin_autostart::MacosLauncher::LaunchAgent, None)
}

/// Na abertura: copia o estado real do sistema para a config.
pub fn reconcile(app: &AppHandle) {
    let Ok(on) = app.autolaunch().is_enabled() else { return };
    let st = app.state::<AppState>();
    let mut cfg = st.config.lock().unwrap();
    if cfg.launch_at_login != on {
        cfg.launch_at_login = on;
        crate::config::save(app, &cfg);
    }
}

/// Liga/desliga no sistema e na config. Devolve o estado final (o real do sistema).
pub fn set(app: &AppHandle, on: bool) -> bool {
    let manager = app.autolaunch();
    let res = if on { manager.enable() } else { manager.disable() };
    let ok = res.is_ok();
    if let Err(e) = res {
        eprintln!("[quato] não consegui mudar \"abrir ao ligar\": {e}");
    }
    let real = manager.is_enabled().unwrap_or(on && ok);
    {
        let st = app.state::<AppState>();
        let mut cfg = st.config.lock().unwrap();
        cfg.launch_at_login = real;
        crate::config::save(app, &cfg);
    }
    crate::actions::emit_settings(app);
    sync_tray(app);
    real
}

pub fn toggle(app: &AppHandle) {
    let on = app.state::<AppState>().config.lock().unwrap().launch_at_login;
    set(app, !on);
}

/// Cria o item do menu da bandeja, antes do bloco final (separador, atualização, configurações, sair).
pub fn add_to_menu(app: &AppHandle, menu: &Menu<Wry>) -> tauri::Result<()> {
    let on = app.state::<AppState>().config.lock().unwrap().launch_at_login;
    let item = CheckMenuItem::with_id(app, MENU_ID, LABEL, true, on, None::<&str>)?;
    let len = menu.items()?.len();
    menu.insert(&item, len.saturating_sub(4))?;
    let _ = ITEM.set(item);
    Ok(())
}

/// Atualiza o check do item conforme a config (sem esperar a main thread).
pub fn sync_tray(app: &AppHandle) {
    let on = app.state::<AppState>().config.lock().unwrap().launch_at_login;
    if let Some(item) = ITEM.get().cloned() {
        let _ = app.run_on_main_thread(move || {
            let _ = item.set_checked(on);
        });
    }
}
