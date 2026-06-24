// hmac.js - HMAC-SHA256 (RFC 2104) over the local SHA-256.

import { sha256 } from './sha256.js';
import { concat } from './bytes.js';

const BLOCK = 64;

/**
 * Computes HMAC-SHA256 over a message with a key of any length.
 *
 * @param {Uint8Array} key - the secret key bytes.
 * @param {Uint8Array} message - the bytes to authenticate.
 * @returns {Uint8Array} the 32-byte MAC.
 */
export function hmacSha256(key, message)
{
  let k = key;
  if (k.length > BLOCK) k = sha256(k);
  const block = new Uint8Array(BLOCK);
  block.set(k);
  const ipad = new Uint8Array(BLOCK);
  const opad = new Uint8Array(BLOCK);
  for (let i = 0; i < BLOCK; i++) { ipad[i] = block[i] ^ 0x36; opad[i] = block[i] ^ 0x5c; }
  return sha256(concat(opad, sha256(concat(ipad, message))));
}
