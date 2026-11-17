use crate::events::Config;
use std::path::{Path, PathBuf};
use tauri::{AppHandle, Manager};

fn path(app: &AppHandle) -> Option<PathBuf> {
    app.path().app_config_dir().ok().map(|d| d.join("config.json"))
}

pub fn sanitize(mut cfg: Config) -> Config {
    cfg.color = crate::look::sanitize_color(&cfg.color);
    cfg.accessories = crate::look::sanitize_accessories(&cfg.accessories);
    cfg.water.interval_min = crate::events::clamp_water_interval(cfg.water.interval_min);
    cfg
}

pub fn parse(text: &str) -> Option<Config> {
    serde_json::from_str::<Config>(text).ok().map(sanitize)
}

pub fn serialize(cfg: &Config) -> String {
    serde_json::to_string_pretty(cfg).unwrap_or_else(|_| "{}".into())
}

pub fn load_from(file: &Path) -> Config {
    match std::fs::read_to_string(file) {
        Ok(text) => parse(&text).unwrap_or_else(|| {
            let _ = std::fs::rename(file, file.with_extension("json.bak"));
            Config::default()
        }),
        Err(_) => Config::default(),
    }
}

pub fn save_to(file: &Path, cfg: &Config) -> std::io::Result<()> {
    if let Some(dir) = file.parent() {
        std::fs::create_dir_all(dir)?;
    }
    let tmp = file.with_extension("json.tmp");
    std::fs::write(&tmp, serialize(cfg))?;
    std::fs::rename(&tmp, file)
}

pub fn load(app: &AppHandle) -> Config {
    path(app).map(|p| load_from(&p)).unwrap_or_default()
}

pub fn save(app: &AppHandle, cfg: &Config) {
    if let Some(p) = path(app) {
        if let Err(e) = save_to(&p, cfg) {
            eprintln!("[quato] não consegui salvar a config: {e}");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sim::mood::Mood;

    fn tmp(name: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("quato-test-{}-{}", name, std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        d
    }

    #[test]
    fn roundtrip() {
        let f = tmp("rt").join("config.json");
        let mut c = Config::default();
        c.mood = Mood::Raiva;
        c.shortcuts.toggle_chase = "Alt+P".into();
        save_to(&f, &c).unwrap();
        let back = load_from(&f);
        assert_eq!(back.mood, Mood::Raiva);
        assert_eq!(back.shortcuts.toggle_chase, "Alt+P");
    }

    #[test]
    fn missing_file_gives_defaults() {
        let f = tmp("missing").join("config.json");
        assert_eq!(load_from(&f).mood, Mood::Feliz);
    }

    #[test]
    fn old_config_without_new_fields_gets_defaults() {
        let c = parse(r#"{"version":1,"mood":"raiva","freeRoam":true}"#).unwrap();
        assert_eq!(c.mood, Mood::Raiva);
        assert!(c.free_roam);
        assert!(!c.water.enabled);
        assert_eq!(c.water.interval_min, 45);
        assert!(c.dance_on_music);
    }

    #[test]
    fn water_settings_roundtrip_and_interval_is_clamped() {
        let c = parse(r#"{"water":{"enabled":true,"intervalMin":30},"danceOnMusic":false}"#).unwrap();
        assert!(c.water.enabled);
        assert_eq!(c.water.interval_min, 30);
        assert!(!c.dance_on_music);
        let text = serialize(&c);
        assert!(text.contains("\"intervalMin\": 30"), "{text}");
        assert!(text.contains("\"danceOnMusic\": false"), "{text}");
        assert_eq!(parse(r#"{"water":{"enabled":true,"intervalMin":0}}"#).unwrap().water.interval_min, 1);
        assert_eq!(parse(r#"{"water":{"enabled":true,"intervalMin":99999}}"#).unwrap().water.interval_min, 720);
        // só "enabled": o intervalo fica no padrão
        assert_eq!(parse(r#"{"water":{"enabled":true}}"#).unwrap().water.interval_min, 45);
    }

    #[test]
    fn update_fields_default_and_roundtrip() {
        let c = parse(r#"{"mood":"raiva"}"#).unwrap();
        assert_eq!(c.notified_update, None);
        assert_eq!(c.notified_at, 0);
        let mut c = Config::default();
        c.notified_update = Some("0.3.0".into());
        c.notified_at = 123;
        let back = parse(&serialize(&c)).unwrap();
        assert_eq!(back.notified_update.as_deref(), Some("0.3.0"));
        assert_eq!(back.notified_at, 123);
    }

    #[test]
    fn corrupt_file_is_backed_up() {
        let d = tmp("corrupt");
        std::fs::create_dir_all(&d).unwrap();
        let f = d.join("config.json");
        std::fs::write(&f, "{ isto não é json").unwrap();
        let c = load_from(&f);
        assert_eq!(c.mood, Mood::Feliz);
        assert!(d.join("config.json.bak").exists());
    }
}
