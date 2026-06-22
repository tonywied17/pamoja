// network-view.js - a live mesh topology of the fleet, with a geo map and node debug.
//
// A full-screen overlay (toggled from the top bar) with two tabs:
//   - Topology: the gateway at the hub, groups around it on their links, sensors as
//     leaves; edges coloured by link type with travelling packets; offline groups go
//     dashed/dim; alarming nodes pulse.
//   - Map: the same fleet placed by (mocked) geo coordinates over a graticule.
// Both pan and zoom. Clicking a group node opens an inspection panel with link debug
// (signal, throughput, latency, uptime, issues); clicking a sensor leaf opens its
// detail modal. Built from currentFleet(), so edits and scenarios are reflected.

import { store } from '../store.js';
import { currentFleet } from '../edits.js';
import { open, back } from '../nav.js';
import { sensorDetailBody, stickLog } from '../detail.js';
import { t, nf } from '../i18n.js';
import { LINK_NAMES, LINK_COLORS, esc } from '../viz.js';

const W = 1040, H = 660, SR = 60;

// Mocked site positions (normalized 0..1) across one rural area, spread so nodes and
// labels never collide. A real build would use Group.location GPS over a tile basemap.
const MAP_POS = {
  __gateway: [0.47, 0.45],
  'cold-chain': [0.29, 0.15], maternity: [0.17, 0.27],
  'silo-3': [0.72, 0.20], weather: [0.88, 0.36],
  solar: [0.66, 0.78], river: [0.36, 0.69],
};

// Nominal link characteristics for the inspection panel.
const LINK_SPEC = {
  lora: { speed: '5 kbps', lat: 780, rssi: -112 }, wifi: { speed: '24 Mbps', lat: 11, rssi: -52 },
  cellular: { speed: '1.4 Mbps', lat: 58, rssi: -84 }, nbiot: { speed: '62 kbps', lat: 240, rssi: -102 },
  satellite: { speed: '128 kbps', lat: 640, rssi: -118 },
  ethernet: { speed: '100 Mbps', lat: 3, rssi: -40 }, mesh: { speed: '42 kbps', lat: 130, rssi: -96 },
};

const clamp = (v, a, b) => Math.max(a, Math.min(b, v));

function closeNet()
{
  const st = store.state;
  if (st.netSensor) { store.dispatch('clearNetSensor'); open(() => { }, closeNet); return; }
  if (st.netInspect) { store.dispatch('clearNetInspect'); open(() => { }, closeNet); return; }
  store.dispatch('closeNetwork');
}

/** Opens the network overlay as a single substate (see closeNet for the unwind logic). */
export function openNetworkOverlay()
{
  open(() => store.dispatch('openNetwork'), closeNet);
}

$.component('network-view', {
  state: { tab: 'topology', tick: 0 },

  mounted()
  {
    this._z = 1; this._px = 0; this._py = 0; this._drag = null;
    this._un = store.subscribe(() => this.setState({}));
    this._eff = $.effect(() => { currentFleet(); this.setState({}); });
    this._move = (e) => { if (!this._drag) return; this._px = this._drag.px + (e.clientX - this._drag.x); this._py = this._drag.py + (e.clientY - this._drag.y); this.applyTransform(); };
    this._up = () => { this._drag = null; };
    document.addEventListener('pointermove', this._move);
    document.addEventListener('pointerup', this._up);
  },
  updated() { this.applyTransform(); stickLog(this._el); },
  destroyed()
  {
    if (this._un) this._un();
    if (typeof this._eff === 'function') this._eff();
    document.removeEventListener('pointermove', this._move);
    document.removeEventListener('pointerup', this._up);
  },

  applyTransform() { const g = this._el && this._el.querySelector('.net-scene'); if (g) g.setAttribute('transform', `translate(${this._px} ${this._py}) scale(${this._z})`); },
  zoomIn() { this._z = clamp(this._z * 1.2, 0.5, 3); this.applyTransform(); },
  zoomOut() { this._z = clamp(this._z / 1.2, 0.5, 3); this.applyTransform(); },
  resetView() { this._z = 1; this._px = 0; this._py = 0; this.applyTransform(); },
  onWheel(e) { e.preventDefault(); this._z = clamp(this._z * (e.deltaY < 0 ? 1.12 : 0.89), 0.5, 3); this.applyTransform(); },
  onDown(e) { if (e.button !== 0) return; if (e.target.closest('[data-sid]') || e.target.closest('[data-gid]')) return; this._drag = { x: e.clientX, y: e.clientY, px: this._px, py: this._py }; },

  close() { store.dispatch('closeNetwork'); back(); },
  onOverlay(e) { if (e.target.classList.contains('net-overlay')) this.close(); },
  setTab(tab) { this.state.tab = tab; this.resetView(); },

  onLeaf(e) { const el = e.target.closest('[data-sid]'); if (el) store.dispatch('setNetSensor', el.dataset.sid); },
  onNode(e) { const el = e.target.closest('[data-gid]'); if (el) store.dispatch('setNetInspect', el.dataset.gid); },
  closeInspect() { store.dispatch('clearNetInspect'); },
  closeSensorPanel() { store.dispatch('clearNetSensor'); },

  groupsOf(f) { const out = []; for (const o of f.orgs) for (const g of o.groups) out.push(g); return out; },

  positions(groups, map)
  {
    if (map)
    {
      const ax = 100, ay = 70, aw = W - 200, ah = H - 170;
      const at = (id) => MAP_POS[id] || [0.5, 0.5];
      const gpos = groups.map((g) => { const [nx, ny] = at(g.id); return { g, x: ax + nx * aw, y: ay + ny * ah, ang: 0 }; });
      const [hx, hy] = MAP_POS.__gateway;
      const hub = { x: ax + hx * aw, y: ay + hy * ah };
      gpos.forEach((p) => { p.ang = Math.atan2(p.y - hub.y, p.x - hub.x); });
      return { hub, gpos };
    }
    const n = groups.length || 1, hub = { x: W / 2, y: H / 2 - 6 }, Rr = 232;
    const gpos = groups.map((g, i) => { const ang = -Math.PI / 2 + (i / n) * Math.PI * 2; return { g, x: hub.x + Rr * Math.cos(ang), y: hub.y + Rr * Math.sin(ang), ang }; });
    return { hub, gpos };
  },

  scene(f, map)
  {
    const groups = this.groupsOf(f);
    const owner = {};
    for (const o of f.orgs) for (const gg of o.groups) owner[gg.id] = o.name;
    const { hub, gpos } = this.positions(groups, map);
    const backdrop = map ? this.mapBackdrop() : '';
    let edges = '', packets = '', nodes = '';
    gpos.forEach((p, i) =>
    {
      const color = LINK_COLORS[p.g.link.kind] || '#38e1ff';
      const online = p.g.link.online;
      const path = `M${hub.x.toFixed(1)},${hub.y.toFixed(1)} L${p.x.toFixed(1)},${p.y.toFixed(1)}`;
      const st = p.g.status;
      const pkColor = st === 'alarm' ? 'var(--alarm)' : st === 'warn' ? 'var(--warn)' : color;
      const pkClass = st === 'alarm' ? 'net-pk hot' : 'net-pk';
      edges += `<line x1="${hub.x.toFixed(1)}" y1="${hub.y.toFixed(1)}" x2="${p.x.toFixed(1)}" y2="${p.y.toFixed(1)}" class="net-edge ${online ? '' : 'down'}" data-status="${st}" style="stroke:${color}"/>`;
      if (online) { const dur = (st === 'alarm' ? 1.2 : st === 'warn' ? 1.8 : 2.4 + i * 0.2).toFixed(2); for (let k = 0; k < 2; k++) { const b = (k * dur / 2).toFixed(2); packets += `<circle r="3.6" class="${pkClass}" style="fill:${pkColor}" opacity="0"><animateMotion dur="${dur}s" begin="${b}s" repeatCount="indefinite" path="${path}"/><animate attributeName="opacity" values="0;1;1;0" keyTimes="0;0.1;0.9;1" dur="${dur}s" begin="${b}s" repeatCount="indefinite"/></circle>`; } }
      if (!map)
      {
        const m = p.g.sensors.length || 1;
        p.g.sensors.forEach((s, j) =>
        {
          const la = p.ang + (-0.55 + (m === 1 ? 0.275 : (j / (m - 1)) * 1.1));
          const lx = p.x + SR * Math.cos(la), ly = p.y + SR * Math.sin(la);
          nodes += `<line x1="${p.x.toFixed(1)}" y1="${p.y.toFixed(1)}" x2="${lx.toFixed(1)}" y2="${ly.toFixed(1)}" class="net-twig"/>`;
          nodes += `<circle cx="${lx.toFixed(1)}" cy="${ly.toFixed(1)}" r="6" class="net-leaf" data-sid="${p.g.id}/${s.id}" data-status="${s.reading.status}" @click="onLeaf"><title>${esc(t('label.' + s.reading.key))}</title></circle>`;
        });
      }
      const below = Math.sin(p.ang) >= 0;
      nodes += `<g class="net-node ${store.state.netInspect === p.g.id ? 'sel' : ''}" data-status="${p.g.status}" data-gid="${p.g.id}" style="--lc:${color}" @click="onNode">
        <circle cx="${p.x.toFixed(1)}" cy="${p.y.toFixed(1)}" r="13" class="net-gn"/>
        <circle cx="${p.x.toFixed(1)}" cy="${p.y.toFixed(1)}" r="13" class="net-gn-ring"/>
        ${p.g.status !== 'ok' ? `<circle cx="${(p.x + 11).toFixed(1)}" cy="${(p.y - 11).toFixed(1)}" r="4.5" class="net-badge" data-status="${p.g.status}"/>` : ''}
      </g>
      <text class="net-label" x="${p.x.toFixed(1)}" y="${(p.y + (below ? 34 : -26)).toFixed(1)}" text-anchor="middle">${esc(p.g.name)}</text>
      <text class="net-sub" x="${p.x.toFixed(1)}" y="${(p.y + (below ? 47 : -13)).toFixed(1)}" text-anchor="middle">${esc(owner[p.g.id] || '')}${online ? '' : ' · ' + t('ui.offline')}</text>`;
    });
    return `<g class="net-scene">${backdrop}${edges}${packets}${nodes}
      <g class="net-hub"><circle cx="${hub.x.toFixed(1)}" cy="${hub.y.toFixed(1)}" r="26" class="net-hub-glow"/><circle cx="${hub.x.toFixed(1)}" cy="${hub.y.toFixed(1)}" r="18" class="net-hub-core"/><text x="${hub.x.toFixed(1)}" y="${(hub.y + 4).toFixed(1)}" text-anchor="middle" class="net-hub-t">⌂</text></g>
      <text x="${hub.x.toFixed(1)}" y="${(hub.y + 44).toFixed(1)}" text-anchor="middle" class="net-label">${t('ui.gateway')}</text></g>`;
  },

  // A stylized rural site backdrop for the map tab: contour rings, fields, a river and
  // a track, plus place labels. Purely decorative; it pans and zooms with the scene.
  mapBackdrop()
  {
    const ax = 100, ay = 70, aw = W - 200, ah = H - 170;
    const P = (nx, ny) => [+(ax + nx * aw).toFixed(1), +(ay + ny * ah).toFixed(1)];
    const hub = P(...MAP_POS.__gateway);
    const town = P(0.23, 0.21), farms = P(0.80, 0.28), sol = P(...MAP_POS.solar), riv = P(...MAP_POS.river);
    const road = (a, b) => `<path class="net-road" d="M${a[0]} ${a[1]} Q${(a[0] + b[0]) / 2 + 20} ${(a[1] + b[1]) / 2 - 20} ${b[0]} ${b[1]}"/>`;

    let grid = '';
    for (let i = 1; i < 8; i++) grid += `<line class="net-grat" x1="${(W / 8) * i}" y1="0" x2="${(W / 8) * i}" y2="${H}"/>`;
    for (let i = 1; i < 6; i++) grid += `<line class="net-grat" x1="0" y1="${(H / 6) * i}" x2="${W}" y2="${(H / 6) * i}"/>`;

    // A lake, a winding river feeding it, a forest patch, and irregular fields.
    const water = `<path class="net-water" d="M70 470 q70 -80 190 -55 q95 18 78 100 q-26 95 -160 88 q-150 -8 -108 -133 z"/>`;
    const river = `<path class="net-river" d="M300 60 q34 150 -36 224 q-66 70 -16 150 q34 56 -16 90"/>`;
    const forest = `<path class="net-forest" d="M768 372 q104 -34 156 50 q24 92 -78 124 q-126 22 -158 -70 q-22 -84 80 -128 z"/>`;
    const fields = `
      <path class="net-field" d="M642 110 q120 -28 196 26 q34 86 -52 132 q-150 40 -196 -44 q-30 -78 52 -114 z"/>
      <path class="net-field b" d="M150 120 q92 -26 168 18 q30 70 -40 120 q-130 36 -160 -44 q-22 -66 32 -94 z"/>`;
    const contours = `
      <path class="net-contour" d="M812 300 q96 50 50 168 q-66 100 -196 86"/>
      <path class="net-contour" d="M790 320 q70 48 36 138 q-50 78 -150 74"/>`;
    const roads = `<g class="net-roads">${road(hub, town)}${road(hub, farms)}${road(hub, sol)}${road(hub, riv)}</g>`;

    const compass = `<g class="net-compass" transform="translate(${W - 70} 60)"><circle r="22" class="net-comp-ring"/><path d="M0 -16 L5 4 L0 0 L-5 4 Z" class="net-comp-n"/><text y="-24" text-anchor="middle" class="net-comp-t">N</text></g>`;
    const scale = `<g class="net-scale" transform="translate(70 ${H - 40})"><line x1="0" y1="0" x2="120" y2="0"/><line x1="0" y1="-4" x2="0" y2="4"/><line x1="120" y1="-4" x2="120" y2="4"/><text x="60" y="-8" text-anchor="middle">2 km</text></g>`;
    const places = `
      <text class="net-place" x="200" y="118">Nkuene</text>
      <text class="net-place" x="840" y="470">Kithoka ridge</text>
      <text class="net-place net-river-t" x="246" y="180">Kazita river</text>
      <text class="net-place net-river-t" x="120" y="420">Marima lake</text>`;
    return `<g class="net-bg">${grid}${water}${river}${forest}${fields}${contours}${roads}${compass}${scale}${places}</g>`;
  },

  findSensor(f, sid)
  {
    if (!sid) return null;
    const [gid, sid2] = sid.split('/');
    for (const o of f.orgs) for (const g of o.groups) { if (g.id !== gid) continue; const s = g.sensors.find((x) => x.id === sid2); if (s) return { org: o, group: g, sensor: s }; }
    return null;
  },

  sensorPanel(f)
  {
    const found = this.findSensor(f, store.state.netSensor); if (!found) return '';
    const { group, sensor: s } = found;
    return `
      <div class="net-detail" data-status="${s.reading.status}">
        <div class="ins-head"><div><div class="ins-title">${esc(t('label.' + s.reading.key))}</div><div class="ins-sub">${esc(group.name)}</div></div>
          <button class="modal-close sm" type="button" @click="closeSensorPanel">✕</button></div>
        ${sensorDetailBody(s)}
      </div>`;
  },

  inspectPanel(f)
  {
    const id = store.state.netInspect; if (!id) return '';
    const g = this.groupsOf(f).find((x) => x.id === id); if (!g) return '';
    const spec = LINK_SPEC[g.link.kind] || LINK_SPEC.lora;
    const rssi = spec.rssi + (g.link.strength - 2) * 6;
    const issues = g.sensors.filter((s) => s.reading.status !== 'ok');
    const pkts = Math.round(4 + g.sensors.length * 1.5);
    const row = (k, v) => `<div class="ins-row"><span>${k}</span><b>${v}</b></div>`;
    const issuesHtml = issues.length
      ? issues.map((s) => `<div class="ins-issue" data-level="${s.reading.status === 'alarm' ? 'error' : 'warn'}">${esc(t('label.' + s.reading.key))} · ${nf(s.reading.value)}${t('unit.' + s.reading.unit)}</div>`).join('')
      : `<div class="ins-ok">${t('ui.allClear')}</div>`;
    return `
      <div class="net-inspect" data-status="${g.status}">
        <div class="ins-head"><div><div class="ins-title">${esc(g.name)}</div><div class="ins-sub">${LINK_NAMES[g.link.kind] || g.link.kind} · ${t('status.' + g.status)}</div></div>
          <button class="modal-close sm" type="button" @click="closeInspect">✕</button></div>
        ${row(t('ui.link'), (g.link.online ? t('ui.online') : t('ui.offline')))}
        ${row(t('ui.signal'), `${rssi} dBm · ${g.link.strength}/4`)}
        ${row(t('ui.throughput'), spec.speed)}
        ${row(t('ui.latency'), `${spec.lat} ms`)}
        ${row(t('ui.packets'), `${pkts}/s`)}
        ${row(t('ui.sensors'), nf(g.sensors.length))}
        <div class="ins-sec">${t('ui.issues')}</div>
        <div class="ins-issues">${issuesHtml}</div>
        <button class="ins-open" type="button" data-gid="${g.id}" @click="onOpenGroup">${t('ui.group')} →</button>
      </div>`;
  },

  onOpenGroup(e)
  {
    const el = e.target.closest('[data-gid]'); if (!el) return;
    const gid = el.dataset.gid;
    open(() => store.dispatch('setGroupView', gid), () => store.dispatch('clearGroupView'));
  },

  render()
  {
    if (!store.state.network) return '<div hidden></div>';
    const f = currentFleet();
    if (!f) return '<div hidden></div>';
    const map = this.state.tab === 'map';
    return `
      <div class="net-overlay" @click="onOverlay">
        <div class="net-panel">
          <div class="net-head">
            <div>
              <div class="net-title">${t('ui.network')}</div>
              <div class="net-subtitle">${nf(this.groupsOf(f).length)} ${t('ui.groups')} · ${t('ui.networkHint')}</div>
            </div>
            <div class="net-tabs">
              <button class="net-tab ${!map ? 'on' : ''}" type="button" @click="setTab('topology')">${t('ui.topology')}</button>
              <button class="net-tab ${map ? 'on' : ''}" type="button" @click="setTab('map')">${t('ui.map')}</button>
            </div>
            <button class="modal-close" type="button" @click="close" aria-label="${esc(t('ui.cancel'))}">✕</button>
          </div>
          <div class="net-stage">
            <svg class="net-svg" viewBox="0 0 ${W} ${H}" preserveAspectRatio="xMidYMid meet" @wheel="onWheel" @pointerdown="onDown">
              ${this.scene(f, map)}
            </svg>
            <div class="net-zoom">
              <button type="button" @click="zoomIn" aria-label="zoom in">+</button>
              <button type="button" @click="zoomOut" aria-label="zoom out">−</button>
              <button type="button" @click="resetView" aria-label="reset">⟲</button>
            </div>
            ${this.inspectPanel(f)}
            ${this.sensorPanel(f)}
          </div>
          <div class="net-legend">${Object.keys(LINK_NAMES).map((k) => `<span class="net-leg"><i style="background:${LINK_COLORS[k]}"></i>${LINK_NAMES[k]}</span>`).join('')}</div>
        </div>
      </div>`;
  },
});
