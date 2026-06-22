// app.js - application entry point.
//
// Bootstraps the dashboard: applies the saved theme, loads localization, registers the
// components, mounts the router (hash mode, org-addressable), opens the live fleet
// stream, and starts the starfield background and parallax. The page is a multi-file
// zQuery app - each concern lives in its own module under app/.

import { store } from './store.js';
import { initI18n, t } from './i18n.js';
import { initNav, back } from './nav.js';
import { connectFeed, connected, fleet } from './feed.js';
import './components/top-bar.js';
import './components/dashboard-page.js';
import './components/sensor-modal.js';
import './components/manage-modal.js';
import './components/group-modal.js';
import './components/mesh-modal.js';
import './components/network-view.js';
import './components/alarm-bar.js';
import { routes } from './routes.js';

await initI18n();
document.documentElement.dataset.theme = store.state.theme;

const router = $.router({ routes, mode: 'hash', fallback: 'dashboard-page' });
connectFeed();

$.effect(() =>
{
  const tag = document.getElementById('offline-tag');
  if (tag) { tag.hidden = connected.value; tag.textContent = t('ui.disconnected'); }
});

let mounted = false, revealed = false;
function reveal()
{
  if (revealed || !mounted || fleet.value == null) return;
  revealed = true;
  const boot = document.getElementById('boot');
  if (boot) { boot.classList.add('gone'); setTimeout(() => boot.remove(), 600); }
}
$.effect(() => { fleet.value; reveal(); });
setTimeout(() => { revealed = true; const b = document.getElementById('boot'); if (b) { b.classList.add('gone'); setTimeout(() => b.remove(), 600); } }, 1500);

$.ready(() =>
{
  initNav(router);
  $.mountAll();
  mounted = true;
  reveal();
  document.addEventListener('pointermove', parallax);
  document.addEventListener('keydown', (e) => { if (e.key === 'Escape') back(); });
});

// --- parallax tilt on the group cards -------------------------------------
let hoverEl = null, amt = 5, glow = false, tx = 0, ty = 0, mx = 50, loop = 0;
function clearHover()
{
  if (hoverEl) { hoverEl.style.removeProperty('--rx'); hoverEl.style.removeProperty('--ry'); }
  hoverEl = null;
}
function applyTilt()
{
  if (!hoverEl) { loop = 0; return; }
  hoverEl.style.setProperty('--rx', tx.toFixed(2) + 'deg');
  hoverEl.style.setProperty('--ry', ty.toFixed(2) + 'deg');
  if (glow) hoverEl.style.setProperty('--mx', mx.toFixed(0) + '%');
  loop = requestAnimationFrame(applyTilt);
}
function parallax(e)
{
  const t = e.target.closest ? e.target : null;
  const modal = t && t.closest('.modal');
  const card = !modal && t ? t.closest('.gcard') : null;
  const el = modal || card;
  if (!el) { clearHover(); return; }
  if (el !== hoverEl) { clearHover(); hoverEl = el; amt = modal ? 3 : 5; glow = !!card; }
  const b = el.getBoundingClientRect();
  const px = (e.clientX - b.left) / b.width - 0.5, py = (e.clientY - b.top) / b.height - 0.5;
  tx = px * amt; ty = -py * amt; mx = px * 100 + 50;
  if (!loop) loop = requestAnimationFrame(applyTilt);
}

