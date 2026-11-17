// Faixa de cima da janela (0..64 px): plaquinha do lembrete de água e balão do grito do lagartinho.
// Só tem timer enquanto algo está na tela; sem plaquinha nem balão, não roda nada.
import { TOP_H, TOP_W, renderBalloon, renderSign, type Pix, type SignKind } from '../sprites/overlay.ts';
import { SCREAM_TEXT } from '../sprites/scream.ts';

/** Escalas da entrada (pop com quique) e da saída (encolhe e some). */
const SIGN_IN = [0.2, 0.55, 0.9, 1.15, 1.05, 0.95, 1];
const SIGN_OUT = [1.05, 0.8, 0.55, 0.3, 0.1];
/** Balanço leve (x em pixels da faixa) enquanto a plaquinha fica parada. */
const SWAY = [0, 1, 1, 0, -1, -1];
const IN_OUT_MS = 55;
const SWAY_MS = 300;
const BALLOON_MS = 100;

interface SignState {
  canvas: HTMLCanvasElement;
  phase: 'in' | 'show' | 'out';
  frame: number;
}

function toCanvas(p: Pix): HTMLCanvasElement {
  const c = document.createElement('canvas');
  c.width = p.w;
  c.height = p.h;
  c.getContext('2d')!.putImageData(new ImageData(p.data as unknown as Uint8ClampedArray<ArrayBuffer>, p.w, p.h), 0, 0);
  return c;
}

export class TopStrip {
  private ctx: CanvasRenderingContext2D;
  private sign: SignState | null = null;
  private balloon: { tailX: number; tick: number } | null = null;
  private timer: number | undefined;
  private hideTimer: number | undefined;

  constructor(canvas: HTMLCanvasElement) {
    canvas.width = TOP_W;
    canvas.height = TOP_H;
    this.ctx = canvas.getContext('2d')!;
    this.ctx.imageSmoothingEnabled = false;
  }

  /** Evento duck:sign. `durationMs <= 0` esconde na hora. */
  showSign(text: string, durationMs: number, kind: SignKind = 'water'): void {
    window.clearTimeout(this.hideTimer);
    if (durationMs <= 0) {
      this.sign = null;
      this.kick();
      return;
    }
    this.sign = { canvas: toCanvas(renderSign(text, kind)), phase: 'in', frame: 0 };
    this.hideTimer = window.setTimeout(() => {
      if (this.sign) this.sign = { ...this.sign, phase: 'out', frame: 0 };
      this.kick();
    }, durationMs);
    this.kick();
  }

  /** Liga/desliga o balão do grito; `tailX` aponta o rabinho para a boca (pixels da faixa). */
  setBalloon(on: boolean, tailX = TOP_W / 2): void {
    if (on) {
      const was = this.balloon;
      this.balloon = { tailX, tick: was?.tick ?? 0 };
      if (!was) this.kick();
    } else if (this.balloon) {
      this.balloon = null;
      this.kick();
    }
  }

  private kick(): void {
    window.clearTimeout(this.timer);
    this.timer = undefined;
    this.step();
  }

  private step(): void {
    this.draw();
    const delay = this.advance();
    if (delay !== null) this.timer = window.setTimeout(() => this.step(), delay);
  }

  /** Avança o estado depois de desenhar; devolve quando desenhar de novo (ou null para parar). */
  private advance(): number | null {
    if (this.balloon) this.balloon.tick++;
    const s = this.sign;
    if (s) {
      s.frame++;
      if (s.phase === 'in' && s.frame >= SIGN_IN.length) { s.phase = 'show'; s.frame = 0; }
      if (s.phase === 'out' && s.frame >= SIGN_OUT.length) this.sign = null;
    }
    if (this.balloon) return BALLOON_MS;
    if (!this.sign) {
      // um último quadro limpo depois que a plaquinha some
      if (s) return IN_OUT_MS;
      return null;
    }
    return this.sign.phase === 'show' ? SWAY_MS : IN_OUT_MS;
  }

  private draw(): void {
    const { ctx } = this;
    ctx.setTransform(1, 0, 0, 1, 0, 0);
    ctx.clearRect(0, 0, TOP_W, TOP_H);
    if (this.balloon) {
      // o balão tem prioridade: a plaquinha volta quando o grito acaba
      const p = renderBalloon(SCREAM_TEXT, this.balloon.tailX, this.balloon.tick);
      ctx.putImageData(new ImageData(p.data as unknown as Uint8ClampedArray<ArrayBuffer>, p.w, p.h), 0, 0);
      return;
    }
    const s = this.sign;
    if (!s) return;
    const scale = s.phase === 'in' ? SIGN_IN[s.frame]! : s.phase === 'out' ? SIGN_OUT[s.frame]! : 1;
    const sway = s.phase === 'show' ? SWAY[s.frame % SWAY.length]! : 0;
    const ax = TOP_W / 2, ay = TOP_H;
    ctx.setTransform(scale, 0, 0, scale, ax - ax * scale + sway, ay - ay * scale);
    ctx.drawImage(s.canvas, 0, 0);
    ctx.setTransform(1, 0, 0, 1, 0, 0);
  }
}
