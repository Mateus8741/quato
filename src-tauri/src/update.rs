//! Aviso de atualização: busca `version.json` no site e avisa quando há versão mais nova.
//!
//! Sem crate HTTP: usa o `curl` do sistema (macOS e Windows 10+). Nada é baixado nem instalado
//! pelo app: o botão só abre a página de download no navegador.
use crate::events::*;
use crate::state::AppState;
use serde::{Deserialize, Serialize};
use std::process::Command;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tauri::{AppHandle, Emitter, Manager};

pub const MANIFEST_URL: &str = "https://quato.codewaveds.com.br/version.json";
pub const DOWNLOAD_URL: &str = "https://quato.codewaveds.com.br/#baixar";
const TRUSTED_HOST: &str = "quato.codewaveds.com.br";
pub const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");
const FIRST_CHECK_DELAY: Duration = Duration::from_secs(20);
const CHECK_EVERY: Duration = Duration::from_secs(6 * 60 * 60);
/// Passado esse tempo, a mesma versão pode ser lembrada de novo.
pub const REMIND_AFTER_SEC: u64 = 24 * 60 * 60;
const SIGN_MS: u32 = 15_000;
const MAX_BODY: usize = 64 * 1024;
const MAX_NOTES: usize = 300;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateInfo {
    pub version: String,
    pub notes: String,
    pub url: String,
}

#[derive(Deserialize)]
struct Manifest {
    version: String,
    #[serde(default)]
    notes: String,
    #[serde(default)]
    url: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum CheckStatus {
    UpToDate,
    Available,
    Error,
}

/// Resposta do comando `check_update`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CheckResult {
    pub status: CheckStatus,
    pub update: Option<UpdateInfo>,
}

// ---------- lógica pura ----------

/// `x.y.z` numérico (aceita um `v` na frente). Pré-lançamento (`-beta`), build (`+x`) e lixo dão `None`.
pub fn parse_version(s: &str) -> Option<(u64, u64, u64)> {
    let s = s.trim();
    let s = s.strip_prefix('v').unwrap_or(s);
    let mut it = s.split('.');
    let mut next = || {
        let p = it.next()?;
        if p.is_empty() || p.len() > 9 || !p.bytes().all(|b| b.is_ascii_digit()) {
            return None;
        }
        p.parse::<u64>().ok()
    };
    let v = (next()?, next()?, next()?);
    if it.next().is_some() {
        return None;
    }
    Some(v)
}

/// `remote` é estritamente mais novo que `current`? Versão inválida nunca é mais nova.
pub fn is_newer(remote: &str, current: &str) -> bool {
    match (parse_version(remote), parse_version(current)) {
        (Some(r), Some(c)) => r > c,
        _ => false,
    }
}

/// Só `https://quato.codewaveds.com.br/...` (host exato, sem porta nem usuário) e sem caracteres
/// que um shell trataria de forma especial; qualquer outra coisa vira a página de download.
pub fn safe_url(url: &str) -> String {
    let u = url.trim();
    let ok_chars = u.bytes().all(|b| b.is_ascii_alphanumeric() || b"-._~:/?#=%+".contains(&b));
    let Some(rest) = u.strip_prefix("https://") else { return DOWNLOAD_URL.into() };
    let host = rest.split(['/', '?', '#']).next().unwrap_or("");
    if ok_chars && host.eq_ignore_ascii_case(TRUSTED_HOST) {
        u.to_string()
    } else {
        DOWNLOAD_URL.into()
    }
}

/// Lê o `version.json`. `None` se for inválido ou a versão não for `x.y.z`.
pub fn parse_manifest(text: &str) -> Option<UpdateInfo> {
    let m: Manifest = serde_json::from_str(text).ok()?;
    parse_version(&m.version)?;
    let notes: String = m.notes.trim().chars().take(MAX_NOTES).collect();
    Some(UpdateInfo { version: m.version.trim().trim_start_matches('v').to_string(), notes, url: safe_url(&m.url) })
}

/// Avisa com a plaquinha? Uma vez por versão; depois de 24 h pode lembrar de novo.
pub fn should_notify(notified: Option<&str>, notified_at: u64, version: &str, now: u64) -> bool {
    match notified {
        Some(v) if v == version => now.saturating_sub(notified_at) >= REMIND_AFTER_SEC,
        _ => true,
    }
}

pub fn sign_text(version: &str) -> String {
    format!("Nova versão {version}!")
}

// ---------- rede e sistema ----------

fn now_secs() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0)
}

fn manifest_url() -> String {
    if cfg!(debug_assertions) {
        if let Ok(u) = std::env::var("QUATO_UPDATE_URL") {
            if !u.trim().is_empty() {
                return u;
            }
        }
    }
    MANIFEST_URL.into()
}

fn quiet(cmd: &mut Command) -> &mut Command {
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x0800_0000); // CREATE_NO_WINDOW
    }
    cmd
}

/// Baixa o texto da URL com o `curl` do sistema.
pub fn fetch(url: &str) -> Result<String, String> {
    let mut cmd = Command::new("curl");
    cmd.args(["-fsSL", "--max-time", "15", "-A", &format!("Quato/{CURRENT_VERSION}"), url]);
    let out = quiet(&mut cmd).output().map_err(|e| format!("curl: {e}"))?;
    if !out.status.success() {
        return Err(format!("curl saiu com {}", out.status));
    }
    if out.stdout.len() > MAX_BODY {
        return Err("resposta grande demais".into());
    }
    String::from_utf8(out.stdout).map_err(|e| e.to_string())
}

/// Busca o manifesto: `Ok(Some)` = há versão mais nova; `Ok(None)` = está em dia.
pub fn fetch_update(url: &str, current: &str) -> Result<Option<UpdateInfo>, String> {
    let text = fetch(url)?;
    let info = parse_manifest(&text).ok_or("version.json inválido")?;
    Ok(is_newer(&info.version, current).then_some(info))
}

/// Abre a URL (já validada aqui) no navegador padrão; a URL vai como argumento, sem shell.
pub fn open_in_browser(url: &str) {
    let url = safe_url(url);
    #[cfg(target_os = "macos")]
    let mut cmd = {
        let mut c = Command::new("open");
        c.arg(&url);
        c
    };
    #[cfg(windows)]
    let mut cmd = {
        let mut c = Command::new("cmd");
        c.args(["/C", "start", ""]).arg(&url);
        c
    };
    #[cfg(not(any(target_os = "macos", windows)))]
    let mut cmd = {
        let mut c = Command::new("xdg-open");
        c.arg(&url);
        c
    };
    if let Err(e) = quiet(&mut cmd).spawn() {
        eprintln!("[quato] não consegui abrir o navegador: {e}");
    }
}

// ---------- integração com o app ----------

pub fn current(app: &AppHandle) -> Option<UpdateInfo> {
    app.state::<AppState>().update.lock().unwrap().clone()
}

fn sign(app: &AppHandle, text: String) {
    crate::speech::note_sign(SIGN_MS);
    let _ = app.emit(EV_DUCK_SIGN, DuckSign { kind: SignKind::Update, text, duration_ms: SIGN_MS });
}

/// Verifica agora. `manual` = a pessoa pediu: sempre mostra a plaquinha com o resultado.
pub fn check(app: &AppHandle, manual: bool) -> CheckResult {
    let result = fetch_update(&manifest_url(), CURRENT_VERSION);
    let out = match result {
        Err(e) => {
            eprintln!("[quato] verificação de atualização falhou: {e}");
            if manual {
                sign(app, "Não consegui verificar agora".into());
            }
            return CheckResult { status: CheckStatus::Error, update: current(app) };
        }
        Ok(None) => {
            *app.state::<AppState>().update.lock().unwrap() = None;
            if manual {
                sign(app, "Você está na versão mais recente".into());
            }
            CheckResult { status: CheckStatus::UpToDate, update: None }
        }
        Ok(Some(info)) => {
            let st = app.state::<AppState>();
            *st.update.lock().unwrap() = Some(info.clone());
            let now = now_secs();
            let notify = {
                let mut cfg = st.config.lock().unwrap();
                let n = manual || should_notify(cfg.notified_update.as_deref(), cfg.notified_at, &info.version, now);
                if n && !(manual && cfg.notified_update.as_deref() == Some(&info.version)) {
                    cfg.notified_update = Some(info.version.clone());
                    cfg.notified_at = now;
                    crate::config::save(app, &cfg);
                }
                n
            };
            if notify {
                sign(app, sign_text(&info.version));
            }
            CheckResult { status: CheckStatus::Available, update: Some(info) }
        }
    };
    crate::tray::sync(app);
    crate::actions::emit_settings(app);
    out
}

/// Clique no item do menu: baixa se há versão nova; senão procura agora.
pub fn on_tray_click(app: &AppHandle) {
    match current(app) {
        Some(u) => open_in_browser(&u.url),
        None => {
            check(app, true);
        }
    }
}

/// Abre a página da versão nova (ou a de download).
pub fn open_current(app: &AppHandle) {
    open_in_browser(&current(app).map(|u| u.url).unwrap_or_else(|| DOWNLOAD_URL.into()));
}

pub fn spawn(app: AppHandle) {
    std::thread::Builder::new()
        .name("quato-update".into())
        .spawn(move || {
            std::thread::sleep(FIRST_CHECK_DELAY);
            loop {
                check(&app, false);
                std::thread::sleep(CHECK_EVERY);
            }
        })
        .expect("não consegui iniciar a verificação de atualização");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn versions_parse_and_compare() {
        assert_eq!(parse_version("0.2.0"), Some((0, 2, 0)));
        assert_eq!(parse_version("v1.10.3"), Some((1, 10, 3)));
        for bad in ["", "1", "1.2", "1.2.3.4", "1.2.3-beta", "1.2.3+b", "a.b.c", "1..3", "-1.0.0", "1.2.x", "99999999999.0.0"] {
            assert_eq!(parse_version(bad), None, "{bad}");
        }
        assert!(is_newer("0.10.0", "0.9.0"));
        assert!(is_newer("1.0.0", "0.99.99"));
        assert!(is_newer("0.2.1", "0.2.0"));
        assert!(!is_newer("0.2.0", "0.2.0"));
        assert!(!is_newer("0.1.9", "0.2.0"));
        assert!(!is_newer("1.0.0-rc1", "0.2.0"));
        assert!(!is_newer("lixo", "0.2.0"));
        assert!(!is_newer("1.0.0", "lixo"));
    }

    #[test]
    fn manifest_parses_and_ignores_extra_fields() {
        let i = parse_manifest(r#"{"version":"0.3.0","notes":"Novidades","url":"https://quato.codewaveds.com.br/#baixar","extra":1}"#).unwrap();
        assert_eq!(i, UpdateInfo { version: "0.3.0".into(), notes: "Novidades".into(), url: DOWNLOAD_URL.into() });
        let j = parse_manifest(r#"{"version":"0.3.0"}"#).unwrap();
        assert_eq!(j.url, DOWNLOAD_URL);
        assert_eq!(j.notes, "");
        assert!(parse_manifest(r#"{"version":"abc"}"#).is_none());
        assert!(parse_manifest(r#"{"version":"1.0.0-beta"}"#).is_none());
        assert!(parse_manifest("<html>").is_none());
        assert!(parse_manifest(r#"{"notes":"x"}"#).is_none());
        let long = format!(r#"{{"version":"1.0.0","notes":"{}"}}"#, "a".repeat(1000));
        assert_eq!(parse_manifest(&long).unwrap().notes.chars().count(), MAX_NOTES);
        // URL maliciosa no JSON vira a página padrão
        assert_eq!(parse_manifest(r#"{"version":"1.0.0","url":"https://evil.com/x"}"#).unwrap().url, DOWNLOAD_URL);
    }

    #[test]
    fn url_validation() {
        let ok = "https://quato.codewaveds.com.br/#baixar";
        assert_eq!(safe_url(ok), ok);
        assert_eq!(safe_url("https://quato.codewaveds.com.br"), "https://quato.codewaveds.com.br");
        assert_eq!(safe_url("https://QUATO.codewaveds.com.br/a?b=1"), "https://QUATO.codewaveds.com.br/a?b=1");
        for bad in [
            "https://quato.codewaveds.com.br.evil.com/",
            "https://evil.com/quato.codewaveds.com.br",
            "https://evil.com#@quato.codewaveds.com.br",
            "https://quato.codewaveds.com.br@evil.com/",
            "https://quato.codewaveds.com.br:8443/",
            "https://sub.quato.codewaveds.com.br/",
            "http://quato.codewaveds.com.br/",
            "javascript:alert(1)",
            "file:///etc/passwd",
            "//quato.codewaveds.com.br/",
            "https://quato.codewaveds.com.br/a&calc",
            "https://quato.codewaveds.com.br/a b",
            "https://quato.codewaveds.com.br/\"x",
            "https://quato.codewaveds.com.br\\@evil.com",
            "",
        ] {
            assert_eq!(safe_url(bad), DOWNLOAD_URL, "{bad}");
        }
    }

    #[test]
    fn notify_once_per_version_or_after_24h() {
        let t = 1_000_000;
        assert!(should_notify(None, 0, "0.3.0", t));
        assert!(should_notify(Some("0.2.5"), t, "0.3.0", t));
        assert!(!should_notify(Some("0.3.0"), t, "0.3.0", t + 60));
        assert!(!should_notify(Some("0.3.0"), t, "0.3.0", t + REMIND_AFTER_SEC - 1));
        assert!(should_notify(Some("0.3.0"), t, "0.3.0", t + REMIND_AFTER_SEC));
        // relógio voltou no tempo: não repete
        assert!(!should_notify(Some("0.3.0"), t, "0.3.0", t - 5));
    }

    #[test]
    fn sign_text_has_version() {
        assert_eq!(sign_text("0.3.0"), "Nova versão 0.3.0!");
    }

    /// Rede de verdade: `QUATO_TEST_URL=http://127.0.0.1:4195/version.json cargo test net -- --ignored`
    #[test]
    #[ignore]
    fn net_fetch_from_local_server() {
        let url = std::env::var("QUATO_TEST_URL").expect("defina QUATO_TEST_URL");
        let got = fetch_update(&url, "0.2.0").unwrap().expect("devia haver versão nova");
        assert_eq!(got.version, "9.9.9");
        assert!(fetch_update(&url, "9.9.9").unwrap().is_none());
        assert!(fetch_update("http://127.0.0.1:1/nada.json", "0.2.0").is_err());
    }
}
