import { Scene, PerspectiveCamera, WebGLRenderer, Group, Color, MathUtils } from 'three';
import { CAM, GLOBE, SCENES, isMobile } from './config.js';
import { loadLandMask } from './geo.js';
import { Globe } from './globe.js';
import { Network } from './network.js';
import { Director } from './scenes.js';
import { initUI, setAccent, showToast } from './ui.js';

const canvas = document.getElementById('globe-canvas');
const loader = document.getElementById('loader');
const fovRad = MathUtils.degToRad(CAM.fov);

function hideLoader() { loader.classList.add('gone'); setTimeout(() => (loader.style.display = 'none'), 800); }

function fail(reason) {
  console.warn('[pamoja] running without the 3D stage:', reason);
  document.body.classList.add('no-webgl');
  hideLoader();
  // The page (including the device consoles) stays fully usable without WebGL.
  initUI({ onScene: (name) => setAccent(SCENES[name] && SCENES[name].theme) });
}

async function boot() {
  let renderer;
  try {
    renderer = new WebGLRenderer({ canvas, antialias: true, powerPreference: 'high-performance' });
  } catch (err) {
    return fail(err.message);
  }
  renderer.setPixelRatio(Math.min(2, window.devicePixelRatio || 1));
  renderer.setSize(window.innerWidth, window.innerHeight, false);
  renderer.setClearColor(new Color('#0a1322'), 1);

  const scene = new Scene();
  const camera = new PerspectiveCamera(CAM.fov, window.innerWidth / window.innerHeight, CAM.near, CAM.far);
  camera.position.set(0, 0, CAM.homeDistance);

  const world = new Group();
  scene.add(world);

  const mask = await loadLandMask(1024);
  const count = isMobile ? GLOBE.dotCountMobile : GLOBE.dotCount;
  const globe = new Globe(mask, count);
  const network = new Network(12, 125);
  world.add(globe.group, network.group);

  const director = new Director(camera, world, globe, network);
  director.onTheme = (theme) => setAccent(theme);

  const applyScale = () => {
    const s = renderer.domElement.height / (2 * Math.tan(fovRad / 2));
    globe.setScale(s);
    network.setScale(s);
  };

  const onResize = () => {
    camera.aspect = window.innerWidth / window.innerHeight;
    camera.updateProjectionMatrix();
    renderer.setSize(window.innerWidth, window.innerHeight, false);
    applyScale();
  };
  window.addEventListener('resize', onResize, { passive: true });
  applyScale();

  if (!isMobile && !window.matchMedia('(prefers-reduced-motion: reduce)').matches) {
    window.addEventListener('pointermove', (e) => {
      director.setPointer((e.clientX / window.innerWidth) * 2 - 1, -((e.clientY / window.innerHeight) * 2 - 1));
    }, { passive: true });
  }

  initUI({ onScene: (name) => director.setScene(name) });

  let last = performance.now();
  let started = false;
  const loop = (now) => {
    const dt = Math.min(0.05, (now - last) / 1000);
    last = now;
    director.update(now / 1000, dt);
    renderer.render(scene, camera);
    if (!started) { started = true; hideLoader(); }
    requestAnimationFrame(loop);
  };
  requestAnimationFrame(loop);

  if (mask.fallback) showToast('Offline map: showing a stylised globe.');
}

boot().catch((err) => fail(err.message));
