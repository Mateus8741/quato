use crate::events::EV_SETTINGS_GOTO;
use std::sync::atomic::{AtomicBool, Ordering};
use tauri::{AppHandle, Emitter, Manager, WebviewUrl, WebviewWindowBuilder, WindowEvent};

pub static DUCK_READY: AtomicBool = AtomicBool::new(false);

pub fn mark_duck_ready() {
    DUCK_READY.store(true, Ordering::SeqCst);
}

pub fn duck_ready() -> bool {
    DUCK_READY.load(Ordering::SeqCst)
}

fn set_policy(app: &AppHandle, regular: bool) {
    #[cfg(target_os = "macos")]
    {
        let a = app.clone();
        let _ = app.run_on_main_thread(move || {
            let _ = a.set_activation_policy(if regular { tauri::ActivationPolicy::Regular } else { tauri::ActivationPolicy::Accessory });
        });
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = (app, regular);
    }
}

pub fn open_settings(app: &AppHandle, section: Option<&str>) {
    if let Some(w) = app.get_webview_window("settings") {
        let _ = w.show();
        let _ = w.unminimize();
        let _ = w.set_focus();
        if let Some(s) = section {
            let _ = app.emit(EV_SETTINGS_GOTO, s);
        }
        return;
    }
    set_policy(app, true);
    let url = match section {
        Some(s) => format!("settings.html?section={s}"),
        None => "settings.html".to_string(),
    };
    let built = WebviewWindowBuilder::new(app, "settings", WebviewUrl::App(url.into()))
        .title("Quato — Configurações")
        .inner_size(460.0, 720.0)
        .resizable(false)
        .center()
        .focused(true)
        .build();
    match built {
        Ok(w) => {
            let handle = app.clone();
            w.on_window_event(move |ev| {
                if matches!(ev, WindowEvent::Destroyed) {
                    // Se o usuário fechou no meio da gravação de um atalho, reativa os atalhos.
                    crate::shortcuts::pause(&handle, false);
                    set_policy(&handle, false);
                }
            });
        }
        Err(e) => {
            eprintln!("[quato] não consegui abrir as configurações: {e}");
            set_policy(app, false);
        }
    }
}

/// Overlay dos tiros da arminha: transparente, sem borda, sem foco, sem cliques, escondido até ter o que mostrar.
/// O tamanho e o lugar são ajustados por `platform::cover_rect` antes de mostrar.
pub fn create_fx(app: &AppHandle) -> tauri::Result<()> {
    let w = WebviewWindowBuilder::new(app, crate::fx::LABEL, WebviewUrl::App("fx.html".into()))
        .title("Quato — efeitos")
        .inner_size(200.0, 200.0)
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
    let _ = w.set_ignore_cursor_events(true);
    Ok(())
}
