#!/usr/bin/env python3
"""Cuts assets/fonts/lucide.ttf down to the icons the game uses (the code points written
'\\u{eXXX}' in src/menu.rs and src/flight_menu.rs), from the full Lucide font kept locally
(not in git) as assets-src/fonts/lucide-backup.ttf. Missing? Take it from before the cut:

    git show 47081e4:assets/fonts/lucide.ttf > assets-src/fonts/lucide-backup.ttf

    pip install fonttools
    tools/subset_lucide.py [full.ttf [out.ttf]]

Run it again after adding an icon to those two files.
"""
import os
import re
import sys

from fontTools import subset
from fontTools.ttLib import TTFont

ROOT = os.path.join(os.path.dirname(os.path.abspath(__file__)), "..")
SOURCES = ["src/menu.rs", "src/flight_menu.rs"]
CODE_POINT = re.compile(r"\\u\{([0-9a-fA-F]+)\}")


def used_icons():
    icons = set()
    for path in SOURCES:
        with open(os.path.join(ROOT, path), encoding="utf-8") as f:
            icons.update(int(m, 16) for m in CODE_POINT.findall(f.read()))
    return sorted(icons)


def main(src, dst):
    icons = used_icons()
    font = TTFont(src)
    missing = [c for c in icons if c not in font.getBestCmap()]
    if missing:
        sys.exit("not in the font: " + ", ".join(f"U+{c:04X}" for c in missing))
    options = subset.Options()
    options.layout_features = []
    options.name_IDs = [0, 1, 2, 3, 4, 5, 6]
    options.notdef_outline = True
    options.glyph_names = False
    options.hinting = False
    subsetter = subset.Subsetter(options)
    subsetter.populate(unicodes=icons)
    subsetter.subset(font)
    font.save(dst)
    print(f"{len(icons)} icons, {os.path.getsize(src)} -> {os.path.getsize(dst)} bytes")


if __name__ == "__main__":
    args = sys.argv[1:]
    main(
        args[0] if args else os.path.join(ROOT, "assets-src", "fonts", "lucide-backup.ttf"),
        args[1] if len(args) > 1 else os.path.join(ROOT, "assets", "fonts", "lucide.ttf"),
    )
