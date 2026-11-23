"""iPhone Safari에서 열어 터치 좌표를 기록하는 테스트 페이지 서버.

Mac에서 실행:  python3 touchlog_server.py
iPhone에서:    http://<Mac 주소>:8765
기록 파일:     /tmp/fga-touch.log  (한 줄에 이벤트 하나, JSON)
"""
import json
import time
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer

LOG = "/tmp/fga-touch.log"

PAGE = """<!doctype html>
<html><head><meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1, maximum-scale=1, user-scalable=no">
<title>touch log</title>
<style>
  html, body { margin: 0; height: 100%; background: #111; color: #9f9; font: 14px monospace;
               overflow: hidden; touch-action: none; -webkit-user-select: none; user-select: none; }
  #info { position: fixed; left: 8px; top: 8px; white-space: pre; pointer-events: none; }
  .dot { position: fixed; width: 10px; height: 10px; margin: -5px 0 0 -5px; border-radius: 5px;
         background: #f33; pointer-events: none; }
</style></head><body>
<div id="info">touch log ready</div>
<script>
const info = document.getElementById('info');
let seq = 0;
function send(type, e) {
  const rec = {
    seq: ++seq, type: type, t: Math.round(performance.now()),
    screenX: e.screenX, screenY: e.screenY, clientX: e.clientX, clientY: e.clientY,
    pointerType: e.pointerType, buttons: e.buttons,
    sw: screen.width, sh: screen.height, iw: innerWidth, ih: innerHeight,
    dpr: devicePixelRatio, orient: (screen.orientation || {}).angle
  };
  if (type !== 'move') {
    info.textContent = type + ' screen=' + e.screenX.toFixed(1) + ',' + e.screenY.toFixed(1) +
      ' client=' + e.clientX.toFixed(1) + ',' + e.clientY.toFixed(1) + ' ' + e.pointerType;
  }
  if (type === 'down') {
    const d = document.createElement('div');
    d.className = 'dot'; d.style.left = e.clientX + 'px'; d.style.top = e.clientY + 'px';
    document.body.appendChild(d);
  }
  fetch('/log', { method: 'POST', body: JSON.stringify(rec), keepalive: true });
}
addEventListener('pointerdown', e => send('down', e));
addEventListener('pointerup', e => send('up', e));
addEventListener('pointercancel', e => send('cancel', e));
addEventListener('pointermove', e => { if (e.buttons || e.pointerType === 'touch') send('move', e); });
addEventListener('touchmove', e => e.preventDefault(), { passive: false });
</script></body></html>
"""


# 스크롤 목록 페이지: 손가락이 움직인 거리와, 손을 뗀 뒤 관성으로 더 굴러간 거리를 기록한다.
SCROLL_PAGE = """<!doctype html>
<html><head><meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1, maximum-scale=1, user-scalable=no">
<title>scroll log</title>
<style>
  html, body { margin: 0; background: #111; color: #ddd; font: 16px monospace; }
  .row { height: 43px; line-height: 43px; padding-left: 16px; border-bottom: 1px solid #333; }
  #info { position: fixed; right: 8px; top: 8px; background: #000a; color: #9f9; padding: 4px 8px;
          pointer-events: none; white-space: pre; }
</style></head><body>
<div id="info">scroll log ready</div>
<script>
for (let i = 0; i < 400; i++) {
  const d = document.createElement('div'); d.className = 'row'; d.textContent = 'row ' + i;
  document.body.appendChild(d);
}
const info = document.getElementById('info');
let seq = 0, start = null, timer = null;
function post(rec) { fetch('/log', { method: 'POST', body: JSON.stringify(rec), keepalive: true }); }
addEventListener('touchstart', e => {
  const t = e.touches[0];
  start = { scrollY: scrollY, y: t.clientY, x: t.clientX, t: performance.now() };
}, { passive: true });
addEventListener('touchend', e => {
  if (!start) return;
  const t = e.changedTouches[0];
  const s = start; start = null;
  const atRelease = scrollY, releasedAt = performance.now();
  let last = scrollY, still = 0;
  clearInterval(timer);
  timer = setInterval(() => {
    if (scrollY === last) still++; else { still = 0; last = scrollY; }
    if (still >= 5) {
      clearInterval(timer);
      const rec = { seq: ++seq, type: 'scroll', startScroll: s.scrollY, releaseScroll: atRelease,
        finalScroll: scrollY, fingerDy: t.clientY - s.y, fingerDx: t.clientX - s.x,
        touchMs: Math.round(releasedAt - s.t), coastMs: Math.round(performance.now() - releasedAt - 300) };
      info.textContent = 'finger ' + rec.fingerDy.toFixed(0) + '  scrolled ' +
        (rec.releaseScroll - rec.startScroll).toFixed(0) + '  coast ' +
        (rec.finalScroll - rec.releaseScroll).toFixed(0);
      post(rec);
    }
  }, 60);
}, { passive: true });
</script></body></html>
"""


class Handler(BaseHTTPRequestHandler):
    def do_GET(self):
        body = (SCROLL_PAGE if self.path.startswith("/scroll") else PAGE).encode()
        self.send_response(200)
        self.send_header("Content-Type", "text/html; charset=utf-8")
        self.send_header("Content-Length", str(len(body)))
        self.send_header("Cache-Control", "no-store")
        self.end_headers()
        self.wfile.write(body)

    def do_POST(self):
        length = int(self.headers.get("Content-Length", "0"))
        raw = self.rfile.read(min(length, 4096))
        try:
            rec = json.loads(raw)
            rec["wall"] = round(time.time(), 3)
            with open(LOG, "a") as f:
                f.write(json.dumps(rec) + "\n")
        except ValueError:
            pass
        self.send_response(204)
        self.end_headers()

    def log_message(self, *args):
        pass


if __name__ == "__main__":
    ThreadingHTTPServer(("0.0.0.0", 8765), Handler).serve_forever()
