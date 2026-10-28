#!/usr/bin/env python3
"""Subsets the Noto Sans CJK fonts for the CJK languages' texts.

    python tools/i18n_fonts.py [zh-Hans ja ko]

Reads the full fonts from assets/fonts/source/ (not committed: download Noto Sans CJK SC, JP
and KR from https://github.com/notofonts/noto-cjk, folder Sans/SubsetOTF) and writes the
subsets the game embeds to assets/fonts/. Each subset has:

- the characters of the language's texts (assets/i18n/<language>/*.ftl),
- the common characters of the country's national standard, for the players' names and for
  the texts to come: GB 2312 level 1 and 2 (Chinese), JIS X 0208 level 1 (Japanese), the
  2350 syllables of KS X 1001 (Korean),
- the punctuation, the Latin letters (Bevy's text uses one font) and the languages' own names.

Run it again when a text of these languages gains a character (`cargo test` fails then:
`cjk_fonts_cover_the_texts` in src/i18n.rs), and commit the subsets.
Needs fontTools (pip install fonttools).
"""
import re
import sys
from pathlib import Path

from fontTools import subset

ROOT = Path(__file__).resolve().parent.parent
SOURCE = ROOT / "assets" / "fonts" / "source"
OUT = ROOT / "assets" / "fonts"
I18N = ROOT / "assets" / "i18n"

# language folder -> (source font, subset, national standard's codec and its row ranges)
LANGS = {
    "zh-Hans": ("NotoSansSC-Regular.otf", "NotoSansSC-Subset.otf", "gb2312", [(0xA1, 0xA9), (0xB0, 0xF7)]),
    "ja": ("NotoSansJP-Regular.otf", "NotoSansJP-Subset.otf", "euc_jp", [(0xA1, 0xA8), (0xB0, 0xCF)]),
    "ko": ("NotoSansKR-Regular.otf", "NotoSansKR-Subset.otf", "euc_kr", [(0xA1, 0xAC), (0xB0, 0xC8)]),
}

# The languages' names (OPTIONS shows them whatever the language), punctuation and spaces.
NAMES = "简体中文日本語한국어"
EXTRA = (
    "".join(chr(c) for c in range(0x20, 0x7F))
    + "".join(chr(c) for c in range(0xA0, 0x180))  # Latin-1 and Latin Extended-A
    + "".join(chr(c) for c in range(0x2010, 0x2060))  # general punctuation
    + "".join(chr(c) for c in range(0x3000, 0x3040))  # CJK punctuation
    + "".join(chr(c) for c in range(0xFF01, 0xFF5F))  # fullwidth forms
    + "".join(chr(c) for c in range(0x3131, 0x318F))  # Hangul compatibility jamo
    + "→←↑↓×°±≤≥≈…·"
)


def standard(codec, rows):
    """The characters a national standard's two-byte rows decode to."""
    out = set()
    for lo, hi in rows:
        for a in range(lo, hi + 1):
            for b in range(0xA1, 0xFF):
                try:
                    out.add(bytes([a, b]).decode(codec))
                except UnicodeDecodeError:
                    pass
    return out


def texts(folder):
    out = set()
    for f in sorted((I18N / folder).glob("*.ftl")):
        out |= set(f.read_text(encoding="utf-8"))
    return out


def build(folder):
    src, dst, codec, rows = LANGS[folder]
    chars = standard(codec, rows) | texts(folder) | set(NAMES) | set(EXTRA)
    chars = {c for c in chars if c >= " " and c != "\x7f"}
    opts = subset.Options()
    opts.layout_features = ["kern", "vert", "vrt2"]
    opts.name_IDs = ["*"]
    opts.notdef_outline = True
    opts.hinting = False
    opts.drop_tables += ["DSIG"]
    font = subset.load_font(str(SOURCE / src), opts)
    sub = subset.Subsetter(opts)
    sub.populate(unicodes=sorted(ord(c) for c in chars))
    sub.subset(font)
    subset.save_font(font, str(OUT / dst), opts)
    size = (OUT / dst).stat().st_size
    print(f"{folder}: {len(chars)} characters, {dst} {size / 1024:.0f} KiB")


if __name__ == "__main__":
    for folder in sys.argv[1:] or list(LANGS):
        build(folder)
