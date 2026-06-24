// bytes.js - small byte helpers for the pure-JS crypto used to authenticate control.
//
// The dashboard is served over plain http on an open hotspot, which is not a secure
// context, so window.crypto.subtle is unavailable; these back a small self-contained
// SHA-256/HMAC/HKDF instead.

/**
 * Encodes a string to its UTF-8 bytes.
 *
 * @param {string} str - the text to encode.
 * @returns {Uint8Array} the UTF-8 bytes.
 */
export const utf8 = (str) => new TextEncoder().encode(str);

/**
 * Renders bytes as a lowercase hex string.
 *
 * @param {Uint8Array} bytes - the bytes to render.
 * @returns {string} the lowercase hex.
 */
export const toHex = (bytes) =>
  Array.from(bytes, (b) => b.toString(16).padStart(2, '0')).join('');

/**
 * Concatenates byte arrays into one.
 *
 * @param {...Uint8Array} parts - the arrays to join, in order.
 * @returns {Uint8Array} the concatenation.
 */
export function concat(...parts)
{
  const total = parts.reduce((n, p) => n + p.length, 0);
  const out = new Uint8Array(total);
  let pos = 0;
  for (const part of parts) { out.set(part, pos); pos += part.length; }
  return out;
}
