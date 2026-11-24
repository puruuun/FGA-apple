import re
import sys
from collections import deque
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent / "fga"
SOURCES = [ROOT / "scripts" / "src" / "main" / "java", ROOT / "libautomata" / "src" / "main" / "java"]
DECL = re.compile(
    r"^\s*(?:@\w+(?:\([^)]*\))?\s+)*(?:(?:public|internal|private|sealed|data|enum|abstract|open|value|annotation|inline|fun)\s+)*"
    r"(?:class|object|interface|typealias)\s+(\w+)", re.M)
TOPFUN = re.compile(r"^(?:(?:public|internal|inline|private)\s+)*fun\s+(?:<[^>]+>\s+)?(?:[\w.<>?, ]+\.)?(\w+)\s*\(", re.M)
WORD = re.compile(r"\b[A-Za-z_]\w*\b")


def loc(text):
    return sum(1 for l in text.splitlines() if l.strip() and not l.strip().startswith(("//", "*", "/*")))


def main():
    start = sys.argv[1]
    exclude = []
    if "--exclude" in sys.argv:
        exclude = sys.argv[sys.argv.index("--exclude") + 1].split(",")

    files = {}
    for src in SOURCES:
        for p in src.rglob("*.kt"):
            files[p] = p.read_text(encoding="utf-8")

    declared = {}
    for p, text in files.items():
        for m in DECL.finditer(text):
            declared.setdefault(m.group(1), p)
        for m in TOPFUN.finditer(text):
            if len(m.group(1)) >= 5:
                declared.setdefault(m.group(1), p)

    def group(p):
        rel = p.relative_to(ROOT).as_posix()
        if rel.startswith("libautomata"):
            return "libautomata"
        parts = rel.split("/scripts/", 1)[1].split("/") if "/scripts/" in rel else [rel]
        return parts[0] if len(parts) > 1 else "(top)"

    seen = set()
    queue = deque([declared[start]])
    while queue:
        p = queue.popleft()
        if p in seen:
            continue
        if group(p) in exclude:
            continue
        seen.add(p)
        for word in set(WORD.findall(files[p])):
            q = declared.get(word)
            if q is not None and q not in seen:
                queue.append(q)

    by_group = {}
    for p in seen:
        g = by_group.setdefault(group(p), [0, 0, []])
        g[0] += 1
        g[1] += loc(files[p])
        g[2].append((loc(files[p]), p.name))
    total_files = sum(g[0] for g in by_group.values())
    total_loc = sum(g[1] for g in by_group.values())
    all_files = len(files)
    all_loc = sum(loc(t) for t in files.values())
    print(f"start={start} exclude={exclude}")
    print(f"reachable: {total_files} files, {total_loc} lines  (of {all_files} files, {all_loc} lines)")
    for name, (n, lines, items) in sorted(by_group.items(), key=lambda kv: -kv[1][1]):
        top = ", ".join(f"{f}({l})" for l, f in sorted(items, reverse=True)[:6])
        print(f"  {name:<18} files={n:>3} lines={lines:>5}  {top}")


if __name__ == "__main__":
    main()
