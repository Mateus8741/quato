//! Tipos e nomes compartilhados com o TypeScript (espelho em src/shared/types.ts).
use crate::sim::mood::Mood;
use serde::{Deserialize, Serialize};

pub const EV_DUCK_VIEW: &str = "duck:view";
pub const EV_SETTINGS_CHANGED: &str = "settings:changed";
pub const EV_ACCESSIBILITY_CHANGED: &str = "accessibility:changed";
pub const EV_SETTINGS_GOTO: &str = "settings:goto";
pub const EV_DUCK_LOOK: &str = "duck:look";
pub const EV_DUCK_SIGN: &str = "duck:sign";
pub const EV_DUCK_GESTURE: &str = "duck:gesture";

/// Texto padrão da plaquinha do lembrete de água.
pub const WATER_TEXT: &str = "Hora de beber água!";
/// Quanto tempo a plaquinha do lembrete fica na tela.
pub const WATER_SIGN_MS: u32 = 15_000;
/// Quanto tempo dura a "pena do meio".
pub const FEATHER_MS: u32 = 2_500;
/// Limites do intervalo do lembrete de água, em minutos.
pub const WATER_MIN_INTERVAL: u32 = 1;
pub const WATER_MAX_INTERVAL: u32 = 720;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SignKind {
    Water,
    Update,
    /// Balão de fala aleatória (`speech.rs`).
    Speech,
}

/// Evento `duck:sign`: plaquinha na faixa de cima da janela do pato.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DuckSign {
    pub kind: SignKind,
    pub text: String,
    /// 0 = esconder agora.
    pub duration_ms: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum GestureKind {
    Feather,
}

/// Evento `duck:gesture`: gesto rápido do pato.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DuckGesture {
    pub kind: GestureKind,
    pub duration_ms: u32,
}

/// Lembrete de água.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct WaterConfig {
    pub enabled: bool,
    /// Minutos entre um lembrete e outro (1..=720).
    pub interval_min: u32,
}

impl Default for WaterConfig {
    fn default() -> Self {
        Self { enabled: false, interval_min: 45 }
    }
}

pub fn clamp_water_interval(min: u32) -> u32 {
    min.clamp(WATER_MIN_INTERVAL, WATER_MAX_INTERVAL)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum DuckMode {
    Idle,
    Walk,
    Chase,
    Grab,
    Sleep,
    /// Segurado pela pessoa (Ctrl + arrastar) para mudar de lugar.
    Held,
    /// Dançando (humor Dançando ou música tocando). Preenchido pelo motor.
    Dance,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Pace {
    Still,
    Slow,
    Fast,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Facing {
    Left,
    Right,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Platform {
    Macos,
    Windows,
}

pub fn current_platform() -> Platform {
    if cfg!(target_os = "macos") {
        Platform::Macos
    } else {
        Platform::Windows
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DuckView {
    pub mode: DuckMode,
    pub mood: Mood,
    pub facing: Facing,
    pub pace: Pace,
    pub chasing: bool,
    pub drag_enabled: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct ShortcutsConfig {
    pub toggle_chase: String,
    pub cycle_mood: String,
    /// Joga uma migalha na posição do cursor (`crumb.rs`).
    pub drop_crumb: String,
}

impl Default for ShortcutsConfig {
    fn default() -> Self {
        Self {
            toggle_chase: "CommandOrControl+Shift+D".into(),
            cycle_mood: "CommandOrControl+Shift+M".into(),
            drop_crumb: "CommandOrControl+Shift+F".into(),
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ShortcutStatus {
    pub toggle_chase: Option<String>,
    pub cycle_mood: Option<String>,
    pub drop_crumb: Option<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccessibilityStatus {
    pub required: bool,
    pub trusted: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Config {
    pub version: u32,
    pub mood: Mood,
    pub shortcuts: ShortcutsConfig,
    pub accessibility_explained: bool,
    /// Falso até a primeira abertura: o app não tem ícone no Dock, então mostra as configurações uma vez.
    pub first_run_done: bool,
    /// Cor do pato (id de `look::COLORS`), separada do humor.
    pub color: String,
    /// Acessórios (ids de `look::ACCESSORIES`), no máximo um por espaço.
    pub accessories: Vec<String>,
    /// Passeia pela tela toda em vez de ficar perto do canto.
    pub free_roam: bool,
    /// Lugar escolhido pela pessoa (espaço da simulação); `None` = canto inferior direito.
    pub home: Option<[f32; 2]>,
    /// Lembrete de água.
    pub water: WaterConfig,
    /// Dança quando tem som saindo do computador.
    pub dance_on_music: bool,
    /// Balão de fala (frequência e frases extras).
    pub speech: crate::speech::SpeechConfig,
    /// Última versão nova que já avisamos com a plaquinha.
    pub notified_update: Option<String>,
    /// Quando avisamos (segundos Unix); depois de 24 h pode lembrar de novo.
    pub notified_at: u64,
}

/// Aparência do pato: o que a janela do pato precisa saber além do `DuckView`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Look {
    pub color: String,
    pub accessories: Vec<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            version: 1,
            mood: Mood::Feliz,
            shortcuts: ShortcutsConfig::default(),
            accessibility_explained: false,
            first_run_done: false,
            color: crate::look::DEFAULT_COLOR.to_string(),
            accessories: vec![],
            free_roam: false,
            home: None,
            water: WaterConfig::default(),
            dance_on_music: true,
            speech: crate::speech::SpeechConfig::default(),
            notified_update: None,
            notified_at: 0,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SettingsView {
    pub config: Config,
    pub drag_enabled: bool,
    pub shortcut_status: ShortcutStatus,
    pub shortcuts_paused: bool,
    pub platform: Platform,
    pub accessibility: AccessibilityStatus,
    pub defaults: ShortcutsConfig,
    /// Segundos até o próximo lembrete de água (`None` = desligado).
    pub water_next_in_sec: Option<u32>,
    /// Versão instalada.
    pub app_version: String,
    /// Versão mais nova disponível, se houver.
    pub update: Option<crate::update::UpdateInfo>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ShortcutErrorKind {
    Invalid,
    NoModifier,
    Duplicate,
    Reserved,
    InUse,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ShortcutError {
    pub field: Option<String>,
    pub kind: ShortcutErrorKind,
    pub message: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum DragErrorKind {
    AccessibilityRequired,
    PointerUnavailable,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DragError {
    pub kind: DragErrorKind,
    pub message: String,
}
