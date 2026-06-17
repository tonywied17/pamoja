import { Vector3, Quaternion } from 'three';

const DEG = Math.PI / 180;

export function latLonToVec3(lat, lon, r = 1)
{
  const la = lat * DEG;
  const lo = lon * DEG;
  const cl = Math.cos(la);
  return new Vector3(r * cl * Math.cos(lo), r * Math.sin(la), r * cl * Math.sin(lo));
}

export function vec3ToLatLon(v)
{
  const n = v.clone().normalize();
  return {
    lat: Math.asin(Math.max(-1, Math.min(1, n.y))) / DEG,
    lon: Math.atan2(n.z, n.x) / DEG,
  };
}

export function lonLatToPixel(lon, lat, w, h)
{
  return [((lon + 180) / 360) * w, ((90 - lat) / 180) * h];
}

export function arcPoints(a, b, segments = 48, lift = 0.42)
{
  const va = latLonToVec3(a.lat, a.lon, 1);
  const vb = latLonToVec3(b.lat, b.lon, 1);
  const angle = va.angleTo(vb);
  const peak = 1 + lift * Math.min(1, angle / Math.PI) + 0.04;
  const pts = [];
  const sinT = Math.sin(angle) || 1e-6;
  for (let i = 0; i <= segments; i++)
  {
    const t = i / segments;
    const wa = Math.sin((1 - t) * angle) / sinT;
    const wb = Math.sin(t * angle) / sinT;
    const dir = new Vector3(
      va.x * wa + vb.x * wb,
      va.y * wa + vb.y * wb,
      va.z * wa + vb.z * wb,
    ).normalize();
    const r = 1 + (peak - 1) * Math.sin(Math.PI * t);
    pts.push(dir.multiplyScalar(r));
  }
  return pts;
}

const _target = new Vector3();
const _forward = new Vector3();
const _north = new Vector3();
const _proj = new Vector3();
const _up = new Vector3();
const _cross = new Vector3();
const _roll = new Quaternion();
export function orientationFor(lat, lon, tilt = 0, out = new Quaternion())
{
  _target.copy(latLonToVec3(lat, lon, 1));
  _forward.set(0, Math.sin(tilt), Math.cos(tilt)).normalize();
  out.setFromUnitVectors(_target, _forward);

  // Roll-correct so the globe's north pole projects to screen-up.
  _north.set(0, 1, 0).applyQuaternion(out);
  _proj.copy(_north).addScaledVector(_forward, -_north.dot(_forward));
  if (_proj.lengthSq() < 1e-8) return out;
  _proj.normalize();
  _up.set(0, 1, 0).addScaledVector(_forward, -_forward.y);
  if (_up.lengthSq() < 1e-8) return out;
  _up.normalize();
  const ang = Math.atan2(_cross.crossVectors(_proj, _up).dot(_forward), _proj.dot(_up));
  _roll.setFromAxisAngle(_forward, ang);
  return out.premultiply(_roll);
}

export async function loadLandMask(width = 1024)
{
  const w = width;
  const h = width / 2;
  const canvas =
    typeof OffscreenCanvas !== 'undefined'
      ? new OffscreenCanvas(w, h)
      : Object.assign(document.createElement('canvas'), { width: w, height: h });
  const ctx = canvas.getContext('2d', { willReadFrequently: true });
  ctx.fillStyle = '#000';
  ctx.fillRect(0, 0, w, h);
  let fallback = false;

  try
  {
    const [{ feature }, topo] = await Promise.all([
      import('topojson-client'),
      fetch('https://cdn.jsdelivr.net/npm/world-atlas@2/land-110m.json').then((r) =>
      {
        if (!r.ok) throw new Error('land fetch ' + r.status);
        return r.json();
      }),
    ]);
    const land = feature(topo, topo.objects.land);
    ctx.fillStyle = '#fff';
    for (const f of land.features) drawGeometry(ctx, f.geometry, w, h);
  } catch (err)
  {
    console.warn('[pamoja] land mask unavailable, using procedural fallback:', err.message);
    proceduralLand(ctx, w, h);
    fallback = true;
  }

  const img = ctx.getImageData(0, 0, w, h).data;
  const sample = (lat, lon) =>
  {
    const [px, py] = lonLatToPixel(lon, lat, w, h);
    const x = Math.max(0, Math.min(w - 1, px | 0));
    const y = Math.max(0, Math.min(h - 1, py | 0));
    return img[(y * w + x) * 4];
  };
  return { w, h, sample, fallback };
}

function drawGeometry(ctx, geom, w, h)
{
  if (!geom) return;
  const polys = geom.type === 'Polygon' ? [geom.coordinates] : geom.coordinates;
  for (const poly of polys)
  {
    for (const ring of poly)
    {
      ctx.beginPath();
      for (let i = 0; i < ring.length; i++)
      {
        const [lon, lat] = ring[i];
        const [px, py] = lonLatToPixel(lon, lat, w, h);
        if (i === 0) ctx.moveTo(px, py);
        else ctx.lineTo(px, py);
      }
      ctx.closePath();
      ctx.fill();
    }
  }
}

function proceduralLand(ctx, w, h)
{
  const blobs = [
    [16, 8, 9, 16], [20, 38, 7, 14], [50, 22, 14, 9], [60, 38, 9, 12],
    [78, 28, 8, 7], [82, 62, 5, 5], [30, 62, 6, 11], [12, 70, 4, 4],
  ];
  ctx.fillStyle = '#fff';
  for (const [cx, cy, rx, ry] of blobs)
  {
    ctx.beginPath();
    ctx.ellipse((cx / 100) * w, (cy / 100) * h, (rx / 100) * w, (ry / 100) * h, 0, 0, Math.PI * 2);
    ctx.fill();
  }
}
