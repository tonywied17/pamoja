import
  {
    Group, Points, BufferGeometry, BufferAttribute, ShaderMaterial, AdditiveBlending,
    LineSegments, Color, Vector3, Matrix4,
  } from 'three';
import { C, GLOBE } from './config.js';
import { NODES } from './data.js';
import { latLonToVec3, arcPoints } from './geo.js';

const GROUP_COLOR = {
  farm: new Color('#1fa995'),
  clinic: new Color('#ffb627'),
  water: new Color('#39c2e0'),
  conservation: new Color('#5fd07a'),
  village: new Color('#33c9a6'),
  storm: new Color('#f26a4b'),
  global: new Color('#fbf3e4'),
};

function activityFor(focus, group, hub)
{
  if (!focus) return group === 'global' ? 0.55 : 0.34 + (hub ? 0.2 : 0);
  if (group === focus) return hub ? 1 : 0.92;
  if (group === 'global') return 0.16;
  return 0.05;
}

const NODE_VERT = `
  uniform float uTime; uniform float uScale;
  attribute vec3 aColor; attribute float aActive; attribute float aSize; attribute float aPhase;
  varying vec3 vColor; varying float vActive; varying float vTw;
  void main() {
    vColor = aColor; vActive = aActive;
    vTw = 0.6 + 0.4 * sin(uTime * 2.6 + aPhase * 6.2831853);
    vec4 mv = modelViewMatrix * vec4(position, 1.0);
    float size = (0.009 + 0.015 * aSize) * (0.55 + 0.45 * vActive) * (0.85 + 0.3 * vTw);
    gl_PointSize = size * uScale / -mv.z;
    gl_Position = projectionMatrix * mv;
  }
`;

const NODE_FRAG = `
  precision mediump float;
  varying vec3 vColor; varying float vActive; varying float vTw;
  void main() {
    vec2 d = gl_PointCoord - vec2(0.5);
    float r = length(d);
    if (r > 0.5) discard;
    float core = smoothstep(0.5, 0.0, r);
    float glow = smoothstep(0.5, 0.12, r);
    float a = (0.22 * glow + core) * (0.12 + 0.88 * vActive);
    gl_FragColor = vec4(vColor * 1.25, a);
  }
`;

const HALO_VERT = `
  uniform float uTime; uniform float uScale;
  attribute vec3 aColor; attribute float aActive; attribute float aSize; attribute float aPhase;
  varying vec3 vColor; varying float vActive; varying float vT;
  void main() {
    vColor = aColor; vActive = aActive;
    vT = fract(uTime * 0.55 + aPhase);
    vec4 mv = modelViewMatrix * vec4(position, 1.0);
    float size = (0.10 + 0.06 * aSize) * uScale;
    gl_PointSize = size / -mv.z;
    gl_Position = projectionMatrix * mv;
  }
`;

const HALO_FRAG = `
  precision mediump float;
  varying vec3 vColor; varying float vActive; varying float vT;
  void main() {
    vec2 d = gl_PointCoord - vec2(0.5);
    float r = length(d) * 2.0;
    float ring = smoothstep(0.06, 0.0, abs(r - vT));
    float a = ring * (1.0 - vT) * vActive * 0.6;
    if (a < 0.01) discard;
    gl_FragColor = vec4(vColor, a);
  }
`;

class NodeField
{
  constructor()
  {
    this.group = new Group();
    const n = NODES.length;
    this.pos = new Float32Array(n * 3);
    this.col = new Float32Array(n * 3);
    this.active = new Float32Array(n);
    this.target = new Float32Array(n);
    const size = new Float32Array(n);
    const phase = new Float32Array(n);

    NODES.forEach((node, i) =>
    {
      const v = latLonToVec3(node.lat, node.lon, GLOBE.radius * 1.012);
      this.pos.set([v.x, v.y, v.z], i * 3);
      const c = GROUP_COLOR[node.g] || C.cream;
      this.col.set([c.r, c.g, c.b], i * 3);
      size[i] = node.hub ? 1 : 0.42;
      phase[i] = Math.random();
    });

    const geo = new BufferGeometry();
    geo.setAttribute('position', new BufferAttribute(this.pos, 3));
    geo.setAttribute('aColor', new BufferAttribute(this.col, 3));
    geo.setAttribute('aActive', new BufferAttribute(this.active, 1));
    geo.setAttribute('aSize', new BufferAttribute(size, 1));
    geo.setAttribute('aPhase', new BufferAttribute(phase, 1));
    this.geo = geo;

    const uniforms = () => ({ uTime: { value: 0 }, uScale: { value: 1000 } });
    this.coreMat = new ShaderMaterial({
      uniforms: uniforms(), vertexShader: NODE_VERT, fragmentShader: NODE_FRAG,
      transparent: true, depthWrite: false, blending: AdditiveBlending,
    });
    this.haloMat = new ShaderMaterial({
      uniforms: uniforms(), vertexShader: HALO_VERT, fragmentShader: HALO_FRAG,
      transparent: true, depthWrite: false, blending: AdditiveBlending,
    });
    this.group.add(new Points(geo, this.haloMat));
    this.group.add(new Points(geo, this.coreMat));
  }

  setFocus(focus)
  {
    NODES.forEach((node, i) => { this.target[i] = activityFor(focus, node.g, node.hub); });
  }

  setScale(s) { this.coreMat.uniforms.uScale.value = s; this.haloMat.uniforms.uScale.value = s; }

  update(t, dt)
  {
    this.coreMat.uniforms.uTime.value = t;
    this.haloMat.uniforms.uTime.value = t;
    const k = 1 - Math.exp(-dt * 3.2);
    for (let i = 0; i < this.active.length; i++)
    {
      this.active[i] += (this.target[i] - this.active[i]) * k;
    }
    this.geo.attributes.aActive.needsUpdate = true;
  }
}

class ArcNetwork
{
  constructor()
  {
    this.group = new Group();
    this.byKey = this._buildArcs();
    this.activeKey = 'global';
    this.keyAlpha = {};
    Object.keys(this.byKey).forEach((k) => (this.keyAlpha[k] = 0));

    // Flatten arcs into one line buffer with a per-vertex alpha we drive.
    this.arcs = [];
    Object.entries(this.byKey).forEach(([key, list]) =>
      list.forEach((a) => this.arcs.push({ ...a, key })),
    );
    this._buildLines();
    this._buildPulses();
  }

  _scenarioArcs(group)
  {
    const idx = NODES.map((n, i) => (n.g === group ? i : -1)).filter((i) => i >= 0);
    const hub = idx.find((i) => NODES[i].hub);
    const out = [];
    for (const i of idx)
    {
      if (i === hub) continue;
      out.push(this._arc(i, hub, group));
    }
    for (const i of idx)
    {
      if (i === hub) continue;
      let best = -1, bestD = Infinity;
      const vi = latLonToVec3(NODES[i].lat, NODES[i].lon, 1);
      for (const j of idx)
      {
        if (j === i || j === hub) continue;
        const d = vi.angleTo(latLonToVec3(NODES[j].lat, NODES[j].lon, 1));
        if (d < bestD) { bestD = d; best = j; }
      }
      if (best >= 0 && i < best) out.push(this._arc(i, best, group));
    }
    return out;
  }

  _globalArcs()
  {
    const idx = NODES.map((n, i) => (n.g === 'global' ? i : -1)).filter((i) => i >= 0);
    const hubs = NODES.map((n, i) => (n.hub ? i : -1)).filter((i) => i >= 0);
    const out = [];
    for (const i of idx)
    {
      const vi = latLonToVec3(NODES[i].lat, NODES[i].lon, 1);
      const near = idx
        .filter((j) => j !== i)
        .map((j) => ({ j, d: vi.angleTo(latLonToVec3(NODES[j].lat, NODES[j].lon, 1)) }))
        .sort((a, b) => a.d - b.d)
        .slice(0, 2);
      for (const { j } of near) if (i < j) out.push(this._arc(i, j, 'global'));
    }
    idx.slice(0, hubs.length * 3).forEach((i, k) => out.push(this._arc(i, hubs[k % hubs.length], 'global')));
    return out;
  }

  _buildArcs()
  {
    return {
      global: this._globalArcs(),
      farm: this._scenarioArcs('farm'),
      clinic: this._scenarioArcs('clinic'),
      water: this._scenarioArcs('water'),
      conservation: this._scenarioArcs('conservation'),
      village: this._scenarioArcs('village'),
      storm: this._scenarioArcs('storm'),
    };
  }

  _arc(i, j, group)
  {
    const pts = arcPoints(NODES[i], NODES[j], 46, 0.42);
    return { pts, color: GROUP_COLOR[group] || C.cream, speed: 0.18 + Math.random() * 0.22, offset: Math.random() };
  }

  _buildLines()
  {
    const segs = this.arcs.reduce((s, a) => s + (a.pts.length - 1), 0);
    const pos = new Float32Array(segs * 2 * 3);
    const col = new Float32Array(segs * 2 * 3);
    this.lineAlpha = new Float32Array(segs * 2);
    let p = 0, c = 0, ai = 0;
    this.arcs.forEach((arc) =>
    {
      arc.alphaStart = ai;
      for (let k = 0; k < arc.pts.length - 1; k++)
      {
        const a = arc.pts[k], b = arc.pts[k + 1];
        pos.set([a.x, a.y, a.z, b.x, b.y, b.z], p); p += 6;
        col.set([arc.color.r, arc.color.g, arc.color.b, arc.color.r, arc.color.g, arc.color.b], c); c += 6;
        ai += 2;
      }
      arc.alphaEnd = ai;
    });
    const geo = new BufferGeometry();
    geo.setAttribute('position', new BufferAttribute(pos, 3));
    geo.setAttribute('aColor', new BufferAttribute(col, 3));
    geo.setAttribute('aAlpha', new BufferAttribute(this.lineAlpha, 1));
    this.lineGeo = geo;
    this.lineMat = new ShaderMaterial({
      uniforms: {},
      vertexShader: `attribute vec3 aColor; attribute float aAlpha; varying vec3 vC; varying float vA;
        void main(){ vC=aColor; vA=aAlpha; gl_Position = projectionMatrix*modelViewMatrix*vec4(position,1.0); }`,
      fragmentShader: `precision mediump float; varying vec3 vC; varying float vA;
        void main(){ if(vA<0.01) discard; gl_FragColor = vec4(vC, vA*0.5); }`,
      transparent: true, depthWrite: false, blending: AdditiveBlending,
    });
    this.group.add(new LineSegments(geo, this.lineMat));
  }

  _buildPulses()
  {
    const n = this.arcs.length;
    this.pulsePos = new Float32Array(n * 3);
    const col = new Float32Array(n * 3);
    const phase = new Float32Array(n);
    this.pulseAlpha = new Float32Array(n);
    this.arcs.forEach((arc, i) =>
    {
      col.set([arc.color.r, arc.color.g, arc.color.b], i * 3);
      phase[i] = arc.offset;
    });
    const geo = new BufferGeometry();
    geo.setAttribute('position', new BufferAttribute(this.pulsePos, 3));
    geo.setAttribute('aColor', new BufferAttribute(col, 3));
    geo.setAttribute('aActive', new BufferAttribute(this.pulseAlpha, 1));
    geo.setAttribute('aSize', new BufferAttribute(new Float32Array(n).fill(0.5), 1));
    geo.setAttribute('aPhase', new BufferAttribute(phase, 1));
    this.pulseGeo = geo;
    this.pulseMat = new ShaderMaterial({
      uniforms: { uTime: { value: 0 }, uScale: { value: 1000 } },
      vertexShader: NODE_VERT, fragmentShader: NODE_FRAG,
      transparent: true, depthWrite: false, blending: AdditiveBlending,
    });
    this.group.add(new Points(geo, this.pulseMat));
  }

  setActiveKey(key) { this.activeKey = key; }
  setScale(s) { this.pulseMat.uniforms.uScale.value = s; }

  update(t, dt)
  {
    this.pulseMat.uniforms.uTime.value = t;
    const k = 1 - Math.exp(-dt * 2.6);
    let lineDirty = false;
    Object.keys(this.keyAlpha).forEach((key) =>
    {
      const tgt = key === this.activeKey ? 1 : 0;
      const before = this.keyAlpha[key];
      this.keyAlpha[key] += (tgt - before) * k;
      if (Math.abs(this.keyAlpha[key] - before) > 1e-4) lineDirty = true;
    });
    if (lineDirty)
    {
      this.arcs.forEach((arc) =>
      {
        const a = this.keyAlpha[arc.key];
        for (let s = arc.alphaStart; s < arc.alphaEnd; s++) this.lineAlpha[s] = a;
      });
      this.lineGeo.attributes.aAlpha.needsUpdate = true;
    }
    // March each pulse along its arc toward the gateway (last point).
    this.arcs.forEach((arc, i) =>
    {
      const a = this.keyAlpha[arc.key];
      this.pulseAlpha[i] = a;
      const u = (t * arc.speed + arc.offset) % 1;
      const f = u * (arc.pts.length - 1);
      const i0 = Math.floor(f);
      const i1 = Math.min(arc.pts.length - 1, i0 + 1);
      const fr = f - i0;
      const p0 = arc.pts[i0], p1 = arc.pts[i1];
      this.pulsePos[i * 3] = p0.x + (p1.x - p0.x) * fr;
      this.pulsePos[i * 3 + 1] = p0.y + (p1.y - p0.y) * fr;
      this.pulsePos[i * 3 + 2] = p0.z + (p1.z - p0.z) * fr;
    });
    this.pulseGeo.attributes.position.needsUpdate = true;
    this.pulseGeo.attributes.aActive.needsUpdate = true;
  }
}

class Storm
{
  constructor(lat, lon)
  {
    this.group = new Group();
    this.amount = 0;
    this.anchor = latLonToVec3(lat, lon, GLOBE.radius);

    const normal = this.anchor.clone().normalize();
    const tangent = new Vector3(0, 1, 0).cross(normal).normalize();
    const bitangent = normal.clone().cross(tangent).normalize();
    this.basis = new Matrix4().makeBasis(tangent, bitangent, normal);
    this.basis.setPosition(this.anchor);

    this._buildSpiral();
    this._buildUplink(lat, lon);
  }

  _buildSpiral()
  {
    const N = 1500;
    const pos = new Float32Array(N * 3);
    const a0 = new Float32Array(N); // angle
    const rad = new Float32Array(N);
    const ph = new Float32Array(N);
    const arms = 5;
    for (let i = 0; i < N; i++)
    {
      const t = i / N;
      const arm = (i % arms) / arms;
      const r = 0.05 + Math.pow(t, 0.7) * 0.34;
      const swirl = t * 9.0 + arm * Math.PI * 2;
      a0[i] = swirl;
      rad[i] = r + (Math.random() - 0.5) * 0.03;
      ph[i] = Math.random();
      pos.set([0, 0, 0], i * 3);
    }
    this.spiralA = a0; this.spiralR = rad;
    const geo = new BufferGeometry();
    geo.setAttribute('position', new BufferAttribute(pos, 3));
    geo.setAttribute('aPhase', new BufferAttribute(ph, 1));
    this.spiralGeo = geo;
    this.spiralMat = new ShaderMaterial({
      uniforms: { uTime: { value: 0 }, uScale: { value: 1000 }, uAmount: { value: 0 }, uFlash: { value: 0 } },
      vertexShader: `
        uniform float uTime; uniform float uScale; uniform float uAmount;
        attribute float aPhase; varying float vA;
        void main(){
          vec4 mv = modelViewMatrix * vec4(position, 1.0);
          float tw = 0.6 + 0.4*sin(uTime*3.0 + aPhase*6.283);
          vA = uAmount * (0.4 + 0.6*tw);
          gl_PointSize = (0.05 + 0.05*aPhase) * uScale / -mv.z;
          gl_Position = projectionMatrix * mv;
        }`,
      fragmentShader: `
        precision mediump float; uniform float uFlash; varying float vA;
        void main(){
          vec2 d = gl_PointCoord - vec2(0.5); float r = length(d); if(r>0.5) discard;
          float c = smoothstep(0.5, 0.0, r);
          vec3 col = mix(vec3(0.5,0.58,0.69), vec3(0.82), uFlash);
          gl_FragColor = vec4(col, c * vA * 0.26);
        }`,
      transparent: true, depthWrite: false, blending: AdditiveBlending,
    });
    const pts = new Points(geo, this.spiralMat);
    pts.matrixAutoUpdate = false;
    pts.matrix.copy(this.basis);
    this.group.add(pts);
  }

  _buildUplink(lat, lon)
  {
    const base = latLonToVec3(lat + 2.4, lon - 4.5, GLOBE.radius * 1.012);
    this.sat = base.clone().multiplyScalar(1.62);
    const pos = new Float32Array([base.x, base.y, base.z, this.sat.x, this.sat.y, this.sat.z]);
    const geo = new BufferGeometry();
    geo.setAttribute('position', new BufferAttribute(pos, 3));
    this.beamMat = new ShaderMaterial({
      uniforms: { uAmount: { value: 0 } },
      vertexShader: `void main(){ gl_Position = projectionMatrix*modelViewMatrix*vec4(position,1.0); }`,
      fragmentShader: `precision mediump float; uniform float uAmount;
        void main(){ gl_FragColor = vec4(1.0, 0.85, 0.55, 0.5*uAmount); }`,
      transparent: true, depthWrite: false, blending: AdditiveBlending,
    });
    this.group.add(new LineSegments(geo, this.beamMat));

    this.beamBase = base;
    const sgeo = new BufferGeometry();
    sgeo.setAttribute('position', new BufferAttribute(new Float32Array([this.sat.x, this.sat.y, this.sat.z]), 3));
    sgeo.setAttribute('aColor', new BufferAttribute(new Float32Array([1, 0.85, 0.55]), 3));
    sgeo.setAttribute('aActive', new BufferAttribute(new Float32Array([0]), 1));
    sgeo.setAttribute('aSize', new BufferAttribute(new Float32Array([1.3]), 1));
    sgeo.setAttribute('aPhase', new BufferAttribute(new Float32Array([0]), 1));
    this.satGeo = sgeo;
    this.satMat = new ShaderMaterial({
      uniforms: { uTime: { value: 0 }, uScale: { value: 1000 } },
      vertexShader: NODE_VERT, fragmentShader: NODE_FRAG,
      transparent: true, depthWrite: false, blending: AdditiveBlending,
    });
    this.group.add(new Points(sgeo, this.satMat));

    // A single pulse climbing the beam.
    const pgeo = new BufferGeometry();
    this.climbPos = new Float32Array(3);
    pgeo.setAttribute('position', new BufferAttribute(this.climbPos, 3));
    pgeo.setAttribute('aColor', new BufferAttribute(new Float32Array([1, 0.9, 0.7]), 3));
    pgeo.setAttribute('aActive', new BufferAttribute(new Float32Array([0]), 1));
    pgeo.setAttribute('aSize', new BufferAttribute(new Float32Array([0.7]), 1));
    pgeo.setAttribute('aPhase', new BufferAttribute(new Float32Array([0]), 1));
    this.climbGeo = pgeo;
    this.group.add(new Points(pgeo, this.satMat));
  }

  setScale(s) { this.spiralMat.uniforms.uScale.value = s; this.satMat.uniforms.uScale.value = s; }

  setAmount(a) { this.amount = a; }

  update(t, dt)
  {
    const easeK = 1 - Math.exp(-dt * 2.5);
    this.spiralMat.uniforms.uAmount.value += (this.amount - this.spiralMat.uniforms.uAmount.value) * easeK;
    this.beamMat.uniforms.uAmount.value = this.spiralMat.uniforms.uAmount.value;
    this.satMat.uniforms.uTime.value = t;

    const flash = Math.max(0, Math.sin(t * 1.3) > 0.985 ? 1 : 0, this.spiralMat.uniforms.uFlash.value - dt * 6);
    this.spiralMat.uniforms.uFlash.value = flash * this.amount * 0.5;

    if (this.spiralMat.uniforms.uAmount.value < 0.01) { this.group.visible = false; return; }
    this.group.visible = true;

    const spin = t * 0.5;
    const pos = this.spiralGeo.attributes.position.array;
    const local = new Vector3();
    for (let i = 0; i < this.spiralA.length; i++)
    {
      const ang = this.spiralA[i] + spin;
      const r = this.spiralR[i];
      const lift = (0.5 - r) * 0.12;
      local.set(Math.cos(ang) * r, Math.sin(ang) * r, lift).applyMatrix4(this.basis);
      pos[i * 3] = local.x; pos[i * 3 + 1] = local.y; pos[i * 3 + 2] = local.z;
    }
    this.spiralGeo.attributes.position.needsUpdate = true;

    this.satGeo.attributes.aActive.array[0] = this.spiralMat.uniforms.uAmount.value;
    this.satGeo.attributes.aActive.needsUpdate = true;
    const u = (t * 0.4) % 1;
    this.climbPos[0] = this.beamBase.x + (this.sat.x - this.beamBase.x) * u;
    this.climbPos[1] = this.beamBase.y + (this.sat.y - this.beamBase.y) * u;
    this.climbPos[2] = this.beamBase.z + (this.sat.z - this.beamBase.z) * u;
    this.climbGeo.attributes.aActive.array[0] = this.spiralMat.uniforms.uAmount.value;
    this.climbGeo.attributes.position.needsUpdate = true;
    this.climbGeo.attributes.aActive.needsUpdate = true;
  }
}

export class Network
{
  constructor(stormLat, stormLon)
  {
    this.group = new Group();
    this.nodes = new NodeField();
    this.arcs = new ArcNetwork();
    this.storm = new Storm(stormLat, stormLon);
    this.group.add(this.arcs.group, this.nodes.group, this.storm.group);
  }

  setScene(scene)
  {
    this.nodes.setFocus(scene.focus);
    this.arcs.setActiveKey(scene.arcsKey);
    this.storm.setAmount(scene.storm);
  }

  setScale(s)
  {
    this.nodes.setScale(s);
    this.arcs.setScale(s);
    this.storm.setScale(s);
  }

  update(t, dt)
  {
    this.nodes.update(t, dt);
    this.arcs.update(t, dt);
    this.storm.update(t, dt);
  }
}
