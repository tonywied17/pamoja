import { prefersReducedMotion } from './config.js';

const ACCENT = { farm: '#1fa995', clinic: '#ffb627', water: '#36b6dd', conservation: '#46c97e', village: '#33c9a6', storm: '#f26a4b' };

const NS = 'http://www.w3.org/2000/svg';
const el = (tag, cls, html) => { const n = document.createElement(tag); if (cls) n.className = cls; if (html != null) n.innerHTML = html; return n; };
const lerp = (a, b, k) => a + (b - a) * k;
const pad = (n) => (n < 10 ? '0' + n : '' + n);
const clamp = (v, a, b) => Math.max(a, Math.min(b, v));

// ---- tile renderers: each returns { node, update(s) } ------------------

function tRadial(spec)
{
  const node = el('div', 'tile t-radial');
  const C = 2 * Math.PI * 30, sweep = 0.75; // 270deg gauge
  node.innerHTML = `
    <svg viewBox="0 0 80 72" class="gauge">
      <circle cx="40" cy="38" r="30" class="g-track" transform="rotate(135 40 38)"
        stroke-dasharray="${(C * sweep).toFixed(1)} ${C.toFixed(1)}" stroke-linecap="round"/>
      <circle cx="40" cy="38" r="30" class="g-val" transform="rotate(135 40 38)"
        stroke-dasharray="0 ${C.toFixed(1)}" stroke-linecap="round"/>
      <text x="40" y="40" class="g-num">0</text>
      <text x="40" y="55" class="g-unit">${spec.unit}</text>
    </svg>
    <div class="t-label">${spec.label}</div>`;
  const valC = node.querySelector('.g-val');
  const num = node.querySelector('.g-num');
  const lbl = node.querySelector('.t-label');
  return {
    node,
    update(s)
    {
      const v = s.v[spec.key] || 0;
      const pct = clamp(v / spec.max, 0, 1);
      valC.setAttribute('stroke-dasharray', `${(C * sweep * pct).toFixed(1)} ${C.toFixed(1)}`);
      num.textContent = spec.fmt ? spec.fmt(v) : Math.round(v);
      if (spec.status)
      {
        const [txt, tone] = spec.status(s);
        lbl.innerHTML = `${spec.label} <i class="t-tag ${tone}">${txt}</i>`;
      }
    },
  };
}

function tBar(spec)
{
  const node = el('div', 'tile t-bar');
  node.innerHTML = `
    <div class="t-row"><span class="t-label">${spec.label}</span><span class="t-val"><b>0</b>${spec.unit}</span></div>
    <div class="bar"><i></i></div>`;
  const fill = node.querySelector('.bar i');
  const val = node.querySelector('.t-val b');
  return {
    node,
    update(s)
    {
      const v = s.v[spec.key] || 0; const pct = clamp(v / spec.max, 0, 1);
      fill.style.width = (pct * 100).toFixed(1) + '%';
      val.textContent = spec.fmt ? spec.fmt(v) : Math.round(v);
    },
  };
}

function tKpi(spec)
{
  const node = el('div', 'tile t-kpi');
  node.innerHTML = `<div class="t-label">${spec.label}</div><div class="kpi"><b>0</b><span>${spec.unit}</span></div>`;
  const b = node.querySelector('.kpi b');
  const lbl = node.querySelector('.t-label');
  return {
    node,
    update(s)
    {
      const v = s.v[spec.key] || 0;
      b.textContent = spec.fmt ? spec.fmt(v) : Math.round(v);
      if (spec.status) { const [txt, tone] = spec.status(s); lbl.innerHTML = `${spec.label} <i class="t-tag ${tone}">${txt}</i>`; }
    },
  };
}

function tChip(spec)
{
  const node = el('div', 'tile t-chip');
  node.innerHTML = `<div class="t-label">${spec.label}</div><div class="chip"><span class="chip-dot"></span><b></b></div>`;
  const dot = node.querySelector('.chip-dot'); const b = node.querySelector('.chip b'); const chip = node.querySelector('.chip');
  return {
    node,
    update(s) { const [txt, tone] = spec.state(s); b.textContent = txt; chip.dataset.tone = tone; },
  };
}

function tSpark(spec)
{
  const node = el('div', 'tile t-spark');
  node.innerHTML = `<div class="t-row"><span class="t-label">${spec.label}</span><span class="t-val"><b>0</b>${spec.unit || ''}</span></div>
    <svg viewBox="0 0 200 46" preserveAspectRatio="none" class="spark"><polyline class="sp-line" points=""/><polyline class="sp-area" points=""/></svg>`;
  const line = node.querySelector('.sp-line'); const area = node.querySelector('.sp-area'); const val = node.querySelector('.t-val b');
  const buf = new Array(48).fill(spec.start ?? 50);
  let acc = 0;
  return {
    node,
    update(s, dt)
    {
      acc += dt; if (acc < 0.22) { val.textContent = spec.fmt ? spec.fmt(s.v[spec.key]) : Math.round(s.v[spec.key]); return; } acc = 0;
      buf.push(s.v[spec.key] || 0); buf.shift();
      const max = spec.max || 100;
      const pts = buf.map((v, i) => `${(i / (buf.length - 1) * 200).toFixed(1)},${(44 - clamp(v / max, 0, 1) * 40).toFixed(1)}`).join(' ');
      line.setAttribute('points', pts);
      area.setAttribute('points', `0,46 ${pts} 200,46`);
      val.textContent = spec.fmt ? spec.fmt(s.v[spec.key]) : Math.round(s.v[spec.key]);
    },
  };
}

// Live acoustic monitor: a bar equaliser that spikes on an alert event.
function tWave(spec)
{
  const node = el('div', 'tile t-wave wide');
  const N = 32;
  node.innerHTML = `<div class="t-row"><span class="t-label">${spec.label}</span><span class="t-val acoustic"><b>OK</b></span></div>
    <div class="wave">${Array.from({ length: N }, () => '<i></i>').join('')}</div>`;
  const bars = [...node.querySelectorAll('.wave i')]; const tag = node.querySelector('.acoustic');
  let alert = 0;
  return {
    node,
    update(s, dt)
    {
      if (s.flags.alert) { alert = 1; s.flags.alert = false; }
      alert = Math.max(0, alert - dt * 0.5);
      tag.classList.toggle('hot', alert > 0.1);
      tag.querySelector('b').textContent = alert > 0.1 ? 'CHAINSAW' : 'ambient';
      bars.forEach((b, i) =>
      {
        const base = 0.12 + 0.18 * Math.abs(Math.sin(s.t * 3 + i));
        const spike = alert * (0.5 + 0.5 * Math.sin(s.t * 22 + i * 1.3)) * (1 - Math.abs(i - N / 2) / N);
        b.style.transform = `scaleY(${(base + spike * 2).toFixed(3)})`;
        b.style.opacity = 0.5 + (base + spike) * 0.8;
      });
    },
  };
}

// The tamper-evident, hash-chained log: blocks prepend as records are signed.
function tChain(spec)
{
  const node = el('div', 'tile t-chain wide');
  node.innerHTML = `<div class="t-row"><span class="t-label">${spec.label}</span><span class="t-val mono chain-c">0</span></div><div class="chain"></div>`;
  const row = node.querySelector('.chain'); const count = node.querySelector('.chain-c');
  const hex = () => Math.floor(Math.random() * 0xffff).toString(16).padStart(4, '0');
  let n = 1041;
  const add = () =>
  {
    const blk = el('span', 'block', `#${hex()}`); row.prepend(blk);
    requestAnimationFrame(() => blk.classList.add('in'));
    while (row.children.length > 5) row.lastChild.remove();
    count.textContent = (++n).toLocaleString();
  };
  for (let i = 0; i < 5; i++) add();
  return {
    node,
    update(s) { if (s.flags.sign) { s.flags.sign = false; add(); } },
  };
}

// A compact mesh / signal-flow schematic with travelling packets (SMIL).
function tMesh(spec)
{
  const node = el('div', 'tile t-mesh wide');
  const { nodes, links, packets, sat } = spec.cfg;
  const linkSvg = links.map((l) => `<line x1="${nodes[l[0]][0]}" y1="${nodes[l[0]][1]}" x2="${nodes[l[1]][0]}" y2="${nodes[l[1]][1]}" class="ml${l[2] ? ' weak' : ''}"/>`).join('');
  const pktSvg = packets.map((p, i) =>
  {
    const path = p.map((k, j) => `${j ? 'L' : 'M'}${nodes[k][0]},${nodes[k][1]}`).join(' ');
    return `<circle r="3.2" class="pk"><animateMotion dur="${(1.6 + i * 0.2).toFixed(2)}s" repeatCount="indefinite" begin="${(i * 0.5).toFixed(2)}s" path="${path}"/><animate attributeName="opacity" values="0;1;1;0" dur="${(1.6 + i * 0.2).toFixed(2)}s" repeatCount="indefinite" begin="${(i * 0.5).toFixed(2)}s"/></circle>`;
  }).join('');
  const satSvg = sat ? `<line x1="${nodes[sat.from][0]}" y1="${nodes[sat.from][1]}" x2="${sat.xy[0]}" y2="${sat.xy[1]}" class="ml up"/>
    <circle r="3" class="pk up"><animateMotion dur="1.8s" repeatCount="indefinite" path="M${nodes[sat.from][0]},${nodes[sat.from][1]} L${sat.xy[0]},${sat.xy[1]}"/></circle>
    <g transform="translate(${sat.xy[0] - 7},${sat.xy[1] - 5})" class="sat"><rect width="14" height="9" rx="1.5"/><rect x="-6" y="2.5" width="5" height="4"/><rect x="15" y="2.5" width="5" height="4"/></g>` : '';
  const nodeSvg = nodes.map((n2, i) => `<circle cx="${n2[0]}" cy="${n2[1]}" r="${n2[2] === 'gw' ? 6 : 4.5}" class="mn${n2[2] === 'gw' ? ' gw' : ''}"/>`).join('');
  node.innerHTML = `<div class="t-row"><span class="t-label">${spec.label}</span>${spec.note ? `<span class="t-val mono mesh-note"></span>` : ''}</div>
    <svg viewBox="0 0 220 120" class="mesh">${linkSvg}${satSvg}${pktSvg}${nodeSvg}</svg>`;
  const note = node.querySelector('.mesh-note');
  return {
    node,
    update(s) { if (note && spec.note) note.textContent = spec.note(s); },
  };
}

const RENDER = { radial: tRadial, bar: tBar, kpi: tKpi, chip: tChip, spark: tSpark, wave: tWave, chain: tChain, mesh: tMesh };

// ---- scenario specs ----------------------------------------------------
function step(s, cycle, steps)
{
  const p = s.t % cycle; let idx = 0;
  for (let i = 0; i < steps.length; i++) if (p >= steps[i].at) idx = i;
  const fire = Math.floor(s.t / cycle) * 100 + idx;
  if (s._fire !== fire)
  {
    s._fire = fire; const st = steps[idx];
    if (st.set) for (const k in st.set) s.target[k] = st.set[k];
    if (st.ev) s.flags[st.ev] = true;
    if (st.msg) s.say(st.msg);
  }
}

const SPECS = {
  farm: {
    id: 'farm node · meru-03', link: 'LoRa', dbm: -87,
    init: { soil: 52, well: 64, batt: 4.1, flow: 0 },
    tiles: [
      { type: 'radial', label: 'soil moisture', key: 'soil', unit: '%', max: 100, status: (s) => (s.v.soil < 36 ? ['dry', 'warn'] : ['ok', 'ok']) },
      { type: 'bar', label: 'well level', key: 'well', unit: '%', max: 100 },
      { type: 'chip', label: 'drip valve', state: (s) => (s.v.soil < 36 ? ['OPEN', 'ok'] : ['CLOSED', 'idle']) },
      { type: 'kpi', label: 'battery', key: 'batt', unit: 'V', fmt: (v) => v.toFixed(2) },
      { type: 'spark', label: 'soil trend · 24h', key: 'soil', max: 100, start: 50 },
    ],
    script(s)
    {
      s.v.batt = 3.9 + 0.25 * (0.5 + 0.5 * Math.sin(s.t * 0.2));
      step(s, 13, [
        { at: 0, set: { soil: 33, well: 60 }, msg: 'soil 33% — below threshold' },
        { at: 2.4, set: { soil: 33 }, msg: 'valve → OPEN' },
        { at: 3, set: { soil: 58, well: 55 }, msg: 'irrigating drip line' },
        { at: 7, set: { soil: 54 }, msg: 'soil 54% — target met' },
        { at: 8, msg: 'valve → CLOSED' },
        { at: 9, set: { soil: 44 }, msg: 'published 84 B ↑ LoRa' },
      ]);
    },
  },
  clinic: {
    id: 'health post · kano-01', link: 'NB-IoT', dbm: -102,
    init: { temp: 4.2, power: 88, o2: 71, queued: 0 },
    tiles: [
      { type: 'kpi', label: 'fridge temp', key: 'temp', unit: '°C', fmt: (v) => v.toFixed(1), status: (s) => (s.v.temp > 7.6 || s.v.temp < 2.2 ? ['excursion', 'alert'] : ['in range', 'ok']) },
      { type: 'bar', label: 'ward power', key: 'power', unit: '%', max: 100 },
      { type: 'bar', label: 'oxygen stock', key: 'o2', unit: '%', max: 100 },
      { type: 'chip', label: 'uplink', state: (s) => (s.flags.offline ? [`buffering · ${Math.round(s.v.queued)}`, 'warn'] : ['synced', 'ok']) },
      { type: 'chain', label: 'tamper-evident log' },
    ],
    script(s)
    {
      s.v.temp = 4.2 + 1.6 * Math.sin(s.t * 0.5) + 0.3 * Math.sin(s.t * 2.1);
      step(s, 16, [
        { at: 0, set: { o2: 74, power: 90, queued: 0 }, ev: 'sign', msg: 'temp logged · signed' },
        { at: 3, ev: 'sign', msg: '#hash chained to prior' },
        { at: 5, set: { queued: 6 }, msg: 'link lost — store & forward' },
        { at: 8, set: { queued: 11 }, msg: '11 records buffered' },
        { at: 11, set: { queued: 0 }, msg: 'link up — draining queue' },
        { at: 13, ev: 'sign', msg: 'synced ✓ nothing lost' },
      ]);
      s.flags.offline = (s.t % 16) >= 5 && (s.t % 16) < 11.5;
    },
  },
  water: {
    id: 'water point · jaipur-07', link: 'LoRa', dbm: -91,
    init: { flow: 0, well: 58, tank: 40, draws: 1204 },
    tiles: [
      { type: 'kpi', label: 'flow rate', key: 'flow', unit: 'L/min', fmt: (v) => v.toFixed(0) },
      { type: 'bar', label: 'well level', key: 'well', unit: '%', max: 100 },
      { type: 'bar', label: 'storage tank', key: 'tank', unit: '%', max: 100 },
      { type: 'chip', label: 'pump health', state: (s) => (s.flags.weak ? ['weakening', 'warn'] : ['nominal', 'ok']) },
      { type: 'spark', label: 'flow · today', key: 'flow', max: 16, start: 0 },
    ],
    script(s)
    {
      s.v.tank = clamp(s.v.tank + (s.flags.draw ? 0.4 : -0.05), 20, 92);
      step(s, 11, [
        { at: 0, set: { flow: 0, well: 60 }, msg: 'idle' },
        { at: 2, set: { flow: 12, well: 52 }, ev: 'draw', msg: 'draw detected' },
        { at: 4, set: { flow: 11, well: 49 }, msg: 'flow 11 L/min' },
        { at: 6, set: { flow: 0, well: 56 }, msg: `logged · ${(1204).toLocaleString()} draws today` },
        { at: 8, set: { flow: 9 }, ev: 'draw', msg: 'pump pressure −6% vs baseline' },
      ]);
      s.flags.draw = s.v.flow > 4;
      s.flags.weak = (s.t % 22) > 16;
    },
  },
  conservation: {
    id: 'ranger relay · luangwa-02', link: 'mesh', dbm: -95,
    init: { river: 38, batt: 92, audio: 20 },
    tiles: [
      { type: 'wave', label: 'acoustic monitor' },
      { type: 'bar', label: 'river level', key: 'river', unit: '%', max: 100 },
      { type: 'kpi', label: 'battery', key: 'batt', unit: '%', fmt: (v) => Math.round(v) },
      { type: 'mesh', label: 'relay → ranger post', note: (s) => (s.flags.relaying ? 'relaying' : 'idle'), cfg: { nodes: [[24, 80], [90, 58, 0], [150, 70, 0], [200, 56, 'gw']], links: [[0, 1], [1, 2], [2, 3]], packets: [[0, 1, 2, 3]] } },
    ],
    script(s)
    {
      s.v.batt = 90 + 6 * Math.sin(s.t * 0.1);
      s.v.river = 36 + 4 * Math.sin(s.t * 0.3);
      step(s, 9, [
        { at: 0, msg: 'ambient 41 dB — quiet' },
        { at: 4, ev: 'alert', msg: 'ALERT chainsaw · 2.1 km NE' },
        { at: 5, msg: 'relaying → ranger post' },
        { at: 7, msg: 'delivered · 3 hops · 0.4 s' },
      ]);
      s.flags.relaying = (s.t % 9) >= 5 && (s.t % 9) < 7.5;
    },
  },
  village: {
    id: 'mesh node · pokhara-04', link: 'mesh', dbm: -83,
    init: { neighbors: 5, hops: 3, relayed: 318 },
    tiles: [
      { type: 'mesh', label: 'neighbour mesh', note: (s) => (s.flags.reroute ? 'rerouting' : `cost ${s.v.hops}`), cfg: { nodes: [[20, 60], [80, 30], [80, 95], [150, 55], [200, 80, 'gw']], links: [[0, 1], [0, 2], [1, 3], [2, 3], [3, 4], [1, 4, 1]], packets: [[0, 1, 3, 4], [0, 2, 3, 4]] } },
      { type: 'kpi', label: 'neighbours', key: 'neighbors', unit: '' },
      { type: 'kpi', label: 'hops to gateway', key: 'hops', unit: '' },
      { type: 'chip', label: 'routing', state: (s) => (s.flags.reroute ? ['learning', 'warn'] : ['optimised', 'ok']) },
      { type: 'spark', label: 'messages relayed', key: 'relayed', max: 360, start: 300 },
    ],
    script(s)
    {
      s.v.relayed += s.dt * 1.4;
      step(s, 12, [
        { at: 0, set: { hops: 3, neighbors: 5 }, msg: 'heard A→B (cost 3)' },
        { at: 3, msg: 'flooding suppressed · seen once' },
        { at: 6, set: { hops: 4 }, msg: 'link B→GW lost' },
        { at: 7, set: { hops: 2, neighbors: 5 }, msg: 'learned route via C (cost 2)' },
        { at: 9, msg: 'route optimised · airtime saved' },
      ]);
      s.flags.reroute = (s.t % 12) >= 6 && (s.t % 12) < 9;
    },
  },
  storm: {
    id: 'coast relay · tacloban-09', link: 'sat', dbm: -118,
    init: { carried: 12, queue: 9, batt: 73 },
    tiles: [
      { type: 'chip', label: 'cell grid', state: () => ['DOWN', 'alert'] },
      { type: 'chip', label: 'pamoja mesh', state: () => ['UP', 'ok'] },
      { type: 'mesh', label: 'relay → shared gateway · sat', note: (s) => `${Math.round(s.v.queue)} queued`, cfg: { nodes: [[20, 92], [70, 80, 0], [120, 88, 0], [170, 74, 'gw']], links: [[0, 1], [1, 2], [2, 3]], packets: [[0, 1, 2, 3]], sat: { from: 3, xy: [205, 22] } } },
      { type: 'kpi', label: 'reports carried', key: 'carried', unit: '' },
      { type: 'bar', label: 'uplink queue', key: 'queue', unit: '', max: 16 },
    ],
    script(s)
    {
      s.v.batt = 70 + 5 * Math.sin(s.t * 0.15);
      step(s, 10, [
        { at: 0, set: { queue: 11 }, msg: 'cell tower lost — landfall' },
        { at: 2, msg: 'relaying via coastal mesh' },
        { at: 4, set: { queue: 4, carried: 31 }, msg: 'uplink ↑ one shared gateway (sat)' },
        { at: 7, set: { queue: 8, carried: 38 }, msg: 'carried 38 location reports' },
      ]);
    },
  },
};

// ---- console + deck ----------------------------------------------------
class Console
{
  constructor(fig, name)
  {
    this.fig = fig; this.active = false;
    const spec = SPECS[name]; this.spec = spec;
    const accent = ACCENT[name];
    fig.style.setProperty('--accent', accent);
    fig.classList.add('console');

    this.s = { v: { ...spec.init }, target: { ...spec.init }, t: 0, dt: 0, flags: {}, min: 12 * 60 + 1, lines: [], say: null };
    this.s.say = (msg) =>
    {
      this.s.min += 1 + Math.floor(Math.random() * 2);
      this.s.lines.push({ t: `${pad((this.s.min / 60 | 0) % 24)}:${pad(this.s.min % 60)}`, msg });
      while (this.s.lines.length > 4) this.s.lines.shift();
      this.renderLog();
    };

    const root = el('div', 'cons');
    root.innerHTML = `
      <div class="cons-head">
        <span class="cons-dot"></span>
        <span class="cons-id">${spec.id}</span>
        <span class="cons-badges">
          <span class="cons-link" title="link">${spec.link} <b>${spec.dbm}</b>dBm</span>
          <span class="cons-sig"><i></i><i></i><i></i><i></i></span>
        </span>
      </div>
      <div class="cons-body"></div>
      <div class="cons-log"></div>`;
    const body = root.querySelector('.cons-body');
    this.logEl = root.querySelector('.cons-log');
    this.tiles = spec.tiles.map((ts) => { const t = RENDER[ts.type](ts); body.appendChild(t.node); return t; });
    fig.replaceChildren(root);

    // Seed a couple of log lines and an initial render.
    spec.script(this.s);
    this.tiles.forEach((t) => t.update(this.s, 0));
    this.renderLog();
    if (prefersReducedMotion) { const svgs = fig.querySelectorAll('svg'); svgs.forEach((sv) => sv.pauseAnimations && sv.pauseAnimations()); }
  }

  renderLog()
  {
    this.logEl.innerHTML = this.s.lines.map((l, i) => `<div class="log-line${i === this.s.lines.length - 1 ? ' new' : ''}"><span>${l.t}</span>${l.msg}</div>`).join('');
  }

  update(t, dt)
  {
    const s = this.s; s.t = t; s.dt = dt;
    this.spec.script(s);
    const k = 1 - Math.exp(-dt * 3.2);
    for (const key in s.target) s.v[key] = lerp(s.v[key], s.target[key], k);
    for (const tile of this.tiles) tile.update(s, dt);
  }
}

export function mountConsoles()
{
  const items = [];
  document.querySelectorAll('[data-diorama]').forEach((fig) =>
  {
    const name = fig.dataset.diorama;
    if (SPECS[name]) items.push(new Console(fig, name));
  });
  if (prefersReducedMotion || !items.length) return;

  const io = new IntersectionObserver(
    (es) => es.forEach((e) => { const it = items.find((i) => i.fig === e.target); if (it) it.active = e.isIntersecting; }),
    { rootMargin: '10% 0px', threshold: 0.01 },
  );
  items.forEach((i) => io.observe(i.fig));

  let last = performance.now();
  const loop = (now) =>
  {
    const dt = Math.min(0.05, (now - last) / 1000); last = now;
    const t = now / 1000;
    for (const it of items) if (it.active) it.update(t, dt);
    requestAnimationFrame(loop);
  };
  requestAnimationFrame(loop);
}
