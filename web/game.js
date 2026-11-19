const img = document.getElementById('screen');
const start = document.getElementById('start');
const play = document.getElementById('play');
const statustext = document.getElementById('statustext');
let seq = 1;
let inventoryOpen = false;
let playing = false;
let currentBlob = null;

function send(obj) {
  fetch('/event', { method: 'POST', headers: { 'content-type': 'application/json' }, body: JSON.stringify(obj) }).catch(() => {});
}
function eventKey(code, down) { send({ t: 'key', code, down }); }
function internalXY(e) {
  const r = img.getBoundingClientRect();
  return {
    x: Math.max(0, Math.min(479, Math.floor((e.clientX - r.left) / r.width * 480))),
    y: Math.max(0, Math.min(269, Math.floor((e.clientY - r.top) / r.height * 270)))
  };
}
function click(button, e) {
  const p = e ? internalXY(e) : { x: 240, y: 135 };
  send({ t: 'click', button, x: p.x, y: p.y, seq: seq++ });
}
function begin() {
  playing = true;
  start.classList.add('hidden');
  img.requestPointerLock?.();
}
play.addEventListener('click', begin);
img.addEventListener('click', e => {
  if (!playing) { begin(); return; }
  if (inventoryOpen) click(0, e);
});
document.addEventListener('pointerlockchange', () => {});
img.addEventListener('contextmenu', e => e.preventDefault());
img.addEventListener('mousedown', e => {
  if (!playing || inventoryOpen) return;
  e.preventDefault();
  click(e.button === 2 ? 1 : 0, null);
});
img.addEventListener('wheel', e => {
  e.preventDefault();
  send({ t: 'wheel', delta: e.deltaY > 0 ? 1 : -1 });
}, { passive: false });
document.addEventListener('mousemove', e => {
  if (!playing || inventoryOpen || document.pointerLockElement !== img) return;
  send({ t: 'mouse', dx: e.movementX || 0, dy: e.movementY || 0 });
});
document.addEventListener('keydown', e => {
  if (!playing) { if (e.code === 'Enter' || e.code === 'Space') begin(); return; }
  if (['F5', 'Space', 'Tab', 'ArrowUp', 'ArrowDown', 'ArrowLeft', 'ArrowRight'].includes(e.code)) e.preventDefault();
  if (e.repeat) return;
  if (e.code === 'KeyE') {
    inventoryOpen = !inventoryOpen;
    if (inventoryOpen) document.exitPointerLock?.();
    else img.requestPointerLock?.();
  }
  eventKey(e.code, true);
});
document.addEventListener('keyup', e => eventKey(e.code, false));
window.addEventListener('blur', () => { send({ t: 'blur' }); });
window.addEventListener('mouseout', e => { if (!e.relatedTarget) send({ t: 'blur' }); });

// Stream frames written by the Rust engine.
let loading = true;
async function poll() {
  try {
    const res = await fetch('/frame.png?t=' + Date.now(), { cache: 'no-store' });
    if (res.ok) {
      const blob = await res.blob();
      if (currentBlob) URL.revokeObjectURL(currentBlob);
      currentBlob = URL.createObjectURL(blob);
      img.src = currentBlob;
      if (loading) { loading = false; statustext.textContent = 'World running — click to play'; }
    }
  } catch (_) { statustext.textContent = 'Waiting for the Rust engine…'; }
  setTimeout(poll, 80);
}
poll();
