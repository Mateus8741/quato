//! "Tem som saindo do computador?" — para o pato dançar quando tiver música (ou qualquer som).
//!
//! Não grava nem escuta nada: só pergunta ao sistema se a saída de áudio padrão está tocando.
//! - macOS: CoreAudio `kAudioDevicePropertyDeviceIsRunningSomewhere` no dispositivo de saída
//!   padrão (sem permissão). Fica verdadeiro enquanto algum app mantém a saída aberta, o que
//!   pode incluir apps que deixam o áudio aberto em silêncio.
//! - Windows: `IAudioMeterInformation::GetPeakValue` do endpoint de saída padrão (pico > limiar).
//! - Outros sistemas: sempre falso.
//!
//! `SoundDebounce` é a parte pura (testável); `spawn` roda a leitura a cada 1 s.
use crate::sim::model::SimCommand;
use crate::state::AppState;
use std::time::Duration;
use tauri::{AppHandle, Manager};

/// Leituras seguidas com som para começar a dançar.
const ON_AFTER: u32 = 2;
/// Leituras seguidas sem som (≈ segundos) para parar.
const OFF_AFTER: u32 = 4;

#[derive(Default, Debug)]
pub struct SoundDebounce {
    on: bool,
    streak: u32,
}

impl SoundDebounce {
    /// Recebe uma leitura; devolve `Some(estado)` quando o estado muda.
    pub fn update(&mut self, sound: bool) -> Option<bool> {
        if sound == self.on {
            self.streak = 0;
            return None;
        }
        self.streak += 1;
        let need = if sound { ON_AFTER } else { OFF_AFTER };
        if self.streak >= need {
            self.on = sound;
            self.streak = 0;
            Some(sound)
        } else {
            None
        }
    }

    pub fn is_on(&self) -> bool {
        self.on
    }
}

#[cfg(target_os = "macos")]
mod sys {
    use std::ffi::c_void;

    #[repr(C)]
    struct AudioObjectPropertyAddress {
        selector: u32,
        scope: u32,
        element: u32,
    }

    #[link(name = "CoreAudio", kind = "framework")]
    extern "C" {
        fn AudioObjectGetPropertyData(
            object: u32,
            address: *const AudioObjectPropertyAddress,
            qualifier_size: u32,
            qualifier: *const c_void,
            data_size: *mut u32,
            data: *mut c_void,
        ) -> i32;
    }

    const fn fourcc(s: &[u8; 4]) -> u32 {
        ((s[0] as u32) << 24) | ((s[1] as u32) << 16) | ((s[2] as u32) << 8) | s[3] as u32
    }

    const SYSTEM_OBJECT: u32 = 1; // kAudioObjectSystemObject
    const DEFAULT_OUTPUT_DEVICE: u32 = fourcc(b"dOut"); // kAudioHardwarePropertyDefaultOutputDevice
    const IS_RUNNING_SOMEWHERE: u32 = fourcc(b"gone"); // kAudioDevicePropertyDeviceIsRunningSomewhere
    const SCOPE_GLOBAL: u32 = fourcc(b"glob"); // kAudioObjectPropertyScopeGlobal
    const ELEMENT_MAIN: u32 = 0; // kAudioObjectPropertyElementMain

    fn get_u32(object: u32, selector: u32) -> Option<u32> {
        let addr = AudioObjectPropertyAddress { selector, scope: SCOPE_GLOBAL, element: ELEMENT_MAIN };
        let mut value: u32 = 0;
        let mut size = std::mem::size_of::<u32>() as u32;
        // SAFETY: consulta de propriedade do CoreAudio com um buffer de 4 bytes do tamanho informado.
        let status = unsafe { AudioObjectGetPropertyData(object, &addr, 0, std::ptr::null(), &mut size, &mut value as *mut u32 as *mut c_void) };
        (status == 0 && size == 4).then_some(value)
    }

    pub struct Probe;

    #[allow(clippy::new_without_default)]
    impl Probe {
        pub fn new() -> Self {
            Probe
        }

        /// `None` = não deu para ler.
        pub fn sound_playing(&mut self) -> Option<bool> {
            let device = get_u32(SYSTEM_OBJECT, DEFAULT_OUTPUT_DEVICE)?;
            if device == 0 {
                return None;
            }
            get_u32(device, IS_RUNNING_SOMEWHERE).map(|v| v != 0)
        }
    }
}

#[cfg(windows)]
mod sys {
    use windows::Win32::Media::Audio::Endpoints::IAudioMeterInformation;
    use windows::Win32::Media::Audio::{eConsole, eRender, IMMDeviceEnumerator, MMDeviceEnumerator};
    use windows::Win32::System::Com::{CoCreateInstance, CoInitializeEx, CLSCTX_ALL, COINIT_MULTITHREADED};

    /// Pico acima disso conta como "tem som".
    const PEAK: f32 = 0.01;

    pub struct Probe {
        enumerator: Option<IMMDeviceEnumerator>,
    }

    impl Probe {
        pub fn new() -> Self {
            // SAFETY: inicializa o COM nesta thread (a thread do áudio é nossa e vive o app todo).
            unsafe {
                let _ = CoInitializeEx(None, COINIT_MULTITHREADED);
            }
            Probe { enumerator: None }
        }

        fn read(&mut self) -> windows::core::Result<bool> {
            // SAFETY: chamadas COM na thread inicializada acima; as interfaces se liberam no Drop.
            unsafe {
                if self.enumerator.is_none() {
                    self.enumerator = Some(CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL)?);
                }
                let en = self.enumerator.as_ref().unwrap();
                // Pega o padrão a cada leitura: a pessoa pode trocar de fone/caixa.
                let device = en.GetDefaultAudioEndpoint(eRender, eConsole)?;
                let meter: IAudioMeterInformation = device.Activate(CLSCTX_ALL, None)?;
                Ok(meter.GetPeakValue()? > PEAK)
            }
        }

        pub fn sound_playing(&mut self) -> Option<bool> {
            match self.read() {
                Ok(v) => Some(v),
                Err(_) => {
                    self.enumerator = None;
                    None
                }
            }
        }
    }
}

#[cfg(not(any(target_os = "macos", windows)))]
mod sys {
    pub struct Probe;
    impl Probe {
        pub fn new() -> Self {
            Probe
        }
        pub fn sound_playing(&mut self) -> Option<bool> {
            Some(false)
        }
    }
}

pub use sys::Probe;

pub fn spawn(app: AppHandle) {
    std::thread::Builder::new()
        .name("quato-audio".into())
        .spawn(move || {
            let mut probe = Probe::new();
            let mut deb = SoundDebounce::default();
            loop {
                std::thread::sleep(Duration::from_secs(1));
                let enabled = app.state::<AppState>().config.lock().unwrap().dance_on_music;
                if !enabled {
                    // Preferência desligada: nem consulta o sistema.
                    if deb.is_on() {
                        deb = SoundDebounce::default();
                        crate::actions::send(&app, SimCommand::SetMusic(false));
                    }
                    continue;
                }
                let sound = probe.sound_playing().unwrap_or(false);
                if let Some(on) = deb.update(sound) {
                    crate::actions::send(&app, SimCommand::SetMusic(on));
                }
            }
        })
        .expect("não consegui iniciar o detector de som");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn turns_on_after_two_readings_in_a_row() {
        let mut d = SoundDebounce::default();
        assert_eq!(d.update(true), None);
        assert_eq!(d.update(true), Some(true));
        assert_eq!(d.update(true), None);
    }

    #[test]
    fn a_single_blip_does_not_turn_on() {
        let mut d = SoundDebounce::default();
        assert_eq!(d.update(true), None);
        assert_eq!(d.update(false), None);
        assert_eq!(d.update(true), None);
        assert!(!d.is_on());
    }

    #[test]
    fn turns_off_only_after_four_silent_readings() {
        let mut d = SoundDebounce::default();
        d.update(true);
        d.update(true);
        assert_eq!(d.update(false), None);
        assert_eq!(d.update(false), None);
        assert_eq!(d.update(false), None);
        // uma pausa curta entre músicas não interrompe
        assert_eq!(d.update(true), None);
        assert_eq!(d.update(false), None);
        assert_eq!(d.update(false), None);
        assert_eq!(d.update(false), None);
        assert_eq!(d.update(false), Some(false));
    }

    #[test]
    fn probe_reads_without_panicking() {
        let mut p = Probe::new();
        let _ = p.sound_playing();
    }

    /// Teste manual (`cargo test -- --ignored audio_probe_sees_afplay --nocapture`):
    /// toca um som curto e confere que a saída padrão aparece como "tocando".
    #[cfg(target_os = "macos")]
    #[test]
    #[ignore]
    fn audio_probe_sees_afplay() {
        let mut p = Probe::new();
        let before = p.sound_playing();
        let mut child = std::process::Command::new("afplay").arg("/System/Library/Sounds/Submarine.aiff").spawn().expect("afplay");
        std::thread::sleep(Duration::from_millis(300));
        let during = p.sound_playing();
        child.wait().unwrap();
        std::thread::sleep(Duration::from_millis(1500));
        let after = p.sound_playing();
        eprintln!("antes={before:?} durante={during:?} depois={after:?}");
        assert_eq!(during, Some(true));
    }
}
