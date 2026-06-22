// viz.js - hand-drawn SVG and markup helpers.
//
// Every visualization is inline SVG or CSS: no images, no chart library. These are
// pure functions used by the components to build sparklines, gauges, connection bars,
// and the expanded detail graph.

import { nf, fmt, t } from './i18n.js';

export const LINK_NAMES = { lora: 'LoRa', wifi: 'Wi-Fi', cellular: 'Cellular', nbiot: 'NB-IoT', satellite: 'Satellite', ethernet: 'Ethernet', mesh: 'Mesh' };
export const LINK_COLORS = { lora: '#38bdf8', wifi: '#22d3ee', cellular: '#a855f7', nbiot: '#818cf8', satellite: '#fb923c', ethernet: '#94a3b8', mesh: '#ec4899' };
export const LINK_RSSI = { lora: -112, wifi: -52, cellular: -84, nbiot: -102, satellite: -118, ethernet: -40, mesh: -96 };
export const statusColor = (s) => `var(--${s === 'ok' ? 'ok' : s === 'warn' ? 'warn' : 'alarm'})`;
export const trendArrow = (tr) => (tr === 'rising' ? '↑' : tr === 'falling' ? '↓' : '→');

const esc = (s) => String(s).replace(/[&<>"]/g, (c) => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;' }[c]));
export { esc };

export function sparkPoints(history, w, h, pad = 3)
{
  if (!history || history.length < 2) return null;
  const min = Math.min(...history), max = Math.max(...history), span = max - min || 1;
  const step = w / (history.length - 1);
  return history.map((v, i) => `${(i * step).toFixed(1)},${(h - pad - ((v - min) / span) * (h - pad * 2)).toFixed(1)}`).join(' ');
}

export function miniSpark(history)
{
  const pts = sparkPoints(history, 100, 26);
  if (!pts) return '<svg class="spark" viewBox="0 0 100 26" preserveAspectRatio="none"></svg>';
  return `<svg class="spark" viewBox="0 0 100 26" preserveAspectRatio="none">
    <polyline class="sp-area" points="0,26 ${pts} 100,26"/><polyline class="sp-line" points="${pts}"/></svg>`;
}

export function bars(strength, online)
{
  let h = '<span class="bars">';
  for (let i = 1; i <= 4; i++) h += `<i class="${online && i <= strength ? 'on' : ''}"></i>`;
  return h + '</span>';
}

export function conn(link)
{
  const name = LINK_NAMES[link.kind] || link.kind;
  const color = LINK_COLORS[link.kind] || 'var(--cyan)';
  const dbm = link.online ? (LINK_RSSI[link.kind] ?? -90) + (link.strength - 2) * 6 : null;
  const sig = dbm != null ? `<span class="conn-speed">${dbm} dBm</span>` : '';
  return `<span class="conn ${link.online ? '' : 'off'}" style="--lc:${color}"><span class="conn-kind">${name}</span>${sig}${bars(link.strength, link.online)}</span>`;
}

export function detailGraph(history, band)
{
  const w = 320, h = 100, pad = 12;
  const pts = sparkPoints(history, w, h, pad);
  if (!pts) return `<div class="graph"></div>`;
  const min = Math.min(...history), max = Math.max(...history), span = max - min || 1;
  const y = (v) => h - pad - ((v - min) / span) * (h - pad * 2);
  // Draw a safe-band limit only where it actually falls within the plotted range, so a
  // band far outside the data does not paint a flat box over the whole graph.
  let limits = '';
  if (band)
  {
    for (const edge of band)
    {
      if (edge > min && edge < max)
      {
        const ly = y(edge).toFixed(1);
        limits += `<line class="dg-limit" x1="0" y1="${ly}" x2="${w}" y2="${ly}"/><text class="dg-limit-t" x="${w - 4}" y="${(+ly - 3).toFixed(1)}" text-anchor="end">${nf(edge)}</text>`;
      }
    }
  }
  const grid = [0.5].map((g) => `<line class="dg-grid" x1="0" y1="${(h * g).toFixed(1)}" x2="${w}" y2="${(h * g).toFixed(1)}"/>`).join('');
  const last = pts.split(' ').pop().split(',');
  return `<div class="graph">
    <svg class="detail-graph" viewBox="0 0 ${w} ${h}" preserveAspectRatio="none">
      ${grid}${limits}
      <polyline class="dg-area" points="0,${h} ${pts} ${w},${h}"/>
      <polyline class="dg-line" points="${pts}"/>
      <circle class="dg-dot" cx="${last[0]}" cy="${last[1]}" r="2.6"/>
    </svg>
    <span class="dg-ax dg-max">${nf(max)}</span><span class="dg-ax dg-min">${nf(min)}</span>
  </div>`;
}

export function bannerRing(status)
{
  const color = `var(--${status === 'ok' ? 'ok' : status === 'warn' ? 'warn' : 'alarm'})`;
  const r = 26, c = 2 * Math.PI * r, frac = status === 'ok' ? 1 : status === 'warn' ? 0.66 : 0.4;
  const mark = status === 'ok'
    ? `<path d="M22 33 L29 40 L43 24" fill="none" stroke="${color}" stroke-width="5" stroke-linecap="round" stroke-linejoin="round"/>`
    : `<path d="M33 19 L33 36 M33 44 L33 46" fill="none" stroke="${color}" stroke-width="5" stroke-linecap="round"/>`;
  return `<svg viewBox="0 0 66 66" width="100%" height="100%" aria-hidden="true">
    <circle cx="33" cy="33" r="${r}" fill="none" stroke="var(--track)" stroke-width="4"/>
    <circle cx="33" cy="33" r="${r}" fill="none" stroke="${color}" stroke-width="4" stroke-linecap="round" stroke-dasharray="${(frac * c).toFixed(1)} ${c.toFixed(1)}" transform="rotate(-90 33 33)" style="filter: drop-shadow(0 0 5px ${color})"/>
    ${mark}</svg>`;
}

// --- per-sensor-kind visualizations --------------------------------------
// Each sensor gets the visualization that suits its quantity, so a wall of tiles
// reads as a varied instrument panel rather than identical line graphs.

function rangeOf(band) { const [lo, hi] = band; const pad = Math.max((hi - lo) * 0.5, 1); return [lo - pad, hi + pad]; }
function fracOf(v, band) { if (!band) return 0.5; const [min, max] = rangeOf(band); return Math.min(1, Math.max(0, (v - min) / (max - min || 1))); }

// Safe-band labels for the 270-degree arch gauges: the low value sits at the left arch
// end, the high value at the right, and the unit floats small in the top-right corner so
// it never widens a value or pushes it off its end.
const unitSup = (unit, x, y) => (unit ? `<text class="vv-unit" x="${x}" y="${y}" text-anchor="end">${unit}</text>` : '');
const ends = (band, big, unit = '') => (big && band
  ? `<text class="vv-end" x="6" y="49">${nf(band[0])}</text><text class="vv-end" x="50" y="49" text-anchor="end">${nf(band[1])}</text>${unitSup(unit, 53, 9)}`
  : '');

function radial(r, big)
{
  const v = r.value, band = r.band, rad = 20, c = 2 * Math.PI * rad, sweep = c * 0.75, f = fracOf(v, band);
  let ticks = '';
  for (const p of [0, 0.25, 0.5, 0.75, 1])
  {
    const a = p * 270 * Math.PI / 180;
    ticks += `<line class="vv-tick" x1="${(28 + 15.5 * Math.cos(a)).toFixed(1)}" y1="${(28 + 15.5 * Math.sin(a)).toFixed(1)}" x2="${(28 + 21 * Math.cos(a)).toFixed(1)}" y2="${(28 + 21 * Math.sin(a)).toFixed(1)}"/>`;
  }
  const center = `<text class="vv-num${big ? '' : ' sm'}" x="28" y="31" text-anchor="middle">${fmt(v)}</text>`;
  return `<svg class="tv" viewBox="0 0 56 52">
    <g transform="rotate(135 28 28)">
      <circle class="vv-track" cx="28" cy="28" r="${rad}" stroke-dasharray="${sweep.toFixed(1)} ${c.toFixed(1)}"/>
      ${ticks}
      <circle class="vv-arc" cx="28" cy="28" r="${rad}" stroke-dasharray="${(f * sweep).toFixed(1)} ${c.toFixed(1)}"/>
    </g>${center}${ends(band, big, t('unit.' + r.unit))}</svg>`;
}

function therm(r, big)
{
  const v = r.value, band = r.band, f = fracOf(v, band), top = 6, bottom = 34, h = bottom - top, mh = f * h;
  let ticks = '';
  for (const p of [0, 0.33, 0.66, 1]) ticks += `<line class="vv-tick" x1="18" y1="${(bottom - p * h).toFixed(1)}" x2="21" y2="${(bottom - p * h).toFixed(1)}"/>`;
  const lab = big && band ? `<text class="vv-end" x="23" y="${(top + 3).toFixed(1)}">${nf(band[1])}</text><text class="vv-end" x="23" y="${bottom}">${nf(band[0])}</text>${unitSup(t('unit.' + r.unit), 38, 8)}` : '';
  return `<svg class="tv" viewBox="0 0 ${big ? 40 : 26} 50">
    <rect class="vv-tube" x="9" y="4" width="8" height="34" rx="4"/>
    ${ticks}
    <rect class="vv-merc" x="11" y="${(bottom - mh).toFixed(1)}" width="4" height="${(mh + 8).toFixed(1)}" rx="2"/>
    <circle class="vv-bulb" cx="13" cy="40" r="6.5"/>${lab}</svg>`;
}

function droplet(r, uid, big)
{
  const f = fracOf(r.value, r.band);
  const path = 'M22 6 C32 22 36 28 36 33 a14 14 0 1 1 -28 0 C8 28 12 22 22 6 Z';
  const pct = big ? `<text class="vv-num" x="22" y="36" text-anchor="middle">${Math.round(f * 100)}</text>` : '';
  const cap = big && r.band ? `<text class="vv-end" x="5" y="55">${nf(r.band[0])}</text><text class="vv-end" x="39" y="55" text-anchor="end">${nf(r.band[1])}</text>${unitSup(t('unit.' + r.unit), 42, 8)}` : '';
  return `<svg class="tv" viewBox="0 0 44 ${big ? 58 : 50}"><defs><clipPath id="d${uid}"><path d="${path}"/></clipPath></defs>
    <path class="vv-back" d="${path}"/>
    <g clip-path="url(#d${uid})"><rect class="vv-fill" x="6" width="32" height="50" y="0" style="transform:translateY(${((1 - f) * 50).toFixed(1)}px)"/>
      <ellipse class="vv-wave" cx="22" cy="${((1 - f) * 50).toFixed(1)}" rx="17" ry="3"/></g>
    <path class="vv-shell" d="${path}"/>
    <ellipse class="vv-shine" cx="16" cy="20" rx="3.2" ry="5.5"/>${pct}${cap}</svg>`;
}

// A battery cell: a segmented fill proportional to charge within the safe band, with a
// charging bolt when the reading is rising. Distinct from the trend sparkline below it.
function battery(r, uid)
{
  const f = fracOf(r.value, r.band);
  const x0 = 9, innerW = 52;
  const segs = [1, 2, 3].map((i) => `<line class="bat-seg" x1="${(x0 + innerW * i / 4).toFixed(1)}" y1="12" x2="${(x0 + innerW * i / 4).toFixed(1)}" y2="34"/>`).join('');
  const bolt = r.trend === 'rising' ? '<path class="bat-bolt" d="M40 13 L31 26 L38 26 L35 35 L47 21 L40 21 Z"/>' : '';
  return `<svg class="tv" viewBox="0 0 80 46">
      <defs><clipPath id="bf${uid}"><rect x="${x0}" y="12" width="${innerW}" height="22" rx="3"/></clipPath></defs>
      <rect class="bat-body" x="6" y="9" width="58" height="28" rx="5"/>
      <rect class="bat-cap" x="65" y="16" width="5" height="14" rx="2"/>
      <g clip-path="url(#bf${uid})"><rect class="bat-fill" x="${x0}" y="12" width="${(f * innerW).toFixed(1)}" height="22"/></g>
      ${segs}${bolt}
    </svg>`;
}

function dial(r, big)
{
  const v = r.value, band = r.band, f = fracOf(v, band), cx = 32, cy = 36, rad = 24;
  const a = Math.PI - f * Math.PI, nx = cx + Math.cos(a) * rad * 0.82, ny = cy - Math.sin(a) * rad * 0.82;
  let ticks = '';
  for (const p of [0, 0.25, 0.5, 0.75, 1]) { const t2 = Math.PI - p * Math.PI; ticks += `<line class="vv-tick" x1="${(cx + Math.cos(t2) * (rad - 4)).toFixed(1)}" y1="${(cy - Math.sin(t2) * (rad - 4)).toFixed(1)}" x2="${(cx + Math.cos(t2) * rad).toFixed(1)}" y2="${(cy - Math.sin(t2) * rad).toFixed(1)}"/>`; }
  const lab = big && band ? `<text class="vv-end" x="${cx - rad}" y="48">${nf(band[0])}</text><text class="vv-end" x="${cx + rad}" y="48" text-anchor="end">${nf(band[1])}</text>${unitSup(t('unit.' + r.unit), 60, 9)}` : '';
  return `<svg class="tv" viewBox="0 0 64 50">
    <path class="vv-track" fill="none" d="M${cx - rad} ${cy} A ${rad} ${rad} 0 0 1 ${cx + rad} ${cy}"/>
    <path class="vv-arcline" fill="none" pathLength="100" stroke-dasharray="${(f * 100).toFixed(1)} 100" d="M${cx - rad} ${cy} A ${rad} ${rad} 0 0 1 ${cx + rad} ${cy}"/>
    ${ticks}
    <line class="vv-needle" x1="${cx}" y1="${cy}" x2="${nx.toFixed(1)}" y2="${ny.toFixed(1)}"/>
    <circle class="vv-hub" cx="${cx}" cy="${cy}" r="3"/>${lab}</svg>`;
}

// Shared 270-degree gauge ticks at centre (28,28), matching the radial gauge.
function gaugeTicks()
{
  let ticks = '';
  for (const p of [0, 0.25, 0.5, 0.75, 1])
  {
    const a = p * 270 * Math.PI / 180;
    ticks += `<line class="vv-tick" x1="${(28 + 15.5 * Math.cos(a)).toFixed(1)}" y1="${(28 + 15.5 * Math.sin(a)).toFixed(1)}" x2="${(28 + 21 * Math.cos(a)).toFixed(1)}" y2="${(28 + 21 * Math.sin(a)).toFixed(1)}"/>`;
  }
  return ticks;
}

// An anemometer: the value fills a 270-degree arc against its range (with band-end
// labels when big), and the central rotor spins faster the stronger the wind.
function wind(r, big)
{
  const v = r.value, band = r.band, rad = 20, c = 2 * Math.PI * rad, sweep = c * 0.75, f = fracOf(v, band);
  const arm = (deg) => { const a = deg * Math.PI / 180, x = 28 + 8 * Math.cos(a), y = 28 + 8 * Math.sin(a); return `<line class="vv-arm" x1="28" y1="28" x2="${x.toFixed(1)}" y2="${y.toFixed(1)}"/><circle class="vv-cup" cx="${x.toFixed(1)}" cy="${y.toFixed(1)}" r="2.8"/>`; };
  return `<svg class="tv" viewBox="0 0 56 52">
    <g transform="rotate(135 28 28)">
      <circle class="vv-track" cx="28" cy="28" r="${rad}" stroke-dasharray="${sweep.toFixed(1)} ${c.toFixed(1)}"/>
      ${gaugeTicks()}
      <circle class="vv-arc" cx="28" cy="28" r="${rad}" stroke-dasharray="${(f * sweep).toFixed(1)} ${c.toFixed(1)}"/>
    </g>
    <g class="vv-rotor">${arm(-90)}${arm(30)}${arm(150)}</g>
    <circle class="vv-hub" cx="28" cy="28" r="2.8"/>${ends(band, big, t('unit.' + r.unit))}</svg>`;
}

// A light gauge: illuminance fills a 270-degree arc against its range, and the central
// sun's core grows and its corona brightens the more light there is.
function sun(r, big)
{
  const v = r.value, band = r.band, rad = 20, c = 2 * Math.PI * rad, sweep = c * 0.75, f = fracOf(v, band);
  let rays = '';
  for (let i = 0; i < 8; i++) { const a = i * 45 * Math.PI / 180, x1 = 28 + 9 * Math.cos(a), y1 = 28 + 9 * Math.sin(a), x2 = 28 + 12.5 * Math.cos(a), y2 = 28 + 12.5 * Math.sin(a); rays += `<line class="vv-ray" x1="${x1.toFixed(1)}" y1="${y1.toFixed(1)}" x2="${x2.toFixed(1)}" y2="${y2.toFixed(1)}"/>`; }
  return `<svg class="tv" viewBox="0 0 56 52">
    <g transform="rotate(135 28 28)">
      <circle class="vv-track" cx="28" cy="28" r="${rad}" stroke-dasharray="${sweep.toFixed(1)} ${c.toFixed(1)}"/>
      ${gaugeTicks()}
      <circle class="vv-arc" cx="28" cy="28" r="${rad}" stroke-dasharray="${(f * sweep).toFixed(1)} ${c.toFixed(1)}"/>
    </g>
    <g class="vv-rays" style="opacity:${(0.35 + f * 0.65).toFixed(2)}">${rays}</g>
    <circle class="vv-core" cx="28" cy="28" r="${(4 + f * 4).toFixed(1)}"/>${ends(band, big, t('unit.' + r.unit))}</svg>`;
}

export function isDiscrete(r)
{
  const k = vizFor(r.key, r.unit);
  return k === 'chip' || k === 'valve' || k === 'chain' || k === 'mesh' || k === 'count' || k === 'wave';
}

function count(r, big)
{
  return `<div class="cnt${big ? ' big' : ''}">${fmt(r.value)}</div>`;
}

function mesh(r, big, nodeCount)
{
  const W = 220, H = 112;
  const peers = Math.max(2, Math.min(7, nodeCount != null ? nodeCount : (Math.round(r && r.value) || 5)));
  const learning = !!(r && r.state && r.state.endsWith('learning'));
  const gw = peers;
  const pts = [];
  for (let i = 0; i < peers; i++)
  {
    const tt = peers === 1 ? 0.5 : i / (peers - 1);
    pts.push([26 + tt * (W - 82), H * 0.5 + (i % 2 ? -24 : 24)]);
  }
  pts.push([W - 18, H * 0.5]);
  const links = [];
  for (let i = 0; i < peers - 1; i++) links.push([i, i + 1]);
  for (let i = 0; i < peers; i++) links.push([i, gw]);
  const packets = [];
  for (let i = 0; i < peers; i += 2) packets.push([i, gw]);
  if (!packets.length) packets.push([0, gw]);
  const linkSvg = links.map((l, i) => `<line x1="${pts[l[0]][0].toFixed(1)}" y1="${pts[l[0]][1].toFixed(1)}" x2="${pts[l[1]][0].toFixed(1)}" y2="${pts[l[1]][1].toFixed(1)}" class="msh-link${learning && i === links.length - 1 ? ' weak' : ''}"/>`).join('');
  const pkSvg = packets.map((p, i) =>
  {
    const path = p.map((k, j) => `${j ? 'L' : 'M'}${pts[k][0].toFixed(1)},${pts[k][1].toFixed(1)}`).join(' ');
    const dur = (2.2 + i * 0.5).toFixed(2), b = (i * 0.6).toFixed(2);
    return `<circle r="3" class="msh-pk" opacity="0"><animateMotion dur="${dur}s" begin="${b}s" repeatCount="indefinite" path="${path}"/><animate attributeName="opacity" values="0;1;1;0" keyTimes="0;0.12;0.88;1" dur="${dur}s" begin="${b}s" repeatCount="indefinite"/></circle>`;
  }).join('');
  const nodeSvg = pts.map((c, i) => `<circle cx="${c[0].toFixed(1)}" cy="${c[1].toFixed(1)}" r="${i === gw ? 7.5 : 5}" class="msh-node${i === gw ? ' gw' : ''}"/>`).join('');
  return `<svg class="msh${big ? ' big' : ''}" viewBox="0 0 ${W} ${H}" preserveAspectRatio="xMidYMid meet" aria-hidden="true">${linkSvg}${pkSvg}${nodeSvg}</svg>`;
}

// An acoustic monitor: a waveform of bars whose heights are a fixed envelope (so it reads
// as a real signal, not a bouncing equaliser) under a uniform pulse - there is no per-bar
// phase, so a re-render never snaps it sideways. Ambient is a gentle wave; a threat (any
// non-ambient class, e.g. chainsaw/gunshot) flips it hot: tall jagged spikes, faster, red.
function wave(r, big)
{
  const n = big ? 40 : 28;
  const threat = r.status !== 'ok' || (r.state && !r.state.endsWith('ambient'));
  let bars = '';
  for (let i = 0; i < n; i++)
  {
    const env = 0.34 + 0.4 * Math.abs(Math.sin(i * 0.5)) + 0.22 * Math.abs(Math.sin(i * 1.3 + 0.7));
    const h = Math.max(0.16, Math.min(1, env));
    const delay = (-((i * 0.37) % 1.1)).toFixed(2);
    bars += `<i style="--h:${h.toFixed(2)};animation-delay:${delay}s"></i>`;
  }
  const tag = r.state ? t(r.state) : t('acoustic.ambient');
  const loc = threat ? '<span class="wv-loc">≈ 2.1 km · NE</span>' : '';
  return `<div class="wv${threat ? ' hot' : ''}${big ? ' big' : ''}"><div class="wv-bars" aria-hidden="true">${bars}</div><span class="wv-tag">${esc(tag)}</span>${loc}</div>`;
}

// The tamper-evident log: a row of hash-chained blocks (newest emphasised) and the count
// of sealed records.
function chain(r, big)
{
  const count = Math.round(r.value);
  const n = big ? 6 : 4;
  const hex = (k) => (((Math.abs(k) * 2654435761) >>> 0) % 0x10000).toString(16).padStart(4, '0');
  let row = '';
  for (let i = n - 1; i >= 0; i--)
  {
    row += `<span class="chn-blk${i === 0 ? ' new' : ''}">#${hex(count - i)}</span>`;
    if (i) row += '<span class="chn-arr">→</span>';
  }
  return `<div class="chn${big ? ' big' : ''}">
    <div class="chn-row">${row}</div>
    <div class="chn-foot"><span class="chn-count">${nf(count)}</span><span class="chn-seal">${t('ui.sealed')}</span></div>
  </div>`;
}

const ON_STATE = /(open|on|synced|online|up|nominal|active|ready|optimised|optimized)$/;
function chip(r, big)
{
  const txt = r.state ? t(r.state) : t('status.' + r.status);
  const on = r.state && ON_STATE.test(r.state) ? '1' : '0';
  return `<div class="vchip${big ? ' big' : ''}" data-status="${r.status}" data-on="${on}"><span class="vchip-dot"></span>${esc(txt)}</div>`;
}

function barViz(v, band, big, unit = '')
{
  const max = band ? band[1] : (v * 1.4 || 1);
  const f = Math.max(0, Math.min(1, v / (max || 1)));
  const ticks = band ? `<span class="vb-tick" style="inset-inline-start:${(Math.max(0, Math.min(1, band[0] / (max || 1))) * 100).toFixed(1)}%"></span>` : '';
  const scale = big ? `<div class="vbar-scale"><span>0</span><span>${nf(max)}${unit ? `<sup class="vb-unit">${unit}</sup>` : ''}</span></div>` : '';
  return `<div class="vbar${big ? ' big' : ''}"><div class="vbar-track">${ticks}<i style="inline-size:${(f * 100).toFixed(1)}%"></i></div>${scale}</div>`;
}

// A drip valve: a pipe with a lever that lies along the flow when open (drops fall) and
// turns across it when closed. The state label sits under the glyph.
function valve(r, big)
{
  const open = !!(r.state && (r.state.endsWith('open') || r.state.endsWith('on')));
  const drops = open ? '<circle class="vlv-drop d1" cx="28" cy="33" r="2.1"/><circle class="vlv-drop d2" cx="28" cy="33" r="2.1"/>' : '';
  return `<div class="vlv${big ? ' big' : ''}" data-on="${open ? 1 : 0}">
    <svg class="vlv-svg" viewBox="0 0 56 48" aria-hidden="true">
      <rect class="vlv-pipe" x="4" y="22" width="48" height="8" rx="4"/>
      <circle class="vlv-body" cx="28" cy="26" r="8.5"/>
      <line class="vlv-handle" x1="17" y1="26" x2="39" y2="26" transform="rotate(${open ? 0 : 90} 28 26)"/>
      <circle class="vlv-hub" cx="28" cy="26" r="2.4"/>
      ${drops}
    </svg>
    <span class="vlv-state">${esc(t(r.state))}</span>
  </div>`;
}

// A richer sparkline for the expanded view: a thin (non-scaling) line over an area,
// with a baseline, sample dots, an emphasized current point, and min/max labels.
function bigSpark(history)
{
  const w = 300, h = 116, pad = 14;
  if (!history || history.length < 2) return `<svg class="bspark" viewBox="0 0 ${w} ${h}"></svg>`;
  const min = Math.min(...history), max = Math.max(...history), span = max - min || 1;
  const step = w / (history.length - 1);
  const pts = history.map((v, i) => [+(i * step).toFixed(1), +(h - pad - ((v - min) / span) * (h - pad * 2)).toFixed(1)]);
  const line = pts.map((p) => p.join(',')).join(' ');
  const dots = pts.map((p, i) => `<circle class="bsp-dot${i === pts.length - 1 ? ' last' : ''}" cx="${p[0]}" cy="${p[1]}" r="${i === pts.length - 1 ? 3 : 1.6}"/>`).join('');
  return `<svg class="bspark" viewBox="0 0 ${w} ${h}" preserveAspectRatio="none">
    <line class="bsp-grid" x1="0" y1="${h / 2}" x2="${w}" y2="${h / 2}"/>
    <polyline class="bsp-area" points="0,${h} ${line} ${w},${h}"/>
    <polyline class="bsp-line" points="${line}"/>
    ${dots}
    <text class="bsp-ax" x="${w - 4}" y="12" text-anchor="end">${nf(max)}</text>
    <text class="bsp-ax" x="${w - 4}" y="${h - 4}" text-anchor="end">${nf(min)}</text>
  </svg>`;
}

// The expanded view for a sparkline sensor: a per-sample bar histogram, so it reads as a
// distinct "recent samples" panel rather than a second copy of the History line below it.
// The most recent bar is emphasised, and the band edges (if any) shade as a target zone.
function bigBars(history, band)
{
  const w = 300, h = 120, pad = 12;
  if (!history || history.length < 2) return `<svg class="bbars" viewBox="0 0 ${w} ${h}"></svg>`;
  const min = Math.min(...history), max = Math.max(...history), span = max - min || 1;
  const n = history.length, slot = (w - pad * 2) / n, bw = slot * 0.6;
  const y = (v) => h - pad - ((v - min) / span) * (h - pad * 2);
  let zone = '';
  if (band)
  {
    const lo = Math.max(0, Math.min(1, (band[0] - min) / span)), hi = Math.max(0, Math.min(1, (band[1] - min) / span));
    if (hi > 0 && lo < 1) { const yt = y(min + hi * span), yb = y(min + lo * span); zone = `<rect class="bbar-zone" x="0" y="${yt.toFixed(1)}" width="${w}" height="${Math.max(0, yb - yt).toFixed(1)}"/>`; }
  }
  const bars = history.map((v, i) =>
  {
    const by = y(v), bh = h - pad - by, x = pad + i * slot + (slot - bw) / 2;
    return `<rect class="bbar${i === n - 1 ? ' last' : ''}" x="${x.toFixed(1)}" y="${by.toFixed(1)}" width="${bw.toFixed(1)}" height="${(bh + 1.5).toFixed(1)}" rx="1.2"/>`;
  }).join('');
  // The min/max labels live in HTML, not the SVG, so the none-aspect stretch that fills
  // the bars to full width never warps the text.
  return `<div class="bbars-wrap">
    <svg class="bbars" viewBox="0 0 ${w} ${h}" preserveAspectRatio="none">${zone}${bars}</svg>
    <span class="bb-ax bb-max">${nf(max)}</span><span class="bb-ax bb-min">${nf(min)}</span>
  </div>`;
}

/** Picks the visualization kind for a reading from its unit and key. */
export function vizFor(key, unit)
{
  if (key === 'mesh_relay' || key === 'neighbour_mesh' || key === 'relay_mesh') return 'mesh';
  if (key === 'tamper_log') return 'chain';
  if (key === 'drip_valve') return 'valve';
  if (unit === 'count') return 'count';
  if (unit === 'state') return 'chip';
  if (key === 'soil_trend' || key.endsWith('_trend')) return 'spark';
  if (unit === 'celsius') return 'therm';
  if (key === 'humidity' || key === 'soil_moisture' || unit === 'millimeter') return 'droplet';
  if (key === 'well_level' || key === 'storage_tank' || key === 'ward_power' || key === 'oxygen_stock') return 'bar';
  if (unit === 'hectopascal' || unit === 'liter_per_minute') return 'dial';
  if (unit === 'meter_per_second') return 'wind';
  if (unit === 'lux') return 'sun';
  if (unit === 'decibel') return 'wave';
  if (unit === 'volt' || key === 'battery_voltage' || key === 'battery_level') return 'battery';
  if (unit === 'watt') return 'bar';
  if (unit === 'percent') return 'radial';
  return 'spark';
}

/** Renders the chosen visualization for a sensor, sized by `big`. */
export function tileViz(s, big = false, nodes)
{
  const r = s.reading;
  const uid = (big ? 'b' : 't') + (s.id || r.key).replace(/[^a-z0-9]/gi, '');
  let inner, full = false, disc = false;
  switch (vizFor(r.key, r.unit))
  {
    case 'therm': inner = therm(r, big); break;
    case 'droplet': inner = droplet(r, uid, big); break;
    case 'radial': inner = radial(r, big); break;
    case 'dial': inner = dial(r, big); break;
    case 'wind': inner = wind(r, big); break;
    case 'sun': inner = sun(r, big); break;
    case 'battery': inner = battery(r, uid); break;
    case 'chip': inner = chip(r, big); disc = true; break;
    case 'valve': inner = valve(r, big); disc = true; break;
    case 'chain': inner = chain(r, big); disc = true; break;
    case 'wave': inner = wave(r, big); disc = true; break;
    case 'mesh': inner = mesh(r, big, nodes); disc = true; break;
    case 'count': inner = count(r, big); disc = true; break;
    case 'bar': inner = barViz(r.value, r.band, big, t('unit.' + r.unit)); full = true; break;
    default: inner = big ? bigBars(s.history, r.band) : miniSpark(s.history); full = true;
  }
  // Discrete glyphs (chip/valve) size to their own content rather than the fixed gauge slot.
  const cls = disc ? 'tv-wrap disc' : `tv-wrap${full ? ' full' : ''}${big ? ' big' : ''}`;
  return `<div class="${cls}">${inner}</div>`;
}

// Re-export the formatters used in markup so components import viz helpers from one place.
export { nf, fmt };
