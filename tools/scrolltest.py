import argparse
import time

from probe import Board
from taptest import read_log

X = 24000
Y_FROM, Y_TO = 26000, 13000
EXPECT = (Y_FROM - Y_TO) / 32767 * 402


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--hold", nargs="+", type=int, default=[50])
    ap.add_argument("--dur", nargs="+", type=int, default=[500])
    ap.add_argument("--step", type=int, default=12)
    ap.add_argument("--repeat", type=int, default=2)
    ap.add_argument("--ease", type=int, default=1)
    ap.add_argument("--port", default="COM3")
    args = ap.parse_args()

    board = Board(args.port)
    board.cmd(f"SET step {args.step}")
    board.cmd(f"SET ease {args.ease}")
    board.cmd("AVOID 15040 16480")
    print(board.cmd("STAT"))
    print(f"기대 스크롤 거리 {EXPECT:.1f} pt (손가락 이동 거리와 같아야 함)")
    print(f"{'hold':>5} {'dur':>5} | {'finger':>7} {'scrolled':>8} {'coast':>7} {'total':>7}  touch ms")

    seen = len(read_log())
    for hold in args.hold:
        board.cmd(f"SET hold {hold}")
        for dur in args.dur:
            for _ in range(args.repeat):
                board.cmd(f"SWIPE {X} {Y_FROM} {X} {Y_TO} {dur}")
                new = []
                deadline = time.time() + 6
                while time.time() < deadline and not new:
                    time.sleep(0.8)
                    recs = read_log()
                    new = [r for r in recs[seen:] if r.get("type") == "scroll"]
                    if new:
                        seen = len(recs)
                if not new:
                    print(f"{hold:>5} {dur:>5} | (기록 없음)")
                    print("ABORT: 스크롤 페이지가 터치를 받지 못했습니다.")
                    return
                r = new[-1]
                scrolled = r["releaseScroll"] - r["startScroll"]
                coast = r["finalScroll"] - r["releaseScroll"]
                print(f"{hold:>5} {dur:>5} | {r['fingerDy']:>7.1f} {scrolled:>8.1f} {coast:>7.1f} "
                      f"{scrolled + coast:>7.1f}  {r['touchMs']}")
    board.cmd("SET hold 50")


if __name__ == "__main__":
    main()
