// Gera os dados do pato para a landing page (site/) a partir dos sprites reais do app,
// confere se o port em JS (site/assets/duck-core.js e overlay-core.js) desenha exatamente os
// mesmos pixels (pato, lagartinho, grito, pena do meio, efeitos, plaquinha, balão e fonte)
// e gera og-image.png e os favicons. FALHA (exit 1) se qualquer pixel divergir.
// Uso: node scripts/gen-site-data.ts
import { mkdirSync, readFileSync, writeFileSync } from 'node:fs';
import { ANIMS, SKINS, MOODS, COLORS, ACCESSORIES, ANIM_NAMES, LIZARD_ANIMS, LIZARD_SCREAM_ANIMS, LIZARD_PALETTE } from '../src/sprites/index.ts';
import { BASE_PALETTE } from '../src/sprites/colors.ts';
import { EFFECTS, EFFECT_COLORS, effectPixels } from '../src/sprites/effects.ts';
import { composeRGBA, framesFor, characterFor } from '../src/sprites/compose.ts';
import { FEATHER_DUCK, FEATHER_LIZARD, GESTURE_EYES } from '../src/sprites/gesture.ts';
import { GLYPHS, CELL_H, ACCENT_ROWS, LINE_H, LETTER_GAP, glyph, hasGlyph, normalizeChar, textWidth, wrapText, wrapBalanced } from '../src/sprites/font.ts';
import { TOP_W, TOP_H, TOP_SCALE, renderSign, renderBalloon, signLayout, mouthToTopX } from '../src/sprites/overlay.ts';
import { SCREAM_MIN_GAP_MS, SCREAM_MAX_GAP_MS, SCREAM_DURATION_MS, SCREAM_TEXT, hash01, screamGap, screamState, shakeOffset } from '../src/sprites/scream.ts';
import { DOUBT_MIN_GAP_MS, DOUBT_MAX_GAP_MS, DOUBT_DURATION_MS, doubtGap, doubtState } from '../src/sprites/doubt.ts';
import { SIZE, type EffectName } from '../src/sprites/types.ts';
import { encodePng, scaleNearest } from './png.ts';

const root = new URL('../', import.meta.url);
const site = new URL('site/', root);
const assets = new URL('assets/', site);
mkdirSync(assets, { recursive: true });

// Espaço de cada acessório (cabeça, rosto...) vem do Rust, que é quem decide "um por lugar".
const rust = readFileSync(new URL('src-tauri/src/look.rs', root), 'utf8');
const block = /pub const ACCESSORIES[^=]*= \[([\s\S]*?)\];/.exec(rust)![1]!;
const slotOf = Object.fromEntries([...block.matchAll(/\("([^"]+)",\s*"([^"]+)"\)/g)].map((m) => [m[1]!, m[2]!]));

// Fonte: todas as letras que a fonte sabe desenhar (já com os acentos montados pelo glyph()).
const fontChars: string[] = [];
for (let cp = 0x20; cp <= 0x24f; cp++) {
  const ch = String.fromCodePoint(cp);
  if (hasGlyph(ch) && normalizeChar(ch) === ch) fontChars.push(ch);
}
for (const ch of Object.keys(GLYPHS)) if (!fontChars.includes(ch)) fontChars.push(ch);
const glyphs = Object.fromEntries(fontChars.map((ch) => [ch, glyph(ch)!]));

const data = {
  size: SIZE,
  moods: MOODS,
  animNames: ANIM_NAMES,
  basePalette: BASE_PALETTE,
  anims: ANIMS,
  lizardAnims: LIZARD_ANIMS,
  lizardScreamAnims: LIZARD_SCREAM_ANIMS,
  lizardPalette: LIZARD_PALETTE,
  skins: SKINS,
  colors: COLORS,
  accessories: ACCESSORIES.map((a) => ({ ...a, slot: slotOf[a.id] ?? 'outro' })),
  effects: EFFECTS,
  effectColors: EFFECT_COLORS,
  gesture: { duck: FEATHER_DUCK, lizard: FEATHER_LIZARD, eyes: GESTURE_EYES },
  scream: { minGap: SCREAM_MIN_GAP_MS, maxGap: SCREAM_MAX_GAP_MS, duration: SCREAM_DURATION_MS, text: SCREAM_TEXT },
  doubt: { minGap: DOUBT_MIN_GAP_MS, maxGap: DOUBT_MAX_GAP_MS, duration: DOUBT_DURATION_MS },
  font: { cellH: CELL_H, accentRows: ACCENT_ROWS, lineH: LINE_H, letterGap: LETTER_GAP, glyphs },
  top: { w: TOP_W, h: TOP_H, scale: TOP_SCALE },
};

const out =
  '// GERADO por scripts/gen-site-data.ts a partir de src/sprites. Não edite à mão.\n' +
  `export const DUCK = ${JSON.stringify(data)};\n`;
writeFileSync(new URL('duck-data.js', assets), out);

// ---- confere o port em JS contra o código original, pixel a pixel ----
const core = await import(new URL('duck-core.js', assets).href);
const ov = await import(new URL('overlay-core.js', assets).href);
const counts: Record<string, number> = {};
const fails: string[] = [];
const same = (what: string, a: ArrayLike<number> | string, b: ArrayLike<number> | string, ctx: () => string) => {
  counts[what] = (counts[what] ?? 0) + 1;
  const eq = typeof a === 'string' ? a === b : Buffer.compare(Buffer.from(a as Uint8ClampedArray), Buffer.from(b as Uint8ClampedArray)) === 0;
  if (!eq && fails.length < 20) fails.push(`${what}: ${ctx()}`);
};
const json = (v: unknown) => JSON.stringify(v);

// 1) composição: humor × cor/acessórios × animação × quadro × olhos × pena do meio × grito
const looks = [
  ...COLORS.map((c) => ({ color: c.id, accessories: [] as string[] })),
  ...ACCESSORIES.map((a) => ({ color: 'amarelo', accessories: [a.id] })),
  { color: 'rosa', accessories: ['peruca-ruiva', 'oculos', 'fones', 'laco', 'flor', 'varinha'] },
  { color: 'amarelo', accessories: ['chapeu-capivara', 'faca-sangue'] },
  { color: 'azul', accessories: ['coroa', 'arminha'] },
  { color: 'preto', accessories: ['cartola', 'oculos-escuros', 'laco'] },
  { color: 'inexistente', accessories: ['nao-existe'] },
];
for (const m of MOODS) {
  const screams = characterFor(m) === 'lizard' ? [false, true] : [false];
  for (const look of looks) {
    for (const name of ANIM_NAMES) {
      for (const screaming of screams) {
        framesFor(m, name, screaming).forEach((f, i) => {
          for (const closed of [false, true]) {
            for (const gesture of [false, true]) {
              const a = composeRGBA(SKINS[m], f, closed, look, { gesture });
              const b = core.composeRGBA(m, name, i, closed, look, { gesture, screaming });
              same('composições', a, b, () => `${m}/${name}/${i}/fechado=${closed}/pena=${gesture}/grito=${screaming}/${json(look)}`);
            }
          }
        });
      }
    }
  }
}

// 2) efeitos (inclusive notas e a gota na âncora da faquinha), virados para os dois lados
for (const m of MOODS) {
  const screams = characterFor(m) === 'lizard' ? [false, true] : [false];
  for (const name of ANIM_NAMES) for (const screaming of screams) {
    framesFor(m, name, screaming).forEach((f, i) => {
      for (const effect of Object.keys(EFFECTS) as EffectName[]) for (const facing of ['left', 'right'] as const) {
        for (let ef = 0; ef < 9; ef++) for (const bloody of [false, true]) {
          same('efeitos', json(effectPixels(effect, f, facing, ef, bloody)), json(core.effectPixelsFor(effect, f, facing, ef, bloody)), () => `${effect}/${m}/${name}/${i}/${facing}/${ef}/sangue=${bloody}`);
        }
      }
      // atalho usado pelo site: efeito do humor do quadro
      for (const facing of ['left', 'right'] as const) {
        for (const bloody of [false, true]) {
          const a = effectPixels(SKINS[m].effect, f, facing, 3, bloody).map((p) => [p.x, p.y, p.color]);
          same('efeitos', json(a), json(core.effectPixels(m, name, i, 3, facing, screaming, null, bloody)), () => `atalho ${m}/${name}/${i}/${facing}/sangue=${bloody}`);
        }
      }
    });
  }
}

// 3) grito: sorteio, estado, tremedeira
for (const seed of [0, 1, 7, 42, 123456, 0x7ffffffe, -5]) {
  for (let i = 0; i < 64; i++) {
    same('grito', String(hash01(seed, i)), String(core.hash01(seed, i)), () => `hash01(${seed},${i})`);
    same('grito', String(screamGap(seed, i)), String(core.screamGap(seed, i)), () => `screamGap(${seed},${i})`);
    same('grito', json(shakeOffset(i, seed)), json(core.shakeOffset(i, seed)), () => `shakeOffset(${i},${seed})`);
  }
  for (let t = 0; t < 400_000; t += 997) {
    same('grito', json(screamState(1000 + t, 1000, seed)), json(core.screamState(1000 + t, 1000, seed)), () => `screamState(${t},${seed})`);
  }
}

// 3b) dúvida aleatória: sorteio e estado
for (const seed of [0, 1, 7, 42, 123456, 0x7ffffffe, -5]) {
  for (let i = 0; i < 64; i++) same('dúvida', String(doubtGap(seed, i)), String(core.doubtGap(seed, i)), () => `doubtGap(${seed},${i})`);
  for (let t = 0; t < 600_000; t += 997) {
    same('dúvida', json(doubtState(1000 + t, 1000, seed)), json(core.doubtState(1000 + t, 1000, seed)), () => `doubtState(${t},${seed})`);
  }
}
// ... e o '?' em qualquer humor/animação/quadro/lado (efeito 'question' forçado)
for (const m of MOODS) for (const name of ANIM_NAMES) {
  framesFor(m, name, false).forEach((f, i) => {
    for (const facing of ['left', 'right'] as const) for (let ef = 0; ef < 5; ef++) {
      const a = effectPixels('question', f, facing, ef).map((p) => [p.x, p.y, p.color]);
      same('dúvida', json(a), json(core.effectPixels(m, name, i, ef, facing, false, 'question')), () => `question ${m}/${name}/${i}/${facing}/${ef}`);
    }
  });
}

// 4) fonte, plaquinha e balão
for (let cp = 0x20; cp <= 0x24f; cp++) {
  const ch = String.fromCodePoint(cp);
  same('fonte', String(hasGlyph(ch)), String(ov.hasGlyph(ch)), () => `hasGlyph(${ch})`);
  same('fonte', json(glyph(ch) ?? null), json(ov.glyph(ch) ?? null), () => `glyph(${ch})`);
}
const texts = ['Hora de beber água!', 'hora de beber água!', 'Beba água', 'AÇÃO É ÓTIMO, VOCÊ? SÉRIO: 10-20.', 'Bebeu? Bebeu!', 'Supercalifragilisticexpialidocious agora', 'Água', '', SCREAM_TEXT, 'Ç ã õ à â ê í ú', 'Hoje eu acordei com vontade de pão de queijo', 'Olhe a 6 m por 20 s'];
for (const t of texts) {
  for (const w of [10, 30, 62, 90]) {
    same('fonte', json(wrapText(t, w)), json(ov.wrapText(t, w)), () => `wrapText(${t},${w})`);
    same('fonte', json(wrapBalanced(t, w)), json(ov.wrapBalanced(t, w)), () => `wrapBalanced(${t},${w})`);
  }
  same('fonte', String(textWidth(t)), String(ov.textWidth(t)), () => `textWidth(${t})`);
  same('plaquinha', json(signLayout(t)), json(ov.signLayout(t)), () => `signLayout(${t})`);
  same('plaquinha', renderSign(t).data, ov.renderSign(t).data, () => `renderSign(${t})`);
  for (const kind of ['water', 'update', 'speech', 'posture', 'stretch', 'eyes'] as const) {
    same('plaquinha', renderSign(t, kind).data, ov.renderSign(t, kind).data, () => `renderSign(${t},${kind})`);
  }
}
for (const t of [SCREAM_TEXT, 'AAAA', 'socorro um pato']) {
  for (let tailX = -4; tailX <= TOP_W + 4; tailX += 3) {
    for (let tick = 0; tick < 24; tick++) {
      same('balão', renderBalloon(t, tailX, tick).data, ov.renderBalloon(t, tailX, tick).data, () => `renderBalloon(${t},${tailX},${tick})`);
    }
  }
}
for (let x = 0; x < SIZE; x++) for (const facing of ['left', 'right'] as const) {
  same('balão', String(mouthToTopX(x, facing)), String(ov.mouthToTopX(x, facing)), () => `mouthToTopX(${x},${facing})`);
}

if (fails.length) {
  console.error(`PARIDADE QUEBRADA (${fails.length}${fails.length >= 20 ? '+' : ''} diferenças):\n  ${fails.join('\n  ')}`);
  process.exit(1);
}
const checked = Object.entries(counts).map(([k, v]) => `${v} ${k}`).join(', ');

// ---- imagens: favicon, apple-touch-icon e og-image ----
type RGBA = [number, number, number, number];
const hex = (h: string): RGBA => [parseInt(h.slice(1, 3), 16), parseInt(h.slice(3, 5), 16), parseInt(h.slice(5, 7), 16), 255];

class Img {
  px: Uint8Array;
  w: number;
  h: number;
  constructor(w: number, h: number) { this.w = w; this.h = h; this.px = new Uint8Array(w * h * 4); }
  set(x: number, y: number, c: RGBA): void {
    if (x < 0 || y < 0 || x >= this.w || y >= this.h) return;
    const k = (y * this.w + x) * 4;
    const a = c[3] / 255;
    this.px[k] = Math.round(c[0] * a + this.px[k]! * (1 - a));
    this.px[k + 1] = Math.round(c[1] * a + this.px[k + 1]! * (1 - a));
    this.px[k + 2] = Math.round(c[2] * a + this.px[k + 2]! * (1 - a));
    this.px[k + 3] = Math.max(this.px[k + 3]!, c[3]);
  }
  rect(x: number, y: number, w: number, h: number, c: RGBA): void {
    for (let j = 0; j < h; j++) for (let i = 0; i < w; i++) this.set(x + i, y + j, c);
  }
  blit(src: Uint8Array, sw: number, sh: number, ox: number, oy: number): void {
    for (let y = 0; y < sh; y++) for (let x = 0; x < sw; x++) {
      const s = (y * sw + x) * 4;
      if (src[s + 3] === 0) continue;
      this.set(ox + x, oy + y, [src[s]!, src[s + 1]!, src[s + 2]!, src[s + 3]!]);
    }
  }
  png(): Buffer { return encodePng(this.w, this.h, this.px); }
}

const duckPx = (mood: (typeof MOODS)[number], look: { color: string; accessories: string[] }, anim = 'idle', fi = 0, effectFrame = 0): Uint8Array => {
  const px = new Uint8Array(composeRGBA(SKINS[mood], ANIMS[anim as keyof typeof ANIMS][fi]!, false, look));
  // efeito do humor por cima, igual ao renderer do app (olhando para a direita)
  const f = ANIMS[anim as keyof typeof ANIMS][fi]!;
  const def = EFFECTS[SKINS[mood].effect];
  const fx = def.frames[effectFrame % def.frames.length]!;
  fx.forEach((row, j) => {
    for (let i = 0; i < row.length; i++) {
      const col = EFFECT_COLORS[row[i]!];
      if (!col) continue;
      const x = f.eye.x + def.dx + i, y = f.eye.y + def.dy + j;
      if (x < 0 || y < 0 || x >= SIZE || y >= SIZE) continue;
      const c = hex(col), k = (y * SIZE + x) * 4;
      px[k] = c[0]; px[k + 1] = c[1]; px[k + 2] = c[2]; px[k + 3] = 255;
    }
  });
  return px;
};

const plain = { color: 'amarelo', accessories: [] as string[] };

// favicon 32 e 64: o pato puro
writeFileSync(new URL('favicon-32.png', site), encodePng(32, 32, new Uint8Array(composeRGBA(SKINS.feliz, ANIMS.idle[0]!, false, plain))));
writeFileSync(new URL('favicon-64.png', site), encodePng(64, 64, scaleNearest(new Uint8Array(composeRGBA(SKINS.feliz, ANIMS.idle[0]!, false, plain)), 32, 32, 2)));

// apple-touch-icon 180: fundo céu + pato ×5
{
  const img = new Img(180, 180);
  for (let y = 0; y < 180; y++) {
    const t = y / 179;
    img.rect(0, y, 180, 1, [Math.round(0x9b + (0xff - 0x9b) * t), Math.round(0xdc + (0xe8 - 0xdc) * t), Math.round(0xff + (0xb0 - 0xff) * t), 255]);
  }
  img.blit(scaleNearest(new Uint8Array(composeRGBA(SKINS.feliz, ANIMS.idle[0]!, false, plain)), 32, 32, 5), 160, 160, 10, 14);
  writeFileSync(new URL('apple-touch-icon.png', site), img.png());
}

// og-image 1200x630
const FONT: Record<string, string[]> = {
  A: ['.###.', '#...#', '#...#', '#####', '#...#', '#...#', '#...#'],
  C: ['.###.', '#...#', '#....', '#....', '#....', '#...#', '.###.'],
  D: ['####.', '#...#', '#...#', '#...#', '#...#', '#...#', '####.'],
  E: ['#####', '#....', '#....', '####.', '#....', '#....', '#####'],
  I: ['.###.', '..#..', '..#..', '..#..', '..#..', '..#..', '.###.'],
  L: ['#....', '#....', '#....', '#....', '#....', '#....', '#####'],
  M: ['#...#', '##.##', '#.#.#', '#.#.#', '#...#', '#...#', '#...#'],
  N: ['#...#', '##..#', '#.#.#', '#..##', '#...#', '#...#', '#...#'],
  O: ['.###.', '#...#', '#...#', '#...#', '#...#', '#...#', '.###.'],
  P: ['####.', '#...#', '#...#', '####.', '#....', '#....', '#....'],
  Q: ['.###.', '#...#', '#...#', '#...#', '#.#.#', '#..#.', '.##.#'],
  R: ['####.', '#...#', '#...#', '####.', '#.#..', '#..#.', '#...#'],
  S: ['.####', '#....', '#....', '.###.', '....#', '....#', '####.'],
  T: ['#####', '..#..', '..#..', '..#..', '..#..', '..#..', '..#..'],
  U: ['#...#', '#...#', '#...#', '#...#', '#...#', '#...#', '.###.'],
  W: ['#...#', '#...#', '#...#', '#.#.#', '#.#.#', '##.##', '#...#'],
  ' ': ['.....', '.....', '.....', '.....', '.....', '.....', '.....'],
};
function text(img: Img, s: string, x: number, y: number, k: number, c: RGBA, shadow?: RGBA): number {
  for (const ch of s) {
    const g = FONT[ch]!;
    g.forEach((row, j) => {
      for (let i = 0; i < 5; i++) if (row[i] === '#') {
        if (shadow) img.rect(x + i * k + Math.ceil(k / 2), y + j * k + Math.ceil(k / 2), k, k, shadow);
      }
    });
    g.forEach((row, j) => { for (let i = 0; i < 5; i++) if (row[i] === '#') img.rect(x + i * k, y + j * k, k, k, c); });
    x += 6 * k;
  }
  return x;
}
{
  const W = 1200, H = 630;
  const img = new Img(W, H);
  const top = hex('#7cc8ff'), mid = hex('#bfe6ff'), bot = hex('#ffe7a3');
  for (let y = 0; y < H; y++) {
    const t = y / (H - 1);
    const [a, b, u] = t < 0.6 ? [top, mid, t / 0.6] : [mid, bot, (t - 0.6) / 0.4];
    // degradê em faixas de 6 px para ficar com cara de pixel art
    const band = Math.floor(u * 12) / 12;
    img.rect(0, y, W, 1, [Math.round(a[0] + (b[0] - a[0]) * band), Math.round(a[1] + (b[1] - a[1]) * band), Math.round(a[2] + (b[2] - a[2]) * band), 255]);
  }
  // nuvens de blocos
  const cloud = (cx: number, cy: number, k: number) => {
    const shape = ['...####.....', '..######.##.', '.###########', '############'];
    shape.forEach((row, j) => { for (let i = 0; i < row.length; i++) if (row[i] === '#') img.rect(cx + i * k, cy + j * k, k, k, [255, 255, 255, 230]); });
  };
  cloud(60, 28, 9); cloud(860, 50, 8); cloud(480, 36, 6);
  // chão de grama
  img.rect(0, 540, W, 90, hex('#7ccf5a'));
  img.rect(0, 540, W, 10, hex('#5fb043'));
  for (let x = 0; x < W; x += 20) img.rect(x + ((x / 20) % 2) * 6, 530, 6, 10, hex('#5fb043'));
  // texto
  text(img, 'QUATO', 72, 110, 20, hex('#3b2a18'), [0x3b, 0x2a, 0x18, 70]);
  text(img, 'O PATO', 76, 290, 9, hex('#ff5c8a'), [0x3b, 0x2a, 0x18, 60]);
  text(img, 'QUE MORA NA SUA TELA', 76, 390, 5, hex('#3b2a18'));
  text(img, 'MAC E WINDOWS', 76, 445, 4, hex('#3b6ea8'));
  // pato grande com coroa, mais dois pequenos andando
  img.blit(scaleNearest(duckPx('feliz', { color: 'amarelo', accessories: ['coroa', 'laco'] }), 32, 32, 13), 416, 416, 720, 150);
  img.blit(scaleNearest(duckPx('feliz', { color: 'rosa', accessories: ['oculos-escuros'] }, 'walk', 1, 2), 32, 32, 4), 128, 128, 560, 425);
  img.blit(scaleNearest(duckPx('sono', { color: 'azul', accessories: ['fones'] }, 'sleep', 0, 0), 32, 32, 4), 128, 128, 1060, 425);
  writeFileSync(new URL('og-image.png', site), img.png());
}

console.log(`ok: duck-data.js gerado; paridade pixel a pixel com src/sprites: ${checked}; imagens em site/`);
