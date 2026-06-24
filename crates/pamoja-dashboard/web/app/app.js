// app.js - application entry point.
//
// Bootstraps the dashboard: applies the saved theme, loads localization, registers the
// components, mounts the router (hash mode, org-addressable), opens the live fleet
// stream, and starts the boot reveal and pointer parallax. The page is a multi-file
// zQuery app - the core files (this entry, the store, the routes, and the overlay nav)
// live in app/; feature and helper modules live under app/lib/.

import { store } from './store.js';
import { initI18n, t } from './lib/i18n.js';
import { initNav, back } from './nav.js';
import { connectFeed, connected, fleet } from './lib/feed.js';
import { initParallax } from './lib/parallax.js';
import { routes } from './routes.js';
import './components/top-bar.js';
import './components/dashboard-page.js';
import './components/sensor-modal.js';
import './components/manage-modal.js';
import './components/pairing-modal.js';
import './components/group-modal.js';
import './components/mesh-modal.js';
import './components/network-view.js';
import './components/alarm-bar.js';

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

/** Removes the boot splash once the app is mounted and the first fleet frame has arrived. */
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
  initParallax();
  document.addEventListener('keydown', (e) => { if (e.key === 'Escape') back(); });
});
