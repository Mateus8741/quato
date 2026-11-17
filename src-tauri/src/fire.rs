//! Arminha: quando o pato está com raiva e com o acessório `arminha`, ele atira no cursor.
//! Aqui vive só a lógica PURA (quando atirar, para onde, de onde sai o cano). Nada de Tauri.
//! O desenho dos tiros é feito pela janela overlay `fx` (veja `fx.rs` e `src/fx/`).
use crate::events::{DuckMode, Facing};
use crate::sim::mood::Mood;
use crate::sim::vec2::Vec2;

pub const WEAPON_ID: &str = "arminha";

/// Ponta do cano no pixel de arte (29, 20) do canvas 32x32, com o pato virado para a direita.
/// A janela do pato é 192x192; o canvas 128x128 fica em left 32 / top 64 (1 pixel de arte = 4 px)
/// e a âncora (pés) em (96, 184). Valores em unidades de design relativas à âncora, virado à direita.
pub const MUZZLE: Vec2 = Vec2 { x: 32.0 + 29.0 * 4.0 + 2.0 - 96.0, y: 64.0 + 20.0 * 4.0 + 2.0 - 184.0 };
/// Saída dos estojos, perto do pixel de arte (24, 19).
pub const EJECT: Vec2 = Vec2 { x: 32.0 + 24.0 * 4.0 + 2.0 - 96.0, y: 64.0 + 19.0 * 4.0 + 2.0 - 184.0 };

/// Tiros por rajada (sorteado) e tempos, em segundos.
pub const BURST_MIN: u8 = 2;
pub const BURST_MAX: u8 = 4;
pub const SHOT_GAP: f64 = 0.12;
pub const PAUSE_MIN: f64 = 0.8;
pub const PAUSE_MAX: f64 = 1.5;
/// Espera antes do primeiro tiro depois de ficar apto (dá tempo de virar e da janela carregar).
pub const FIRST_DELAY: f64 = 0.4;
/// Espalhamento do tiro em volta do cursor, em unidades de design.
pub const SPREAD_MIN: f32 = 6.0;
pub const SPREAD_MAX: f32 = 20.0;
/// Perto demais do cursor (perseguindo e colado nele) ele não atira.
pub const MIN_DIST: f32 = 48.0;

pub fn is_armed(accessories: &[String]) -> bool {
    accessories.iter().any(|a| a == WEAPON_ID)
}

/// Só atira com raiva, e parado, andando ou perseguindo (não dormindo, dançando, carregado nem arrastando o mouse).
pub fn can_fire(armed: bool, mood: Mood, mode: DuckMode) -> bool {
    armed && mood == Mood::Raiva && matches!(mode, DuckMode::Idle | DuckMode::Walk | DuckMode::Chase)
}

fn sign(f: Facing) -> f32 {
    if f == Facing::Right {
        1.0
    } else {
        -1.0
    }
}

/// Um tiro, em coordenadas da simulação.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Shot {
    pub muzzle: Vec2,
    pub hit: Vec2,
    pub eject: Vec2,
    /// Lado para onde os estojos saem: +1 direita, -1 esquerda (sempre para trás do cano).
    pub eject_dir: f32,
    /// Altura do "chão" (pés do pato), no eixo y da simulação.
    pub floor_y: f32,
}

/// `pos` = pés do pato; `unit_scale` = unidades de design -> simulação.
pub fn shot_geometry(pos: Vec2, facing: Facing, unit_scale: f32, cursor: Vec2, rng: &mut fastrand::Rng) -> Shot {
    let s = sign(facing);
    let muzzle = pos + Vec2::new(MUZZLE.x * s, MUZZLE.y) * unit_scale;
    let eject = pos + Vec2::new(EJECT.x * s, EJECT.y) * unit_scale;
    let ang = rng.f32() * std::f32::consts::TAU;
    let r = (SPREAD_MIN + rng.f32() * (SPREAD_MAX - SPREAD_MIN)) * unit_scale;
    let hit = cursor + Vec2::new(ang.cos() * r, ang.sin() * r);
    Shot { muzzle, hit, eject, eject_dir: -s, floor_y: pos.y }
}

/// Agenda as rajadas. `poll` diz se é hora de dar um tiro; `due` diz quando acordar de novo.
pub struct Burst {
    next_at: Option<f64>,
    left: u8,
    rng: fastrand::Rng,
}

impl Burst {
    pub fn new(seed: u64) -> Self {
        Self { next_at: None, left: 0, rng: fastrand::Rng::with_seed(seed) }
    }

    /// `eligible` falso zera tudo (a próxima vez recomeça com `FIRST_DELAY`).
    pub fn poll(&mut self, now: f64, eligible: bool) -> bool {
        if !eligible {
            self.next_at = None;
            self.left = 0;
            return false;
        }
        let at = *self.next_at.get_or_insert(now + FIRST_DELAY);
        if now < at {
            return false;
        }
        if self.left == 0 {
            self.left = BURST_MIN + self.rng.u8(0..=(BURST_MAX - BURST_MIN));
        }
        self.left -= 1;
        self.next_at = Some(if self.left > 0 { now + SHOT_GAP } else { now + PAUSE_MIN + self.rng.f64() * (PAUSE_MAX - PAUSE_MIN) });
        true
    }

    /// Quanto falta para o próximo tiro (None = nada agendado, não precisa acordar por causa disso).
    pub fn due(&self, now: f64) -> Option<f64> {
        self.next_at.map(|t| (t - now).max(0.0))
    }

    pub fn rng(&mut self) -> &mut fastrand::Rng {
        &mut self.rng
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn muzzle_matches_sprite_contract() {
        // pixel (29,20): x = 32+29*4+2 = 150 (54 à direita da âncora), y = 64+20*4+2 = 146 (38 acima dos pés)
        assert_eq!((MUZZLE.x, MUZZLE.y), (54.0, -38.0));
        assert_eq!((EJECT.x, EJECT.y), (34.0, -42.0));
    }

    #[test]
    fn only_armed_angry_and_awake() {
        assert!(can_fire(true, Mood::Raiva, DuckMode::Idle));
        assert!(can_fire(true, Mood::Raiva, DuckMode::Walk));
        assert!(can_fire(true, Mood::Raiva, DuckMode::Chase));
        assert!(!can_fire(false, Mood::Raiva, DuckMode::Idle));
        assert!(!can_fire(true, Mood::Feliz, DuckMode::Idle));
        for m in [DuckMode::Sleep, DuckMode::Held, DuckMode::Grab, DuckMode::Dance] {
            assert!(!can_fire(true, Mood::Raiva, m), "{m:?}");
        }
        assert!(is_armed(&["coroa".into(), "arminha".into()]));
        assert!(!is_armed(&["coroa".into()]));
    }

    #[test]
    fn geometry_mirrors_with_facing_and_scales() {
        let mut rng = fastrand::Rng::with_seed(1);
        let pos = Vec2::new(500.0, 400.0);
        let cur = Vec2::new(900.0, 300.0);
        let r = shot_geometry(pos, Facing::Right, 1.0, cur, &mut rng);
        let l = shot_geometry(pos, Facing::Left, 1.0, cur, &mut rng);
        assert_eq!(r.muzzle, Vec2::new(554.0, 362.0));
        assert_eq!(l.muzzle, Vec2::new(446.0, 362.0));
        assert_eq!(r.eject_dir, -1.0);
        assert_eq!(l.eject_dir, 1.0);
        assert_eq!(r.floor_y, 400.0);
        let w = shot_geometry(pos, Facing::Right, 2.0, cur, &mut rng);
        assert_eq!(w.muzzle, Vec2::new(608.0, 324.0));
    }

    #[test]
    fn hit_spreads_within_range() {
        let mut rng = fastrand::Rng::with_seed(9);
        let cur = Vec2::new(100.0, 100.0);
        for _ in 0..200 {
            let s = shot_geometry(Vec2::ZERO, Facing::Right, 1.5, cur, &mut rng);
            let d = s.hit.dist(cur);
            assert!(d >= SPREAD_MIN * 1.5 - 0.01 && d <= SPREAD_MAX * 1.5 + 0.01, "{d}");
        }
    }

    #[test]
    fn bursts_are_short_and_spaced() {
        let mut b = Burst::new(3);
        assert!(b.due(0.0).is_none());
        let mut t = 0.0;
        let mut shots: Vec<f64> = vec![];
        while t < 30.0 {
            if b.poll(t, true) {
                shots.push(t);
            }
            t += 0.01;
        }
        assert!(shots[0] >= FIRST_DELAY - 0.001 && shots[0] < FIRST_DELAY + 0.03);
        // separa em rajadas pelo intervalo
        let mut bursts: Vec<Vec<f64>> = vec![vec![shots[0]]];
        for w in shots.windows(2) {
            let gap = w[1] - w[0];
            if gap < 0.5 {
                assert!((gap - SHOT_GAP).abs() < 0.03, "{gap}");
                bursts.last_mut().unwrap().push(w[1]);
            } else {
                assert!(gap >= PAUSE_MIN - 0.001 && gap <= PAUSE_MAX + 0.03, "{gap}");
                bursts.push(vec![w[1]]);
            }
        }
        assert!(bursts.len() >= 8);
        // a última rajada pode ter sido cortada pelo fim do teste
        for bu in &bursts[..bursts.len() - 1] {
            assert!((BURST_MIN as usize..=BURST_MAX as usize).contains(&bu.len()), "{}", bu.len());
        }
    }

    #[test]
    fn not_eligible_resets_and_restarts_with_delay() {
        let mut b = Burst::new(5);
        assert!(!b.poll(0.0, true));
        assert!(b.poll(0.5, true));
        assert!(!b.poll(0.6, false));
        assert!(b.due(0.6).is_none());
        // volta a ser apto: espera de novo antes do primeiro tiro
        assert!(!b.poll(10.0, true));
        assert!(!b.poll(10.2, true));
        assert!(b.poll(10.45, true));
        assert!(b.due(10.45).unwrap() > 0.0);
    }
}
