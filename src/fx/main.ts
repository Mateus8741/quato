import { emit } from '@tauri-apps/api/event';
import { EV, on } from '../shared/ipc.ts';
import { draw } from './render.ts';
import { FxState, type FxShot } from './state.ts';

const canvas = document.getElementById('fx') as HTMLCanvasElement;
const ctx = canvas.getContext('2d') as CanvasRenderingContext2D;
const state = new FxState();
const now = () => performance.now() / 1000;

let w = 0;
let h = 0;
let timer: number | null = null;
let raf = 0;
let last = now();

function resize(): void {
  const dpr = window.devicePixelRatio || 1;
  w = window.innerWidth;
  h = window.innerHeight;
  canvas.width = Math.max(1, Math.round(w * dpr));
  canvas.height = Math.max(1, Math.round(h * dpr));
  ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
  ctx.imageSmoothingEnabled = false;
  paint();
}

function paint(): void {
  draw(ctx, state, now(), w, h);
}

/** Um quadro: quadros rápidos só enquanto algo se mexe; só marcas paradas = 5 Hz; vazio = nada roda. */
function frame(): void {
  raf = 0;
  const t = now();
  const dt = Math.min(0.05, t - last);
  last = t;
  state.step(dt, t);
  paint();
  schedule();
}

function schedule(): void {
  if (raf || timer !== null) return;
  if (state.empty()) return;
  if (state.animating()) {
    raf = requestAnimationFrame(frame);
  } else {
    timer = window.setTimeout(() => {
      timer = null;
      last = now();
      frame();
    }, 200);
  }
}

function wake(): void {
  if (timer !== null) {
    clearTimeout(timer);
    timer = null;
  }
  if (!raf) {
    last = now();
    raf = requestAnimationFrame(frame);
  }
}

async function start(): Promise<void> {
  window.addEventListener('resize', resize);
  resize();
  await on<FxShot>(EV.fxShot, (s) => {
    state.addShot(s, now());
    wake();
  });
  await on<null>(EV.fxClear, () => {
    state.clear();
    paint();
  });
  await emit(EV.fxReady);
}

start().catch((e) => console.error('[quato] falha ao iniciar os efeitos', e));
