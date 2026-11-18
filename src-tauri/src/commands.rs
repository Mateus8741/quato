use crate::actions;
use crate::events::*;
use crate::shortcuts;
use crate::sim::model::SimCommand;
use crate::sim::mood::Mood;
use tauri::AppHandle;

#[tauri::command]
pub async fn duck_ready(app: AppHandle) {
    crate::windows::mark_duck_ready();
    actions::send(&app, SimCommand::Wake);
}

#[tauri::command]
pub async fn get_duck_view(app: AppHandle) -> DuckView {
    actions::current_view(&app)
}

#[tauri::command]
pub async fn get_settings_view(app: AppHandle) -> SettingsView {
    actions::settings_view(&app)
}

#[tauri::command]
pub async fn set_mood(app: AppHandle, mood: Mood) {
    actions::set_mood(&app, mood);
}

#[tauri::command]
pub async fn set_chasing(app: AppHandle, enabled: bool) {
    actions::set_chasing(&app, enabled);
}

#[tauri::command]
pub async fn set_drag_enabled(app: AppHandle, enabled: bool) -> Result<bool, DragError> {
    actions::set_drag(&app, enabled)
}

#[tauri::command]
pub async fn release_mouse(app: AppHandle) {
    actions::release_mouse(&app);
}

#[tauri::command]
pub async fn update_shortcuts(app: AppHandle, shortcuts: ShortcutsConfig) -> Result<SettingsView, ShortcutError> {
    shortcuts::apply(&app, &shortcuts)?;
    Ok(actions::settings_view(&app))
}

#[tauri::command]
pub async fn reset_shortcuts(app: AppHandle) -> Result<SettingsView, ShortcutError> {
    shortcuts::apply(&app, &ShortcutsConfig::default())?;
    Ok(actions::settings_view(&app))
}

#[tauri::command]
pub async fn validate_shortcut(field: String, accelerator: String) -> Result<String, ShortcutError> {
    shortcuts::validate_one(&field, &accelerator, current_platform()).map(|_| accelerator)
}

#[tauri::command]
pub async fn pause_shortcuts(app: AppHandle, paused: bool) {
    shortcuts::pause(&app, paused);
}

#[tauri::command]
pub async fn accessibility_status() -> AccessibilityStatus {
    crate::accessibility::status()
}

#[tauri::command]
pub async fn request_accessibility(app: AppHandle) -> AccessibilityStatus {
    crate::accessibility::prompt();
    let s = crate::accessibility::status();
    actions::emit_settings(&app);
    s
}

#[tauri::command]
pub async fn get_look(app: AppHandle) -> Look {
    actions::current_look(&app)
}

#[tauri::command]
pub async fn set_color(app: AppHandle, color: String) {
    actions::set_color(&app, &color);
}

#[tauri::command]
pub async fn set_accessories(app: AppHandle, accessories: Vec<String>) {
    actions::set_accessories(&app, &accessories);
}

#[tauri::command]
pub async fn set_seasonal_accessories(app: AppHandle, enabled: bool) {
    actions::set_seasonal_accessories(&app, enabled);
}

#[tauri::command]
pub async fn set_free_roam(app: AppHandle, enabled: bool) {
    actions::set_free_roam(&app, enabled);
}

#[tauri::command]
pub async fn reset_home(app: AppHandle) {
    actions::reset_home(&app);
}

/// Liga/desliga o lembrete de água; `interval_min` é ajustado para 1..=720.
#[tauri::command]
pub async fn set_water(app: AppHandle, enabled: bool, interval_min: u32) -> SettingsView {
    crate::water::set(&app, enabled, interval_min);
    actions::settings_view(&app)
}

/// "Já bebi água": recomeça a contagem e esconde a plaquinha.
#[tauri::command]
pub async fn water_drank(app: AppHandle) {
    crate::water::drank(&app);
}

/// Mostra a plaquinha agora.
#[tauri::command]
pub async fn water_test(app: AppHandle) {
    crate::water::show_now(&app);
}

/// Liga/desliga um lembrete novo (postura, alongar, vista); intervalo e texto são ajustados aqui.
#[tauri::command]
pub async fn set_reminder(app: AppHandle, kind: crate::reminders::ReminderKind, enabled: bool, interval_min: u32, text: String) -> SettingsView {
    crate::reminders::set(&app, kind, enabled, interval_min, &text);
    actions::settings_view(&app)
}

/// "Já fiz": recomeça a contagem e esconde a plaquinha.
#[tauri::command]
pub async fn reminder_done(app: AppHandle, kind: crate::reminders::ReminderKind) {
    crate::reminders::done(&app, kind);
}

/// Mostra a plaquinha do lembrete agora.
#[tauri::command]
pub async fn reminder_test(app: AppHandle, kind: crate::reminders::ReminderKind) {
    crate::reminders::show_now(&app, kind);
}

/// Durações do modo foco, em minutos (ajustadas aqui).
#[tauri::command]
pub async fn set_focus_config(app: AppHandle, focus_min: u32, break_min: u32) -> SettingsView {
    crate::reminders::set_focus_config(&app, focus_min, break_min);
    actions::settings_view(&app)
}

/// Inicia o modo foco (ou para, se já estiver rodando).
#[tauri::command]
pub async fn focus_toggle(app: AppHandle) -> SettingsView {
    crate::reminders::toggle_focus(&app);
    actions::settings_view(&app)
}

#[tauri::command]
pub async fn set_dance_on_music(app: AppHandle, enabled: bool) {
    actions::set_dance_on_music(&app, enabled);
}

/// Procura atualização agora (mostra a plaquinha com o resultado) e devolve o resultado.
#[tauri::command]
pub async fn check_update(app: AppHandle) -> crate::update::CheckResult {
    let a = app.clone();
    tauri::async_runtime::spawn_blocking(move || crate::update::check(&a, true))
        .await
        .unwrap_or(crate::update::CheckResult { status: crate::update::CheckStatus::Error, update: None })
}

/// Modo "não perturbe" (esconder o pato com app em tela cheia).
#[tauri::command]
pub async fn set_dnd(app: AppHandle, enabled: bool) {
    actions::set_dnd(&app, enabled);
}

/// Abrir ao ligar o computador; devolve o estado real depois de tentar.
#[tauri::command]
pub async fn set_launch_at_login(app: AppHandle, enabled: bool) -> bool {
    crate::autostart::set(&app, enabled)
}

/// Abre a página de download da versão nova no navegador.
#[tauri::command]
pub async fn open_update(app: AppHandle) {
    crate::update::open_current(&app);
}
