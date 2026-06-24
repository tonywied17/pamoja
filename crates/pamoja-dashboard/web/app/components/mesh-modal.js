// mesh-modal.js - the interactive neighbour-mesh map for a mesh sensor.
//
// A full-screen, pan/zoom map of one group's mesh node: the gateway at the top, the group
// hub in the middle, and the group's sensors as leaves around it (mock geo positions),
// with packets travelling sensor -> hub -> gateway. Click a node to dock an inspector
// (role, status, reading, link speed, traffic, latency) and open the sensor's full detail.
// The dashboard/group tile stays the static preview; this is the "show more" view.

import { store } from '../store.js';
import { currentFleet } from '../lib/edits.js';
import { open, back } from '../nav.js';
import { t, nf, fmt } from '../lib/i18n.js';
import { catalog } from '../lib/catalog.js';
import { LINK_NAMES, LINK_COLORS, LINK_RSSI, isDiscrete, vizFor, esc } from '../lib/viz/index.js';

const W = 900, H = 560;

/**
 * Finds a group and its owning org by group id.
 *
 * @param {object} f - the current fleet.
 * @param {string} gid - the group id.
 * @returns {{org: object, group: object}|null} the match, or null.
 */
function findGroup(f, gid)
{
  for (const o of f.orgs) for (const g of o.groups) if (g.id === gid) return { org: o, group: g };
  return null;
}

/**
 * Closes the mesh overlay one layer at a time: first a docked node, then the map.
 *
 * @returns {void}
 */
function closeMesh()
{
  if (store.state.meshNode) { store.dispatch('clearMeshNode'); open(() => { }, closeMesh); return; }
  store.dispatch('closeMeshView');
}

/**
 * Opens the mesh map overlay for a sensor id (gid/sid).
 *
 * @param {string} sid - the `gid/sid` key of the mesh sensor.
 * @returns {void}
 */
export function openMeshOverlay(sid)
{
  open(() => store.dispatch('openMeshView', sid), closeMesh);
}

$.component('mesh-modal', {
  /** Initializes pan/zoom state, subscriptions, and document pointer listeners. */
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
  /** Re-applies the pan/zoom transform after a re-render. */
  updated() { this.applyTransform(); },
  /** Tears down subscriptions and document pointer listeners. */
  destroyed()
  {
    if (this._un) this._un();
    if (typeof this._eff === 'function') this._eff();
    document.removeEventListener('pointermove', this._move);
    document.removeEventListener('pointerup', this._up);
  },

  /** Writes the current pan/zoom onto the scene group. */
  applyTransform() { const g = this._el && this._el.querySelector('.mm-scene'); if (g) g.setAttribute('transform', `translate(${this._px} ${this._py}) scale(${this._z})`); },
  /** Zooms in one step. */
  zoomIn() { this._z = $.clamp(this._z * 1.2, 0.5, 3); this.applyTransform(); },
  /** Zooms out one step. */
  zoomOut() { this._z = $.clamp(this._z / 1.2, 0.5, 3); this.applyTransform(); },
  /** Resets pan and zoom to the default view. */
  resetView() { this._z = 1; this._px = 0; this._py = 0; this.applyTransform(); },
  /**
   * Zooms toward the wheel direction.
   *
   * @param {WheelEvent} e - the wheel event.
   * @returns {void}
   */
  onWheel(e) { e.preventDefault(); this._z = $.clamp(this._z * (e.deltaY < 0 ? 1.12 : 0.89), 0.5, 3); this.applyTransform(); },
  /**
   * Begins a pan drag, unless the pointer landed on a node.
   *
   * @param {PointerEvent} e - the pointer-down event.
   * @returns {void}
   */
  onDown(e) { if (e.button !== 0) return; if (e.target.closest('[data-node]')) return; this._drag = { x: e.clientX, y: e.clientY, px: this._px, py: this._py }; },
  /** Closes the overlay and unwinds one history entry. */
  close() { store.dispatch('closeMeshView'); back(); },
  /**
   * Closes the overlay when the scrim itself is clicked.
   *
   * @param {MouseEvent} e - the click event.
   * @returns {void}
   */
  onOverlay(e) { if (e.target.classList.contains('net-overlay')) this.close(); },
  /**
   * Docks the inspector for the clicked node.
   *
   * @param {MouseEvent} e - the click event.
   * @returns {void}
   */
  onNode(e) { const el = e.target.closest('[data-node]'); if (el) store.dispatch('setMeshNode', el.dataset.node); },
  /** Closes the docked node inspector. */
  closeNode() { store.dispatch('clearMeshNode'); },
  /**
   * Opens the full detail modal for the inspected sensor.
   *
   * @param {MouseEvent} e - the click event.
   * @returns {void}
   */
  viewSensor(e)
  {
    const el = e.target.closest('[data-sid]'); if (!el) return;
    open(() => store.dispatch('selectSensor', el.dataset.sid), () => store.dispatch('closeSensor'));
  },

  /**
   * Builds the node positions, links, and packet paths for a group's mesh.
   *
   * @param {object} group - the group to lay out.
   * @returns {{nodes: Array, links: Array, packets: Array, pos: object}} the topology.
   */
  topology(group)
  {
    const cx = W * 0.5, cy = H * 0.54;
    const hub = { key: 'hub', role: 'hub', x: cx, y: cy, name: group.name, status: group.status, link: group.link, group };
    const gw = { key: 'gw', role: 'gateway', x: cx, y: H * 0.13, status: 'ok' };
    const nodes = [gw, hub];
    const sensors = (group.sensors || []).filter((s) => vizFor(s.reading.key, s.reading.unit) !== 'mesh');
    const n = sensors.length || 1;
    const gap = 0.62;
    sensors.forEach((s, i) =>
    {
      const tt = n === 1 ? 0.5 : i / (n - 1);
      const a = (-Math.PI / 2 + gap) + tt * (2 * Math.PI - 2 * gap);
      const jit = 0.92 + 0.08 * Math.abs(Math.sin(i * 2.7));
      nodes.push({
        key: 's:' + s.id, role: 'sensor', sid: group.id + '/' + s.id, sensor: s,
        x: cx + Math.cos(a) * W * 0.3 * jit,
        y: cy + Math.sin(a) * H * 0.34 * jit,
        name: t('label.' + s.reading.key), status: s.reading.status,
      });
    });
    const links = [['hub', 'gw']];
    sensors.forEach((s) => links.push(['s:' + s.id, 'hub']));
    const packets = [['hub', 'gw']];
    sensors.forEach((s, i) => { if (i % 2 === 0) packets.push(['s:' + s.id, 'hub', 'gw']); });
    const pos = {}; nodes.forEach((nd) => { pos[nd.key] = nd; });
    return { nodes, links, packets, pos };
  },

  /**
   * Renders the docked inspector for a selected node (gateway, hub, or sensor).
   *
   * @param {object} group - the group being inspected.
   * @param {object} [node] - the selected node; an empty node renders nothing.
   * @returns {string} the inspector markup.
   */
  inspector(group, node)
  {
    if (!node) return '';
    const dbm = (LINK_RSSI[group.link.kind] ?? -90) + (group.link.strength - 2) * 6;
    const speed = catalog.linkSpec[group.link.kind]?.speed || '-';
    const row = (k, v) => `<div class="ins-row"><span>${k}</span><b>${v}</b></div>`;
    let title, sub, body, foot = '';
    if (node.role === 'gateway')
    {
      title = t('ui.gateway'); sub = LINK_NAMES[group.link.kind] || group.link.kind;
      body = row(t('ui.link'), t('ui.online')) + row(t('ui.throughput'), speed) + row(t('ui.latency'), '12 ms');
    } else if (node.role === 'hub')
    {
      title = group.name; sub = `${LINK_NAMES[group.link.kind] || group.link.kind} · ${t('status.' + group.status)}`;
      const sCount = group.sensors.filter((s) => vizFor(s.reading.key, s.reading.unit) !== 'mesh').length;
      body = row(t('ui.signal'), `${dbm} dBm`) + row(t('ui.throughput'), speed) + row(t('ui.sensors'), nf(sCount));
    } else
    {
      const r = node.sensor.reading;
      const reading = isDiscrete(r) ? (r.state ? t(r.state) : t('status.' + r.status)) : `${fmt(r.value)} ${t('unit.' + r.unit)}`;
      const traffic = Math.round(3 + (node.sensor.id.length % 5) + (r.status === 'alarm' ? 9 : 0));
      title = node.name; sub = t('status.' + r.status);
      body = row(t('ui.reading') || 'Reading', reading) + row(t('ui.throughput'), `${traffic}/s`) + row(t('ui.latency'), `${40 + traffic * 6} ms`);
      foot = `<button class="ins-open" type="button" data-sid="${node.sid}" @click="viewSensor">${t('ui.sensorId')} →</button>`;
    }
    return `<div class="net-inspect" data-status="${node.status || 'ok'}">
        <div class="ins-head"><div><div class="ins-title">${esc(title)}</div><div class="ins-sub">${esc(sub)}</div></div>
          <button class="modal-close sm" type="button" @click="closeNode" aria-label="${esc(t('ui.cancel'))}">✕</button></div>
        ${body}${foot}
      </div>`;
  },

  /**
   * Renders the mesh overlay for the active mesh sensor's group.
   *
   * @returns {string} the overlay markup, or an empty placeholder when inactive.
   */
  render()
  {
    const id = store.state.meshView;
    if (!id) return '<div hidden></div>';
    const f = currentFleet();
    if (!f) return '<div hidden></div>';
    const found = findGroup(f, id.split('/')[0]);
    if (!found) return '<div hidden></div>';
    const group = found.group;
    const color = LINK_COLORS[group.link.kind] || '#38e1ff';
    const { nodes, links, packets, pos } = this.topology(group);
    const sel = store.state.meshNode;
    const sCount = nodes.filter((nd) => nd.role === 'sensor').length;

    const linkSvg = links.map((l) => `<line x1="${pos[l[0]].x.toFixed(1)}" y1="${pos[l[0]].y.toFixed(1)}" x2="${pos[l[1]].x.toFixed(1)}" y2="${pos[l[1]].y.toFixed(1)}" class="mm-link"/>`).join('');
    const pkSvg = packets.map((p, i) =>
    {
      const path = p.map((k, j) => `${j ? 'L' : 'M'}${pos[k].x.toFixed(1)},${pos[k].y.toFixed(1)}`).join(' ');
      const dur = (2.4 + i * 0.4).toFixed(2), b = (i * 0.5).toFixed(2);
      return `<circle r="3.4" class="mm-pk" opacity="0"><animateMotion dur="${dur}s" begin="${b}s" repeatCount="indefinite" path="${path}"/><animate attributeName="opacity" values="0;1;1;0" keyTimes="0;0.12;0.88;1" dur="${dur}s" begin="${b}s" repeatCount="indefinite"/></circle>`;
    }).join('');
    const nodeSvg = nodes.map((nd) =>
    {
      const r = nd.role === 'gateway' ? 16 : nd.role === 'hub' ? 18 : 10;
      const cls = `mm-node ${nd.role}${sel === nd.key ? ' sel' : ''}`;
      const glyph = nd.role === 'gateway' ? `<text class="mm-gly" x="${nd.x.toFixed(1)}" y="${(nd.y + 4).toFixed(1)}" text-anchor="middle">⌂</text>` : '';
      const label = `<text class="mm-label" x="${nd.x.toFixed(1)}" y="${(nd.y + r + 14).toFixed(1)}" text-anchor="middle">${esc(nd.role === 'gateway' ? t('ui.gateway') : nd.name)}</text>`;
      return `<g class="${cls}" data-node="${nd.key}" data-status="${nd.status || 'ok'}" @click="onNode"><circle cx="${nd.x.toFixed(1)}" cy="${nd.y.toFixed(1)}" r="${r}" class="mm-dot"/></g>${glyph}${label}`;
    }).join('');

    return `
      <div class="net-overlay" @click="onOverlay">
        <div class="net-panel" data-status="${group.status}" style="--lc:${color}" role="dialog" aria-modal="true">
          <div class="net-head">
            <div>
              <div class="net-title">${esc(group.name)}</div>
              <div class="net-subtitle">${LINK_NAMES[group.link.kind] || group.link.kind} · ${t('ui.sensorsCount', { n: sCount })} · ${t('ui.networkHint')}</div>
            </div>
            <div class="spacer"></div>
            <button class="modal-close" type="button" @click="close" aria-label="${esc(t('ui.cancel'))}">✕</button>
          </div>
          <div class="net-stage">
            <svg class="net-svg" viewBox="0 0 ${W} ${H}" preserveAspectRatio="xMidYMid meet" @wheel="onWheel" @pointerdown="onDown">
              <g class="mm-scene">${linkSvg}${pkSvg}${nodeSvg}</g>
            </svg>
            <div class="net-zoom">
              <button type="button" @click="zoomIn" aria-label="zoom in">+</button>
              <button type="button" @click="zoomOut" aria-label="zoom out">−</button>
              <button type="button" @click="resetView" aria-label="reset">⟲</button>
            </div>
            ${this.inspector(group, sel ? pos[sel] : null)}
          </div>
        </div>
      </div>`;
  },
});
