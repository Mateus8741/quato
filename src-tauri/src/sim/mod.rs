pub mod model;
pub mod mood;
pub mod vec2;

use crate::events::EV_ACCESSIBILITY_CHANGED;
use crate::platform::{self, MonitorSim};
use crate::pointer::Pointer;
use crate::state::{AppState, DuckGeom};
use model::{Sim, SimCommand, SimInput};
use mood::Mood;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::mpsc::{Receiver, RecvTimeoutError};
use std::time::{Duration, Instant};
use tauri::{AppHandle, Emitter, Manager};
use vec2::{Rect, Vec2};

const ACTIVE_TICK: Duration = Duration::from_millis(16);
const SETTLED_TICK: Duration = Duration::from_millis(50);
const IDLE_TICK: Duration = Duration::from_millis(500);
/// Trava extra, fora da simulação: nunca mexe no mouse por mais que isso.
const HARD_GRAB_LIMIT: f64 = 10.5;

struct Runtime {
    app: AppHandle,
    sim: Sim,
    pointer: Option<Pointer>,
    pointer_failed: bool,
    trusted_cache: (Instant, bool),
    monitors: (Instant, Vec<MonitorSim>),
    size_checked: Instant,
    last_view: Option<crate::events::DuckView>,
    last_pos: Vec2,
    shown: bool,
    grab_started: Option<Instant>,
    /// Arminha equipada? (relida da config a cada ~1 s, sem travar o tick)
    armed: (Instant, bool),
    burst: crate::fire::Burst,
    fx: crate::fx::Fx,
    /// Vigia do modo "não perturbe" e se a janela já foi escondida por ele.
    dnd: crate::dnd::Watcher,
    dnd_applied: bool,
}

pub fn spawn(app: AppHandle, rx: Receiver<SimCommand>, mood: Mood) {
    std::thread::Builder::new()
        .name("quato-sim".into())
        .spawn(move || {
            let seed = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_nanos() as u64).unwrap_or(7);
            let mut rt = Runtime {
                sim: Sim::new(mood, seed),
                app,
                pointer: None,
                pointer_failed: false,
                trusted_cache: (Instant::now() - Duration::from_secs(10), false),
                monitors: (Instant::now() - Duration::from_secs(10), vec![]),
                size_checked: Instant::now() - Duration::from_secs(10),
                last_view: None,
                last_pos: Vec2::new(-9999.0, -9999.0),
                shown: false,
                grab_started: None,
                armed: (Instant::now() - Duration::from_secs(10), false),
                burst: crate::fire::Burst::new(seed ^ 0x9e37_79b9),
                fx: crate::fx::Fx::new(),
                dnd: crate::dnd::Watcher::new(),
                dnd_applied: false,
            };
            run(&mut rt, rx);
        })
        .expect("não consegui iniciar a thread da simulação");
}

fn run(rt: &mut Runtime, rx: Receiver<SimCommand>) {
    let start = Instant::now();
    let mut last = Instant::now();
    let mut tempo = IDLE_TICK;
    loop {
        let timeout = tempo;
        let mut shutdown = false;
        match rx.recv_timeout(timeout) {
            Ok(cmd) => {
                let now = start.elapsed().as_secs_f64();
                shutdown |= matches!(cmd, SimCommand::Shutdown);
                rt.sim.apply(cmd, now);
                while let Ok(c) = rx.try_recv() {
                    shutdown |= matches!(c, SimCommand::Shutdown);
                    rt.sim.apply(c, now);
                }
            }
            Err(RecvTimeoutError::Timeout) => {}
            Err(RecvTimeoutError::Disconnected) => break,
        }
        if shutdown {
            break;
        }
        let dt = last.elapsed().as_secs_f32().min(0.05);
        last = Instant::now();
        let now = start.elapsed().as_secs_f64();
        let result = catch_unwind(AssertUnwindSafe(|| tick(rt, dt, now)));
        match result {
            Ok(t) => tempo = t,
            Err(_) => {
                eprintln!("[quato] panic na simulação; soltando o mouse e voltando ao normal");
                rt.sim.panic_reset();
                rt.grab_started = None;
                tempo = IDLE_TICK;
            }
        }
    }
}

fn trusted(rt: &mut Runtime) -> bool {
    if rt.trusted_cache.0.elapsed() > Duration::from_secs(1) {
        let t = crate::accessibility::is_trusted();
        if t != rt.trusted_cache.1 {
            let _ = rt.app.emit(EV_ACCESSIBILITY_CHANGED, crate::accessibility::status());
            crate::actions::emit_settings(&rt.app);
        }
        rt.trusted_cache = (Instant::now(), t);
    }
    rt.trusted_cache.1
}

fn armed(rt: &mut Runtime) -> bool {
    if rt.armed.0.elapsed() > Duration::from_secs(1) {
        let on = rt.app.state::<AppState>().config.lock().map(|c| crate::fire::is_armed(&c.accessories)).unwrap_or(false);
        rt.armed = (Instant::now(), on);
    }
    rt.armed.1
}

/// Arminha: atira no cursor (só visual). Devolve em quantos segundos precisa acordar de novo para o próximo tiro.
fn gunplay(rt: &mut Runtime, out: &model::SimOutput, all: &[MonitorSim], cursor: Vec2, unit_scale: f32, now: f64) -> Option<f64> {
    let armed = armed(rt);
    if !armed && !rt.fx.busy(&rt.app) {
        rt.sim.set_aiming(false);
        return None;
    }
    let mon = platform::monitor_at(all, out.pos);
    // O cursor precisa estar no mesmo monitor do pato (a janela de efeitos cobre só ele) e longe o bastante.
    let eligible = match mon {
        Some(m) => {
            crate::fire::can_fire(armed, out.view.mood, out.view.mode)
                && cursor.x >= m.rect.x
                && cursor.x <= m.rect.right()
                && cursor.y >= m.rect.y
                && cursor.y <= m.rect.bottom()
                && cursor.dist(out.pos) > crate::fire::MIN_DIST * unit_scale
        }
        None => false,
    };
    rt.sim.set_aiming(eligible);
    if eligible {
        rt.fx.want(&rt.app, now);
    }
    if rt.burst.poll(now, eligible) {
        if let Some(m) = mon {
            let shot = crate::fire::shot_geometry(out.pos, out.view.facing, unit_scale, cursor, rt.burst.rng());
            rt.fx.shoot(&rt.app, now, &m, &shot);
        }
    }
    rt.fx.housekeeping(&rt.app, now);
    let a = rt.burst.due(now);
    let b = rt.fx.next_check(now);
    match (a, b) {
        (Some(x), Some(y)) => Some(x.min(y)),
        (x, y) => x.or(y),
    }
}

/// Um passo. Retorna quanto esperar até o próximo (60 Hz ativo, 20 Hz colado no cursor, 2 Hz parado).
fn tick(rt: &mut Runtime, dt: f32, now: f64) -> Duration {
    if !crate::windows::duck_ready() {
        return IDLE_TICK;
    }
    let Some(win) = rt.app.get_webview_window("duck") else { return IDLE_TICK };

    if rt.monitors.0.elapsed() > Duration::from_secs(2) || rt.monitors.1.is_empty() {
        rt.monitors = (Instant::now(), platform::monitors(&rt.app));
    }
    let all = rt.monitors.1.clone();
    let Some(prim) = platform::primary(&rt.app, &all) else { return IDLE_TICK };
    let Some(bounds) = platform::union_work(&all) else { return IDLE_TICK };
    let Some(cursor) = platform::cursor(&rt.app, prim.scale) else { return IDLE_TICK };

    // Modo "não perturbe": com app em tela cheia no monitor do pato, só esconde as janelas e para
    // a simulação (nada de perseguir nem mexer no mouse). Ao voltar, `rt.shown = false` mostra de novo.
    if rt.dnd.poll(&rt.app, &all, rt.sim.pos()) {
        if !rt.dnd_applied {
            rt.dnd_applied = true;
            rt.sim.apply(SimCommand::ReleaseMouse, now);
            rt.grab_started = None;
            let _ = win.hide();
            rt.fx.suspend(&rt.app);
            rt.shown = false;
            rt.app.state::<AppState>().duck_geom.lock().unwrap().shown = false;
        }
        return Duration::from_millis(250);
    }
    rt.dnd_applied = false;

    let unit_scale = platform::unit_scale(&all, rt.sim.pos());
    // Canto padrão: afastado o bastante para a plaquinha (192 de largura) não sair da tela.
    let home = Vec2::new(prim.work.right() - (platform::WIN_SIZE / 2.0 + 4.0) * unit_scale, prim.work.bottom() - 6.0 * unit_scale);
    // O pato precisa caber na tela: os pés ficam entre o topo (+altura da janela acima da âncora) e a base.
    let pad_top = platform::ANCHOR.y * unit_scale;
    let bounds = Rect::new(bounds.x + 20.0 * unit_scale, bounds.y + pad_top, bounds.w - 40.0 * unit_scale, bounds.h - pad_top);

    // Windows: garante que a janela tenha mesmo 192 unidades de design em pixels físicos. Se o DPI mudou
    // (outro monitor, escala trocada) ela pode ficar menor e o pato aparece cortado embaixo/na direita.
    #[cfg(not(target_os = "macos"))]
    if !rt.shown || rt.size_checked.elapsed() > Duration::from_secs(1) {
        rt.size_checked = Instant::now();
        let want = (platform::WIN_SIZE * unit_scale).round().max(1.0) as u32;
        if let Ok(cur) = win.inner_size() {
            if cur.width.abs_diff(want) > 1 || cur.height.abs_diff(want) > 1 {
                eprintln!("[quato] janela com {}x{} (esperado {want}x{want}, escala {unit_scale}); corrigindo", cur.width, cur.height);
                let _ = win.set_size(tauri::PhysicalSize::new(want, want));
            }
        }
    }

    let pointer_ok = !rt.pointer_failed && trusted(rt);
    let out = rt.sim.step(&SimInput { dt, now, cursor, bounds, home, unit_scale, pointer_ok });

    // Arrastar o mouse: só move o cursor, nunca clica.
    if let Some(target) = out.warp_cursor {
        let started = *rt.grab_started.get_or_insert_with(Instant::now);
        if started.elapsed().as_secs_f64() > HARD_GRAB_LIMIT {
            rt.sim.apply(SimCommand::ReleaseMouse, now);
            rt.grab_started = None;
        } else {
            if rt.pointer.is_none() {
                match Pointer::new() {
                    Ok(p) => rt.pointer = Some(p),
                    Err(e) => {
                        eprintln!("[quato] não consegui controlar o mouse: {e}");
                        rt.pointer_failed = true;
                    }
                }
            }
            if let Some(p) = rt.pointer.as_mut() {
                if let Err(e) = p.move_to(target.x, target.y) {
                    eprintln!("[quato] erro ao mover o mouse: {e}");
                    rt.pointer_failed = true;
                }
            }
        }
    } else {
        rt.grab_started = None;
    }
    if out.released.is_some() {
        eprintln!("[quato] mouse solto: {:?}", out.released);
    }

    if out.pos.dist(rt.last_pos) > 0.4 || !rt.shown {
        platform::place_window(&win, out.pos, unit_scale);
        rt.last_pos = out.pos;
        // O vigia de Ctrl+arrastar precisa saber onde o pato está.
        *rt.app.state::<AppState>().duck_geom.lock().unwrap() = DuckGeom { pos: out.pos, scale: unit_scale, shown: true };
    }
    if let Some(h) = out.new_home {
        crate::actions::home_changed(&rt.app, h);
    }
    if !rt.shown {
        let _ = win.show();
        let _ = win.set_ignore_cursor_events(true);
        let _ = win.set_always_on_top(true);
        rt.shown = true;
    }
    if rt.last_view != Some(out.view) {
        rt.last_view = Some(out.view);
        crate::actions::on_view_changed(&rt.app, out.view);
    }
    let wake = gunplay(rt, &out, &all, cursor, unit_scale, now);
    let base = if !out.active {
        IDLE_TICK
    } else if out.settled {
        SETTLED_TICK
    } else {
        ACTIVE_TICK
    };
    // Só acorda mais cedo quando há tiro agendado: sem arma o tempo é o de sempre.
    match wake {
        Some(w) => base.min(Duration::from_secs_f64(w.max(0.016))),
        None => base,
    }
}
