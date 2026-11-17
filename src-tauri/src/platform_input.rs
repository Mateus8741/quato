//! Estado global do teclado e do mouse, sem pedir permissão de Acessibilidade
//! (só consulta o estado; não escuta nem injeta eventos).
use crate::sim::vec2::Vec2;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct InputState {
    pub ctrl: bool,
    /// Botão esquerdo do mouse apertado.
    pub button: bool,
    pub cursor: Option<Vec2>,
}

#[cfg(target_os = "macos")]
mod sys {
    use super::*;
    use std::ffi::c_void;

    #[repr(C)]
    #[derive(Clone, Copy)]
    struct CGPoint {
        x: f64,
        y: f64,
    }

    #[link(name = "CoreGraphics", kind = "framework")]
    extern "C" {
        fn CGEventSourceFlagsState(state_id: i32) -> u64;
        fn CGEventSourceButtonState(state_id: i32, button: u32) -> bool;
        fn CGEventCreate(source: *const c_void) -> *mut c_void;
        fn CGEventGetLocation(event: *const c_void) -> CGPoint;
    }
    #[link(name = "CoreFoundation", kind = "framework")]
    extern "C" {
        fn CFRelease(cf: *const c_void);
    }

    const COMBINED_SESSION: i32 = 0;
    const FLAG_CONTROL: u64 = 0x0004_0000;
    const BUTTON_LEFT: u32 = 0;

    pub fn read() -> InputState {
        // SAFETY: chamadas de consulta do CoreGraphics; o evento criado é liberado logo em seguida.
        unsafe {
            let ctrl = CGEventSourceFlagsState(COMBINED_SESSION) & FLAG_CONTROL != 0;
            let button = CGEventSourceButtonState(COMBINED_SESSION, BUTTON_LEFT);
            let ev = CGEventCreate(std::ptr::null());
            let cursor = if ev.is_null() {
                None
            } else {
                let p = CGEventGetLocation(ev);
                CFRelease(ev);
                Some(Vec2::new(p.x as f32, p.y as f32))
            };
            InputState { ctrl, button, cursor }
        }
    }
}

#[cfg(windows)]
mod sys {
    use super::*;
    use windows_sys::Win32::Foundation::POINT;
    use windows_sys::Win32::UI::Input::KeyboardAndMouse::{GetAsyncKeyState, VK_CONTROL, VK_LBUTTON};
    use windows_sys::Win32::UI::WindowsAndMessaging::GetCursorPos;

    pub fn read() -> InputState {
        // SAFETY: consultas simples da API do Windows.
        unsafe {
            let down = |vk: u16| (GetAsyncKeyState(vk as i32) as u16) & 0x8000 != 0;
            let mut pt = POINT { x: 0, y: 0 };
            let cursor = if GetCursorPos(&mut pt) != 0 { Some(Vec2::new(pt.x as f32, pt.y as f32)) } else { None };
            InputState { ctrl: down(VK_CONTROL), button: down(VK_LBUTTON), cursor }
        }
    }
}

#[cfg(not(any(target_os = "macos", windows)))]
mod sys {
    use super::*;
    pub fn read() -> InputState {
        InputState::default()
    }
}

pub fn read() -> InputState {
    sys::read()
}
