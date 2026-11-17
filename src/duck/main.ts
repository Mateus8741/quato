import { api, EV, on } from '../shared/ipc.ts';
import type { DuckGesture, DuckSign, DuckView, Look } from '../shared/types.ts';
import { Animator } from './animator.ts';
import { Renderer } from './renderer.ts';
import { TopStrip } from './top.ts';

// Ctrl + clique no Mac vira clique direito: não mostra o menu do navegador.
window.addEventListener('contextmenu', (e) => e.preventDefault());

/**
 * Cada pixel de arte ocupa um número INTEIRO de pixels físicos. Com escala do Windows fracionada
 * (110%, 125%...) o tamanho CSS fixo (128px) vira pixel quebrado e a arte fica irregular.
 * Arredonda para baixo, mantém o pato centrado em x (96) e com a base em y=192 (pés em 184).
 */
function layout(duck: HTMLCanvasElement, top: HTMLCanvasElement): void {
  const dpr = window.devicePixelRatio || 1;
  // A janela pode sair com tamanho diferente de 192 CSS (escala/DPI diferente entre monitores):
  // mede o que realmente existe e distribui tudo proporcionalmente, para o pato nunca ser cortado.
  const w = window.innerWidth > 0 ? window.innerWidth : 192;
  const h = window.innerHeight > 0 ? window.innerHeight : 192;
  const ux = w / 192; // pixels CSS por unidade de design
  const uy = h / 192;
  const unit = Math.min(ux, uy);
  const snap = (css: number) => Math.round(css * dpr) / dpr;
  const place = (el: HTMLCanvasElement, perPx: number, cx: number, bottom: number) => {
    const k = Math.max(1, Math.floor(perPx * unit * dpr));
    const wCss = (k * el.width) / dpr;
    const hCss = (k * el.height) / dpr;
    el.style.width = `${wCss}px`;
    el.style.height = `${hCss}px`;
    el.style.left = `${snap(cx * ux - wCss / 2)}px`;
    el.style.top = `${snap(bottom * uy - hCss)}px`;
  };
  place(duck, 4, 96, 192);
  place(top, 2, 96, 64);
}

// Janela 192x192: faixa de cima (plaquinha/balão) em 0..64 e o pato 128x128 em left 32, top 64.
async function start(): Promise<void> {
  const duck = document.getElementById('duck') as HTMLCanvasElement;
  const topCanvas = document.getElementById('top') as HTMLCanvasElement;
  const top = new TopStrip(topCanvas);
  const relayout = () => layout(duck, topCanvas);
  relayout();
  window.addEventListener('resize', relayout);
  const animator = new Animator(new Renderer(duck), top);
  animator.setLook(await api.getLook());
  animator.update(await api.getDuckView());
  await on<DuckView>(EV.duckView, (v) => animator.update(v));
  await on<Look>(EV.duckLook, (l) => animator.setLook(l));
  await on<DuckSign>(EV.duckSign, (s) => top.showSign(s.text, s.durationMs, s.kind));
  await on<DuckGesture>(EV.duckGesture, (g) => {
    if (g.kind === 'feather') animator.gesture(g.durationMs);
  });
  await api.duckReady();
}

start().catch((e) => console.error('[quato] falha ao iniciar o pato', e));
