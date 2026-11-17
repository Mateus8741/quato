use crate::actions;
use crate::events::*;
use crate::state::AppState;
use std::str::FromStr;
use std::sync::atomic::Ordering;
use tauri::{AppHandle, Manager};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutEvent, ShortcutState};

pub fn parse(s: &str) -> Option<Shortcut> {
    Shortcut::from_str(s).ok()
}

fn allowed_key(s: &Shortcut) -> bool {
    let name = format!("{:?}", s.key);
    let letter = name.len() == 4 && name.starts_with("Key");
    let digit = name.len() == 6 && name.starts_with("Digit");
    let f = name.strip_prefix('F').and_then(|n| n.parse::<u8>().ok()).map(|n| (1..=12).contains(&n)).unwrap_or(false);
    letter || digit || f || matches!(name.as_str(), "Space" | "ArrowUp" | "ArrowDown" | "ArrowLeft" | "ArrowRight")
}

fn has_real_modifier(s: &Shortcut) -> bool {
    use tauri_plugin_global_shortcut::Modifiers;
    s.mods.intersects(Modifiers::CONTROL | Modifiers::SUPER | Modifiers::ALT)
}

fn reserved_list(platform: Platform) -> Vec<String> {
    let mut v: Vec<String> = vec![];
    match platform {
        Platform::Macos => {
            for c in "QWHMCVXZASFNTP".chars() {
                v.push(format!("CommandOrControl+{c}"));
            }
            v.extend(["CommandOrControl+Space", "Control+Space", "CommandOrControl+Shift+3", "CommandOrControl+Shift+4", "CommandOrControl+Shift+5", "CommandOrControl+Shift+Q"].map(String::from));
        }
        Platform::Windows => {
            for c in "CVXZASFNTPW".chars() {
                v.push(format!("CommandOrControl+{c}"));
            }
            v.extend(["Alt+F4", "Super+L", "Super+D", "Super+E", "Super+R"].map(String::from));
        }
    }
    v
}

fn err(field: Option<&str>, kind: ShortcutErrorKind, message: &str) -> ShortcutError {
    ShortcutError { field: field.map(String::from), kind, message: message.to_string() }
}

pub fn validate_one(field: &str, accel: &str, platform: Platform) -> Result<Shortcut, ShortcutError> {
    let s = parse(accel).ok_or_else(|| err(Some(field), ShortcutErrorKind::Invalid, "Combinação inválida."))?;
    if !allowed_key(&s) {
        return Err(err(Some(field), ShortcutErrorKind::Invalid, "Essa tecla não pode ser usada. Use letras, números, F1–F12, espaço ou setas."));
    }
    if !has_real_modifier(&s) {
        return Err(err(Some(field), ShortcutErrorKind::NoModifier, "Use pelo menos Ctrl, Alt ou ⌘ junto com a tecla."));
    }
    let id = s.id();
    if reserved_list(platform).iter().filter_map(|r| parse(r)).any(|r| r.id() == id) {
        return Err(err(Some(field), ShortcutErrorKind::Reserved, "Essa combinação é usada pelo sistema. Escolha outra."));
    }
    Ok(s)
}

pub fn validate_set(cfg: &ShortcutsConfig, platform: Platform) -> Result<(Shortcut, Shortcut), ShortcutError> {
    let t = validate_one("toggleChase", &cfg.toggle_chase, platform)?;
    let c = validate_one("cycleMood", &cfg.cycle_mood, platform)?;
    if t.id() == c.id() {
        return Err(err(Some("cycleMood"), ShortcutErrorKind::Duplicate, "As duas ações não podem usar a mesma combinação."));
    }
    Ok((t, c))
}

/// Registra os atalhos da config atual. Falhas não derrubam o app: viram `ShortcutStatus`.
pub fn register_all(app: &AppHandle) {
    let st = app.state::<AppState>();
    let cfg = st.config.lock().unwrap().shortcuts.clone();
    let gs = app.global_shortcut();
    let _ = gs.unregister_all();
    let mut status = ShortcutStatus::default();
    let mut active = crate::state::ActiveShortcuts::default();
    let platform = current_platform();
    for (field, accel) in [("toggleChase", &cfg.toggle_chase), ("cycleMood", &cfg.cycle_mood)] {
        match validate_one(field, accel, platform) {
            Ok(sc) => match gs.register(sc.clone()) {
                Ok(()) => {
                    if field == "toggleChase" {
                        active.toggle = Some(sc);
                    } else {
                        active.cycle = Some(sc);
                    }
                }
                Err(e) => {
                    let msg = format!("Não consegui registrar {}: {e}. Outro app pode estar usando.", format_accelerator(accel, platform));
                    if field == "toggleChase" {
                        status.toggle_chase = Some(msg);
                    } else {
                        status.cycle_mood = Some(msg);
                    }
                }
            },
            Err(e) => {
                if field == "toggleChase" {
                    status.toggle_chase = Some(e.message);
                } else {
                    status.cycle_mood = Some(e.message);
                }
            }
        }
    }
    *st.active.lock().unwrap() = active;
    *st.shortcut_status.lock().unwrap() = status;
}

/// Troca os atalhos sem reiniciar. Se algo falhar, volta ao que estava funcionando.
pub fn apply(app: &AppHandle, new: &ShortcutsConfig) -> Result<(), ShortcutError> {
    let platform = current_platform();
    let (t, c) = validate_set(new, platform)?;
    let st = app.state::<AppState>();
    let paused = st.paused.load(Ordering::SeqCst);
    if !paused {
        let gs = app.global_shortcut();
        let (old_t, old_c) = {
            let a = st.active.lock().unwrap();
            (a.toggle.clone(), a.cycle.clone())
        };
        for o in [&old_t, &old_c].into_iter().flatten() {
            let _ = gs.unregister(o.clone());
        }
        let mut done: Vec<Shortcut> = vec![];
        let mut failed: Option<(&str, String)> = None;
        for (field, sc) in [("toggleChase", &t), ("cycleMood", &c)] {
            match gs.register(sc.clone()) {
                Ok(()) => done.push(sc.clone()),
                Err(e) => {
                    failed = Some((field, e.to_string()));
                    break;
                }
            }
        }
        if let Some((field, e)) = failed {
            for d in done {
                let _ = gs.unregister(d);
            }
            for o in [&old_t, &old_c].into_iter().flatten() {
                let _ = gs.register(o.clone());
            }
            return Err(err(Some(field), ShortcutErrorKind::InUse, &format!("Essa combinação não está disponível ({e}). Outro app pode estar usando.")));
        }
        let mut a = st.active.lock().unwrap();
        a.toggle = Some(t);
        a.cycle = Some(c);
        *st.shortcut_status.lock().unwrap() = ShortcutStatus::default();
    }
    {
        let mut cfg = st.config.lock().unwrap();
        cfg.shortcuts = new.clone();
        crate::config::save(app, &cfg);
    }
    actions::emit_settings(app);
    crate::tray::sync(app);
    Ok(())
}

/// Pausa (ou retoma) os atalhos globais, usado enquanto o usuário grava uma combinação nova.
pub fn pause(app: &AppHandle, paused: bool) {
    let st = app.state::<AppState>();
    let was = st.paused.swap(paused, Ordering::SeqCst);
    if was == paused {
        return;
    }
    if paused {
        let _ = app.global_shortcut().unregister_all();
        *st.active.lock().unwrap() = crate::state::ActiveShortcuts::default();
    } else {
        register_all(app);
    }
    actions::emit_settings(app);
    crate::tray::sync(app);
}

pub fn handler(app: &AppHandle, shortcut: &Shortcut, event: ShortcutEvent) {
    if event.state() != ShortcutState::Pressed {
        return;
    }
    let (is_toggle, is_cycle) = {
        let st = app.state::<AppState>();
        let a = st.active.lock().unwrap();
        (a.toggle.as_ref().map(|s| s.id() == shortcut.id()).unwrap_or(false), a.cycle.as_ref().map(|s| s.id() == shortcut.id()).unwrap_or(false))
    };
    // Este handler roda na main thread: o trabalho (arquivo, menu) vai para outra thread.
    let app = app.clone();
    if is_toggle {
        std::thread::spawn(move || actions::toggle_chase(&app));
    } else if is_cycle {
        std::thread::spawn(move || actions::cycle_mood(&app));
    }
}

/// Texto legível do atalho: ⇧⌘D no Mac, Ctrl+Shift+D no Windows.
pub fn format_accelerator(accel: &str, platform: Platform) -> String {
    let parts: Vec<&str> = accel.split('+').collect();
    let (key, mods) = match parts.split_last() {
        Some((k, m)) => (*k, m),
        None => return String::new(),
    };
    let has = |names: &[&str]| mods.iter().any(|m| names.contains(m));
    let key_txt = match key {
        "ArrowUp" => "↑".to_string(),
        "ArrowDown" => "↓".to_string(),
        "ArrowLeft" => "←".to_string(),
        "ArrowRight" => "→".to_string(),
        k => k.to_string(),
    };
    match platform {
        Platform::Macos => {
            let mut s = String::new();
            if has(&["Control", "Ctrl"]) {
                s.push('⌃');
            }
            if has(&["Alt", "Option"]) {
                s.push('⌥');
            }
            if has(&["Shift"]) {
                s.push('⇧');
            }
            if has(&["CommandOrControl", "CmdOrCtrl", "Command", "Cmd", "Super"]) {
                s.push('⌘');
            }
            s + &key_txt
        }
        Platform::Windows => {
            let mut v: Vec<&str> = vec![];
            if has(&["CommandOrControl", "CmdOrCtrl", "Control", "Ctrl"]) {
                v.push("Ctrl");
            }
            if has(&["Alt", "Option"]) {
                v.push("Alt");
            }
            if has(&["Shift"]) {
                v.push("Shift");
            }
            if has(&["Super", "Command", "Cmd"]) {
                v.push("Win");
            }
            v.push(&key_txt);
            v.join("+")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cfg(a: &str, b: &str) -> ShortcutsConfig {
        ShortcutsConfig { toggle_chase: a.into(), cycle_mood: b.into() }
    }

    #[test]
    fn defaults_are_valid() {
        let d = ShortcutsConfig::default();
        assert!(validate_set(&d, Platform::Macos).is_ok());
        assert!(validate_set(&d, Platform::Windows).is_ok());
    }

    #[test]
    fn all_allowed_keys_parse() {
        let mut keys: Vec<String> = ('A'..='Z').map(|c| c.to_string()).collect();
        keys.extend((0..=9).map(|n| n.to_string()));
        keys.extend((1..=12).map(|n| format!("F{n}")));
        keys.extend(["Space", "ArrowUp", "ArrowDown", "ArrowLeft", "ArrowRight"].map(String::from));
        for k in keys {
            let s = parse(&format!("Alt+{k}")).unwrap_or_else(|| panic!("não parseou {k}"));
            assert!(allowed_key(&s), "tecla {k} deveria ser permitida");
        }
    }

    #[test]
    fn rejects_without_modifier() {
        assert_eq!(validate_one("toggleChase", "Shift+D", Platform::Macos).unwrap_err().kind, ShortcutErrorKind::NoModifier);
        assert_eq!(validate_one("toggleChase", "D", Platform::Macos).unwrap_err().kind, ShortcutErrorKind::NoModifier);
    }

    #[test]
    fn rejects_duplicates_even_when_equivalent() {
        let e = validate_set(&cfg("Alt+P", "Alt+P"), Platform::Macos).unwrap_err();
        assert_eq!(e.kind, ShortcutErrorKind::Duplicate);
        let e = validate_set(&cfg("CommandOrControl+Shift+D", "Super+Shift+D"), Platform::Macos);
        if cfg!(target_os = "macos") {
            assert_eq!(e.unwrap_err().kind, ShortcutErrorKind::Duplicate);
        }
    }

    #[test]
    fn rejects_reserved() {
        assert_eq!(validate_one("toggleChase", "CommandOrControl+Q", Platform::Macos).unwrap_err().kind, ShortcutErrorKind::Reserved);
        assert_eq!(validate_one("toggleChase", "Alt+F4", Platform::Windows).unwrap_err().kind, ShortcutErrorKind::Reserved);
    }

    #[test]
    fn rejects_invalid_text() {
        assert_eq!(validate_one("toggleChase", "banana", Platform::Macos).unwrap_err().kind, ShortcutErrorKind::Invalid);
    }

    #[test]
    fn formats() {
        assert_eq!(format_accelerator("CommandOrControl+Shift+D", Platform::Macos), "⇧⌘D");
        assert_eq!(format_accelerator("CommandOrControl+Shift+D", Platform::Windows), "Ctrl+Shift+D");
        assert_eq!(format_accelerator("Control+Alt+ArrowUp", Platform::Macos), "⌃⌥↑");
    }
}
