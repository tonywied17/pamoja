// edits.js - client-side group/sensor management (demo).
//
// The device owns the real fleet; this overlays user edits (added/removed groups and
// sensors) on top of the live snapshot so the management UX exists without touching
// device provisioning. Edits persist in localStorage via the store. A real build would
// send these as authenticated provisioning commands; here they are applied in-browser.

import { fleet } from './feed.js';
import { store } from './store.js';

// Sensor types a user can add, each carrying the canonical key/unit/band so the right
// visualization and localized label appear automatically.
export const SENSOR_PRESETS = [
  { id: 'temperature', key: 'temperature', unit: 'celsius', band: [2, 8] },
  { id: 'humidity', key: 'humidity', unit: 'percent', band: [30, 60] },
  { id: 'soc', key: 'state_of_charge', unit: 'percent', band: [20, 100] },
  { id: 'power', key: 'pv_power', unit: 'watt', band: [0, 400] },
  { id: 'pressure', key: 'pressure', unit: 'hectopascal', band: [980, 1040] },
  { id: 'wind', key: 'wind_speed', unit: 'meter_per_second', band: [0, 20] },
  { id: 'light', key: 'illuminance', unit: 'lux', band: [0, 100000] },
  { id: 'voltage', key: 'battery_voltage', unit: 'volt', band: [11.8, 14.6] },
  // Field-kit sensors (Farm node, Health post).
  { id: 'soil', key: 'soil_moisture', unit: 'percent', band: [36, 100] },
  { id: 'well', key: 'well_level', unit: 'percent', band: [20, 100] },
  { id: 'soiltrend', key: 'soil_trend', unit: 'percent', band: [0, 100] },
  { id: 'fridge', key: 'fridge_temp', unit: 'celsius', band: [2, 8] },
  { id: 'wardpower', key: 'ward_power', unit: 'percent', band: [50, 100] },
  { id: 'oxygen', key: 'oxygen_stock', unit: 'percent', band: [30, 100] },
  { id: 'flow', key: 'flow_rate', unit: 'liter_per_minute', band: [0, 16] },
  { id: 'tank', key: 'storage_tank', unit: 'percent', band: [20, 100] },
  { id: 'flowtrend', key: 'flow_trend', unit: 'liter_per_minute', band: [0, 16] },
  // Ranger relay / mesh node.
  { id: 'acoustic', key: 'acoustic', unit: 'decibel', band: [0, 120] },
  { id: 'batterylevel', key: 'battery_level', unit: 'percent', band: [20, 100] },
  { id: 'neighbours', key: 'neighbours', unit: 'count', value: 5, band: [1, 12] },
  { id: 'hops', key: 'hops', unit: 'count', value: 3, band: [1, 8] },
  { id: 'relayed', key: 'messages_relayed', unit: 'count', value: 300, band: [0, 99999] },
  // Discrete (state chip / valve / mesh) and the tamper-evident chain log.
  { id: 'valve', key: 'drip_valve', unit: 'state', state: 'state.closed' },
  { id: 'uplink', key: 'uplink', unit: 'state', state: 'state.synced' },
  { id: 'pump', key: 'pump_health', unit: 'state', state: 'state.nominal' },
  { id: 'relaystatus', key: 'relay_status', unit: 'state', state: 'state.online' },
  { id: 'routing', key: 'routing', unit: 'state', state: 'mesh.optimised' },
  // The mesh map is one sensor that only applies to mesh-link groups; its value is the
  // number of mesh nodes drawn.
  { id: 'meshrelay', key: 'mesh_relay', unit: 'state', state: 'mesh.optimised', value: 5, meshOnly: true },
  { id: 'tamper', key: 'tamper_log', unit: 'record', value: 1000 },
];

export const LINK_KINDS = ['lora', 'wifi', 'cellular', 'satellite', 'ethernet', 'mesh'];

export const uid = (p) => p + Date.now().toString(36) + Math.random().toString(36).slice(2, 5);

export function statusFor(value, band)
{
  if (!band) return 'ok';
  const [lo, hi] = band, margin = (hi - lo) * 0.18;
  if (value < lo - margin || value > hi + margin) return 'alarm';
  if (value < lo || value > hi) return 'warn';
  return 'ok';
}

const worst = (a, b) => (['ok', 'warn', 'alarm'].indexOf(a) >= ['ok', 'warn', 'alarm'].indexOf(b) ? a : b);

// Sorts items by a saved order of ids; anything not in the order keeps its relative
// position after the ordered ones (stable sort), so new items append cleanly.
function applyOrder(items, order, idOf)
{
  if (!order || !order.length) return items;
  const idx = new Map(order.map((id, i) => [id, i]));
  return items.slice().sort((a, b) => (idx.has(idOf(a)) ? idx.get(idOf(a)) : 1e9) - (idx.has(idOf(b)) ? idx.get(idOf(b)) : 1e9));
}

/** Builds a fresh custom sensor for a group (numeric, discrete-state, or chain log). */
export function makeSensor(groupId, presetId, value)
{
  const p = SENSOR_PRESETS.find((x) => x.id === presetId) || SENSOR_PRESETS[0];
  const base = { id: uid('s'), groupId, battery: null, mode: 'active', events: [], custom: true };
  // A discrete sensor (valve, uplink) carries a state code instead of a numeric reading.
  if (p.state)
  {
    return { ...base, reading: { key: p.key, value: p.value || 0, unit: p.unit || 'state', status: 'ok', state: p.state }, history: [] };
  }
  const v = Number.isFinite(value) ? value : (p.band ? (p.band[0] + p.band[1]) / 2 : (p.value ?? 0));
  return {
    ...base,
    reading: { key: p.key, value: v, unit: p.unit, status: statusFor(v, p.band), band: p.band, trend: 'steady' },
    history: Array.from({ length: 12 }, () => v),
  };
}

/** Builds a fresh custom group for an org. */
export function makeGroup(orgId, name, kind)
{
  return { id: uid('g'), orgId, name: name || 'New group', link: { kind: kind || 'lora', strength: 3, online: true }, status: 'ok', sensors: [], custom: true };
}

/** Applies the store's edits to a raw fleet snapshot, returning a new fleet. */
export function applyEdits(raw, edits)
{
  if (!raw) return raw;
  const rmG = new Set(edits.rmGroups), rmS = new Set(edits.rmSensors);
  const gOrder = edits.groupOrder || {}, sOrder = edits.sensorOrder || {};
  let fleetStatus = 'ok';
  const orgs = raw.orgs.map((o) =>
  {
    let groups = o.groups.filter((g) => !rmG.has(g.id)).map((g) => ({ ...g }));
    groups = groups.concat(edits.addGroups.filter((ag) => ag.orgId === o.id));
    groups = applyOrder(groups, gOrder[o.id], (g) => g.id);
    groups = groups.map((g) =>
    {
      let sensors = (g.sensors || []).filter((s) => !rmS.has(g.id + '/' + s.id));
      sensors = sensors.concat(edits.addSensors.filter((s) => s.groupId === g.id));
      sensors = applyOrder(sensors, sOrder[g.id], (s) => s.id);
      let st = g.link && g.link.online === false ? 'warn' : 'ok';
      for (const s of sensors) st = worst(st, s.reading.status || 'ok');
      fleetStatus = worst(fleetStatus, st);
      return { ...g, sensors, status: st };
    });
    return { ...o, groups };
  });
  return { ...raw, orgs, status: fleetStatus };
}

/** The live fleet with the user's edits applied; recomputed on each read. */
export const currentFleet = () => applyEdits(fleet.value, store.state.edits);
