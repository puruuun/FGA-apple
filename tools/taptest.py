"""터치 기록 페이지(touchlog_server.py)를 띄운 iPhone에 탭을 하나씩 보내고,
페이지가 받은 실제 터치 좌표를 출력한다.

    python taptest.py 8192,8192 16000,16383 --settle 300
"""
import argparse
import json
import subprocess
import time

from probe import MAC, Board

LOG = "/tmp/fga-touch.log"


def read_log():
    out = subprocess.run(["ssh", "-o", "BatchMode=yes", MAC, f"cat {LOG} 2>/dev/null"],
                         capture_output=True).stdout.decode(errors="replace")
    recs = []
    for line in out.splitlines():
        try:
            recs.append(json.loads(line))
        except ValueError:
            pass
    return recs


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("points", nargs="+")
    ap.add_argument("--port", default="COM3")
    ap.add_argument("--settle", type=int, help="SET settle 값(ms)")
    ap.add_argument("--hold", type=int, help="SET hold 값(ms)")
    ap.add_argument("--after", type=int, help="SET after 값(ms)")
    ap.add_argument("--burst", action="store_true", help="탭을 쉬지 않고 연달아 보낸다")
    ap.add_argument("--start", default=None, help="각 탭 전에 포인터를 보낼 위치 'x y'")
    args = ap.parse_args()

    # Safari 가로 화면에서 위쪽 약 110pt(좌표 9000 미만)는 주소창·탭 막대라서 누르지 않는다.
    for p in args.points:
        if int(p.split(",")[1]) < 11000:
            ap.error(f"{p}: y가 11000 미만이면 Safari 탭 막대를 누를 수 있어 거부합니다")

    board = Board(args.port)
    if args.settle is not None:
        board.cmd(f"SET settle {args.settle}")
    if args.hold is not None:
        board.cmd(f"SET hold {args.hold}")
    print(board.cmd("STAT"))

    if args.after is not None:
        board.cmd(f"SET after {args.after}")

    seen = len(read_log())
    if args.burst:
        # 쉬지 않고 연달아 보낸 뒤 한꺼번에 기록을 읽는다.
        t0 = time.time()
        for p in args.points:
            x, y = p.split(",")
            board.cmd(f"TAP {x} {y}")
        elapsed = time.time() - t0
        time.sleep(1.0)
        new = read_log()[seen:]
        if not new:
            print("ABORT: 테스트 페이지가 탭을 받지 못했습니다.")
            return
        downs = [r for r in new if r["type"] == "down"]
        ups = [r for r in new if r["type"] == "up"]
        moves = sum(1 for r in new if r["type"] == "move")
        print(f"burst: {len(args.points)} taps in {elapsed:.2f}s "
              f"({elapsed / len(args.points) * 1000:.0f} ms/tap), "
              f"downs={len(downs)} ups={len(ups)} moves={moves}")
        for p, d, u in zip(args.points, downs, ups):
            ok = "" if (d["clientX"], d["clientY"]) == (u["clientX"], u["clientY"]) else "  <-- moved while down"
            print(f"{p:>13} | down {d['clientX']:.1f},{d['clientY']:.1f}  "
                  f"up {u['clientX']:.1f},{u['clientY']:.1f}{ok}")
        return

    print(f"{'sent':>13} | events (type x,y in page points)")
    for p in args.points:
        x, y = (int(v) for v in p.split(","))
        if args.start:
            board.cmd(f"MOVE {args.start}")
            time.sleep(0.6)
        board.cmd(f"TAP {x} {y}")
        time.sleep(0.9)
        recs = read_log()
        new, seen = recs[seen:], len(recs)
        parts = []
        moves = 0
        for r in new:
            if r["type"] == "move":
                moves += 1
                continue
            parts.append(f"{r['type']} {r['clientX']:.1f},{r['clientY']:.1f}")
        if moves:
            parts.append(f"(+{moves} move)")
        print(f"{x:>6},{y:<6} | " + "  ".join(parts or ["(no event)"]))
        if not new:
            # 페이지가 탭을 받지 못했다면 다른 화면을 누르고 있다는 뜻이므로 즉시 멈춘다.
            print("ABORT: 테스트 페이지가 탭을 받지 못했습니다. 나머지 탭은 보내지 않습니다.")
            break
    if recs:
        r = recs[-1]
        print(f"page: inner={r['iw']}x{r['ih']} screen={r['sw']}x{r['sh']} dpr={r['dpr']} orient={r['orient']}")


if __name__ == "__main__":
    main()
