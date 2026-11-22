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

// WebAudio Synthesizer + Real Resource-Pack (.ogg/.wav) Sound Loader
let audioCtx = null;
let musicReverb = null;
let musicBus = null;
let sfxBus = null;
let musicNextAt = 0;
let musicStep = 0;
let customSoundVersion = -1;
const customAudioBuffers = new Map();

// 100% Original Ambient Music Suites per Dimension: [notes_hz[], hold_sec, pause_sec, vel]
// Overworld ("Sunlight & Moss"): warm Dmaj7 / Bm9 / Gmaj7 / A6 felt-piano progression
const OVERWORLD_PHRASES = [
  [[146.83, 293.66, 369.99], 3.6, 1.40, 0.055],
  [[440.00], 2.8, 1.15, 0.048],
  [[587.33], 3.2, 1.65, 0.052],
  [[493.88], 3.8, 2.20, 0.045],
  [[123.47, 246.94, 369.99], 4.0, 1.55, 0.052],
  [[554.37], 2.9, 1.15, 0.045],
  [[493.88], 3.5, 2.10, 0.046],
  [[98.00, 196.00, 293.66], 4.4, 1.70, 0.052],
  [[369.99], 2.6, 1.10, 0.046],
  [[329.63], 3.4, 1.45, 0.044],
  [[293.66], 4.2, 2.75, 0.042],
  [[110.00, 220.00, 329.63], 4.2, 1.60, 0.050],
  [[493.88], 2.8, 1.25, 0.044],
  [[440.00], 3.6, 1.80, 0.042],
  [[369.99], 4.6, 3.20, 0.040],
];

// Nether ("Ember Choir"): deep Phrygian cello/brass drones & distant bells
const NETHER_PHRASES = [
  [[73.42, 110.00], 5.6, 2.40, 0.065],
  [[293.66], 3.4, 1.90, 0.036],
  [[77.78, 116.54], 5.8, 2.50, 0.062],
  [[311.13], 3.6, 2.10, 0.034],
  [[65.41, 98.00], 5.4, 2.30, 0.064],
  [[246.94], 3.8, 2.40, 0.035],
];

// The End ("Astral Drift"): cosmic Lydian floating pads & crystal chimes
const END_PHRASES = [
  [[130.81, 196.00], 5.0, 1.80, 0.052],
  [[523.25, 739.99], 3.2, 1.60, 0.040],
  [[185.00, 261.63], 4.8, 2.10, 0.048],
  [[659.25], 3.6, 1.70, 0.038],
  [[146.83, 220.00], 5.2, 2.30, 0.050],
  [[587.33, 739.99], 4.0, 2.50, 0.038],
];
let lastMusicDim = -1;
const realSfxBuffers = new Map();
const realSfxPending = new Set();
const realMusicBuffers = new Map();
const realMusicPending = new Set();
let activeMusicSource = null;
let activeMusicDim = -1;

function loadRealSfx(id) {
  if (!audioCtx || realSfxBuffers.has(id) || realSfxPending.has(id)) return;
  realSfxPending.add(id);
  fetch(`sounds/${id}.ogg`)
    .then(r => (r.ok ? r.arrayBuffer() : null))
    .then(buf => (buf ? audioCtx.decodeAudioData(buf) : null))
    .then(decoded => { if (decoded) realSfxBuffers.set(id, decoded); })
    .catch(() => {});
}

function loadRealMusic(dim) {
  if (!audioCtx || realMusicBuffers.has(dim) || realMusicPending.has(dim)) return;
  realMusicPending.add(dim);
  fetch(`sounds/music_${dim}.ogg`)
    .then(r => (r.ok ? r.arrayBuffer() : null))
    .then(buf => (buf ? audioCtx.decodeAudioData(buf) : null))
    .then(decoded => { if (decoded) realMusicBuffers.set(dim, decoded); })
    .catch(() => {});
}

function createRoomImpulse(ctx, dur = 3.0, decay = 2.6) {
  const len = Math.floor(ctx.sampleRate * dur);
  const ir = ctx.createBuffer(2, len, ctx.sampleRate);
  for (let ch = 0; ch < 2; ch++) {
    const data = ir.getChannelData(ch);
    let lp = 0;
    for (let i = 0; i < len; i++) {
      const env = Math.pow(1 - i / len, decay);
      const white = Math.random() * 2 - 1;
      lp = lp * 0.72 + white * 0.28;
      data[i] = lp * env;
    }
  }
  return ir;
}

function ensureAudio() {
  if (!audioCtx) {
    const AC = window.AudioContext || window.webkitAudioContext;
    if (!AC) return null;
    audioCtx = new AC();

    // Warm stereo hall reverb bus for ambient music + subtle room depth for SFX
    musicReverb = audioCtx.createConvolver();
    musicReverb.buffer = createRoomImpulse(audioCtx, 3.1, 2.7);
    const wetGain = audioCtx.createGain();
    wetGain.gain.value = 0.46;
    const dryGain = audioCtx.createGain();
    dryGain.gain.value = 0.68;
    musicBus = audioCtx.createGain();
    musicBus.gain.value = 0.90;
    musicBus.connect(dryGain);
    musicBus.connect(musicReverb);
    musicReverb.connect(wetGain);
    dryGain.connect(audioCtx.destination);
    wetGain.connect(audioCtx.destination);

    sfxBus = audioCtx.createDynamicsCompressor();
    sfxBus.threshold.value = -14;
    sfxBus.ratio.value = 4;
    sfxBus.connect(audioCtx.destination);
    // Warm up common real recorded SFX + Overworld music immediately
    [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 14, 15, 28, 29].forEach(loadRealSfx);
    loadRealMusic(0);
  }
  if (audioCtx.state === 'suspended') {
    audioCtx.resume().catch(() => {});
  }
  return audioCtx;
}

function syncCustomSounds() {
  if (!wasm || !wasm.game_sound_version) return;
  const ctx = ensureAudio();
  if (!ctx) return;
  const ver = wasm.game_sound_version();
  if (ver === customSoundVersion) return;
  customSoundVersion = ver;
  customAudioBuffers.clear();
  const n = wasm.game_sound_count();
  for (let i = 0; i < n; i++) {
    const sfx = wasm.game_sound_sfx(i);
    const ptr = wasm.game_sound_ptr(i);
    const len = wasm.game_sound_len(i);
    if (sfx >= 0 && ptr && len > 0) {
      const raw = new Uint8Array(wasm.memory.buffer, ptr, len).slice().buffer;
      ctx.decodeAudioData(raw).then(buf => {
        customAudioBuffers.set(sfx, buf);
      }).catch(() => {});
    }
  }
}

function playFeltPianoNote(freq, dur, vol, delay = 0) {
  const ctx = ensureAudio();
  if (!ctx || ctx.state !== 'running' || !musicBus) return;
  const t = ctx.currentTime + delay;
  const flt = ctx.createBiquadFilter();
  flt.type = 'lowpass';
  flt.frequency.setValueAtTime(Math.min(1550, freq * 3.6), t);
  flt.frequency.exponentialRampToValueAtTime(Math.max(240, freq * 1.35), t + dur * 0.75);

  const env = ctx.createGain();
  env.gain.setValueAtTime(0.0001, t);
  env.gain.linearRampToValueAtTime(vol, t + 0.032);
  env.gain.exponentialRampToValueAtTime(vol * 0.35, t + dur * 0.35);
  env.gain.exponentialRampToValueAtTime(0.0001, t + dur);

  // Soft fundamental + warm 2nd harmonic + gentle felt transient
  const o1 = ctx.createOscillator();
  o1.type = 'triangle';
  o1.frequency.setValueAtTime(freq, t);
  const g1 = audioCtx.createGain();
  g1.gain.value = 0.76;

  const o2 = ctx.createOscillator();
  o2.type = 'sine';
  o2.frequency.setValueAtTime(freq * 2.001, t);
  const g2 = audioCtx.createGain();
  g2.gain.value = 0.16;

  const oSub = ctx.createOscillator();
  oSub.type = 'sine';
  oSub.frequency.setValueAtTime(freq * 0.5, t);
  const gSub = audioCtx.createGain();
  gSub.gain.value = 0.14;

  o1.connect(g1); g1.connect(flt);
  o2.connect(g2); g2.connect(flt);
  oSub.connect(gSub); gSub.connect(flt);
  flt.connect(env);
  env.connect(musicBus);

  o1.start(t); o2.start(t); oSub.start(t);
  o1.stop(t + dur + 0.05); o2.stop(t + dur + 0.05); oSub.stop(t + dur + 0.05);
}

function playTone(freq, dur, vol, type = 'sine', slideTo = null, delay = 0, lpHz = 2800) {
  const ctx = ensureAudio();
  if (!ctx || ctx.state !== 'running') return;
  const t = ctx.currentTime + delay;
  const osc = ctx.createOscillator();
  const flt = ctx.createBiquadFilter();
  flt.type = 'lowpass';
  flt.frequency.value = lpHz;
  const gain = ctx.createGain();
  osc.type = type;
  osc.frequency.setValueAtTime(freq, t);
  if (slideTo) osc.frequency.exponentialRampToValueAtTime(Math.max(30, slideTo), t + dur);
  gain.gain.setValueAtTime(0.0001, t);
  gain.gain.linearRampToValueAtTime(vol, t + 0.010);
  gain.gain.exponentialRampToValueAtTime(0.0001, t + dur);
  osc.connect(flt);
  flt.connect(gain);
  gain.connect(sfxBus || ctx.destination);
  osc.start(t);
  osc.stop(t + dur + 0.02);
}

function playNoise(dur, vol, filterFreq = 900, filterType = 'lowpass', delay = 0, q = 0.9) {
  const ctx = ensureAudio();
  if (!ctx || ctx.state !== 'running') return;
  const len = Math.max(1, Math.floor(ctx.sampleRate * dur));
  const buf = ctx.createBuffer(1, len, ctx.sampleRate);
  const data = buf.getChannelData(0);
  // Pinkish smoothed noise sounds far more organic than harsh white noise
  let b0 = 0, b1 = 0;
  for (let i = 0; i < len; i++) {
    const w = Math.random() * 2 - 1;
    b0 = 0.88 * b0 + w * 0.12;
    b1 = 0.60 * b1 + w * 0.40;
    data[i] = (b0 + b1) * 0.75;
  }
  const src = ctx.createBufferSource();
  src.buffer = buf;
  const flt = ctx.createBiquadFilter();
  flt.type = filterType;
  flt.frequency.value = filterFreq;
  flt.Q.value = q;
  const gain = ctx.createGain();
  const t = ctx.currentTime + delay;
  gain.gain.setValueAtTime(0.0001, t);
  gain.gain.linearRampToValueAtTime(vol, t + Math.min(0.012, dur * 0.25));
  gain.gain.exponentialRampToValueAtTime(0.0001, t + dur);
  src.connect(flt);
  flt.connect(gain);
  gain.connect(sfxBus || ctx.destination);
  src.start(t);
}

function playWebSfx(id) {
  const ctx = ensureAudio();
  if (ctx && ctx.state === 'running' && customAudioBuffers.has(id)) {
    const src = ctx.createBufferSource();
    src.buffer = customAudioBuffers.get(id);
    const gain = ctx.createGain();
    gain.gain.value = 0.72;
    src.connect(gain);
    gain.connect(ctx.destination);
    src.start();
    return;
  }
  if (ctx && ctx.state === 'running') {
    if (realSfxBuffers.has(id)) {
      const src = ctx.createBufferSource();
      src.buffer = realSfxBuffers.get(id);
      const pitch = (id === 0 || id === 29 || id === 31 || id === 32 || id === 34 || id === 35)
        ? 1.0 : (0.92 + Math.random() * 0.16);
      src.playbackRate.value = pitch;
      const gain = ctx.createGain();
      gain.gain.value = (id >= 3 && id <= 8) ? 0.36 : id === 0 ? 0.45 : 0.65;
      src.connect(gain);
      gain.connect(sfxBus || ctx.destination);
      src.start();
      return;
    }
    loadRealSfx(id);
  }
  const p = 0.92 + Math.random() * 0.16; // natural human/Foley pitch variation
  switch (id) {
    case 0: playTone(720 * p, 0.025, 0.045, 'triangle', 540 * p, 0, 1800); break; // UI Click
    case 1: playNoise(0.07, 0.11, 520 * p, 'lowpass'); playTone(145 * p, 0.055, 0.09, 'sine', 78 * p, 0, 600); break; // Place block thud
    case 2: playNoise(0.14, 0.17, 920 * p, 'bandpass', 0, 1.1); playTone(118 * p, 0.075, 0.10, 'triangle', 56 * p, 0, 700); break; // Break block
    case 3:
    case 4: playNoise(0.075, 0.09, 640 * p, 'lowpass', 0, 0.7); break; // Grass rustle step
    case 5: playNoise(0.045, 0.08, 1350 * p, 'bandpass', 0, 1.4); playTone(260 * p, 0.035, 0.05, 'triangle', 140 * p, 0, 1000); break; // Stone tap step
    case 6: playTone(185 * p, 0.055, 0.08, 'triangle', 120 * p, 0, 900); playNoise(0.04, 0.06, 800 * p, 'bandpass'); break; // Wood creak step
    case 7: playNoise(0.085, 0.09, 1750 * p, 'bandpass', 0, 0.8); break; // Sand crunch step
    case 8: playNoise(0.16, 0.11, 820 * p, 'bandpass', 0, 1.6); playTone(260 * p, 0.08, 0.06, 'sine', 520 * p, 0.01, 1400); break; // Water bubble step
    case 9: playNoise(0.06, 0.05, 520, 'lowpass'); playTone(145 * p, 0.07, 0.04, 'sine', 225 * p, 0, 800); break; // Jump
    case 10: playNoise(0.07, 0.10, 420, 'lowpass'); playTone(88 * p, 0.08, 0.08, 'sine', 48 * p, 0, 400); break; // Land
    case 11: playNoise(0.09, 0.12, 760, 'lowpass'); playTone(165 * p, 0.10, 0.10, 'triangle', 76 * p, 0, 800); break; // Hurt
    case 12: playNoise(0.07, 0.10, 880, 'bandpass'); playTone(190 * p, 0.09, 0.10, 'triangle', 86 * p, 0, 900); break; // MobHurt
    case 13: playNoise(0.11, 0.09, 680, 'lowpass'); playTone(220 * p, 0.16, 0.07, 'sine', 70 * p, 0, 600); break; // MobDie
    case 14: playTone(760 * p, 0.055, 0.055, 'sine', 1280 * p, 0, 2400); break; // Pickup pop
    case 15: playTone(420, 0.04, 0.05, 'triangle', 320, 0, 1200); playTone(680, 0.06, 0.05, 'triangle', 860, 0.045, 1800); break; // Craft
    case 16: playNoise(0.26, 0.10, 460, 'lowpass'); playNoise(0.04, 0.06, 2200, 'highpass', 0.09); break; // Furnace crackle
    case 17: playNoise(0.68, 0.28, 340, 'lowpass'); playTone(56, 0.45, 0.20, 'sine', 28, 0, 260); break; // Explode boom
    case 18: playNoise(0.28, 0.13, 980 * p, 'bandpass', 0, 1.3); playTone(310 * p, 0.08, 0.07, 'sine', 640 * p, 0.03, 1600); break; // Splash
    case 19: playTone(98 * p, 0.36, 0.09, 'sawtooth', 76 * p, 0, 460); playTone(146 * p, 0.32, 0.05, 'triangle', 108 * p, 0, 520); break; // Zombie groan
    case 20: playNoise(0.035, 0.08, 1850 * p, 'bandpass', 0, 2.2); playNoise(0.035, 0.07, 2200 * p, 'bandpass', 0.06, 2.2); break; // Skeleton bone clack
    case 21: playTone(132 * p, 0.48, 0.09, 'sawtooth', 106 * p, 0, 480); playTone(264 * p, 0.42, 0.04, 'sine', 212 * p, 0, 700); break; // Cow moo
    case 22: playTone(255 * p, 0.30, 0.08, 'triangle', 228 * p, 0, 900); playTone(510 * p, 0.26, 0.03, 'sine', 456 * p, 0, 1200); break; // Sheep baa
    case 23: playTone(165 * p, 0.15, 0.08, 'triangle', 215 * p, 0, 820); break; // Pig oink
    case 24: playTone(610 * p, 0.055, 0.06, 'triangle', 740 * p, 0, 1600); playTone(520 * p, 0.055, 0.05, 'triangle', 660 * p, 0.07, 1500); break; // Chicken cluck
    case 25: playNoise(0.14, 0.08, 1800 * p, 'bandpass', 0, 2.0); break; // Spider hiss
    case 26: playNoise(0.55, 0.14, 2400, 'highpass', 0, 1.4); playTone(1650, 0.48, 0.03, 'sine', 2900, 0, 3400); break; // Creeper fuse hiss
    case 27: playTone(195 * p, 0.28, 0.07, 'sawtooth', 128 * p, 0, 620); playTone(390 * p, 0.24, 0.04, 'sine', 190 * p, 0, 900); break; // Enderman
    case 28: playTone(135 * p, 0.11, 0.08, 'triangle', 180 * p, 0, 650); playTone(520, 0.04, 0.05, 'sine', 380, 0.09, 1200); break; // Door
    case 29: [659.25, 830.61, 987.77, 1244.51, 1479.98].forEach((f, idx) => playFeltPianoNote(f, 0.55 + idx * 0.1, 0.06, idx * 0.07)); break; // LevelUp
    case 30: playTone(320 * p, 0.12, 0.09, 'triangle', 220 * p, 0, 1200); playTone(515 * p, 0.10, 0.07, 'sine', 480 * p, 0, 1600); break; // Shield block
    case 31: [523.25, 659.25, 783.99, 1046.5, 1318.5].forEach((f, idx) => playFeltPianoNote(f, 0.65, 0.075, idx * 0.04)); break; // Totem
    case 32: [261.63, 523.25, 783.99, 1046.5, 1567.98].forEach((f, idx) => playFeltPianoNote(f, 0.90, 0.065, idx * 0.05)); break; // Beacon
    case 33: playTone(68 * p, 0.65, 0.12, 'sawtooth', 46 * p, 0, 380); playNoise(0.55, 0.12, 420, 'lowpass'); break; // Wither roar
    case 34: [554.37, 659.25, 830.61, 1108.73, 987.77, 830.61, 659.25, 554.37].forEach((f, idx) => playTone(f, 0.24, 0.055, 'triangle', null, idx * 0.22, 2200)); break; // Meadow Promenade
    case 35: [146.83, 146.83, 220.0, 146.83, 174.61, 174.61, 261.63, 196.0].forEach((f, idx) => playTone(f, 0.19, 0.075, 'sawtooth', null, idx * 0.20, 1100)); break; // Basalt Bounce
    case 36: playTone(152 * p, 0.34, 0.08, 'sawtooth', 132 * p, 0, 620); break; // Villager hmm
    case 37: playNoise(0.06, 0.09, 900 * p, 'lowpass'); playTone(280 * p, 0.14, 0.10, 'sawtooth', 140 * p, 0, 880); break; // Wolf bark
    case 38: playTone(195 * p, 0.28, 0.05, 'sawtooth', 210 * p, 0, 740); break; // Bee buzz
    case 39: playTone(54 * p, 0.65, 0.14, 'sawtooth', 42 * p, 0, 280); playTone(235 * p, 0.22, 0.05, 'sine', 120 * p, 0.05, 500); break; // Warden sonic pulse
    default: playNoise(0.06, 0.06, 750); break;
  }
}

function pumpAudio() {
  if (!wasm) return;
  syncCustomSounds();
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
    const dim = (flags >> 2) & 3;
    if (dim !== lastMusicDim) {
      lastMusicDim = dim;
      musicStep = 0;
      musicNextAt = audioCtx.currentTime + 0.35;
      loadRealMusic(dim);
      if (activeMusicSource && activeMusicDim !== dim) {
        try { activeMusicSource.stop(); } catch (_) {}
        activeMusicSource = null;
        activeMusicDim = -1;
      }
    }
    if (realMusicBuffers.has(dim)) {
      if (!activeMusicSource) {
        const src = audioCtx.createBufferSource();
        src.buffer = realMusicBuffers.get(dim);
        src.loop = true;
        const g = audioCtx.createGain();
        g.gain.value = 0.34;
        src.connect(g);
        g.connect(musicBus || audioCtx.destination);
        src.start();
        activeMusicSource = src;
        activeMusicDim = dim;
      }
    } else {
      loadRealMusic(dim);
    }
  } else if (!musicOn && activeMusicSource) {
    try { activeMusicSource.stop(); } catch (_) {}
    activeMusicSource = null;
    activeMusicDim = -1;
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
  KeyA: 65, KeyB: 66, KeyC: 67, KeyD: 68, KeyE: 69, KeyF: 70, KeyG: 71, KeyQ: 81, KeyS: 83, KeyT: 84, KeyW: 87, KeyZ: 90,
  F1: 112, F2: 113, F3: 114, F4: 115, F5: 116, F11: 122,
  Slash: 191,
  Digit1: 49, Digit2: 50, Digit3: 51, Digit4: 52, Digit5: 53,
  Digit6: 54, Digit7: 55, Digit8: 56, Digit9: 57,
  Numpad1: 97, Numpad2: 98, Numpad3: 99, Numpad4: 100, Numpad5: 101,
  Numpad6: 102, Numpad7: 103, Numpad8: 104, Numpad9: 105,
};

const preventDefault = new Set([
  'Space', 'Tab', 'F1', 'F2', 'F3', 'F4', 'F5', 'F11',
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
  if (len !== 960 * 540 * 4) return;
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
  else if (e.button === 1) wasm.game_mouse_button(2, 1);
  else if (e.button === 2) wasm.game_mouse_button(1, 1);
});

canvas.addEventListener('mouseup', e => {
  if (!wasm || packsOpen) return;
  if (e.button === 0) wasm.game_mouse_button(0, 0);
  else if (e.button === 1) wasm.game_mouse_button(2, 0);
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
  const isGameCtrlKey =
    e.code === 'ControlLeft' || e.code === 'ControlRight' ||
    (e.ctrlKey && (
      e.code === 'KeyW' || e.code === 'KeyA' || e.code === 'KeyS' || e.code === 'KeyD' ||
      e.code === 'Space' || e.code === 'ShiftLeft' || e.code === 'ShiftRight'
    ));
  if (isF5 || preventDefault.has(e.code) || isGameCtrlKey) {
    e.preventDefault();
    e.stopPropagation();
  }
  if (e.metaKey || e.altKey) return;
  if (e.ctrlKey && !isGameCtrlKey && VK[e.code] === undefined) return;

  const code = isF5 ? 116 : VK[e.code];
  if (code === undefined) {
    if (!e.repeat) sendChar(e.key);
    return;
  }

  const inChat = wasm.game_chat_open ? (wasm.game_chat_open() !== 0) : chatOpen;

  if (e.repeat) {
    if (e.code === 'Backspace') sendKey(code, true);
    else if (inChat) sendChar(e.key);
    return;
  }

  sendKey(code, true);
  if (e.key.length === 1) sendChar(e.key);
  if (wasm.game_chat_open) chatOpen = wasm.game_chat_open() !== 0;
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
    imageData = ctx.createImageData(960, 540);

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
