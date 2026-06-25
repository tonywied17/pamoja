// i18n.js - localization for the dashboard.
//
// One JSON file per locale is the single source of translations: the device serves a
// language-neutral snapshot, and this loads the active locale's JSON (fetched and cached)
// and renders every label and all number, percent, plural, and relative-time formatting
// from it plus the browser's Intl engine, which already carries CLDR. So non-Latin
// numerals, plural forms, and right-to-left need nothing generated and nothing shipped
// from the device beyond the messages themselves. A message is either a string or, for a
// counted message, a plural map keyed by CLDR category ({ one, other, ... }); `{n}` and
// other `{name}` placeholders are filled at render time.

import { store } from '../store.js';

/** The locales shipped as JSON bundles, in menu order. */
export const LOCALES = ['en', 'sw', 'ar', 'fr', 'pt', 'hi'];

const bundles = {};
let fallback;

// Labels a device-served catalog supplies for its custom elements, so a profile can name a
// sensor the shipped bundles never carried. Keyed by locale (`'en'`, `'sw'`, ...) plus `'*'`
// for a single non-localized fallback. Shipped bundles still win; these only fill gaps.
const labelExtra = {};

/**
 * Loads and caches a locale's JSON bundle, resolved relative to this module so it works
 * on a device and under any base path on a static host.
 *
 * @param {string} l - the locale tag.
 * @returns {Promise<object>} the loaded bundle.
 */
async function load(l)
{
  if (!bundles[l])
  {
    const res = await fetch(new URL('../i18n/' + l + '.json', import.meta.url), { cache: 'no-store' });
    bundles[l] = await res.json();
  }
  return bundles[l];
}

/**
 * Loads the English fallback and the active locale bundle, then applies text direction.
 *
 * @returns {Promise<void>} resolves once the active locale is ready to render.
 */
export async function initI18n()
{
  fallback = bundles.en = await load('en');
  await load(store.state.locale);
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
  await load(l);
  store.dispatch('setLocale', l);
  applyDir();
}

/**
 * Registers labels from a device-served catalog so a custom element's `label.<key>`
 * resolves through {@link t}. A preset's per-locale `labels` register under each locale; a
 * single `label` registers as the non-localized fallback. Shipped bundles still take
 * precedence, so this only fills keys the bundles do not define.
 *
 * @param {Array<{key: string, label?: string, labels?: Object<string,string>}>} presets - the catalog presets.
 * @returns {void}
 */
export function registerLabels(presets)
{
  for (const p of presets || [])
  {
    if (!p || !p.key) continue;
    const mk = 'label.' + p.key;
    if (p.labels) for (const [loc, text] of Object.entries(p.labels)) (labelExtra[loc] ??= {})[mk] = text;
    if (p.label) (labelExtra['*'] ??= {})[mk] = p.label;
  }
}

/**
 * Registers localized text for stable codes a device-served catalog supplies (its custom
 * discrete states and events), so `t('state.x')` / `t('event.y')` resolve to words instead
 * of the raw code. A value may be one string (shown in every locale) or a per-locale object.
 * Shipped bundles still win.
 *
 * @param {Object<string, (string|Object<string,string>)>} [messages] - code -> text or per-locale map.
 * @returns {void}
 */
export function registerMessages(messages)
{
  for (const [key, val] of Object.entries(messages || {}))
  {
    if (val == null) continue;
    if (typeof val === 'string') (labelExtra['*'] ??= {})[key] = val;
    else for (const [loc, text] of Object.entries(val)) (labelExtra[loc] ??= {})[key] = text;
  }
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
const plural = (locale, n) =>
  (pluralRules[locale] ??= new Intl.PluralRules(locale, { type: 'cardinal' })).select(n);

/**
 * Fills `{name}` placeholders in a message, formatting numeric arguments in the locale.
 *
 * @param {string} text - the message text.
 * @param {object} args - the named arguments.
 * @returns {string} the filled message.
 */
function fill(text, args)
{
  return text.replace(/\{(\w+)\}/g, (_, name) =>
  {
    const v = args[name];
    if (v == null) return '';
    return typeof v === 'number' ? nf(v) : String(v);
  });
}

/**
 * Looks up a localized message by its stable key.
 *
 * @param {string} k - the message key, such as `"ui.status"`.
 * @param {object} [args] - arguments for a counted or interpolated message, such as `{ n: 3 }`.
 * @returns {string} the active-locale text, the English fallback, or the key itself.
 */
export const t = (k, args = {}) =>
{
  const b = active();
  let m = b.messages[k];
  if (m == null) m = fallback.messages[k];
  // A device-served catalog fills labels the bundles never carried (a custom element):
  // the active locale's label, then English, then the non-localized fallback.
  if (m == null) m = labelExtra[b.locale]?.[k];
  if (m == null) m = labelExtra.en?.[k];
  if (m == null) m = labelExtra['*']?.[k];
  // A unit we do not ship a symbol for (a profile's custom unit, such as `ntu`) degrades to
  // its bare, language-neutral token rather than the raw `unit.ntu` key.
  if (m == null && k.startsWith('unit.')) return k.slice(5);
  if (m == null) return k;
  if (typeof m === 'object')
  {
    // A plural map: choose the CLDR category for the count, falling back to `other`.
    m = m[plural(b.locale, args.n)] ?? m.other ?? Object.values(m)[0];
  }
  return m.includes('{') ? fill(m, args) : m;
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
