// manage-modal.js - create a group or add a sensor (demo provisioning).
//
// Driven by store.create ({ mode:'group', orgId } or { mode:'sensor', groupId }). The
// edits are client-side only (see edits.js); a real build would post these to the
// device as authenticated provisioning commands.

import { store } from '../store.js';
import { back } from '../nav.js';
import { t } from '../i18n.js';
import { SENSOR_PRESETS, LINK_KINDS, makeGroup, makeSensor, currentFleet } from '../edits.js';
import { LINK_NAMES, esc } from '../viz.js';

$.component('manage-modal', {
  state: { name: '', linkKind: 'lora', sensorKind: 'temperature', value: '', last: null },

  mounted() { this._un = store.subscribe(() => this.sync()); },
  destroyed() { if (this._un) this._un(); },

  // Reset the form whenever a new create dialog opens.
  sync()
  {
    const c = store.state.create;
    const id = c ? c.mode + (c.orgId || c.groupId) : null;
    if (id !== this.state.last)
    {
      this.state.last = id;
      this.state.name = '';
      this.state.value = '';
      this.state.linkKind = 'lora';
      this.state.sensorKind = 'temperature';
    }
    this.setState({});
  },

  setLink(k) { this.state.linkKind = k; },
  setKind(k) { this.state.sensorKind = k; },

  // Mesh-only sensors (the mesh map) appear only when the target group is on a mesh link.
  presetsFor(groupId)
  {
    const f = currentFleet();
    let kind = null;
    if (f) for (const o of f.orgs) for (const g of o.groups) if (g.id === groupId) kind = g.link.kind;
    return SENSOR_PRESETS.filter((p) => !p.meshOnly || kind === 'mesh');
  },
  cancel() { back(); },
  onOverlay(e) { if (e.target.classList.contains('modal-overlay')) back(); },

  submit()
  {
    const c = store.state.create; if (!c) return;
    if (c.mode === 'group')
    {
      store.dispatch('addGroup', makeGroup(c.orgId, this.state.name.trim() || t('ui.newGroup'), this.state.linkKind));
    } else
    {
      const v = parseFloat(this.state.value);
      store.dispatch('addSensor', makeSensor(c.groupId, this.state.sensorKind, Number.isFinite(v) ? v : NaN));
    }
    back();
  },

  render()
  {
    const c = store.state.create;
    if (!c) return '<div hidden></div>';
    const s = this.state;
    const body = c.mode === 'group'
      ? `
        <label class="field"><span>${t('ui.name')}</span>
          <input class="field-input" type="text" z-model="name" placeholder="${esc(t('ui.newGroup'))}" /></label>
        <div class="field"><span>${t('ui.connection')}</span>
          <div class="chips">${LINK_KINDS.map((k) => `<button type="button" class="chip-opt ${s.linkKind === k ? 'on' : ''}" @click="setLink('${k}')">${esc(LINK_NAMES[k] || k)}</button>`).join('')}</div>
        </div>`
      : `
        <div class="field"><span>${t('ui.type')}</span>
          <div class="chips">${this.presetsFor(c.groupId).map((p) => `<button type="button" class="chip-opt ${s.sensorKind === p.id ? 'on' : ''}" @click="setKind('${p.id}')">${esc(t('label.' + p.key))}</button>`).join('')}</div>
        </div>
        <label class="field"><span>${t('ui.value')}</span>
          <input class="field-input" type="number" step="any" z-model="value" placeholder="${t('ui.auto')}" /></label>`;
    return `
      <div class="modal-overlay" @click="onOverlay">
        <div class="modal modal-form" role="dialog" aria-modal="true">
          <div class="modal-head">
            <div class="modal-title">${c.mode === 'group' ? t('ui.addGroup') : t('ui.addSensor')}</div>
            <button class="modal-close" type="button" @click="cancel" aria-label="${esc(t('ui.cancel'))}">✕</button>
          </div>
          <div class="form">${body}</div>
          <div class="form-actions">
            <button class="seg" type="button" @click="cancel">${t('ui.cancel')}</button>
            <button class="seg primary" type="button" @click="submit">${t('ui.create')}</button>
          </div>
        </div>
      </div>`;
  },
});
