import
  {
    STATS, CRATES, PLANNED_CRATES, SCENARIO_CRATES, LANGUAGES, PLANNED_LANGS, TIERS, UPLINKS, TRACKS,
  } from './data.js';
import { prefersReducedMotion } from './config.js';
import { mountConsoles } from './consoles.js';

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
  { label: 'GitHub', href: 'https://github.com/tonywied17/pamoja' },
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

// Backing is not live yet; the whole section is a preview. Buttons and the form
// surface this instead of doing anything.
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
    const to = +node.dataset.to;
    const prefix = node.textContent.startsWith('$') ? '$' : '';
    if (prefersReducedMotion) { node.textContent = `${prefix}${to}`; return; }
    const start = performance.now();
    const dur = 1300;
    const tick = (now) =>
    {
      const p = Math.min(1, (now - start) / dur);
      const e = 1 - Math.pow(1 - p, 3);
      node.textContent = `${prefix}${Math.round(to * e)}`;
      if (p < 1) requestAnimationFrame(tick);
    };
    requestAnimationFrame(tick);
  });
}

function buildScenarioChips()
{
  $$('.crate-chips').forEach((box) =>
  {
    const list = SCENARIO_CRATES[box.dataset.crates] || [];
    list.forEach((id) => box.appendChild(el('span', 'crate-chip', chip(id))));
  });
}

// A capability board: every crate shown at once, grouped by the area it serves,
// so what the SDK offers is scannable without hovering, and the grid reflows to
// one column on a phone. Replaces the old radial constellation.
function buildCrateBoard()
{
  const board = $('#crate-board');
  if (!board) return;

  const areaOf = (role) =>
    ({
      'the core': 'Core & data', serialize: 'Core & data',
      messaging: 'Messaging & radio', radio: 'Messaging & radio', mesh: 'Messaging & radio',
      'field I/O': 'Field I/O & sensors',
      trust: 'Security & trust',
      observe: 'Resilience & power', energy: 'Resilience & power', resilience: 'Resilience & power',
      ergonomics: 'Ergonomics & reach', language: 'Ergonomics & reach',
      robotics: 'Robotics & drones', drones: 'Robotics & drones',
    })[role.replace(' · planned', '')] || 'More';

  const order = [
    'Core & data', 'Messaging & radio', 'Field I/O & sensors',
    'Resilience & power', 'Security & trust', 'Ergonomics & reach', 'Robotics & drones',
  ];

  const byArea = {};
  [...CRATES, ...PLANNED_CRATES].forEach((c) =>
  {
    const a = areaOf(c.role);
    (byArea[a] = byArea[a] || []).push(c);
  });
  const areas = order.filter((a) => byArea[a]).concat(Object.keys(byArea).filter((a) => !order.includes(a)));

  const card = (c) =>
  {
    const t = el('article', 'cb-card' + (c.planned ? ' planned' : '') + (c.id === 'pamoja-core' ? ' is-core' : ''));
    t.dataset.accent = c.color || 'amber';
    t.dataset.kind = c.planned ? 'roadmap' : 'shipping';
    t.innerHTML =
      `<div class="cb-top"><span class="cb-role">${c.role.replace(' · planned', '')}</span>`
      + `<span class="cb-status">${c.planned ? 'roadmap' : 'live'}</span></div>`
      + `<h4 class="cb-name">${c.name}</h4>`
      + `<p class="cb-blurb">${c.blurb}</p>`;
    return t;
  };

  areas.forEach((areaName) =>
  {
    const section = el('div', 'crate-area');
    section.appendChild(el('p', 'ca-head', areaName));
    const cards = el('div', 'cb-cards');
    byArea[areaName].forEach((c) => cards.appendChild(card(c)));
    section.appendChild(cards);
    board.appendChild(section);
  });
}

// The filter chips isolate the shipping crates, the roadmap, or both, and hide
// any capability area the current filter leaves empty.
function wireCrateFilter()
{
  const filter = $('#crate-filter');
  const board = $('#crate-board');
  if (!filter || !board) return;
  const apply = (f) =>
  {
    board.dataset.filter = f;
    $$('.cf-btn', filter).forEach((b) =>
    {
      const on = b.dataset.filter === f;
      b.classList.toggle('active', on);
      b.setAttribute('aria-selected', on ? 'true' : 'false');
    });
    $$('.crate-area', board).forEach((area) =>
    {
      const has = $$('.cb-card', area).some((c) => f === 'all' || c.dataset.kind === f);
      area.classList.toggle('empty', !has);
    });
  };
  $$('.cf-btn', filter).forEach((b) => b.addEventListener('click', () => apply(b.dataset.filter)));
  apply('shipping');
}

// Hamburger menu for narrow screens: toggle the panel, close on link tap or
// once the viewport is wide enough to show the inline links again.
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

  // Backing is not live yet: the form is disabled and submitting does nothing
  // but surface the preview message.
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
  mountConsoles();
  buildCrateBoard();
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
