# pamoja showcase

An interactive, single-page showcase for pamoja: a live WebGL globe where the
mesh wires the world together, six field scenarios each shown as the live
telemetry console a pamoja node serves (smallholder farm, rural clinic, clean
water, conservation, off-grid village mesh, hurricane relay), the capability
crates fanning out from the core, the same code in every language, a platform
roadmap, and a way for donors and vendors to fund hardware kits.

Built with plain ES modules and Three.js. No build step, no framework, no
bundler. It is just static files.

## Run it

ES modules cannot load from `file://` (the browser blocks them as cross-origin),
so the page must be served over HTTP. The bundled launcher is easiest:

```sh
node serve.mjs            # opens http://localhost:8099 in your browser
node serve.mjs 5050       # custom port
node serve.mjs --no-open  # do not auto-open
```

On Windows you can just double-click `dev.bat`. The launcher has **live reload**:
editing any file refreshes the open tab automatically, so you can iterate with no
build step.

Plain alternatives: `python -m http.server 8099`, or `npx serve`.

Opening `index.html` directly will only show the loader and a CORS error in the
console. That is expected; use the server.

## Deploy it

The whole `web/` folder is the site. Drag it onto Netlify, push it to GitHub
Pages, or upload it to any static host. No server-side code runs. The pledge form
composes a `mailto:` to the maintainer; wire in a real handler when ready.

## How it is built

```
index.html      structure + scrollytelling sections + import map
styles.css      all styling; --accent shifts with the active scene
serve.mjs       zero-dependency static server with live reload
dev.bat         double-click launcher for Windows
assets/         brand mark + logo
js/
  config.js     palette, tunables, and the ordered scene list
  data.js       nodes, crates, languages, tiers, roadmap tracks (the content)
  geo.js        lat/lon math, great-circle arcs, the coastline land mask
  globe.js      the dotted Earth, atmosphere, occluding inner sphere
  network.js    device nodes, mesh arcs with pulses, the hurricane swirl
  consoles.js   the live telemetry console (dashboard) per field scenario
  scenes.js     the director that eases the camera and mood per scene
  ui.js         scroll observer, constellation, language tabs, tracks, the form
  main.js       boot, render loop, resize, WebGL + reduced-motion fallback
```

Scroll position drives everything. Each section carries a `data-scene`; an
IntersectionObserver hands the active one to the director, which eases the
camera, the orientation onto a region, the colour theme, and which nodes light up.

Each field scenario is rendered as the **live telemetry console** that node
serves: animated gauges, bars and sparklines, a status/link/battery header, a
mesh or signal-flow schematic with travelling packets, a tamper-evident
hash-chained log, an acoustic monitor, and a scrolling event log. Each console
runs a short scripted timeline so the data tells the scenario's story (soil drops
then the valve opens; the clinic link drops then store-and-forward catches up; the
storm takes the towers and the mesh relays to a satellite). Consoles are pure DOM
and SVG, driven by one light rAF loop that only animates the console in view.

### Loaded from a CDN at runtime

- `three@0.169` and `topojson-client@3` via the import map (jsDelivr).
- Natural Earth land outline (`world-atlas` land-110m) for the dotted globe; a
  stylised fallback is used if it is unreachable.
- Google Fonts, degrading to system fonts offline.

If WebGL is unavailable the canvas is hidden and the page stays readable.
`prefers-reduced-motion` freezes the idle spin and pointer parallax.
