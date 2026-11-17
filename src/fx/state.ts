// Estado puro dos efeitos da arminha (sem DOM): rastro, marcas de tiro e estojos.
// Tempos em segundos, posições em pixels CSS relativos ao canto da janela fx.

export interface Pt {
  x: number;
  y: number;
}

/** Espelho de `FxShot` em src-tauri/src/fx.rs. */
export interface FxShot {
  from: Pt;
  to: Pt;
  eject: Pt;
  /** +1 / -1: lado para onde os estojos saem. */
  ejectDir: number;
  /** y do chão (pés do pato). */
  floor: number;
}

export const MAX_MARKS = 60;
export const MAX_SHELLS = 40;
/** A marca fica inteira por MARK_HOLD s e some em MARK_FADE s (~20 s no total). */
export const MARK_HOLD = 14;
export const MARK_FADE = 6;
export const TRACER_LIFE = 0.12;
export const FLASH_LIFE = 0.07;
export const GRAVITY = 1500;
/** Estojo parado: fica SHELL_REST s e some em SHELL_FADE s. */
export const SHELL_REST = 6.5;
export const SHELL_FADE = 1.5;

export interface Mark {
  x: number;
  y: number;
  born: number;
  seed: number;
}

export interface Shell {
  x: number;
  y: number;
  vx: number;
  vy: number;
  rot: number;
  vr: number;
  floor: number;
  bounces: number;
  maxBounces: number;
  born: number;
  /** Quando parou no chão (null = ainda se mexendo). */
  restAt: number | null;
}

export interface Tracer {
  from: Pt;
  to: Pt;
  born: number;
}

export type Rnd = () => number;

export function markAlpha(age: number): number {
  if (age <= MARK_HOLD) return 1;
  return Math.max(0, 1 - (age - MARK_HOLD) / MARK_FADE);
}

export function shellAlpha(s: Shell, now: number): number {
  if (s.restAt === null) return 1;
  const t = now - s.restAt - SHELL_REST;
  if (t <= 0) return 1;
  return Math.max(0, 1 - t / SHELL_FADE);
}

export function makeShell(shot: FxShot, now: number, rnd: Rnd): Shell {
  const dir = shot.ejectDir >= 0 ? 1 : -1;
  return {
    x: shot.eject.x,
    y: shot.eject.y,
    vx: dir * (70 + rnd() * 90),
    vy: -(200 + rnd() * 140),
    rot: rnd() * Math.PI * 2,
    vr: (rnd() < 0.5 ? -1 : 1) * (12 + rnd() * 14),
    floor: shot.floor + (rnd() * 8 - 3),
    bounces: 0,
    maxBounces: 1 + Math.floor(rnd() * 2),
    born: now,
    restAt: null,
  };
}

/** Avança um estojo. Quica `maxBounces` vezes e para. */
export function stepShell(s: Shell, dt: number, now: number): void {
  if (s.restAt !== null) return;
  s.vy += GRAVITY * dt;
  s.x += s.vx * dt;
  s.y += s.vy * dt;
  s.rot += s.vr * dt;
  if (s.y >= s.floor) {
    s.y = s.floor;
    if (s.bounces >= s.maxBounces || Math.abs(s.vy) < 60) {
      s.vx = 0;
      s.vy = 0;
      s.vr = 0;
      s.restAt = now;
    } else {
      s.bounces += 1;
      s.vy = -s.vy * 0.42;
      s.vx *= 0.55;
      s.vr *= 0.5;
    }
  }
}

export class FxState {
  marks: Mark[] = [];
  shells: Shell[] = [];
  tracers: Tracer[] = [];
  private rnd: Rnd;

  constructor(rnd: Rnd = Math.random) {
    this.rnd = rnd;
  }

  addShot(shot: FxShot, now: number): void {
    this.tracers.push({ from: shot.from, to: shot.to, born: now });
    // A marca aparece quando o rastro chega (o desenho a esconde até lá).
    this.marks.push({ x: shot.to.x, y: shot.to.y, born: now + TRACER_LIFE * 0.4, seed: Math.floor(this.rnd() * 0x7fffffff) });
    if (this.marks.length > MAX_MARKS) this.marks.splice(0, this.marks.length - MAX_MARKS);
    this.shells.push(makeShell(shot, now, this.rnd));
    if (this.shells.length > MAX_SHELLS) this.shells.splice(0, this.shells.length - MAX_SHELLS);
  }

  step(dt: number, now: number): void {
    for (const s of this.shells) stepShell(s, dt, now);
    this.tracers = this.tracers.filter((t) => now - t.born < TRACER_LIFE);
    this.marks = this.marks.filter((m) => now - m.born < MARK_HOLD + MARK_FADE);
    this.shells = this.shells.filter((s) => s.restAt === null || now - s.restAt < SHELL_REST + SHELL_FADE);
  }

  /** Precisa de quadros rápidos (algo se mexendo)? */
  animating(): boolean {
    return this.tracers.length > 0 || this.shells.some((s) => s.restAt === null);
  }

  empty(): boolean {
    return this.marks.length === 0 && this.shells.length === 0 && this.tracers.length === 0;
  }

  clear(): void {
    this.marks = [];
    this.shells = [];
    this.tracers = [];
  }
}
