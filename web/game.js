// RustCraft WebAssembly client.
// Vercel only serves these static files; the game itself runs in the browser.

const canvas = document.getElementById('screen');
const start = document.getElementById('start');
const play = document.getElementById('play');
const statustext = document.getElementById('statustext');

const ctx = canvas.getContext('2d', { alpha: false });
canvas.tabIndex = 0;

let wasm = null;
let imageData = null;
let ready = false;
let playing = false;
let pointerLocked = false;
let lockPending = false;
let captureRequested = false;
let chatOpen = false;
let firstFrame = true;

const VK = {
  Backspace: 8, Tab: 9, Enter: 13,
  ShiftLeft: 16, ShiftRight: 16, ControlLeft: 17, ControlRight: 17,
  Escape: 27, Space: 32,
  ArrowLeft: 37, ArrowUp: 38, ArrowRight: 39, ArrowDown: 40,
  KeyA: 65, KeyD: 68, KeyE: 69, KeyF: 70, KeyQ: 81, KeyS: 83, KeyT: 84, KeyW: 87,
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

function setCapture(captured) {
  captureRequested = !!captured;
  if (!wasm) return;
  if (captured) {
    if (!pointerLocked && !lockPending && document.pointerLockElement !== canvas) {
      lockPending = true;
      try {
        const result = canvas.requestPointerLock();
        if (result && typeof result.catch === 'function') result.catch(() => {});
      } catch (_) {
        lockPending = false;
        // Pointer lock is optional; the game still works with drag-to-look.
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
  if (firstFrame) {
    firstFrame = false;
    setStatus('Ready - click to play');
  }
  requestAnimationFrame(loop);
}

async function begin() {
  if (!ready || !wasm) return;

  // Capture the mouse while we still have user activation from the click.
  // The Rust side asks for pointer lock too, but by then the click handler may
  // have yielded while the browser paints the loading status.
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

  // Give the browser one painted frame so the status text is visible before
  // the first world render blocks for a moment.
  await new Promise(resolve => requestAnimationFrame(() => requestAnimationFrame(resolve)));

  // Click the title screen's Singleplayer button.
  wasm.game_mouse_to(240, 143);
  wasm.game_mouse_button(0, 1);
  wasm.game_mouse_button(0, 0);
  wasm.game_frame();
  drawFrame();

  setStatus(pointerLocked ? 'Mouse captured - Esc to pause' : 'Click the world to capture the mouse');
}

play.addEventListener('click', begin);

canvas.addEventListener('contextmenu', e => e.preventDefault());

canvas.addEventListener('mousedown', e => {
  if (!playing || !wasm) return;
  e.preventDefault();
  if (!pointerLocked && !captureRequested) {
    const p = canvasPoint(e);
    wasm.game_mouse_to(p.x, p.y);
  }
  if (e.button === 0) wasm.game_mouse_button(0, 1);
  else if (e.button === 2) wasm.game_mouse_button(1, 1);
});

canvas.addEventListener('mouseup', e => {
  if (!wasm) return;
  if (e.button === 0) wasm.game_mouse_button(0, 0);
  else if (e.button === 2) wasm.game_mouse_button(1, 0);
});

canvas.addEventListener('mousemove', e => {
  if (!playing || !wasm) return;
  if (pointerLocked || captureRequested) {
    // `movementX/Y` also works without pointer lock, which keeps the game
    // playable in embedded browsers that deny the pointer-lock permission.
    wasm.game_mouse_move(e.movementX || 0, e.movementY || 0);
  } else {
    const p = canvasPoint(e);
    wasm.game_mouse_to(p.x, p.y);
  }
});

canvas.addEventListener('wheel', e => {
  if (!playing || !wasm) return;
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
  if (!playing || !wasm) return;

  if (preventDefault.has(e.code)) e.preventDefault();
  if (e.ctrlKey || e.metaKey || e.altKey) return;

  const code = VK[e.code];
  if (code === undefined) {
    if (!e.repeat) sendChar(e.key);
    return;
  }

  if (e.repeat) {
    if (e.code === 'Backspace') sendKey(code, true);
    else if (chatOpen) sendChar(e.key);
    return;
  }

  // The first T opens chat; don't insert a stray "t" into the input.
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
  if (!playing || !wasm) return;
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
    // The game released the mouse (inventory, pause, focus loss).
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
