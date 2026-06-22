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

import { store } from './store.js';

export const SCENARIOS = ['normal', 'alarm', 'sensor-fault', 'low-battery', 'link-lost', 'cold-start'];

export const fleet = $.signal(null);
export const connected = $.signal(true);

let es;
let lastScenario;
let replay;

async function open()
{
  if (es) { es.close(); es = null; }
  clearInterval(replay);
  lastScenario = store.state.scenario;
  const query = '?scenario=' + encodeURIComponent(store.state.scenario);

  // Probe the device endpoint once. If it answers we go live; if not, this is a static
  // host and we load the bundled snapshot for the chosen scenario.
  let live = false;
  try
  {
    const res = await fetch('/state' + query, { cache: 'no-store' });
    if (res.ok) { fleet.value = await res.json(); connected.value = true; live = true; }
  } catch { /* no device endpoint here */ }
  if (lastScenario !== store.state.scenario) return; // a newer open() superseded this one
  if (!live) { snapshot(); return; }

  if (typeof EventSource !== 'undefined')
  {
    es = new EventSource('/events' + query);
    es.onmessage = (e) => { connected.value = true; try { fleet.value = JSON.parse(e.data); } catch { /* partial frame */ } };
    es.onerror = () => { connected.value = false; };
  } else
  {
    poll(query);
  }
}

async function poll(query)
{
  try { fleet.value = await (await fetch('/state' + query)).json(); connected.value = true; }
  catch { connected.value = false; }
  setTimeout(() => poll(query), 2500);
}

// Static-host fallback: load the bundled snapshot for the current scenario (relative path,
// so it works under any base), with the normal scenario as a last resort.
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

/** Drives the static fallback: cycles the bundled frames on the live cadence so the page animates like the device feed. */
function play(data)
{
  clearInterval(replay);
  const frames = Array.isArray(data) ? data : [data];
  connected.value = true;
  let i = 0;
  fleet.value = frames[0];
  if (frames.length > 1) replay = setInterval(() => { i = (i + 1) % frames.length; fleet.value = frames[i]; }, 2000);
}

/** Opens the stream and reconnects whenever the scenario preference changes. */
export function connectFeed()
{
  open();
  store.subscribe(() => { if (store.state.scenario !== lastScenario) open(); });
}
