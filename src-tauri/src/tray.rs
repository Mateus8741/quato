use crate::actions;
use crate::events::*;
use crate::shortcuts::format_accelerator;
use crate::sim::model::SimCommand;
use crate::sim::mood::Mood;
use crate::state::{AppState, TrayHandles};
use tauri::image::Image;
use tauri::menu::{CheckMenuItem, IsMenuItem, Menu, MenuItem, PredefinedMenuItem, Submenu};
use tauri::tray::TrayIconBuilder;
use tauri::{AppHandle, Manager, Wry};

fn update_label(u: Option<&crate::update::UpdateInfo>) -> String {
    match u {
        Some(u) => format!("⬇ Baixar a versão {} (nova!)", u.version),
        None => "Procurar atualizações…".into(),
    }
}

fn chase_label(app: &AppHandle) -> String {
    let st = app.state::<AppState>();
    let accel = st.config.lock().unwrap().shortcuts.toggle_chase.clone();
    let broken = st.shortcut_status.lock().unwrap().toggle_chase.is_some();
    let txt = format_accelerator(&accel, current_platform());
    if broken {
        format!("Perseguir o cursor  ⚠ ({txt} indisponível)")
    } else {
        format!("Perseguir o cursor  ({txt})")
    }
}

pub fn create(app: &AppHandle) -> tauri::Result<()> {
    let chase = CheckMenuItem::with_id(app, "chase", chase_label(app), true, false, None::<&str>)?;
    let roam = CheckMenuItem::with_id(app, "roam", "Passear pela tela toda", true, false, None::<&str>)?;
    let mut mood_items: Vec<(Mood, CheckMenuItem<Wry>)> = vec![];
    for m in Mood::ALL {
        mood_items.push((m, CheckMenuItem::with_id(app, format!("mood:{}", m.id()), m.label(), true, false, None::<&str>)?));
    }
    let refs: Vec<&dyn IsMenuItem<Wry>> = mood_items.iter().map(|(_, i)| i as &dyn IsMenuItem<Wry>).collect();
    let moods = Submenu::with_items(app, "Humor", true, &refs)?;
    let drag = CheckMenuItem::with_id(app, "drag", "Arrastar o mouse", true, false, None::<&str>)?;
    let release = MenuItem::with_id(app, "release", "Soltar o mouse agora", true, None::<&str>)?;
    let water = CheckMenuItem::with_id(app, "water", "Lembrete de água", true, false, None::<&str>)?;
    let drank = MenuItem::with_id(app, "water_drank", "Já bebi água", true, None::<&str>)?;
    let update = MenuItem::with_id(app, "update", update_label(None), true, None::<&str>)?;
    let settings = MenuItem::with_id(app, "settings", "Configurações…", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "Sair", true, None::<&str>)?;
    let sep = PredefinedMenuItem::separator(app)?;
    let sep2 = PredefinedMenuItem::separator(app)?;
    let menu = Menu::with_items(app, &[&chase, &roam, &moods, &drag, &release, &sep2, &water, &drank, &sep, &update, &settings, &quit])?;

    crate::autostart::add_to_menu(app, &menu)?;

    *app.state::<AppState>().tray.lock().unwrap() = Some(TrayHandles { chase, roam, moods: mood_items, drag, water, update });

    TrayIconBuilder::with_id("main")
        .icon(Image::from_bytes(include_bytes!("../icons/tray@2x.png"))?)
        .icon_as_template(true)
        .tooltip("Quato")
        .menu(&menu)
        .show_menu_on_left_click(true)
        .on_menu_event(|app, event| {
            let id = event.id().as_ref().to_string();
            let app = app.clone();
            // Fora da main thread: abrir janelas dentro do handler pode travar no Windows.
            std::thread::spawn(move || {
                on_menu(&app, &id);
                sync(&app);
            });
        })
        .build(app)?;
    sync(app);
    Ok(())
}

fn on_menu(app: &AppHandle, id: &str) {
    match id {
        "chase" => actions::toggle_chase(app),
        "roam" => {
            let on = app.state::<AppState>().config.lock().unwrap().free_roam;
            actions::set_free_roam(app, !on);
        }
        "drag" => {
            let target = !actions::current_view(app).drag_enabled;
            let _ = actions::set_drag(app, target);
        }
        "release" => actions::release_mouse(app),
        "water" => crate::water::toggle(app),
        "water_drank" => crate::water::drank(app),
        "update" => crate::update::on_tray_click(app),
        crate::autostart::MENU_ID => crate::autostart::toggle(app),
        "settings" => crate::windows::open_settings(app, None),
        "quit" => {
            actions::send(app, SimCommand::Shutdown);
            std::thread::sleep(std::time::Duration::from_millis(100));
            app.exit(0);
        }
        other => {
            if let Some(name) = other.strip_prefix("mood:") {
                if let Some(m) = Mood::ALL.into_iter().find(|m| m.id() == name) {
                    actions::set_mood(app, m);
                }
            }
        }
    }
}

/// Atualiza checks e rótulos do menu conforme o estado atual.
///
/// Não segura nenhum lock enquanto fala com a main thread e não espera por ela:
/// antes, a thread da simulação segurava o lock do tray esperando a main thread,
/// enquanto a main thread (atalho de teclado) esperava o mesmo lock: travava o app.
pub fn sync(app: &AppHandle) {
    let st = app.state::<AppState>();
    let view = *st.view.lock().unwrap();
    let (mood, free_roam, water_on) = {
        let cfg = st.config.lock().unwrap();
        (cfg.mood, cfg.free_roam, cfg.water.enabled)
    };
    crate::autostart::sync_tray(app);
    let label = chase_label(app);
    let update_text = update_label(crate::update::current(app).as_ref());
    let handles = st.tray.lock().unwrap().as_ref().map(|h| (h.chase.clone(), h.roam.clone(), h.drag.clone(), h.moods.clone(), h.water.clone(), h.update.clone()));
    let Some((chase, roam, drag, moods, water, update)) = handles else { return };
    let _ = app.run_on_main_thread(move || {
        let _ = chase.set_checked(view.chasing);
        let _ = chase.set_text(label);
        let _ = roam.set_checked(free_roam);
        let _ = drag.set_checked(view.drag_enabled);
        let _ = water.set_checked(water_on);
        let _ = update.set_text(update_text);
        for (m, item) in &moods {
            let _ = item.set_checked(*m == mood);
        }
    });
}
