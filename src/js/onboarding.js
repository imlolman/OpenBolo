const { invoke } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;

let step = 0;
let selectedMic = null;
let holdShortcut = 'mouse:middle';
let toggleShortcut = 'key:Alt_R';
let capturing = null;
let modelReady = false;
let platform = 'macos';

const CODE_MAP = {
  AltLeft:'key:Alt_L', AltRight:'key:Alt_R',
  ControlLeft:'key:Control_L', ControlRight:'key:Control_R',
  ShiftLeft:'key:Shift_L', ShiftRight:'key:Shift_R',
  MetaLeft:'key:Super_L', MetaRight:'key:Super_R',
  Space:'key:space', Enter:'key:Return', Escape:'key:Escape',
  Backspace:'key:BackSpace', Tab:'key:Tab', CapsLock:'key:Caps_Lock',
  Delete:'key:Delete',
};
for (let i = 1; i <= 12; i++) CODE_MAP['F' + i] = 'key:F' + i;
for (let i = 0; i <= 9; i++) CODE_MAP['Digit' + i] = 'key:' + i;
'ABCDEFGHIJKLMNOPQRSTUVWXYZ'.split('').forEach(c => { CODE_MAP['Key' + c] = 'key:' + c.toLowerCase(); });

function keyDisp(code) {
  if (platform === 'macos') {
    const m = {
      ControlLeft:'\u2303', ControlRight:'\u2303',
      ShiftLeft:'\u21e7', ShiftRight:'\u21e7',
      AltLeft:'\u2325', AltRight:'\u2325',
      MetaLeft:'\u2318', MetaRight:'\u2318',
      Escape:'Esc', Enter:'\u21a9', Backspace:'\u232b',
      Delete:'\u2326', Tab:'\u21e5', Space:'\u2423',
    };
    if (m[code]) return m[code];
  } else {
    const m = {
      ControlLeft: 'Ctrl', ControlRight: 'Ctrl',
      ShiftLeft: 'Shift', ShiftRight: 'Shift',
      AltLeft: 'Alt', AltRight: 'Alt',
      MetaLeft: platform === 'windows' ? 'Win' : 'Super',
      MetaRight: platform === 'windows' ? 'Win' : 'Super',
      Escape: 'Esc', Enter: 'Enter', Backspace: 'Backspace',
      Delete: 'Del', Tab: 'Tab', Space: 'Space',
    };
    if (m[code]) return m[code];
  }
  if (code.startsWith('Key')) return code.slice(3);
  if (code.startsWith('Digit')) return code.slice(5);
  return code;
}

async function init() {
  platform = await invoke('get_platform');
  
  const mics = await invoke('get_microphones');
  const list = document.getElementById('micList');
  list.innerHTML = '';
  mics.forEach((m, i) => {
    const div = document.createElement('div');
    div.className = 'mic-item' + (m.is_default ? ' sel' : '');
    div.innerHTML = `<div class="mic-radio"></div><span>${m.name}${m.is_default ? ' (Default)' : ''}</span>`;
    div.onclick = () => {
      list.querySelectorAll('.mic-item').forEach(el => el.classList.remove('sel'));
      div.classList.add('sel');
      selectedMic = m.name;
      testMic();
    };
    if (m.is_default) selectedMic = m.name;
    list.appendChild(div);
  });

  invoke('request_mic_permission');
  setTimeout(() => {
    document.getElementById('permStatus').innerHTML = '<span class="perm-badge perm-ok">Microphone access granted</span>';
    testMic();
  }, 1500);

  checkAndDownloadModel();
  updatePlatformLabels();
}

async function testMic() {
  document.getElementById('levelWrap').style.display = 'block';
  try {
    const peak = await invoke('test_mic', { device: selectedMic });
    const pct = Math.min(100, Math.round(peak * 100 * 3));
    document.getElementById('levelFill').style.width = pct + '%';
  } catch (e) {
    document.getElementById('levelFill').style.width = '0%';
  }
}

async function checkAndDownloadModel() {
  const exists = await invoke('check_model_exists');
  if (exists) {
    modelReady = true;
    return;
  }
  document.getElementById('dlProgress').style.display = 'block';
  document.getElementById('nextBtn').disabled = true;

  listen('model-download-progress', (event) => {
    const [downloaded, total] = event.payload;
    if (total > 0) {
      const pct = Math.round((downloaded / total) * 100);
      document.getElementById('dlFill').style.width = pct + '%';
      document.getElementById('dlText').textContent = `Downloading speech model... ${pct}%`;
    }
  });

  try {
    await invoke('download_model');
    document.getElementById('dlText').textContent = 'Model downloaded!';
    document.getElementById('dlFill').style.width = '100%';
    modelReady = true;
    document.getElementById('nextBtn').disabled = false;
  } catch (e) {
    document.getElementById('dlText').textContent = 'Download failed: ' + e;
  }
}

function showStep(n) {
  document.querySelectorAll('.step').forEach(s => s.classList.remove('active'));
  document.getElementById('step' + (n + 1)).classList.add('active');
  for (let i = 0; i < 3; i++) {
    document.getElementById('d' + i).classList.toggle('on', i === n);
  }
  document.getElementById('backBtn').style.visibility = n === 0 ? 'hidden' : 'visible';
  if (n === 2) {
    document.getElementById('nextBtn').textContent = 'Start App';
  } else {
    document.getElementById('nextBtn').textContent = 'Continue';
  }
}

function next() {
  if (step === 0) {
    if (!modelReady) return;
    step = 1;
    showStep(1);
  } else if (step === 1) {
    step = 2;
    showStep(2);
  } else if (step === 2) {
    invoke('finish_onboarding', { mic: selectedMic, hold: holdShortcut, toggle: toggleShortcut });
  }
}

function back() {
  if (step > 0) {
    step--;
    showStep(step);
  }
}

function capture(which) {
  if (capturing) return;
  capturing = which;
  const disp = document.getElementById(which + 'Disp');
  const btn = document.getElementById(which + 'Btn');
  const prevText = disp.textContent;
  btn.textContent = 'Press shortcut...';
  btn.classList.add('cap');
  disp.textContent = '\u00a0';
  document.querySelectorAll('.btn').forEach(b => { if (b !== btn) b.style.pointerEvents = 'none'; });

  let done = false;
  const MODS = new Set(['AltRight','AltLeft','ControlRight','ControlLeft','MetaRight','MetaLeft','ShiftRight','ShiftLeft']);
  const MOD_ORD = ['ControlLeft','ControlRight','ShiftLeft','ShiftRight','AltLeft','AltRight','MetaLeft','MetaRight'];
  let held = new Set(), peak = new Set(), timer = null;

  function show() {
    const p = [];
    for (const m of MOD_ORD) if (held.has(m)) p.push(keyDisp(m));
    disp.textContent = p.length ? p.join(' + ') : '\u00a0';
  }

  function onKD(e) {
    e.preventDefault(); e.stopPropagation();
    if (e.repeat || done) return;
    if (timer) { clearTimeout(timer); timer = null; }
    const c = e.code;
    if (!CODE_MAP[c]) return;
    if (MODS.has(c)) {
      held.add(c);
      if (held.size > peak.size) peak = new Set(held);
      show();
    } else {
      if (c === 'Escape' && held.size === 0) { finish(null); return; }
      const p = [];
      for (const m of MOD_ORD) if (held.has(m)) { const s = CODE_MAP[m]; if (s) p.push(s.split(':')[1]); }
      const ts = CODE_MAP[c];
      if (ts) p.push(ts.split(':')[1]);
      if (!p.length) return;
      disp.textContent = [...Array.from(held).map(m => keyDisp(m)), keyDisp(c)].join(' + ');
      finish(p.length === 1 ? CODE_MAP[c] : 'combo:' + p.join('+'));
    }
  }

  function onKU(e) {
    e.preventDefault(); e.stopPropagation();
    if (done) return;
    if (!MODS.has(e.code)) return;
    held.delete(e.code);
    if (held.size === 0) {
      if (timer) clearTimeout(timer);
      timer = setTimeout(() => {
        const p = [];
        for (const m of MOD_ORD) if (peak.has(m)) { const s = CODE_MAP[m]; if (s) p.push(s.split(':')[1]); }
        if (!p.length) return;
        finish(p.length === 1 ? CODE_MAP[[...peak][0]] : 'combo:' + p.join('+'));
      }, 200);
      return;
    }
    show();
  }

  function finish(sc) {
    if (done) return;
    done = true;
    if (timer) { clearTimeout(timer); timer = null; }
    capturing = null;
    document.removeEventListener('keydown', onKD, true);
    document.removeEventListener('keyup', onKU, true);
    invoke('cancel_capture');
    btn.textContent = 'Set';
    btn.classList.remove('cap');
    document.querySelectorAll('.btn').forEach(b => b.style.pointerEvents = '');
    if (sc) {
      if (which === 'hold') holdShortcut = sc;
      else toggleShortcut = sc;
      invoke('shortcut_display_name', { value: sc }).then(n => { disp.textContent = n; });
    } else {
      disp.textContent = prevText;
    }
  }

  document.addEventListener('keydown', onKD, true);
  document.addEventListener('keyup', onKU, true);
  invoke('capture_mouse').then(s => { if (s) finish(s); });
}

function updatePlatformLabels() {
  if (platform !== 'macos') {
    document.getElementById('toggleDisp').textContent = 'Right Alt';
    document.getElementById('doneSubtext').innerHTML = 'OpenBolo runs in your system tray.<br>Use your shortcuts to start dictating.';
  }
}

init();
