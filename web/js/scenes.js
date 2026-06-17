import { Quaternion, Vector3, MathUtils } from 'three';
import { SCENES, THEME_COLOR, CAM, prefersReducedMotion } from './config.js';
import { orientationFor } from './geo.js';

const damp = (cur, tgt, rate, dt) => MathUtils.lerp(cur, tgt, 1 - Math.exp(-rate * dt));

export class Director
{
  constructor(camera, world, globe, network)
  {
    this.camera = camera;
    this.world = world;
    this.globe = globe;
    this.network = network;

    this.scene = SCENES.hero;
    this.name = 'hero';

    this.distance = this.scene.distance;
    this.height = this.scene.height;
    this.dim = 0;
    this.spinRate = this.scene.idleSpin;
    this.localSpin = 0;

    this.quat = new Quaternion();
    this.targetQuat = new Quaternion();
    orientationFor(this.scene.lat, this.scene.lon, this.scene.tilt, this.quat);

    this.pointer = new Vector3();
    this.onTheme = null;
  }

  setScene(name)
  {
    if (!SCENES[name] || name === this.name) return;
    this.name = name;
    this.scene = SCENES[name];
    this.localSpin = 0;
    this.network.setScene(this.scene);
    if (this.onTheme) this.onTheme(this.scene.theme);
  }

  setPointer(nx, ny)
  {
    this.pointer.set(nx, ny, 0);
  }

  update(t, dt)
  {
    const s = this.scene;
    this.distance = damp(this.distance, s.distance, 2.2, dt);
    this.height = damp(this.height, s.height, 2.2, dt);
    this.dim = damp(this.dim, s.dim, 2.4, dt);
    this.spinRate = damp(this.spinRate, s.idleSpin, 1.6, dt);
    this.localSpin += this.spinRate * dt * (prefersReducedMotion ? 0 : 1);

    orientationFor(s.lat, s.lon + this.localSpin, s.tilt, this.targetQuat);
    const slerpRate = prefersReducedMotion ? 1 : 1 - Math.exp(-2.0 * dt);
    this.quat.slerp(this.targetQuat, slerpRate);
    this.world.quaternion.copy(this.quat);

    const px = this.pointer.x * 0.05;
    const py = this.pointer.y * 0.035;
    this.camera.position.set(px, this.height + py, this.distance);
    this.camera.lookAt(0, this.height * 0.25, 0);

    this.globe.setTheme(THEME_COLOR[s.theme]);
    this.globe.setDim(this.dim);
    this.globe.update(t);
    this.network.update(t, dt);
  }
}

export { CAM };
