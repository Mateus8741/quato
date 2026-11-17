use crate::events::AccessibilityStatus;

pub fn required() -> bool {
    cfg!(target_os = "macos")
}

#[cfg(target_os = "macos")]
pub fn is_trusted() -> bool {
    macos_accessibility_client::accessibility::application_is_trusted()
}

#[cfg(not(target_os = "macos"))]
pub fn is_trusted() -> bool {
    true
}

/// Mostra o diálogo do sistema e abre os Ajustes na página de Acessibilidade.
#[cfg(target_os = "macos")]
pub fn prompt() {
    let _ = macos_accessibility_client::accessibility::application_is_trusted_with_prompt();
    let _ = std::process::Command::new("open")
        .arg("x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility")
        .spawn();
}

#[cfg(not(target_os = "macos"))]
pub fn prompt() {}

pub fn status() -> AccessibilityStatus {
    AccessibilityStatus { required: required(), trusted: is_trusted() }
}
