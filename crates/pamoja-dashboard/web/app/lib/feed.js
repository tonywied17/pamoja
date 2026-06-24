// feed.js - the live fleet data stream.
//
// The device serves the fleet snapshot at GET /state and a server-sent-event stream at
// GET /events. This module keeps the latest snapshot in a signal so any component can
// react to it with $.effect, and reconnects when the dev scenario changes. The
// connection state is a second signal, used to show the offline indicator.
//
// On a plain static host (the GitHub Pages showcase) there is no device: GET /state does
// not answer, so we fall back to a bundled snapshot shipped beside the page
// (state.<scenario>.json, relative so it works under any base path). The same build runs
// live against a real node or the dev server and serverless on Pages, with no flag.
//
// A snapshot may carry an optional `catalog` manifest; when present it is folded into the
// layout catalog (see lib/catalog.js), so a device can supply its own sensor presets,
// link kinds, and site map without a separate request.

import { store } from '../store.js';
import { mergeCatalog } from './catalog.js';

/** The mock scenarios the dev server and static fallback expose, in menu order. */
export const SCENARIOS = ['normal', 'alarm', 'sensor-fault', 'low-battery', 'link-lost', 'cold-start'];

/** The latest fleet snapshot, or `null` before the first frame arrives. */
export const fleet = $.signal(null);

/** Whether the feed is currently connected; drives the offline indicator. */
export const connected = $.signal(true);

/**
 * Whether a real device is answering (vs the static-host snapshot fallback). When true the
 * device is the source of truth and provisioning goes through authenticated commands; when
 * false (the Pages showcase) provisioning stays a client-side localStorage overlay.
 */
export const live = $.signal(false);

let es;
let lastScenario;
let replay;

/**
 * Folds any catalog manifest a snapshot carries into the shared layout catalog, then
 * publishes the snapshot to the {@link fleet} signal.
 *
 * @param {object} snap - the decoded fleet snapshot.
 * @returns {void}
 */
function publish(snap)
{
  if (snap && snap.catalog) mergeCatalog(snap.catalog);
  fleet.value = snap;
}

/**
 * Opens the feed for the current scenario: probes the device, then either streams live
 * updates or replays the bundled snapshot on a static host.
 *
 * @returns {Promise<void>} resolves once the source has been chosen and started.
 */
async function open()
{
  if (es) { es.close(); es = null; }
  clearInterval(replay);
  lastScenario = store.state.scenario;
  const query = '?scenario=' + encodeURIComponent(store.state.scenario);

  // Probe the device endpoint once. If it answers we go live; if not, this is a static
  // host and we load the bundled snapshot for the chosen scenario.
  let isLive = false;
  try
  {
    const res = await fetch('/state' + query, { cache: 'no-store' });
    if (res.ok) { publish(await res.json()); connected.value = true; isLive = true; }
  } catch { /* no device endpoint here */ }
  if (lastScenario !== store.state.scenario) return; // a newer open() superseded this one
  live.value = isLive;
  if (!isLive) { snapshot(); return; }

  if (typeof EventSource !== 'undefined')
  {
    es = new EventSource('/events' + query);
    es.onmessage = (e) => { connected.value = true; try { publish(JSON.parse(e.data)); } catch { /* partial frame */ } };
    es.onerror = () => { connected.value = false; };
  } else
  {
    poll(query);
  }
}

/**
 * Polls GET /state on a fixed cadence, for browsers without EventSource.
 *
 * @param {string} query - the `?scenario=` query string to request.
 * @returns {Promise<void>} resolves after scheduling the next poll.
 */
async function poll(query)
{
  try { publish(await (await fetch('/state' + query)).json()); connected.value = true; }
  catch { connected.value = false; }
  setTimeout(() => poll(query), 2500);
}

/**
 * Static-host fallback: loads the bundled snapshot for the current scenario, with the
 * normal scenario as a last resort.
 *
 * @returns {Promise<void>} resolves once a snapshot is loaded or all candidates fail.
 */
async function snapshot()
{
  for (const url of ['./state.' + store.state.scenario + '.json', './state.normal.json'])
  {
    try
    {
      const res = await fetch(url, { cache: 'no-store' });
      if (res.ok) { play(await res.json()); return; }
    } catch { /* try the next candidate */ }
  }
  connected.value = false;
}

/**
 * Drives the static fallback: cycles the bundled frames on the live cadence so the page
 * animates like the device feed.
 *
 * @param {object|object[]} data - a single snapshot or an array of frames.
 * @returns {void}
 */
function play(data)
{
  clearInterval(replay);
  const frames = Array.isArray(data) ? data : [data];
  connected.value = true;
  let i = 0;
  publish(frames[0]);
  if (frames.length > 1) replay = setInterval(() => { i = (i + 1) % frames.length; publish(frames[i]); }, 2000);
}

/**
 * Fetches the current snapshot once and publishes it, so a control action shows its
 * effect immediately instead of waiting for the next live tick. A no-op on a static host.
 *
 * @returns {Promise<void>} resolves once a fresh snapshot has been published, if available.
 */
export async function refresh()
{
  const query = '?scenario=' + encodeURIComponent(store.state.scenario);
  try
  {
    const res = await fetch('/state' + query, { cache: 'no-store' });
    if (res.ok) { publish(await res.json()); connected.value = true; }
  } catch { /* static host or device unreachable; the live feed will catch up */ }
}

/**
 * Opens the stream and reconnects whenever the scenario preference changes.
 *
 * @returns {void}
 */
export function connectFeed()
{
  open();
  store.subscribe(() => { if (store.state.scenario !== lastScenario) open(); });
}
