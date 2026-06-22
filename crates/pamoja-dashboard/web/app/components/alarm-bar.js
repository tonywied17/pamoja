// alarm-bar.js - a slide-in panel listing everything that needs attention.
//
// Toggled by the bell in the top bar. Lists every warning/alarm sensor across the
// fleet, newest concern first, grouped by org and group, each row jumping straight to
// that sensor's detail modal - so you triage alarms instead of hunting for them.

import { store } from '../store.js';
import { currentFleet } from '../edits.js';
import { open, back } from '../nav.js';
import { t, nf, fmt } from '../i18n.js';
import { esc } from '../viz.js';

export function problems(f)
{
  const out = [];
  if (!f) return out;
  for (const o of f.orgs) for (const g of o.groups)
  {
    for (const s of g.sensors)
    {
      if (s.reading.status !== 'ok') out.push({ org: o, group: g, sensor: s });
    }
    if (!g.link.online) out.push({ org: o, group: g, link: true });
  }
  return out.sort((a, b) => (b.sensor?.reading.status === 'alarm' ? 1 : 0) - (a.sensor?.reading.status === 'alarm' ? 1 : 0));
}

$.component('alarm-bar', {
  state: { tick: 0 },

  mounted()
  {
    this._un = store.subscribe(() => this.setState({}));
    this._eff = $.effect(() => { currentFleet(); this.setState({}); });
  },
  destroyed()
  {
    if (this._un) this._un();
    if (typeof this._eff === 'function') this._eff();
  },

  close() { back(); },
  onOverlay(e) { if (e.target.classList.contains('drawer-scrim')) back(); },

  onGoSensor(e)
  {
    const el = e.target.closest('[data-sid]'); if (!el) return;
    open(() => store.dispatch('selectSensor', el.dataset.sid), () => store.dispatch('closeSensor'));
  },
  onGoGroup(e)
  {
    const el = e.target.closest('[data-gid]'); if (!el) return;
    const gid = el.dataset.gid;
    open(() => store.dispatch('setGroupView', gid), () => store.dispatch('clearGroupView'));
  },

  render()
  {
    if (!store.state.alarms) return '<div hidden></div>';
    const list = problems(currentFleet());
    const groupBtn = (gid) => `<button class="al-btn" data-gid="${gid}" @click="onGoGroup">${t('ui.group')}</button>`;
    const rows = list.length ? list.map((p) =>
    {
      if (p.link)
      {
        return `<div class="al-row" data-level="error">
          <span class="al-dot" data-level="error"></span>
          <span class="al-text"><span class="al-name">${t('event.link.lost')}</span><span class="al-meta">${esc(p.org.name)} · ${esc(p.group.name)}</span></span>
          <span class="al-actions">${groupBtn(p.group.id)}</span></div>`;
      }
      const s = p.sensor, lvl = s.reading.status === 'alarm' ? 'error' : 'warn';
      return `<div class="al-row" data-level="${lvl}">
        <span class="al-dot" data-level="${lvl}"></span>
        <span class="al-text"><span class="al-name">${esc(t('label.' + s.reading.key))} · ${fmt(s.reading.value)}${t('unit.' + s.reading.unit)}</span>
          <span class="al-meta">${esc(p.group.name)} · ${esc(p.org.name)}</span></span>
        <span class="al-actions">
          <button class="al-btn" data-sid="${p.group.id}/${s.id}" @click="onGoSensor">${t('ui.sensorId')}</button>
          ${groupBtn(p.group.id)}
        </span></div>`;
    }).join('') : `<div class="al-empty">${t('ui.allClear')}</div>`;

    return `
      <div class="drawer-scrim" @click="onOverlay">
        <aside class="drawer" role="dialog" aria-modal="true">
          <div class="drawer-head">
            <div class="drawer-title">${t('ui.alarmsTitle')} ${list.length ? `<span class="al-count">${nf(list.length)}</span>` : ''}</div>
            <button class="modal-close" type="button" @click="close" aria-label="${esc(t('ui.cancel'))}">✕</button>
          </div>
          <div class="drawer-body">${rows}</div>
        </aside>
      </div>`;
  },
});
