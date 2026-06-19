import { PALETTE, prefersReducedMotion } from './config.js';

const ACCENT = { cream: PALETTE.cream, teal: PALETTE.teal, amber: PALETTE.amber, coral: PALETTE.coral, sky: PALETTE.sky, forest: PALETTE.forest };
const rgb = (hex) => { const n = parseInt(hex.slice(1), 16); return `${(n >> 16) & 255},${(n >> 8) & 255},${n & 255}`; };

const fx = [];
let running = false;

export function mountCardFx(host, colorName, motif)
{
  const canvas = document.createElement('canvas');
  canvas.className = 'bc-fx';
  canvas.setAttribute('aria-hidden', 'true');
  host.prepend(canvas);

  const item = {
    host, canvas, ctx: canvas.getContext('2d'),
    a: rgb(ACCENT[colorName] || PALETTE.amber),
    b: rgb(PALETTE.teal),
    motif: motif || 'aurora',
    w: 0, h: 0, dpr: 1, visible: true,
  };
  if (item.motif === 'mesh')
  {
    item.nodes = [[0.18, 0.34], [0.44, 0.62], [0.72, 0.28], [0.85, 0.64], [0.52, 0.86], [0.28, 0.72]];
    item.edges = [[0, 1], [1, 2], [2, 3], [1, 4], [4, 5], [5, 0], [1, 3]];
  }
  if (item.motif === 'core')
    item.parts = Array.from({ length: 18 }, () => ({ a: Math.random() * 6.283, sp: 0.05 + Math.random() * 0.06, ph: Math.random() }));
  fx.push(item);
  new IntersectionObserver(
    (es) => es.forEach((e) => { item.visible = e.isIntersecting; }),
    { threshold: 0.01 },
  ).observe(host);

  if (prefersReducedMotion) { if (fit(item)) paint(item, 3); return; }
  if (!running) { running = true; requestAnimationFrame(loop); }
}

function fit(it)
{
  const dpr = Math.min(2, window.devicePixelRatio || 1);
  const w = it.host.clientWidth, h = it.host.clientHeight;
  if (!w || !h) return false;
  if (w === it.w && h === it.h && dpr === it.dpr) return true;
  it.w = w; it.h = h; it.dpr = dpr;
  it.canvas.width = Math.round(w * dpr); it.canvas.height = Math.round(h * dpr);
  it.ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
  return true;
}

function paint(it, t)
{
  const { ctx, w, h } = it;
  ctx.clearRect(0, 0, w, h);
  ctx.globalCompositeOperation = 'lighter';
  const A = (al) => `rgba(${it.a},${al})`;
  const B = (al) => `rgba(${it.b},${al})`;
  switch (it.motif)
  {
    case 'core': core(ctx, w, h, t, A, it); break;
    case 'broadcast': broadcast(ctx, w, h, t, A); break;
    case 'mesh': mesh(ctx, w, h, t, it, A, B); break;
    case 'shield': shield(ctx, w, h, t, A); break;
    case 'bars': bars(ctx, w, h, t, A, B); break;
    default: aurora(ctx, w, h, t, A, B);
  }
  ctx.globalCompositeOperation = 'source-over';
}

function core(ctx, w, h, t, A, it)
{
  const cx = w / 2, cy = h / 2, max = Math.max(w, h) * 0.6;
  const g = ctx.createRadialGradient(cx, cy, 0, cx, cy, max * 0.5);
  g.addColorStop(0, A(0.2)); g.addColorStop(1, A(0));
  ctx.fillStyle = g; ctx.fillRect(0, 0, w, h);
  for (const p of it.parts)
  {
    const f = (t * p.sp + p.ph) % 1;
    const r = (1 - f) * max;
    ctx.fillStyle = A(0.12 + f * 0.45);
    ctx.beginPath(); ctx.arc(cx + Math.cos(p.a) * r, cy + Math.sin(p.a) * r, 1.6, 0, 7); ctx.fill();
  }
}

function broadcast(ctx, w, h, t, A)
{
  const ox = w * 0.2, oy = h * 0.52, max = w * 0.85;
  for (let i = 0; i < 4; i++)
  {
    const p = (t * 0.24 + i / 4) % 1;
    ctx.strokeStyle = A((1 - p) * 0.18); ctx.lineWidth = 2;
    ctx.beginPath(); ctx.arc(ox, oy, p * max, 0, 7); ctx.stroke();
  }
  ctx.fillStyle = A(0.5);
  ctx.beginPath(); ctx.arc(ox, oy, 3, 0, 7); ctx.fill();
}

function mesh(ctx, w, h, t, it, A, B)
{
  const ns = it.nodes.map(([x, y]) => [x * w, y * h]);
  ctx.lineWidth = 1;
  it.edges.forEach(([i, j], k) =>
  {
    ctx.strokeStyle = B(0.12);
    ctx.beginPath(); ctx.moveTo(ns[i][0], ns[i][1]); ctx.lineTo(ns[j][0], ns[j][1]); ctx.stroke();
    const p = (t * 0.26 + k * 0.16) % 1;
    const px = ns[i][0] + (ns[j][0] - ns[i][0]) * p, py = ns[i][1] + (ns[j][1] - ns[i][1]) * p;
    const pg = ctx.createRadialGradient(px, py, 0, px, py, 6);
    pg.addColorStop(0, A(0.6)); pg.addColorStop(1, A(0));
    ctx.fillStyle = pg; ctx.beginPath(); ctx.arc(px, py, 6, 0, 7); ctx.fill();
  });
  ns.forEach(([x, y], i) =>
  {
    const pulse = 0.5 + 0.5 * Math.sin(t * 1.6 + i);
    const g = ctx.createRadialGradient(x, y, 0, x, y, 7);
    g.addColorStop(0, A(0.22 + pulse * 0.16)); g.addColorStop(1, A(0));
    ctx.fillStyle = g; ctx.beginPath(); ctx.arc(x, y, 7, 0, 7); ctx.fill();
    ctx.fillStyle = A(0.5); ctx.beginPath(); ctx.arc(x, y, 1.8, 0, 7); ctx.fill();
  });
}

function shield(ctx, w, h, t, A)
{
  const s = Math.min(w, h) * 0.62, wS = s * 0.62, hS = s * 0.84, pulse = 0.5 + 0.5 * Math.sin(t * 1.1);
  ctx.save();
  ctx.translate(w / 2, h / 2);
  ctx.beginPath();
  ctx.moveTo(0, -hS / 2);
  ctx.lineTo(wS / 2, -hS * 0.3);
  ctx.lineTo(wS / 2, hS * 0.1);
  ctx.quadraticCurveTo(wS / 2, hS * 0.42, 0, hS / 2);
  ctx.quadraticCurveTo(-wS / 2, hS * 0.42, -wS / 2, hS * 0.1);
  ctx.lineTo(-wS / 2, -hS * 0.3);
  ctx.closePath();
  const g = ctx.createRadialGradient(0, 0, 0, 0, 0, hS * 0.6);
  g.addColorStop(0, A(0.05 + pulse * 0.07)); g.addColorStop(1, A(0));
  ctx.fillStyle = g; ctx.fill();
  ctx.strokeStyle = A(0.12 + pulse * 0.1); ctx.lineWidth = 1.5; ctx.stroke();
  ctx.restore();
}

function bars(ctx, w, h, t, A, B)
{
  const n = 9, mid = h / 2, maxH = h * 0.5;
  for (let i = 0; i < n; i++)
  {
    const x = w * (0.1 + 0.8 * (i / (n - 1)));
    const hh = (0.22 + 0.55 * (0.5 + 0.5 * Math.sin(t * 1.3 + i * 0.6))) * maxH;
    ctx.fillStyle = i % 2 ? B(0.12) : A(0.15);
    ctx.fillRect(x - 2, mid - hh / 2, 4, hh);
  }
}

function aurora(ctx, w, h, t, A, B)
{
  [[A, 0.06, 0.045, 0, 1.3, 0.95], [B, -0.05, 0.055, 2.1, 0.4, 1.05]].forEach(([C, sx, sy, px, py, r]) =>
  {
    const x = w * 0.5 + Math.cos(t * sx + px) * w * 0.42;
    const y = h * 0.5 + Math.sin(t * sy + py) * h * 0.55;
    const g = ctx.createRadialGradient(x, y, 0, x, y, Math.max(w, h) * r);
    g.addColorStop(0, C(0.17)); g.addColorStop(1, C(0));
    ctx.fillStyle = g; ctx.fillRect(0, 0, w, h);
  });
}

function loop(now)
{
  requestAnimationFrame(loop);
  const t = now / 1000;
  for (const it of fx) { if (it.visible && fit(it)) paint(it, t); }
}
