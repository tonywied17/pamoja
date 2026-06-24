// pair.js - unlocking and sending authenticated control commands.
//
// Reading the dashboard is anonymous; control is not. The device shows a pairing code
// out of band; the user enters it here. The code is normalized, mixed with a server
// nonce through HKDF into a session key, and proven to the device with an HMAC - the
// code itself never leaves the browser. Each command then carries a monotonic counter
// and an HMAC over (counter, command), so an eavesdropper on the open hotspot can
// neither forge a command nor replay a captured one.
//
// The derived key and counter live in sessionStorage (per tab); a refresh keeps the
// unlocked session, closing the tab drops it.

import { hkdfSha256 } from './crypto/hkdf.js';
import { hmacSha256 } from './crypto/hmac.js';
import { utf8, toHex } from './crypto/bytes.js';
import { refresh } from './feed.js';
import { store } from '../store.js';
import { open } from '../nav.js';

const INFO = utf8('pamoja/dashboard/cmd v1');
const STORE_KEY = 'pamoja.pair';

/** Whether control is unlocked, i.e. a paired session exists in this tab. */
export const unlocked = $.signal(session() != null);

function session()
{
  try { return JSON.parse(sessionStorage.getItem(STORE_KEY)); } catch { return null; }
}

function save(s)
{
  if (s) sessionStorage.setItem(STORE_KEY, JSON.stringify(s));
  else sessionStorage.removeItem(STORE_KEY);
  unlocked.value = s != null;
}

const fromHex = (hex) =>
  Uint8Array.from({ length: hex.length / 2 }, (_, i) => parseInt(hex.substr(i * 2, 2), 16));

/** Normalizes a typed pairing code to the canonical secret: lowercase hex, no separators. */
const normalize = (code) => code.toLowerCase().replace(/[^0-9a-f]/g, '');

/**
 * Pairs this tab with the device using a pairing code.
 *
 * @param {string} code - the pairing code shown by the device (separators are ignored).
 * @returns {Promise<{ok: boolean, error?: string}>} success, or a stable error code.
 */
export async function pair(code)
{
  const secret = normalize(code);
  if (!secret) return { ok: false, error: 'auth.bad_mac' };
  let challenge;
  try { challenge = await (await fetch('/pair/challenge', { cache: 'no-store' })).json(); }
  catch { return { ok: false, error: 'auth.unreachable' }; }

  const key = hkdfSha256(utf8(challenge.nonce), utf8(secret), INFO, 32);
  const mac = toHex(hmacSha256(key, utf8('confirm\n' + challenge.sessionId)));
  const res = await fetch('/pair/confirm', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ sessionId: challenge.sessionId, mac }),
  });
  if (!res.ok)
  {
    const body = await res.json().catch(() => ({}));
    return { ok: false, error: body.error || 'auth.bad_mac' };
  }
  save({ sessionId: challenge.sessionId, key: toHex(key), counter: 0 });
  return { ok: true };
}

/** Locks control again, forgetting this tab's paired session. */
export function lock() { save(null); }

/** Opens the pairing dialog so the user can unlock control. */
export function promptUnlock() { open(() => store.dispatch('openPairing'), () => store.dispatch('closePairing')); }

/**
 * Sends an authenticated command, then refreshes the fleet so the change shows at once.
 *
 * @param {object} command - the command object to send.
 * @returns {Promise<{ok: boolean, error?: string}>} success, or a stable error code.
 */
export async function sendCommand(command)
{
  const s = session();
  if (!s) return { ok: false, error: 'auth.not_paired' };

  const counter = s.counter + 1;
  const payload = JSON.stringify(command);
  const mac = toHex(hmacSha256(fromHex(s.key), utf8(counter + '\n' + payload)));
  // Consume the counter even if the request fails, so a counter is never reused.
  save({ ...s, counter });

  let res;
  try
  {
    res = await fetch('/command', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ sessionId: s.sessionId, counter, cmd: payload, mac }),
    });
  } catch { return { ok: false, error: 'auth.unreachable' }; }

  if (res.ok) { await refresh(); return { ok: true }; }
  const body = await res.json().catch(() => ({}));
  // An expired or unknown session can no longer command; drop it so the UI re-locks.
  if (res.status === 401) lock();
  return { ok: false, error: body.error || 'auth.bad_mac' };
}
