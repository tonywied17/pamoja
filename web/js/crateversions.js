const TTL = 12 * 60 * 60 * 1000;
const KEY = 'pamoja:crate-versions';

let store = {};
try { store = JSON.parse(localStorage.getItem(KEY) || '{}'); } catch { store = {}; }
const save = () => { try { localStorage.setItem(KEY, JSON.stringify(store)); } catch { /* ignore */ } };

function indexPath(name)
{
  const n = name.toLowerCase();
  if (n.length === 1) return `1/${n}`;
  if (n.length === 2) return `2/${n}`;
  if (n.length === 3) return `3/${n[0]}/${n}`;
  return `${n.slice(0, 2)}/${n.slice(2, 4)}/${n}`;
}

function latestNonYanked(text)
{
  let v = null;
  for (const line of text.trim().split('\n'))
  {
    if (!line) continue;
    try { const o = JSON.parse(line); if (!o.yanked && o.vers) v = o.vers; } catch { /* skip */ }
  }
  return v;
}

const inflight = {};

export function crateVersion(name)
{
  const cached = store[name];
  if (cached && Date.now() - cached.t < TTL) return Promise.resolve(cached.v);
  if (inflight[name]) return inflight[name];

  // ?d=<day> makes the CDN return an Origin/CORS response (the plain object is cached without it).
  const day = new Date().toISOString().slice(0, 10);
  const url = `https://index.crates.io/${indexPath(name)}?d=${day}`;
  inflight[name] = fetch(url)
    .then((r) => (r.ok ? r.text() : ''))
    .then((text) =>
    {
      const v = latestNonYanked(text);
      if (v) { store[name] = { v, t: Date.now() }; save(); }
      delete inflight[name];
      return v || (cached && cached.v) || null;
    })
    .catch(() => { delete inflight[name]; return (cached && cached.v) || null; });
  return inflight[name];
}
