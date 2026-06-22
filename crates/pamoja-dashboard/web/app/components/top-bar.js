// top-bar.js - brand + global controls (language, theme, dev scenario).
//
// Sits outside the router outlet so it persists across navigation. Reads preferences
// from the app store and re-renders when they change. The dropdowns follow zQuery's
// own pattern - boolean state, z-show, @click.outside - so they behave like the
// framework expects, with no custom z-index or document listeners.

import { store } from '../store.js';
import { t, nf, LOCALES, setLocale, localeName } from '../i18n.js';
import { SCENARIOS } from '../feed.js';
import { currentFleet } from '../edits.js';
import { open } from '../nav.js';
import { openNetworkOverlay } from './network-view.js';
import { problems } from './alarm-bar.js';
import { esc } from '../viz.js';

const CHEVRON = '<svg class="chev" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round"><path d="M6 9l6 6 6-6"/></svg>';

$.component('top-bar', {
  state: { localeOpen: false, scenarioOpen: false },

  mounted()
  {
    this._un = store.subscribe(() => this.setState({}));
    this._eff = $.effect(() => { currentFleet(); this.setState({}); });
  },
  destroyed() { if (this._un) this._un(); if (typeof this._eff === 'function') this._eff(); },

  toggleLocale() { this.state.scenarioOpen = false; this.state.localeOpen = !this.state.localeOpen; },
  toggleScenario() { this.state.localeOpen = false; this.state.scenarioOpen = !this.state.scenarioOpen; },
  closeLocale() { this.state.localeOpen = false; },
  closeScenario() { this.state.scenarioOpen = false; },

  async pickLocale(l) { this.state.localeOpen = false; await setLocale(l); },
  pickScenario(s) { this.state.scenarioOpen = false; store.dispatch('setScenario', s); },
  openNetwork() { openNetworkOverlay(); },
  openAlarms() { open(() => store.dispatch('openAlarms'), () => store.dispatch('closeAlarms')); },
  toggleTheme()
  {
    const next = store.state.theme === 'night' ? 'day' : 'night';
    store.dispatch('setTheme', next);
    document.documentElement.dataset.theme = next;
  },

  render()
  {
    const s = this.state;
    const alarmCount = problems(currentFleet()).length;
    const locales = LOCALES.map((l) => `<li class="dd-option" aria-selected="${l === store.state.locale}" @click="pickLocale('${l}')">${esc(localeName(l))}</li>`).join('');
    const scenarios = SCENARIOS.map((sc) => `<li class="dd-option" aria-selected="${sc === store.state.scenario}" @click="pickScenario('${sc}')">${esc(t('scenario.' + sc))}</li>`).join('');
    return `
      <header class="topbar">
        <a class="brand" z-link="/" aria-label="pamoja">
          <span class="brand-mark"></span>
          <span class="brand-text"><span class="brand-word">pamoja</span><span class="brand-sub">${t('ui.subtitle')}</span></span>
        </a>
        <div class="spacer"></div>
        <div class="controls">
          <button class="seg bell ${alarmCount ? 'has' : ''}" type="button" @click="openAlarms" aria-label="${esc(t('ui.alarmsTitle'))}">⚠${alarmCount ? `<span class="bell-count">${nf(alarmCount)}</span>` : ''}</button>
          <button class="seg" type="button" @click="openNetwork" aria-label="${esc(t('ui.network'))}">⬡ ${t('ui.network')}</button>
          <div class="dd ${s.localeOpen ? 'open' : ''}" @click.outside="closeLocale">
            <button class="dd-button" type="button" @click="toggleLocale" aria-label="${esc(t('ui.language'))}">
              <span>${esc(localeName(store.state.locale))}</span>${CHEVRON}
            </button>
            <ul class="dd-menu" role="listbox" z-show="localeOpen">${locales}</ul>
          </div>
          <button class="seg" type="button" @click="toggleTheme" aria-label="${esc(t('ui.theme'))}">${store.state.theme === 'night' ? '☀ ' + t('ui.day') : '☾ ' + t('ui.night')}</button>
          <div class="dd ${s.scenarioOpen ? 'open' : ''}" @click.outside="closeScenario">
            <button class="dd-button" type="button" @click="toggleScenario" aria-label="${esc(t('ui.scenario'))}">
              <span>${esc(t('scenario.' + store.state.scenario))}</span>${CHEVRON}
            </button>
            <ul class="dd-menu" role="listbox" z-show="scenarioOpen">${scenarios}</ul>
          </div>
        </div>
      </header>`;
  },
});
