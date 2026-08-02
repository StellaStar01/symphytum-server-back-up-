const listEl = document.getElementById('list');
const searchEl = document.getElementById('search');
const fileInput = document.getElementById('file-input');
const fileLabel = document.getElementById('file-label');
const uploadBtn = document.getElementById('upload-btn');
const state = { sort: 'time', dir: -1, open: new Set() };

function updateButtons() {
  const nameBtn = document.getElementById('sort-name');
  const timeBtn = document.getElementById('sort-time');
  nameBtn.classList.toggle('active', state.sort === 'name');
  timeBtn.classList.toggle('active', state.sort === 'time');
  const arrow = state.dir === -1 ? ' ↓' : ' ↑';
  nameBtn.textContent = 'name' + (state.sort === 'name' ? arrow : '');
  timeBtn.textContent = 'time' + (state.sort === 'time' ? arrow : '');
}

function updateCount() {
  const all = Array.from(listEl.querySelectorAll('details'));
  const shown = all.filter(el => !el.hidden).length;
  document.getElementById('count').textContent = shown === all.length ? String(all.length) : shown + ' / ' + all.length;
}

function applyFilter() {
  const q = searchEl.value.trim().toLowerCase();
  for (const el of listEl.querySelectorAll('details')) {
    const hay = (el.dataset.name + ' ' + el.dataset.route).toLowerCase();
    el.hidden = q !== '' && !hay.includes(q);
  }
  updateCount();
}

function applySort() {
  const items = Array.from(listEl.querySelectorAll('details'));
  const key = state.sort === 'name'
    ? (el) => el.dataset.name
    : (el) => Number(el.dataset.ts);
  items.sort((a, b) => {
    const ka = key(a), kb = key(b);
    if (ka !== kb) return ka < kb ? -state.dir : state.dir;
    return Number(b.dataset.ts) - Number(a.dataset.ts);
  });
  for (const el of items) listEl.appendChild(el);
  for (const el of items) el.open = state.open.has(el.dataset.file);
  updateCount();
  updateButtons();
}

function renderList(html) {
  listEl.innerHTML = html;
  applyFilter();
  applySort();
}

listEl.addEventListener('click', (e) => {
  if (e.target.closest('a.view-link')) return;
  const d = e.target.closest('details');
  if (!d) return;
  const summary = e.target.closest('summary');
  if (summary) {
    if (e.button !== 0 || e.ctrlKey || e.metaKey || e.shiftKey || e.altKey) return;
    e.preventDefault();
    d.open = !d.open;
    setTimeout(() => {
      if (d.open) state.open.add(d.dataset.file);
      else state.open.delete(d.dataset.file);
    }, 0);
  } else if (d.open) {
    if (window.getSelection().toString() !== '') return;
    d.open = false;
    state.open.delete(d.dataset.file);
  }
});

searchEl.addEventListener('input', applyFilter);

fileInput.addEventListener('change', () => {
  const n = fileInput.files.length;
  fileLabel.textContent = n === 0 ? 'browse…' : n === 1 ? fileInput.files[0].name : n + ' files';
  uploadBtn.disabled = n === 0;
});

document.getElementById('refresh').addEventListener('click', async () => {
  try {
    const r = await fetch('/list');
    renderList(await r.text());
  } catch (e) {}
});

document.getElementById('refresh-full').addEventListener('click', async () => {
  try {
    const r = await fetch('/refresh', { method: 'POST' });
    renderList(await r.text());
  } catch (e) {}
});

const themeBtn = document.getElementById('theme');
themeBtn.textContent = 'theme: ' + (localStorage.getItem('theme') || 'dark');
themeBtn.addEventListener('click', () => {
  const next = document.documentElement.dataset.theme === 'dark' ? 'light' : 'dark';
  document.documentElement.dataset.theme = next;
  localStorage.setItem('theme', next);
  themeBtn.textContent = 'theme: ' + next;
});

function pickSort(which) {
  if (state.sort === which) state.dir = -state.dir;
  else {
    state.sort = which;
    state.dir = which === 'name' ? 1 : -1;
  }
  applySort();
}
document.getElementById('sort-name').addEventListener('click', () => pickSort('name'));
document.getElementById('sort-time').addEventListener('click', () => pickSort('time'));

applyFilter();
applySort();
