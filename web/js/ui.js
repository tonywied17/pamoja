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

function buildConstellation()
{
  const stage = $('#crate-constellation');
  const ship = CRATES.filter((c) => c.id !== 'pamoja-core');
  const inner = ship.slice(0, 6);
  const mid = ship.slice(6);

  const svgNS = 'http://www.w3.org/2000/svg';
  const svg = document.createElementNS(svgNS, 'svg');
  svg.setAttribute('viewBox', '0 0 100 100');
  svg.setAttribute('preserveAspectRatio', 'none');
  stage.appendChild(svg);

  stage.appendChild(el('div', 'crate-core', 'pamoja<br>core'));

  const place = (group, radius, startAngle, planned) =>
  {
    group.forEach((c, i) =>
    {
      const a = startAngle + (i / group.length) * Math.PI * 2;
      const x = 50 + radius * Math.cos(a);
      const y = 50 + radius * Math.sin(a);

      const line = document.createElementNS(svgNS, 'line');
      line.setAttribute('x1', 50); line.setAttribute('y1', 50);
      line.setAttribute('x2', x); line.setAttribute('y2', y);
      line.setAttribute('stroke', 'rgba(251,243,228,0.16)');
      line.setAttribute('stroke-width', '0.4');
      if (planned) line.setAttribute('stroke-dasharray', '1.6 1.8');
      line.dataset.line = c.id;
      svg.appendChild(line);

      const node = el('button', 'crate-node' + (planned ? ' planned' : ''), c.name);
      node.style.left = x + '%';
      node.style.top = y + '%';
      node.dataset.crate = c.id;
      stage.appendChild(node);
    });
  };
  place(inner, 20, -Math.PI / 2, false);
  place(mid, 32.5, -Math.PI / 2 + Math.PI / mid.length, false);
  place(PLANNED_CRATES, 45, -Math.PI / 2 + Math.PI / PLANNED_CRATES.length, true);

  const detail = $('#crate-detail');
  const show = (c) =>
  {
    detail.querySelector('.cd-role').textContent = c.role;
    detail.querySelector('.cd-name').textContent = c.name;
    detail.querySelector('.cd-blurb').textContent = c.blurb;
    detail.classList.toggle('is-planned', !!c.planned);
  };
  const setActive = (id) =>
  {
    $$('.crate-node', stage).forEach((n) =>
    {
      const on = n.dataset.crate === id;
      n.classList.toggle('active', on);
      n.classList.toggle('dim', id && !on);
    });
    $$('line', svg).forEach((l) =>
    {
      l.setAttribute('stroke', l.dataset.line === id ? 'var(--accent)' : 'rgba(251,243,228,0.16)');
      l.setAttribute('stroke-width', l.dataset.line === id ? '0.8' : '0.4');
    });
  };

  $$('.crate-node', stage).forEach((node) =>
  {
    const c = crateById[node.dataset.crate];
    node.addEventListener('mouseenter', () => { show(c); setActive(c.id); });
    node.addEventListener('focus', () => { show(c); setActive(c.id); });
    node.addEventListener('click', () => { show(c); setActive(c.id); });
  });
  stage.addEventListener('mouseleave', () =>
  {
    setActive(null);
    show(crateById['pamoja-core']);
  });
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
  buildConstellation();
  buildLanguages();
  const form = wireForm();
  buildTiers(form);
  buildUplinks(form);
  buildTracks();
  buildPackages();
  animateGoal();
  wireParallax();
  wireScroll(onScene);
  countUp();
}
