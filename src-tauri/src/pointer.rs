//! Mover o cursor de verdade. Criado e usado só dentro da thread da simulação
//! (no macOS o Enigo guarda objetos que não podem trocar de thread).
use enigo::{Coordinate, Enigo, Mouse, Settings};

pub struct Pointer {
    enigo: Enigo,
}

impl Pointer {
    pub fn new() -> Result<Self, String> {
        let settings = Settings { open_prompt_to_get_permissions: false, ..Settings::default() };
        Enigo::new(&settings).map(|enigo| Self { enigo }).map_err(|e| e.to_string())
    }

    pub fn move_to(&mut self, x: f32, y: f32) -> Result<(), String> {
        self.enigo.move_mouse(x.round() as i32, y.round() as i32, Coordinate::Abs).map_err(|e| e.to_string())
    }
}
