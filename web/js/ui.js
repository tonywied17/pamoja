import
{
  STATS, CRATES, PLANNED_CRATES, SCENARIO_CRATES, LANGUAGES, PLANNED_LANGS, TIERS, UPLINKS, TRACKS,
  ROBOTS, AREA_ORDER, areaOf, packagesFor,
} from './data.js';
import { prefersReducedMotion } from './config.js';
import { mountConsoles } from './consoles.js';
import { mountCardFx } from './cardfx.js';
import { crateVersion } from './crateversions.js';

const $ = (sel, root = document) => root.querySelector(sel);
const $$ = (sel, root = document) => [...root.querySelectorAll(sel)];
const el = (tag, cls, html) =>
{
  const n = document.createElement(tag);
  if (cls) n.className = cls;
  if (html != null) n.innerHTML = html;
  return n;
};
const crateById = Object.fromEntries([...CRATES, ...PLANNED_CRATES].map((c) => [c.id, c]));
const chip = (id) => `pamoja-<b>${id.replace('pamoja-', '')}</b>`;

const PACKAGES = [
  { label: 'crates.io', href: 'https://crates.io/users/tonywied17' },
  { label: '@pamoja/core', href: 'https://www.npmjs.com/org/pamoja' },
  { label: 'PyPI', href: 'https://pypi.org/user/tonywied17/' },
  { label: 'NuGet', href: 'https://www.nuget.org/profiles/tonywied17' },
  { label: 'GitHub', href: 'https://github.com/molexxxx/pamoja' },
];

export function setAccent(theme)
{
  const map = {
    teal: 'var(--teal)', amber: 'var(--amber)', coral: 'var(--coral)',
    sky: '#36b6dd', forest: '#46c97e',
  };
  document.documentElement.style.setProperty('--accent', map[theme] || 'var(--teal)');
}

let toastTimer;
export function showToast(msg)
{
  const t = $('#toast');
  t.textContent = msg;
  t.classList.add('show');
  clearTimeout(toastTimer);
  toastTimer = setTimeout(() => t.classList.remove('show'), 4200);
}

const PREVIEW_MSG = 'Backing is not open yet - this section is a preview of how it will work.';
function comingSoon()
{
  $('#back').scrollIntoView({ behavior: prefersReducedMotion ? 'auto' : 'smooth' });
  showToast(PREVIEW_MSG);
}

// ---- builders ----------------------------------------------------------

function buildStats()
{
  const row = $('#stat-row');
  STATS.forEach((s) =>
  {
    const li = el('li');
    li.innerHTML = `<span class="num" data-to="${s.value}">${s.prefix || ''}0${s.suffix || ''}</span><span class="lbl">${s.label}</span>`;
    row.appendChild(li);
  });
}

function countUp()
{
  $$('#stat-row .num').forEach((node) =>
  {
    const to = +node.dataset.to || 0;
    const prefix = node.textContent.startsWith('$') ? '$' : '';
    if (prefersReducedMotion) { node.textContent = `${prefix}${to}`; return; }
    let start = null;
    const dur = 1300;
    const tick = (now) =>
    {
      // Seed the clock from the first frame's own timestamp; clamp to [0,1] so the count
      // never reads negative if that timestamp predates a separately sampled start.
      if (start === null) start = now;
      const p = Math.max(0, Math.min(1, (now - start) / dur));
      const e = 1 - Math.pow(1 - p, 3);
      node.textContent = `${prefix}${Math.round(to * e)}`;
      if (p < 1) requestAnimationFrame(tick);
    };
    requestAnimationFrame(tick);
  });
}

function buildScenarioChips()
{
  $$('.crate-chips[data-crates]').forEach((box) =>
  {
    const list = SCENARIO_CRATES[box.dataset.crates] || [];
    list.forEach((id) => box.appendChild(el('span', 'crate-chip', chip(id))));
  });
}

function renderRoboCard(r)
{
  const card = $('#robo-card');
  if (!card) return;
  card.innerHTML = `<p class="eyebrow ${r.eyebrowClass}">${r.eyebrow}</p><h3>${r.h3}</h3><p>${r.body}</p>`;
  const chips = el('div', 'crate-chips');
  r.crates.forEach((id) => chips.appendChild(el('span', 'crate-chip', chip(id))));
  card.appendChild(chips);
}

function wireRoboTabs()
{
  const tabsEl = $('#robo-tabs');
  if (!tabsEl) return;
  const figs = {};
  $$('#robotics .diorama').forEach((f) => { figs[f.dataset.diorama] = f; });
  const select = (r, btn) =>
  {
    $$('.robo-tab', tabsEl).forEach((b) =>
    {
      const on = b === btn;
      b.classList.toggle('active', on);
      b.setAttribute('aria-selected', on ? 'true' : 'false');
    });
    Object.values(figs).forEach((f) => f.classList.remove('is-active'));
    if (figs[r.id]) figs[r.id].classList.add('is-active');
    renderRoboCard(r);
  };
  ROBOTS.forEach((r, i) =>
  {
    const btn = el('button', 'robo-tab' + (i === 0 ? ' active' : ''));
    btn.type = 'button';
    btn.setAttribute('role', 'tab');
    btn.dataset.accent = r.accent;
    btn.innerHTML = `<b>${r.tab}</b><span>${r.sub}</span>`;
    btn.addEventListener('click', () => select(r, btn));
    tabsEl.appendChild(btn);
  });
  select(ROBOTS[0], $$('.robo-tab', tabsEl)[0]);
}

function pkgButtons(crate)
{
  const box = el('div', 'pkg-btns');
  const pkgs = packagesFor(crate);
  if (!pkgs.length)
  {
    box.appendChild(el('span', 'pkg-none', crate.planned ? 'On the roadmap - not published yet' : ''));
    return box;
  }
  pkgs.forEach((p) =>
  {
    const a = el('a', 'pkg-btn ' + p.kind, p.label);
    a.href = p.href; a.target = '_blank'; a.rel = 'noopener';
    a.setAttribute('aria-label', `${crate.name} on ${p.label}`);
    box.appendChild(a);
  });
  // Getting started: the crate's own README on GitHub (its richer overview, with a docs.rs
  // button), set apart on the right from the package-registry links.
  const docs = el('a', 'pkg-btn docs', 'README');
  docs.href = `https://github.com/molexxxx/pamoja/blob/main/crates/${crate.id}/README.md#${crate.id}`;
  docs.target = '_blank'; docs.rel = 'noopener';
  docs.setAttribute('aria-label', `${crate.name} README - getting started`);
  box.appendChild(docs);
  return box;
}

const BENTO_BIG = 'pamoja-core';
const BENTO_WIDE = new Set(['pamoja-mqtt', 'pamoja-mesh', 'pamoja-security', 'pamoja-profile']);
const BENTO_FX = {
  'pamoja-core': 'core', 'pamoja-mqtt': 'broadcast', 'pamoja-mesh': 'mesh',
  'pamoja-security': 'shield', 'pamoja-profile': 'bars',
};

function buildBento()
{
  const board = $('#crate-bento');
  if (!board) return;

  const byArea = {};
  [...CRATES, ...PLANNED_CRATES].forEach((c) => { (byArea[areaOf(c.role)] ||= []).push(c); });
  const areas = AREA_ORDER.filter((a) => byArea[a]).concat(Object.keys(byArea).filter((a) => !AREA_ORDER.includes(a)));
  const ordered = [];
  areas.forEach((a) => byArea[a].forEach((c) => ordered.push(c)));
  ordered.sort((a, b) => (a.id === BENTO_BIG ? -1 : b.id === BENTO_BIG ? 1 : 0));

  ordered.forEach((c) =>
  {
    const span = c.id === BENTO_BIG ? ' span-big' : BENTO_WIDE.has(c.id) ? ' span-wide' : '';
    const card = el('article', 'bento-card' + (c.planned ? ' planned' : '') + span);
    card.dataset.accent = c.color || 'amber';
    card.dataset.kind = c.planned ? 'roadmap' : 'shipping';
    card.dataset.area = areaOf(c.role);
    card.tabIndex = 0;
    card.innerHTML =
      `<div class="bc-face">`
      + `<div class="bc-top"><span class="bc-role">${c.role.replace(' · planned', '')}</span>`
      + `<span class="bc-ver">${c.planned ? 'roadmap' : ''}</span></div>`
      + `<h4 class="bc-name">${c.name}</h4>`
      + `</div>`
      + `<div class="bc-pop"><p class="bc-blurb">${c.blurb}</p></div>`;
    card.querySelector('.bc-pop').appendChild(pkgButtons(c));
    board.appendChild(card);
    if (!c.planned)
    {
      const ver = card.querySelector('.bc-ver');
      crateVersion(c.id).then((v) => { if (v) ver.textContent = `v${v}`; });
    }
    if (BENTO_FX[c.id]) mountCardFx(card, c.id === BENTO_BIG ? 'amber' : (c.color || 'amber'), BENTO_FX[c.id]);
  });

  wireBento(board);
}

function wireBento(board)
{
  const cards = $$('.bento-card', board);
  const lively = () => !prefersReducedMotion && window.matchMedia('(min-width: 821px)').matches;
  const resetTilt = (card) => { card.style.removeProperty('--rx'); card.style.removeProperty('--ry'); };
  const clamp = (v, a, b) => Math.max(a, Math.min(b, v));

  cards.forEach((card) =>
  {
    card.addEventListener('pointermove', (e) =>
    {
      if (!lively() || card.classList.contains('pinned')) return;
      const r = card.getBoundingClientRect();
      const px = clamp((e.clientX - r.left) / r.width - 0.5, -0.5, 0.5);
      const py = clamp((e.clientY - r.top) / r.height - 0.5, -0.5, 1.1);
      card.style.setProperty('--rx', `${(-py * 8).toFixed(2)}deg`);
      card.style.setProperty('--ry', `${(px * 10).toFixed(2)}deg`);
    });
    card.addEventListener('pointerleave', () => resetTilt(card));
    card.addEventListener('click', (e) =>
    {
      if (e.target.closest('a')) return;
      const open = card.classList.contains('pinned');
      cards.forEach((c) => { c.classList.remove('pinned'); resetTilt(c); });
      if (!open) card.classList.add('pinned');
    });
    card.addEventListener('keydown', (e) =>
    {
      if (e.key === 'Enter' || e.key === ' ') { e.preventDefault(); card.click(); }
      if (e.key === 'Escape') { card.classList.remove('pinned'); resetTilt(card); }
    });
  });

  document.addEventListener('click', (e) =>
  {
    if (!e.target.closest('.bento-card')) cards.forEach((c) => { c.classList.remove('pinned'); resetTilt(c); });
  });
}

const AREA_SHORT = {
  'Core & data': 'Core', 'Messaging & radio': 'Messaging', 'Field I/O & sensors': 'Field I/O',
  'Resilience & power': 'Resilience', 'Security & trust': 'Security', 'Ergonomics & reach': 'Ergonomics',
  'Robotics & drones': 'Robotics',
};

function wireCrateFilter()
{
  const filter = $('#crate-filter');
  const areaWrap = $('#crate-areas');
  const board = $('#crate-bento');
  if (!filter || !board) return;
  const cards = $$('.bento-card', board);
  const state = { status: 'shipping', area: 'all' };

  const apply = () =>
  {
    $$('.cf-btn', filter).forEach((b) =>
    {
      const on = b.dataset.filter === state.status;
      b.classList.toggle('active', on);
      b.setAttribute('aria-selected', on ? 'true' : 'false');
    });
    if (areaWrap) $$('.cf-btn', areaWrap).forEach((b) =>
    {
      const on = b.dataset.area === state.area;
      b.classList.toggle('active', on);
      b.setAttribute('aria-selected', on ? 'true' : 'false');
    });
    cards.forEach((c) =>
    {
      const show = (state.status === 'all' || c.dataset.kind === state.status)
        && (state.area === 'all' || c.dataset.area === state.area);
      c.classList.remove('pinned');
      c.classList.toggle('hide', !show);
      if (show) { c.classList.remove('in'); void c.offsetWidth; c.classList.add('in'); }
    });
  };

  if (areaWrap)
  {
    const present = AREA_ORDER.filter((a) => cards.some((c) => c.dataset.area === a));
    const addChip = (val, label) =>
    {
      const b = el('button', 'cf-btn', label);
      b.type = 'button'; b.dataset.area = val; b.setAttribute('role', 'tab');
      b.addEventListener('click', () => { state.area = val; apply(); });
      areaWrap.appendChild(b);
    };
    addChip('all', 'All areas');
    present.forEach((a) => addChip(a, AREA_SHORT[a] || a));
  }

  $$('.cf-btn', filter).forEach((b) => b.addEventListener('click', () => { state.status = b.dataset.filter; apply(); }));
  apply();
}

function wireNav()
{
  const nav = $('#nav');
  const toggle = $('#nav-toggle');
  if (!nav || !toggle) return;
  const close = () =>
  {
    nav.classList.remove('open');
    toggle.setAttribute('aria-expanded', 'false');
  };
  toggle.addEventListener('click', () =>
  {
    const open = nav.classList.toggle('open');
    toggle.setAttribute('aria-expanded', open ? 'true' : 'false');
  });
  $$('.nav-links a', nav).forEach((a) => a.addEventListener('click', close));
  window.addEventListener('resize', () => { if (window.innerWidth > 760) close(); }, { passive: true });
}

function buildLanguages()
{
  const tabs = $('#lang-tabs');
  const code = $('#lang-code');
  const render = (lang) =>
  {
    code.textContent = lang.code;
    $$('.lang-tab', tabs).forEach((t) => t.classList.toggle('active', t.dataset.lang === lang.id));
  };
  LANGUAGES.forEach((lang, i) =>
  {
    const tab = el('button', 'lang-tab' + (i === 0 ? ' active' : ''));
    tab.innerHTML = `${lang.name}<span class="st">${lang.status}</span>`;
    tab.dataset.lang = lang.id;
    tab.setAttribute('role', 'tab');
    tab.addEventListener('click', () => render(lang));
    tabs.appendChild(tab);
  });
  render(LANGUAGES[0]);

  const chips = $('#lang-planned-chips');
  PLANNED_LANGS.forEach((p) => chips.appendChild(el('span', 'planned-chip', p)));
}

function buildTiers(form)
{
  const wrap = $('#tiers');
  TIERS.forEach((t) =>
  {
    const card = el('div', 'tier' + (t.featured ? ' featured' : ''));
    card.dataset.accent = t.accent;
    card.innerHTML = `
      ${t.featured ? '<span class="tag">most impact</span>' : ''}
      <h3>${t.name}</h3>
      <div class="amt">$${t.amount}<span> one-time</span></div>
      <p class="head">${t.headline}</p>
      <ul>${t.items.map((i) => `<li>${i}</li>`).join('')}</ul>
      <button class="btn btn-ghost" type="button">Back the ${t.name}</button>`;
    card.querySelector('button').addEventListener('click', comingSoon);
    wrap.appendChild(card);
  });
}

function buildUplinks(form)
{
  const wrap = $('#uplinks');
  if (!wrap) return;
  UPLINKS.forEach((u) =>
  {
    const card = el('div', 'tier uplink');
    card.dataset.accent = u.accent;
    const recurring = u.amount != null;
    const amt = recurring ? `$${u.amount}<span> ${u.per}</span>` : '<span class="partner-amt">partner</span>';
    card.innerHTML = `
      <span class="tag soft">${recurring ? 'recurring' : 'partner'}</span>
      <h3>${u.name}</h3>
      <div class="amt">${amt}</div>
      <p class="head">${u.headline}</p>
      <ul>${u.items.map((i) => `<li>${i}</li>`).join('')}</ul>
      <button class="btn btn-ghost" type="button">${u.role === 'vendor' ? 'Become a partner' : 'Sponsor this'}</button>`;
    card.querySelector('button').addEventListener('click', comingSoon);
    wrap.appendChild(card);
  });
}

function buildTracks()
{
  const wrap = $('#tracks');
  if (!wrap) return;
  TRACKS.forEach((tr) =>
  {
    const card = el('article', 'track');
    card.dataset.accent = tr.accent;
    const tags = tr.tags
      .map((g) => `<span class="track-tag${g.on ? ' on' : ''}">${g.t}</span>`)
      .join('');
    card.innerHTML = `
      <h3>${tr.title}</h3>
      <p>${tr.lead}</p>
      <div class="track-tags">${tags}</div>`;
    wrap.appendChild(card);
  });
}

function buildPackages()
{
  const box = $('#pkg-links');
  PACKAGES.forEach((p) =>
  {
    const a = el('a', null, p.label);
    a.href = p.href; a.target = '_blank'; a.rel = 'noopener';
    box.appendChild(a);
  });
}

function wireForm()
{
  const form = $('#pledge-form');
  let role = 'donor';
  $$('.role', form).forEach((btn) =>
    btn.addEventListener('click', () =>
    {
      role = btn.dataset.role;
      $$('.role', form).forEach((b) => b.classList.toggle('active', b === btn));
      $$('[data-when]', form).forEach((n) => (n.hidden = n.dataset.when !== role));
    }),
  );

  form.addEventListener('submit', (e) =>
  {
    e.preventDefault();
    showToast(PREVIEW_MSG);
  });

  return form;
}

function animateGoal()
{
  $('#goal-fill').style.width = '0%';
  $('#goal-count').textContent = 'opens later';
}

// ---- scroll wiring -----------------------------------------------------

function wireParallax()
{
  if (prefersReducedMotion || window.matchMedia('(max-width: 960px)').matches) return;
  const stages = $$('.scene-stage');
  let ticking = false;
  const update = () =>
  {
    ticking = false;
    const vh = window.innerHeight;
    for (const stage of stages)
    {
      const r = stage.getBoundingClientRect();
      if (r.bottom < -vh * 0.4 || r.top > vh * 1.4) continue;
      const p = (r.top + r.height / 2 - vh / 2) / vh;
      const dio = stage.querySelector('.diorama');
      const card = stage.querySelector('.card');
      if (dio) dio.style.transform = `translateY(${(p * 26).toFixed(1)}px)`;
      if (card) card.style.transform = `translateY(${(p * -16).toFixed(1)}px)`;
    }
  };
  const onScroll = () => { if (!ticking) { ticking = true; requestAnimationFrame(update); } };
  window.addEventListener('scroll', onScroll, { passive: true });
  window.addEventListener('resize', onScroll, { passive: true });
  update();
}

function wireScroll(onScene)
{
  const rail = $('.field-rail');
  const railDots = $$('.rail-dot');
  const fieldScenes = new Set(['farm', 'clinic', 'water', 'conservation', 'village', 'storm']);

  const io = new IntersectionObserver(
    (entries) =>
    {
      entries.forEach((entry) =>
      {
        if (!entry.isIntersecting) return;
        const name = entry.target.dataset.scene;
        onScene(name);
        railDots.forEach((d) => d.classList.toggle('active', d.dataset.rail === name));
      });
    },
    { rootMargin: '-45% 0px -45% 0px', threshold: 0 },
  );
  $$('[data-scene]').forEach((s) => io.observe(s));

  const nav = $('#nav');
  const field = $('#field');
  const onScrollEffects = () =>
  {
    nav.classList.toggle('solid', window.scrollY > 50);
    const r = field.getBoundingClientRect();
    rail.classList.toggle('show', r.top < window.innerHeight * 0.5 && r.bottom > window.innerHeight * 0.5);
  };
  window.addEventListener('scroll', onScrollEffects, { passive: true });
  onScrollEffects();
}

// ---- entry -------------------------------------------------------------

export function initUI({ onScene })
{
  buildStats();
  buildScenarioChips();
  wireRoboTabs();
  mountConsoles();
  buildBento();
  wireCrateFilter();
  buildLanguages();
  const form = wireForm();
  buildTiers(form);
  buildUplinks(form);
  buildTracks();
  buildPackages();
  animateGoal();
  wireNav();
  wireParallax();
  wireScroll(onScene);
  countUp();
}
