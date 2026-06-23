#!/usr/bin/env node

// Generates pamoja's own status badges as themed SVGs - no third-party badge service.
// Each badge pulls live data (published version per registry, CI conclusion) from the
// public registry and GitHub APIs and renders a dark-glass pill in the pamoja palette.
// Run by .github/workflows/badges.yml on a schedule, on release, and on demand; it commits
// the refreshed SVGs back. A GitHub token (GITHUB_TOKEN in CI, `gh auth token` locally) is
// optional and only lifts the rate limit on the CI-status lookup.

import { writeFileSync, readFileSync, existsSync, mkdirSync } from 'node:fs';
import { createHash } from 'node:crypto';
import path from 'node:path';

const OWNER = 'molexxxx';
const REPO = 'pamoja';
const OUT = '.github/badges';
const TOKEN = process.env.GITHUB_TOKEN || process.env.GH_TOKEN || '';

// pamoja palette (the dashboard's): deep navy glass, teal accent, warm/cool dots that echo
// the logo. Status borrows the dashboard's ok/alarm tokens.
const THEME = {
  bg: '#0b1124',
  border: '#1fd3b0',
  borderOpacity: 0.32,
  label: '#8b95a7',
  value: '#36e0c2',
  ok: '#36e0c2',
  fail: '#ff6f55',
};
const DOT = {
  crates: '#ffb627',
  npm: '#ff6f55',
  pypi: '#38e1ff',
  nuget: '#8b7bff',
  teal: '#1fd3b0',
  fail: '#ff6f55',
};

const BADGES = [
  { id: 'crates', kind: 'crates', label: 'crates.io', pkg: 'pamoja-core', dot: DOT.crates },
  { id: 'npm', kind: 'npm', label: 'npm', pkg: '@pamoja/core', dot: DOT.npm },
  { id: 'pypi', kind: 'pypi', label: 'PyPI', pkg: 'pamoja-core', dot: DOT.pypi },
  { id: 'nuget', kind: 'nuget', label: 'NuGet', pkg: 'Pamoja.Core', dot: DOT.nuget },
  { id: 'ci', kind: 'workflow', label: 'CI', workflow: 'ci.yml', branch: 'main', dot: DOT.teal },
  { id: 'license', kind: 'static', label: 'license', message: 'MIT', dot: DOT.teal },
];

const UA = 'pamoja-badge-gen (https://github.com/molexxxx/pamoja)';

async function getJson(url, headers = {}) {
  const r = await fetch(url, { headers: { 'User-Agent': UA, ...headers } });
  if (!r.ok) throw new Error(`${url}: HTTP ${r.status}`);
  return r.json();
}

async function getValue(b) {
  if (b.kind === 'static') return { text: b.message, color: THEME.value };
  if (b.kind === 'crates') {
    const j = await getJson(`https://crates.io/api/v1/crates/${b.pkg}`);
    return { text: 'v' + (j.crate.max_stable_version || j.crate.max_version), color: THEME.value };
  }
  if (b.kind === 'npm') {
    const j = await getJson(`https://registry.npmjs.org/${encodeURIComponent(b.pkg)}/latest`);
    return { text: 'v' + j.version, color: THEME.value };
  }
  if (b.kind === 'pypi') {
    const j = await getJson(`https://pypi.org/pypi/${b.pkg}/json`);
    return { text: 'v' + j.info.version, color: THEME.value };
  }
  if (b.kind === 'nuget') {
    const j = await getJson(`https://api.nuget.org/v3-flatcontainer/${b.pkg.toLowerCase()}/index.json`);
    return { text: 'v' + j.versions[j.versions.length - 1], color: THEME.value };
  }
  if (b.kind === 'workflow') {
    const headers = TOKEN ? { Authorization: `Bearer ${TOKEN}`, Accept: 'application/vnd.github+json' } : {};
    const j = await getJson(`https://api.github.com/repos/${OWNER}/${REPO}/actions/workflows/${b.workflow}/runs?branch=${b.branch}&per_page=1`, headers);
    const run = j.workflow_runs?.[0];
    const conclusion = run?.status === 'completed' ? run.conclusion : run?.status;
    const ok = conclusion === 'success';
    return { text: ok ? 'passing' : (conclusion || 'no runs'), color: ok ? THEME.ok : THEME.fail, dot: ok ? DOT.teal : DOT.fail };
  }
  throw new Error(`unknown kind ${b.kind}`);
}

// Per-character advance widths for Segoe UI at 11px, so every pill is sized tightly with no
// trailing dead space (calibrated against the rendered font).
const W11 = {
  ' ': 3.1, '!': 3.7, '"': 4.7, '#': 7.0, $: 6.2, '%': 9.6, '&': 7.6, "'": 2.6, '(': 3.7, ')': 3.7,
  '*': 5.5, '+': 6.5, ',': 3.1, '-': 3.7, '.': 3.1, '/': 4.2, 0: 6.2, 1: 6.2, 2: 6.2, 3: 6.2, 4: 6.2,
  5: 6.2, 6: 6.2, 7: 6.2, 8: 6.2, 9: 6.2, ':': 3.1, ';': 3.1, '<': 6.5, '=': 6.5, '>': 6.5, '?': 5.2, '@': 10.5,
  A: 7.0, B: 6.9, C: 7.2, D: 7.8, E: 6.3, F: 6.0, G: 7.9, H: 7.9, I: 2.9, J: 3.2, K: 6.7, L: 5.7, M: 9.3,
  N: 7.9, O: 8.2, P: 6.7, Q: 8.2, R: 7.1, S: 6.4, T: 6.2, U: 7.7, V: 6.8, W: 10.2, X: 6.6, Y: 6.2, Z: 6.7,
  a: 5.7, b: 6.2, c: 5.2, d: 6.2, e: 5.9, f: 3.6, g: 6.2, h: 6.2, i: 2.6, j: 2.6, k: 5.6, l: 2.6, m: 9.5,
  n: 6.2, o: 6.2, p: 6.2, q: 6.2, r: 4.1, s: 5.0, t: 3.8, u: 6.2, v: 5.6, w: 8.2, x: 5.6, y: 5.6, z: 5.0,
};
const textWidth = (s) => Math.ceil([...s].reduce((w, c) => w + (W11[c] ?? 6.2), 0));
const esc = (s) => s.replace(/[<>&'"]/g, (c) => ({ '<': '&lt;', '>': '&gt;', '&': '&amp;', "'": '&apos;', '"': '&quot;' }[c]));

const H = 22, RX = 6, PAD = 9, DOT_R = 3, DOT_GAP = 7, SEP = 6;

function pill(b, value) {
  const labelW = textWidth(b.label) + SEP;
  const valueW = textWidth(value.text);
  const W = PAD + DOT_R * 2 + DOT_GAP + labelW + valueW + PAD;
  const dotCx = PAD + DOT_R;
  const labelX = PAD + DOT_R * 2 + DOT_GAP;
  const valueX = labelX + labelW;
  return `<svg xmlns="http://www.w3.org/2000/svg" width="${W}" height="${H}" role="img" aria-label="${esc(b.label)}: ${esc(value.text)}">
  <defs><linearGradient id="g" x1="0" y1="0" x2="0" y2="1"><stop offset="0" stop-color="#101935"/><stop offset="1" stop-color="${THEME.bg}"/></linearGradient></defs>
  <rect x="0.5" y="0.5" width="${W - 1}" height="${H - 1}" rx="${RX}" fill="url(#g)" stroke="${THEME.border}" stroke-opacity="${THEME.borderOpacity}"/>
  <circle cx="${dotCx}" cy="${H / 2}" r="${DOT_R}" fill="${value.dot || b.dot}"/>
  <g font-family="'Segoe UI',Inter,-apple-system,BlinkMacSystemFont,sans-serif" font-size="11">
    <text x="${labelX}" y="15" fill="${THEME.label}" font-weight="500">${esc(b.label)}</text>
    <text x="${valueX}" y="15" fill="${value.color}" font-weight="700">${esc(value.text)}</text>
  </g>
</svg>
`;
}

mkdirSync(OUT, { recursive: true });
const hashes = {};
for (const b of BADGES) {
  try {
    const value = await getValue(b);
    const svg = pill(b, value);
    writeFileSync(path.join(OUT, `${b.id}.svg`), svg);
    hashes[b.id] = createHash('sha1').update(svg).digest('hex').slice(0, 8);
    console.log(`ok  ${b.id.padEnd(10)} ${b.label} ${value.text}`);
  } catch (e) {
    console.error(`err ${b.id.padEnd(10)} ${e.message}`);
    process.exitCode = 1;
  }
}

// Stamp each badge URL in the README with its content hash so GitHub's image proxy serves
// the fresh SVG the moment a version or CI status changes, instead of a cached copy.
if (existsSync('README.md')) {
  let readme = readFileSync('README.md', 'utf8');
  for (const [id, h] of Object.entries(hashes)) {
    readme = readme.replace(new RegExp(`(/\\.github/badges/${id}\\.svg)(\\?v=[a-f0-9]+)?`, 'g'), `$1?v=${h}`);
  }
  writeFileSync('README.md', readme);
}
