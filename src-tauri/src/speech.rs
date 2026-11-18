//! Fala do pato: de vez em quando aparece um balão com uma frase aleatória, de acordo com o humor.
//!
//! Tudo que decide (frequência, escolha da frase, quando pode falar) é função pura com o sorteio
//! injetado, para testar sem relógio nem sorte. O balão reaproveita o evento `duck:sign` da
//! plaquinha, com `kind: "speech"`.
use crate::events::*;
use crate::sim::mood::Mood;
use crate::state::AppState;
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;
use std::time::Duration;
use tauri::{AppHandle, Emitter, Manager};

/// Quanto tempo o balão fica na tela.
pub const SPEECH_MS: u32 = 5_000;
/// Limites das frases do usuário (o balão tem 2 linhas de ~18 letras).
pub const MAX_EXTRA: usize = 30;
pub const MAX_PHRASE_CHARS: usize = 36;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub enum SpeechFrequency {
    Never,
    #[default]
    Rare,
    Sometimes,
    Always,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct SpeechConfig {
    pub frequency: SpeechFrequency,
    /// Frases extras da pessoa (valem para qualquer humor).
    pub extra: Vec<String>,
}

impl Default for SpeechConfig {
    fn default() -> Self {
        Self { frequency: SpeechFrequency::Rare, extra: vec![] }
    }
}

/// Frases prontas de cada humor (só letras, números e `! ? . , - : '`, que a fonte do balão desenha).
pub fn builtin(mood: Mood) -> &'static [&'static str] {
    match mood {
        Mood::Feliz => &["Que dia lindo!", "Quá quá quá!", "Estou feliz!", "Adoro você!", "Bora passear?", "Tudo ótimo por aqui!", "Hoje é um bom dia!", "Cadê o pão?"],
        Mood::Raiva => &["Quá! Sai da frente!", "Estou furioso!", "Quem mexeu no meu lago?", "Não me provoque!", "Grrr!", "Devolve minha água!", "Hoje não, hein!", "Quá!!"],
        Mood::Triste => &["Hoje está difícil...", "Ninguém brinca comigo.", "Sinto falta do lago.", "Quá... só quá.", "Queria um abraço.", "Ninguém me chama pra nada.", "Suspiro."],
        Mood::Sozinho => &["Tem alguém aí?", "Está tão quieto aqui.", "Fica comigo?", "Cadê todo mundo?", "Eu aqui, sozinho.", "Me faz companhia?", "Oi? Oi?"],
        Mood::Sono => &["Só mais cinco minutinhos.", "Que soneca boa.", "Preciso de um cochilo.", "Bocejo...", "Quero minha caminha.", "Zzz... quá.", "Estou quase dormindo."],
        Mood::Danca => &["Bora dançar!", "Põe um som aí!", "Mexe esse bico!", "Que ritmo!", "Quá quá no ritmo!", "Ninguém me segura!", "É hora do show!"],
        Mood::Lagarto => &["Sou um lagartinho!", "Cadê a mosca?", "Sol bom, hein.", "Sssss!", "Olha minha língua!", "Não sou pato!", "Rastejando por aí."],
        Mood::Duvida => &["Será que sim?", "Hum, será?", "Quem sabe?", "Será que deixei o fogão ligado?", "Pato ou lagarto?", "Isso é um pão?", "Eu hein?"],
    }
}

/// Arruma a lista de frases do usuário: tira espaços sobrando, vazias, repetidas (sem diferenciar
/// maiúsculas), corta no tamanho máximo e limita a quantidade.
pub fn sanitize_extra(list: &[String]) -> Vec<String> {
    let mut out: Vec<String> = vec![];
    for raw in list {
        let clean: String = raw.chars().filter(|c| !c.is_control()).collect::<String>().split_whitespace().collect::<Vec<_>>().join(" ");
        let clean: String = clean.chars().take(MAX_PHRASE_CHARS).collect::<String>().trim().to_string();
        if clean.is_empty() || out.iter().any(|p| p.to_lowercase() == clean.to_lowercase()) {
            continue;
        }
        out.push(clean);
        if out.len() >= MAX_EXTRA {
            break;
        }
    }
    out
}

/// Escolhe a frase. `rng` devolve números em [0, 1). Com frases extras, metade das vezes sai uma
/// delas (senão se perdiam no meio das prontas). Nunca repete a `last` se houver outra opção.
pub fn pick<R: FnMut() -> f64>(mood: Mood, extra: &[String], last: Option<&str>, rng: &mut R) -> Option<String> {
    let base: Vec<&str> = builtin(mood).to_vec();
    let own: Vec<&str> = extra.iter().map(String::as_str).collect();
    let use_own = !own.is_empty() && (base.is_empty() || rng() < 0.5);
    let mut pool = if use_own { own } else { base };
    if pool.len() > 1 {
        if let Some(l) = last {
            pool.retain(|p| *p != l);
        }
    }
    if pool.is_empty() {
        return None;
    }
    let i = ((rng() * pool.len() as f64) as usize).min(pool.len() - 1);
    Some(pool[i].to_string())
}

/// Intervalo (mín, máx) em segundos entre uma fala e outra; `None` = nunca.
pub fn delay_range(f: SpeechFrequency) -> Option<(f64, f64)> {
    match f {
        SpeechFrequency::Never => None,
        SpeechFrequency::Rare => Some((360.0, 720.0)),
        SpeechFrequency::Sometimes => Some((90.0, 180.0)),
        SpeechFrequency::Always => Some((20.0, 40.0)),
    }
}

/// Espera até a próxima fala para um sorteio `r` em [0, 1).
pub fn next_delay(f: SpeechFrequency, r: f64) -> Option<f64> {
    delay_range(f).map(|(lo, hi)| lo + r.clamp(0.0, 1.0) * (hi - lo))
}

/// Dormindo, no colo ou arrastando o mouse ele não fala; com plaquinha de lembrete na tela também não.
pub fn can_speak(mode: DuckMode, sign_busy: bool) -> bool {
    !sign_busy && !matches!(mode, DuckMode::Sleep | DuckMode::Held | DuckMode::Grab)
}

/// Agendador puro (segundos num relógio qualquer).
#[derive(Clone, Debug, PartialEq)]
pub struct SpeechTimer {
    freq: SpeechFrequency,
    due: f64,
}

impl SpeechTimer {
    pub fn new(freq: SpeechFrequency, now: f64, r: f64) -> Self {
        Self { freq, due: now + next_delay(freq, r).unwrap_or(f64::INFINITY) }
    }

    pub fn frequency(&self) -> SpeechFrequency {
        self.freq
    }

    /// `true` quando é hora de falar (e já agenda a próxima).
    pub fn poll(&mut self, now: f64, r: f64) -> bool {
        if now < self.due {
            return false;
        }
        self.due = now + next_delay(self.freq, r).unwrap_or(f64::INFINITY);
        self.freq != SpeechFrequency::Never
    }
}

/// Até quando há uma plaquinha de lembrete na tela (bits de f64, relógio de `water::clock`).
static SIGN_UNTIL: AtomicU64 = AtomicU64::new(0);
static LAST: Mutex<Option<String>> = Mutex::new(None);

/// Chamado quando uma plaquinha (água, atualização) é mostrada ou escondida (`duration_ms == 0`).
pub fn note_sign(duration_ms: u32) {
    let until = if duration_ms == 0 { 0.0 } else { crate::water::clock() + duration_ms as f64 / 1000.0 };
    SIGN_UNTIL.store(until.to_bits(), Ordering::SeqCst);
}

fn sign_busy() -> bool {
    f64::from_bits(SIGN_UNTIL.load(Ordering::SeqCst)) > crate::water::clock()
}

fn say(app: &AppHandle) {
    let (mood, extra) = {
        let st = app.state::<AppState>();
        let cfg = st.config.lock().unwrap();
        (cfg.mood, cfg.speech.extra.clone())
    };
    let mut rng = fastrand::Rng::new();
    let last = LAST.lock().unwrap().clone();
    let Some(text) = pick(mood, &extra, last.as_deref(), &mut || rng.f64()) else { return };
    *LAST.lock().unwrap() = Some(text.clone());
    let _ = app.emit(EV_DUCK_SIGN, DuckSign { kind: SignKind::Speech, text, duration_ms: SPEECH_MS });
}

/// Botão "Testar fala": fala agora, sem olhar a frequência.
pub fn say_now(app: &AppHandle) {
    say(app);
}

/// Muda a frequência e as frases extras (já arrumadas aqui). Salva na config.
pub fn set(app: &AppHandle, frequency: SpeechFrequency, extra: &[String]) {
    {
        let st = app.state::<AppState>();
        let mut cfg = st.config.lock().unwrap();
        cfg.speech = SpeechConfig { frequency, extra: sanitize_extra(extra) };
        crate::config::save(app, &cfg);
    }
    crate::actions::emit_settings(app);
}

pub fn spawn(app: AppHandle) {
    std::thread::Builder::new()
        .name("quato-speech".into())
        .spawn(move || {
            let mut rng = fastrand::Rng::new();
            let first = app.state::<AppState>().config.lock().unwrap().speech.frequency;
            let mut timer = SpeechTimer::new(first, crate::water::clock(), rng.f64());
            loop {
                std::thread::sleep(Duration::from_secs(1));
                let now = crate::water::clock();
                let freq = app.state::<AppState>().config.lock().unwrap().speech.frequency;
                if freq != timer.frequency() {
                    timer = SpeechTimer::new(freq, now, rng.f64());
                    continue;
                }
                if timer.poll(now, rng.f64()) {
                    let mode = app.state::<AppState>().view.lock().unwrap().mode;
                    if can_speak(mode, sign_busy()) {
                        say(&app);
                    }
                }
            }
        })
        .expect("não consegui iniciar a fala do pato");
}

#[cfg(test)]
mod tests {
    use super::*;

    fn seq(v: Vec<f64>) -> impl FnMut() -> f64 {
        let mut i = 0;
        move || {
            let x = v[i % v.len()];
            i += 1;
            x
        }
    }

    #[test]
    fn every_mood_has_phrases_the_font_can_draw() {
        for m in Mood::ALL {
            assert!(builtin(m).len() >= 5, "{m:?}");
            for p in builtin(m) {
                assert!(p.chars().count() <= MAX_PHRASE_CHARS, "{p}");
                assert!(p.chars().all(|c| c.is_alphanumeric() || " !?.,-:'".contains(c)), "caractere sem glifo em {p:?}");
            }
        }
    }

    #[test]
    fn pick_is_deterministic_with_injected_rng() {
        let n = builtin(Mood::Raiva).len();
        assert_eq!(pick(Mood::Raiva, &[], None, &mut seq(vec![0.0])).unwrap(), builtin(Mood::Raiva)[0]);
        assert_eq!(pick(Mood::Raiva, &[], None, &mut seq(vec![0.999])).unwrap(), builtin(Mood::Raiva)[n - 1]);
        // sorteio 1.0 (fora do contrato) não estoura o índice
        assert_eq!(pick(Mood::Raiva, &[], None, &mut seq(vec![1.0])).unwrap(), builtin(Mood::Raiva)[n - 1]);
    }

    #[test]
    fn pick_depends_on_mood() {
        let a = pick(Mood::Feliz, &[], None, &mut seq(vec![0.0])).unwrap();
        let b = pick(Mood::Triste, &[], None, &mut seq(vec![0.0])).unwrap();
        assert!(builtin(Mood::Feliz).contains(&a.as_str()));
        assert!(builtin(Mood::Triste).contains(&b.as_str()));
        assert_ne!(a, b);
    }

    #[test]
    fn extra_phrases_are_picked_about_half_the_time() {
        let extra = vec!["Minha frase".to_string()];
        // primeiro sorteio < 0.5 escolhe das extras
        assert_eq!(pick(Mood::Feliz, &extra, None, &mut seq(vec![0.1, 0.9])).unwrap(), "Minha frase");
        // >= 0.5 escolhe das prontas
        let p = pick(Mood::Feliz, &extra, None, &mut seq(vec![0.7, 0.0])).unwrap();
        assert_eq!(p, builtin(Mood::Feliz)[0]);
    }

    #[test]
    fn pick_avoids_repeating_the_last_phrase() {
        let first = builtin(Mood::Feliz)[0];
        let p = pick(Mood::Feliz, &[], Some(first), &mut seq(vec![0.0])).unwrap();
        assert_ne!(p, first);
        // com uma frase só, repete mesmo assim
        let one = vec!["Só uma".to_string()];
        assert_eq!(pick(Mood::Feliz, &one, Some("Só uma"), &mut seq(vec![0.0, 0.0])).unwrap(), "Só uma");
    }

    #[test]
    fn sanitize_extra_cleans_dedups_and_limits() {
        let long = "a".repeat(100);
        let list: Vec<String> = ["  oi   mundo  ", "", "   ", "OI MUNDO", "ok\u{0007}", &long].iter().map(|s| s.to_string()).collect();
        let out = sanitize_extra(&list);
        assert_eq!(out[0], "oi mundo");
        assert_eq!(out[1], "ok");
        assert_eq!(out[2].chars().count(), MAX_PHRASE_CHARS);
        assert_eq!(out.len(), 3);
        let many: Vec<String> = (0..100).map(|i| format!("frase {i}")).collect();
        assert_eq!(sanitize_extra(&many).len(), MAX_EXTRA);
    }

    #[test]
    fn delays_follow_frequency() {
        assert_eq!(next_delay(SpeechFrequency::Never, 0.5), None);
        let rare = next_delay(SpeechFrequency::Rare, 0.0).unwrap();
        let often = next_delay(SpeechFrequency::Always, 1.0).unwrap();
        assert!(rare >= 360.0 && often <= 40.0);
        for (lo, hi) in [SpeechFrequency::Rare, SpeechFrequency::Sometimes, SpeechFrequency::Always].map(|f| delay_range(f).unwrap()) {
            assert!(lo < hi);
        }
        assert!(delay_range(SpeechFrequency::Rare).unwrap().0 > delay_range(SpeechFrequency::Sometimes).unwrap().1);
        assert!(delay_range(SpeechFrequency::Sometimes).unwrap().0 > delay_range(SpeechFrequency::Always).unwrap().1);
        assert_eq!(SpeechConfig::default().frequency, SpeechFrequency::Rare);
    }

    #[test]
    fn timer_fires_then_reschedules_and_never_stays_quiet() {
        let mut t = SpeechTimer::new(SpeechFrequency::Always, 0.0, 0.0);
        assert!(!t.poll(19.0, 0.0));
        assert!(t.poll(20.0, 0.0));
        assert!(!t.poll(21.0, 0.0), "não repete logo em seguida");
        assert!(t.poll(40.0, 0.0));
        let mut n = SpeechTimer::new(SpeechFrequency::Never, 0.0, 0.0);
        assert!(!n.poll(1e9, 0.0));
    }

    #[test]
    fn silent_while_sleeping_held_grabbing_or_with_a_sign() {
        assert!(can_speak(DuckMode::Idle, false));
        assert!(can_speak(DuckMode::Walk, false));
        assert!(can_speak(DuckMode::Dance, false));
        for m in [DuckMode::Sleep, DuckMode::Held, DuckMode::Grab] {
            assert!(!can_speak(m, false));
        }
        assert!(!can_speak(DuckMode::Idle, true), "sem balão durante lembretes");
    }

    #[test]
    fn config_roundtrip_and_payload() {
        let c: SpeechConfig = serde_json::from_str(r#"{"frequency":"sometimes","extra":["oi"]}"#).unwrap();
        assert_eq!(c.frequency, SpeechFrequency::Sometimes);
        let d: SpeechConfig = serde_json::from_str("{}").unwrap();
        assert_eq!(d, SpeechConfig::default());
        let s = DuckSign { kind: SignKind::Speech, text: "Oi".into(), duration_ms: 5000 };
        assert_eq!(serde_json::to_string(&s).unwrap(), r#"{"kind":"speech","text":"Oi","durationMs":5000}"#);
    }
}
