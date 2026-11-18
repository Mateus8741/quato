//! Modo "não perturbe": esconde o pato (e a janela de efeitos) quando há um app em tela cheia
//! no monitor dele, e mostra de novo quando a tela cheia acaba. Só visibilidade: enquanto
//! escondido, a thread da simulação não dá passos (nada de perseguir nem arrastar o mouse) e o
//! vigia de Ctrl + arrastar não faz nada; o lembrete de água e o resto seguem normais.
//!
//! A decisão é pura (`covers`, `fullscreen_on`, `DndState`) e testável. Ler o sistema fica em
//! `fullscreen_now` (macOS: lista de janelas do CoreGraphics, sem nenhuma permissão nova;
//! Windows: `SHQueryUserNotificationState`).
use crate::platform::MonitorSim;
use crate::sim::vec2::{Rect, Vec2};
use crate::state::AppState;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};
use tauri::{AppHandle, Manager};

static HIDDEN: AtomicBool = AtomicBool::new(false);

/// O pato está escondido pelo modo "não perturbe"? (lido pelo vigia de Ctrl + arrastar)
pub fn is_hidden() -> bool {
    HIDDEN.load(Ordering::SeqCst)
}

/// Intervalo entre duas olhadas no sistema.
const POLL: Duration = Duration::from_millis(750);
/// Leituras seguidas iguais para esconder (evita piscar ao trocar de app).
const HIDE_AFTER: u8 = 2;
/// Leituras seguidas para mostrar de novo.
const SHOW_AFTER: u8 = 1;
/// Folga em pixels/pontos para dizer que uma janela cobre o monitor inteiro.
const TOLERANCE: f32 = 2.0;

/// Uma janela cobre o monitor inteiro (com folga)?
pub fn covers(win: Rect, mon: Rect, tol: f32) -> bool {
    win.x <= mon.x + tol && win.y <= mon.y + tol && win.right() >= mon.right() - tol && win.bottom() >= mon.bottom() - tol
}

/// Alguma das janelas (já filtradas: só as normais de outros apps) cobre o monitor?
pub fn fullscreen_on(windows: &[Rect], mon: Rect) -> bool {
    windows.iter().any(|w| covers(*w, mon, TOLERANCE))
}

/// Windows: a pessoa está numa tela cheia / jogo / apresentação? (valores de QUERY_USER_NOTIFICATION_STATE)
/// 2 = QUNS_BUSY (tela cheia), 3 = QUNS_RUNNING_D3D_FULL_SCREEN, 4 = QUNS_PRESENTATION_MODE.
pub fn windows_state_is_fullscreen(state: i32) -> bool {
    matches!(state, 2..=4)
}

/// Decide quando esconder/mostrar, com um pouco de histerese. Desligar a opção mostra na hora.
#[derive(Debug, Default)]
pub struct DndState {
    hidden: bool,
    streak: u8,
}

impl DndState {
    pub fn hidden(&self) -> bool {
        self.hidden
    }

    /// Uma leitura nova; devolve se deve estar escondido.
    pub fn update(&mut self, enabled: bool, fullscreen: bool) -> bool {
        if !enabled {
            self.hidden = false;
            self.streak = 0;
            return false;
        }
        if fullscreen == self.hidden {
            self.streak = 0;
        } else {
            self.streak += 1;
            let need = if fullscreen { HIDE_AFTER } else { SHOW_AFTER };
            if self.streak >= need {
                self.hidden = fullscreen;
                self.streak = 0;
            }
        }
        self.hidden
    }
}

/// Estado do vigia dentro da thread da simulação.
pub struct Watcher {
    state: DndState,
    checked: Option<Instant>,
}

impl Watcher {
    pub fn new() -> Self {
        Self { state: DndState::default(), checked: None }
    }

    /// Chamado todo tick; só olha o sistema a cada `POLL`. Devolve se o pato deve estar escondido.
    pub fn poll(&mut self, app: &AppHandle, all: &[MonitorSim], duck: Vec2) -> bool {
        let due = self.checked.map(|t| t.elapsed() >= POLL).unwrap_or(true);
        if due {
            self.checked = Some(Instant::now());
            let enabled = app.state::<AppState>().config.lock().map(|c| c.dnd).unwrap_or(false);
            let fullscreen = enabled && fullscreen_now(all, duck);
            self.state.update(enabled, fullscreen);
            HIDDEN.store(self.state.hidden(), Ordering::SeqCst);
        }
        self.state.hidden()
    }
}

fn fullscreen_now(all: &[MonitorSim], duck: Vec2) -> bool {
    #[cfg(target_os = "macos")]
    {
        let mon = crate::platform::monitor_at(all, duck).or_else(|| all.first().copied());
        match mon {
            Some(m) => fullscreen_on(&mac::other_app_windows(), m.rect),
            None => false,
        }
    }
    #[cfg(windows)]
    {
        let _ = (all, duck);
        win::fullscreen()
    }
    #[cfg(not(any(target_os = "macos", windows)))]
    {
        let _ = (all, duck);
        false
    }
}

#[cfg(target_os = "macos")]
mod mac {
    use super::Rect;
    use core_foundation::base::{CFType, TCFType};
    use core_foundation::dictionary::{CFDictionary, CFDictionaryRef};
    use core_foundation::number::CFNumber;
    use core_foundation::string::{CFString, CFStringRef};
    use core_graphics::window::{
        copy_window_info, kCGNullWindowID, kCGWindowAlpha, kCGWindowBounds, kCGWindowLayer, kCGWindowListExcludeDesktopElements,
        kCGWindowListOptionOnScreenOnly, kCGWindowOwnerPID,
    };

    fn num(d: &CFDictionary<CFString, CFType>, key: CFStringRef) -> Option<f64> {
        let k = unsafe { CFString::wrap_under_get_rule(key) };
        d.find(&k)?.downcast::<CFNumber>()?.to_f64()
    }

    fn bounds(d: &CFDictionary<CFString, CFType>) -> Option<Rect> {
        let k = unsafe { CFString::wrap_under_get_rule(kCGWindowBounds) };
        let b = d.find(&k)?;
        let b: CFDictionary<CFString, CFType> = unsafe { CFDictionary::wrap_under_get_rule(b.as_CFTypeRef() as CFDictionaryRef) };
        let get = |name: &str| -> Option<f32> { b.find(&CFString::new(name))?.downcast::<CFNumber>()?.to_f64().map(|v| v as f32) };
        Some(Rect::new(get("X")?, get("Y")?, get("Width")?, get("Height")?))
    }

    /// Janelas normais (camada 0, visíveis) de outros apps na tela. Não precisa de permissão:
    /// só camada, dono e retângulo (nunca o título nem o conteúdo).
    pub fn other_app_windows() -> Vec<Rect> {
        let Some(list) = copy_window_info(kCGWindowListOptionOnScreenOnly | kCGWindowListExcludeDesktopElements, kCGNullWindowID) else {
            return vec![];
        };
        let me = std::process::id() as f64;
        let mut out = vec![];
        for item in list.iter() {
            let d: CFDictionary<CFString, CFType> = unsafe { CFDictionary::wrap_under_get_rule(*item as CFDictionaryRef) };
            if num(&d, unsafe { kCGWindowLayer }) != Some(0.0) || num(&d, unsafe { kCGWindowOwnerPID }) == Some(me) {
                continue;
            }
            if num(&d, unsafe { kCGWindowAlpha }).map(|a| a <= 0.01).unwrap_or(false) {
                continue;
            }
            if let Some(r) = bounds(&d) {
                out.push(r);
            }
        }
        out
    }
}

#[cfg(windows)]
mod win {
    use windows_sys::Win32::UI::Shell::SHQueryUserNotificationState;

    /// Tela cheia / jogo / apresentação em algum monitor (o Windows não diz qual; por isso, aqui, o pato esconde em todos).
    pub fn fullscreen() -> bool {
        let mut state: i32 = 0;
        let hr = unsafe { SHQueryUserNotificationState(&mut state) };
        hr >= 0 && super::windows_state_is_fullscreen(state)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mon() -> Rect {
        Rect::new(0.0, 0.0, 1512.0, 982.0)
    }

    #[test]
    fn full_window_covers_but_maximized_with_menu_bar_does_not() {
        assert!(covers(Rect::new(0.0, 0.0, 1512.0, 982.0), mon(), TOLERANCE));
        assert!(covers(Rect::new(-1.0, -1.0, 1514.0, 984.0), mon(), TOLERANCE));
        // maximizada: sobra a barra de menu (~37 pt) em cima
        assert!(!covers(Rect::new(0.0, 37.0, 1512.0, 945.0), mon(), TOLERANCE));
        assert!(!covers(Rect::new(100.0, 100.0, 800.0, 600.0), mon(), TOLERANCE));
    }

    #[test]
    fn only_the_ducks_monitor_counts() {
        let second = Rect::new(1512.0, 0.0, 1920.0, 1080.0);
        let on_second = [Rect::new(1512.0, 0.0, 1920.0, 1080.0)];
        assert!(fullscreen_on(&on_second, second));
        assert!(!fullscreen_on(&on_second, mon()));
        assert!(!fullscreen_on(&[], mon()));
    }

    #[test]
    fn windows_states() {
        assert!(!windows_state_is_fullscreen(1)); // QUNS_NOT_PRESENT
        assert!(windows_state_is_fullscreen(2));
        assert!(windows_state_is_fullscreen(3));
        assert!(windows_state_is_fullscreen(4));
        assert!(!windows_state_is_fullscreen(5)); // aceita notificações
        assert!(!windows_state_is_fullscreen(6));
    }

    #[test]
    fn hides_after_two_reads_and_shows_after_one() {
        let mut s = DndState::default();
        assert!(!s.update(true, true));
        assert!(s.update(true, true));
        assert!(s.update(true, true));
        assert!(!s.update(true, false));
    }

    #[test]
    fn a_blip_does_not_hide() {
        let mut s = DndState::default();
        assert!(!s.update(true, true));
        assert!(!s.update(true, false));
        assert!(!s.update(true, true));
    }

    #[test]
    fn turning_the_option_off_shows_at_once() {
        let mut s = DndState::default();
        s.update(true, true);
        assert!(s.update(true, true));
        assert!(!s.update(false, true));
        assert!(!s.hidden());
        // ligando de novo com a tela cheia ainda ativa: precisa das leituras de novo
        assert!(!s.update(true, true));
        assert!(s.update(true, true));
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn macos_window_list_reads_without_panicking() {
        let list = mac::other_app_windows();
        eprintln!("{} janelas normais", list.len());
    }
}
