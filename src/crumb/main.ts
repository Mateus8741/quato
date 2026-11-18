// Janelinha da migalha: desenha a migalha parada e, quando o pato chega (`crumb:eat`), a animação
// de comer. Sem timers enquanto está parada; o Rust fecha a janela depois.
import { EV, on } from '../shared/ipc.ts';
import { burst, crumbScale, CRUMB_GRID, CRUMB_H, CRUMB_PALETTE, CRUMB_W, EAT_MS, particleAt, type Particle } from './art.ts';

const canvas = document.getElementById('crumb') as HTMLCanvasElement;
const ctx = canvas.getContext('2d') as CanvasRenderingContext2D;

let eatStart: number | null = null;
let particles: Particle[] = [];
let raf = 0;

/** Tamanho de 1 pixel de arte em pixels CSS: inteiro, para a arte não ficar quebrada. */
function unit(): number {
  const side = Math.min(window.innerWidth || 48, window.innerHeight || 48);
  return Math.max(1, Math.floor(side / 12));
}

function paint(): void {
  const dpr = window.devicePixelRatio || 1;
  const w = window.innerWidth || 48;
  const h = window.innerHeight || 48;
  if (canvas.width !== Math.round(w * dpr) || canvas.height !== Math.round(h * dpr)) {
    canvas.width = Math.max(1, Math.round(w * dpr));
    canvas.height = Math.max(1, Math.round(h * dpr));
  }
  ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
  ctx.imageSmoothingEnabled = false;
  ctx.clearRect(0, 0, w, h);
  const k = unit();
  const ox = Math.round((w - CRUMB_W * k) / 2);
  const oy = Math.round((h - CRUMB_H * k) / 2);
  const t = eatStart === null ? 0 : performance.now() - eatStart;
  const scale = crumbScale(t);
  if (scale > 0) {
    const cw = Math.max(1, Math.round(CRUMB_W * scale));
    const ch = Math.max(1, Math.round(CRUMB_H * scale));
    // desenha a migalha inteira e a encolhe pelo centro, para as mordidas "comerem" as bordas
    ctx.save();
    ctx.translate(ox + (CRUMB_W * k) / 2, oy + (CRUMB_H * k) / 2);
    ctx.scale(cw / CRUMB_W, ch / CRUMB_H);
    ctx.translate(-(CRUMB_W * k) / 2, -(CRUMB_H * k) / 2);
    CRUMB_GRID.forEach((row, j) => [...row].forEach((c, i) => {
      const color = CRUMB_PALETTE[c];
      if (!color) return;
      ctx.fillStyle = color;
      ctx.fillRect(i * k, j * k, k, k);
    }));
    ctx.restore();
  }
  for (const p of particles) {
    const q = particleAt(p, t);
    if (q.alpha <= 0) continue;
    ctx.globalAlpha = q.alpha;
    ctx.fillStyle = p.color;
    ctx.fillRect(ox + Math.round(q.x) * k, oy + Math.round(q.y) * k, k, k);
  }
  ctx.globalAlpha = 1;
}

function frame(): void {
  raf = 0;
  paint();
  if (eatStart !== null && performance.now() - eatStart < EAT_MS) raf = requestAnimationFrame(frame);
}

async function start(): Promise<void> {
  window.addEventListener('resize', paint);
  paint();
  await on<null>(EV.crumbShow, () => {
    eatStart = null;
    particles = [];
    paint();
  });
  await on<null>(EV.crumbEat, () => {
    eatStart = performance.now();
    particles = burst(Math.random);
    if (!raf) raf = requestAnimationFrame(frame);
  });
}

start().catch((e) => console.error('[quato] falha ao iniciar a migalha', e));
