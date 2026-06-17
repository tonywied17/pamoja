export const NODES = [
  // Smallholder farms - East Africa. Soil probes, irrigation, well level.
  { lat: -1.29, lon: 36.82, g: 'farm', label: 'Nairobi gateway', hub: true },
  { lat: 0.28, lon: 34.75, g: 'farm', label: 'Kakamega irrigation node' },
  { lat: -0.7, lon: 36.43, g: 'farm', label: 'Nyandarua soil probe' },
  { lat: -3.39, lon: 36.68, g: 'farm', label: 'Arusha well-level sensor' },
  { lat: 0.05, lon: 37.65, g: 'farm', label: 'Meru drip controller' },
  { lat: -1.95, lon: 30.06, g: 'farm', label: 'Kigali field hub' },

  // Rural clinics - solar health posts across the Sahel and Central Africa.
  { lat: 9.08, lon: 7.49, g: 'clinic', label: 'Abuja health gateway', hub: true },
  { lat: 12.0, lon: 8.59, g: 'clinic', label: 'Kano ward power monitor' },
  { lat: 13.51, lon: 2.11, g: 'clinic', label: 'Niamey health-post relay' },
  { lat: 9.31, lon: 13.39, g: 'clinic', label: 'Garoua cold-chain sensor' },
  { lat: 4.85, lon: 31.58, g: 'clinic', label: 'Juba oxygen monitor' },
  { lat: 11.59, lon: 43.15, g: 'clinic', label: 'Djibouti stock sensor' },

  // Clean water - well, handpump, and flow sensors across South Asia.
  { lat: 26.91, lon: 75.79, g: 'water', label: 'Jaipur water gateway', hub: true },
  { lat: 24.59, lon: 73.71, g: 'water', label: 'Udaipur handpump node' },
  { lat: 23.26, lon: 77.41, g: 'water', label: 'Bhopal well-level sensor' },
  { lat: 22.72, lon: 75.86, g: 'water', label: 'Indore flow meter' },
  { lat: 21.15, lon: 79.09, g: 'water', label: 'Nagpur tank-level node' },
  { lat: 23.02, lon: 72.57, g: 'water', label: 'Ahmedabad pump node' },

  // Conservation - ranger relays and acoustic nodes across Southern Africa.
  { lat: -15.42, lon: 28.28, g: 'conservation', label: 'Lusaka conservancy gateway', hub: true },
  { lat: -13.1, lon: 31.8, g: 'conservation', label: 'Luangwa ranger relay' },
  { lat: -15.8, lon: 26.0, g: 'conservation', label: 'Kafue river-level sensor' },
  { lat: -19.3, lon: 23.1, g: 'conservation', label: 'Okavango wildlife node' },
  { lat: -18.6, lon: 26.5, g: 'conservation', label: 'Hwange waterhole node' },
  { lat: -16.5, lon: 23.6, g: 'conservation', label: 'Zambezi acoustic node' },

  // Off-grid mesh - Himalaya and South Asia villages.
  { lat: 27.7, lon: 85.32, g: 'village', label: 'Kathmandu relay', hub: true },
  { lat: 28.21, lon: 83.99, g: 'village', label: 'Pokhara mesh node' },
  { lat: 27.33, lon: 88.61, g: 'village', label: 'Gangtok relay' },
  { lat: 29.46, lon: 80.34, g: 'village', label: 'Darchula village node' },
  { lat: 26.49, lon: 87.28, g: 'village', label: 'Dharan store-and-forward' },
  { lat: 30.07, lon: 79.02, g: 'village', label: 'Chamoli ridge node' },
  { lat: 27.05, lon: 84.34, g: 'village', label: 'Chitwan field node' },

  // Disaster relay - typhoon coast, Philippines + western Pacific.
  { lat: 14.6, lon: 120.98, g: 'storm', label: 'Manila uplink gateway', hub: true },
  { lat: 11.24, lon: 125.0, g: 'storm', label: 'Tacloban mesh node' },
  { lat: 10.31, lon: 123.89, g: 'storm', label: 'Cebu relay' },
  { lat: 9.31, lon: 123.3, g: 'storm', label: 'Dumaguete shelter node' },
  { lat: 13.14, lon: 123.74, g: 'storm', label: 'Legazpi flood sensor' },
  { lat: 12.97, lon: 121.91, g: 'storm', label: 'Mindoro relay' },
  { lat: 15.48, lon: 120.6, g: 'storm', label: 'Tarlac inland gateway' },

  // Background community - the wider world the mesh keeps reaching into.
  { lat: -19.84, lon: 34.84, g: 'global', label: 'Beira, Mozambique' },
  { lat: -13.98, lon: 33.78, g: 'global', label: 'Lilongwe, Malawi' },
  { lat: 6.52, lon: 3.37, g: 'global', label: 'Lagos, Nigeria' },
  { lat: -6.79, lon: 39.21, g: 'global', label: 'Dar es Salaam' },
  { lat: 23.81, lon: 90.41, g: 'global', label: 'Dhaka, Bangladesh' },
  { lat: -13.53, lon: -71.97, g: 'global', label: 'Cusco, Peru' },
  { lat: -16.5, lon: -68.15, g: 'global', label: 'La Paz, Bolivia' },
  { lat: 14.64, lon: -90.51, g: 'global', label: 'Guatemala City' },
  { lat: 18.59, lon: -72.31, g: 'global', label: 'Port-au-Prince, Haiti' },
  { lat: 18.47, lon: -66.11, g: 'global', label: 'San Juan, Puerto Rico' },
  { lat: 21.03, lon: 105.85, g: 'global', label: 'Hanoi, Vietnam' },
  { lat: -8.34, lon: 115.09, g: 'global', label: 'Bali, Indonesia' },
  { lat: 17.97, lon: 102.63, g: 'global', label: 'Vientiane, Laos' },
  { lat: 3.14, lon: 101.69, g: 'global', label: 'Kuala Lumpur' },
  { lat: -1.94, lon: 30.06, g: 'global', label: 'Kigali, Rwanda' },
  { lat: 31.95, lon: 35.91, g: 'global', label: 'Amman, Jordan' },
  { lat: 35.69, lon: 51.39, g: 'global', label: 'Tehran, Iran' },
  { lat: 6.93, lon: 79.86, g: 'global', label: 'Colombo, Sri Lanka' },
  { lat: 12.97, lon: 77.59, g: 'global', label: 'Bengaluru, India' },
  { lat: -25.97, lon: 32.57, g: 'global', label: 'Maputo, Mozambique' },
  { lat: 0.39, lon: 9.45, g: 'global', label: 'Libreville, Gabon' },
  { lat: 51.51, lon: -0.13, g: 'global', label: 'maintainers, London' },
  { lat: 40.71, lon: -74.0, g: 'global', label: 'contributors, NYC' },
  { lat: 48.86, lon: 2.35, g: 'global', label: 'contributors, Paris' },
];

export const SCENARIOS = ['farm', 'clinic', 'water', 'conservation', 'village', 'storm'];

export const CRATES = [
  {
    id: 'pamoja-core',
    name: 'pamoja-core',
    role: 'the core',
    color: 'cream',
    blurb:
      'The device model: Transport, Device, Sensor, Actuator, Store and event-bus traits, plus one shared error and result type. Knows nothing about any specific protocol.',
  },
  {
    id: 'pamoja-codec',
    name: 'pamoja-codec',
    role: 'serialize',
    color: 'teal',
    blurb:
      'Pluggable serialization: compact CBOR, JSON, and raw bytes behind one Codec trait, so a few hundred bytes go over a metered link instead of kilobytes.',
  },
  {
    id: 'pamoja-mqtt',
    name: 'pamoja-mqtt',
    role: 'messaging',
    color: 'teal',
    blurb:
      'The MQTT transport: publish and subscribe over the broker pattern most IoT fleets already speak, behind the core Transport trait.',
  },
  {
    id: 'pamoja-coap',
    name: 'pamoja-coap',
    role: 'messaging',
    color: 'teal',
    blurb:
      'The CoAP transport: request/response built for constrained devices and lossy networks, the lightweight half of the messaging story.',
  },
  {
    id: 'pamoja-lora',
    name: 'pamoja-lora',
    role: 'radio',
    color: 'amber',
    blurb:
      'A no_std LoRa link model: the exact integer time-on-air of a payload and the duty-cycle off-time it forces, so a long-range node stays inside regulation and power budget.',
  },
  {
    id: 'pamoja-lorawan',
    name: 'pamoja-lorawan',
    role: 'radio',
    color: 'amber',
    blurb:
      'LoRaWAN 1.0.x MAC framing with AES-CMAC integrity and AES payload encryption, validated against the FIPS-197 and RFC 4493 reference vectors. Real over-the-air traffic, not a round-trip toy.',
  },
  {
    id: 'pamoja-mesh',
    name: 'pamoja-mesh',
    role: 'mesh',
    color: 'coral',
    blurb:
      'An addressed, hop-limited, CRC-checked frame for cheap local radio (ESP-NOW, nRF24) plus a duplicate suppressor that floods a packet across the mesh exactly once. The backbone where there is no infrastructure.',
  },
  {
    id: 'pamoja-routing',
    name: 'pamoja-routing',
    role: 'mesh',
    color: 'coral',
    blurb:
      'Reverse-path mesh routing: a node learns routes from the traffic it overhears, keeps the cheapest one, and decides per packet whether to deliver, relay, or fall back to flooding. The airtime and battery that flooding wastes, saved.',
  },
  {
    id: 'pamoja-modbus',
    name: 'pamoja-modbus',
    role: 'field I/O',
    color: 'teal',
    blurb:
      'Modbus RTU framing with CRC-16/Modbus and builders for the standard reads and writes, so a node can talk to long-cable RS485 field sensors: soil probes, energy meters, water meters.',
  },
  {
    id: 'pamoja-can',
    name: 'pamoja-can',
    role: 'field I/O',
    color: 'teal',
    blurb:
      'CAN 2.0 and CAN-FD frames with 11- and 29-bit IDs, plus J1939 decode and compose for the trucks, tractors, and gensets that speak it. The path to diesel and hydraulic machinery.',
  },
  {
    id: 'pamoja-security',
    name: 'pamoja-security',
    role: 'trust',
    color: 'amber',
    blurb:
      'Memory safety by construction, heading to TLS 1.3 / DTLS, X.509 device identity, and signed OTA with verified rollback. Trust that survives a hostile link.',
  },
  {
    id: 'pamoja-audit',
    name: 'pamoja-audit',
    role: 'trust',
    color: 'amber',
    blurb:
      'A no_std tamper-evident log: signed, SHA-256 hash-chained entries where each commits to the one before, so altering, reordering, or dropping any record breaks verification. Proof for anything that has to be trusted after the fact - a cold chain, a water log, a clinic record.',
  },
  {
    id: 'pamoja-telemetry',
    name: 'pamoja-telemetry',
    role: 'observe',
    color: 'teal',
    blurb:
      'Allocation-free observability: structured leveled events and a reporter that ships only what is worth the bytes as link cost rises, while counting everything so a periodic snapshot stays complete.',
  },
  {
    id: 'pamoja-power',
    name: 'pamoja-power',
    role: 'energy',
    color: 'amber',
    blurb:
      'Energy-aware duty cycling: async scheduling that wakes, works, and sleeps so a node lives on a small solar panel and a battery instead of mains power.',
  },
  {
    id: 'pamoja-sync',
    name: 'pamoja-sync',
    role: 'resilience',
    color: 'teal',
    blurb:
      'Offline-first store-and-forward: a device disconnected for days buffers locally and loses nothing, then drains the backlog when a link returns.',
  },
  {
    id: 'pamoja-kit',
    name: 'pamoja-kit',
    role: 'ergonomics',
    color: 'cream',
    blurb:
      'The plain-language helper layer: keep a temperature, smooth a noisy reading, warn before a tank runs dry. Each names the goal over the math, with the real algorithm one layer down.',
  },
  {
    id: 'pamoja-profile',
    name: 'pamoja-profile',
    role: 'ergonomics',
    color: 'cream',
    blurb:
      'Named, ready-to-run device profiles: pick a preset like an irrigation node or a water-point monitor, or load one from a shareable JSON manifest. The profile is plain data and the control policy is pure, so a whole node is testable with no hardware.',
  },
];

// Crates on the committed roadmap, shown in the constellation as a dashed outer
// ring. Same idea as the vision tracks: this is direction, not shipped.
export const PLANNED_CRATES = [
  {
    id: 'pamoja-satellite',
    name: 'pamoja-satellite',
    role: 'radio · planned',
    planned: true,
    blurb:
      'LoRa-to-satellite and NB-IoT NTN uplink. When local mesh and community gateways are gone, the same node is heard by a satellite passing overhead. The last and most expensive rung of the transport ladder, kept to tens of bytes so one shared, sponsored gateway can carry a whole area.',
  },
  {
    id: 'pamoja-cellular',
    name: 'pamoja-cellular',
    role: 'radio · planned',
    planned: true,
    blurb:
      'Cellular uplink (LTE-M / NB-IoT) as a metered rung above mesh and below satellite, with the same store-and-forward buffering and compact codecs, so a node uses it sparingly and only when the cheaper links are unavailable.',
  },
  {
    id: 'pamoja-meshtastic',
    name: 'pamoja-meshtastic',
    role: 'mesh · planned',
    planned: true,
    blurb:
      'A Meshtastic bridge so pamoja meshes interoperate with the large off-grid LoRa-mesh community already deployed in the field, instead of starting a separate island.',
  },
  {
    id: 'pamoja-gpio',
    name: 'pamoja-gpio',
    role: 'field I/O · planned',
    planned: true,
    blurb:
      'GPIO, I2C and SPI for the cheap sensors and actuators wired straight to a microcontroller, behind the same Sensor and Actuator traits as every other driver.',
  },
  {
    id: 'pamoja-tls',
    name: 'pamoja-tls',
    role: 'trust · planned',
    planned: true,
    blurb:
      'TLS 1.3 and DTLS with X.509 device identity: confidentiality and an authenticated identity that survive a hostile or shared link, sized to run on constrained hardware.',
  },
  {
    id: 'pamoja-ota',
    name: 'pamoja-ota',
    role: 'trust · planned',
    planned: true,
    blurb:
      'Signed over-the-air updates with verified rollback, so a fleet scattered across a region can be fixed and trusted without a truck roll, and a bad update can never brick a node.',
  },
  {
    id: 'pamoja-dashboard',
    name: 'pamoja-dashboard',
    role: 'resilience · planned',
    planned: true,
    blurb:
      'A local-first dashboard a node serves over its own hotspot, so a community can see its own data with no cloud and no internet - the offline-first answer to "where does the data go".',
  },
  {
    id: 'pamoja-mavlink',
    name: 'pamoja-mavlink',
    role: 'robotics · planned',
    planned: true,
    blurb:
      'MAVLink for drones and a ROS 2 / Zenoh bridge for robots, modelled as ordinary pamoja devices so the same code that reads a sensor can fly a survey or drive an actuator.',
  },
];

export const SCENARIO_CRATES = {
  farm: ['pamoja-modbus', 'pamoja-lora', 'pamoja-kit', 'pamoja-power', 'pamoja-sync', 'pamoja-profile'],
  clinic: ['pamoja-telemetry', 'pamoja-security', 'pamoja-audit', 'pamoja-sync', 'pamoja-power', 'pamoja-profile'],
  water: ['pamoja-modbus', 'pamoja-kit', 'pamoja-audit', 'pamoja-sync', 'pamoja-lora', 'pamoja-profile'],
  conservation: ['pamoja-mesh', 'pamoja-routing', 'pamoja-lora', 'pamoja-telemetry', 'pamoja-power', 'pamoja-codec'],
  village: ['pamoja-mesh', 'pamoja-routing', 'pamoja-lora', 'pamoja-sync', 'pamoja-codec'],
  storm: ['pamoja-mesh', 'pamoja-routing', 'pamoja-lorawan', 'pamoja-telemetry', 'pamoja-power'],
};

export const LANGUAGES = [
  {
    id: 'rust',
    name: 'Rust',
    pkg: 'pamoja-core, pamoja-mqtt',
    status: 'available',
    code: `use pamoja_core::Transport;
use pamoja_mqtt::{MqttConfig, MqttTransport};

let mut transport = MqttTransport::new(
    MqttConfig::new("sensor-1", "localhost", 1883),
);
transport.connect().await?;
transport.subscribe("sensors/+/temperature").await?;
transport.send("sensors/1/temperature", b"21.5").await?;`,
  },
  {
    id: 'ts',
    name: 'TypeScript',
    pkg: '@pamoja/core',
    status: 'in progress',
    code: `import { MqttClient } from '@pamoja/core'

const client = new MqttClient({
  clientId: 'sensor-1', host: 'localhost', port: 1883,
})
await client.connect()
await client.subscribe('sensors/+/temperature')
await client.publish('sensors/1/temperature', '21.5')

for await (const message of client) {
  console.log(message.topic, message.payload.toString())
}`,
  },
  {
    id: 'python',
    name: 'Python',
    pkg: 'pamoja-core',
    status: 'in progress',
    code: `import asyncio
from pamoja import MqttClient

async def main():
    async with MqttClient(
        client_id="sensor-1", host="localhost", port=1883,
    ) as client:
        await client.subscribe("sensors/+/temperature")
        await client.publish("sensors/1/temperature", "21.5")
        async for message in client:
            print(message.topic, message.payload.decode())

asyncio.run(main())`,
  },
  {
    id: 'csharp',
    name: 'C# / .NET',
    pkg: 'Pamoja.Core',
    status: 'in progress',
    code: `using Pamoja.Core;

await using var client = new MqttClient(new MqttClientOptions
{
    ClientId = "sensor-1",
    Host = "localhost",
    Port = 1883,
});
await client.ConnectAsync();
await client.SubscribeAsync("sensors/+/temperature");
await client.PublishAsync("sensors/1/temperature", "21.5");

await foreach (var message in client)
    Console.WriteLine($"{message.Topic}: {message.Payload.Length} bytes");`,
  },
];

export const PLANNED_LANGS = ['Lua', 'WebAssembly', 'Kotlin', 'Swift', 'Go'];

export const TIERS = [
  {
    id: 'spark',
    name: 'Spark',
    amount: 15,
    accent: 'teal',
    headline: 'One sensing node',
    items: [
      'A microcontroller and one sensor',
      'Flashed with a pamoja device profile',
      'Enough to read and report one thing reliably',
    ],
  },
  {
    id: 'link',
    name: 'Link',
    amount: 40,
    accent: 'amber',
    headline: 'Reach off-grid',
    items: [
      'Everything in Spark',
      'A LoRa radio for kilometres of range',
      'Joins the mesh where there is no tower',
    ],
  },
  {
    id: 'fieldkit',
    name: 'Field Kit',
    amount: 120,
    accent: 'coral',
    featured: true,
    headline: 'A node that lives outside',
    items: [
      'Board, sensors, solar panel and battery',
      'A USB / SD card with the SDK preloaded',
      'Duty-cycled to run for a season unattended',
    ],
  },
  {
    id: 'lifeline',
    name: 'Lifeline',
    amount: 300,
    accent: 'amber',
    headline: 'Guard something critical',
    items: [
      'A clinic, water-point, or ranger node',
      'Tamper-evident, hash-chained records',
      'Store-and-forward over whatever link exists',
    ],
  },
];

// Sponsor the uplink: hardware is one-time, but the link is the one recurring
// cost. These keep the last, most expensive rung of the ladder paid for, so it
// stays free at the point of use. `role` routes the button to donor or vendor.
export const UPLINKS = [
  {
    id: 'sat',
    name: 'Gateway uplink',
    amount: 20,
    per: '/ month',
    accent: 'coral',
    role: 'donor',
    headline: 'Reach a satellite when the towers fall',
    items: [
      'A LoRa-to-satellite plan for one shared gateway',
      "Carries a whole area's emergency traffic",
      'The last rung of the ladder, kept to tens of bytes',
    ],
  },
  {
    id: 'cell',
    name: 'Cellular backhaul',
    amount: 8,
    per: '/ month',
    accent: 'sky',
    role: 'donor',
    headline: 'Where there is coverage but no wifi',
    items: [
      'An LTE-M / NB-IoT plan for a gateway',
      'A cheaper rung below satellite',
      'Store-and-forward, so it sips data',
    ],
  },
  {
    id: 'partner',
    name: 'Carrier / integrator',
    per: 'partner',
    accent: 'amber',
    role: 'vendor',
    headline: 'Operators and integrators',
    items: [
      'Donate satellite or cellular airtime',
      'Co-develop the uplink crates',
      'Help one sponsored link reach more places',
    ],
  },
];

export const TRACKS = [
  {
    id: 'radio',
    title: 'Messaging & radio',
    accent: 'teal',
    lead: 'A cost-aware ladder that tries the cheapest link first and buffers when there is none.',
    tags: [
      { t: 'MQTT', on: true }, { t: 'CoAP', on: true }, { t: 'LoRa', on: true },
      { t: 'LoRaWAN', on: true }, { t: 'mesh', on: true }, { t: 'Meshtastic bridge', on: false },
      { t: 'cellular uplink', on: false }, { t: 'LoRa-to-satellite', on: false },
    ],
  },
  {
    id: 'hardware',
    title: 'Hardware & sensors',
    accent: 'amber',
    lead: 'Talk to the cheap, salvageable parts the field already runs on, by name instead of by pin.',
    tags: [
      { t: 'Modbus / RS485', on: true }, { t: 'CAN / J1939', on: true }, { t: 'device profiles', on: true },
      { t: 'serial', on: false }, { t: 'GPIO / I2C / SPI', on: false }, { t: 'driver catalog', on: false },
    ],
  },
  {
    id: 'robotics',
    title: 'Robotics & drones',
    accent: 'coral',
    lead: 'Robots and drones modelled as ordinary pamoja devices, controlled from any language.',
    tags: [
      { t: 'ROS 2 bridge', on: false }, { t: 'Zenoh', on: false }, { t: 'MAVLink', on: false },
    ],
  },
  {
    id: 'resilience',
    title: 'Resilience & power',
    accent: 'sky',
    lead: 'Offline-first by default, awake only when it must be, so a node lives on sun and a battery.',
    tags: [
      { t: 'store-and-forward', on: true }, { t: 'duty cycling', on: true }, { t: 'tamper-evident audit', on: true },
      { t: 'local-first dashboards', on: false }, { t: 'data-mule sync', on: false },
    ],
  },
  {
    id: 'security',
    title: 'Security & trust',
    accent: 'amber',
    lead: 'Memory-safe by construction, with identity and signed updates that survive a hostile link.',
    tags: [
      { t: 'memory safety', on: true }, { t: 'TLS 1.3 / DTLS', on: false }, { t: 'X.509 identity', on: false },
      { t: 'signed OTA + rollback', on: false },
    ],
  },
  {
    id: 'reach',
    title: 'Reach',
    accent: 'forest',
    lead: 'One engine, idiomatic in every language a device developer actually uses, plus plain-language helpers.',
    tags: [
      { t: 'Rust', on: true }, { t: 'TypeScript', on: true }, { t: 'Python', on: true }, { t: 'C# / .NET', on: true },
      { t: 'Lua', on: false }, { t: 'WebAssembly', on: false }, { t: 'Kotlin', on: false }, { t: 'Swift', on: false }, { t: 'Go', on: false },
    ],
  },
];

export const STATS = [
  { value: 22, label: 'capability crates', suffix: '' },
  { value: 4, label: 'languages shipping', suffix: '' },
  { value: 256, label: 'KB of RAM, target floor', suffix: '' },
  { value: 0, label: 'cost to use, forever', prefix: '$', suffix: '' },
];
