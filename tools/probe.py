import argparse
import io
import os
import subprocess
import sys
import time

import numpy as np
import serial
from PIL import Image

MAC = os.environ.get("FGA_MAC", "")
FRAME = "/tmp/fga-frame.jpg"
COORD_MAX = 32767


def grab() -> np.ndarray:
    data = subprocess.run(
        ["ssh", "-o", "BatchMode=yes", MAC, f"cat {FRAME}"],
        capture_output=True).stdout
    # Windows OpenSSH can exit non-zero after a complete transfer, so judge by the data.
    if len(data) < 1000:
        raise RuntimeError("frame fetch failed")
    return np.asarray(Image.open(io.BytesIO(data)).convert("RGB")).astype(np.int16)


def yellow_mask(img: np.ndarray) -> np.ndarray:
    r, g, b = img[..., 0], img[..., 1], img[..., 2]
    return (r > 200) & (g > 150) & (g < 235) & (b < 90)


def locate(img: np.ndarray, baseline: np.ndarray):
    mask = yellow_mask(img) & ~baseline
    ys, xs = np.nonzero(mask)
    if len(xs) < 80:
        return None
    cx, cy = np.median(xs), np.median(ys)
    for _ in range(3):
        near = (np.abs(xs - cx) < 70) & (np.abs(ys - cy) < 70)
        if near.sum() < 80:
            return None
        cx, cy = xs[near].mean(), ys[near].mean()
    return cx, cy, int(near.sum())


class Board:
    def __init__(self, port: str):
        # The ESP32-C3 USB port treats RTS as reset, so RTS must be off before the port opens.
        self.sp = serial.Serial()
        self.sp.port = port
        self.sp.baudrate = 115200
        self.sp.timeout = 1.5
        self.sp.rts = False
        self.sp.dtr = True
        self.sp.open()
        time.sleep(0.3)
        self.sp.reset_input_buffer()

    def cmd(self, text: str) -> str:
        self.sp.write((text + "\n").encode())
        deadline = time.time() + 3
        while time.time() < deadline:
            line = self.sp.readline().decode(errors="replace").strip()
            if line.startswith(("OK", "ERR")):
                return line
        return "ERR timeout"


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("points", nargs="*", help="x,y (0..32767)")
    ap.add_argument("--scan-x", nargs=3, type=int, metavar=("FROM", "TO", "STEP"))
    ap.add_argument("--scan-y", nargs=3, type=int, metavar=("FROM", "TO", "STEP"))
    ap.add_argument("--x", type=int, default=16383)
    ap.add_argument("--y", type=int, default=16383)
    ap.add_argument("--port", default="COM3")
    ap.add_argument("--settle", type=float, default=0.9)
    ap.add_argument("--start", default="300 32000", help="각 측정 전 출발 위치 'x y'")
    ap.add_argument("--via", action="append", default=[],
                    help="목표 직전에 거칠 지점, 목표 기준 상대값 'dx,dy' (여러 번 지정 가능)")
    args = ap.parse_args()

    pts = [tuple(int(v) for v in p.split(",")) for p in args.points]
    if args.scan_x:
        a, b, s = args.scan_x
        pts += [(x, args.y) for x in range(a, b + 1, s)]
    if args.scan_y:
        a, b, s = args.scan_y
        pts += [(args.x, y) for y in range(a, b + 1, s)]
    if not pts:
        ap.error("no points")

    board = Board(args.port)
    board.cmd("MOVE 300 32000")
    time.sleep(args.settle)
    base_img = grab()
    h, w = base_img.shape[:2]
    baseline = yellow_mask(base_img)
    grown = baseline.copy()
    for dy in (-3, 0, 3):
        for dx in (-3, 0, 3):
            grown |= np.roll(np.roll(baseline, dy, 0), dx, 1)
    print(f"frame {w}x{h}")
    print(f"{'sent':>13} {'expect px':>12} {'actual px':>12} {'err px':>10}  n")

    for x, y in pts:
        board.cmd(f"MOVE {args.start}")
        time.sleep(0.25)
        for via in args.via:
            vx, vy = (int(v) for v in via.split(","))
            board.cmd(f"MOVE {x + vx} {y + vy}")
            time.sleep(0.08)
        board.cmd(f"MOVE {x} {y}")
        time.sleep(args.settle)
        found = locate(grab(), grown)
        ex, ey = x / COORD_MAX * w, y / COORD_MAX * h
        if found is None:
            print(f"{x:>6},{y:<6} {ex:>6.0f},{ey:<5.0f} {'(not found)':>12}")
            continue
        cx, cy, n = found
        print(f"{x:>6},{y:<6} {ex:>6.0f},{ey:<5.0f} {cx:>6.0f},{cy:<5.0f} "
              f"{cx - ex:>+5.0f},{cy - ey:<+4.0f} {n}")
    sys.stdout.flush()


if __name__ == "__main__":
    main()
