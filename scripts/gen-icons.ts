// Gera: app-icon.png (1024), src-tauri/icons/tray.png (22) e tray@2x.png (44, silhueta para template do Mac)
// e uma folha de prévia dos humores em preview/. Uso: node scripts/gen-icons.ts
import { mkdirSync, writeFileSync } from 'node:fs';
import { composeRGBA } from '../src/sprites/compose.ts';
import { ANIMS, SKINS, MOODS } from '../src/sprites/index.ts';
import { encodePng, scaleNearest } from './png.ts';

const SIZE = 32;
const skin = SKINS.feliz;
const idle = new Uint8Array(composeRGBA(skin, ANIMS.idle[0]!, false));


mkdirSync('src-tauri/icons', { recursive: true });
mkdirSync('preview', { recursive: true });

// Ícone do app: pato ×24 (768 px) centralizado numa tela 1024 com fundo arredondado suave.
const big = scaleNearest(idle, SIZE, SIZE, 24);
const canvas = new Uint8Array(1024 * 1024 * 4);
const off = (1024 - 768) / 2;
for (let y = 0; y < 1024; y++) {
  for (let x = 0; x < 1024; x++) {
    const cx = x - 512, cy = y - 512;
    const r = 512 - 40;
    const inside = Math.max(Math.abs(cx), Math.abs(cy)) < r && (Math.abs(cx) < r - 140 || Math.abs(cy) < r - 140 || (Math.abs(cx) - (r - 140)) ** 2 + (Math.abs(cy) - (r - 140)) ** 2 < 140 ** 2);
    const k = (y * 1024 + x) * 4;
    if (inside) { canvas[k] = 0xcf; canvas[k + 1] = 0xec; canvas[k + 2] = 0xff; canvas[k + 3] = 255; }
  }
}
for (let y = 0; y < 768; y++) {
  for (let x = 0; x < 768; x++) {
    const s = (y * 768 + x) * 4;
    if (big[s + 3]! === 0) continue;
    const d = ((y + off) * 1024 + (x + off)) * 4;
    canvas[d] = big[s]!; canvas[d + 1] = big[s + 1]!; canvas[d + 2] = big[s + 2]!; canvas[d + 3] = 255;
  }
}
writeFileSync('app-icon.png', encodePng(1024, 1024, canvas));

// Tray: silhueta preta com alpha (macOS "template").
function silhouette(px: Uint8Array): Uint8Array {
  const out = new Uint8Array(px.length);
  for (let i = 0; i < px.length; i += 4) if (px[i + 3]! > 0) out[i + 3] = 255;
  return out;
}
// recorta as linhas vazias para o pato ocupar o ícone todo
const sil = silhouette(idle);
let minY = SIZE, maxY = 0, minX = SIZE, maxX = 0;
for (let y = 0; y < SIZE; y++) for (let x = 0; x < SIZE; x++) if (sil[(y * SIZE + x) * 4 + 3]) { minY = Math.min(minY, y); maxY = Math.max(maxY, y); minX = Math.min(minX, x); maxX = Math.max(maxX, x); }
const side = Math.max(maxX - minX + 1, maxY - minY + 1);
const sq = new Uint8Array(side * side * 4);
for (let y = minY; y <= maxY; y++) for (let x = minX; x <= maxX; x++) {
  const s = (y * SIZE + x) * 4, d = ((y - minY) * side + (x - minX)) * 4;
  sq[d + 3] = sil[s + 3]!;
}
function fit(target: number): Uint8Array {
  const out = new Uint8Array(target * target * 4);
  for (let y = 0; y < target; y++) for (let x = 0; x < target; x++) {
    const sx = Math.min(side - 1, Math.floor((x * side) / target)), sy = Math.min(side - 1, Math.floor((y * side) / target));
    out[(y * target + x) * 4 + 3] = sq[(sy * side + sx) * 4 + 3]!;
  }
  return out;
}
writeFileSync('src-tauri/icons/tray.png', encodePng(22, 22, fit(22)));
writeFileSync('src-tauri/icons/tray@2x.png', encodePng(44, 44, fit(44)));

// ---------------------------------------------------------------------------------------------
// Prévias em preview/ (para conferir o desenho lendo os PNGs):
//  sprites.png   linha 1 cores; 2 acessórios; 3 humores (lagarto = lagartinho); 4 poses + dança;
//                5 lagartinho (idle/walk/run/grab/sleep/dance/grito); 6 pena do meio, gota pingando, notas
//  zoom-*.png    ampliados x12 para ver de perto
//  window-*.png  a janela inteira 192x192 (x2) com a plaquinha / o balão do grito
import { ACCESSORIES, COLORS, LIZARD_ANIMS, LIZARD_SCREAM_ANIMS } from '../src/sprites/index.ts';
import { effectPixels } from '../src/sprites/effects.ts';
import { TOP_H, TOP_W, TOP_SCALE, mouthToTopX, renderBalloon, renderSign } from '../src/sprites/overlay.ts';
import { SCREAM_TEXT } from '../src/sprites/scream.ts';
import type { Frame, Look, MoodSkin } from '../src/sprites/types.ts';

const BG = [0xe8, 0xea, 0xee];

function sprite(skin: MoodSkin, f: Frame, opts: { look?: Look; closed?: boolean; gesture?: boolean; fx?: number; facing?: 'left' | 'right' } = {}): Uint8Array {
  const lk = opts.look ?? { color: 'amarelo', accessories: [] };
  const px = composeRGBA(skin, f, opts.closed ?? false, lk, { gesture: opts.gesture });
  const facing = opts.facing ?? 'right';
  let out = new Uint8Array(px);
  if (facing === 'left') {
    const m = new Uint8Array(out.length);
    for (let y = 0; y < SIZE; y++) for (let x = 0; x < SIZE; x++) {
      const s = (y * SIZE + x) * 4, d = (y * SIZE + (31 - x)) * 4;
      for (let c = 0; c < 4; c++) m[d + c] = out[s + c]!;
    }
    out = m;
  }
  if (opts.fx !== undefined) {
    for (const p of effectPixels(skin.effect, f, facing, opts.fx, lk.accessories.includes('faca-sangue'))) {
      if (p.x < 0 || p.y < 0 || p.x >= SIZE || p.y >= SIZE) continue;
      const n = parseInt(p.color.slice(1), 16), k = (p.y * SIZE + p.x) * 4;
      out[k] = (n >> 16) & 255; out[k + 1] = (n >> 8) & 255; out[k + 2] = n & 255; out[k + 3] = 255;
    }
  }
  return out;
}

class Sheet {
  data: Uint8Array;
  w: number;
  h: number;
  constructor(w: number, h: number) {
    this.w = w;
    this.h = h;
    this.data = new Uint8Array(w * h * 4);
    for (let i = 0; i < this.data.length; i += 4) { this.data[i] = BG[0]!; this.data[i + 1] = BG[1]!; this.data[i + 2] = BG[2]!; this.data[i + 3] = 255; }
  }
  blit(px: Uint8Array, sw: number, sh: number, k: number, ox: number, oy: number): void {
    const up = scaleNearest(px, sw, sh, k);
    for (let y = 0; y < sh * k; y++) for (let x = 0; x < sw * k; x++) {
      const s = (y * sw * k + x) * 4;
      if (up[s + 3] === 0) continue;
      const tx = ox + x, ty = oy + y;
      if (tx < 0 || ty < 0 || tx >= this.w || ty >= this.h) continue;
      const d = (ty * this.w + tx) * 4;
      this.data[d] = up[s]!; this.data[d + 1] = up[s + 1]!; this.data[d + 2] = up[s + 2]!; this.data[d + 3] = 255;
    }
  }
  save(path: string): void { writeFileSync(path, encodePng(this.w, this.h, this.data)); }
}

const K = 5, cols = 16, rows = 8;
const sheet = new Sheet(cols * SIZE * K, rows * SIZE * K);
const put = (r: number, c: number, px: Uint8Array) => sheet.blit(px, SIZE, SIZE, K, c * SIZE * K, r * SIZE * K);
COLORS.forEach((c, i) => put(0, i, sprite(SKINS.feliz, ANIMS.idle[0]!, { look: { color: c.id, accessories: [] } })));
ACCESSORIES.forEach((a, i) => put(1, i, sprite(SKINS.feliz, ANIMS.idle[0]!, { look: { color: 'amarelo', accessories: [a.id] } })));
const combo = { color: 'rosa', accessories: ['peruca-ruiva', 'oculos', 'fones', 'laco', 'flor'] };
MOODS.forEach((m, i) => put(2, i, sprite(SKINS[m], (m === 'lagarto' ? LIZARD_ANIMS : ANIMS).idle[0]!, { look: combo, fx: 1 })));
MOODS.forEach((m, i) => put(2, 7 + i, sprite(SKINS[m], (m === 'lagarto' ? LIZARD_ANIMS : ANIMS).idle[0]!, { fx: 2 })));
const hat = { color: 'preto', accessories: ['chapeu-festa', 'laco'] };
put(3, 0, sprite(SKINS.sono, ANIMS.sleep[0]!, { look: combo, closed: true }));
put(3, 1, sprite(SKINS.feliz, ANIMS.walk[1]!, { look: hat }));
put(3, 2, sprite(SKINS.raiva, ANIMS.run[0]!, { look: { color: 'marrom', accessories: ['coroa', 'fones'] } }));
put(3, 3, sprite(SKINS.feliz, ANIMS.grab[0]!, { look: { color: 'menta', accessories: ['cartola', 'oculos'] } }));
put(3, 4, sprite(SKINS.triste, ANIMS.idle[0]!, { look: { color: 'lavanda', accessories: ['peruca-curta', 'flor'] } }));
put(3, 5, sprite(SKINS.sozinho, ANIMS.idle[0]!, { look: { color: 'laranja', accessories: ['peruca-curta', 'oculos-escuros', 'fones'] } }));
ANIMS.dance.forEach((f, i) => put(3, 6 + i, sprite(SKINS.danca, f, { fx: i })));
ANIMS.dance.forEach((f, i) => put(3, 10 + i, sprite(SKINS.danca, f, { fx: i + 2, look: { color: 'azul', accessories: ['fones'] } })));
// lagartinho
const L = SKINS.lagarto;
// o lagartinho só pinga com a faquinha suja de sangue
const KN = { color: 'amarelo', accessories: ['faca-sangue'] };
const lz: Frame[] = [...LIZARD_ANIMS.idle, ...LIZARD_ANIMS.walk, ...LIZARD_ANIMS.run, LIZARD_ANIMS.grab[0]!, ...LIZARD_ANIMS.dance];
lz.forEach((f, i) => put(4, i, sprite(L, f, { fx: i % 8, look: KN })));
put(4, 12, sprite(L, LIZARD_ANIMS.sleep[0]!, { closed: true, fx: 3, look: KN }));
put(4, 13, sprite(L, LIZARD_SCREAM_ANIMS.idle[0]!, { fx: 2, look: KN }));
put(4, 14, sprite(L, LIZARD_SCREAM_ANIMS.walk[1]!, { fx: 4, look: { color: 'rosa', accessories: ['chapeu-festa', 'oculos-escuros'] } }));
// pena do meio, gota pingando (8 quadros), lagartinho com acessórios
put(5, 0, sprite(SKINS.feliz, ANIMS.idle[0]!, { gesture: true }));
put(5, 1, sprite(SKINS.raiva, ANIMS.walk[1]!, { gesture: true, look: { color: 'preto', accessories: ['oculos-escuros'] } }));
put(5, 2, sprite(SKINS.danca, ANIMS.dance[1]!, { gesture: true, look: { color: 'rosa', accessories: ['coroa'] } }));
put(5, 3, sprite(L, LIZARD_ANIMS.idle[0]!, { gesture: true, look: KN }));
put(5, 4, sprite(L, LIZARD_SCREAM_ANIMS.run[0]!, { gesture: true, facing: 'left', look: KN }));
for (let i = 0; i < 8; i++) put(5, 5 + i, sprite(L, LIZARD_ANIMS.idle[0]!, { fx: i, look: KN }));
put(5, 13, sprite(L, LIZARD_ANIMS.idle[0]!, { look: { color: 'amarelo', accessories: ['peruca-ruiva', 'oculos', 'fones', 'laco', 'flor', 'faca'] } }));
put(5, 14, sprite(L, LIZARD_ANIMS.walk[0]!, { look: { color: 'amarelo', accessories: ['chapeu-seletor', 'varinha'] }, facing: 'left' }));
// linha 7: Duvidoso (pato e lagartinho, 3 quadros do '?'), dúvida aleatória em outros humores, no lagarto e dormindo
const DV = SKINS.duvida;
for (let i = 0; i < 3; i++) put(6, i, sprite(DV, ANIMS.idle[0]!, { fx: i }));
for (let i = 0; i < 3; i++) put(6, 3 + i, sprite({ ...L, effect: 'question' }, LIZARD_ANIMS.idle[0]!, { fx: i }));
put(6, 6, sprite({ ...SKINS.feliz, effect: 'question' }, ANIMS.idle[0]!, { fx: 1 }));
put(6, 7, sprite({ ...SKINS.raiva, effect: 'question' }, ANIMS.walk[1]!, { fx: 0, look: { color: 'rosa', accessories: ['chapeu-festa'] } }));
put(6, 8, sprite({ ...L, effect: 'question' }, LIZARD_ANIMS.idle[0]!, { fx: 1, facing: 'left' }));
put(6, 9, sprite(DV, ANIMS.idle[0]!, { fx: 2, facing: 'left', look: { color: 'azul', accessories: ['coroa'] } }));
// linha 8: os acessórios novos (capivara, faca, faca com sangue, arminha, varinha) no pato feliz e no lagartinho
const NEW_ACC = ['chapeu-capivara', 'faca', 'faca-sangue', 'arminha', 'varinha'];
NEW_ACC.forEach((id, i) => {
  put(7, i, sprite(SKINS.feliz, ANIMS.idle[0]!, { look: { color: 'amarelo', accessories: [id] } }));
  put(7, 5 + i, sprite(L, LIZARD_ANIMS.idle[0]!, { fx: 2, look: { color: 'amarelo', accessories: [id] } }));
});
put(7, 10, sprite(SKINS.raiva, ANIMS.walk[1]!, { look: { color: 'amarelo', accessories: ['chapeu-capivara', 'arminha'] } }));
put(7, 11, sprite(SKINS.feliz, ANIMS.walk[0]!, { facing: 'left', look: { color: 'rosa', accessories: ['chapeu-capivara', 'faca-sangue', 'oculos'] } }));
put(7, 12, sprite(L, LIZARD_ANIMS.run[0]!, { fx: 1, look: { color: 'amarelo', accessories: ['chapeu-capivara', 'faca-sangue'] } }));
put(7, 13, sprite(L, LIZARD_ANIMS.dance[0]!, { look: { color: 'amarelo', accessories: ['arminha'] } }));
put(7, 14, sprite(SKINS.sono, ANIMS.sleep[0]!, { closed: true, look: { color: 'amarelo', accessories: ['chapeu-capivara', 'arminha'] } }));
put(7, 15, sprite(SKINS.danca, ANIMS.dance[2]!, { look: { color: 'azul', accessories: ['chapeu-capivara', 'faca'] } }));
sheet.save('preview/sprites.png');

// Zooms (x12)
function zoom(path: string, items: Uint8Array[]): void {
  const Z = 12, z = new Sheet(items.length * SIZE * Z + (items.length - 1) * 8, SIZE * Z);
  items.forEach((px, i) => z.blit(px, SIZE, SIZE, Z, i * (SIZE * Z + 8), 0));
  z.save(path);
}
zoom('preview/zoom-lizard.png', [sprite(L, LIZARD_ANIMS.idle[0]!, { fx: 3, look: KN }), sprite(L, LIZARD_SCREAM_ANIMS.idle[0]!, { fx: 5, look: KN }), sprite(L, LIZARD_ANIMS.idle[0]!, { gesture: true, fx: 1, look: KN })]);
zoom('preview/zoom-acessorios.png', NEW_ACC.map((id) => sprite(SKINS.feliz, ANIMS.idle[0]!, { look: { color: 'amarelo', accessories: [id] } })));
zoom('preview/zoom-acessorios-lagarto.png', NEW_ACC.map((id) => sprite(L, LIZARD_ANIMS.idle[0]!, { fx: 3, look: { color: 'amarelo', accessories: [id] } })));
zoom('preview/zoom-duvida.png', [sprite(DV, ANIMS.idle[0]!, { fx: 0 }), sprite(DV, ANIMS.idle[0]!, { fx: 1 }), sprite(DV, ANIMS.idle[0]!, { fx: 2 }), sprite({ ...SKINS.feliz, effect: 'question' }, ANIMS.idle[0]!, { fx: 1 })]);
zoom('preview/zoom-dance.png', ANIMS.dance.map((f, i) => sprite(SKINS.danca, f, { fx: i })));
zoom('preview/zoom-feather.png', [sprite(SKINS.feliz, ANIMS.idle[0]!, { gesture: true }), sprite(SKINS.feliz, ANIMS.idle[0]!, { gesture: true, look: { color: 'azul', accessories: ['chapeu-festa'] } })]);

// Janela inteira 192x192: faixa de cima 96x32 (x2) + quadro 32x32 (x4) em left 32, top 64. Salva x2.
function windowShot(path: string, top: Uint8Array, body: Uint8Array): void {
  const S = 2, win = new Sheet(192 * S, 192 * S);
  for (let i = 0; i < win.data.length; i += 4) { win.data[i] = 0x9f; win.data[i + 1] = 0xc9; win.data[i + 2] = 0xe8; }
  win.blit(top, TOP_W, TOP_H, TOP_SCALE * S, 0, 0);
  win.blit(body, SIZE, SIZE, 4 * S, 32 * S, 64 * S);
  win.save(path);
}
windowShot('preview/window-sign.png', new Uint8Array(renderSign('Hora de beber água!').data), sprite(SKINS.feliz, ANIMS.idle[0]!, { fx: 0 }));
const scream = LIZARD_SCREAM_ANIMS.idle[0]!;
windowShot('preview/window-scream.png', new Uint8Array(renderBalloon(SCREAM_TEXT, mouthToTopX(27, 'right'), 1).data), sprite(L, scream, { fx: 3, look: KN }));
windowShot('preview/window-scream-left.png', new Uint8Array(renderBalloon(SCREAM_TEXT, mouthToTopX(27, 'left'), 2).data), sprite(L, scream, { fx: 5, facing: 'left', look: KN }));
windowShot('preview/window-sign-long.png', new Uint8Array(renderSign('Ação: ÇÃO É ÔÕ Í Ú À Â, 0123456789 ?!').data), sprite(SKINS.danca, ANIMS.dance[0]!, { fx: 1 }));
windowShot('preview/window-sign-update.png', new Uint8Array(renderSign('Nova versão disponível!', 'update').data), sprite(SKINS.feliz, ANIMS.idle[0]!, { fx: 0 }));
console.log('ok');
