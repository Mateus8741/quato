import { composeRGBA } from '../sprites/compose.ts';
import type { Facing, Look } from '../shared/types.ts';
import { effectPixels } from '../sprites/effects.ts';
import { SIZE, type Frame, type MoodSkin } from '../sprites/types.ts';

/** Tamanho na tela de 1 pixel de arte do pato (32x32 -> 128x128). */
export const DUCK_SCALE = 4;

export interface DrawExtra {
  /** "Pena do meio" levantada. */
  gesture?: boolean;
  /** Tremedeira, em pixels de arte (-1..1); aplicada como transform CSS no canvas. */
  shake?: { x: number; y: number };
}

export class Renderer {
  private canvas: HTMLCanvasElement;
  private ctx: CanvasRenderingContext2D;
  private layer: HTMLCanvasElement;
  private layerCtx: CanvasRenderingContext2D;
  private shakeKey = '0,0';

  constructor(canvas: HTMLCanvasElement) {
    this.canvas = canvas;
    canvas.width = SIZE;
    canvas.height = SIZE;
    this.ctx = canvas.getContext('2d')!;
    this.ctx.imageSmoothingEnabled = false;
    this.layer = document.createElement('canvas');
    this.layer.width = SIZE;
    this.layer.height = SIZE;
    this.layerCtx = this.layer.getContext('2d')!;
  }

  draw(skin: MoodSkin, frame: Frame, facing: Facing, eyesClosed: boolean, effectFrame: number, look: Look, extra: DrawExtra = {}): void {
    const pixels = composeRGBA(skin, frame, eyesClosed, look, { gesture: extra.gesture });
    const img = new ImageData(pixels as unknown as Uint8ClampedArray<ArrayBuffer>, SIZE, SIZE);
    this.layerCtx.clearRect(0, 0, SIZE, SIZE);
    this.layerCtx.putImageData(img, 0, 0);

    const { ctx } = this;
    ctx.setTransform(1, 0, 0, 1, 0, 0);
    ctx.clearRect(0, 0, SIZE, SIZE);
    if (facing === 'left') ctx.setTransform(-1, 0, 0, 1, SIZE, 0);
    ctx.drawImage(this.layer, 0, 0);
    ctx.setTransform(1, 0, 0, 1, 0, 0);

    // efeito do humor (já espelhado quando olha para a esquerda)
    for (const p of effectPixels(skin.effect, frame, facing, effectFrame, look.accessories.includes('faca-sangue'))) {
      ctx.fillStyle = p.color;
      ctx.fillRect(p.x, p.y, 1, 1);
    }

    const s = extra.shake ?? { x: 0, y: 0 };
    const key = `${s.x},${s.y}`;
    if (key !== this.shakeKey) {
      this.shakeKey = key;
      this.canvas.style.transform = s.x || s.y ? `translate(${s.x * DUCK_SCALE}px, ${s.y * DUCK_SCALE}px)` : '';
    }
  }
}
