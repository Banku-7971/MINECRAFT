// Tiny dependency-free server: serves the viewer and streams frames rendered by the Rust engine.
const http = require('http');
const fs = require('fs');
const path = require('path');

const ROOT = path.resolve(__dirname, '..');
const WEB = __dirname;
const PORT = Number(process.env.PORT || 3000);
const EVENTS = path.join(ROOT, 'runtime', 'events.log');

fs.mkdirSync(path.dirname(EVENTS), { recursive: true });

const types = { '.html': 'text/html; charset=utf-8', '.js': 'text/javascript; charset=utf-8', '.css': 'text/css; charset=utf-8', '.png': 'image/png' };

const server = http.createServer((req, res) => {
  const url = new URL(req.url, 'http://localhost');
  if (req.method === 'POST' && url.pathname === '/event') {
    let body = '';
    req.on('data', chunk => { body += chunk; if (body.length > 4096) req.destroy(); });
    req.on('end', () => {
      const line = body.trim();
      if (line.startsWith('{')) fs.appendFileSync(EVENTS, line + '\n');
      res.writeHead(200, { 'content-type': 'application/json', 'access-control-allow-origin': '*' });
      res.end('{"ok":true}');
    });
    return;
  }
  if (url.pathname === '/frame.png') {
    const p = path.join(ROOT, 'runtime', 'frame.png');
    fs.readFile(p, (err, data) => {
      if (err) { res.writeHead(404); res.end('no frame yet'); return; }
      res.writeHead(200, { 'content-type': 'image/png', 'cache-control': 'no-store, no-cache, must-revalidate', 'access-control-allow-origin': '*' });
      res.end(data);
    });
    return;
  }
  if (url.pathname === '/status') {
    res.writeHead(200, { 'content-type': 'application/json' });
    res.end(JSON.stringify({ ok: true, events: fs.existsSync(EVENTS) ? fs.statSync(EVENTS).size : 0 }));
    return;
  }
  let name = url.pathname === '/' ? 'index.html' : url.pathname.replace(/^\/+/, '');
  const file = path.join(WEB, path.basename(name));
  fs.readFile(file, (err, data) => {
    if (err) { res.writeHead(404); res.end('not found'); return; }
    res.writeHead(200, { 'content-type': types[path.extname(file)] || 'application/octet-stream', 'cache-control': 'no-store' });
    res.end(data);
  });
});

server.listen(PORT, '0.0.0.0', () => console.log(`Minecraft Rust viewer listening on 0.0.0.0:${PORT}`));
