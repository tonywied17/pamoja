// dashboard-page.js - the fleet view.
//
// Reads the live fleet (with the user's client-side edits applied) and the user's
// preferences, and renders the status banner, organization tabs, and the selected
// org's groups. Each group shows its link type and signal strength; each sensor gets
// the visualization that suits it and opens a detail modal on click. In "Manage" mode
// it also shows add/remove affordances for groups and sensors.

import { store } from '../store.js';
import { open } from '../nav.js';
import { t, nf, fmt } from '../i18n.js';
import { currentFleet } from '../edits.js';
import { conn, tileViz, bannerRing, trendArrow, isDiscrete, vizFor, esc } from '../viz.js';
import { openMeshOverlay } from './mesh-modal.js';

const ICON_EDIT = '<svg class="ic" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M12 20h9"/><path d="M16.5 3.5a2.1 2.1 0 0 1 3 3L7 19.5l-4 1 1-4z"/></svg>';
const ICON_DONE = '<svg class="ic" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.4" stroke-linecap="round" stroke-linejoin="round"><path d="M20 6 9 17l-5-5"/></svg>';

function meshPeerCount(g)
{
  return g.sensors.filter((x) => vizFor(x.reading.key, x.reading.unit) !== 'mesh').length;
}

/** Worst sensor status across an org, so its dot can carry the org's health. */
function orgStatus(o)
{
  let worst = 'ok';
  for (const g of o.groups) for (const s of g.sensors)
  {
    if (s.reading.status === 'alarm') return 'alarm';
    if (s.reading.status === 'warn') worst = 'warn';
  }
  return worst;
}

function isMeshSensor(sid)
{
  const f = currentFleet();
  if (!f) return false;
  const [gid, sd] = sid.split('/');
  for (const o of f.orgs) for (const g of o.groups) if (g.id === gid)
  {
    const s = g.sensors.find((x) => x.id === sd);
    return !!s && vizFor(s.reading.key, s.reading.unit) === 'mesh';
  }
  return false;
}

$.component('dashboard-page', {
  state: { tick: 0, orgOpen: false },

  mounted()
  {
    this._place = {};
    this._stop = $.effect(() => { currentFleet(); if (!this._drag) this.setState({}); });
    this._un = store.subscribe(() => { if (!this._drag) this.setState({}); });
    this._ro = new ResizeObserver(() => this.scheduleLayout());
    this._onResize = () => this.scheduleLayout();
    window.addEventListener('resize', this._onResize);
    this.bindDrag();
  },
  updated()
  {
    const grid = this._el && this._el.querySelector('.groups');
    if (!grid) return;
    if (!grid._obs) { grid._obs = true; this._ro.observe(grid); }
    grid.querySelectorAll('.gcard').forEach((c) => { if (!c._obs) { c._obs = true; this._ro.observe(c); } });
  },
  destroyed()
  {
    if (typeof this._stop === 'function') this._stop();
    if (this._un) this._un();
    if (this._ro) this._ro.disconnect();
    if (this._onResize) window.removeEventListener('resize', this._onResize);
    if (this._raf) cancelAnimationFrame(this._raf);
  },

  scheduleLayout() { if (!this._raf) this._raf = requestAnimationFrame(() => this.layout()); },

  /** Masonry: place each card at column i%cols, stacked at that column's running bottom (row-major, no gaps). Placement is cached so render() re-emits it inline, since the morph strips JS-set styles. One column drops to natural stacking (see .groups.mono). */
  layout()
  {
    this._raf = 0;
    const grid = this._el && this._el.querySelector('.groups');
    if (!grid) return;
    const cards = [...grid.children].filter((c) => c.classList.contains('gcard'));
    if (!cards.length) return;
    const colGap = parseFloat(getComputedStyle(grid).columnGap) || 18;
    // Derive the column count from the container width and the CSS min track (372px), not
    // from the live grid - our explicit placements would otherwise pin the desktop column
    // count on a phone and squish the cards instead of collapsing.
    const cols = Math.max(1, Math.floor((grid.clientWidth + colGap) / (372 + colGap)));
    // _mono is re-emitted by render() so the morph keeps the class between ticks.
    this._mono = cols <= 1;
    grid.classList.toggle('mono', this._mono);
    if (this._mono)
    {
      this._place = {};
      cards.forEach((c) => { c.style.gridColumn = ''; c.style.gridRow = ''; });
      return;
    }
    const gap = 18;
    const colBottom = new Array(cols).fill(1);
    cards.forEach((c, i) =>
    {
      const col = i % cols;
      const h = Math.max(1, Math.ceil(c.offsetHeight));
      const row = colBottom[col];
      c.style.gridColumn = String(col + 1);
      c.style.gridRow = row + ' / span ' + h;
      colBottom[col] = row + h + gap;
      this._place[c.dataset.gid || '__add'] = { c: col + 1, r: row, s: h };
    });
  },

  bindDrag()
  {
    const root = this._el;
    root.addEventListener('dragstart', (e) =>
    {
      const el = e.target.closest('[draggable="true"]');
      if (!el) return;
      const group = el.classList.contains('gcard');
      this._drag = { el, container: el.parentNode, sel: group ? '.gcard[draggable]' : '.stile[draggable]', group };
      el.classList.add('dragging');
      e.dataTransfer.effectAllowed = 'move';
      try { e.dataTransfer.setData('text/plain', ''); } catch (err) { /* Safari */ }
    });
    root.addEventListener('dragover', (e) =>
    {
      if (!this._drag) return;
      e.preventDefault();
      const over = e.target.closest(this._drag.sel);
      if (!over || over === this._drag.el || over.parentNode !== this._drag.container) return;
      const rect = over.getBoundingClientRect();
      const before = (e.clientX - rect.left) < rect.width / 2;
      const ref = before ? over : over.nextSibling;
      if (ref !== this._drag.el) { this._drag.container.insertBefore(this._drag.el, ref); if (this._drag.group) this.scheduleLayout(); }
    });
    root.addEventListener('drop', (e) => { if (this._drag) e.preventDefault(); });
    root.addEventListener('dragend', () => this.endDrag());
  },

  endDrag()
  {
    if (!this._drag) return;
    const { el, container, group } = this._drag;
    el.classList.remove('dragging');
    this._drag = null;
    if (group)
    {
      const ids = [...container.querySelectorAll(':scope > .gcard[data-gid]')].map((c) => c.dataset.gid);
      const org = this.selectedOrg(currentFleet());
      if (org) store.dispatch('reorderGroups', { orgId: org.id, ids });
    } else
    {
      const card = container.closest('.gcard');
      const gid = card && card.dataset.gid;
      if (gid)
      {
        const ids = [...container.querySelectorAll(':scope > .stile[data-sid]')].map((c) => c.dataset.sid.split('/')[1]);
        store.dispatch('reorderSensors', { gid, ids });
      }
    }
    this.setState({});
    this.scheduleLayout();
  },

  onSensor(e)
  {
    if (e.target.closest('.tile-rm')) return;
    const el = e.target.closest('[data-sid]'); if (!el) return;
    const sid = el.dataset.sid;
    if (isMeshSensor(sid)) { openMeshOverlay(sid); return; }
    open(() => store.dispatch('selectSensor', sid), () => store.dispatch('closeSensor'));
  },
  onRemoveSensor(e) { e.stopPropagation(); const el = e.target.closest('[data-key]'); if (el) store.dispatch('removeSensor', el.dataset.key); },
  onRemoveGroup(e) { const el = e.target.closest('[data-gid]'); if (el) store.dispatch('removeGroup', el.dataset.gid); },
  onOpenGroup(e) { const el = e.target.closest('[data-gid]'); if (el) { const gid = el.dataset.gid; open(() => store.dispatch('setGroupView', gid), () => store.dispatch('clearGroupView')); } },
  onAddSensor(e) { const el = e.target.closest('[data-gid]'); if (el) { const gid = el.dataset.gid; open(() => store.dispatch('openCreate', { mode: 'sensor', groupId: gid }), () => store.dispatch('closeCreate')); } },
  onAddGroup(e) { const el = e.target.closest('[data-oid]'); if (el) { const oid = el.dataset.oid; open(() => store.dispatch('openCreate', { mode: 'group', orgId: oid }), () => store.dispatch('closeCreate')); } },

  selectedOrg(f)
  {
    const orgs = f.orgs || [];
    const id = this.props.$params && this.props.$params.id;
    return orgs.find((o) => o.id === id) || orgs[0] || null;
  },

  render()
  {
    const f = currentFleet();
    if (!f)
    {
      return `<div class="shell"><section class="banner"><div class="banner-text"><span class="banner-eyebrow">${t('ui.status')}</span><h1 class="banner-title">${t('ui.connecting')}</h1></div></section></div>`;
    }
    return `<div class="shell">${this.banner(f)}${this.orgtabs(f)}${this.groups(f)}</div>`;
  },

  banner(f)
  {
    let groups = 0, sensors = 0, alarms = 0, warns = 0;
    for (const o of f.orgs) for (const g of o.groups)
    {
      groups++;
      for (const s of g.sensors) { sensors++; if (s.reading.status === 'alarm') alarms++; else if (s.reading.status === 'warn') warns++; }
    }
    return `
      <section class="banner" data-status="${f.status}">
        <div class="banner-ring">${bannerRing(f.status)}</div>
        <div class="banner-text">
          <span class="banner-eyebrow">${t('ui.status')}</span>
          <h1 class="banner-title">${t('ui.hero.' + f.status)}</h1>
          <span class="banner-sub">${nf(f.orgs.length)} ${t('ui.orgs')} · ${nf(groups)} ${t('ui.groups')} · ${nf(sensors)} ${t('ui.sensors')}</span>
        </div>
        <div class="banner-stats">
          <div class="bstat"><b>${nf(sensors)}</b><span>${t('ui.sensors')}</span></div>
          <div class="bstat" data-tone="${warns ? 'warn' : ''}"><b>${nf(warns)}</b><span>${t('status.warn')}</span></div>
          <div class="bstat" data-tone="${alarms ? 'alarm' : ''}"><b>${nf(alarms)}</b><span>${t('status.alarm')}</span></div>
        </div>
      </section>`;
  },

  orgtabs(f)
  {
    const sel = this.selectedOrg(f) || f.orgs[0];
    const open = this.state.orgOpen;
    const menu = f.orgs.map((o) => `<a class="orgsel-item ${sel && o.id === sel.id ? 'on' : ''}" z-link="/org/${o.id}" @click="closeOrg">
        <span class="dotc" data-status="${orgStatus(o)}"></span><span class="orgsel-iname">${esc(o.name)}</span><span class="count">${nf(o.groups.length)}</span></a>`).join('');
    return `<div class="orgbar">
      <div class="orgsel ${open ? 'open' : ''}" @click.outside="closeOrg">
        <button class="orgsel-btn" type="button" @click="toggleOrg" aria-expanded="${open ? 'true' : 'false'}">
          <span class="dotc" data-status="${sel ? orgStatus(sel) : 'ok'}"></span>
          <span class="orgsel-cur">${esc(sel ? sel.name : t('ui.orgs'))}</span>
          <span class="count">${sel ? nf(sel.groups.length) : ''}</span>
          <svg class="orgsel-chev" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round"><path d="M6 9l6 6 6-6"/></svg>
        </button>
        <div class="orgsel-menu">${menu}</div>
      </div>
      <button class="manage-btn ${store.state.editing ? 'on' : ''}" type="button" @click="onManage">${store.state.editing ? ICON_DONE : ICON_EDIT}<span>${store.state.editing ? t('ui.done') : t('ui.manage')}</span></button>
    </div>`;
  },

  toggleOrg() { this.setState({ orgOpen: !this.state.orgOpen }); },
  closeOrg() { if (this.state.orgOpen) this.setState({ orgOpen: false }); },
  onManage() { store.dispatch('toggleEditing'); },

  groups(f)
  {
    const org = this.selectedOrg(f);
    if (!org) return '';
    const editing = store.state.editing;
    const ap = (this._place || {}).__add;
    const astyle = ap ? `grid-column:${ap.c};grid-row:${ap.r} / span ${ap.s}` : 'grid-row:auto / span 170';
    const add = editing ? `<button class="gcard add-card" data-oid="${org.id}" z-key="__add" @click="onAddGroup" style="${astyle}"><span class="add-plus">+</span> ${t('ui.addGroup')}</button>` : '';
    return `<div class="groups${this._mono ? ' mono' : ''}">${org.groups.map((g) => this.groupCard(g, editing)).join('')}${add}</div>`;
  },

  groupCard(g, editing)
  {
    const rm = editing ? `<button class="icon-btn danger" data-gid="${g.id}" @click="onRemoveGroup" aria-label="${esc(t('ui.remove'))}">✕</button>` : '';
    const addS = editing ? `<button class="stile add-tile" data-gid="${g.id}" z-key="__addtile" @click="onAddSensor"><span class="add-plus">+</span><span>${t('ui.addSensor')}</span></button>` : '';
    const p = this._place && this._place[g.id];
    const style = p ? `grid-column:${p.c};grid-row:${p.r} / span ${p.s}` : 'grid-row:auto / span 320';
    return `
      <article class="gcard" data-status="${g.status}" data-gid="${g.id}" z-key="${g.id}" style="${style}"${editing ? ' draggable="true"' : ''}>
        <div class="ghead">
          <div class="gtitle-wrap">
            <div class="gtitle">${esc(g.name)}</div>
            <div class="gmeta"><span class="gstatus"><span class="dotc"></span>${t('status.' + g.status)}</span></div>
          </div>
          <div class="ghead-conn">${conn(g.link)}</div>
          ${rm}
        </div>
        <div class="sensors">
          ${g.sensors.map((s) => this.sensorTile(g, s, editing)).join('')}
          ${addS}
        </div>
        <div class="gfoot">
          <button class="gexpand" type="button" data-gid="${g.id}" @click="onOpenGroup" aria-label="${esc(t('ui.group'))}" title="${esc(t('ui.group'))}">⤢</button>
        </div>
      </article>`;
  },

  sensorTile(g, s, editing)
  {
    const r = s.reading;
    const sid = g.id + '/' + s.id;
    const battery = s.battery != null ? `<span class="sbatt">${nf(s.battery, { style: 'percent', maximumFractionDigits: 0 })}</span>` : '';
    const rm = editing ? `<button class="tile-rm" data-key="${sid}" @click="onRemoveSensor" aria-label="${esc(t('ui.remove'))}">✕</button>` : '';
    const readout = isDiscrete(r)
      ? ''
      : `<div><span class="sval">${fmt(r.value)}</span><span class="sunit">${t('unit.' + r.unit)}</span>${r.trend ? `<span class="strend" data-dir="${r.trend}">${trendArrow(r.trend)}</span>` : ''}</div>`;
    const vk = vizFor(r.key, r.unit);
    const span = vk === 'spark' || vk === 'chain' || vk === 'wave' || vk === 'mesh' ? ' span' : '';
    const nodes = vk === 'mesh' ? meshPeerCount(g) : undefined;
    return `
      <article class="stile${span} ${store.state.selected === sid ? 'open' : ''}" data-status="${r.status}" data-sid="${sid}" z-key="${s.id}" @click="onSensor" tabindex="0" role="button"${editing ? ' draggable="true"' : ''}>
        ${rm}
        <div class="stop"><span class="slabel">${esc(t('label.' + r.key))}</span><span class="sdot"></span></div>
        ${battery}
        ${readout}
        ${tileViz(s, false, nodes)}
      </article>`;
  },
});
