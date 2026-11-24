import re
import sys


def main():
    path, names = sys.argv[1], sys.argv[2:]
    text = open(path, encoding="utf-8").read()
    for name in names:
        if f"/* {name} */" in text:
            print(f"{name}: already there")
            continue
        used = [int(m) for m in re.findall(r"\bA[12]00000000000000000000(\d\d)\b", text)]
        n = max(used) + 1
        build, ref = f"A100000000000000000000{n:02d}", f"A200000000000000000000{n:02d}"
        text = text.replace(
            "/* End PBXBuildFile section */",
            f"\t\t{build} /* {name} in Sources */ = {{isa = PBXBuildFile; fileRef = {ref} /* {name} */; }};\n"
            "/* End PBXBuildFile section */")
        text = text.replace(
            "/* End PBXFileReference section */",
            f"\t\t{ref} /* {name} */ = {{isa = PBXFileReference; lastKnownFileType = sourcecode.swift; "
            f"path = {name}; sourceTree = \"<group>\"; }};\n/* End PBXFileReference section */")
        anchor_group = "A20000000000000000000001 /* ProbeApp.swift */,"
        anchor_sources = "A10000000000000000000001 /* ProbeApp.swift in Sources */,"
        assert anchor_group in text and anchor_sources in text, "project layout changed"
        text = text.replace(anchor_group, f"{anchor_group}\n\t\t\t\t{ref} /* {name} */,", 1)
        text = text.replace(anchor_sources, f"{anchor_sources}\n\t\t\t\t{build} /* {name} in Sources */,", 1)
        print(f"{name}: added as {ref}")
    open(path, "w", encoding="utf-8", newline="\n").write(text)


if __name__ == "__main__":
    main()
