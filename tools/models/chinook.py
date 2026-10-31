"""Builds the CH-47 Chinook (`aircraft_049`) in aircraft.blend with three detail levels, and saves it:

    blender -b assets-src/blend/aircraft.blend --python-exit-code 1 --python tools/models/chinook.py
    tools/models/export.sh aircraft

The long boxy fuselage with its rounded corners, the bulbous nose under the steep windshield,
the forward pylon over the cockpit carrying the front rotor and the tall aft pylon carrying the
rear one, the two engines on the aft pylon's sides, the fuel sponsons along the lower sides, the
loading ramp under the tail, twin front wheels and single rear ones. Proportions are taken from
a three-view drawing: the side view gives the lengths and heights (35 px per metre, the ground at
row 733, y 0 halfway between the rotor hubs at column 822.5), the plan and front views the widths
(33.6 px per metre). The drawing sits on its wheels with the roof sloping down a little to the
rear, and so does the model. The origin is on the ground between the rotors.

The cockpit's windows are cut into the nose's skin exactly along the drawing's outlines: each
window is the part of the skin whose side view and front view both fall inside the drawn shapes
(`WINDOWS`); the skin's triangles are clipped along them, so frames and posts are what is left.

Each level carries its two rotors (`_part1` the front rotor about its hub, built upside down so it
turns anticlockwise seen from above, `_part2` the rear rotor turning clockwise; both on slot 17
about the model's up axis), navigation lights (`_s12v1`) and red beacons (`_s20v1`). Built in
metres and scaled to aircraft units (K per metre). Model space: x right, y forward, z up.
"""

import math
import os
import sys

import bpy
from mathutils import Vector

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.abspath(os.path.join(HERE, "..", ".."))
sys.path.insert(0, HERE)

import heli  # noqa: E402
import mklib  # noqa: E402
from mklib import Geo, Ramp, frustum, v3  # noqa: E402

NAME = "aircraft_049"
K = 16.38
# max_dist of each level (the game draws them 4x further): about 600 m and 2.2 km.
LOD_DISTS = [2500, 9000, 32769]

BODY = Ramp(0xA7, 0xAE)
DECK = Ramp(0xA8, 0xAF)
GLASS = Ramp(144, 145)
WINDOW = 145
DARK = Ramp(0xEB, 0xEF)
METAL = Ramp(0xE8, 0xEC)
BLADE = Ramp(0xEC, 0xEF)
TYRE = 0xEF
SEAM = 0xAF
RED, GREEN = 147, 151


def side(px, row):
    """A side view drawing point (column, row) as (y, z) metres."""
    return ((822.5 - px) / 35.0, (733.0 - row) / 35.0)


# The front view's rows don't share the side view's vertical scale: they are matched to it on
# the windows' tops and bottoms.
FRONT_ROWS = [(226, 3.60), (246, 3.20), (270, 2.70), (321, 1.60)]


def front(px, row):
    """A front view drawing point as (x, z) metres."""
    x = (px - 884.5) / 33.6
    rows = FRONT_ROWS
    for (r0, z0), (r1, z1) in zip(rows, rows[1:]):
        if row <= r1 or (r1, z1) == rows[-1]:
            return (x, z0 + (z1 - z0) * (row - r0) / (r1 - r0))
    raise ValueError(row)


def interp(table, t):
    """Piecewise linear through (t, value) pairs sorted by t; clamped at the ends."""
    if t <= table[0][0]:
        return table[0][1]
    for (t0, v0), (t1, v1) in zip(table, table[1:]):
        if t <= t1:
            return v0 + (v1 - v0) * (t - t0) / (t1 - t0)
    return table[-1][1]


# The fuselage's side view outline: its top (the windshield, then the roof; hidden under the
# pylons) and its bottom (the nose, the belly, the ramp), as rows by column.
TOP = [(545, 654), (545.5, 652), (547, 649), (549, 647), (551, 645), (553, 643), (557, 641), (561, 639), (566, 637), (572, 635), (577, 633),
       (578, 631), (579, 628), (580, 625), (582, 621), (584, 617), (586, 613), (588, 609), (590, 605), (592, 604), (683, 603),
       (930, 612), (1100, 618)]
BOTTOM = [(545, 659), (545.5, 661), (547, 666), (548, 668), (550, 670), (552, 672), (555, 674), (557, 676), (561, 678), (566, 680),
          (570, 681), (580, 682), (600, 684), (650, 690), (690, 697), (700, 700), (800, 704), (880, 707), (945, 709),
          (960, 705), (1000, 698), (1050, 682), (1080, 665), (1100, 650)]
# Half width by distance back from the nose's tip (plan view, metres), and the tail's narrowing.
NOSE_W = [(0.0, 0.30), (0.03, 0.50), (0.08, 0.64), (0.15, 0.76), (0.23, 0.86), (0.32, 0.95), (0.41, 0.99), (0.50, 1.05), (0.58, 1.11),
          (0.67, 1.14), (0.76, 1.20), (0.93, 1.25), (1.20, 1.31), (1.55, 1.34), (1.72, 1.35)]
TAIL_W = [(-7.95, 0.75), (-7.87, 0.88), (-7.70, 1.08), (-7.52, 1.30), (-7.30, 1.35)]
TIP_Y = side(545, 0)[0]
# The fuselage's stations (drawing columns), fine through the nose for the windows' cuts.
COLUMNS = [545, 545.5, 547, 549, 551, 553, 555, 557, 561, 566, 572, 577, 579, 582, 584, 586, 588, 590, 593, 597, 602, 607, 613,
           620, 630, 645, 665, 683, 700, 740, 790, 840, 890, 930, 960, 1000, 1040, 1070, 1085, 1093, 1100]
COLUMNS1 = [545, 545.5, 547, 551, 555, 561, 566, 572, 577, 580, 584, 588, 590, 597, 607, 620, 645, 683, 760, 860, 930, 960, 1000,
            1050, 1085, 1100]
COLUMNS2 = [545, 551, 561, 577, 590, 620, 683, 930, 1000, 1070, 1100]


def width(y):
    if y > 0:
        return interp(NOSE_W, TIP_Y - y)
    return interp(TAIL_W, y)


def profile(zb, zt, w, rb, rt, segs):
    """Right half of a cross-section from the belly's centre to the roof's centre: the flat
    belly, its rounded corner, the flat side, the roof's rounded corner and the flat roof.
    `segs` = (belly, corner, side, roof corner, roof) segment counts, the same at every station."""
    nb, na, ns, nr, nt = segs
    h = zt - zb
    rb, rt = min(rb, w, h * 0.5), min(rt, w, h * 0.5)
    pts = []
    for k in range(nb):
        pts.append(((w - rb) * k / nb, zb))
    for k in range(na):
        a = -math.pi / 2 + math.pi / 2 * k / na
        pts.append((w - rb + rb * math.cos(a), zb + rb + rb * math.sin(a)))
    for k in range(ns):
        pts.append((w, zb + rb + (h - rb - rt) * k / ns))
    for k in range(nr):
        a = math.pi / 2 * k / nr
        pts.append((w - rt + rt * math.cos(a), zt - rt + rt * math.sin(a)))
    for k in range(nt + 1):
        pts.append(((w - rt) * (1 - k / nt), zt))
    return pts


def station(px, segs):
    """The fuselage's ring at a drawing column: points round from the belly's centre."""
    y, zt = side(px, interp(TOP, px))
    _, zb = side(px, interp(BOTTOM, px))
    w = width(y)
    nose = min(max((TIP_Y - y) / 0.9, 0.0), 1.0)
    rb = 0.18 + 0.12 * nose
    rt = 0.55
    half = profile(zb, zt, w, rb, rt, segs)
    right = [Vector((x, y, z)) for x, z in half]
    left = [Vector((-x, y, z)) for x, z in reversed(half[1:-1])]
    return right + left


# --- Windows -----------------------------------------------------------------------------------
# A window is a list of constraints, all of which a point of the skin must meet: a convex outline
# in the side view ((y, z), either side) or in the front view ((x, z)).

def S(*pts):
    return ("side", [side(*p) for p in pts])


def F(*pts, mirror=False):
    out = [front(*p) for p in pts]
    if mirror:
        out = [(-x, z) for x, z in reversed(out)]
    return ("front", out)


# Front view outlines (left side, drawing columns and rows; the right ones are mirrored).
CHIN_F = [(842, 275), (873, 306), (873, 321), (857, 321), (846, 315), (842, 300)]
# The eyebrow window's outer edge is the body's outline there: it runs out to the skin.
BROW_F = [(820, 240), (820, 226), (861, 226), (861, 240)]
# The windshield: two big panes and a narrow one between them.
SHIELD_F = [[(845, 246), (874, 246), (874, 270), (845, 270)], [(878, 246), (890, 246), (890, 266), (878, 266)]]
# Side view outlines: the chin window wrapping round the nose's corner (how far forward it reaches
# on the nose's front is the front view's outline); the big lower side window behind the diagonal post (both by the pilots' feet); the two
# upper side windows (the rear one's chevron in two pieces); the eyebrow window over the
# windshield's corner; the windshield's side, forward of its frame.
CHIN_S = S((582, 641), (567, 677), (540, 677), (540, 641))
LOWER_S = S((587, 646), (577, 667), (599, 668), (609, 647))
UPPER_S = [S((598, 622), (588, 642), (600, 642), (606, 630), (602, 622)),
           S((606, 622), (610, 630), (617, 630), (613, 622)),
           S((610, 630), (605, 642), (612, 642), (617, 630))]
BROW_S = S((575, 618), (610, 618), (606, 611), (600, 607), (580, 605))
SHIELD_S = S((540, 600), (540, 645), (583, 645), (601.6, 600))

WINDOWS = (
    [[CHIN_S, F(*CHIN_F)], [CHIN_S, F(*CHIN_F, mirror=True)], [LOWER_S]]
    + [[u] for u in UPPER_S]
    + [[BROW_S, F(*BROW_F)], [BROW_S, F(*BROW_F, mirror=True)]]
    + [[SHIELD_S, F(*p)] for p in SHIELD_F]
    + [[SHIELD_S, F(*SHIELD_F[0], mirror=True)]]
)


def project(view, p):
    return (p.y, p.z) if view == "side" else (p.x, p.z)


def half_planes(view, poly):
    """The outline's edges as functions >= 0 inside (whichever way the outline winds)."""
    area = sum(a[0] * b[1] - b[0] * a[1] for a, b in zip(poly, poly[1:] + poly[:1]))
    s = 1.0 if area > 0 else -1.0
    out = []
    for (ax, ay), (bx, by) in zip(poly, poly[1:] + poly[:1]):
        def f(p, ax=ax, ay=ay, bx=bx, by=by):
            u, v = project(view, p)
            return s * ((bx - ax) * (v - ay) - (by - ay) * (u - ax))
        out.append(f)
    return out


def clip(poly, f, keep_inside):
    """Sutherland-Hodgman: the part of a 3D polygon where f >= 0 (or < 0)."""
    out = []
    n = len(poly)
    for i in range(n):
        a, b = poly[i], poly[(i + 1) % n]
        fa, fb = f(a), f(b)
        ina, inb = (fa >= 0) == keep_inside, (fb >= 0) == keep_inside
        if ina:
            out.append(a)
        if ina != inb:
            t = fa / (fa - fb)
            out.append(a + (b - a) * t)
    return out if len(out) >= 3 else []


def cut(poly, windows):
    """Splits a skin polygon into (glass pieces, body pieces) along the windows."""
    glass, body = [], []
    rest = [poly]
    for win in windows:
        nxt = []
        for piece in rest:
            inside = piece
            for view, outline in win:
                for f in half_planes(view, outline):
                    out = clip(inside, f, False)
                    if out:
                        nxt.append(out)
                    inside = clip(inside, f, True)
                    if not inside:
                        break
                if not inside:
                    break
            if inside:
                glass.append(inside)
        rest = nxt
    body += rest
    return glass, body


def area(poly):
    n = Vector()
    for i in range(len(poly)):
        n += poly[i].cross(poly[(i + 1) % len(poly)])
    return n.length / 2


def fuselage(columns, segs, glazed=True):
    """The fuselage's skin through the stations, the nose's windows cut into it."""
    g = Geo()
    rings = [station(px, segs) for px in columns]
    m = len(rings[0])
    window_y = side(625, 0)[0]
    for i in range(len(rings) - 1):
        a, b = rings[i], rings[i + 1]
        for k in range(m):
            k1 = (k + 1) % m
            # Wound to face outwards.
            quad = [a[k], a[k1], b[k1], b[k]]
            tris = [[quad[0], quad[1], quad[2]], [quad[0], quad[2], quad[3]]]
            for t in tris:
                if area(t) < 1e-7:
                    continue
                if glazed and max(p.y for p in t) > window_y:
                    gl, bd = cut(t, WINDOWS)
                else:
                    gl, bd = [], [t]
                for p in gl:
                    if area(p) > 1e-6:
                        g.face(p, GLASS)
                for p in bd:
                    if area(p) > 1e-6:
                        g.face(p, BODY)
    # Caps: the flat front of the nose and the rear end.
    for r, other in ((rings[0], rings[1]), (rings[-1], rings[-2])):
        c = sum(other, Vector()) / m
        g.face(r, BODY, c)
    return g


# --- Pylons, engines, sponsons, gear ----------------------------------------------------------

def column_ring(y, zb, zt, wb, wt, r, n):
    """A pylon's section: sides leaning in from the foot, the top's corners rounded."""
    r = min(r, wt, (zt - zb) * 0.5)
    right = [(wb, zb)]
    for k in range(n + 1):
        a = math.pi / 2 * k / n
        right.append((wt - r + r * math.cos(a), zt - r + r * math.sin(a)))
    pts = right + [(-x, z) for x, z in reversed(right)]
    return [Vector((x, y, z)) for x, z in pts]


# The pylons' side view top lines (columns, rows) over the roof; their foot is sunk into it.
FWD_PYLON = [(594, 603), (595, 590), (597, 578), (605, 572), (615, 568), (630, 568), (650, 573), (670, 578), (680, 581),
             (683, 590), (684, 602)]
AFT_PYLON = [(931, 611), (936, 604), (942, 592), (948, 580), (955, 567), (962, 556), (970, 550), (985, 546), (1005, 541),
             (1030, 538), (1060, 540), (1085, 546), (1095, 551), (1100, 556)]


def pylon(top, wb, wt, level, sink=0.35):
    """A pylon along its side view top line, its foot `sink` under the roof."""
    n = (4, 2, 1)[level]
    pts = top if level < 2 else top[::2] + ([top[-1]] if len(top) % 2 == 0 else [])
    rings = []
    for px, row in pts:
        y, zt = side(px, row)
        _, roof = side(px, interp(TOP, px))
        rings.append(column_ring(y, roof - sink, max(zt, roof - sink + 0.02), wb, wt, 0.35, n))
    return mklib.tube(rings, DECK)


# Rotor hubs, the rotors' radius and chord (60 ft, 3 blades of 32 in).
FWD_HUB = (0.0, *side(622, 563))
AFT_HUB = (0.0, *side(1023, 522))
ROTOR_R, ROTOR_CHORD = 9.14, 0.81


def heads(level):
    """The rotor heads' fairings and masts on top of the pylons."""
    g = Geo()
    n = 8 if level == 0 else 6
    for hub, foot in ((FWD_HUB, side(622, 568)[1]), (AFT_HUB, side(1023, 537)[1])):
        x, y, z = hub
        g.add(frustum((x, y, foot - 0.10), (x, y, foot + 0.10), 0.55, 0.42, METAL, n=n))
        g.add(frustum((x, y, foot + 0.10), (x, y, z - 0.08), 0.20, 0.17, METAL, n=n))
    return g


# The engines: the nacelle's sections along its axis (y, radius), its centre's height and side.
# The intake's wide screened cone in front, the narrower nacelle, the exhaust.
ENGINE_X, ENGINE_Z = 1.35, 3.40
NACELLE = [(side(950, 0)[0], 0.22), (side(976, 0)[0], 0.62), (side(978, 0)[0], 0.44), (side(1005, 0)[0], 0.44),
           (side(1020, 0)[0], 0.38), (side(1022, 0)[0], 0.34), (side(1040, 0)[0], 0.33), (side(1041, 0)[0], 0.26)]


def engines(level):
    g = Geo()
    n = (12, 8, 6)[level]
    st = NACELLE if level == 0 else [NACELLE[i] for i in (0, 1, 2, 4, 6)]
    for sx in (-1, 1):
        x = sx * ENGINE_X
        rings = [mklib.ring((x, y, ENGINE_Z), r, r, n) for y, r in st]
        paints = [METAL] + [BODY] * (len(st) - 3) + [DARK, DARK]
        g.add(mklib.tube(rings, BODY, paints=paints[:len(st) - 1]))
        if level < 2:
            # The pylon between the nacelle and the aft pylon's side.
            y0, y1 = side(978, 0)[0], side(1010, 0)[0]
            xi, xo = sx * 0.75, sx * (ENGINE_X - 0.3)
            g.add(mklib.block(min(xi, xo), max(xi, xo), y1, y0, ENGINE_Z - 0.18, ENGINE_Z + 0.18, BODY))
    return g


# The sponsons along the lower sides: front and rear ends (columns), the top's rows there.
SPONSON = (683, 1003)
SPONSON_TOP = [(683, 655), (1003, 669)]
SPONSON_W = (1.30, 1.95)


def sponson_ring(px, sx, scale=1.0):
    y, _ = side(px, 0)
    _, zb = side(px, interp(BOTTOM, px))
    _, zt = side(px, interp(SPONSON_TOP, px))
    zt = zb + (zt - zb) * scale
    wi, wo = SPONSON_W
    wo = wi + (wo - wi) * min(1.0, scale * 1.1)
    zb += 0.02
    pts = [(wi, zb), (wo - 0.15, zb), (wo, zb + 0.15), (wo, zt - 0.35 * (zt - zb)), (wo - 0.25, zt - 0.05), (wi, zt)]
    return [Vector((sx * x, y, z)) for x, z in pts]


def sponsons(level):
    g = Geo()
    a, b = SPONSON
    # Rounded ends: the front's quarter round, the rear's longer taper.
    ends = [(a, 0.15), (a + 3, 0.6), (a + 8, 0.9), (a + 16, 1.0), (b - 30, 1.0), (b - 15, 0.85), (b - 6, 0.6), (b, 0.15)]
    if level > 0:
        ends = [ends[i] for i in (0, 2, 4, 6, 7)]
    for sx in (-1, 1):
        rings = [sponson_ring(px, sx, s) for px, s in ends]
        g.add(mklib.tube(rings, BODY))
    return g


FRONT_WHEELS = side(757, 722)
REAR_WHEELS = side(985, 722)
WHEEL_R, WHEEL_X = 0.31, 1.65


def gear(level):
    g = Geo()
    n = 10 if level == 0 else 6
    y, z = FRONT_WHEELS
    _, belly = side(757, interp(BOTTOM, 757))
    for sx in (-1, 1):
        for dx in (-0.21, 0.21):
            g.add(mklib.wheel((sx * WHEEL_X + dx, y, WHEEL_R), WHEEL_R, 0.2, TYRE, n=n))
        g.add(frustum((sx * WHEEL_X, y, WHEEL_R), (sx * WHEEL_X, y + 0.05, belly + 0.1), 0.07, 0.08, DARK, n=4))
        if level == 0:
            g.add(frustum((sx * (WHEEL_X - 0.3), y, WHEEL_R), (sx * (WHEEL_X + 0.3), y, WHEEL_R), 0.05, 0.05, DARK, n=4))
    y, z = REAR_WHEELS
    _, belly = side(985, interp(BOTTOM, 985))
    for sx in (-1, 1):
        g.add(mklib.wheel((sx * WHEEL_X, y, WHEEL_R), WHEEL_R, 0.24, TYRE, n=n))
        g.add(frustum((sx * (WHEEL_X - 0.15), y, WHEEL_R), (sx * (WHEEL_X - 0.15), y + 0.1, belly + 0.15), 0.07, 0.08, DARK, n=4))
        if level == 0:
            # The drag strut back to the fuselage.
            g.add(frustum((sx * (WHEEL_X - 0.15), y, WHEEL_R + 0.1), (sx * 1.1, y + 0.6, belly + 0.15), 0.05, 0.05, DARK, n=4))
    return g


def portholes(g, level):
    """The cabin's round windows (painted on), the escape hatch's frame round the first."""
    n = 10 if level == 0 else 6
    for px, row in ((658, 642), (728, 645), (800, 650), (872, 652), (942, 655)):
        y, z = side(px, row)
        for sx in (-1, 1):
            x = sx * 1.365
            pts = [(x, y + 0.33 * math.cos(2 * math.pi * k / n), z + 0.33 * math.sin(2 * math.pi * k / n)) for k in range(n)]
            g.face(pts, WINDOW, (0, y, z))
    if level == 0:
        hatch = [side(*p) for p in ((641, 628), (689, 628), (689, 657), (641, 657))]
        for sx in (-1, 1):
            p = [(sx * 1.362, y, z) for y, z in hatch]
            for a, b in zip(p, p[1:] + p[:1]):
                g.line(a, b, SEAM)
        # The ramp's hinge and edges, the cabin door's outline.
        for sx in (-1, 1):
            ramp = [side(*p) for p in ((985, 700), (1095, 652))]
            g.line((sx * 1.36, *ramp[0]), (sx * 1.36, *ramp[1]), SEAM)


def details(level):
    g = Geo()
    if level < 2:
        # The synchronising shaft's fairing along the roof between the pylons.
        rings = []
        for px in (680, 935):
            y, zt = side(px, interp(TOP, px))
            rings.append(column_ring(y, zt - 0.1, zt + 0.16, 0.36, 0.30, 0.12, 2 if level == 0 else 1))
        g.add(mklib.tube(rings, DECK))
    if level == 0:
        # The cargo hook under the belly, the step under the cockpit.
        _, belly = side(822.5, interp(BOTTOM, 822.5))
        g.add(mklib.block(-0.08, 0.08, -0.15, 0.15, belly - 0.18, belly + 0.02, DARK))
        y, z = side(626, 690)
        g.add(mklib.block(0.9, 1.05, y - 0.1, y + 0.1, z - 0.45, z + 0.05, DARK))
        g.add(mklib.block(-1.05, -0.9, y - 0.1, y + 0.1, z - 0.45, z + 0.05, DARK))
        # The pitot tubes on the nose's top.
        for sx in (-1, 1):
            y, z = side(568, 636)
            g.add(frustum((sx * 0.2, y, z), (sx * 0.2, y + 0.45, z + 0.05), 0.02, 0.012, METAL, n=4))
    return g


SEGS = [(3, 4, 4, 5, 4), (2, 3, 3, 3, 2), (1, 2, 1, 2, 1)]


def body(level):
    g = Geo()
    cols = (COLUMNS, COLUMNS1, COLUMNS2)[level]
    g.add(fuselage(cols, SEGS[level]))
    # The front one's foot just under the roof: sunk deeper it would show through the cockpit's
    # roof, drawn from inside.
    g.add(pylon(FWD_PYLON, 0.95, 0.75, level, sink=0.06))
    g.add(pylon(AFT_PYLON, 1.10, 0.80, level))
    g.add(heads(level))
    g.add(engines(level))
    g.add(sponsons(level))
    g.add(details(level))
    portholes(g, level)
    if level < 2:
        g.add(gear(level))
    return g


# Lights in metres: navigation lights on the sponsons' front corners, beacons on the aft pylon
# behind the rotor and under the belly.
NAV = {RED: [(-SPONSON_W[1], side(690, 0)[0], side(0, 668)[1])], GREEN: [(SPONSON_W[1], side(690, 0)[0], side(0, 668)[1])]}
BEACONS = [(0.0, side(1080, 0)[0], side(0, 545)[1] + 0.08), (0.0, side(860, 0)[0], side(0, 706)[1] - 0.04)]


def root_object():
    root = bpy.data.objects.get(NAME)
    if root is None:
        root = bpy.data.objects.new(NAME, None)
        for col in bpy.data.objects["aircraft_025"].users_collection:
            col.objects.link(root)
    return root


def build():
    root = root_object()
    root["attr"] = 0x83
    for c in list(root.children):
        mklib.delete_tree(c)
    root["lod_dists"] = LOD_DISTS
    for level, dist in enumerate(LOD_DISTS):
        geo = body(level).scaled(K)
        lod = heli.mesh_with_lines(root, f"{NAME}_lod{level}", geo)
        lod["max_dist"] = dist
        # Three blades each, meshing in the overlap: the front rotor turns anticlockwise seen
        # from above (built upside down, see heli.flipped), the rear one clockwise.
        fwd = heli.flipped(heli.rotor(level, 3, ROTOR_R, ROTOR_CHORD, BLADE, METAL, clockwise=False, phase=math.pi / 2)).scaled(K)
        aft = heli.rotor(level, 3, ROTOR_R, ROTOR_CHORD, BLADE, METAL, clockwise=True, phase=math.pi / 6).scaled(K)
        parts = []
        for k, (rot, hub, rest) in enumerate(((fwd, FWD_HUB, (math.pi, 0.0, 0.0)), (aft, AFT_HUB, (0.0, 0.0, 0.0))), 1):
            part = heli.mesh_with_lines(lod, f"{lod.name}_part{k}", rot, v3(hub) * K)
            part.rotation_euler = rest
            part["state_slot"] = 17
            part["axis"] = 0
            parts.append(rot.transformed(lambda p, h=v3(hub) * K: p + h))
        lod["radius"] = heli.radius(geo, *parts)
        heli.lights(lod, 12, {c: [v3(p) * K for p in pts] for c, pts in NAV.items()})
        heli.lights(lod, 20, {RED: [v3(p) * K for p in BEACONS]})
        print(f"{lod.name}: {len(geo.faces) + sum(len(p.faces) for p in parts)} faces, radius {lod['radius']}")


if __name__ == "__main__":
    mklib.load_palette(os.path.join(ROOT, "assets", "palettes", "tod_central_europe.ron"))
    build()
    bpy.ops.wm.save_mainfile()
