#!/usr/bin/env python3
"""Draws the cockpit sprite atlas (assets/images/sprites.png).

The atlas is an indexed PNG, 320 pixels wide, whose pixel values are game palette indices.
Only the sprites the game draws are in it; their rectangles (below) mirror `cockpit::sprites`
in src/cockpit.rs. The weapon side views fill the top rows, four to a row; the HUD symbols and
map icons sit beside the last row, the heading tape and LED digits below. Aircraft on the TAC
display are drawn as rotated triangles in code, not sprites.

    python3 tools/gen_images.py
"""

import math
import os
import re

ROOT = os.path.join(os.path.dirname(os.path.abspath(__file__)), "..")
OUT = os.path.join(ROOT, "assets", "images")

W = 320


class Img:
    def __init__(self, h):
        self.h = h
        self.px = bytearray(W * h)

    def set(self, x, y, c):
        if 0 <= x < W and 0 <= y < self.h:
            self.px[y * W + x] = c

    def get(self, x, y):
        return self.px[y * W + x] if 0 <= x < W and 0 <= y < self.h else 0

    def hline(self, x0, x1, y, c):
        for x in range(x0, x1 + 1):
            self.set(x, y, c)

    def vline(self, x, y0, y1, c):
        for y in range(y0, y1 + 1):
            self.set(x, y, c)

    def rect(self, x, y, w, h, c):
        self.hline(x, x + w - 1, y, c)
        self.hline(x, x + w - 1, y + h - 1, c)
        self.vline(x, y, y + h - 1, c)
        self.vline(x + w - 1, y, y + h - 1, c)

    def line(self, x0, y0, x1, y1, c):
        dx, dy = abs(x1 - x0), -abs(y1 - y0)
        sx, sy = (1 if x0 < x1 else -1), (1 if y0 < y1 else -1)
        err = dx + dy
        while True:
            self.set(x0, y0, c)
            if x0 == x1 and y0 == y1:
                return
            e2 = 2 * err
            if e2 >= dy:
                err += dy
                x0 += sx
            if e2 <= dx:
                err += dx
                y0 += sy

    def pattern(self, x0, y0, rows, colours):
        """Draws a character picture: each character maps to a colour, '.' is skipped."""
        for j, row in enumerate(rows):
            for i, ch in enumerate(row):
                if ch != ".":
                    self.set(x0 + i, y0 + j, colours[ch])


# --- Sprite atlas -------------------------------------------------------------------------

# Sprite rectangles (x, y, w, h). These mirror `cockpit::sprites` in src/cockpit.rs, which the
# game blits from: change both together.
ORDNANCE = (0, 0, 77, 15)  # picture p at + (p % 4 * 80, p // 4 * 16)
ORDNANCE_STEP = (80, 16)
FPM = (290, 65, 13, 9)  # centre (6, 5)
AIRBASE = (304, 66, 7, 7)
TARGET = (312, 66, 7, 7)
HEADING_TAPE = (0, 80, 151, 3)
LED_DIGIT = (100, 83, 4, 7)  # digit d at + (5 * d, 0)
LED_STEP = 5
GUN_SIGHT = (162, 80, 13, 11)  # centre (6, 5)
WATERLINE = (178, 80, 25, 4)  # centre (12, 0)
HEIGHT = 91

HUD = 15  # HUD white (the gun sight and flight path marker are drawn recoloured)
LED = 47  # lit LED segment (the unlit "8" is drawn recoloured to 44)
MAP = 40  # TAC map icons


def heading_tape(img):
    """One strip, a period wider than the 141 px shown; the game offsets into it by the scroll
    phase. A ruler: a base line (on the caret's side) with 10 deg ticks rising 2 px from it and
    5 deg ticks 1 px."""
    x0, y0, w, _ = HEADING_TAPE
    img.hline(x0, x0 + w - 1, y0 + 2, HUD)
    for x in range(w):
        if x % 10 == 9:
            img.vline(x0 + x, y0, y0 + 1, HUD)
        elif x % 10 == 4:
            img.set(x0 + x, y0 + 1, HUD)


# Slanted seven-segment digits in a 4x7 cell, as on a real LED readout: the upper half
# (rows 0..3) leans one pixel right of the lower half (rows 4..6). Segment -> pixels.
SEGMENTS = {
    "a": [(1, 0), (2, 0), (3, 0)],
    "b": [(3, 0), (3, 1), (3, 2), (3, 3)],
    "c": [(2, 4), (2, 5), (2, 6)],
    "d": [(0, 6), (1, 6), (2, 6)],
    "e": [(0, 4), (0, 5), (0, 6)],
    "f": [(1, 0), (1, 1), (1, 2), (1, 3)],
    "g": [(1, 3), (2, 3), (3, 3)],
}
DIGITS = ["abcdef", "bc", "abged", "abgcd", "fgbc", "afgcd", "afgedc", "abc", "abcdefg", "abcdfg"]


def led_digits(img):
    x0, y0, _, _ = LED_DIGIT
    for d, segs in enumerate(DIGITS):
        for s in segs:
            for x, y in SEGMENTS[s]:
                img.set(x0 + LED_STEP * d + x, y0 + y, LED)


def circle(img, cx, cy, r, c):
    """Midpoint circle."""
    x, y, e = r, 0, 1 - r
    while x >= y:
        for px, py in ((x, y), (y, x), (-y, x), (-x, y), (-x, -y), (-y, -x), (y, -x), (x, -y)):
            img.set(cx + px, cy + py, c)
        y += 1
        if e < 0:
            e += 2 * y + 1
        else:
            x -= 1
            e += 2 * (y - x) + 1


def hud_symbols(img):
    # Flight path marker, centre (6,5): a circle (radius 3) with stub wings set off it and a
    # tail fin, as on F-16/F-18 HUDs.
    x, y, _, _ = FPM
    circle(img, x + 6, y + 5, 3, HUD)
    img.vline(x + 6, y, y + 1, HUD)
    img.hline(x, x + 1, y + 5, HUD)
    img.hline(x + 11, x + 12, y + 5, HUD)
    # Gun sight, centre (6,5): a gun reticle ring (radius 5) with a centre pipper, a
    # 12 o'clock index inside and range stubs outside at 3 and 9.
    x, y, _, _ = GUN_SIGHT
    circle(img, x + 6, y + 5, 5, HUD)
    img.set(x + 6, y + 5, HUD)
    img.set(x + 6, y + 2, HUD)
    img.set(x + 0, y + 5, HUD)
    img.set(x + 12, y + 5, HUD)
    # Boresight waterline, centre (12,0): level arms and a W whose middle peak marks the
    # aircraft's nose line.
    x, y, _, _ = WATERLINE
    img.hline(x + 1, x + 6, y, HUD)
    img.hline(x + 18, x + 23, y, HUD)
    img.line(x + 6, y, x + 9, y + 3, HUD)
    img.line(x + 9, y + 3, x + 12, y, HUD)
    img.line(x + 12, y, x + 15, y + 3, HUD)
    img.line(x + 15, y + 3, x + 18, y, HUD)


def map_icons(img):
    # Airbase, 7x7: an aircraft in plan view (the airfield symbol of map legends).
    x, y, _, _ = AIRBASE
    img.pattern(x, y, [
        "...#...",
        "...#...",
        "..###..",
        ".#####.",
        "#..#..#",
        "...#...",
        "..###..",
    ], {"#": MAP})
    # Target, 7x7: a hostile diamond (NATO style) with a centre dot.
    x, y, _, _ = TARGET
    img.pattern(x, y, [
        "...#...",
        "..#.#..",
        ".#...#.",
        "#..#..#",
        ".#...#.",
        "..#.#..",
        "...#...",
    ], {"#": MAP})


def sprites():
    img = Img(HEIGHT)
    ordnance(img)
    hud_symbols(img)
    map_icons(img)
    heading_tape(img)
    led_digits(img)
    return img


# --- Weapon side views ---------------------------------------------------------------------

OUTLINE = 10  # bright green
DETAIL = 46  # dim green seams
FRAME = 47  # box frame, as the WPN page frame

# Picture p is the ORDNANCE box moved by ORDNANCE_STEP (p % 4, p // 4), 77x15 including its
# frame.
BOX_W, BOX_H = ORDNANCE[2], ORDNANCE[3]
CY = 7.5  # centre line of the box (pixel centres are at +0.5)


def inside(poly, x, y):
    n, hit = len(poly), False
    for i in range(n):
        (x0, y0), (x1, y1) = poly[i], poly[(i + 1) % n]
        if (y0 > y) != (y1 > y) and x < x0 + (y - y0) * (x1 - x0) / (y1 - y0):
            hit = not hit
    return hit


class Shape:
    """Union of polygons and ellipses in box coordinates, outlined in green."""

    def __init__(self):
        self.polys = []
        self.ellipses = []
        self.details = []

    def poly(self, *pts):
        self.polys.append(pts)

    def mirror(self, *pts):
        """A fin above the centre line and its mirror below."""
        self.poly(*pts)
        self.poly(*[(x, 2 * CY - y) for x, y in pts])

    def fin(self, x, r, root, tip, span, sweep=0.0):
        """A fin pair on a body of half height r: root chord `root` from x (its trailing edge),
        tip chord `tip`, `span` px out, trailing edge `sweep` px further back at the tip."""
        y0, y1 = CY - r + 0.5, CY - r - span
        self.mirror((x, y0), (x + root, y0), (x + tip - sweep, y1), (x - sweep, y1))

    def body(self, x0, x1, r, nose=0.0, tail=0.0, tip=0.0):
        """Round body from x0 (tail) to x1 (nose) with half height r, tapering to `tip` over
        `nose` px at the front and to r/2 over `tail` px at the back."""
        pts = [(x0, CY - r / 2 if tail else CY - r), (x0 + tail, CY - r), (x1 - nose, CY - r)]
        if nose:
            k = 6
            for i in range(1, k + 1):
                t = i / k
                h = tip + (r - tip) * math.sqrt(1 - t * t)
                pts.append((x1 - nose + nose * t, CY - h))
        pts += [(x, 2 * CY - y) for x, y in reversed(pts)]
        self.poly(*pts)

    def ellipse(self, cx, cy, rx, ry):
        self.ellipses.append((cx, cy, rx, ry))

    def seam(self, x0, y0, x1, y1):
        self.details.append((x0, y0, x1, y1))

    def ring(self, x, r):
        """A vertical seam across a body of half height r at x."""
        self.seam(x, int(CY - r) + 1, x, int(CY + r) - 1)

    def hit(self, x, y):
        px, py = x + 0.5, y + 0.5
        if any(inside(p, px, py) for p in self.polys):
            return True
        return any(((px - cx) / rx) ** 2 + ((py - cy) / ry) ** 2 <= 1 for cx, cy, rx, ry in self.ellipses)

    def draw(self, img, bx, by):
        mask = {(x, y) for y in range(1, BOX_H - 1) for x in range(1, BOX_W - 1) if self.hit(x, y)}
        for x, y in mask:
            if any((x + dx, y + dy) not in mask for dx, dy in ((1, 0), (-1, 0), (0, 1), (0, -1))):
                img.set(bx + x, by + y, OUTLINE)
        for x0, y0, x1, y1 in self.details:
            for x in range(x0, x1 + 1) if y0 == y1 else [x0]:
                for y in range(y0, y1 + 1) if x0 == x1 else [y0]:
                    if (x, y) in mask and img.get(bx + x, by + y) != OUTLINE:
                        img.set(bx + x, by + y, DETAIL)


def sidewinder(s):
    s.body(26, 74, 1.5, nose=5, tip=0.5)
    s.fin(64, 1.5, 4, 2, 2.5, sweep=1.5)  # canards
    s.fin(26, 1.5, 7, 3.5, 2)  # tail fins


def amraam(s):
    s.body(26, 74, 2, nose=7, tip=0.5)
    s.fin(46, 2, 7, 3, 2.5, sweep=3)
    s.fin(26, 2, 5, 3, 2)
    s.ring(62, 2)


def harm(s):
    s.body(22, 74, 2.5, nose=7, tip=1)
    s.fin(48, 2.5, 9, 5, 3.5, sweep=3)
    s.fin(22, 2.5, 5, 3, 2.5)
    s.ring(62, 2.5)
    s.ring(36, 2.5)


def penguin(s):
    s.body(28, 72, 2, nose=5, tip=1)
    s.fin(42, 2, 10, 4, 4, sweep=3)  # folding wings
    s.fin(60, 2, 3, 1.5, 1.5, sweep=1)  # canards
    s.fin(28, 2, 4, 2, 2)
    s.ring(56, 2)


def harpoon(s):
    s.body(24, 74, 3, nose=7, tip=1)
    s.fin(44, 3, 8, 4, 3, sweep=3)
    s.fin(24, 3, 6, 3, 2.5)
    s.poly((36, 9.5), (44, 9.5), (44, 11.5), (38, 11.5))  # intake
    s.ring(58, 2.5)


def maverick(s):
    s.body(30, 72, 3, nose=5, tip=2)
    s.fin(30, 3, 22, 12, 2.5, sweep=4)  # long-chord wings
    s.ring(64, 3)


def paveway(s):
    s.body(30, 64, 3, nose=6, tail=8, tip=1.5)
    s.body(62, 74, 1.5, nose=3, tip=1)  # seeker
    s.fin(65, 1.5, 4, 2, 2.5, sweep=1)  # canards
    s.fin(30, 2, 10, 4, 3.5, sweep=3)  # wings
    s.ring(46, 3)


def rockeye(s):
    s.body(32, 72, 2.5, nose=6, tip=1)
    s.fin(30, 2.5, 6, 3, 2.5)
    s.seam(40, 7, 64, 7)


def durandal(s):
    s.body(24, 74, 2.5, nose=4, tip=1.5)
    s.fin(24, 2.5, 5, 3, 2)
    s.ring(36, 2.5)
    s.ring(60, 2.5)


def slick(s):
    s.body(36, 72, 3, nose=10, tail=10, tip=1)
    s.fin(30, 1.5, 8, 4, 3)
    s.ring(50, 3)


def snakeye(s):
    s.body(40, 72, 3, nose=10, tail=8, tip=1)
    s.poly((30, 6.5), (42, 6.5), (42, 8.5), (30, 8.5))
    s.mirror((40, 6.5), (42, 6.5), (33, 1.5), (31, 1.5))  # retarder fins, opened
    s.ring(54, 3)


def rockeye2(s):
    s.body(32, 72, 2.5, nose=8, tip=0.5)
    s.fin(30, 2.5, 6, 3, 2.5)
    s.ring(44, 2.5)
    s.ring(56, 2.5)


def fireeye(s):
    s.body(30, 72, 3, nose=8, tail=6, tip=1.5)
    s.fin(28, 1.5, 6, 3, 2)
    s.seam(40, 7, 62, 7)
    s.seam(40, 8, 62, 8)


def fuel_air(s):
    s.body(36, 66, 3.5, nose=8, tail=6, tip=2)
    s.body(64, 74, 0.5)  # fuze probe
    s.fin(30, 2, 8, 4, 2)
    s.ring(50, 3.5)


def in_cluster(s):
    s.body(30, 72, 3.5, nose=6, tail=4, tip=2)
    s.fin(28, 2, 6, 3, 2)
    for x in range(40, 63, 4):
        s.ring(x, 3.5)


def minelets(s):
    s.body(26, 74, 3, nose=4, tail=3, tip=2)
    s.seam(32, 7, 68, 7)
    s.seam(32, 8, 68, 8)
    for x in range(36, 68, 6):
        s.ring(x, 3)


def camera(s):
    s.body(32, 72, 3, nose=8, tail=6, tip=2)
    s.poly((52, 10), (60, 10), (59, 12), (53, 12))  # lens fairing
    s.ring(46, 3)


def fuel_tank(s):
    s.body(24, 74, 4, nose=16, tail=18, tip=1)
    s.fin(24, 1.5, 7, 3, 1)
    s.ring(50, 4)
    s.poly((44, 2), (52, 2), (52, 4), (46, 4))  # pylon lug


def gun_pod(s):
    s.body(32, 66, 3, nose=6, tail=4, tip=1.5)
    s.poly((64, 6.5), (75, 6.5), (75, 8.5), (64, 8.5))  # barrel
    s.ring(44, 3)
    s.ring(56, 3)


# Picture number (weapon table `picture`) -> drawing.
WEAPONS = [
    sidewinder,  # 0 Sidewinder
    paveway,  # 1 Paveway
    fireeye,  # 2 Fireeye
    amraam,  # 3 AMRAAM
    rockeye,  # 4 Rockeye
    gun_pod,  # 5 Equip / Guns
    fuel_air,  # 6 Fuel-Air
    harm,  # 7 HARM
    durandal,  # 8 Durandal
    in_cluster,  # 9 IN Cluster
    penguin,  # 10 Penguin
    slick,  # 11 Slick
    harpoon,  # 12 Harpoon
    snakeye,  # 13 Snakeye
    minelets,  # 14 Minelets
    camera,  # 15 Camera
    maverick,  # 16 Maverick
    rockeye2,  # 17 Rockeye II
    fuel_tank,  # 18 Extra Fuel
]


def ordnance(img):
    for p, draw in enumerate(WEAPONS):
        bx = ORDNANCE[0] + p % 4 * ORDNANCE_STEP[0]
        by = ORDNANCE[1] + p // 4 * ORDNANCE_STEP[1]
        img.rect(bx, by, BOX_W, BOX_H, FRAME)
        s = Shape()
        draw(s)
        s.draw(img, bx, by)


# --- Output ----------------------------------------------------------------------------------


def palette():
    with open(os.path.join(ROOT, "assets", "palettes", "base.ron")) as f:
        cols = re.findall(r"\((\d+), (\d+), (\d+)\)", f.read())
    return [tuple(int(v) * 255 // 63 for v in c) for c in cols[:256]]


def write(name, img):
    from PIL import Image

    p = Image.frombytes("P", (W, img.h), bytes(img.px))
    p.putpalette([v for c in palette() for v in c])
    p.save(os.path.join(OUT, name + ".png"))


if __name__ == "__main__":
    write("sprites", sprites())
