import
  {
    Group, Points, BufferGeometry, BufferAttribute, ShaderMaterial, AdditiveBlending,
    Mesh, SphereGeometry, BackSide, Color, Vector3, MathUtils,
  } from 'three';
import { GLOBE, C } from './config.js';
import { vec3ToLatLon } from './geo.js';

const POINT_VERT = `
  uniform float uTime;
  uniform float uSize;
  uniform float uScale; // drawingBufferHeight / (2 tan(fov/2)), set on resize
  attribute float aPhase;
  varying float vTwinkle;
  varying float vFacing;
  varying float vPhase;
  void main() {
    vec4 mv = modelViewMatrix * vec4(position, 1.0);
    vec3 viewNormal = normalize(normalMatrix * normalize(position));
    vFacing = clamp(dot(viewNormal, normalize(-mv.xyz)), 0.0, 1.0);
    vTwinkle = 0.62 + 0.38 * sin(uTime * 1.7 + aPhase * 6.2831853);
    vPhase = aPhase;
    float size = uSize * (1.0 + 0.5 * vTwinkle) * (0.28 + 0.72 * vFacing);
    gl_PointSize = size * uScale / -mv.z;
    gl_Position = projectionMatrix * mv;
  }
`;

const POINT_FRAG = `
  precision mediump float;
  uniform vec3 uColorA;
  uniform vec3 uColorB;
  uniform vec3 uTheme;
  uniform float uDim;
  uniform float uOpacity;
  varying float vTwinkle;
  varying float vFacing;
  varying float vPhase;
  void main() {
    vec2 d = gl_PointCoord - vec2(0.5);
    float r = length(d);
    if (r > 0.5) discard;
    float core = smoothstep(0.5, 0.0, r);
    vec3 base = mix(uColorA, uColorB, vPhase);
    vec3 col = mix(base, uTheme, 0.32 + 0.34 * vTwinkle);
    // Fade dots to nothing at the silhouette so foreshortened land does not pile
    // into a bright ring around the globe's edge.
    float limb = smoothstep(0.04, 0.4, vFacing);
    float a = core * uOpacity * limb * (0.68 + 0.42 * vTwinkle) * (1.0 - 0.72 * uDim);
    gl_FragColor = vec4(col * 1.12, a);
  }
`;

const ATMO_VERT = `
  varying vec3 vNormal;
  varying vec3 vView;
  void main() {
    vNormal = normalize(normalMatrix * normal);
    vec4 mv = modelViewMatrix * vec4(position, 1.0);
    vView = normalize(-mv.xyz);
    gl_Position = projectionMatrix * mv;
  }
`;

const ATMO_FRAG = `
  precision mediump float;
  uniform vec3 uTheme;
  uniform float uIntensity;
  varying vec3 vNormal;
  varying vec3 vView;
  void main() {
    float f = pow(1.0 - abs(dot(vNormal, vView)), 3.2);
    gl_FragColor = vec4(uTheme, f * uIntensity);
  }
`;

export class Globe
{
  constructor(mask, count)
  {
    this.group = new Group();
    this.mask = mask;
    this._buildInner();
    this._buildDots(count);
    this._buildAtmosphere();
  }

  _buildInner()
  {
    const geo = new SphereGeometry(GLOBE.radius * 0.992, 64, 48);
    const mat = new ShaderMaterial({
      uniforms: { uTop: { value: new Color('#16263f') }, uBottom: { value: new Color('#091625') } },
      vertexShader: `varying float vY; void main(){ vY = normalize(position).y; gl_Position = projectionMatrix * modelViewMatrix * vec4(position,1.0); }`,
      fragmentShader: `precision mediump float; uniform vec3 uTop; uniform vec3 uBottom; varying float vY;
        void main(){ vec3 c = mix(uBottom, uTop, smoothstep(-0.8, 0.8, vY)); gl_FragColor = vec4(c, 1.0); }`,
    });
    this.inner = new Mesh(geo, mat);
    this.group.add(this.inner);
  }

  _buildDots(count)
  {
    const positions = [];
    const phases = [];
    const r = GLOBE.radius * 1.002;
    const golden = Math.PI * (3 - Math.sqrt(5));
    const v = new Vector3();
    for (let i = 0; i < count; i++)
    {
      const y = 1 - (i / (count - 1)) * 2;
      const rad = Math.sqrt(1 - y * y);
      const theta = golden * i;
      v.set(Math.cos(theta) * rad, y, Math.sin(theta) * rad);
      const { lat, lon } = vec3ToLatLon(v);
      if (lat < -58) continue;
      if (this.mask.sample(lat, lon) < 90) continue;
      positions.push(v.x * r, v.y * r, v.z * r);
      phases.push(Math.random());
    }
    const geo = new BufferGeometry();
    geo.setAttribute('position', new BufferAttribute(new Float32Array(positions), 3));
    geo.setAttribute('aPhase', new BufferAttribute(new Float32Array(phases), 1));

    this.dotMat = new ShaderMaterial({
      uniforms: {
        uTime: { value: 0 },
        uSize: { value: 0.0062 },
        uScale: { value: 1000 },
        uColorA: { value: C.teal.clone() },
        uColorB: { value: C.cream.clone() },
        uTheme: { value: C.teal.clone() },
        uDim: { value: 0 },
        uOpacity: { value: 1 },
      },
      vertexShader: POINT_VERT,
      fragmentShader: POINT_FRAG,
      transparent: true,
      depthWrite: false,
    });
    this.dots = new Points(geo, this.dotMat);
    this.landCount = positions.length / 3;
    this.group.add(this.dots);
  }

  _buildAtmosphere()
  {
    const geo = new SphereGeometry(GLOBE.atmosphere, 48, 32);
    this.atmoMat = new ShaderMaterial({
      uniforms: { uTheme: { value: C.teal.clone() }, uIntensity: { value: 1.05 } },
      vertexShader: ATMO_VERT,
      fragmentShader: ATMO_FRAG,
      transparent: true,
      blending: AdditiveBlending,
      side: BackSide,
      depthWrite: false,
    });
    this.atmosphere = new Mesh(geo, this.atmoMat);
    this.group.add(this.atmosphere);
  }

  setScale(scale)
  {
    this.dotMat.uniforms.uScale.value = scale;
  }

  setTheme(color)
  {
    this.dotMat.uniforms.uTheme.value.lerp(color, 0.08);
    this.atmoMat.uniforms.uTheme.value.lerp(color, 0.08);
  }

  setDim(dim)
  {
    this.dotMat.uniforms.uDim.value = MathUtils.lerp(this.dotMat.uniforms.uDim.value, dim, 0.08);
    this.atmoMat.uniforms.uIntensity.value = MathUtils.lerp(
      this.atmoMat.uniforms.uIntensity.value, 1.05 - dim * 0.4, 0.08,
    );
  }

  update(t)
  {
    this.dotMat.uniforms.uTime.value = t;
  }
}
