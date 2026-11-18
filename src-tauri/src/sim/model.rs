//! Simulação PURA do pato: sem Tauri, sem relógio, sem I/O. Tudo testável.
use super::mood::{profile, Mood};
use super::vec2::{Rect, Vec2};
use crate::events::{DuckMode, DuckView, Facing, Pace};

pub const MAX_GRAB_SECS: f64 = 10.0;
const GRAB_COOLDOWN: f64 = 4.0;
const RELEASE_COOLDOWN: f64 = 10.0;
const BEAK: Vec2 = Vec2 { x: 14.0, y: -18.0 };
const FIGHT_DIST: f32 = 40.0;
const FIGHT_TICKS: u32 = 3;
const GRAB_ENTER_DIST: f32 = 12.0;
const MOVE_EPS: f32 = 5.0;
/// Migalha: quanto tempo ele "come" parado, e quanto tempo desiste se não chegar nela.
pub const CRUMB_EAT_SECS: f64 = 0.9;
pub const CRUMB_TIMEOUT: f64 = 20.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReleaseReason {
    Timeout,
    Fight,
    Command,
    PointerLost,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Mode {
    Idle,
    Wander,
    Chase,
    Grab,
    Sleep,
    Held,
    /// Dançando no lugar (humor Dançando, ou som tocando com "dançar com som" ligado).
    Dance,
    /// Indo até uma migalha (ou comendo ela, parado).
    Crumb,
}

/// Onde a migalha cai: no cursor se ele está na área útil; senão, ao lado do pato (do lado com mais espaço).
pub fn crumb_spot(cursor: Vec2, bounds: Rect, duck: Vec2, scale: f32) -> Vec2 {
    let inside = cursor.x >= bounds.x && cursor.x <= bounds.right() && cursor.y >= bounds.y && cursor.y <= bounds.bottom();
    if inside {
        return cursor;
    }
    let reach = 110.0 * scale;
    let side = if duck.x - bounds.x > bounds.right() - duck.x { -1.0 } else { 1.0 };
    bounds.clamp(Vec2::new(duck.x + side * reach, duck.y + BEAK.y * scale))
}

/// O que a simulação conta para quem desenha a migalha (`crumb.rs`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum CrumbEvent {
    /// Caiu uma migalha nessa posição.
    Dropped(Vec2),
    /// O pato chegou e começou a comer.
    Eaten(Vec2),
    /// A migalha sumiu sem ser comida (desistiu, humor mudou, pegaram o pato).
    Gone,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct Crumb {
    pos: Vec2,
    since: f64,
    eating_until: Option<f64>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SimCommand {
    SetChasing(bool),
    SetMood(Mood),
    SetDragEnabled(bool),
    ReleaseMouse,
    Wake,
    Shutdown,
    SetFreeRoam(bool),
    /// Lugar do pato (`None` = canto padrão).
    SetHome(Option<Vec2>),
    /// A pessoa pegou o pato (Ctrl + clique) com o cursor nessa posição.
    UserGrab(Vec2),
    /// A pessoa soltou o pato: o lugar onde ele está vira o novo "canto".
    UserDrop,
    /// Tem som saindo do computador (já com debounce, vindo de `audio.rs`).
    SetMusic(bool),
    /// Preferência "Dançar quando tiver som tocando".
    SetDanceOnMusic(bool),
    /// Modo foco: o pato fica quieto e dormindo (não persegue, não dança, o cursor não o acorda).
    SetFocus(bool),
    /// Atalho da migalha: joga uma na posição do cursor (só visual; ignorado dormindo).
    DropCrumb,
}

pub struct SimInput {
    pub dt: f32,
    pub now: f64,
    pub cursor: Vec2,
    pub bounds: Rect,
    pub home: Vec2,
    pub unit_scale: f32,
    pub pointer_ok: bool,
}

pub struct SimOutput {
    pub pos: Vec2,
    pub warp_cursor: Option<Vec2>,
    pub view: DuckView,
    pub active: bool,
    /// Preenchido no instante em que a pessoa solta o pato num lugar novo.
    pub new_home: Option<Vec2>,
    /// Perseguindo, mas já colado no cursor e sem se mexer: pode checar mais devagar.
    pub settled: bool,
    pub released: Option<ReleaseReason>,
    /// Migalha: caiu, foi comida ou sumiu (no máximo um por passo).
    pub crumb: Option<CrumbEvent>,
}

pub struct Sim {
    mode: Mode,
    mood: Mood,
    chasing: bool,
    drag_enabled: bool,
    pos: Vec2,
    vel: Vec2,
    facing: Facing,
    pace: Pace,
    wander_target: Vec2,
    next_wander_at: f64,
    last_cursor: Vec2,
    last_move_at: f64,
    grab_since: f64,
    grab_target: Vec2,
    last_set: Option<Vec2>,
    fight_ticks: u32,
    cooldown_until: f64,
    pending_release: Option<ReleaseReason>,
    free_roam: bool,
    home_override: Option<Vec2>,
    held_off: Vec2,
    pending_new_home: Option<Vec2>,
    music: bool,
    dance_on_music: bool,
    /// Modo foco (Pomodoro) ligado.
    focus: bool,
    /// Atirando no cursor (arminha + raiva): fica sempre virado para ele.
    aiming: bool,
    crumb: Option<Crumb>,
    crumb_request: bool,
    crumb_event: Option<CrumbEvent>,
    initialized: bool,
    rng: fastrand::Rng,
}

impl Sim {
    pub fn new(mood: Mood, seed: u64) -> Self {
        Self {
            mode: Mode::Idle,
            mood,
            chasing: false,
            drag_enabled: false,
            pos: Vec2::ZERO,
            vel: Vec2::ZERO,
            facing: Facing::Left,
            pace: Pace::Still,
            wander_target: Vec2::ZERO,
            next_wander_at: 0.0,
            last_cursor: Vec2::ZERO,
            last_move_at: 0.0,
            grab_since: 0.0,
            grab_target: Vec2::ZERO,
            last_set: None,
            fight_ticks: 0,
            cooldown_until: 0.0,
            pending_release: None,
            free_roam: false,
            home_override: None,
            held_off: Vec2::ZERO,
            pending_new_home: None,
            music: false,
            dance_on_music: true,
            focus: false,
            aiming: false,
            crumb: None,
            crumb_request: false,
            crumb_event: None,
            initialized: false,
            rng: fastrand::Rng::with_seed(seed),
        }
    }

    pub fn pos(&self) -> Vec2 {
        self.pos
    }

    /// Mirando no cursor: enquanto estiver ligado o pato fica virado para ele (só andando/parado/perseguindo).
    pub fn set_aiming(&mut self, on: bool) {
        self.aiming = on;
    }

    /// Quer dançar agora? (o modo só vira Dance se estiver parado/passeando)
    fn wants_dance(&self) -> bool {
        !self.focus && (self.mood == Mood::Danca || (self.music && self.dance_on_music))
    }

    pub fn is_grabbing(&self) -> bool {
        self.mode == Mode::Grab
    }

    pub fn view(&self) -> DuckView {
        DuckView {
            mode: match self.mode {
                Mode::Idle => DuckMode::Idle,
                Mode::Wander => DuckMode::Walk,
                Mode::Chase => DuckMode::Chase,
                Mode::Grab => DuckMode::Grab,
                Mode::Sleep => DuckMode::Sleep,
                Mode::Held => DuckMode::Held,
                Mode::Dance => DuckMode::Dance,
                Mode::Crumb if self.crumb.is_some_and(|c| c.eating_until.is_some()) => DuckMode::Eat,
                Mode::Crumb => DuckMode::Walk,
            },
            mood: self.mood,
            facing: self.facing,
            pace: self.pace,
            chasing: self.chasing,
            drag_enabled: self.drag_enabled,
        }
    }

    /// Chamado depois de um panic na thread: nunca deixa o arrasto continuar.
    pub fn panic_reset(&mut self) {
        self.mode = Mode::Idle;
        self.chasing = false;
        self.drag_enabled = false;
        self.vel = Vec2::ZERO;
        self.last_set = None;
        self.crumb = None;
        self.crumb_request = false;
    }

    /// A migalha acabou (comida, desistência ou cancelada): volta ao que fazia.
    fn finish_crumb(&mut self, now: f64) {
        self.crumb = None;
        if self.mode == Mode::Crumb {
            self.mode = if self.chasing { Mode::Chase } else { Mode::Idle };
            self.last_move_at = now;
            self.schedule_wander(now);
        }
    }

    /// Cancela a migalha (se houver) avisando que ela sumiu.
    fn cancel_crumb(&mut self, now: f64) {
        if self.crumb.is_some() {
            self.crumb_event = Some(CrumbEvent::Gone);
            self.finish_crumb(now);
        }
    }

    /// Pedido de migalha: só vale se ele não estiver dormindo de sono, no colo nem arrastando o mouse.
    ///
    /// Se o cursor está fora da área útil (ex.: no menu da bandeja, na barra de menus), a migalha cai perto do pato.
    fn try_drop_crumb(&mut self, cursor: Vec2, bounds: Rect, scale: f32, now: f64) {
        let cursor = crumb_spot(cursor, bounds, self.pos, scale);
        let eating = self.crumb.is_some_and(|c| c.eating_until.is_some());
        if self.mood == Mood::Sono || matches!(self.mode, Mode::Held | Mode::Grab) || eating {
            return;
        }
        self.crumb = Some(Crumb { pos: cursor, since: now, eating_until: None });
        self.mode = Mode::Crumb;
        self.last_move_at = now;
        self.crumb_event = Some(CrumbEvent::Dropped(cursor));
    }

    fn release_grab(&mut self, reason: ReleaseReason, now: f64, cooldown: f64) {
        self.mode = if self.chasing { Mode::Chase } else { Mode::Idle };
        self.cooldown_until = now + cooldown;
        self.pending_release = Some(reason);
        self.last_set = None;
        self.fight_ticks = 0;
        self.last_move_at = now;
        self.next_wander_at = now + self.rng.f64() * 12.0 + 8.0;
    }

    pub fn apply(&mut self, cmd: SimCommand, now: f64) {
        match cmd {
            SimCommand::SetChasing(on) => {
                self.chasing = on;
                if !on && self.mode == Mode::Grab {
                    self.release_grab(ReleaseReason::Command, now, GRAB_COOLDOWN);
                }
                if on && self.mode == Mode::Sleep && !self.focus {
                    self.mode = Mode::Chase;
                }
                self.last_move_at = now;
            }
            SimCommand::SetMood(m) => {
                self.mood = m;
                if m == Mood::Sono {
                    self.cancel_crumb(now);
                }
            }
            SimCommand::SetDragEnabled(on) => {
                self.drag_enabled = on;
                if !on && self.mode == Mode::Grab {
                    self.release_grab(ReleaseReason::Command, now, GRAB_COOLDOWN);
                }
            }
            SimCommand::ReleaseMouse => {
                if self.mode == Mode::Grab {
                    self.release_grab(ReleaseReason::Command, now, RELEASE_COOLDOWN);
                }
            }
            SimCommand::Wake => {
                self.last_move_at = now;
                if self.mode == Mode::Sleep {
                    self.mode = if self.chasing { Mode::Chase } else { Mode::Idle };
                }
            }
            SimCommand::Shutdown => {
                self.cancel_crumb(now);
                if self.mode == Mode::Grab {
                    self.release_grab(ReleaseReason::Command, now, RELEASE_COOLDOWN);
                }
            }
            SimCommand::SetFreeRoam(on) => {
                self.free_roam = on;
                self.replan_wander(now);
            }
            SimCommand::SetHome(h) => {
                self.home_override = h;
                self.replan_wander(now);
            }
            SimCommand::UserGrab(cursor) => {
                self.cancel_crumb(now);
                if self.mode == Mode::Grab {
                    self.release_grab(ReleaseReason::Command, now, RELEASE_COOLDOWN);
                }
                self.mode = Mode::Held;
                self.held_off = cursor - self.pos;
                self.vel = Vec2::ZERO;
                self.last_move_at = now;
            }
            SimCommand::UserDrop => {
                if self.mode == Mode::Held {
                    self.mode = if self.chasing { Mode::Chase } else { Mode::Idle };
                    self.home_override = Some(self.pos);
                    self.pending_new_home = Some(self.pos);
                    self.cooldown_until = now + GRAB_COOLDOWN;
                    self.last_move_at = now;
                    self.schedule_wander(now);
                }
            }
            SimCommand::SetMusic(on) => self.music = on,
            SimCommand::SetDanceOnMusic(on) => self.dance_on_music = on,
            SimCommand::SetFocus(on) => {
                self.focus = on;
                if !on && self.mode == Mode::Sleep {
                    // Fim do foco: acorda.
                    self.mode = if self.chasing { Mode::Chase } else { Mode::Idle };
                    self.last_move_at = now;
                    self.schedule_wander(now);
                }
            }
            SimCommand::DropCrumb => self.crumb_request = true,
        }
    }

    fn approach(vel: Vec2, desired: Vec2, max_delta: f32) -> Vec2 {
        let d = desired - vel;
        let l = d.len();
        if l <= max_delta || l < 1e-6 {
            desired
        } else {
            vel + d * (max_delta / l)
        }
    }

    fn beak_offset(&self, s: f32) -> Vec2 {
        let sign = if self.facing == Facing::Right { 1.0 } else { -1.0 };
        Vec2::new(BEAK.x * sign * s, BEAK.y * s)
    }

    fn far_target(&mut self, from: Vec2, bounds: &Rect, s: f32) -> Vec2 {
        let mut best = from;
        for _ in 0..8 {
            let p = Vec2::new(bounds.x + self.rng.f32() * bounds.w, bounds.y + self.rng.f32() * bounds.h);
            best = p;
            if p.dist(from) >= 300.0 * s {
                break;
            }
        }
        bounds.clamp(best)
    }

    fn schedule_wander(&mut self, now: f64) {
        // Passeando pela tela toda ele não fica parado tanto tempo.
        let (base, spread) = if self.free_roam { (2.0, 5.0) } else { (8.0, 12.0) };
        self.next_wander_at = now + base + self.rng.f64() * spread;
    }

    /// Mudou o modo de passeio ou o lugar dele: volta a decidir para onde ir logo.
    fn replan_wander(&mut self, now: f64) {
        if matches!(self.mode, Mode::Idle | Mode::Wander) {
            self.mode = Mode::Idle;
            self.next_wander_at = now;
        }
    }

    pub fn step(&mut self, i: &SimInput) -> SimOutput {
        let s = i.unit_scale;
        let now = i.now;
        let prof = profile(self.mood);
        let mut warp = None;

        if !self.initialized {
            self.initialized = true;
            // Nasce no lugar que a pessoa escolheu (se houver), não no canto padrão.
            self.pos = self.home_override.unwrap_or(i.home);
            self.last_cursor = i.cursor;
            self.last_move_at = now;
            self.schedule_wander(now);
        }

        let moved = i.cursor.dist(self.last_cursor) > MOVE_EPS * s;
        if moved && self.mode != Mode::Grab {
            self.last_cursor = i.cursor;
            self.last_move_at = now;
        }

        if std::mem::take(&mut self.crumb_request) {
            self.try_drop_crumb(i.cursor, i.bounds, i.unit_scale, now);
        }

        // Transições por flag de perseguição (perseguir tem prioridade sobre dançar).
        // Foco: dorme na hora (de parado, passeando, dançando ou perseguindo) e fica dormindo.
        if self.focus && matches!(self.mode, Mode::Idle | Mode::Wander | Mode::Dance | Mode::Chase) {
            self.mode = Mode::Sleep;
            self.vel = Vec2::ZERO;
        }
        if self.chasing && !self.focus && matches!(self.mode, Mode::Idle | Mode::Wander | Mode::Dance) {
            self.mode = Mode::Chase;
        }
        if self.chasing && !self.focus && self.mode == Mode::Sleep && (moved || self.mood != Mood::Sono) {
            self.mode = Mode::Chase;
        }
        if !self.chasing && self.mode == Mode::Chase {
            self.mode = Mode::Idle;
            self.schedule_wander(now);
        }

        // Soneca (com vontade de dançar ele não dorme).
        let dance = self.wants_dance();
        let can_sleep = (matches!(self.mode, Mode::Idle | Mode::Wander) && !dance) || (self.mode == Mode::Chase && self.mood == Mood::Sono);
        if can_sleep && now - self.last_move_at >= prof.sleep_after {
            self.mode = Mode::Sleep;
            self.vel = Vec2::ZERO;
        }
        if self.mode == Mode::Sleep && !self.chasing && !self.focus && moved {
            self.mode = Mode::Idle;
            self.schedule_wander(now);
        }

        // Dança: só a partir de parado/passeando; nunca interrompe perseguição, arrasto,
        // colo ou soneca (som não acorda o pato: ele dança quando acordar).
        if dance && matches!(self.mode, Mode::Idle | Mode::Wander) {
            self.mode = Mode::Dance;
        } else if !dance && self.mode == Mode::Dance {
            self.mode = Mode::Idle;
            self.last_move_at = now;
            self.schedule_wander(now);
        }

        match self.mode {
            Mode::Idle => {
                self.vel = Self::approach(self.vel, Vec2::ZERO, 2000.0 * s * i.dt);
                if now >= self.next_wander_at {
                    let t = if self.free_roam {
                        // Qualquer ponto da tela (de preferência longe de onde está).
                        self.far_target(self.pos, &i.bounds, s)
                    } else {
                        let c = self.home_override.unwrap_or(i.home);
                        Vec2::new(c.x + (self.rng.f32() * 2.0 - 1.0) * 150.0 * s, c.y + (self.rng.f32() * 2.0 - 1.0) * 60.0 * s)
                    };
                    self.wander_target = i.bounds.clamp(t);
                    self.mode = Mode::Wander;
                }
            }
            Mode::Wander => {
                let d = self.wander_target - self.pos;
                if d.len() < 4.0 * s {
                    self.mode = Mode::Idle;
                    self.vel = Vec2::ZERO;
                    self.schedule_wander(now);
                } else {
                    let speed = if self.free_roam { prof.wander_speed.max(60.0) * 1.5 } else { prof.wander_speed };
                    self.vel = Self::approach(self.vel, d.norm() * speed * s, 1500.0 * s * i.dt);
                    self.pos = self.pos + self.vel * i.dt;
                }
            }
            Mode::Chase => {
                let tgt = i.cursor - self.beak_offset(s);
                let d = tgt - self.pos;
                let dist = d.len();
                if self.drag_enabled && i.pointer_ok && prof.can_grab && now >= self.cooldown_until && dist < GRAB_ENTER_DIST * s {
                    self.mode = Mode::Grab;
                    self.grab_since = now;
                    self.last_set = None;
                    self.fight_ticks = 0;
                    self.grab_target = self.far_target(self.pos, &i.bounds, s);
                } else {
                    let desired = if dist > (prof.stop_distance * s).max(3.0 * s) { d.norm() * prof.speed * s } else { Vec2::ZERO };
                    self.vel = Self::approach(self.vel, desired, prof.accel * s * i.dt);
                    self.pos = self.pos + self.vel * i.dt;
                    if prof.jitter > 0.0 && self.vel.len() > 10.0 * s {
                        let j = prof.jitter * s;
                        self.pos = self.pos + Vec2::new((self.rng.f32() * 2.0 - 1.0) * j, (self.rng.f32() * 2.0 - 1.0) * j);
                    }
                }
            }
            Mode::Grab => {
                if !i.pointer_ok {
                    self.release_grab(ReleaseReason::PointerLost, now, GRAB_COOLDOWN);
                } else if now - self.grab_since >= MAX_GRAB_SECS {
                    self.release_grab(ReleaseReason::Timeout, now, GRAB_COOLDOWN);
                } else {
                    if let Some(last) = self.last_set {
                        if i.cursor.dist(last) > FIGHT_DIST * s {
                            self.fight_ticks += 1;
                        } else {
                            self.fight_ticks = 0;
                        }
                    }
                    if self.fight_ticks >= FIGHT_TICKS {
                        self.release_grab(ReleaseReason::Fight, now, GRAB_COOLDOWN);
                    } else {
                        let d = self.grab_target - self.pos;
                        if d.len() < 8.0 * s {
                            self.grab_target = self.far_target(self.pos, &i.bounds, s);
                        }
                        self.vel = d.norm() * prof.grab_speed.max(160.0) * s;
                        self.pos = self.pos + self.vel * i.dt;
                        let w = self.pos + self.beak_offset(s);
                        warp = Some(w);
                        self.last_set = Some(w);
                    }
                }
            }
            Mode::Sleep => {
                self.vel = Vec2::ZERO;
            }
            Mode::Held => {
                self.pos = i.cursor - self.held_off;
                self.vel = Vec2::ZERO;
                self.last_move_at = now;
            }
            Mode::Dance => {
                // Fica no lugar; a animação é toda do frontend.
                self.vel = Vec2::ZERO;
                self.last_move_at = now;
            }
            Mode::Crumb => match self.crumb {
                None => self.finish_crumb(now),
                Some(mut c) => {
                    let dir = if c.pos.x >= self.pos.x { 1.0 } else { -1.0 };
                    self.last_move_at = now;
                    if let Some(until) = c.eating_until {
                        self.vel = Vec2::ZERO;
                        self.facing = if dir > 0.0 { Facing::Right } else { Facing::Left };
                        if now >= until {
                            self.finish_crumb(now);
                        }
                    } else if now - c.since > CRUMB_TIMEOUT {
                        self.cancel_crumb(now);
                    } else {
                        // O bico (não os pés) vai até a migalha; o alvo respeita os limites da tela.
                        let goal = i.bounds.clamp(Vec2::new(c.pos.x - BEAK.x * dir * s, c.pos.y - BEAK.y * s));
                        let d = goal - self.pos;
                        if d.len() < 4.0 * s {
                            self.vel = Vec2::ZERO;
                            c.eating_until = Some(now + CRUMB_EAT_SECS);
                            self.crumb = Some(c);
                            self.crumb_event = Some(CrumbEvent::Eaten(c.pos));
                            self.facing = if dir > 0.0 { Facing::Right } else { Facing::Left };
                        } else {
                            let speed = prof.speed.clamp(160.0, 450.0);
                            self.vel = Self::approach(self.vel, d.norm() * speed * s, 2500.0 * s * i.dt);
                            let step = self.vel * i.dt;
                            self.pos = if step.len() >= d.len() { goal } else { self.pos + step };
                        }
                    }
                }
            },
        }

        self.pos = i.bounds.clamp(self.pos);

        // Direção, com histerese.
        if self.aiming && matches!(self.mode, Mode::Idle | Mode::Wander | Mode::Chase) {
            if (i.cursor.x - self.pos.x).abs() > 6.0 * s {
                self.facing = if i.cursor.x >= self.pos.x { Facing::Right } else { Facing::Left };
            }
        } else if self.vel.x > 20.0 * s {
            self.facing = Facing::Right;
        } else if self.vel.x < -20.0 * s {
            self.facing = Facing::Left;
        } else if prof.look_at_cursor && matches!(self.mode, Mode::Idle | Mode::Chase | Mode::Sleep) {
            self.facing = if i.cursor.x >= self.pos.x { Facing::Right } else { Facing::Left };
        }

        let speed = self.vel.len() / s;
        self.pace = if self.mode == Mode::Sleep || speed < 5.0 {
            Pace::Still
        } else if speed <= 260.0 {
            Pace::Slow
        } else {
            Pace::Fast
        };

        SimOutput {
            pos: self.pos,
            warp_cursor: warp,
            view: self.view(),
            active: matches!(self.mode, Mode::Wander | Mode::Chase | Mode::Grab | Mode::Held | Mode::Dance | Mode::Crumb),
            new_home: self.pending_new_home.take(),
            // Dançando ele não sai do lugar: não precisa de 60 Hz.
            settled: (self.mode == Mode::Chase && self.vel.len() < 1.0) || self.mode == Mode::Dance || (self.mode == Mode::Crumb && self.vel.len() < 1.0),
            released: self.pending_release.take(),
            crumb: self.crumb_event.take(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const DT: f32 = 1.0 / 60.0;

    fn input(cursor: Vec2, now: f64, pointer_ok: bool) -> SimInput {
        let b = Rect::new(0.0, 0.0, 1600.0, 1000.0);
        SimInput { dt: DT, now, cursor, bounds: b, home: Vec2::new(1400.0, 950.0), unit_scale: 1.0, pointer_ok }
    }

    /// Roda `secs` segundos; o cursor é uma função do tempo e da última posição de warp.
    fn run(sim: &mut Sim, start: f64, secs: f64, cursor: Vec2, follow_warp: bool, pointer_ok: bool) -> (f64, Vec2, Option<ReleaseReason>) {
        let mut now = start;
        let mut cur = cursor;
        let mut released = None;
        let end = start + secs;
        while now < end {
            let out = sim.step(&input(cur, now, pointer_ok));
            if follow_warp {
                if let Some(w) = out.warp_cursor {
                    cur = w;
                }
            }
            if out.released.is_some() && released.is_none() {
                released = out.released;
            }
            now += DT as f64;
        }
        (now, cur, released)
    }

    #[test]
    fn chase_goes_to_cursor() {
        let mut sim = Sim::new(Mood::Feliz, 1);
        sim.apply(SimCommand::SetChasing(true), 0.0);
        run(&mut sim, 0.0, 5.0, Vec2::new(300.0, 300.0), false, true);
        let tgt = Vec2::new(300.0, 300.0) - Vec2::new(-14.0, -18.0);
        let tgt2 = Vec2::new(300.0, 300.0) - Vec2::new(14.0, -18.0);
        let d = sim.pos().dist(tgt).min(sim.pos().dist(tgt2));
        assert!(d < 20.0, "dist {d}");
    }

    #[test]
    fn sad_duck_keeps_distance() {
        let mut sim = Sim::new(Mood::Triste, 1);
        sim.apply(SimCommand::SetChasing(true), 0.0);
        run(&mut sim, 0.0, 15.0, Vec2::new(300.0, 300.0), false, true);
        let d = sim.pos().dist(Vec2::new(300.0, 300.0));
        assert!(d > 180.0 && d < 300.0, "dist {d}");
    }

    #[test]
    fn sleeps_when_idle_and_wakes_when_cursor_moves() {
        let mut sim = Sim::new(Mood::Feliz, 1);
        let (now, cur, _) = run(&mut sim, 0.0, 31.0, Vec2::new(10.0, 10.0), false, true);
        assert_eq!(sim.view().mode, DuckMode::Sleep);
        sim.step(&input(cur + Vec2::new(200.0, 0.0), now, true));
        assert_eq!(sim.view().mode, DuckMode::Idle);
    }

    #[test]
    fn grab_needs_drag_enabled() {
        let mut sim = Sim::new(Mood::Feliz, 1);
        sim.apply(SimCommand::SetChasing(true), 0.0);
        run(&mut sim, 0.0, 6.0, Vec2::new(300.0, 300.0), true, true);
        assert!(!sim.is_grabbing());
    }

    #[test]
    fn grab_needs_pointer() {
        let mut sim = Sim::new(Mood::Feliz, 1);
        sim.apply(SimCommand::SetChasing(true), 0.0);
        sim.apply(SimCommand::SetDragEnabled(true), 0.0);
        run(&mut sim, 0.0, 6.0, Vec2::new(300.0, 300.0), true, false);
        assert!(!sim.is_grabbing());
    }

    #[test]
    fn grab_releases_after_ten_seconds() {
        let mut sim = Sim::new(Mood::Feliz, 1);
        sim.apply(SimCommand::SetChasing(true), 0.0);
        sim.apply(SimCommand::SetDragEnabled(true), 0.0);
        let mut now = 0.0;
        let mut cur = Vec2::new(300.0, 300.0);
        let mut grab_start = None;
        let mut released_at = None;
        while now < 30.0 && released_at.is_none() {
            let out = sim.step(&input(cur, now, true));
            if let Some(w) = out.warp_cursor {
                cur = w;
                grab_start.get_or_insert(now);
            }
            if out.released == Some(ReleaseReason::Timeout) {
                released_at = Some(now);
            }
            now += DT as f64;
        }
        let (a, b) = (grab_start.expect("never grabbed"), released_at.expect("never released"));
        assert!((b - a - MAX_GRAB_SECS).abs() < 0.2, "held for {}", b - a);
    }

    #[test]
    fn grab_releases_when_user_fights() {
        let mut sim = Sim::new(Mood::Feliz, 1);
        sim.apply(SimCommand::SetChasing(true), 0.0);
        sim.apply(SimCommand::SetDragEnabled(true), 0.0);
        let mut now = 0.0;
        let mut cur = Vec2::new(300.0, 300.0);
        let mut grabbed = false;
        let mut reason = None;
        while now < 20.0 && reason.is_none() {
            let out = sim.step(&input(cur, now, true));
            if grabbed {
                // o usuário puxa o mouse para longe do bico
                cur = cur + Vec2::new(60.0, 0.0);
            } else if let Some(w) = out.warp_cursor {
                cur = w;
                grabbed = true;
            }
            reason = out.released;
            now += DT as f64;
        }
        assert_eq!(reason, Some(ReleaseReason::Fight));
    }

    #[test]
    fn toggling_chase_releases_grab_and_cooldown_blocks_regrab() {
        let mut sim = Sim::new(Mood::Feliz, 1);
        sim.apply(SimCommand::SetChasing(true), 0.0);
        sim.apply(SimCommand::SetDragEnabled(true), 0.0);
        let mut now = 0.0;
        let mut cur = Vec2::new(300.0, 300.0);
        while !sim.is_grabbing() && now < 10.0 {
            let out = sim.step(&input(cur, now, true));
            if let Some(w) = out.warp_cursor {
                cur = w;
            }
            now += DT as f64;
        }
        assert!(sim.is_grabbing());
        sim.apply(SimCommand::SetChasing(false), now);
        assert!(!sim.is_grabbing());
        let out = sim.step(&input(cur, now, true));
        assert_eq!(out.released, Some(ReleaseReason::Command));
        // religa a perseguição logo em seguida: o cooldown (4 s) impede novo arrasto
        sim.apply(SimCommand::SetChasing(true), now);
        let (_, _, _) = run(&mut sim, now, 2.0, cur, false, true);
        assert!(!sim.is_grabbing());
    }

    #[test]
    fn mood_cycle_order() {
        let mut m = Mood::Feliz;
        let mut seen = vec![];
        for _ in 0..8 {
            m = m.next();
            seen.push(m.id());
        }
        assert_eq!(seen, ["raiva", "triste", "sozinho", "sono", "danca", "lagarto", "duvida", "feliz"]);
    }

    #[test]
    fn stays_inside_bounds() {
        let mut sim = Sim::new(Mood::Raiva, 1);
        sim.apply(SimCommand::SetChasing(true), 0.0);
        run(&mut sim, 0.0, 5.0, Vec2::new(5000.0, -3000.0), false, true);
        let p = sim.pos();
        assert!(p.x <= 1600.0 && p.y >= 0.0);
    }

    fn max_dist_from(sim: &mut Sim, start: f64, secs: f64, cursor: Vec2, center: Vec2) -> f32 {
        let mut now = start;
        let mut far = 0.0f32;
        while now < start + secs {
            let out = sim.step(&input(cursor, now, true));
            far = far.max(out.pos.dist(center));
            now += DT as f64;
        }
        far
    }

    #[test]
    fn corner_mode_stays_near_home() {
        let mut sim = Sim::new(Mood::Feliz, 3);
        // o cursor mexe de vez em quando para ele não dormir; aqui só observamos o passeio
        let far = max_dist_from(&mut sim, 0.0, 28.0, Vec2::new(10.0, 10.0), Vec2::new(1400.0, 950.0));
        assert!(far < 260.0, "andou {far} para longe do canto");
    }

    #[test]
    fn free_roam_covers_the_screen() {
        let mut sim = Sim::new(Mood::Feliz, 3);
        sim.apply(SimCommand::SetFreeRoam(true), 0.0);
        let far = max_dist_from(&mut sim, 0.0, 29.0, Vec2::new(10.0, 10.0), Vec2::new(1400.0, 950.0));
        assert!(far > 450.0, "só chegou a {far} do canto");
    }

    #[test]
    fn turning_free_roam_off_goes_back_home() {
        let mut sim = Sim::new(Mood::Feliz, 3);
        sim.apply(SimCommand::SetFreeRoam(true), 0.0);
        let mut now = 0.0;
        // mexe o cursor de leve (ele não dorme) e deixa passear pela tela
        let wiggle = |t: f64| Vec2::new(10.0 + ((t as i64 % 2) as f32) * 20.0, 10.0);
        while now < 14.0 {
            sim.step(&input(wiggle(now), now, true));
            now += DT as f64;
        }
        let away = sim.pos().dist(Vec2::new(1400.0, 950.0));
        sim.apply(SimCommand::SetFreeRoam(false), now);
        let end = now + 35.0;
        while now < end {
            sim.step(&input(wiggle(now), now, true));
            now += DT as f64;
        }
        let back = sim.pos().dist(Vec2::new(1400.0, 950.0));
        assert!(back < 300.0, "voltou só até {back} do canto (estava a {away})");
    }

    #[test]
    fn user_can_carry_the_duck_and_it_stays_there() {
        let mut sim = Sim::new(Mood::Feliz, 3);
        sim.step(&input(Vec2::new(700.0, 400.0), 0.0, true));
        let start = sim.pos();
        sim.apply(SimCommand::UserGrab(start + Vec2::new(5.0, -30.0)), 0.1);
        assert_eq!(sim.view().mode, DuckMode::Held);
        // a pessoa arrasta o cursor para o meio da tela
        let mut now = 0.1;
        let mut cur = start + Vec2::new(5.0, -30.0);
        for _ in 0..120 {
            cur = cur + Vec2::new(-5.0, -2.0);
            sim.step(&input(cur, now, true));
            now += DT as f64;
        }
        let carried = sim.pos();
        assert!(carried.dist(cur - Vec2::new(5.0, -30.0)) < 1.0, "o pato não acompanhou o cursor");
        sim.apply(SimCommand::UserDrop, now);
        let out = sim.step(&input(cur, now, true));
        assert_eq!(out.new_home.map(|h| h.dist(carried) < 1.5), Some(true));
        // depois de solto, passeia por ali (não volta para o canto)
        let far = max_dist_from(&mut sim, now, 25.0, cur, carried);
        assert!(far < 260.0, "foi para {far} do lugar novo");
        assert_ne!(sim.view().mode, DuckMode::Held);
    }

    #[test]
    fn carrying_the_duck_releases_the_mouse_grab() {
        let mut sim = Sim::new(Mood::Feliz, 1);
        sim.apply(SimCommand::SetChasing(true), 0.0);
        sim.apply(SimCommand::SetDragEnabled(true), 0.0);
        let mut now = 0.0;
        let mut cur = Vec2::new(300.0, 300.0);
        while !sim.is_grabbing() && now < 10.0 {
            let out = sim.step(&input(cur, now, true));
            if let Some(w) = out.warp_cursor {
                cur = w;
            }
            now += DT as f64;
        }
        assert!(sim.is_grabbing());
        sim.apply(SimCommand::UserGrab(cur), now);
        let out = sim.step(&input(cur, now, true));
        assert_eq!(out.released, Some(ReleaseReason::Command));
        assert!(out.warp_cursor.is_none());
        assert!(!sim.is_grabbing());
    }

    #[test]
    fn changing_home_moves_the_duck_there_and_clearing_sends_it_back() {
        let mut sim = Sim::new(Mood::Feliz, 3);
        let wiggle = |t: f64| Vec2::new(10.0 + ((t as i64 % 2) as f32) * 20.0, 10.0);
        let mut now = 0.0;
        sim.step(&input(wiggle(now), now, true)); // nasce no canto padrão
        sim.apply(SimCommand::SetHome(Some(Vec2::new(300.0, 300.0))), now);
        let end = now + 40.0;
        while now < end {
            sim.step(&input(wiggle(now), now, true));
            now += DT as f64;
        }
        let at_new = sim.pos().dist(Vec2::new(300.0, 300.0));
        assert!(at_new < 300.0, "ficou a {at_new} do lugar novo");
        sim.apply(SimCommand::SetHome(None), now);
        let end = now + 45.0;
        while now < end {
            sim.step(&input(wiggle(now), now, true));
            now += DT as f64;
        }
        let back = sim.pos().dist(Vec2::new(1400.0, 950.0));
        assert!(back < 300.0, "voltou só até {back} do canto");
    }

    fn mode_after(sim: &mut Sim, start: f64, secs: f64, cursor: Vec2) -> (f64, DuckMode) {
        let (now, _, _) = run(sim, start, secs, cursor, false, true);
        (now, sim.view().mode)
    }

    #[test]
    fn music_makes_the_duck_dance_in_place_and_stopping_ends_it() {
        let mut sim = Sim::new(Mood::Feliz, 3);
        let (now, _) = mode_after(&mut sim, 0.0, 0.5, Vec2::new(10.0, 10.0));
        let here = sim.pos();
        sim.apply(SimCommand::SetMusic(true), now);
        let out = sim.step(&input(Vec2::new(10.0, 10.0), now, true));
        assert_eq!(out.view.mode, DuckMode::Dance);
        assert!(out.active, "dançando conta como ativo");
        assert_eq!(out.view.pace, Pace::Still);
        // dança 60 s sem sair do lugar e sem dormir
        let (now, mode) = mode_after(&mut sim, now, 60.0, Vec2::new(10.0, 10.0));
        assert_eq!(mode, DuckMode::Dance);
        assert!(sim.pos().dist(here) < 0.5, "saiu do lugar dançando");
        sim.apply(SimCommand::SetMusic(false), now);
        let out = sim.step(&input(Vec2::new(10.0, 10.0), now, true));
        assert_eq!(out.view.mode, DuckMode::Idle);
        // não cai no sono logo depois de parar de dançar
        let (_, mode) = mode_after(&mut sim, now, 5.0, Vec2::new(10.0, 10.0));
        assert_ne!(mode, DuckMode::Sleep);
    }

    #[test]
    fn after_dancing_it_goes_back_to_wandering() {
        let mut sim = Sim::new(Mood::Feliz, 3);
        sim.apply(SimCommand::SetMusic(true), 0.0);
        let (now, mode) = mode_after(&mut sim, 0.0, 3.0, Vec2::new(10.0, 10.0));
        assert_eq!(mode, DuckMode::Dance);
        sim.apply(SimCommand::SetMusic(false), now);
        let wiggle = |t: f64| Vec2::new(10.0 + ((t as i64 % 2) as f32) * 20.0, 10.0);
        let mut now = now;
        let mut walked = false;
        while now < 40.0 {
            walked |= sim.step(&input(wiggle(now), now, true)).view.mode == DuckMode::Walk;
            now += DT as f64;
        }
        assert!(walked, "não voltou a passear depois da dança");
    }

    #[test]
    fn dancing_mood_dances_without_music() {
        let mut sim = Sim::new(Mood::Danca, 3);
        let (_, mode) = mode_after(&mut sim, 0.0, 120.0, Vec2::new(10.0, 10.0));
        assert_eq!(mode, DuckMode::Dance, "humor Dançando dança sozinho (e não dorme)");
        sim.apply(SimCommand::SetMood(Mood::Feliz), 120.0);
        assert_eq!(sim.step(&input(Vec2::new(10.0, 10.0), 120.0, true)).view.mode, DuckMode::Idle);
    }

    #[test]
    fn music_with_dance_on_music_off_does_not_dance() {
        let mut sim = Sim::new(Mood::Feliz, 3);
        sim.apply(SimCommand::SetDanceOnMusic(false), 0.0);
        sim.apply(SimCommand::SetMusic(true), 0.0);
        let (now, mode) = mode_after(&mut sim, 0.0, 3.0, Vec2::new(10.0, 10.0));
        assert_ne!(mode, DuckMode::Dance);
        // religar a preferência com o som ainda tocando: dança
        sim.apply(SimCommand::SetDanceOnMusic(true), now);
        assert_eq!(sim.step(&input(Vec2::new(10.0, 10.0), now, true)).view.mode, DuckMode::Dance);
        sim.apply(SimCommand::SetDanceOnMusic(false), now);
        assert_eq!(sim.step(&input(Vec2::new(10.0, 10.0), now, true)).view.mode, DuckMode::Idle);
    }

    #[test]
    fn focus_sleeps_at_once_ignores_cursor_and_wakes_after() {
        let mut sim = Sim::new(Mood::Danca, 1);
        sim.apply(SimCommand::SetChasing(true), 0.0);
        let (now, mode) = mode_after(&mut sim, 0.0, 1.0, Vec2::new(300.0, 300.0));
        assert_eq!(mode, DuckMode::Chase);
        sim.apply(SimCommand::SetFocus(true), now);
        let (now, mode) = mode_after(&mut sim, now, 0.5, Vec2::new(300.0, 300.0));
        assert_eq!(mode, DuckMode::Sleep);
        // o cursor mexendo não acorda nem faz perseguir
        let (now, mode) = mode_after(&mut sim, now, 2.0, Vec2::new(900.0, 100.0));
        assert_eq!(mode, DuckMode::Sleep);
        sim.apply(SimCommand::SetFocus(false), now);
        let (_, mode) = mode_after(&mut sim, now, 0.5, Vec2::new(900.0, 100.0));
        assert_ne!(mode, DuckMode::Sleep, "acorda quando o foco acaba");
    }

    #[test]
    fn music_does_not_interrupt_chase_and_chase_wins_over_dance() {
        let mut sim = Sim::new(Mood::Danca, 1);
        sim.apply(SimCommand::SetMusic(true), 0.0);
        let (now, mode) = mode_after(&mut sim, 0.0, 1.0, Vec2::new(300.0, 300.0));
        assert_eq!(mode, DuckMode::Dance);
        sim.apply(SimCommand::SetChasing(true), now);
        let (now, mode) = mode_after(&mut sim, now, 3.0, Vec2::new(300.0, 300.0));
        assert_eq!(mode, DuckMode::Chase);
        sim.apply(SimCommand::SetChasing(false), now);
        let (_, mode) = mode_after(&mut sim, now, 0.5, Vec2::new(300.0, 300.0));
        assert_eq!(mode, DuckMode::Dance, "parou de perseguir com som tocando: volta a dançar");
    }

    #[test]
    fn music_does_not_interrupt_grab_or_held() {
        let mut sim = Sim::new(Mood::Feliz, 1);
        sim.apply(SimCommand::SetChasing(true), 0.0);
        sim.apply(SimCommand::SetDragEnabled(true), 0.0);
        let mut now = 0.0;
        let mut cur = Vec2::new(300.0, 300.0);
        while !sim.is_grabbing() && now < 10.0 {
            if let Some(w) = sim.step(&input(cur, now, true)).warp_cursor {
                cur = w;
            }
            now += DT as f64;
        }
        assert!(sim.is_grabbing());
        sim.apply(SimCommand::SetMusic(true), now);
        let out = sim.step(&input(cur, now, true));
        assert_eq!(out.view.mode, DuckMode::Grab);
        // carregado no colo também não dança
        sim.apply(SimCommand::UserGrab(cur), now);
        assert_eq!(sim.step(&input(cur, now, true)).view.mode, DuckMode::Held);
        sim.apply(SimCommand::SetChasing(false), now);
        sim.apply(SimCommand::UserDrop, now);
        assert_eq!(sim.step(&input(cur, now, true)).view.mode, DuckMode::Dance, "solto com som tocando: dança");
    }

    #[test]
    fn music_does_not_wake_a_sleeping_duck_but_it_dances_on_waking() {
        let mut sim = Sim::new(Mood::Feliz, 1);
        let (now, cur, _) = run(&mut sim, 0.0, 31.0, Vec2::new(10.0, 10.0), false, true);
        assert_eq!(sim.view().mode, DuckMode::Sleep);
        sim.apply(SimCommand::SetMusic(true), now);
        let (now, mode) = mode_after(&mut sim, now, 5.0, cur);
        assert_eq!(mode, DuckMode::Sleep, "som não acorda o pato");
        sim.step(&input(cur + Vec2::new(200.0, 0.0), now, true));
        assert_eq!(sim.view().mode, DuckMode::Dance, "acordou com som tocando: dança");
    }

    #[test]
    fn starts_at_saved_home() {
        let mut sim = Sim::new(Mood::Feliz, 3);
        sim.apply(SimCommand::SetHome(Some(Vec2::new(300.0, 400.0))), 0.0);
        let out = sim.step(&input(Vec2::new(10.0, 10.0), 0.0, true));
        assert!(out.pos.dist(Vec2::new(300.0, 400.0)) < 1.0, "nasceu em {:?}", out.pos);
    }

    // ---- migalha ----

    /// Roda até `secs` segundos coletando os eventos de migalha; devolve (agora, eventos, algum warp?).
    fn run_crumb(sim: &mut Sim, start: f64, secs: f64, cursor: Vec2) -> (f64, Vec<CrumbEvent>, bool) {
        let mut now = start;
        let mut events = vec![];
        let mut warped = false;
        while now < start + secs {
            let out = sim.step(&input(cursor, now, true));
            warped |= out.warp_cursor.is_some();
            if let Some(e) = out.crumb {
                events.push(e);
            }
            now += DT as f64;
        }
        (now, events, warped)
    }

    fn calm_sim(mood: Mood) -> (Sim, f64) {
        let mut sim = Sim::new(mood, 3);
        let (now, _, _) = run(&mut sim, 0.0, 1.0, Vec2::new(200.0, 200.0), false, true);
        (sim, now)
    }

    #[test]
    fn crumb_outside_the_work_area_lands_near_the_duck() {
        let b = Rect::new(0.0, 25.0, 1600.0, 975.0);
        let inside = Vec2::new(300.0, 400.0);
        assert_eq!(crumb_spot(inside, b, Vec2::new(800.0, 900.0), 1.0), inside);
        // cursor na barra de menus (acima da área útil)
        let spot = crumb_spot(Vec2::new(1500.0, 5.0), b, Vec2::new(200.0, 900.0), 1.0);
        assert!(spot.x > 200.0 && spot.x <= 400.0 && b.clamp(spot) == spot, "{spot:?}");
        // pato perto da borda direita: a migalha vai para a esquerda, dentro da área
        let spot = crumb_spot(Vec2::new(-50.0, 5.0), b, Vec2::new(1590.0, 900.0), 1.0);
        assert!(spot.x < 1590.0 && b.clamp(spot) == spot, "{spot:?}");
    }

    #[test]
    fn crumb_is_walked_to_eaten_and_the_duck_goes_back_to_idle() {
        let (mut sim, now) = calm_sim(Mood::Feliz);
        let target = Vec2::new(300.0, 400.0);
        sim.apply(SimCommand::DropCrumb, now);
        let (end, events, warped) = run_crumb(&mut sim, now, 12.0, target);
        assert!(!warped, "migalha nunca mexe no mouse");
        assert_eq!(events.first(), Some(&CrumbEvent::Dropped(target)));
        assert_eq!(events.get(1), Some(&CrumbEvent::Eaten(target)));
        assert_eq!(events.len(), 2, "{events:?}");
        // o bico ficou na migalha (pés abaixo dela, como na perseguição)
        let beak = sim.pos() + Vec2::new(BEAK.x * if sim.view().facing == Facing::Right { 1.0 } else { -1.0 }, BEAK.y);
        assert!(beak.dist(target) < 8.0, "bico em {beak:?}");
        // depois de comer volta a ficar parado/passeando e a migalha acabou
        assert!(matches!(sim.view().mode, DuckMode::Idle | DuckMode::Walk));
        assert!(end > now);
        assert!(sim.crumb.is_none());
    }

    #[test]
    fn crumb_shows_walk_then_eat_while_eating() {
        let (mut sim, now) = calm_sim(Mood::Feliz);
        sim.apply(SimCommand::DropCrumb, now);
        let target = Vec2::new(900.0, 300.0);
        let out = sim.step(&input(target, now, true));
        assert_eq!(out.view.mode, DuckMode::Walk);
        assert!(out.active);
        let mut t = now + 0.02;
        let mut saw_eating_idle = false;
        while t < now + 10.0 {
            let o = sim.step(&input(target, t, true));
            if matches!(o.crumb, Some(CrumbEvent::Eaten(_))) {
                assert_eq!(o.view.mode, DuckMode::Eat);
                saw_eating_idle = true;
            }
            t += DT as f64;
        }
        assert!(saw_eating_idle);
        let after = sim.step(&input(target, t + 2.0, true));
        assert_ne!(after.view.mode, DuckMode::Eat, "depois de comer volta ao normal");
    }

    #[test]
    fn sleepy_mood_ignores_the_crumb() {
        let (mut sim, now) = calm_sim(Mood::Sono);
        let before = sim.pos();
        sim.apply(SimCommand::DropCrumb, now);
        let (_, events, _) = run_crumb(&mut sim, now, 3.0, Vec2::new(900.0, 300.0));
        assert!(events.is_empty(), "{events:?}");
        assert!(sim.crumb.is_none());
        assert!(sim.pos().dist(before) < 1.0, "não saiu do lugar");
    }

    #[test]
    fn becoming_sleepy_drops_the_crumb() {
        let (mut sim, now) = calm_sim(Mood::Feliz);
        sim.apply(SimCommand::DropCrumb, now);
        let out = sim.step(&input(Vec2::new(900.0, 300.0), now, true));
        assert!(matches!(out.crumb, Some(CrumbEvent::Dropped(_))));
        sim.apply(SimCommand::SetMood(Mood::Sono), now + 0.1);
        let out = sim.step(&input(Vec2::new(900.0, 300.0), now + 0.1, true));
        assert_eq!(out.crumb, Some(CrumbEvent::Gone));
        assert!(sim.crumb.is_none());
        assert_ne!(sim.view().mode, DuckMode::Walk);
    }

    #[test]
    fn crumb_resumes_chasing_afterwards() {
        let (mut sim, now) = calm_sim(Mood::Feliz);
        sim.apply(SimCommand::SetChasing(true), now);
        sim.apply(SimCommand::DropCrumb, now);
        let target = Vec2::new(700.0, 500.0);
        let (end, events, _) = run_crumb(&mut sim, now, 8.0, target);
        assert!(events.contains(&CrumbEvent::Eaten(target)));
        assert_eq!(sim.view().mode, DuckMode::Chase);
        assert!(end > now);
    }

    #[test]
    fn crumb_is_ignored_while_held_or_grabbing_and_cancelled_by_a_grab() {
        let (mut sim, now) = calm_sim(Mood::Feliz);
        sim.apply(SimCommand::UserGrab(Vec2::new(210.0, 210.0)), now);
        sim.apply(SimCommand::DropCrumb, now);
        let (_, events, _) = run_crumb(&mut sim, now, 0.5, Vec2::new(300.0, 300.0));
        assert!(events.is_empty());
        assert_eq!(sim.view().mode, DuckMode::Held);
        // pegar o pato no meio do caminho cancela a migalha
        sim.apply(SimCommand::UserDrop, now + 1.0);
        sim.apply(SimCommand::DropCrumb, now + 1.0);
        sim.step(&input(Vec2::new(900.0, 300.0), now + 1.0, true));
        sim.apply(SimCommand::UserGrab(Vec2::new(900.0, 300.0)), now + 1.1);
        let out = sim.step(&input(Vec2::new(900.0, 300.0), now + 1.1, true));
        assert_eq!(out.crumb, Some(CrumbEvent::Gone));
        assert_eq!(sim.view().mode, DuckMode::Held);
    }

    #[test]
    fn crumb_gives_up_after_the_timeout() {
        let (mut sim, now) = calm_sim(Mood::Feliz);
        sim.apply(SimCommand::DropCrumb, now);
        sim.step(&input(Vec2::new(900.0, 300.0), now, true));
        let out = sim.step(&input(Vec2::new(900.0, 300.0), now + CRUMB_TIMEOUT + 1.0, true));
        assert_eq!(out.crumb, Some(CrumbEvent::Gone));
        assert!(sim.crumb.is_none());
        assert_ne!(sim.view().mode, DuckMode::Walk);
    }

    #[test]
    fn a_new_crumb_replaces_the_old_one() {
        let (mut sim, now) = calm_sim(Mood::Feliz);
        sim.apply(SimCommand::DropCrumb, now);
        sim.step(&input(Vec2::new(500.0, 300.0), now, true));
        sim.apply(SimCommand::DropCrumb, now + 0.1);
        let out = sim.step(&input(Vec2::new(800.0, 300.0), now + 0.1, true));
        assert_eq!(out.crumb, Some(CrumbEvent::Dropped(Vec2::new(800.0, 300.0))));
        // come: durante a comida um pedido novo é ignorado
        let (t, events, _) = run_crumb(&mut sim, now + 0.2, 10.0, Vec2::new(800.0, 300.0));
        assert!(events.iter().any(|e| matches!(e, CrumbEvent::Eaten(_))));
        let _ = t;
    }
}
