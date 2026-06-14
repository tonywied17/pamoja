// Smoke test: confirms the native module loads and the core is reachable.
const assert = require("node:assert");
const { version, formatTransportError } = require("./index.js");

const v = version();
console.log("zero-edge-core version:", v);
assert.strictEqual(typeof v, "string", "version() should return a string");

const rendered = formatTransportError("connection timed out");
console.log("formatted error:", rendered);
assert.ok(
  rendered.includes("transport error"),
  "formatTransportError should render through the core error model",
);

console.log("ok");
