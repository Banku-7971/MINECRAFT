// RustCraft WebAssembly client with live Resource/Shader/Datapack/Modpack loader.
// Vercel only serves these static files; the game itself runs in the browser.

const canvas = document.getElementById('screen');
const start = document.getElementById('start');
const play = document.getElementById('play');
const statustext = document.getElementById('statustext');

const packsModal = document.getElementById('packs-modal');
const openPacksTitle = document.getElementById('open-packs-title');
const openPacksSide = document.getElementById('open-packs-side');
const closePacksBtn = document.getElementById('close-packs');
const resetPacksBtn = document.getElementById('reset-packs');
const packFileInput = document.getElementById('pack-file');
const dropzone = document.getElementById('dropzone');
const packUrlInput = document.getElementById('pack-url');
const loadUrlBtn = document.getElementById('load-url');
const packToast = document.getElementById('pack-toast');
const packSummary = document.getElementById('pack-summary');
const sideShaderInfo = document.getElementById('side-shader-info');
const modrinthQ = document.getElementById('modrinth-q');
const modrinthGo = document.getElementById('modrinth-go');
const modrinthList = document.getElementById('modrinth-list');
const activeList = document.getElementById('active-list');
const modsHeading = document.getElementById('mods-heading');
const modsList = document.getElementById('mods-list');
const frameWrap = document.getElementById('frame-wrap');

const ctx = canvas.getContext('2d', { alpha: false });
ctx.imageSmoothingEnabled = false;
canvas.tabIndex = 0;

let wasm = null;
let imageData = null;
let ready = false;
let playing = false;
let pointerLocked = false;
let lockPending = false;
let captureRequested = false;
let chatOpen = false;
let packsOpen = false;
let firstFrame = true;
let activeModrinthType = 'resourcepack';
let activeMods = [];

// WebAudio Synthesizer (Title/In-Game calm piano music + material footsteps + mob/combat SFX)
let audioCtx = null;
let musicNextAt = 0;
let musicStep = 0;
const MUSIC_MELODY = [523.25, 392.0, 329.63, 392.0, 440.0, 392.0, 329.63, 261.63, 392.0, 329.63, 261.63, 329.63, 440.0, 523.25, 440.0, 392.0];
const MUSIC_CHORDS = [[261.63, 329.63, 392.0], [220.0, 261.63, 329.63], [174.61, 220.0, 261.63], [196.0, 246.94, 293.66]];

function ensureAudio() {
  if (!audioCtx) {
    const AC = window.AudioContext || window.webkitAudioContext;
    if (!AC) return null;
    audioCtx = new AC();
  }
  if (audioCtx.state === 'suspended') {
    audioCtx.resume().catch(() => {});
  }
  return audioCtx;
}

function playTone(freq, dur, vol, type = 'sine', slideTo = null) {
  const ctx = ensureAudio();
  if (!ctx || ctx.state !== 'running') return;
  const t = ctx.currentTime;
  const osc = ctx.createOscillator();
  const gain = ctx.createGain();
  osc.type = type;
  osc.frequency.setValueAtTime(freq, t);
  if (slideTo) osc.frequency.exponentialRampToValueAtTime(Math.max(40, slideTo), t + dur);
  gain.gain.setValueAtTime(0.0001, t);
  gain.gain.linearRampToValueAtTime(vol, t + 0.01);
  gain.gain.exponentialRampToValueAtTime(0.0001, t + dur);
  osc.connect(gain);
  gain.connect(ctx.destination);
  osc.start(t);
  osc.stop(t + dur + 0.02);
}

function playNoise(dur, vol, filterFreq = 900) {
  const ctx = ensureAudio();
  if (!ctx || ctx.state !== 'running') return;
  const len = Math.max(1, Math.floor(ctx.sampleRate * dur));
  const buf = ctx.createBuffer(1, len, ctx.sampleRate);
  const data = buf.getChannelData(0);
  for (let i = 0; i < len; i++) data[i] = Math.random() * 2 - 1;
  const src = ctx.createBufferSource();
  src.buffer = buf;
  const flt = ctx.createBiquadFilter();
  flt.type = 'lowpass';
  flt.frequency.value = filterFreq;
  const gain = ctx.createGain();
  const t = ctx.currentTime;
  gain.gain.setValueAtTime(vol, t);
  gain.gain.exponentialRampToValueAtTime(0.0001, t + dur);
  src.connect(flt);
  flt.connect(gain);
  gain.connect(ctx.destination);
  src.start(t);
}

function playWebSfx(id) {
  switch (id) {
    case 0: playTone(1200, 0.04, 0.06, 'triangle'); break; // Click
    case 1: playTone(175, 0.08, 0.14, 'triangle', 95); playNoise(0.06, 0.06, 600); break; // Place
    case 2: playNoise(0.14, 0.16, 1200); playTone(140, 0.09, 0.09, 'square', 65); break; // Break
    case 3:
    case 4: playNoise(0.08, 0.08, 650); playTone(105, 0.05, 0.06, 'sine', 70); break; // StepGrass
    case 5: playTone(420, 0.05, 0.09, 'triangle', 220); playNoise(0.05, 0.07, 1800); break; // StepStone
    case 6: playTone(215, 0.07, 0.10, 'triangle', 140); break; // StepWood
    case 7: playNoise(0.10, 0.08, 2200); break; // StepSand
    case 8: playNoise(0.15, 0.09, 850); break; // StepWater
    case 9: playTone(165, 0.10, 0.08, 'sine', 330); break; // Jump
    case 10: playTone(95, 0.10, 0.11, 'triangle', 55); break; // Land
    case 11: playTone(300, 0.18, 0.14, 'sawtooth', 110); break; // Hurt
    case 12: playTone(250, 0.14, 0.14, 'triangle', 95); playNoise(0.08, 0.09, 1100); break; // MobHurt
    case 13: playTone(340, 0.24, 0.12, 'triangle', 80); break; // MobDie
    case 14: playTone(880, 0.06, 0.07, 'square', 1180); break; // Pickup
    case 15: playTone(660, 0.08, 0.07, 'square'); break; // Craft
    case 17: playNoise(0.45, 0.22, 420); playTone(65, 0.35, 0.16, 'sawtooth', 40); break; // Explode
    case 19: playTone(105, 0.42, 0.10, 'sawtooth', 85); break; // Zombie
    case 20: playTone(620, 0.12, 0.07, 'square', 440); break; // Skeleton
    case 21: playTone(148, 0.45, 0.11, 'sawtooth', 118); break; // Cow
    case 22: playTone(290, 0.34, 0.09, 'triangle', 260); break; // Sheep
    case 23: playTone(195, 0.20, 0.10, 'triangle', 250); break; // Pig
    case 24: playTone(690, 0.12, 0.07, 'triangle', 840); break; // Chicken
    case 25: playNoise(0.18, 0.09, 1900); break; // Spider
    case 26: playNoise(0.40, 0.12, 2400); break; // Creeper
    case 27: playTone(240, 0.28, 0.09, 'sawtooth', 150); break; // Enderman
    case 29: playTone(523, 0.09, 0.08, 'triangle'); setTimeout(() => playTone(784, 0.12, 0.08, 'triangle'), 80); break; // LevelUp
    default: playNoise(0.08, 0.06, 900); break;
  }
}

function pumpAudio() {
  if (!wasm) return;
  const flags = wasm.game_audio_flags ? wasm.game_audio_flags() : 3;
  const musicOn = (flags & 1) !== 0;
  const soundOn = (flags & 2) !== 0;
  if (wasm.game_audio_pop) {
    let s;
    while ((s = wasm.game_audio_pop()) >= 0) {
      if (soundOn) playWebSfx(s);
    }
  }
  if (musicOn && audioCtx && audioCtx.state === 'running') {
    const now = audioCtx.currentTime;
    if (now >= musicNextAt) {
      musicNextAt = now + 0.92;
      const mel = MUSIC_MELODY[musicStep % MUSIC_MELODY.length];
      playTone(mel, 1.35, 0.045, 'sine');
      if (musicStep % 4 === 0) {
        const chord = MUSIC_CHORDS[(musicStep / 4) % MUSIC_CHORDS.length];
        for (const f of chord) playTone(f * 0.5, 2.4, 0.028, 'sine');
      }
      musicStep++;
    }
  }
}

['pointerdown', 'keydown'].forEach(ev => {
  window.addEventListener(ev, () => ensureAudio(), { passive: true });
});

const VK = {
  Backspace: 8, Tab: 9, Enter: 13,
  ShiftLeft: 16, ShiftRight: 16, ControlLeft: 17, ControlRight: 17,
  Escape: 27, Space: 32,
  ArrowLeft: 37, ArrowUp: 38, ArrowRight: 39, ArrowDown: 40,
  KeyA: 65, KeyC: 67, KeyD: 68, KeyE: 69, KeyF: 70, KeyQ: 81, KeyS: 83, KeyT: 84, KeyW: 87,
  F5: 116,
  Digit1: 49, Digit2: 50, Digit3: 51, Digit4: 52, Digit5: 53,
  Digit6: 54, Digit7: 55, Digit8: 56, Digit9: 57,
  Numpad1: 97, Numpad2: 98, Numpad3: 99, Numpad4: 100, Numpad5: 101,
  Numpad6: 102, Numpad7: 103, Numpad8: 104, Numpad9: 105,
};

const preventDefault = new Set([
  'Space', 'Tab', 'F5',
  'ArrowUp', 'ArrowDown', 'ArrowLeft', 'ArrowRight',
]);

function setStatus(text) {
  statustext.textContent = text;
}

function showToast(msg, isErr = false) {
  packToast.textContent = msg;
  packToast.classList.toggle('err', !!isErr);
}

function readWasmStatus() {
  if (!wasm) return null;
  const ptr = wasm.game_status_ptr();
  const len = wasm.game_status_len();
  if (!ptr || !len) return null;
  const slice = new Uint8Array(wasm.memory.buffer, ptr, len);
  const txt = new TextDecoder().decode(slice);
  try {
    return JSON.parse(txt);
  } catch {
    return null;
  }
}

function refreshPackUi(statusObj) {
  if (!statusObj) {
    if (wasm) {
      wasm.game_query_packs();
      statusObj = readWasmStatus();
    }
    if (!statusObj) return;
  }
  const shaderName = statusObj.shader || 'VANILLA';
  const packs = Array.isArray(statusObj.packs) ? statusObj.packs : [];
  packSummary.textContent = `Shader: ${shaderName} \u2022 ${packs.length} pack(s) active`;
  sideShaderInfo.textContent = `Shader: ${shaderName} \u2022 ${packs.length} pack(s)`;

  document.querySelectorAll('.shader-chip').forEach(btn => {
    const label = btn.textContent.toUpperCase();
    const match = (shaderName === 'VANILLA' && btn.dataset.shader === '0')
      || shaderName.includes(label.split(' ')[0].replace("'", ''));
    btn.classList.toggle('active', match);
  });

  if (packs.length === 0) {
    activeList.innerHTML = '<div class="empty-msg">No custom packs loaded yet.</div>';
  } else {
    activeList.innerHTML = '';
    for (const p of packs) {
      const row = document.createElement('div');
      row.className = 'pack-item';
      const meta = document.createElement('div');
      meta.className = 'pack-meta';
      const title = document.createElement('div');
      title.className = 'pack-title';
      const badge = document.createElement('span');
      badge.className = 'pack-badge';
      badge.textContent = p.kind;
      title.appendChild(badge);
      title.appendChild(document.createTextNode(p.name));
      const desc = document.createElement('div');
      desc.className = 'pack-desc';
      desc.textContent = p.desc || `${p.textures} tex, ${p.recipes} recipes`;
      meta.appendChild(title);
      meta.appendChild(desc);
      row.appendChild(meta);
      activeList.appendChild(row);
    }
  }

  if (statusObj.latest && Array.isArray(statusObj.latest.mods) && statusObj.latest.mods.length > 0) {
    activeMods = statusObj.latest.mods;
  }
  if (activeMods.length > 0) {
    modsHeading.classList.remove('hidden');
    modsList.classList.remove('hidden');
    modsList.innerHTML = '';
    for (const m of activeMods) {
      const row = document.createElement('div');
      row.className = 'pack-item';
      const meta = document.createElement('div');
      meta.className = 'pack-meta';
      const title = document.createElement('div');
      title.className = 'pack-title';
      title.textContent = m.file;
      meta.appendChild(title);
      const link = document.createElement('a');
      link.href = m.url;
      link.target = '_blank';
      link.rel = 'noopener noreferrer';
      link.textContent = 'Jar';
      link.style.cssText = 'font-size:11px;color:#8ad0ff;text-decoration:none;';
      row.appendChild(meta);
      row.appendChild(link);
      modsList.appendChild(row);
    }
  } else {
    modsHeading.classList.add('hidden');
    modsList.classList.add('hidden');
  }
}

function loadPackBytesToWasm(filename, bytes) {
  if (!wasm) return false;
  const nameBytes = new TextEncoder().encode(filename || 'pack.zip');
  const nPtr = wasm.game_alloc(nameBytes.length);
  const dPtr = wasm.game_alloc(bytes.length);
  new Uint8Array(wasm.memory.buffer, nPtr, nameBytes.length).set(nameBytes);
  new Uint8Array(wasm.memory.buffer, dPtr, bytes.length).set(bytes);
  const ok = wasm.game_load_pack(nPtr, nameBytes.length, dPtr, bytes.length);
  wasm.game_free(nPtr, nameBytes.length);
  wasm.game_free(dPtr, bytes.length);
  const statusObj = readWasmStatus();
  refreshPackUi(statusObj);
  if (statusObj) {
    showToast(statusObj.message, !ok);
  }
  return !!ok;
}

async function handleFiles(fileList) {
  for (const file of fileList) {
    showToast(`Loading ${file.name}...`);
    const buf = new Uint8Array(await file.arrayBuffer());
    loadPackBytesToWasm(file.name, buf);
  }
}

async function loadFromUrl(url, suggestedName) {
  if (!url) return;
  try {
    showToast(`Downloading ${suggestedName || url}...`);
    const resp = await fetch(url);
    if (!resp.ok) throw new Error(`HTTP ${resp.status}`);
    const buf = new Uint8Array(await resp.arrayBuffer());
    const fname = suggestedName || url.split('/').pop().split('?')[0] || 'pack.zip';
    loadPackBytesToWasm(fname, buf);
  } catch (err) {
    showToast(`Download failed (${err.message}). If the host blocks CORS, download the file and drop it here.`, true);
  }
}

async function searchModrinth() {
  const q = modrinthQ.value.trim();
  modrinthList.innerHTML = '<div class="empty-msg">Searching Modrinth...</div>';
  try {
    const facets = encodeURIComponent(JSON.stringify([[`project_type:${activeModrinthType}`]]));
    const url = `https://api.modrinth.com/v2/search?limit=15&facets=${facets}&query=${encodeURIComponent(q)}`;
    const resp = await fetch(url);
    if (!resp.ok) throw new Error(`HTTP ${resp.status}`);
    const data = await resp.json();
    const hits = data.hits || [];
    if (hits.length === 0) {
      modrinthList.innerHTML = '<div class="empty-msg">No matching packs found on Modrinth.</div>';
      return;
    }
    modrinthList.innerHTML = '';
    for (const h of hits) {
      const row = document.createElement('div');
      row.className = 'pack-item';
      const meta = document.createElement('div');
      meta.className = 'pack-meta';
      const title = document.createElement('div');
      title.className = 'pack-title';
      title.textContent = h.title || h.slug;
      const desc = document.createElement('div');
      desc.className = 'pack-desc';
      desc.textContent = `${h.author || 'Modrinth'} \u2022 ${(h.downloads || 0).toLocaleString()} downloads \u2022 ${h.description || ''}`;
      meta.appendChild(title);
      meta.appendChild(desc);

      const btn = document.createElement('button');
      btn.className = 'btn-small';
      btn.textContent = 'Install';
      btn.addEventListener('click', async () => {
        btn.disabled = true;
        btn.textContent = '...';
        try {
          const vResp = await fetch(`https://api.modrinth.com/v2/project/${encodeURIComponent(h.project_id)}/version`);
          if (!vResp.ok) throw new Error(`HTTP ${vResp.status}`);
          const versions = await vResp.json();
          const ver = versions[0];
          const file = ver && ver.files && (ver.files.find(f => f.primary) || ver.files[0]);
          if (!file || !file.url) throw new Error('No downloadable file found');
          await loadFromUrl(file.url, file.filename || `${h.slug}.zip`);
          btn.textContent = 'Loaded';
        } catch (err) {
          showToast(`Install failed: ${err.message}`, true);
          btn.textContent = 'Retry';
        } finally {
          btn.disabled = false;
        }
      });

      row.appendChild(meta);
      row.appendChild(btn);
      modrinthList.appendChild(row);
    }
  } catch (err) {
    modrinthList.innerHTML = `<div class="empty-msg">Modrinth search unavailable (${err.message}). Drop a local .zip/.mrpack above!</div>`;
  }
}

function openPacksModal() {
  packsOpen = true;
  if (document.pointerLockElement === canvas) {
    document.exitPointerLock?.();
  }
  packsModal.classList.remove('hidden');
  refreshPackUi();
  if (modrinthList.children.length === 0) {
    searchModrinth();
  }
}

function closePacksModal() {
  packsOpen = false;
  packsModal.classList.add('hidden');
  if (playing) canvas.focus();
}

openPacksTitle.addEventListener('click', openPacksModal);
openPacksSide.addEventListener('click', openPacksModal);
closePacksBtn.addEventListener('click', closePacksModal);

resetPacksBtn.addEventListener('click', () => {
  if (!wasm) return;
  activeMods = [];
  wasm.game_reset_packs();
  const s = readWasmStatus();
  refreshPackUi(s);
  if (s) showToast(s.message, false);
});

document.querySelectorAll('.shader-chip').forEach(btn => {
  btn.addEventListener('click', () => {
    if (!wasm) return;
    wasm.game_set_shader(Number(btn.dataset.shader || 0));
    const s = readWasmStatus();
    refreshPackUi(s);
    if (s) showToast(s.message, false);
  });
});

document.querySelectorAll('.tab').forEach(tab => {
  tab.addEventListener('click', () => {
    document.querySelectorAll('.tab').forEach(t => t.classList.remove('active'));
    tab.classList.add('active');
    activeModrinthType = tab.dataset.ptype || 'resourcepack';
    searchModrinth();
  });
});

modrinthGo.addEventListener('click', searchModrinth);
modrinthQ.addEventListener('keydown', e => {
  if (e.key === 'Enter') searchModrinth();
});

loadUrlBtn.addEventListener('click', () => loadFromUrl(packUrlInput.value.trim()));
packFileInput.addEventListener('change', e => {
  if (e.target.files && e.target.files.length > 0) {
    handleFiles(e.target.files);
    e.target.value = '';
  }
});

[dropzone, frameWrap].forEach(el => {
  el.addEventListener('dragover', e => {
    e.preventDefault();
    dropzone.classList.add('dragover');
  });
  el.addEventListener('dragleave', () => dropzone.classList.remove('dragover'));
  el.addEventListener('drop', e => {
    e.preventDefault();
    dropzone.classList.remove('dragover');
    if (e.dataTransfer?.files?.length) {
      if (!packsOpen) openPacksModal();
      handleFiles(e.dataTransfer.files);
    }
  });
});

function setCapture(captured) {
  captureRequested = !!captured;
  if (!wasm || packsOpen) return;
  if (captured) {
    if (!pointerLocked && !lockPending && document.pointerLockElement !== canvas) {
      lockPending = true;
      try {
        const result = canvas.requestPointerLock();
        if (result && typeof result.catch === 'function') result.catch(() => {});
      } catch (_) {
        lockPending = false;
      }
    }
  } else if (document.pointerLockElement === canvas) {
    document.exitPointerLock?.();
  }
}

function setCursor(visible) {
  canvas.style.cursor = visible ? 'default' : 'none';
}

function canvasPoint(e) {
  const r = canvas.getBoundingClientRect();
  return {
    x: Math.max(0, Math.min(479, Math.floor((e.clientX - r.left) / r.width * 480))),
    y: Math.max(0, Math.min(269, Math.floor((e.clientY - r.top) / r.height * 270))),
  };
}

function drawFrame() {
  if (!wasm || !imageData) return;
  const len = wasm.game_frame_len();
  if (len !== 480 * 270 * 4) return;
  const ptr = wasm.game_frame_ptr();
  if (!ptr) return;
  const pixels = new Uint8ClampedArray(wasm.memory.buffer, ptr, len);
  imageData.data.set(pixels);
  ctx.putImageData(imageData, 0, 0);
}

function loop() {
  if (!wasm) return;
  wasm.game_frame();
  drawFrame();
  pumpAudio();
  if (firstFrame) {
    firstFrame = false;
    setStatus('Ready - click to play');
  }
  requestAnimationFrame(loop);
}

async function begin() {
  if (!ready || !wasm) return;

  if (document.pointerLockElement !== canvas) {
    try {
      const result = canvas.requestPointerLock();
      if (result && typeof result.catch === 'function') result.catch(() => {});
    } catch (_) {}
  }

  start.classList.add('hidden');
  playing = true;
  chatOpen = false;
  setStatus('Generating world...');
  canvas.focus();

  await new Promise(resolve => requestAnimationFrame(() => requestAnimationFrame(resolve)));

  wasm.game_mouse_to(240, 126);
  wasm.game_mouse_button(0, 1);
  wasm.game_mouse_button(0, 0);
  wasm.game_frame();
  drawFrame();

  setStatus(pointerLocked ? 'Mouse captured - Esc to pause' : 'Click the world to capture the mouse');
}

play.addEventListener('click', begin);

canvas.addEventListener('contextmenu', e => e.preventDefault());

canvas.addEventListener('mousedown', e => {
  if (!playing || !wasm || packsOpen) return;
  e.preventDefault();
  if (!pointerLocked && !captureRequested) {
    const p = canvasPoint(e);
    wasm.game_mouse_to(p.x, p.y);
  }
  if (e.button === 0) wasm.game_mouse_button(0, 1);
  else if (e.button === 2) wasm.game_mouse_button(1, 1);
});

canvas.addEventListener('mouseup', e => {
  if (!wasm || packsOpen) return;
  if (e.button === 0) wasm.game_mouse_button(0, 0);
  else if (e.button === 2) wasm.game_mouse_button(1, 0);
});

canvas.addEventListener('mousemove', e => {
  if (!playing || !wasm || packsOpen) return;
  if (pointerLocked || captureRequested) {
    wasm.game_mouse_move(e.movementX || 0, e.movementY || 0);
  } else {
    const p = canvasPoint(e);
    wasm.game_mouse_to(p.x, p.y);
  }
});

canvas.addEventListener('wheel', e => {
  if (!playing || !wasm || packsOpen) return;
  e.preventDefault();
  wasm.game_wheel(e.deltaY > 0 ? 1 : -1);
}, { passive: false });

function sendKey(code, down) {
  if (wasm) wasm.game_key(code, down ? 1 : 0);
}

function sendChar(ch) {
  if (wasm && ch.length === 1) wasm.game_char(ch.charCodeAt(0));
}

document.addEventListener('keydown', e => {
  if (packsOpen) {
    if (e.code === 'Escape') {
      e.preventDefault();
      closePacksModal();
    }
    return;
  }
  if (!playing || !wasm) return;

  const isF5 = e.code === 'F5' || e.key === 'F5' || e.keyCode === 116;
  if (isF5 || preventDefault.has(e.code)) {
    e.preventDefault();
    e.stopPropagation();
  }
  if (e.ctrlKey || e.metaKey || e.altKey) return;

  const code = isF5 ? 116 : VK[e.code];
  if (code === undefined) {
    if (!e.repeat) sendChar(e.key);
    return;
  }

  if (e.repeat) {
    if (e.code === 'Backspace') sendKey(code, true);
    else if (chatOpen) sendChar(e.key);
    return;
  }

  if (e.code === 'KeyT' && !chatOpen) {
    chatOpen = true;
    sendKey(code, true);
    return;
  }
  if (chatOpen && (e.code === 'Enter' || e.code === 'Escape')) {
    chatOpen = false;
    sendKey(code, true);
    return;
  }

  sendKey(code, true);
  if (e.key.length === 1) sendChar(e.key);
});

document.addEventListener('keyup', e => {
  if (!playing || !wasm || packsOpen) return;
  const code = VK[e.code];
  if (code !== undefined) sendKey(code, false);
});

window.addEventListener('blur', () => {
  chatOpen = false;
  if (wasm && playing) wasm.game_blur();
});

document.addEventListener('pointerlockchange', () => {
  const wasLocked = pointerLocked;
  lockPending = false;
  pointerLocked = document.pointerLockElement === canvas;
  if (wasLocked && !pointerLocked && wasm && playing) {
    wasm.game_blur();
  }
  if (pointerLocked) setStatus('Mouse captured - Esc to pause');
});

document.addEventListener('pointerlockerror', () => {
  lockPending = false;
  if (playing) {
    setStatus('Pointer lock unavailable - move the mouse to look, Esc to pause');
  } else {
    setStatus('Ready - click to play');
  }
});

async function boot() {
  try {
    const response = await fetch('/rustcraft.wasm', { cache: 'no-store' });
    if (!response.ok) throw new Error('HTTP ' + response.status);
    const bytes = await response.arrayBuffer();

    const imports = {
      env: {
        js_now: () => performance.now() / 1000,
        js_set_capture: captured => setCapture(!!captured),
        js_set_cursor: visible => setCursor(!!visible),
      },
    };

    const { instance } = await WebAssembly.instantiate(bytes, imports);
    wasm = instance.exports;
    imageData = ctx.createImageData(480, 270);

    setStatus('Starting engine...');
    wasm.game_init();
    refreshPackUi();
    ready = true;
    requestAnimationFrame(loop);
  } catch (err) {
    console.error(err);
    setStatus('Failed to load the game: ' + (err && err.message ? err.message : err));
    play.textContent = 'Load failed';
    play.disabled = true;
  }
}

boot();
