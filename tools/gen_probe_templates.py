import base64
import sys
from pathlib import Path

from PIL import Image

ROOT = Path(__file__).resolve().parent.parent
ASSETS = ROOT / "fga" / "app" / "src" / "main" / "assets"
OUT = ROOT / "ios" / "FGAProbe" / "FGAProbe" / "Templates.swift"

TEMPLATES = {"menu": "menu.png", "supportScreen": "support_screen.png"}


def main():
    server = sys.argv[1] if len(sys.argv) > 1 else "Kr"
    lines = [
        "// tools/gen_probe_templates.py가 만든 파일. 직접 고치지 않는다.",
        f"// 원본: FGA-Modified의 app/src/main/assets ({server}, 없으면 En). 8비트 흑백, 행 우선.",
        "",
        "import Foundation",
        "",
        "enum Templates {",
    ]
    for name, file in TEMPLATES.items():
        path = ASSETS / server / file
        if not path.is_file():
            path = ASSETS / "En" / file
        img = Image.open(path).convert("L")
        data = base64.b64encode(img.tobytes()).decode()
        lines.append(f"    // {path.relative_to(ASSETS).as_posix()}")
        lines.append(f'    static let {name} = Template(name: "{name}", width: {img.width}, height: {img.height}, base64: "{data}")')
        print(f"{name}: {path.relative_to(ASSETS).as_posix()} {img.width}x{img.height}")
    lines.append("}")
    OUT.write_text("\n".join(lines) + "\n", encoding="utf-8", newline="\n")


if __name__ == "__main__":
    main()
