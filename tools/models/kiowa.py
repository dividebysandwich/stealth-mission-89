"""Builds the OH-58D Kiowa Warrior (`aircraft_048`) in aircraft.blend with three detail levels, and saves it:

    blender -b assets-src/blend/aircraft.blend --python-exit-code 1 --python tools/models/kiowa.py
    tools/models/export.sh aircraft

The Jet Ranger's rounded cabin for two side by side under a deep, wrap-round windshield with
the chin windows below it and a window in each of the four doors, the transmission and engine
fairing on its roof, a slim tapering tail boom with the stabiliser and its end plates halfway
down, the swept fin above and the ventral fin with the tail skid below its end, and skids on two
arched cross tubes. Above the four-blade main rotor the mast-mounted sight's ball sits on its
post; the wire strike cutters stand above and below the nose. Proportions are taken from a
three-view drawing (fuselage 9.7 m, rotor 10.67 m, hub 2.87 m above the ground, sight top 3.93 m,
skid track 2.3 m). The origin is on the ground under the rotor mast, so the skids touch at z 0.

Each level carries its own rotors (`_part1` four-blade main rotor about the mast, built upside
down so it turns anticlockwise, `_part2` two-blade tail rotor on the left of the boom; both on
slot 17), navigation lights (`_s12v1`) and red beacons (`_s20v1`). Levels 0 and 1 have the
universal weapons pylons, one each side, on slot 26 (left) and 27 (right): 0 none (the pylon is
left off), 1 a two-rail launcher with two Hellfires, 2 a launcher with two Stingers, 3 a .50 cal
machine gun pod; plus 4 for each missile fired (the outer one first). Dark grey. Built in metres and scaled to aircraft units (K per metre). Model
space: x right, y forward, z up.
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
from heli import ring  # noqa: E402
from mklib import Geo, Ramp, frustum, loft  # noqa: E402

NAME = "aircraft_048"
K = 16.38
# max_dist of each level (the game draws them 4x further): about 600 m and 2.2 km.
LOD_DISTS = [2500, 9000, 32769]

BODY = Ramp(0xE6, 0xED)
DECK = Ramp(0xE7, 0xEE)
GLASS = Ramp(144, 145)
DARK = Ramp(0xEB, 0xEF)
METAL = Ramp(0xE8, 0xEC)
BLADE = Ramp(0xEC, 0xEF)
SENSOR = Ramp(0xE6, 0xEC)
STORE = Ramp(0xE2, 0xE8)
SEAM = 0xEE
RED, GREEN = 147, 151

# Fuselage stations from the nose: y, then the heights of the belly, the lower and upper edge
# of the flat side, the roof shoulder and the top, then the half widths at those heights. The
# chin windows lie in the belly chamfers under the nose. The cabin's sides from the nose to the
# rear doors' aft edge (station 8) are glazed below the roof shoulder and covered by the frame
# (`FRAME`) round the windows; the front of the windshield and the roof slopes over the front
# seats are glass too. Behind the cabin the lower fuselage tapers into the tail boom.
STATIONS = [
    # y      zb    zl    zu    zs    zt     wb    wl    wu    ws    wt
    (2.94, 0.76, 0.84, 0.88, 0.91, 0.93, 0.08, 0.13, 0.13, 0.10, 0.05),  # nose tip
    (2.74, 0.58, 0.82, 0.94, 1.01, 1.04, 0.26, 0.42, 0.42, 0.32, 0.16),
    (2.42, 0.50, 0.79, 1.00, 1.09, 1.12, 0.38, 0.62, 0.64, 0.50, 0.26),  # windshield base
    (2.22, 0.47, 0.72, 1.02, 1.25, 1.30, 0.41, 0.67, 0.69, 0.55, 0.29),
    (2.00, 0.45, 0.66, 1.04, 1.41, 1.47, 0.44, 0.70, 0.72, 0.58, 0.31),
    (1.76, 0.45, 0.60, 1.07, 1.58, 1.64, 0.46, 0.72, 0.72, 0.59, 0.32),
    (1.44, 0.45, 0.57, 1.10, 1.71, 1.79, 0.46, 0.72, 0.72, 0.60, 0.32),  # windshield top
    (0.88, 0.45, 0.57, 1.12, 1.75, 1.81, 0.46, 0.72, 0.72, 0.60, 0.32),  # front door's aft edge
    (-0.06, 0.47, 0.59, 1.12, 1.75, 1.81, 0.46, 0.72, 0.72, 0.60, 0.32),  # rear door's aft edge
    (-0.30, 0.50, 0.61, 1.12, 1.75, 1.81, 0.46, 0.71, 0.71, 0.59, 0.32),  # cabin rear wall
    (-1.00, 0.61, 0.72, 1.20, 1.78, 1.86, 0.40, 0.58, 0.58, 0.48, 0.26),
    (-1.75, 0.78, 0.88, 1.32, 1.84, 1.93, 0.30, 0.44, 0.44, 0.36, 0.20),
    (-2.30, 1.08, 1.15, 1.50, 1.84, 1.91, 0.20, 0.28, 0.28, 0.23, 0.13),
    (-2.55, 1.34, 1.42, 1.74, 1.83, 1.87, 0.14, 0.24, 0.24, 0.17, 0.09),  # tail boom
    (-6.20, 1.57, 1.61, 1.74, 1.78, 1.80, 0.06, 0.11, 0.11, 0.08, 0.04),
]
# Glazed loft quads by segment, under the frame: the belly chamfers round the chin windows, the
# flat and upper sides round the windshield's sides, the upper sides round the doors' windows;
# the windshield's front (roof slopes and top) and the roof windows over the front seats.
CHIN, SHIELD, DOOR = {0, 8}, {1, 2, 3, 4, 5, 6, 7}, {2, 6}
GLAZED = {0: CHIN, 1: CHIN, 2: CHIN | SHIELD, 3: CHIN | SHIELD, 4: SHIELD, 5: SHIELD, 6: DOOR | {3, 5}, 7: DOOR}
# The glazed sides run from the nose tip to the rear doors' aft edge.
SIDE_Y = (STATIONS[0][0], STATIONS[8][0])
# The windows as seen from the side, (y, z) outlines from the drawing: the chin window, the
# windshield's side (its front and top edges beyond the skin: it runs up to the roof), the front
# door's window behind the leaning windshield post, the rear door's window.
WINDOWS = [
    [(2.98, 0.85), (2.54, 0.80), (2.19, 0.78), (2.05, 0.66), (2.09, 0.47), (2.31, 0.52), (2.54, 0.59), (2.89, 0.73)],
    [(2.44, 1.12), (2.44, 2.40), (1.47, 2.40), (1.47, 1.74), (1.76, 0.95), (2.31, 0.99)],
    [(1.32, 1.66), (0.94, 1.66), (0.96, 1.22), (1.48, 1.11)],
    [(0.78, 1.57), (-0.05, 1.57), (-0.04, 1.18), (0.78, 1.18)],
]
# Doors' outlines (seams): the front door behind the windshield post, the rear door.
DOORS = [
    [(1.49, 1.76), (0.88, 1.76), (0.88, 0.60), (1.86, 0.60), (1.78, 0.90)],
    [(0.86, 1.76), (-0.06, 1.76), (-0.06, 0.62), (0.86, 0.60)],
]
# Coarser levels: the stations kept, and their glazing (level 1 keeps the first 12 stations).
LEVEL1 = (0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 13, 14)
LEVEL2 = (0, 2, 4, 6, 8, 10, 12, 14)
GLAZED2 = {1: {1, 2, 3}, 2: {1, 2, 3}, 3: {1}}

# The transmission and engine fairing on the cabin roof: its front sloping up from behind the
# windshield to the top, which runs back past the mast rising a little and narrowing, its end
# curving down into the tail boom. Boxy sections with near vertical sides and a wide, flat top:
# (y, bottom, top, half width).
FAIRING = [
    (1.00, 1.74, 1.80, 0.18),
    (0.90, 1.74, 1.95, 0.26),
    (0.76, 1.74, 2.14, 0.31),
    (0.64, 1.74, 2.30, 0.33),
    (0.09, 1.74, 2.33, 0.33),
    (-0.40, 1.74, 2.41, 0.32),
    (-1.40, 1.75, 2.48, 0.28),
    (-2.10, 1.77, 2.49, 0.22),
    (-2.20, 1.79, 2.28, 0.19),
    (-2.30, 1.80, 2.10, 0.15),
    (-2.42, 1.82, 1.92, 0.10),
]

MAST = (0.0, 0.0, 2.87)
ROTOR_R, ROTOR_CHORD = 5.33, 0.24
TAIL_HUB = (-0.30, -6.28, 1.72)
TAIL_R, TAIL_CHORD = 0.83, 0.15
# The mast-mounted sight: the ball's centre and radius, on a post from the hub.
SIGHT, SIGHT_R = (0.0, 0.04, 3.62), 0.31
# Skids: half track, height of the tube, fore and aft ends; the cross tubes.
SKID_X, SKID_Z, SKID_Y = 1.15, 0.05, (1.62, -0.66)
CROSS_TUBES = (1.13, -0.35)
# Horizontal stabiliser: leading edge, chord, half span, height.
STAB_Y, STAB_CHORD, STAB_SPAN, STAB_Z = -3.89, 0.47, 0.89, 1.60
# The universal weapons pylon: the arm's root and tip x, its leading edge and chord, height; the
# launcher's centre x under the tip and the height of the stores' centre lines.
ARM_X, ARM_Y, ARM_CHORD, ARM_Z = (0.66, 1.36), 0.36, 0.60, 0.86
STORE_X, STORE_Z = 1.36, 0.46


def side_x(y, z):
    return heli.side_x(STATIONS, y, z)


def side_outline(pts, g):
    heli.side_outline(STATIONS, pts, SEAM, g)


def fairing_section(y, z0, z1, w):
    """A fairing section: the sides leaning in a little from the foot, the top's corners rounded."""
    h = z1 - z0
    r = min(0.06, h * 0.3, w * 0.3)
    right = [(w * 0.9, z0), (w, z0 + h * 0.3), (w, z1 - r), (w - r * 0.3, z1 - r * 0.3), (w - r, z1)]
    pts = right + [(-x, z) for x, z in reversed(right)]
    return [Vector((x, y, z)) for x, z in pts]


def fairing_x(y, z):
    """Half width of the fairing's side at (y, z)."""
    for s0, s1 in zip(FAIRING, FAIRING[1:]):
        if s1[0] <= y <= s0[0]:
            t = (s0[0] - y) / (s0[0] - s1[0])
            return s0[3] + (s1[3] - s0[3]) * t
    raise ValueError(y)


def fairing(level):
    """The transmission and engine fairing with the grille on its front sides, the louvres along
    the engine and the exhaust on top."""
    g = Geo()
    st = FAIRING if level < 2 else [FAIRING[i] for i in (0, 3, 6, 7, 10)]
    g.add(mklib.tube([fairing_section(*f) for f in st], DECK))
    if level == 0:
        for sx in (-1, 1):
            # The transmission's grille, the louvres along the engine.
            panels = [(0.06, -0.33, 1.86, 2.24)] + [(y + 0.05, y - 0.05, 1.88, 2.16) for y in (-0.52, -0.94, -1.39)]
            for y0, y1, z0, z1 in panels:
                pts = [(sx * (fairing_x(y, 0) + 0.01), y, z) for y, z in ((y0, z0), (y1, z0), (y1, z1), (y0, z1))]
                g.face(pts, DARK, (0, (y0 + y1) / 2, 2.05))
    if level < 2:
        # Exhaust on top at the aft end, its mouth dark.
        g.add(frustum((0, -1.92, 2.36), (0, -1.96, 2.46), 0.12, 0.12, METAL, n=8 if level == 0 else 6))
        if level == 0:
            g.add(frustum((0, -1.96, 2.46), (0, -1.965, 2.47), 0.09, 0.09, DARK, n=8))
    return g


def mast(level):
    """The mast from the fairing to the hub, the sight's post above the hub and the ball."""
    g = Geo()
    mx, my, mz = MAST
    n = 8 if level == 0 else 6
    g.add(frustum((mx, my, 2.30), (mx, my, mz - 0.10), 0.17, 0.12, METAL, n=n))
    sx_, sy, sz = SIGHT
    g.add(frustum((sx_, sy, mz + 0.08), (sx_, sy, sz - SIGHT_R * 0.8), 0.10 if level < 2 else 0.08, 0.07, METAL, n=6))
    rings = []
    m = (14, 10, 8)[level]
    for k in range(1, m // 2 + 1):
        a = math.pi * k / (m // 2 + 1) - math.pi / 2
        rings.append(mklib.ring((sx_, sy, sz + SIGHT_R * math.sin(a)), SIGHT_R * math.cos(a), SIGHT_R * math.cos(a), m, axis=(0, 0, 1)))
    g.add(mklib.tube(rings, SENSOR))
    if level < 2:
        # The sensor windows: two dark lenses side by side on the front, the laser below.
        for x, z, r in ((-0.12, 0.05, 0.10), (0.12, 0.05, 0.10), (0.0, -0.13, 0.05)) if level == 0 else ((0.0, 0.03, 0.17),):
            y = sy + math.sqrt(max(SIGHT_R ** 2 - x * x - z * z, 0.0)) + 0.01
            g.add(frustum((x, y - 0.02, sz + z), (x, y, sz + z), r, r, DARK, n=8 if level == 0 else 6))
    return g


def gear(level):
    """Skids on two arched cross tubes, turned up at the front."""
    g = Geo()
    n = 6 if level == 0 else 4
    y0, y1 = SKID_Y
    for sx in (-1, 1):
        x = sx * SKID_X
        g.add(frustum((x, y1, SKID_Z), (x, y0, SKID_Z), 0.045, 0.045, METAL, n=n))
        if level == 0:
            g.add(frustum((x, y0, SKID_Z), (x, y0 + 0.24, SKID_Z + 0.20), 0.045, 0.045, METAL, n=n))
            g.add(frustum((x, y1, SKID_Z), (x, y1 - 0.08, SKID_Z + 0.04), 0.045, 0.04, METAL, n=n))
        for y in CROSS_TUBES:
            g.add(frustum((sx * 0.40, y, 0.50), (sx * 0.95, y, 0.36), 0.045, 0.045, METAL, n=n))
            g.add(frustum((sx * 0.95, y, 0.36), (x, y, SKID_Z), 0.045, 0.045, METAL, n=n))
    return g


def tail(level):
    """The stabiliser with its end plates, the fin above the boom's end and the ventral fin with
    the tail skid below it, and the tail rotor's gearbox on the left."""

    def section(le, te, z, t):
        le, te = Vector((0, le, z)), Vector((0, te, z))
        a, b = le + (te - le) * 0.25, le + (te - le) * 0.7
        return [le, a + Vector((t, 0, 0)), b + Vector((t * 0.7, 0, 0)), te, b - Vector((t * 0.7, 0, 0)), a - Vector((t, 0, 0))]

    g = Geo()
    g.add(mklib.tube([section(-5.66, -6.30, 1.74, 0.07), section(-6.42, -6.78, 2.92, 0.04)], BODY))
    g.add(mklib.wing((0.0, STAB_Y, STAB_Z), STAB_CHORD, (STAB_SPAN, STAB_Y - 0.03, STAB_Z), STAB_CHORD * 0.95, BODY, t=0.12))
    if level < 2:
        g.add(mklib.tube([section(-5.84, -6.32, 1.62, 0.05), section(-6.04, -6.40, 0.72, 0.035)], BODY))
        hx, hy, hz = TAIL_HUB
        g.add(mklib.block(hx + 0.04, -0.04, hy - 0.16, hy + 0.20, hz - 0.12, hz + 0.12, BODY, taper=0.02 if level == 0 else 0.0))
        # End plates on the stabiliser's tips.
        for sx in (-1, 1):
            x = sx * (STAB_SPAN + 0.02)
            g.add(mklib.block(x - 0.02, x + 0.02, STAB_Y - 0.52, STAB_Y - 0.02, STAB_Z - 0.12, STAB_Z + 0.16, BODY))
    if level == 0:
        # The tail skid from the ventral fin's foot.
        g.add(frustum((0, -6.28, 0.78), (0, -7.20, 0.52), 0.03, 0.02, METAL, n=4))
    return g


def cutters(level):
    """The wire strike cutters: the upper one standing up and forward from the roof's front, the
    lower one under the chin; the deflector strip down the windshield's centre post; the pitot."""
    g = Geo()
    blade = lambda p0, p1, w, t: mklib.hexa(
        [(-t, p0[0], p0[1]), (t, p0[0], p0[1]), (t, p1[0], p1[1]), (-t, p1[0], p1[1]),
         (-t, p0[0] + w[0], p0[1] + w[1]), (t, p0[0] + w[0], p0[1] + w[1]), (t, p1[0] + w[0], p1[1] + w[1]), (-t, p1[0] + w[0], p1[1] + w[1])],
        METAL)
    g.add(blade((1.30, 1.80), (2.12, 2.42), (-0.12, -0.09), 0.025))
    g.add(blade((2.06, 0.50), (2.52, 0.22), (-0.06, 0.06), 0.02))
    if level == 0:
        g.add(frustum((0, 2.95, 0.84), (0, 3.14, 0.84), 0.02, 0.012, METAL, n=4))
        # The antenna mast on the roof's front fairing.
        g.add(frustum((0, 1.24, 1.82), (0, 1.24, 2.50), 0.02, 0.015, METAL, n=4))
        g.add(frustum((0, 1.24, 2.50), (0, 0.80, 2.52), 0.015, 0.015, METAL, n=4))
    return g


def station_at(y):
    """The station at `y`, interpolated."""
    for s0, s1 in zip(STATIONS, STATIONS[1:]):
        if s1[0] <= y <= s0[0]:
            t = (s0[0] - y) / (s0[0] - s1[0])
            return [a + (b - a) * t for a, b in zip(s0, s1)]
    raise ValueError(y)


def skin_x(y, z):
    """Half width of the loft's skin at (y, z) on the sides, from the belly corner up to the top."""
    _, zb, zl, zu, zs, zt, wb, wl, wu, ws, wt = station_at(y)
    prof = [(zb, wb), (zl, wl), (zu, wu), (zs, ws), (zt, wt)]
    if z <= zb:
        return wb
    for (z0, w0), (z1, w1) in zip(prof, prof[1:]):
        if z <= z1:
            return w0 + (w1 - w0) * ((z - z0) / (z1 - z0) if z1 > z0 else 1.0)
    return wt


def on_skin(sx, y, z, lift=0.02):
    return Vector((sx * (skin_x(y, z) + lift), y, z))


def crossings(poly, y):
    """The edges of a closed (y, z) outline crossing the vertical line at `y`."""
    out = []
    for a, b in zip(poly, poly[1:] + poly[:1]):
        if min(a[0], b[0]) < y < max(a[0], b[0]):
            out.append((a, b))
    return out


def edge_z(e, y):
    (y0, z0), (y1, z1) = e
    return z0 + (z1 - z0) * (y - y0) / (y1 - y0)


def frame(level):
    """The cabin's sides round the windows: body panels just outside the glazed loft quads, from
    the belly corner to the roof shoulder, cut round the window outlines; strips the width of
    the drawing's vertical lines, each split up and down to follow the skin's curve."""
    g = Geo()
    y0, y1 = SIDE_Y
    cuts = {y0, y1} | {p[0] for w in WINDOWS for p in w if y1 < p[0] < y0} | {st[0] for st in STATIONS if y1 < st[0] < y0}
    step = 0.12 if level == 0 else 0.25
    ys = sorted(cuts, reverse=True)
    stops = []
    for a, b in zip(ys, ys[1:]):
        n = max(1, math.ceil((a - b) / step))
        stops += [a + (b - a) * k / n for k in range(n)]
    stops.append(ys[-1])
    for ya, yb in zip(stops, stops[1:]):
        mid = (ya + yb) / 2
        edges = sorted((e for w in WINDOWS for e in crossings(w, mid)), key=lambda e: edge_z(e, mid))
        # From the lowest glazed quad's foot (the belly corner by the chin, the flat side's foot
        # by the windshield, its top by the doors) to the roof shoulder.
        foot = 1 if mid > STATIONS[4][0] else 2 if mid > STATIONS[6][0] else 3
        lo = lambda y: station_at(y)[foot]
        hi = lambda y: station_at(y)[4]
        # Bottom and top of each body piece at both ends: the belly corner, the window edges, the
        # roof shoulder.
        bounds = [lo] + [(lambda e: lambda y: edge_z(e, y))(e) for e in edges] + [hi]
        for k in range(0, len(bounds), 2):
            fb, ft = bounds[k], bounds[k + 1]
            za0, za1 = min(max(fb(ya), lo(ya)), hi(ya)), min(max(ft(ya), lo(ya)), hi(ya))
            zb0, zb1 = min(max(fb(yb), lo(yb)), hi(yb)), min(max(ft(yb), lo(yb)), hi(yb))
            if za1 - za0 < 0.005 and zb1 - zb0 < 0.005:
                continue
            m = max(1, math.ceil(max(za1 - za0, zb1 - zb0) / (0.10 if level == 0 else 0.2)))
            for j in range(m):
                t0, t1 = j / m, (j + 1) / m
                pa0, pa1 = za0 + (za1 - za0) * t0, za0 + (za1 - za0) * t1
                pb0, pb1 = zb0 + (zb1 - zb0) * t0, zb0 + (zb1 - zb0) * t1
                for sx in (-1, 1):
                    pts = [on_skin(sx, ya, pa0), on_skin(sx, yb, pb0), on_skin(sx, yb, pb1), on_skin(sx, ya, pa1)]
                    uniq = [p for i, p in enumerate(pts) if (p - pts[i - 1]).length > 1e-5]
                    if len(uniq) >= 3:
                        g.face(uniq, BODY, Vector((0, mid, (pa0 + pb1) / 2)))
    # The windshield's centre post down the front, from the nose to the roof.
    for k in range(4):
        s0, s1 = STATIONS[2 + k], STATIONS[3 + k]
        w = 0.035
        pts = [(-w, s0[0], s0[5] + 0.012), (w, s0[0], s0[5] + 0.012), (w, s1[0], s1[5] + 0.012), (-w, s1[0], s1[5] + 0.012)]
        g.face(pts, BODY, (0, (s0[0] + s1[0]) / 2, 1.0))
    return g


def details(g):
    """Door and panel seams (level 0 only)."""
    for door in DOORS:
        for sx in (-1, 1):
            for (ya, za), (yb, zb) in zip(door, door[1:] + door[:1]):
                n = max(1, math.ceil(max(abs(ya - yb), abs(za - zb)) / 0.1))
                pts = [on_skin(sx, ya + (yb - ya) * k / n, za + (zb - za) * k / n, 0.016) for k in range(n + 1)]
                for p, q in zip(pts, pts[1:]):
                    g.line(p, q, SEAM)
    # Access panels on the lower fuselage behind the cabin.
    side_outline([(-0.45, 0.80), (-1.20, 0.92), (-1.20, 1.18), (-0.45, 1.12)], g)


def body(level):
    g = Geo()
    if level == 0:
        g.add(heli.fuselage(STATIONS, BODY, GLASS, True, GLAZED))
        g.add(frame(level))
        details(g)
    elif level == 1:
        g.add(heli.fuselage([STATIONS[i] for i in LEVEL1], BODY, GLASS, True, GLAZED))
        g.add(frame(level))
    else:
        g.add(heli.fuselage([STATIONS[i] for i in LEVEL2], BODY, GLASS, False, GLAZED2))
    g.add(fairing(level))
    g.add(mast(level))
    g.add(tail(level))
    g.add(cutters(level))
    if level < 2:
        g.add(gear(level))
    return g


def hellfire(p, level):
    """An AGM-114 along y with its nose at `p`."""
    g = Geo()
    n = 6 if level == 0 else 4
    x, y, z = p
    g.add(frustum((x, y, z), (x, y - 0.18, z), 0.03, 0.089, STORE, n=n))
    g.add(frustum((x, y - 0.18, z), (x, y - 1.63, z), 0.089, 0.089, STORE, n=n))
    if level == 0:
        for a in (45, 135, 225, 315):
            d = Vector((math.cos(math.radians(a)), 0, math.sin(math.radians(a))))
            base = Vector(p)
            g.face2([base + Vector((0, -1.25, 0)) + d * 0.08, base + Vector((0, -1.60, 0)) + d * 0.08, base + Vector((0, -1.58, 0)) + d * 0.17], METAL)
    return g


def stinger_tube(p, level):
    """A Stinger's launch tube along y with its front at `p`: the seeker's dark cover in front."""
    g = Geo()
    x, y, z = p
    n = 6 if level == 0 else 4
    g.add(frustum((x, y, z), (x, y - 1.52, z), 0.06, 0.06, STORE, n=n))
    if level == 0:
        g.add(frustum((x, y + 0.01, z), (x, y, z), 0.05, 0.05, DARK, n=n))
    return g


def gun_pod(x, level):
    """The M3P: the .50 cal machine gun in a pod with its ammunition box, the barrel and its
    flash suppressor out in front."""
    g = Geo()
    n = 8 if level == 0 else 6
    z = STORE_Z + 0.06
    g.add(loft([(0.50, z, 0.08, 0.10), (0.38, z, 0.15, 0.18), (-0.55, z, 0.15, 0.18), (-0.80, z, 0.10, 0.10)], STORE, n=n, x=x))
    g.add(frustum((x, 0.50, z - 0.02), (x, 1.30, z - 0.02), 0.03, 0.03, DARK, n=6 if level == 0 else 4))
    if level == 0:
        g.add(frustum((x, 1.30, z - 0.02), (x, 1.45, z - 0.02), 0.045, 0.045, DARK, n=6))
        # The spent case chute under the pod.
        g.add(mklib.block(x - 0.05, x + 0.05, -0.20, 0.05, z - 0.24, z - 0.16, DARK))
    return g


def pylon(sx, kind, level, left=2):
    """The universal weapons pylon on one side (`sx` -1 left, 1 right) with its stores: `kind`
    1 Hellfires, 2 Stingers (`left` of the two, the inner one last), 3 the gun pod."""
    g = Geo()
    x0, x1 = ARM_X
    y = ARM_Y
    # The arm: an airfoil out from the cabin floor, the hanger down from its tip.
    g.add(mklib.wing((sx * x0, y, ARM_Z), ARM_CHORD, (sx * x1, y - 0.02, ARM_Z), ARM_CHORD * 0.9, BODY, t=0.14, mirror=False))
    x = sx * STORE_X
    if kind == 3:
        g.add(mklib.block(x - 0.04, x + 0.04, y - 0.52, y - 0.08, STORE_Z + 0.22, ARM_Z, METAL))
        g.add(gun_pod(x, level))
        return g
    g.add(mklib.block(x - 0.04, x + 0.04, y - 0.50, y - 0.10, STORE_Z + 0.20, ARM_Z, METAL))
    # The launcher: a beam across under the hanger, a rail down to each store.
    xs = (x - sx * 0.18, x + sx * 0.18) if kind == 1 else (x - sx * 0.11, x + sx * 0.11)
    top = STORE_Z + 0.20
    g.add(mklib.block(min(xs) - 0.05, max(xs) + 0.05, y - 0.80 if kind == 1 else y - 0.60, y + 0.20, top - 0.06, top, METAL))
    for k, sx_ in enumerate(xs):
        if k >= left:
            continue
        if kind == 1:
            g.add(hellfire((sx_, 1.14, STORE_Z), level))
            if level == 0:
                g.add(mklib.block(sx_ - 0.02, sx_ + 0.02, -0.30, 0.90, STORE_Z + 0.08, top - 0.06, METAL))
        else:
            g.add(stinger_tube((sx_, 0.90, STORE_Z + 0.06), level))
    return g


# Lights in metres: navigation lights on the stabiliser's end plates, beacons on the fairing's
# aft end and under the belly.
NAV = {RED: [(-STAB_SPAN - 0.05, STAB_Y - 0.25, STAB_Z + 0.06)], GREEN: [(STAB_SPAN + 0.05, STAB_Y - 0.25, STAB_Z + 0.06)]}
BEACONS = [(0.0, -1.40, 2.50), (0.0, -0.20, 0.46)]


def root_object():
    root = bpy.data.objects.get(NAME)
    if root is None:
        root = bpy.data.objects.new(NAME, None)
        for col in bpy.data.objects["aircraft_025"].users_collection:
            col.objects.link(root)
    return root


def group(lod, conds, geo):
    """A part of `lod` shown while every (slot, value) of `conds` holds."""
    name = lod.name + "".join(f"_s{s}v{v}" for s, v in conds)
    ob = bpy.data.objects.new(name, None)
    ob["conds"] = [list(c) for c in conds]
    for col in lod.users_collection:
        col.objects.link(ob)
    ob.parent = lod
    heli.mesh_with_lines(ob, f"{name}_body", geo.scaled(K))
    return ob


def build():
    root = root_object()
    root["attr"] = 0x83
    levels = []
    for level, dist in enumerate(LOD_DISTS):
        # The main rotor turns anticlockwise seen from above: built upside down (see heli.flipped).
        main = heli.flipped(heli.rotor(level, 4, ROTOR_R, ROTOR_CHORD, BLADE, METAL, clockwise=False, phase=math.pi / 4))
        tail_rot = heli.tail_rotor(level, 2, TAIL_R, TAIL_CHORD, BLADE, METAL)
        levels.append((dist, body(level), main, tail_rot, (math.pi, 0.0, 0.0)))
    heli.build_levels(root, levels, K, MAST, TAIL_HUB, (0.0, 0.0, 0.0), NAV, {RED: BEACONS})
    for level in (0, 1):
        lod = bpy.data.objects[f"{NAME}_lod{level}"]
        for sx, slot in ((-1, 26), (1, 27)):
            group(lod, [(slot, 3)], pylon(sx, 3, level))
            for kind in (1, 2):
                for fired in range(3):
                    group(lod, [(slot, kind + 4 * fired)], pylon(sx, kind, level, 2 - fired))


mklib.load_palette(os.path.join(ROOT, "assets", "palettes", "tod_central_europe.ron"))
build()
bpy.ops.wm.save_mainfile()
