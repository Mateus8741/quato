// Desenho em pixel art dos efeitos da arminha (canvas 2D, sem suavização).
import { FLASH_LIFE, TRACER_LIFE, markAlpha, shellAlpha, type FxState, type Mark, type Pt, type Shell } from './state.ts';

/** Tamanho do "pixel de arte" dos efeitos, em px CSS. */
const PX = 3;
const MARK_PX = 2;

export function flashDir(from: Pt, to: Pt): number {
  return to.x >= from.x ? 1 : -1;
}

/** Pixels de uma linha (Bresenham) numa grade de `step` px. */
export function linePixels(a: Pt, b: Pt, step: number): Pt[] {
  let x0 = Math.round(a.x / step);
  let y0 = Math.round(a.y / step);
  const x1 = Math.round(b.x / step);
  const y1 = Math.round(b.y / step);
  const dx = Math.abs(x1 - x0);
  const dy = -Math.abs(y1 - y0);
  const sx = x0 < x1 ? 1 : -1;
  const sy = y0 < y1 ? 1 : -1;
  let err = dx + dy;
  const out: Pt[] = [];
  for (let i = 0; i < 4000; i++) {
    out.push({ x: x0 * step, y: y0 * step });
    if (x0 === x1 && y0 === y1) break;
    const e2 = 2 * err;
    if (e2 >= dy) {
      err += dy;
      x0 += sx;
    }
    if (e2 <= dx) {
      err += dx;
      y0 += sy;
    }
  }
  return out;
}

function px(ctx: CanvasRenderingContext2D, x: number, y: number, w: number, h: number, color: string, step = PX): void {
  ctx.fillStyle = color;
  ctx.fillRect(Math.round(x / step) * step, Math.round(y / step) * step, w, h);
}

function drawTracer(ctx: CanvasRenderingContext2D, from: Pt, to: Pt, age: number): void {
  const head = Math.min(1, age / (TRACER_LIFE * 0.4));
  const tail = Math.max(0, (age - TRACER_LIFE * 0.3) / (TRACER_LIFE * 0.7));
  if (tail >= head) return;
  const pts = linePixels(from, to, PX);
  const a = Math.floor(tail * (pts.length - 1));
  const b = Math.ceil(head * (pts.length - 1));
  const fade = 1 - Math.max(0, age - TRACER_LIFE * 0.5) / (TRACER_LIFE * 0.5);
  ctx.globalAlpha = Math.max(0, fade);
  for (let i = a; i <= b; i++) {
    const p = pts[i];
    if (!p) continue;
    // miolo claro, borda amarela
    ctx.fillStyle = '#ffd23c';
    ctx.fillRect(p.x, p.y, PX, PX);
    if (i % 2 === 0) {
      ctx.fillStyle = '#fff7c2';
      ctx.fillRect(p.x + 1, p.y + 1, PX - 2, PX - 2);
    }
  }
  ctx.globalAlpha = 1;
}

function drawFlash(ctx: CanvasRenderingContext2D, from: Pt, to: Pt, age: number): void {
  if (age >= FLASH_LIFE) return;
  const d = flashDir(from, to);
  const big = age < FLASH_LIFE / 2;
  const cx = Math.round(from.x / PX) * PX;
  const cy = Math.round(from.y / PX) * PX;
  const cell = (ix: number, iy: number, c: string) => {
    ctx.fillStyle = c;
    ctx.fillRect(cx + ix * PX, cy + iy * PX, PX, PX);
  };
  cell(0, 0, '#fffbe0');
  cell(d, 0, '#fff0a0');
  cell(0, -1, '#ffd23c');
  cell(0, 1, '#ffd23c');
  cell(-d, 0, '#ff9f1c');
  if (big) {
    cell(2 * d, 0, '#ffd23c');
    cell(3 * d, 0, '#ff9f1c');
    cell(d, -1, '#ff9f1c');
    cell(d, 1, '#ff9f1c');
    cell(0, -2, '#ff9f1c');
    cell(0, 2, '#ff9f1c');
  }
}

function lcg(seed: number): () => number {
  let a = seed >>> 0;
  return () => {
    a = (Math.imul(a, 1664525) + 1013904223) >>> 0;
    return a / 0x100000000;
  };
}

/** Buraquinho escuro com rachaduras (o desenho depende só da semente: não muda entre quadros). */
function drawMark(ctx: CanvasRenderingContext2D, m: Mark, alpha: number): void {
  const r = lcg(m.seed);
  const S = MARK_PX;
  const cx = Math.round(m.x / S) * S;
  const cy = Math.round(m.y / S) * S;
  ctx.globalAlpha = alpha;
  // rachaduras: 5 a 7 raios curtos
  const rays = 5 + Math.floor(r() * 3);
  for (let i = 0; i < rays; i++) {
    const ang = (i / rays) * Math.PI * 2 + r() * 0.6;
    const len = 2 + Math.floor(r() * 4);
    for (let k = 2; k <= len + 1; k++) {
      const x = cx + Math.round((Math.cos(ang) * k) / 1) * S;
      const y = cy + Math.round((Math.sin(ang) * k) / 1) * S;
      ctx.fillStyle = k > len ? 'rgba(60,52,46,0.55)' : 'rgba(34,28,24,0.8)';
      ctx.fillRect(x, y, S, S);
    }
  }
  // buraco: 2x2 pixels com borda mais clara
  ctx.fillStyle = 'rgba(70,62,56,0.8)';
  ctx.fillRect(cx - S, cy - S, S * 3, S * 3);
  ctx.fillStyle = '#0d0a08';
  ctx.fillRect(cx, cy, S * 2 - S, S * 2 - S);
  ctx.fillRect(cx - S, cy, S, S);
  ctx.fillStyle = '#0d0a08';
  ctx.fillRect(cx, cy - S, S, S);
  ctx.globalAlpha = 1;
}

function drawShell(ctx: CanvasRenderingContext2D, s: Shell, alpha: number): void {
  const S = 2;
  const x = Math.round(s.x / S) * S;
  const y = Math.round(s.y / S) * S;
  ctx.globalAlpha = alpha;
  const flat = s.restAt !== null || Math.floor(s.rot / (Math.PI / 2)) % 2 === 0;
  if (flat) {
    ctx.fillStyle = '#d9a521';
    ctx.fillRect(x - S * 2, y - S, S * 4, S * 2);
    ctx.fillStyle = '#fff0a0';
    ctx.fillRect(x - S * 2, y - S, S * 3, S);
    ctx.fillStyle = '#9c7414';
    ctx.fillRect(x + S, y, S, S);
  } else {
    ctx.fillStyle = '#d9a521';
    ctx.fillRect(x - S, y - S * 2, S * 2, S * 4);
    ctx.fillStyle = '#fff0a0';
    ctx.fillRect(x - S, y - S * 2, S, S * 3);
    ctx.fillStyle = '#9c7414';
    ctx.fillRect(x, y + S, S, S);
  }
  ctx.globalAlpha = 1;
}

export function draw(ctx: CanvasRenderingContext2D, st: FxState, now: number, w: number, h: number): void {
  ctx.imageSmoothingEnabled = false;
  ctx.clearRect(0, 0, w, h);
  for (const m of st.marks) {
    const age = now - m.born;
    if (age < 0) continue; // o rastro ainda não chegou
    const a = markAlpha(age);
    if (a > 0) drawMark(ctx, m, a);
  }
  for (const s of st.shells) {
    const a = shellAlpha(s, now);
    if (a > 0) drawShell(ctx, s, a);
  }
  for (const t of st.tracers) {
    const age = now - t.born;
    drawTracer(ctx, t.from, t.to, age);
    drawFlash(ctx, t.from, t.to, age);
  }
}

