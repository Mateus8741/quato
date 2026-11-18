import type { DuckView, Look } from '../shared/types.ts';
import { DEFAULT_LOOK, characterFor, framesFor } from '../sprites/compose.ts';
import { SKINS } from '../sprites/index.ts';
import { EFFECTS } from '../sprites/effects.ts';
import { mouthToTopX } from '../sprites/overlay.ts';
import { doubtState } from '../sprites/doubt.ts';
import { screamState, shakeOffset } from '../sprites/scream.ts';
import type { AnimName, EffectName } from '../sprites/types.ts';
import type { Renderer } from './renderer.ts';
import type { TopStrip } from './top.ts';

export function animFor(view: DuckView): AnimName {
  if (view.mode === 'sleep') return 'sleep';
  if (view.mode === 'grab' || view.mode === 'held') return 'grab';
  if (view.mode === 'dance') return 'dance';
  if (view.mode === 'eat') return 'eat';
  if (view.pace === 'fast') return 'run';
  if (view.pace === 'slow') return 'walk';
  return 'idle';
}

/** Ritmo do grito (tremedeira + gota pingando mais rápido). */
const SCREAM_FPS = 10;
const SCREAM_DRIP_BOOST = 3;
const DEFAULT_GESTURE_MS = 2500;

/** Anima com setTimeout em fps baixo. Parado, quase não gasta CPU. */
export class Animator {
  private view: DuckView | null = null;
  private look: Look = DEFAULT_LOOK;
  private frame = 0;
  private tick = 0;
  private timer: number | undefined;
  private blinkTimer: number | undefined;
  private blinking = false;
  private gestureTimer: number | undefined;
  private gesturing = false;
  // grito do lagartinho
  private screamTimer: number | undefined;
  private screamStart = 0;
  private screamSeed = 0;
  private screamOn = false;
  // dúvida aleatória
  private doubtTimer: number | undefined;
  private doubtStart = 0;
  private doubtSeed = 0;
  private doubtOn = false;

  private renderer: Renderer;
  private top: TopStrip | null;

  constructor(renderer: Renderer, top: TopStrip | null = null) {
    this.renderer = renderer;
    this.top = top;
  }

  setLook(look: Look): void {
    this.look = look;
    this.paint();
  }

  update(view: DuckView): void {
    const prev = this.view;
    const changed = !prev || prev.mood !== view.mood || animFor(prev) !== animFor(view);
    this.view = view;
    if (changed) this.frame = 0;
    if (!prev || prev.mood !== view.mood) { this.resetScream(); this.resetDoubt(); }
    this.restart();
    this.scheduleBlink();
  }

  /** Evento duck:gesture: levanta a "pena do meio" por um tempo. */
  gesture(durationMs: number): void {
    window.clearTimeout(this.gestureTimer);
    this.gesturing = true;
    this.paint();
    this.gestureTimer = window.setTimeout(() => {
      this.gesturing = false;
      this.paint();
    }, durationMs > 0 ? durationMs : DEFAULT_GESTURE_MS);
  }

  private screaming(): boolean {
    return this.screamOn && this.view?.mode !== 'sleep';
  }

  /** Liga a máquina do grito quando o humor é 'lagarto' (um único setTimeout até a próxima troca). */
  private resetScream(): void {
    window.clearTimeout(this.screamTimer);
    this.screamOn = false;
    this.top?.setBalloon(false);
    if (!this.view || characterFor(this.view.mood) !== 'lizard') return;
    this.screamStart = Date.now();
    this.screamSeed = Math.floor(Math.random() * 0x7fffffff);
    this.scheduleScream();
  }

  private scheduleScream(): void {
    const now = Date.now();
    const st = screamState(now, this.screamStart, this.screamSeed);
    const was = this.screaming();
    this.screamOn = st.screaming;
    if (was !== this.screaming()) this.restart();
    else this.syncBalloon();
    this.screamTimer = window.setTimeout(() => this.scheduleScream(), Math.max(50, st.until - now));
  }

  /** Liga a dúvida aleatória (um único setTimeout até a próxima troca; 'duvida' já mostra sempre). */
  private resetDoubt(): void {
    window.clearTimeout(this.doubtTimer);
    this.doubtOn = false;
    if (!this.view || this.view.mood === 'duvida') return;
    this.doubtStart = Date.now();
    this.doubtSeed = Math.floor(Math.random() * 0x7fffffff);
    this.scheduleDoubt();
  }

  private scheduleDoubt(): void {
    const now = Date.now();
    const st = doubtState(now, this.doubtStart, this.doubtSeed);
    const was = this.effectName();
    this.doubtOn = st.doubting;
    if (was !== this.effectName()) this.restart();
    this.doubtTimer = window.setTimeout(() => this.scheduleDoubt(), Math.max(50, st.until - now));
  }

  private effectName(): EffectName {
    const v = this.view!;
    if (this.doubtOn && v.mode !== 'sleep' && !this.screaming()) return 'question';
    return SKINS[v.mood].effect;
  }

  private effectFps(): number {
    const fps = EFFECTS[this.effectName()].fps;
    return this.screaming() ? fps * SCREAM_DRIP_BOOST : fps;
  }

  private fps(): number {
    const v = this.view!;
    if (this.screaming()) return SCREAM_FPS;
    const anim = animFor(v);
    // Parado só precisa do ritmo do efeito.
    if (anim === 'idle') return Math.min(this.effectFps(), 4);
    return SKINS[v.mood].fps[anim];
  }

  private restart(): void {
    window.clearTimeout(this.timer);
    this.paint();
    if (!this.view) return;
    const loop = () => {
      this.tick++;
      this.frame++;
      this.paint();
      this.timer = window.setTimeout(loop, 1000 / this.fps());
    };
    this.timer = window.setTimeout(loop, 1000 / this.fps());
  }

  private scheduleBlink(): void {
    window.clearTimeout(this.blinkTimer);
    const v = this.view;
    if (!v || v.mode === 'sleep') return;
    this.blinkTimer = window.setTimeout(() => {
      this.blinking = true;
      this.paint();
      window.setTimeout(() => {
        this.blinking = false;
        this.paint();
        this.scheduleBlink();
      }, 150);
    }, 3000 + Math.random() * 3000);
  }

  private currentFrame() {
    const v = this.view!;
    const frames = framesFor(v.mood, animFor(v), this.screaming());
    return frames[this.frame % frames.length]!;
  }

  private syncBalloon(): void {
    if (!this.top || !this.view) return;
    if (!this.screaming()) {
      this.top.setBalloon(false);
      return;
    }
    const f = this.currentFrame();
    this.top.setBalloon(true, mouthToTopX(f.eye.x + 5, this.view.facing));
  }

  private paint(): void {
    const v = this.view;
    if (!v) return;
    const effect = this.effectName();
    const skin = effect === SKINS[v.mood].effect ? SKINS[v.mood] : { ...SKINS[v.mood], effect };
    const f = this.currentFrame();
    const screaming = this.screaming();
    const effectFrame = Math.floor((performance.now() * this.effectFps()) / 1000);
    const closed = !this.gesturing && !screaming && (v.mode === 'sleep' || this.blinking);
    this.renderer.draw(skin, f, v.facing, closed, effectFrame, this.look, {
      gesture: this.gesturing,
      shake: screaming ? shakeOffset(this.tick) : undefined,
    });
    this.syncBalloon();
  }
}
