use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum Mood {
    #[default]
    Feliz,
    Raiva,
    Triste,
    Sozinho,
    Sono,
    Danca,
    Lagarto,
    Duvida,
}

impl Mood {
    pub const ALL: [Mood; 8] = [Mood::Feliz, Mood::Raiva, Mood::Triste, Mood::Sozinho, Mood::Sono, Mood::Danca, Mood::Lagarto, Mood::Duvida];

    pub fn next(self) -> Mood {
        let i = Mood::ALL.iter().position(|m| *m == self).unwrap_or(0);
        Mood::ALL[(i + 1) % Mood::ALL.len()]
    }

    pub fn id(self) -> &'static str {
        match self {
            Mood::Feliz => "feliz",
            Mood::Raiva => "raiva",
            Mood::Triste => "triste",
            Mood::Sozinho => "sozinho",
            Mood::Sono => "sono",
            Mood::Danca => "danca",
            Mood::Lagarto => "lagarto",
            Mood::Duvida => "duvida",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Mood::Feliz => "Feliz",
            Mood::Raiva => "Com raiva",
            Mood::Triste => "Triste",
            Mood::Sozinho => "Sozinho",
            Mood::Sono => "Com sono",
            Mood::Danca => "Dançando",
            Mood::Lagarto => "Lagartinho",
            Mood::Duvida => "Duvidoso",
        }
    }
}

/// Valores em unidades lógicas (multiplicados por `unit_scale` na simulação).
#[derive(Clone, Copy, Debug)]
pub struct MoodProfile {
    pub speed: f32,
    pub accel: f32,
    pub stop_distance: f32,
    pub can_grab: bool,
    pub grab_speed: f32,
    pub wander_speed: f32,
    pub sleep_after: f64,
    pub jitter: f32,
    pub look_at_cursor: bool,
}

pub fn profile(m: Mood) -> MoodProfile {
    match m {
        Mood::Feliz => MoodProfile { speed: 420.0, accel: 1800.0, stop_distance: 0.0, can_grab: true, grab_speed: 260.0, wander_speed: 70.0, sleep_after: 30.0, jitter: 0.0, look_at_cursor: false },
        Mood::Raiva => MoodProfile { speed: 700.0, accel: 4000.0, stop_distance: 0.0, can_grab: true, grab_speed: 420.0, wander_speed: 110.0, sleep_after: 60.0, jitter: 2.0, look_at_cursor: false },
        Mood::Triste => MoodProfile { speed: 140.0, accel: 600.0, stop_distance: 220.0, can_grab: false, grab_speed: 0.0, wander_speed: 40.0, sleep_after: 20.0, jitter: 0.0, look_at_cursor: false },
        Mood::Sozinho => MoodProfile { speed: 220.0, accel: 900.0, stop_distance: 120.0, can_grab: false, grab_speed: 0.0, wander_speed: 50.0, sleep_after: 30.0, jitter: 0.0, look_at_cursor: true },
        Mood::Danca => MoodProfile { speed: 300.0, accel: 1500.0, stop_distance: 0.0, can_grab: false, grab_speed: 0.0, wander_speed: 60.0, sleep_after: 90.0, jitter: 0.0, look_at_cursor: false },
        Mood::Lagarto => MoodProfile { speed: 560.0, accel: 3500.0, stop_distance: 0.0, can_grab: true, grab_speed: 380.0, wander_speed: 120.0, sleep_after: 120.0, jitter: 1.5, look_at_cursor: false },
        Mood::Duvida => MoodProfile { speed: 200.0, accel: 900.0, stop_distance: 90.0, can_grab: false, grab_speed: 0.0, wander_speed: 45.0, sleep_after: 30.0, jitter: 0.0, look_at_cursor: true },
        Mood::Sono => MoodProfile { speed: 90.0, accel: 400.0, stop_distance: 60.0, can_grab: false, grab_speed: 0.0, wander_speed: 30.0, sleep_after: 8.0, jitter: 0.0, look_at_cursor: false },
    }
}
