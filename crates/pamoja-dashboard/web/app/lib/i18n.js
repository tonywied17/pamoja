// i18n.js - localization for the dashboard.
//
// The device serves a language-neutral snapshot; every label and all number, percent,
// and relative-time formatting is produced here from the active locale bundle plus the
// browser's own Intl engine (which carries CLDR), so non-Latin numerals and RTL need
// nothing shipped from the device. Bundles are loaded on demand and cached; the active
// locale lives in the store.

import { store } from '../store.js';

/** The locales shipped as seed bundles, in menu order. */
export const LOCALES = ['en', 'sw', 'ar', 'fr', 'pt', 'hi'];

const bundles = {};
let fallback;

/**
 * Loads the English fallback and the active locale bundle, then applies text direction.
 *
 * @returns {Promise<void>} resolves once the active locale is ready to render.
 */
export async function initI18n()
{
  fallback = bundles.en = (await import('../i18n/en.js')).default;
  const l = store.state.locale;
  if (!bundles[l]) bundles[l] = (await import('../i18n/' + l + '.js')).default;
  applyDir();
}

/**
 * Switches the active locale, loading its bundle on demand and updating direction.
 *
 * @param {string} l - the locale tag to switch to; ignored if not in {@link LOCALES}.
 * @returns {Promise<void>} resolves once the locale is active.
 */
export async function setLocale(l)
{
  if (!LOCALES.includes(l)) return;
  if (!bundles[l]) bundles[l] = (await import('../i18n/' + l + '.js')).default;
  store.dispatch('setLocale', l);
  applyDir();
}

/**
 * Returns the active locale bundle, falling back to English.
 *
 * @returns {object} the active message bundle.
 */
function active() { return bundles[store.state.locale] || fallback; }

/** Reflects the active bundle's language and text direction onto the document. */
function applyDir() { const b = active(); document.documentElement.lang = b.locale; document.documentElement.dir = b.dir; }

const pluralRules = {};
const pluralRulesFor = (locale) =>
  (pluralRules[locale] ??= new Intl.PluralRules(locale, { type: 'cardinal' }));

/**
 * Runtime helpers passed to function-valued messages (counts and plurals). A generated
 * bundle compiles a Fluent selector to `(a, h) => ...` and calls back into these.
 */
const helpers =
{
  num: (n) => nf(n),
  plural: (n) => pluralRulesFor(active().locale).select(n),
  sel: (n, variants, fallbackKey) =>
    variants[n] ?? variants[helpers.plural(n)] ?? variants[fallbackKey],
};

/**
 * Looks up a localized message by its stable key.
 *
 * @param {string} k - the message key, such as `"ui.status"`.
 * @param {object} [args] - arguments for a counted or plural message, such as `{ n: 3 }`.
 * @returns {string} the active-locale text, the English fallback, or the key itself.
 */
export const t = (k, args) =>
{
  const m = active().messages[k] ?? fallback.messages[k];
  if (m == null) return k;
  return typeof m === 'function' ? m(args || {}, helpers) : m;
};

/**
 * Formats a number in the active locale and numbering system.
 *
 * @param {number} v - the value to format.
 * @param {Intl.NumberFormatOptions} [o] - extra Intl number-format options.
 * @returns {string} the formatted number.
 */
export const nf = (v, o = {}) =>
  new Intl.NumberFormat(active().locale, { numberingSystem: active().numberingSystem, maximumFractionDigits: 1, ...o }).format(v);

/**
 * Formats a reading value with sensible precision (whole numbers above 100).
 *
 * @param {number} v - the value to format.
 * @returns {string} the formatted value.
 */
export const fmt = (v) => nf(v, { maximumFractionDigits: Math.abs(v) >= 100 ? 0 : 1 });

/**
 * Formats an elapsed duration as a localized relative time, such as "2 min ago".
 *
 * @param {number} seconds - how many seconds ago the event happened.
 * @returns {string} the localized relative-time phrase.
 */
export function ago(seconds)
{
  const r = new Intl.RelativeTimeFormat(active().locale, { numeric: 'auto' });
  if (seconds < 60) return r.format(-seconds, 'second');
  if (seconds < 3600) return r.format(-Math.round(seconds / 60), 'minute');
  return r.format(-Math.round(seconds / 3600), 'hour');
}

/**
 * Returns a locale's own endonym (its name in its own language).
 *
 * @param {string} l - the locale tag, such as `"sw"`.
 * @returns {string} the locale's display name, or the tag if unavailable.
 */
export const localeName = (l) => { try { return new Intl.DisplayNames([l], { type: 'language' }).of(l); } catch { return l; } };
