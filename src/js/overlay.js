const { invoke } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;

const bars = [...document.querySelectorAll('.bar')];
const cross = document.getElementById('cross');
const spinner = document.getElementById('spinner');

cross.addEventListener('click', () => {
  invoke('cancel_recording');
});

function setAmplitude(a) {
  const h = [.35, .6, 1, 1, .6, .35];
  bars.forEach((b, i) => {
    b.style.height = Math.max(3, Math.min(20, a * h[i] * 26)) + 'px';
  });
}

function showHold() {
  bars.forEach(b => b.style.display = 'block');
  cross.style.display = 'block';
  spinner.style.display = 'none';
}

function showToggle() {
  bars.forEach(b => b.style.display = 'block');
  cross.style.display = 'block';
  spinner.style.display = 'none';
}

function showProcessing() {
  bars.forEach(b => b.style.display = 'none');
  cross.style.display = 'none';
  spinner.style.display = 'block';
}

listen('amplitude-update', (event) => {
  setAmplitude(event.payload);
});

listen('show-hold', () => { showHold(); });
listen('show-toggle', () => { showToggle(); });
listen('show-processing', () => { showProcessing(); });
