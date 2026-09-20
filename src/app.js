const { invoke } = window.__TAURI__.core;
const { getCurrentWindow } = window.__TAURI__.window;

const PRESETS = [
  { name: 'Low', description: '720p · 15fps · 2 Mbps' },
  { name: 'Medium', description: '720p · 30fps · 4 Mbps' },
  { name: 'High', description: '1080p · 60fps · 8 Mbps' },
  { name: 'Ultra', description: 'Native resolution · unlimited fps · 50 Mbps' },
];
const ACCENTS = ['pink', 'cyan', 'purple', 'green', 'orange'];

const appWindow = getCurrentWindow();
const el = (id) => document.getElementById(id);

const state = {
  settings: {
    ip: '', connect_port: '5555', pair_port: '5555',
    quality: 'Medium', theme: 'dark', accent: 'pink', auto_connect: false,
  },
  devices: [],
  selected: null,
  device: { name: 'No device detected', addr: '—', state: 'idle' },
  env: null,
};

/* ---------- helpers ---------- */

function escapeHtml(value) {
  return String(value).replace(/[&<>"']/g, (c) => (
    { '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;', "'": '&#39;' }[c]
  ));
}

function toast(message, level = 'info') {
  const node = document.createElement('div');
  node.className = 'toast';
  node.dataset.level = level;
  node.textContent = message;
  el('toasts').appendChild(node);
  setTimeout(() => node.remove(), 3200);
}

function log(message, level = 'info') {
  const stamp = new Date().toLocaleTimeString();
  const line = `[${stamp}] ${escapeHtml(message)}`;
  const cls = level === 'error' ? 'err' : level === 'success' ? 'ok' : level === 'warning' ? 'warn' : '';
  const html = cls ? `<span class="${cls}">${line}</span>` : line;

  for (const box of ['activity', 'logs']) {
    const target = el(box);
    if (!target) continue;
    target.insertAdjacentHTML('beforeend', `${html}\n`);
    target.scrollTop = target.scrollHeight;
  }
}

/** Run a backend command, surfacing failures as a toast plus a log line. */
async function call(command, args, { silent = false } = {}) {
  try {
    return await invoke(command, args);
  } catch (error) {
    const text = typeof error === 'string' ? error : (error?.message ?? String(error));
    if (!silent) {
      toast(text, 'error');
      log(`${command} failed: ${text}`, 'error');
    }
    return null;
  }
}

/* ---------- device card ---------- */

function renderDevice() {
  el('device-name').textContent = state.device.name;
  el('device-addr').textContent = state.device.addr;
  const badge = el('device-badge');
  badge.textContent = state.device.state.toUpperCase();
  badge.dataset.state = state.device.state;
}

function setDevice(name, address, status) {
  state.device = { name: name || state.device.name, addr: address || '—', state: status };
  renderDevice();
}

function renderDeviceList() {
  const list = el('device-list');
  if (!state.devices.length) {
    list.innerHTML = '<li class="empty">Nothing found yet. Try Refresh or Detect.</li>';
    return;
  }
  list.innerHTML = state.devices.map((device, index) => `
    <li>
      <span>
        <strong>${escapeHtml(device.model || device.serial)}</strong><br />
        <span class="muted small mono">${escapeHtml(device.serial)} · ${escapeHtml(device.source)} · ${escapeHtml(device.state)}</span>
      </span>
      <button class="btn tiny" data-pick="${index}">Use</button>
    </li>`).join('');

  list.querySelectorAll('[data-pick]').forEach((button) => {
    button.addEventListener('click', () => {
      const device = state.devices[Number(button.dataset.pick)];
      applyDevice(device);
      toast(`Selected ${device.model || device.serial}`, 'success');
    });
  });
}

function applyDevice(device) {
  state.selected = device;
  if (device.ip) el('ip').value = device.ip;
  if (device.port) el('port').value = device.port;
  setDevice(device.model || device.serial, `${device.ip}:${device.port}`, 'ready');
  refreshQr(device.serial);
}

/* ---------- QR ---------- */

async function refreshQr(serial) {
  if (!serial) return;
  const svg = await call('qr_for', { serial }, { silent: true });
  if (svg) el('qr-slot').innerHTML = svg;
}

/* ---------- settings ---------- */

function applyAppearance() {
  document.documentElement.dataset.theme = state.settings.theme;
  document.documentElement.dataset.accent = state.settings.accent;
  el('theme-toggle').textContent = state.settings.theme === 'dark' ? '☾' : '☀';
}

function renderPresets() {
  el('preset-row').innerHTML = PRESETS.map((preset) => `
    <button class="btn${preset.name === state.settings.quality ? ' is-selected' : ''}"
            role="radio" aria-checked="${preset.name === state.settings.quality}"
            data-preset="${preset.name}">${preset.name}</button>`).join('');

  el('preset-row').querySelectorAll('[data-preset]').forEach((button) => {
    button.addEventListener('click', async () => {
      state.settings.quality = button.dataset.preset;
      renderPresets();
      await call('save_settings', { settings: state.settings });
      toast(`Quality: ${state.settings.quality}`, 'success');
    });
  });

  const active = PRESETS.find((p) => p.name === state.settings.quality);
  el('preset-desc').textContent = active ? `${active.name} — ${active.description}` : '';
}

function renderSwatches() {
  el('swatches').innerHTML = ACCENTS.map((accent) => `
    <button class="swatch${accent === state.settings.accent ? ' is-selected' : ''}"
            role="radio" aria-checked="${accent === state.settings.accent}"
            aria-label="${accent} accent" data-accent="${accent}"
            style="background:${accentColor(accent)}"></button>`).join('');

  el('swatches').querySelectorAll('[data-accent]').forEach((button) => {
    button.addEventListener('click', async () => {
      state.settings.accent = button.dataset.accent;
      applyAppearance();
      renderSwatches();
      await call('save_settings', { settings: state.settings });
    });
  });
}

function accentColor(name) {
  return { pink: '#e94560', cyan: '#00a8cc', purple: '#a855f7', green: '#1fa85c', orange: '#ff6b35' }[name] || '#e94560';
}

async function loadSettings() {
  const stored = await call('get_settings', {}, { silent: true });
  if (stored) state.settings = { ...state.settings, ...stored };
  el('ip').value = state.settings.ip || '';
  el('port').value = state.settings.connect_port || '5555';
  el('pair-port').value = state.settings.pair_port || '5555';
  el('auto-connect').checked = Boolean(state.settings.auto_connect);
  applyAppearance();
  renderPresets();
  renderSwatches();
}

async function persistConnection() {
  state.settings.ip = el('ip').value.trim();
  state.settings.connect_port = el('port').value.trim();
  state.settings.pair_port = el('pair-port').value.trim();
  await call('save_settings', { settings: state.settings }, { silent: true });
}

/* ---------- actions ---------- */

function needDevice() {
  const serial = state.selected?.serial;
  if (!serial) {
    toast('Pick a device first', 'warning');
    log('No device selected. Detect or pick one from the Devices page.', 'warning');
    return null;
  }
  return serial;
}

async function detect() {
  toast('Scanning for devices…');
  log('Scanning for devices…');
  const devices = await call('discover_devices', {});
  if (!devices || !devices.length) {
    toast('No devices found', 'warning');
    log('No devices found. Check USB debugging and the Wi-Fi network.', 'warning');
    return;
  }
  state.devices = devices;
  renderDeviceList();
  const best = devices.find((d) => d.state === 'device' && d.source === 'adb')
    ?? devices.find((d) => d.state === 'device') ?? devices[0];
  applyDevice(best);
  toast(`Found ${devices.length} device(s)`, 'success');
  log(`Found ${devices.length} device(s); using ${best.serial}`, 'success');
}

async function connect() {
  const ip = el('ip').value.trim();
  const port = el('port').value.trim();
  if (!ip) return toast('Enter an IP address', 'warning');
  log(`Connecting to ${ip}:${port}…`);
  const result = await call('connect_device', { ip, port });
  if (result === null) return;
  await persistConnection();
  setDevice(state.device.name, `${ip}:${port}`, 'connected');
  toast('Connected', 'success');
  log(result || `Connected to ${ip}:${port}`, 'success');
}

async function pair() {
  const ip = el('ip').value.trim();
  const port = el('pair-port').value.trim();
  const code = el('pair-code').value.trim();
  if (!ip || !port || !code) return toast('Need IP, pair port and code', 'warning');
  log(`Pairing with ${ip}:${port}…`);
  const result = await call('pair_device', { ip, port, code });
  if (result === null) return;
  toast('Paired', 'success');
  log(result, 'success');
}

async function mirror() {
  const serial = state.selected?.serial || (el('ip').value.trim() ? `${el('ip').value.trim()}:${el('port').value.trim()}` : '');
  if (!serial) return toast('Pick a device first', 'warning');
  log(`Starting scrcpy at ${state.settings.quality} quality…`);
  const result = await call('start_mirror', { serial, preset: state.settings.quality });
  if (result === null) return;
  await persistConnection();
  setDevice(state.device.name, state.device.addr, 'mirroring');
  toast('Mirroring', 'success');
  log(result, 'success');
}

async function quickMirror() {
  toast('Quick Mirror: scanning…');
  log('Quick Mirror: scanning…');
  const result = await call('quick_mirror', { preset: state.settings.quality });
  if (result === null) return;
  const devices = await call('discover_devices', {}, { silent: true });
  if (devices?.length) {
    state.devices = devices;
    renderDeviceList();
    applyDevice(devices.find((d) => d.state === 'device') ?? devices[0]);
  }
  setDevice(state.device.name, state.device.addr, 'mirroring');
  toast('Mirroring', 'success');
  log(result, 'success');
}

async function disconnect() {
  const ip = el('ip').value.trim();
  if (!ip) return toast('No address to disconnect', 'warning');
  await call('disconnect_device', { ip });
  setDevice('No device detected', '—', 'idle');
  toast('Disconnected', 'success');
  log(`Disconnected ${ip}`, 'success');
}

async function runQuickAction(action) {
  const serial = needDevice();
  if (!serial) return;

  if (action === 'screenshot') {
    const path = await call('capture_screen', { serial });
    if (path) { toast('Screenshot saved'); log(`Screenshot saved to ${path}`, 'success'); }
  } else if (action === 'get-clipboard') {
    const text = await call('read_clipboard', { serial });
    if (text) {
      await navigator.clipboard.writeText(text);
      toast(`Copied ${text.length} characters`);
      log(`Copied ${text.length} characters from the device clipboard`, 'success');
    }
  } else if (action === 'set-clipboard') {
    const text = await navigator.clipboard.readText();
    if (!text) return toast('PC clipboard is empty', 'warning');
    const done = await call('send_clipboard', { serial, text });
    if (done !== null) { toast('Sent to device'); log('Clipboard sent to the device', 'success'); }
  } else if (action === 'push-file') {
    const path = await call('pick_file', {}, { silent: true });
    if (!path) return;
    const target = await call('push_file', { serial, path });
    if (target) { toast('File pushed'); log(`Pushed to ${target}`, 'success'); }
  } else if (action === 'device-info') {
    const info = await call('device_info', { serial });
    if (info) { toast(info); log(info); }
  } else if (action === 'restart-adb') {
    await call('restart_adb', {});
    toast('ADB restarted', 'success');
    log('ADB server restarted', 'success');
  }
}

/* ---------- navigation ---------- */

function showPage(key) {
  document.querySelectorAll('.page').forEach((page) => {
    page.classList.toggle('is-active', page.id === `page-${key}`);
  });
  document.querySelectorAll('.rail-item').forEach((item) => {
    const active = item.dataset.page === key;
    item.classList.toggle('is-active', active);
    if (active) item.setAttribute('aria-current', 'page');
    else item.removeAttribute('aria-current');
  });
}

/* ---------- environment ---------- */

async function loadEnvironment() {
  state.env = await call('environment', {}, { silent: true });
  if (!state.env) return;
  el('env-list').innerHTML = `
    <li><span class="k">adb</span><span class="v">${state.env.adb ? 'bundled' : 'not found'}</span></li>
    <li><span class="k">scrcpy</span><span class="v">${state.env.scrcpy ? 'bundled' : 'not found'}</span></li>
    <li><span class="k">Data folder</span><span class="v mono">${escapeHtml(state.env.data_dir)}</span></li>`;
  if (!state.env.adb || !state.env.scrcpy) {
    log('adb or scrcpy is missing. Mirroring will not work until they are present.', 'warning');
  }
}

/* ---------- wiring ---------- */

function wire() {
  document.querySelectorAll('.rail-item').forEach((item) => {
    item.addEventListener('click', () => showPage(item.dataset.page));
  });

  el('win-min').addEventListener('click', () => appWindow.minimize());
  el('win-max').addEventListener('click', () => appWindow.toggleMaximize());
  el('win-close').addEventListener('click', () => appWindow.close());

  el('theme-toggle').addEventListener('click', async () => {
    state.settings.theme = state.settings.theme === 'dark' ? 'light' : 'dark';
    applyAppearance();
    await call('save_settings', { settings: state.settings }, { silent: true });
  });

  el('theme-dark').addEventListener('click', async () => {
    state.settings.theme = 'dark'; applyAppearance();
    await call('save_settings', { settings: state.settings }, { silent: true });
  });
  el('theme-light').addEventListener('click', async () => {
    state.settings.theme = 'light'; applyAppearance();
    await call('save_settings', { settings: state.settings }, { silent: true });
  });

  el('auto-connect').addEventListener('change', async (event) => {
    state.settings.auto_connect = event.target.checked;
    await call('save_settings', { settings: state.settings }, { silent: true });
  });

  el('reset-settings').addEventListener('click', async () => {
    state.settings = {
      ip: '', connect_port: '5555', pair_port: '5555',
      quality: 'Medium', theme: 'dark', accent: 'pink', auto_connect: false,
    };
    el('ip').value = ''; el('port').value = '5555'; el('pair-port').value = '5555';
    el('pair-code').value = ''; el('auto-connect').checked = false;
    applyAppearance(); renderPresets(); renderSwatches();
    await call('save_settings', { settings: state.settings });
    setDevice('No device detected', '—', 'idle');
    toast('Settings reset', 'warning');
    log('Settings reset to defaults', 'warning');
  });

  el('quick-mirror').addEventListener('click', quickMirror);
  el('detect').addEventListener('click', detect);
  el('connect').addEventListener('click', connect);
  el('pair').addEventListener('click', pair);
  el('mirror').addEventListener('click', mirror);
  el('disconnect').addEventListener('click', disconnect);

  el('refresh-devices').addEventListener('click', async () => {
    const devices = await call('list_devices', {});
    state.devices = devices ?? [];
    renderDeviceList();
    toast(`${state.devices.length} device(s)`);
  });

  el('scan-subnet').addEventListener('click', async () => {
    toast('Scanning the subnet…');
    log('Scanning the local subnet for hosts…');
    const alive = await call('scan_subnet', {});
    if (!alive) return;
    log(`Found ${alive.length} host(s): ${alive.join(', ') || 'none'}`, 'success');
    toast(`Found ${alive.length} host(s)`, 'success');
  });

  document.querySelectorAll('[data-act]').forEach((button) => {
    button.addEventListener('click', () => runQuickAction(button.dataset.act));
  });

  el('clear-log').addEventListener('click', () => { el('activity').textContent = ''; });
  el('clear-logs').addEventListener('click', () => { el('logs').textContent = ''; });

  el('open-shots').addEventListener('click', async () => {
    if (state.env?.data_dir) await call('open_folder', { path: `${state.env.data_dir}\\screenshots` });
  });
  el('open-config').addEventListener('click', async () => {
    if (state.env?.data_dir) await call('open_folder', { path: state.env.data_dir });
  });

  document.addEventListener('keydown', (event) => {
    if (event.key === 'Escape') appWindow.close();
    if (event.key === 'F11') { event.preventDefault(); appWindow.toggleMaximize(); }
    if (event.ctrlKey && event.key >= '1' && event.key <= '6') {
      event.preventDefault();
      const order = ['dashboard', 'devices', 'actions', 'settings', 'logs', 'about'];
      showPage(order[Number(event.key) - 1]);
    }
  });

  document.querySelectorAll('a[data-external]').forEach((link) => {
    link.addEventListener('click', (event) => {
      event.preventDefault();
      call('open_folder', { path: link.href });
    });
  });
}

async function start() {
  wire();
  await loadSettings();
  await loadEnvironment();
  renderDevice();
  log('MirrorPy ready');
  if (state.settings.auto_connect) {
    log('Auto-connect is on, looking for a device…');
    detect();
  }
}

window.addEventListener('DOMContentLoaded', start);
