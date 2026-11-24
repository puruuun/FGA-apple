import re
import subprocess
import sys

BLOCKED_SUFFIXES = (
    ".jpg", ".jpeg", ".png", ".gif", ".heic", ".bmp", ".webp",
    ".mp4", ".mov", ".mkv", ".fga", ".log", ".mobileprovision", ".p12",
)
IMAGE_SUFFIXES = (".jpg", ".jpeg", ".png", ".gif", ".heic", ".bmp", ".webp")
ALLOWED_IMAGE_PREFIXES = ("core/testdata/synthetic/",)
PRIVATE_ADDRESS = re.compile(
    r"(?<![\d.])(?:192\.168|10\.\d{1,3}|172\.(?:1[6-9]|2\d|3[01]))\.\d{1,3}\.\d{1,3}(?![\d.])"
)
SIGNING_TEAM = re.compile(r"DEVELOPMENT_TEAM\s*=\s*\"?[A-Z0-9]{10}\"?\s*;")
SSH_TARGET = re.compile(r"\b[a-z][a-z0-9_-]*@(?:\d{1,3}\.){3}\d{1,3}\b")
DEVICE_ID = re.compile(r"\b0000[0-9A-F]{4}-[0-9A-F]{16}\b")


def tracked_files():
    out = subprocess.run(["git", "ls-files", "-z"], capture_output=True, check=True).stdout
    return [p for p in out.decode().split("\0") if p]


def is_blocked(path):
    lowered = path.lower()
    if not lowered.endswith(BLOCKED_SUFFIXES):
        return False
    if lowered.endswith(IMAGE_SUFFIXES) and path.startswith(ALLOWED_IMAGE_PREFIXES):
        return False
    return True


def main():
    problems = []
    for path in tracked_files():
        if path.startswith("fga/") or path == "fga":
            continue
        if is_blocked(path):
            problems.append(f"{path}: this kind of file is not committed")
            continue
        try:
            with open(path, encoding="utf-8") as f:
                text = f.read()
        except (UnicodeDecodeError, OSError):
            continue
        for number, line in enumerate(text.splitlines(), 1):
            for name, pattern in (
                ("private network address", PRIVATE_ADDRESS),
                ("signing team id", SIGNING_TEAM),
                ("ssh target", SSH_TARGET),
                ("device identifier", DEVICE_ID),
            ):
                if pattern.search(line):
                    problems.append(f"{path}:{number}: {name}")
    for problem in problems:
        print(problem)
    if problems:
        print(f"{len(problems)} problem(s)")
        return 1
    print("clean")
    return 0


if __name__ == "__main__":
    sys.exit(main())
