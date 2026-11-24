import argparse
import math
import time

from probe import Board
from taptest import read_log

PAGE_X_OFFSET = 62.0
PAGE_Y_OFFSET = 110.0
SCREEN_W, SCREEN_H = 874.0, 402.0


def to_page(x, y):
    return x / 32767 * SCREEN_W - PAGE_X_OFFSET, y / 32767 * SCREEN_H - PAGE_Y_OFFSET


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("swipes", nargs="+", help='"x1 y1 x2 y2 duration_ms"')
    ap.add_argument("--port", default="COM3")
    ap.add_argument("--step", type=int, help="SET step 값(ms)")
    ap.add_argument("--hold", type=int, help="SET hold 값(ms)")
    args = ap.parse_args()

    for s in args.swipes:
        v = [int(t) for t in s.split()]
        if min(v[1], v[3]) < 11000:
            ap.error(f"{s}: y가 11000 미만이면 Safari 탭 막대를 건드릴 수 있어 거부합니다")

    board = Board(args.port)
    if args.step is not None:
        board.cmd(f"SET step {args.step}")
    if args.hold is not None:
        board.cmd(f"SET hold {args.hold}")
    print(board.cmd("STAT"))

    seen = len(read_log())
    for s in args.swipes:
        x1, y1, x2, y2, dur = (int(t) for t in s.split())
        t0 = time.time()
        res = board.cmd(f"SWIPE {x1} {y1} {x2} {y2} {dur}")
        sent_for = time.time() - t0
        time.sleep(1.0)
        recs = read_log()
        new, seen = recs[seen:], len(recs)
        print(f"\nSWIPE {s} -> {res} (명령 소요 {sent_for * 1000:.0f} ms)")
        if not new:
            print("ABORT: 테스트 페이지가 터치를 받지 못했습니다.")
            return
        downs = [r for r in new if r["type"] == "down"]
        ups = [r for r in new if r["type"] in ("up", "cancel")]
        moves = [r for r in new if r["type"] == "move"]
        ex1, ey1 = to_page(x1, y1)
        ex2, ey2 = to_page(x2, y2)
        print(f"  기대: 시작 {ex1:.0f},{ey1:.0f} -> 끝 {ex2:.0f},{ey2:.0f}")
        print(f"  이벤트: down={len(downs)} move={len(moves)} up/cancel={len(ups)} "
              f"({', '.join(r['type'] for r in ups)})")
        if not downs or not ups:
            continue
        d, u = downs[0], ups[-1]
        print(f"  실제: 시작 {d['clientX']:.0f},{d['clientY']:.0f} -> 끝 {u['clientX']:.0f},{u['clientY']:.0f}"
              f"  접촉 시간 {u['t'] - d['t']} ms")
        pts = [d] + moves + [u]
        lx, ly = u["clientX"] - d["clientX"], u["clientY"] - d["clientY"]
        length = math.hypot(lx, ly) or 1.0
        max_dev = max(abs((p["clientX"] - d["clientX"]) * ly - (p["clientY"] - d["clientY"]) * lx) / length
                      for p in pts)
        jumps = [math.hypot(b["clientX"] - a["clientX"], b["clientY"] - a["clientY"])
                 for a, b in zip(pts, pts[1:])]
        print(f"  직선 이탈 최대 {max_dev:.1f} pt, 이벤트 간 최대 이동 {max(jumps):.1f} pt, "
              f"평균 {sum(jumps) / len(jumps):.1f} pt")
        if len(moves) >= 4:
            a, b = moves[-4], moves[-1]
            dt = (b["t"] - a["t"]) or 1
            v = math.hypot(b["clientX"] - a["clientX"], b["clientY"] - a["clientY"]) / dt * 1000
            print(f"  떼기 직전 속도 약 {v:.0f} pt/s")


if __name__ == "__main__":
    main()
