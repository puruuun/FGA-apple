import re
import subprocess
import sys

IMAGE_SUFFIXES = (
    ".jpg", ".jpeg", ".png", ".gif", ".heic", ".heif", ".bmp", ".webp",
    ".tif", ".tiff", ".avif",
)
BLOCKED_SUFFIXES = IMAGE_SUFFIXES + (
    ".mp4", ".mov", ".m4v", ".mkv", ".webm", ".avi", ".pdf",
    ".fga", ".log", ".mobileprovision", ".p12",
)
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


def is_allowed_image(path):
    return path.lower().endswith(IMAGE_SUFFIXES) and path.startswith(ALLOWED_IMAGE_PREFIXES)


def is_blocked(path):
    return path.lower().endswith(BLOCKED_SUFFIXES) and not is_allowed_image(path)


def is_media(data):
    if data.startswith((
        b"\x89PNG\r\n\x1a\n", b"\xff\xd8\xff", b"GIF87a", b"GIF89a",
        b"II*\x00", b"MM\x00*", b"\x1a\x45\xdf\xa3", b"%PDF-",
    )):
        return True
    if data[:4] == b"RIFF" and data[8:12] in (b"WEBP", b"AVI "):
        return True
    if data[4:8] == b"ftyp":
        return True
    return data[:2] == b"BM" and data[6:10] == b"\x00\x00\x00\x00"


def as_text(data):
    try:
        return data.decode("utf-8")
    except UnicodeDecodeError:
        return data.replace(b"\x00", b"").decode("latin-1")


def main():
    problems = []
    for path in tracked_files():
        if is_blocked(path):
            problems.append(f"{path}: this kind of file is not committed")
            continue
        try:
            with open(path, "rb") as f:
                data = f.read()
        except OSError:
            continue
        if is_media(data) and not is_allowed_image(path):
            problems.append(f"{path}: this kind of file is not committed")
            continue
        for number, line in enumerate(as_text(data).splitlines(), 1):
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
