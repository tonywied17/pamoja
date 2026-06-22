// group-modal.js - the advanced group view.
//
// A full-screen modal showing a whole group's sensors at once, larger and cleaner than
// the dashboard tiles. A labelled tab strip switches between groups across the fleet; a
// sensor card opens that sensor's detail modal on top. Big groups paginate their sensor
// grid (a numbered pager) so the cards stay a comfortable size instead of one long
// scroll. Opened from the dashboard group card, the network inspect panel, and the alarm
// drawer's "Group" button. Driven by store.group; built from currentFleet().

import { store } from '../store.js';
import { currentFleet } from '../edits.js';
import { open, back } from '../nav.js';
import { t, nf, fmt } from '../i18n.js';
import { conn, tileViz, trendArrow, isDiscrete, vizFor, esc } from '../viz.js';
import { openMeshOverlay } from './mesh-modal.js';

// Sensors shown per page in the group view before it paginates.
const PAGE = 6;

function flat(f)
{
  const out = [];
  for (const o of f.orgs) for (const g of o.groups) out.push({ org: o, group: g });
  return out;
}

$.component('group-modal', {
  state: { page: 0, pickerOpen: false },

  mounted()
  {
    this._un = store.subscribe(() => this.setState({}));
    this._eff = $.effect(() => { currentFleet(); this.setState({}); });
  },
  destroyed() { if (this._un) this._un(); if (typeof this._eff === 'function') this._eff(); },

  close() { back(); },
  onOverlay(e) { if (e.target.classList.contains('modal-overlay')) back(); },
  swap(e) { const el = e.target.closest('[data-gid]'); if (el) { this.state.page = 0; this.state.pickerOpen = false; store.dispatch('setGroupView', el.dataset.gid); } },
  togglePicker() { this.setState({ pickerOpen: !this.state.pickerOpen }); },
  closePicker() { if (this.state.pickerOpen) this.setState({ pickerOpen: false }); },
  setPage(e) { const el = e.target.closest('[data-page]'); if (el) this.setState({ page: Number(el.dataset.page) }); },
  onSensor(e)
  {
    const el = e.target.closest('[data-sid]'); if (!el) return;
    const sid = el.dataset.sid;
    const s = this.sensorBySid(sid);
    if (s && vizFor(s.reading.key, s.reading.unit) === 'mesh') { openMeshOverlay(sid); return; }
    open(() => store.dispatch('selectSensor', sid), () => store.dispatch('closeSensor'));
  },
  sensorBySid(sid)
  {
    const f = currentFleet(); if (!f) return null;
    const [gid, sd] = sid.split('/');
    for (const o of f.orgs) for (const g of o.groups) if (g.id === gid) return (g.sensors.find((x) => x.id === sd) || null);
    return null;
  },

  card(group, s)
  {
    const r = s.reading;
    const vk = vizFor(r.key, r.unit);
    const span = vk === 'chain' || vk === 'wave' || vk === 'mesh' ? ' span' : '';
    const head = `<div class="gv-top"><span class="gv-label">${esc(t('label.' + r.key))}</span><span class="pill" data-status="${r.status}">${t('status.' + r.status)}</span></div>`;
    if (isDiscrete(r))
    {
      const nodes = vk === 'mesh' ? group.sensors.filter((x) => vizFor(x.reading.key, x.reading.unit) !== 'mesh').length : undefined;
      return `<article class="gv-card${span}" data-status="${r.status}" data-sid="${group.id}/${s.id}" @click="onSensor" tabindex="0" role="button">
          ${head}<div class="gv-viz gv-viz-disc">${tileViz(s, true, nodes)}</div>
        </article>`;
    }
    const h = s.history || [];
    const min = h.length ? Math.min(...h) : r.value;
    const max = h.length ? Math.max(...h) : r.value;
    return `<article class="gv-card" data-status="${r.status}" data-sid="${group.id}/${s.id}" @click="onSensor" tabindex="0" role="button">
        ${head}
        <div class="gv-viz">${tileViz(s, true)}</div>
        <div class="gv-foot"><span class="gv-val">${fmt(r.value)}<span class="tile-unit">${t('unit.' + r.unit)}</span></span>${r.trend ? `<span class="trend" data-dir="${r.trend}">${trendArrow(r.trend)}</span>` : ''}</div>
        <div class="gv-stats">${t('ui.min')} ${fmt(min)} · ${t('ui.max')} ${fmt(max)} ${t('unit.' + r.unit)}</div>
      </article>`;
  },

  render()
  {
    const id = store.state.group;
    const f = currentFleet();
    if (!id || !f) return '<div hidden></div>';
    const list = flat(f);
    const idx = list.findIndex((x) => x.group.id === id);
    if (idx < 0) return '<div hidden></div>';
    const { org, group } = list[idx];
    const n = list.length;

    if (this._lastId !== id) { this._lastId = id; this.state.page = 0; }

    const sensors = group.sensors;
    const pages = Math.max(1, Math.ceil(sensors.length / PAGE));
    const page = Math.min(Math.max(this.state.page || 0, 0), pages - 1);
    const start = page * PAGE;
    const shown = sensors.slice(start, start + PAGE);

    const cards = shown.map((s) => this.card(group, s)).join('') || `<div class="empty">${t('ui.noReadings')}</div>`;

    const pager = pages > 1
      ? `<div class="gv-pages">
          ${Array.from({ length: pages }, (_, i) => `<button class="gv-page ${i === page ? 'on' : ''}" type="button" data-page="${i}" @click="setPage">${nf(i + 1)}</button>`).join('')}
        </div>`
      : '';

    return `
      <div class="modal-overlay" @click="onOverlay">
        <div class="modal gv-modal" data-status="${group.status}" role="dialog" aria-modal="true">
          <div class="gv-head">
            <div class="gv-head-main">
              <div class="modal-title">${esc(group.name)}</div>
              <div class="modal-sub">${esc(org.name)} · ${nf(idx + 1)} / ${nf(n)} · ${nf(sensors.length)} ${t('ui.sensors')}</div>
            </div>
            <div class="gv-head-side">
              ${conn(group.link)}
              <button class="modal-close" type="button" @click="close" aria-label="Close">✕</button>
            </div>
          </div>
          <div class="gv-main">
            <div class="gv-nav ${this.state.pickerOpen ? 'open' : ''}" @click.outside="closePicker">
              <button class="gv-picker" type="button" @click="togglePicker" aria-expanded="${this.state.pickerOpen ? 'true' : 'false'}">
                <span class="gv-tab-dot" data-status="${group.status}"></span>
                <span class="gv-picker-txt"><span class="gv-picker-org">${esc(org.name)}</span><span class="gv-picker-cur">${esc(group.name)}</span></span>
                <svg class="gv-picker-chev" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round"><path d="M6 9l6 6 6-6"/></svg>
              </button>
              <nav class="gv-rail" aria-label="${esc(t('ui.groups'))}">
                ${f.orgs.map((o) => `<div class="gv-railorg ${o.groups.some((g) => g.id === id) ? 'active' : ''}">
                  <div class="gv-railhead">${esc(o.name)}</div>
                  ${o.groups.map((g) => `<button class="gv-railitem ${g.id === id ? 'on' : ''}" type="button" data-gid="${g.id}" @click="swap"><span class="gv-tab-dot" data-status="${g.status}"></span><span class="gv-railname">${esc(g.name)}</span></button>`).join('')}
                </div>`).join('')}
              </nav>
            </div>
            <div class="gv-content">
              <div class="gv-grid">${cards}</div>
              ${pager}
            </div>
          </div>
        </div>
      </div>`;
  },
});
