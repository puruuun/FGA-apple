"""FGACapture.app이 계속 덮어쓰는 최신 iPhone 화면 프레임을 HTTP로 내준다.

Mac에서 실행:  python3 frame_server.py
    GET /frame   최신 프레임(JPEG). 헤더 X-Frame-Age-Ms에 프레임이 만들어진 뒤 지난 시간.
    GET /status  프레임 파일 상태(JSON)
"""
import json
import os
import time
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer

FRAME = "/tmp/fga-frame.jpg"
PORT = 8766


class Handler(BaseHTTPRequestHandler):
    def do_GET(self):
        try:
            st = os.stat(FRAME)
            age_ms = int((time.time() - st.st_mtime) * 1000)
        except OSError:
            self.send_error(503, "no frame yet")
            return

        if self.path.startswith("/status"):
            body = json.dumps({"ageMs": age_ms, "bytes": st.st_size}).encode()
            ctype = "application/json"
        else:
            # 캡처 도구가 임시 파일에 쓴 뒤 이름을 바꿔 넣으므로, 읽는 도중 반쯤 쓰인 파일을 볼 일은 없다.
            with open(FRAME, "rb") as f:
                body = f.read()
            ctype = "image/jpeg"

        self.send_response(200)
        self.send_header("Content-Type", ctype)
        self.send_header("Content-Length", str(len(body)))
        self.send_header("X-Frame-Age-Ms", str(age_ms))
        self.send_header("Cache-Control", "no-store")
        self.end_headers()
        self.wfile.write(body)

    def log_message(self, *args):
        pass


if __name__ == "__main__":
    ThreadingHTTPServer(("0.0.0.0", PORT), Handler).serve_forever()
