// hkdf.js - HKDF-SHA256 (RFC 5869) over the local HMAC-SHA256.

import { hmacSha256 } from './hmac.js';
import { concat } from './bytes.js';

/**
 * Derives key material from input keying material with HKDF-SHA256.
 *
 * @param {Uint8Array} salt - the non-secret salt (a per-session nonce here).
 * @param {Uint8Array} ikm - the input keying material (the pairing secret here).
 * @param {Uint8Array} info - a context label binding the output to its purpose.
 * @param {number} length - how many bytes of output to produce.
 * @returns {Uint8Array} the derived key material.
 */
export function hkdfSha256(salt, ikm, info, length)
{
  const prk = hmacSha256(salt && salt.length ? salt : new Uint8Array(32), ikm);
  const out = new Uint8Array(length);
  let t = new Uint8Array(0);
  let pos = 0;
  let counter = 1;
  while (pos < length)
  {
    t = hmacSha256(prk, concat(t, info, Uint8Array.of(counter)));
    const take = Math.min(t.length, length - pos);
    out.set(t.subarray(0, take), pos);
    pos += take;
    counter++;
  }
  return out;
}
