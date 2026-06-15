// Smoke test: confirms the facade loads, the native core is reachable, and the
// MQTT transport surfaces errors as rejected promises (no broker required).
const assert = require("node:assert");
const { version, MqttClient, Qos } = require("./dist/index.js");

async function main() {
  const v = version();
  console.log("pamoja version:", v);
  assert.strictEqual(typeof v, "string", "version() should return a string");

  assert.strictEqual(Qos.AtLeastOnce, "AtLeastOnce", "Qos should expose string levels");

  const client = new MqttClient({
    clientId: "smoke",
    host: "127.0.0.1",
    port: 47811,
    keepAliveSecs: 1,
  });

  assert.strictEqual(
    await client.isConnected(),
    false,
    "a fresh client should not be connected",
  );

  await assert.rejects(
    () => client.connect(),
    /transport error/,
    "connecting to a closed port should reject with a transport error",
  );

  assert.strictEqual(
    await client.isConnected(),
    false,
    "a failed connect should leave the client disconnected",
  );

  console.log("ok");
}

main().catch((err) => {
  console.error(err);
  process.exit(1);
});
