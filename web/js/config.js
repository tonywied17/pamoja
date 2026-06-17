import { Color } from 'three';

export const PALETTE = {
  navyDeep: '#0a1322',
  navy: '#0e1b2e',
  navyLift: '#1b2d49',
  amber: '#ffb627',
  coral: '#f26a4b',
  teal: '#1fa995',
  sky: '#36b6dd',
  forest: '#46c97e',
  cream: '#fbf3e4',
  ink: '#06101d',
};

export const C = {
  amber: new Color(PALETTE.amber),
  coral: new Color(PALETTE.coral),
  teal: new Color(PALETTE.teal),
  sky: new Color(PALETTE.sky),
  forest: new Color(PALETTE.forest),
  cream: new Color(PALETTE.cream),
  navy: new Color(PALETTE.navy),
  navyLift: new Color(PALETTE.navyLift),
};

export const GLOBE = {
  radius: 1,
  dotCount: 26000,
  dotCountMobile: 9000,
  atmosphere: 1.16,
};

export const CAM = {
  fov: 38,
  near: 0.1,
  far: 100,
  homeDistance: 3.25,
};

export const SCENES = {
  hero: {
    lat: 6, lon: 24, tilt: 0.05,
    distance: 3.25, height: 0.0,
    idleSpin: 5.0, theme: 'teal', dim: 0, storm: 0, focus: null, arcsKey: 'global',
  },
  farm: {
    lat: -2, lon: 37, tilt: 0.14,
    distance: 2.9, height: 0.05,
    idleSpin: 1.3, theme: 'teal', dim: 0.5, storm: 0, focus: 'farm', arcsKey: 'farm',
  },
  clinic: {
    lat: 9, lon: 18, tilt: 0.14,
    distance: 2.9, height: 0.05,
    idleSpin: 1.3, theme: 'amber', dim: 0.5, storm: 0, focus: 'clinic', arcsKey: 'clinic',
  },
  water: {
    lat: 23, lon: 76, tilt: 0.14,
    distance: 2.85, height: 0.05,
    idleSpin: 1.3, theme: 'sky', dim: 0.5, storm: 0, focus: 'water', arcsKey: 'water',
  },
  conservation: {
    lat: -16, lon: 27, tilt: 0.14,
    distance: 2.85, height: 0.05,
    idleSpin: 1.3, theme: 'forest', dim: 0.5, storm: 0, focus: 'conservation', arcsKey: 'conservation',
  },
  village: {
    lat: 27, lon: 85, tilt: 0.14,
    distance: 2.8, height: 0.06,
    idleSpin: 1.3, theme: 'teal', dim: 0.5, storm: 0, focus: 'village', arcsKey: 'village',
  },
  storm: {
    lat: 12, lon: 125, tilt: 0.16,
    distance: 2.75, height: 0.06,
    idleSpin: 0.8, theme: 'coral', dim: 0.6, storm: 1, focus: 'storm', arcsKey: 'storm',
  },
  crates: {
    lat: 8, lon: -40, tilt: 0.05,
    distance: 4.6, height: 0.0,
    idleSpin: 7.0, theme: 'amber', dim: 0.82, storm: 0, focus: null, arcsKey: 'global',
  },
  languages: {
    lat: 20, lon: -80, tilt: 0.05,
    distance: 5.2, height: 0.0,
    idleSpin: 7.0, theme: 'teal', dim: 0.86, storm: 0, focus: null, arcsKey: 'global',
  },
  roadmap: {
    lat: 12, lon: 60, tilt: 0.05,
    distance: 4.3, height: 0.0,
    idleSpin: 6.0, theme: 'coral', dim: 0.8, storm: 0, focus: null, arcsKey: 'global',
  },
  back: {
    lat: 0, lon: -20, tilt: 0.05,
    distance: 3.6, height: 0.0,
    idleSpin: 4.0, theme: 'amber', dim: 0.35, storm: 0, focus: null, arcsKey: 'global',
  },
};

export const THEME_COLOR = {
  teal: C.teal,
  amber: C.amber,
  coral: C.coral,
  sky: C.sky,
  forest: C.forest,
};

export const prefersReducedMotion =
  typeof window !== 'undefined' &&
  window.matchMedia &&
  window.matchMedia('(prefers-reduced-motion: reduce)').matches;

export const isMobile =
  typeof window !== 'undefined' &&
  window.matchMedia &&
  window.matchMedia('(max-width: 820px)').matches;
