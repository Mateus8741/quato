use crate::events::{Config, DuckView, ShortcutStatus};
use crate::sim::model::SimCommand;
use crate::sim::mood::Mood;
use crate::sim::vec2::Vec2;
use crate::water::WaterTimer;
use std::sync::atomic::AtomicBool;
use std::sync::mpsc::Sender;
use std::sync::Mutex;
use tauri::menu::{CheckMenuItem, MenuItem};
use tauri::Wry;
use tauri_plugin_global_shortcut::Shortcut;

#[derive(Default)]
pub struct ActiveShortcuts {
    pub toggle: Option<Shortcut>,
    pub cycle: Option<Shortcut>,
    pub crumb: Option<Shortcut>,
}

pub struct TrayHandles {
    pub chase: CheckMenuItem<Wry>,
    pub roam: CheckMenuItem<Wry>,
    pub moods: Vec<(Mood, CheckMenuItem<Wry>)>,
    pub drag: CheckMenuItem<Wry>,
    pub water: CheckMenuItem<Wry>,
    pub update: MenuItem<Wry>,
}

/// Onde o pato está agora (espaço da simulação), para o vigia de Ctrl + arrastar.
#[derive(Clone, Copy, Default)]
pub struct DuckGeom {
    pub pos: Vec2,
    pub scale: f32,
    pub shown: bool,
}

pub struct AppState {
    pub tx: Sender<SimCommand>,
    pub view: Mutex<DuckView>,
    pub config: Mutex<Config>,
    pub active: Mutex<ActiveShortcuts>,
    pub shortcut_status: Mutex<ShortcutStatus>,
    pub paused: AtomicBool,
    pub tray: Mutex<Option<TrayHandles>>,
    pub duck_geom: Mutex<DuckGeom>,
    /// Agendador do lembrete de água (relógio: `water::clock`).
    pub water: Mutex<WaterTimer>,
    /// Versão nova encontrada (None = em dia ou ainda não verificou).
    pub update: Mutex<Option<crate::update::UpdateInfo>>,
}

impl AppState {
    pub fn new(tx: Sender<SimCommand>, config: Config, view: DuckView) -> Self {
        let water = WaterTimer::new(config.water.enabled, config.water.interval_min, crate::water::clock());
        Self {
            water: Mutex::new(water),
            update: Mutex::new(None),
            tx,
            view: Mutex::new(view),
            config: Mutex::new(config),
            active: Mutex::new(ActiveShortcuts::default()),
            shortcut_status: Mutex::new(ShortcutStatus::default()),
            paused: AtomicBool::new(false),
            tray: Mutex::new(None),
            duck_geom: Mutex::new(DuckGeom::default()),
        }
    }
}
