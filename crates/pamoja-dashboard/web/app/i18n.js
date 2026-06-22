// i18n.js - localization for the dashboard.
//
// The device serves a language-neutral snapshot; every label and all number, percent,
// and relative-time formatting is produced here from the active locale bundle plus the
// browser's own Intl engine (which carries CLDR), so non-Latin numerals and RTL need
// nothing shipped from the device. Bundles are loaded on demand and cached; the active
// locale lives in the store.

import { store } from './store.js';

export const LOCALES = ['en', 'sw', 'ar', 'fr', 'pt', 'hi'];

const bundles = {};
let fallback;

/** Loads the English fallback and the active locale, then applies text direction. */
export async function initI18n()
{
  fallback = bundles.en = (await import('./i18n/en.js')).default;
  const l = store.state.locale;
  if (!bundles[l]) bundles[l] = (await import('./i18n/' + l + '.js')).default;
  applyDir();
}

/** Switches the active locale, loading its bundle if needed. */
export async function setLocale(l)
{
  if (!LOCALES.includes(l)) return;
  if (!bundles[l]) bundles[l] = (await import('./i18n/' + l + '.js')).default;
  store.dispatch('setLocale', l);
  applyDir();
}

function active() { return bundles[store.state.locale] || fallback; }
function applyDir() { const b = active(); document.documentElement.lang = b.locale; document.documentElement.dir = b.dir; }

export const t = (k) => active().messages[k] ?? fallback.messages[k] ?? k;

export const nf = (v, o = {}) =>
  new Intl.NumberFormat(active().locale, { numberingSystem: active().numberingSystem, maximumFractionDigits: 1, ...o }).format(v);

export const fmt = (v) => nf(v, { maximumFractionDigits: Math.abs(v) >= 100 ? 0 : 1 });

export function ago(seconds)
{
  const r = new Intl.RelativeTimeFormat(active().locale, { numeric: 'auto' });
  if (seconds < 60) return r.format(-seconds, 'second');
  if (seconds < 3600) return r.format(-Math.round(seconds / 60), 'minute');
  return r.format(-Math.round(seconds / 3600), 'hour');
}

export const localeName = (l) => { try { return new Intl.DisplayNames([l], { type: 'language' }).of(l); } catch { return l; } };
