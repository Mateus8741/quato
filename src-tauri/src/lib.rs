pub mod accessibility;
pub mod actions;
pub mod audio;
pub mod autostart;
pub mod commands;
pub mod config;
pub mod dnd;
pub mod events;
pub mod fire;
pub mod fx;
pub mod look;
pub mod hold;
pub mod platform;
pub mod platform_input;
pub mod reminders;
pub mod pointer;
pub mod shortcuts;
pub mod sim;
pub mod state;
pub mod tray;
pub mod update;
pub mod water;
pub mod windows;

use sim::model::{Sim, SimCommand};
use state::AppState;
use std::sync::mpsc::channel;
use tauri::{Manager, RunEvent};

pub fn run() {
    let app = tauri::Builder::default()
        .plugin(tauri_plugin_global_shortcut::Builder::new().with_handler(shortcuts::handler).build())
        .plugin(autostart::plugin())
        .invoke_handler(tauri::generate_handler![
            commands::duck_ready,
            commands::get_duck_view,
            commands::get_settings_view,
            commands::set_mood,
            commands::set_chasing,
            commands::set_drag_enabled,
            commands::release_mouse,
            commands::update_shortcuts,
            commands::reset_shortcuts,
            commands::validate_shortcut,
            commands::pause_shortcuts,
            commands::accessibility_status,
            commands::request_accessibility,
            commands::get_look,
            commands::set_color,
            commands::set_accessories,
            commands::set_free_roam,
            commands::reset_home,
            commands::set_water,
            commands::water_drank,
            commands::water_test,
            commands::set_reminder,
            commands::reminder_done,
            commands::reminder_test,
            commands::set_focus_config,
            commands::focus_toggle,
            commands::set_dance_on_music,
            commands::set_seasonal_accessories,
            commands::check_update,
            commands::open_update,
            commands::set_dnd,
            commands::set_launch_at_login,
        ])
        .setup(|app| {
            #[cfg(target_os = "macos")]
            app.set_activation_policy(tauri::ActivationPolicy::Accessory);

            let handle = app.handle().clone();
            let cfg = config::load(&handle);
            let (tx, rx) = channel::<SimCommand>();
            let view = Sim::new(cfg.mood, 1).view();
            let mood = cfg.mood;
            app.manage(AppState::new(tx, cfg, view));

            autostart::reconcile(&handle);
            shortcuts::register_all(&handle);
            tray::create(&handle)?;
            sim::spawn(handle.clone(), rx, mood);
            hold::spawn(handle.clone());
            water::spawn(handle.clone());
            reminders::spawn(handle.clone());
            audio::spawn(handle.clone());
            update::spawn(handle.clone());
            {
                let st = handle.state::<AppState>();
                let cfg = st.config.lock().unwrap().clone();
                actions::send(&handle, SimCommand::SetFreeRoam(cfg.free_roam));
                actions::send(&handle, SimCommand::SetDanceOnMusic(cfg.dance_on_music));
                if let Some(h) = cfg.home {
                    actions::send(&handle, SimCommand::SetHome(Some(sim::vec2::Vec2::new(h[0], h[1]))));
                }
            }

            // Primeira abertura: o app só tem ícone na barra de menu, então mostra as configurações uma vez.
            if !handle.state::<AppState>().config.lock().unwrap().first_run_done {
                let h = handle.clone();
                std::thread::spawn(move || {
                    std::thread::sleep(std::time::Duration::from_millis(1500));
                    windows::open_settings(&h, None);
                    let st = h.state::<AppState>();
                    let mut cfg = st.config.lock().unwrap();
                    cfg.first_run_done = true;
                    config::save(&h, &cfg);
                });
            }

            // Se o frontend do pato nunca avisar que está pronto, mostra mesmo assim.
            let h = handle.clone();
            std::thread::spawn(move || {
                std::thread::sleep(std::time::Duration::from_secs(3));
                windows::mark_duck_ready();
                actions::send(&h, SimCommand::Wake);
            });
            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("erro ao iniciar o Quato");

    app.run(|handle, event| {
        if let RunEvent::Exit = event {
            actions::send(handle, SimCommand::Shutdown);
        }
    });
}
