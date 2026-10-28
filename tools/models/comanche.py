"""Builds the RAH-66 Comanche (`aircraft_046`) in aircraft.blend with three detail levels, and saves it:

    blender -b assets-src/blend/aircraft.blend --python-exit-code 1 --python tools/models/comanche.py
    tools/models/export.sh aircraft

A faceted stealth fuselage with a sharp chine all round: the pilot in front and the gunner
behind under a flat-panelled tandem canopy, a round, sharp-rimmed nose with the sensor
turret on top and the three-barrel gun under the chin, the engines buried under a raised deck
with the exhaust slots along the top of the tail boom, the five-blade main rotor with the Longbow
radome above the hub, and a shrouded fan in the tail under a fin with a T-tail stabiliser and end plates. Proportions are taken from
a three-view drawing (fuselage 13.2 m, rotor 11.9 m, hub 3.4 m above the ground, radome top
4.6 m, wheelbase 6.7 m, main wheel track 2.2 m). The origin is on the ground under the rotor
mast, so the wheels touch at z 0.

Each level carries its own rotors (`_part1` five-blade main rotor about the mast, built upside
down so it turns anticlockwise, `_part2` eight-blade fan in the shroud; both on slot 17),
navigation lights (`_s12v1`) and red beacons (`_s20v1`). Levels 0 and 1 have:
- the retractable gear (`_s5v1` two trailing-arm main wheels and a tail wheel, `_s5v0` the
  shut wheel well doors);
- a weapon bay door on each side, hinged at the chine (`_s8v0` shut, `_s8v1` half open,
  `_s8v2` swung up and out level), with the door's stores on slot 23 (left) and 24 (right):
  0 three Hellfires, 1 six Stingers, 2 none.
Dark grey. Built in metres and scaled to aircraft units (K per metre). Model space: x right,
y forward, z up.
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
from mklib import Geo, Ramp, frustum  # noqa: E402

NAME = "aircraft_046"
K = 16.38
# max_dist of each level (the game draws them 4x further): about 600 m and 2.2 km.
LOD_DISTS = [2500, 9000, 32769]

BODY = Ramp(0xE4, 0xEE)
DECK = Ramp(0xE5, 0xEE)
GLASS = Ramp(144, 145)
DARK = Ramp(0xEB, 0xEF)
METAL = Ramp(0xE8, 0xEC)
BLADE = Ramp(0xEB, 0xEF)
SENSOR = Ramp(0xE9, 0xEE)
STORE = Ramp(0xE2, 0xE8)
SEAM = 0xEE
RED, GREEN = 147, 151

# Fuselage stations from the nose: y, then the heights of the belly, the lower and upper edge
# of the flat side (the chine), the roof shoulder and the dorsal line, then the half widths at
# those heights. The lower facet (quads 0/8) slopes in from the chine to the belly; the bay doors
# are cut in it. Over the cockpits the upper edge is the canopy sill, well inside the chine, so the
# side (quads 1/7) is a shoulder sloping up to the narrow canopy. Behind the canopy the side stands
# upright, leaning out a little up to the upper edge, the widest point of the fuselage.
STATIONS = [
    # y      zb    zl    zu    zs    zt     wb    wl    wu    ws    wt
    (4.40, 0.90, 1.00, 1.20, 1.40, 1.46, 0.20, 0.40, 0.36, 0.20, 0.10),  # inside the round nose
    (4.15, 0.80, 1.06, 1.32, 1.56, 1.62, 0.28, 0.54, 0.50, 0.30, 0.15),
    (3.55, 0.72, 1.14, 1.80, 1.80, 1.80, 0.32, 0.66, 0.32, 0.32, 0.16),  # windshield base
    (3.25, 0.69, 1.19, 1.48, 2.05, 2.05, 0.33, 0.71, 0.66, 0.302, 0.151),  # quarter panes' front
    (2.48, 0.62, 1.31, 1.47, 2.51, 2.51, 0.37, 0.88, 0.70, 0.27, 0.14),  # pilot canopy bow
    (1.84, 0.62, 1.38, 1.60, 2.56, 2.56, 0.40, 0.95, 0.76, 0.28, 0.14),  # gunner canopy bow
    (0.82, 0.70, 1.37, 1.90, 2.60, 2.60, 0.48, 1.10, 0.78, 0.29, 0.15),  # canopy aft
    (0.55, 0.72, 1.37, 1.88, 2.80, 2.97, 0.48, 1.12, 1.17, 0.66, 0.44),  # engine deck
    (-1.35, 0.73, 1.37, 1.72, 2.80, 2.96, 0.50, 1.12, 1.16, 0.68, 0.44),
    (-1.70, 0.74, 1.37, 1.68, 2.56, 2.70, 0.50, 1.12, 1.14, 0.62, 0.38),
    (-3.05, 0.84, 1.35, 1.56, 1.90, 2.00, 0.40, 1.00, 0.96, 0.50, 0.26),
    (-3.70, 0.82, 1.38, 1.50, 1.84, 1.92, 0.28, 0.72, 0.70, 0.40, 0.20),  # tail boom
    (-5.25, 0.74, 1.30, 1.42, 1.66, 1.72, 0.20, 0.52, 0.50, 0.32, 0.15),
    (-6.15, 0.70, 1.22, 1.38, 1.64, 1.70, 0.18, 0.44, 0.42, 0.27, 0.13),
]
# Glazed panes: a block of loft segments (first, last; segment i between station i and i+1) and
# quads (first, last; 2..6 the canopy's sides, roof slopes and top strip), set flat in a frame,
# the bows between them painted. The canopy's roof is flat (shoulder and top at one height) and its
# sides lean in. The one-piece windshield starts narrow on the nose, the quarter panes beside it
# sweep back and out to the sill (the sill points of the first station sit on the windshield's
# corners), then the canopy widens at the bow into a side window each side and the roof over each
# seat.
GLAZED = [((2, 3), (3, 5)), ((2, 3), (2, 2)), ((2, 3), (6, 6))] + [
    ((i, i), q) for i in (4, 5) for q in ((2, 2), (3, 5), (6, 6))
]
# The canopy bows lean back towards their feet: the roof points of these stations (by y) sit this
# far forward of the sill.
LEAN = {2.48: 0.23, 1.84: 0.08, 0.82: 0.05}
FRAME = 0.06
# The windshield and the quarter panes beside it (the first three of GLAZED) have slim posts.
FRONT_FRAME = 0.035

MAST = (0.0, 0.0, 3.39)
ROTOR_R, ROTOR_CHORD = 5.95, 0.32
FAN = (0.0, -6.97, 1.40)
FAN_R, FAN_CHORD = 0.69, 0.15
# The fan's shroud: duct radius, outer half sizes (fore-aft, up-down), half thickness.
DUCT_R, SHROUD_Y, SHROUD_Z, SHROUD_T = 0.72, 1.04, 0.86, 0.27
# Radome above the hub: base and top heights and radii.
RADOME = (3.80, 4.58, 0.64, 0.34)
# Gear: main wheel axles (x, y, z) and radius, width; the tail wheel.
MAIN_WHEEL, MAIN_R, MAIN_W = (1.10, 1.30, 0.28), 0.28, 0.16
TAIL_WHEEL, TAIL_R = (0.0, -5.40, 0.18), 0.18
# Bay doors: hinge at the chine and the lower edge at the belly corner (x, z), fore and aft
# ends; the open angles of the variants (degrees, swung up and out).
BAY_HINGE, BAY_EDGE, BAY_Y = (1.12, 1.37), (0.49, 0.72), (0.70, -1.40)
BAY_ANGLES = (0, 70, 135)
# Tail: the fin's sections (leading edge y, trailing edge y, z, half thickness) from the top of
# the boom, over the shroud, to the tip; the stabiliser's leading edge, chords, half span and
# height; the end plates' bottom and top (leading edge y, trailing edge y, z, half span).
FIN = [(-5.25, -6.10, 1.70, 0.12), (-6.05, -7.70, 2.15, 0.16), (-7.98, -8.60, 3.50, 0.07)]
STAB_Y, STAB_ROOT_CHORD, STAB_TIP_CHORD, STAB_SPAN, STAB_Z = -7.25, 1.03, 1.00, 1.45, 3.40
# End plates canted in about 20 degrees at the top.
PLATE = ((-7.25, -7.85, 3.00, 1.58), (-7.30, -8.40, 3.80, 1.29))
# Sensor turret on the nose, gun turret and muzzle under the chin.
TURRET, MUZZLE = (0.0, 4.02, 0.60), (0.0, 4.75, 0.56)
# The round nose jutting out of the fuselage: a sharp rim with the underside curving back under it
# and a cone above it up to the sensor turret's base. Its centre (y), then (z, radius) from the
# bottom to the top.
DRUM_Y = 4.34
DRUM = [(0.83, 0.24), (0.87, 0.38), (0.97, 0.52), (1.12, 0.58), (1.48, 0.33)]


def interp(y):
    """The station at `y`, interpolated."""
    for s0, s1 in zip(STATIONS, STATIONS[1:]):
        if s1[0] <= y <= s0[0]:
            t = (s0[0] - y) / (s0[0] - s1[0])
            return [a + (b - a) * t for a, b in zip(s0, s1)]
    raise ValueError(y)


def slope_x(y, z):
    """Half width of the upper slope (quads 2/6) at (y, z)."""
    _, _, _, zu, zs, _, _, _, wu, ws, _ = interp(y)
    return wu + (ws - wu) * min(max((z - zu) / (zs - zu), 0.0), 1.0)


def side_outline(pts, g):
    heli.side_outline(STATIONS, pts, SEAM, g)


def upper_deck(level):
    """Mast, the radome on its stalk, exhaust slots along the boom and the engine intakes."""
    g = Geo()
    mx, my, mz = MAST
    n = 8 if level == 0 else 6
    g.add(frustum((mx, my, 2.98), (mx, my, mz - 0.08), 0.24, 0.18, METAL, n=n))
    z0, z1, r0, r1 = RADOME
    if level < 2:
        g.add(frustum((mx, my, mz + 0.10), (mx, my, z0), 0.11, 0.11, METAL, n=6))
    g.add(frustum((mx, my, z0), (mx, my, z1), r0, r1, BODY, n=10 if level == 0 else 6))
    if level == 0:
        for sx in (-1, 1):
            # The exhaust slots: dark strips on the roof slopes either side of the boom.
            for (y0, y1), w in (((-2.10, -3.40), 0.10), ((-3.60, -5.10), 0.07)):
                s0, s1 = interp(y0), interp(y1)
                pts = []
                for s, y in ((s0, y0), (s1, y1)):
                    zs, zt, ws, wt = s[4], s[5], s[9], s[10]
                    a = Vector((sx * (ws + (wt - ws) * 0.35), y, zs + (zt - zs) * 0.35 + 0.012))
                    b = Vector((sx * (ws + (wt - ws) * (0.35 + w * 4)), y, zs + (zt - zs) * (0.35 + w * 4) + 0.012))
                    pts.append((a, b))
                (a0, b0), (a1, b1) = pts
                g.face([a0, b0, b1, a1], DARK, (0, (y0 + y1) / 2, 1.0))
            # Intakes: long dark scoops on the shoulders behind the canopy, under the mast.
            scoop = [(0.55, 2.50), (-1.30, 2.68), (-1.55, 2.22), (0.30, 2.06)]
            pts = [(sx * (slope_x(y, z) + 0.012), y, z) for y, z in scoop]
            g.face(pts, DARK, (0, -0.7, 2.5))
    return g


def nose(level):
    """The round drum at the front, the sensor turret on top of the nose and the chin gun with
    its three barrels."""
    g = Geo()
    n = (16, 10, 8)[level]
    g.add(mklib.tube([mklib.ring((0.0, DRUM_Y, z), r, r, n, axis=(0, 0, 1)) for z, r in DRUM], BODY))
    g.add(mklib.block(-0.20, 0.20, 4.04, 4.64, 1.46, 1.71, SENSOR, taper=0.04, taper_y=0.06))
    if level == 0:
        g.face([(-0.13, 4.645, 1.52), (0.13, 4.645, 1.52), (0.13, 4.645, 1.62), (-0.13, 4.645, 1.62)], DARK, (0, 4.3, 1.57))
    x, y, z = TURRET
    n = 8 if level == 0 else 6
    g.add(frustum((x, y, z + 0.20), (x, y, z - 0.10), 0.22, 0.16, METAL, n=n))
    mx, my, mz = MUZZLE
    if level == 0:
        for k in range(3):
            a = math.pi / 2 + 2 * math.pi * k / 3
            dx, dz = 0.04 * math.cos(a), 0.04 * math.sin(a)
            g.add(frustum((x + dx, y + 0.12, z - 0.04 + dz), (mx + dx, my, mz + dz), 0.02, 0.02, DARK, n=4))
        g.add(frustum((x, y + 0.10, z - 0.04), (x, y + 0.60, z - 0.05), 0.10, 0.08, METAL, n=6))
    elif level == 1:
        g.add(frustum((x, y + 0.10, z - 0.04), (mx, my, mz), 0.06, 0.05, DARK, n=4))
    return g


def shroud(level):
    """The fan's shroud: duct, outer skin and the side faces, the stators and the fan hub."""
    g = Geo()
    cx, cy, cz = FAN
    n = 16 if level == 0 else 10
    t = SHROUD_T
    inner = [(math.cos(2 * math.pi * k / n), math.sin(2 * math.pi * k / n)) for k in range(n)]
    duct = lambda c, s, x: Vector((x, cy + c * DUCT_R, cz + s * DUCT_R))
    outer = lambda c, s, x: Vector((x, cy + c * SHROUD_Y, cz + s * SHROUD_Z))
    centre = Vector(FAN)
    for k in range(n):
        (c0, s0), (c1, s1) = inner[k], inner[(k + 1) % n]
        # Duct (faces the fan) and the outer skin.
        mid_in = Vector((0, cy + (c0 + c1) * DUCT_R * 0.6, cz + (s0 + s1) * DUCT_R * 0.6))
        g.face([duct(c0, s0, -t), duct(c1, s1, -t), duct(c1, s1, t), duct(c0, s0, t)], DARK if level == 0 else BODY, mid_in * 2 - centre)
        g.face([outer(c0, s0, -t * 0.8), outer(c1, s1, -t * 0.8), outer(c1, s1, t * 0.8), outer(c0, s0, t * 0.8)], BODY, centre)
        for sx in (-1, 1):
            g.face([duct(c0, s0, sx * t), duct(c1, s1, sx * t), outer(c1, s1, sx * t * 0.8), outer(c0, s0, sx * t * 0.8)], BODY,
                   centre - Vector((sx, 0, 0)))
    if level < 2:
        # Hub and stators (on the right, behind the fan) and the fan's hub fairing.
        g.add(frustum((-0.12, cy, cz), (t * 0.9, cy, cz), 0.16, 0.14, METAL, n=8 if level == 0 else 6))
        for k in range(6 if level == 0 else 3):
            a = math.pi / 6 + 2 * math.pi * k / 6
            d = Vector((0, math.cos(a), math.sin(a)))
            p0, p1 = centre + d * 0.14 + Vector((0.16, 0, 0)), centre + d * (DUCT_R + 0.01) + Vector((0.16, 0, 0))
            g.face2([p0 - Vector((0.03, 0, 0)), p1 - Vector((0.03, 0, 0)), p1 + Vector((0.03, 0, 0)), p0 + Vector((0.03, 0, 0))], DARK)
    return g


def tail(level):
    """The fin above the shroud and the T-tail stabiliser with its end plates."""

    def section(le, te, z, t):
        le, te = Vector((0, le, z)), Vector((0, te, z))
        a, b = le + (te - le) * 0.25, le + (te - le) * 0.7
        return [le, a + Vector((t, 0, 0)), b + Vector((t * 0.7, 0, 0)), te, b - Vector((t * 0.7, 0, 0)), a - Vector((t, 0, 0))]

    g = Geo()
    g.add(mklib.tube([section(*f) for f in (FIN if level < 2 else FIN[1:])], BODY))
    y, z, span = STAB_Y, STAB_Z, STAB_SPAN
    g.add(mklib.wing((0.0, y, z), STAB_ROOT_CHORD, (span, y + 0.05, z), STAB_TIP_CHORD, BODY, t=0.08))
    if level < 2:
        # End plates, swept and leaning in at the top.
        (le0, te0, z0, x0), (le1, te1, z1, x1) = PLATE
        for sx in (-1, 1):
            pts = [(sx * x0, le0, z0), (sx * x0, te0, z0), (sx * x1, te1, z1), (sx * x1, le1, z1)]
            g.add(mklib.extrude([(p[0] - 0.025, p[1], p[2]) for p in pts], (0.05, 0, 0), BODY))
    return g


def door_frame(sx, angle):
    """A bay door's hinge point, its unit vector from the hinge to the lower edge and its inner
    normal, swung `angle` degrees up and out (`sx` -1 left, 1 right)."""
    (hx, hz), (ex, ez) = BAY_HINGE, BAY_EDGE
    u = Vector((ex - hx, 0, ez - hz)).normalized()
    inward = Vector((u.z, 0, -u.x))  # u turned 90 degrees towards the inside (up and in)
    a = math.radians(angle)
    # Right side, seen from the front: turning u from down-and-in through straight down to
    # level outwards.
    rot = lambda v: Vector((v.x * math.cos(a) - v.z * math.sin(a), 0, v.x * math.sin(a) + v.z * math.cos(a)))
    u, inward = rot(u), rot(inward)
    hinge = Vector((hx, 0, hz))
    width = (Vector((ex, 0, ez)) - Vector((hx, 0, hz))).length
    m = lambda v: Vector((sx * v.x, v.y, v.z))
    return m(hinge), m(u), m(inward), width


def door(sx, angle):
    g = Geo()
    hinge, u, inward, width = door_frame(sx, angle)
    out = -inward * 0.012
    y0, y1 = BAY_Y
    pts = [hinge + Vector((0, y0, 0)), hinge + Vector((0, y1, 0)), hinge + u * width + Vector((0, y1, 0)), hinge + u * width + Vector((0, y0, 0))]
    if angle == 0:
        for a, b in zip(pts, pts[1:] + pts[:1]):
            g.line(a + out, b + out, SEAM)
        return g
    # The door (painted outside, the inside darker) and the dark bay behind it.
    g.face([p + out for p in pts], BODY, hinge + inward)
    g.face([p + out * 0.5 for p in pts], DARK, hinge - inward * 2)
    h0, u0, inward0, _ = door_frame(sx, 0)
    bay = [h0 + Vector((0, y0, 0)), h0 + Vector((0, y1, 0)), h0 + u0 * width + Vector((0, y1, 0)), h0 + u0 * width + Vector((0, y0, 0))]
    g.face([p - inward0 * 0.01 for p in bay], DARK, h0 + inward0)
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


def stinger(p, level):
    """A FIM-92 along y with its nose at `p`."""
    g = Geo()
    x, y, z = p
    g.add(frustum((x, y, z), (x, y - 0.12, z), 0.015, 0.035, STORE, n=4))
    g.add(frustum((x, y - 0.12, z), (x, y - 1.52, z), 0.035, 0.035, STORE, n=4))
    return g


def stores(sx, angle, kind, level):
    """The stores on a door: `kind` 0 three Hellfires, 1 six Stingers."""
    g = Geo()
    hinge, u, inward, width = door_frame(sx, angle)
    nose_y = BAY_Y[0] - 0.20
    if kind == 0:
        for s in (0.17, 0.45, 0.73):
            c = hinge + u * width * s + inward * 0.13
            g.add(hellfire((c.x, nose_y, c.z), level))
    else:
        for k in range(6):
            c = hinge + u * width * (0.12 + 0.152 * k) + inward * 0.07
            g.add(stinger((c.x, nose_y - 0.05, c.z), level))
    return g


def gear(level):
    """The gear down: the two trailing-arm main wheels and the tail wheel on its leg."""
    g = Geo()
    n = 8 if level == 0 else 6
    x, y, z = MAIN_WHEEL
    for sx in (-1, 1):
        axle = Vector((sx * x, y, z))
        g.add(mklib.wheel(tuple(axle), MAIN_R, MAIN_W, DARK, n=n))
        pivot = Vector((sx * 0.50, y + 0.55, 0.70))
        g.add(frustum(tuple(pivot), tuple(axle - Vector((sx * 0.09, 0, 0))), 0.06, 0.05, METAL, n=5))
        if level == 0:
            g.add(frustum((sx * 0.48, y - 0.20, 0.72), tuple(axle - Vector((sx * 0.09, -0.05, -0.10))), 0.04, 0.04, METAL, n=4))
    x, y, z = TAIL_WHEEL
    g.add(mklib.wheel((x, y, z), TAIL_R, 0.10, DARK, n=n))
    g.add(frustum((0, y + 0.45, 0.76), (0, y + 0.02, z + 0.10), 0.05, 0.04, METAL, n=5))
    if level == 0:
        for sx in (-1, 1):
            g.add(frustum((sx * 0.07, y + 0.08, z + 0.12), (sx * 0.07, y, z), 0.02, 0.02, METAL, n=4))
    return g


def gear_doors():
    """The shut wheel wells: seams on the belly."""
    g = Geo()
    x, y, _ = MAIN_WHEEL
    for sx in (-1, 1):
        s = interp(y)
        z = s[1] - 0.008
        pts = [(sx * 0.30, y + 0.75, z), (sx * 0.30, y - 0.40, z), (sx * (s[6] - 0.02), y - 0.40, z), (sx * (s[6] - 0.02), y + 0.75, z)]
        for a, b in zip(pts, pts[1:] + pts[:1]):
            g.line(a, b, SEAM)
    _, y, _ = TAIL_WHEEL
    z = interp(y)[1] - 0.008
    pts = [(-0.12, y + 0.65, z), (-0.12, y - 0.30, z), (0.12, y - 0.30, z), (0.12, y + 0.65, z)]
    for a, b in zip(pts, pts[1:] + pts[:1]):
        g.line(a, b, SEAM)
    return g


def inset(pts, d):
    """A flat convex polygon shrunk by `d` on every edge."""
    pts = [Vector(p) for p in pts]
    n = len(pts)
    c = sum(pts, Vector()) / n
    normal = mklib.normal(pts)
    lines = []
    for i in range(n):
        a, b = pts[i], pts[(i + 1) % n]
        e = (b - a).normalized()
        inward = normal.cross(e)
        if inward.dot(c - a) < 0:
            inward = -inward
        lines.append((a + inward * d, e))
    out = []
    for i in range(n):
        (p, e), (q, f) = lines[i - 1], lines[i]
        # p + t e = q + s f, solved in the least squares sense (the lines are coplanar).
        m00, m01, m11 = e.dot(e), -e.dot(f), f.dot(f)
        r = q - p
        b0, b1 = e.dot(r), -f.dot(r)
        det = m00 * m11 - m01 * m01
        t = (b0 * m11 - m01 * b1) / det if abs(det) > 1e-9 else 0.0
        out.append(p + e * t)
    return out


def corners(pts):
    """The outline without repeated points and points on a straight run."""
    out = []
    for p in pts:
        if not out or (p - out[-1]).length > 1e-4:
            out.append(p)
    if (out[0] - out[-1]).length <= 1e-4:
        out.pop()
    keep = []
    n = len(out)
    for i, p in enumerate(out):
        a, b = (p - out[i - 1]).normalized(), (out[(i + 1) % n] - p).normalized()
        if a.cross(b).length > 0.08:
            keep.append(p)
    return keep


def fuselage(stations):
    """The loft of heli.fuselage, with the glazed blocks of quads panes of glass set in frames."""
    g = Geo()
    rings = [ring(s) for s in stations]
    for s, r in zip(stations, rings):
        for p in r[3:7]:
            p.y += LEAN.get(s[0], 0.0)
    m = len(rings[0])
    glazed = {(i, k) for (i0, i1), (k0, k1) in GLAZED for i in range(i0, i1 + 1) for k in range(k0, k1 + 1)}
    for i in range(len(rings) - 1):
        a, b = rings[i], rings[i + 1]
        mid = Vector((0, (stations[i][0] + stations[i + 1][0]) / 2, (a[0].z + a[m // 2 - 1].z + b[0].z + b[m // 2 - 1].z) / 4))
        for k in range(m):
            if (i, k) not in glazed:
                k1 = (k + 1) % m
                g.face([a[k], a[k1], b[k1], b[k]], BODY, mid)
    for pane_no, ((i0, i1), (k0, k1)) in enumerate(GLAZED):
        outline = ([rings[i0][k] for k in range(k0, k1 + 2)] + [rings[i][k1 + 1] for i in range(i0 + 1, i1 + 2)]
                   + [rings[i1 + 1][k] for k in range(k1, k0 - 1, -1)] + [rings[i][k0] for i in range(i1, i0, -1)])
        outline = corners(outline)
        centre = Vector((0, sum(p.y for p in outline) / len(outline), 1.3))
        pane = inset(outline, FRONT_FRAME if pane_no < 3 else FRAME)
        n = len(outline)
        for j in range(n):
            g.face([outline[j], outline[(j + 1) % n], pane[(j + 1) % n], pane[j]], DARK, centre)
        g.face(pane, GLASS, centre)
    for r, other in ((rings[0], rings[1]), (rings[-1], rings[-2])):
        g.face(r, BODY, sum(other, Vector()) / m)
    return g


def details(g):
    """Canopy frames, access panel seams (level 0 only)."""
    # Avionics bays under the canopy, the aft side panel behind the bay door, the boom panels.
    side_outline([(3.60, 1.20), (2.80, 1.30), (2.80, 1.44), (3.60, 1.42)], g)
    side_outline([(2.50, 1.34), (1.30, 1.40), (1.30, 1.50), (2.50, 1.45)], g)
    for sx in (-1, 1):
        for y0, y1 in ((-1.48, -3.25),):
            pts = []
            for y in (y0, y1):
                s = interp(y)
                pts.append(Vector((sx * (s[7] + 0.01), y, s[2] - 0.02)))
                pts.append(Vector((sx * (s[6] + 0.03), y, s[1] + 0.05)))
            a0, b0, a1, b1 = pts
            for p, q in ((a0, a1), (a1, b1), (b1, b0), (b0, a0)):
                g.line(p, q, SEAM)


def body(level):
    g = Geo()
    if level == 0:
        g.add(fuselage(STATIONS))
        details(g)
    elif level == 1:
        st = [STATIONS[i] for i in (0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 12, 13)]
        g.add(fuselage(st))
    else:
        st = [STATIONS[i] for i in (0, 2, 4, 6, 8, 10, 13)]
        g.add(heli.fuselage(st, BODY, GLASS, False, {1: {1, 2, 3}, 2: {1, 2, 3}}))
    g.add(upper_deck(level))
    g.add(nose(level))
    g.add(shroud(level))
    g.add(tail(level))
    return g


# Lights in metres: navigation lights on the end plates, beacons behind the mast and under the
# belly.
NAV = {RED: [(-PLATE[0][3] - 0.06, -7.80, 3.20)], GREEN: [(PLATE[0][3] + 0.06, -7.80, 3.20)]}
BEACONS = [(0.0, -1.50, 2.94), (0.0, -2.20, 0.76)]


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
    geo = geo.scaled(K)
    if geo.faces:
        heli.mesh_with_lines(ob, f"{name}_body", geo)
    else:
        for c, segs in sorted(geo.lines.items()):
            ln = f"{name}_lines_pal_{c:03}"
            mklib.link_child(ob, ln, mklib.lines_mesh(ln, segs, c))
    return ob


def build():
    root = root_object()
    root["attr"] = 0x83
    levels = []
    for level, dist in enumerate(LOD_DISTS):
        # The main rotor turns anticlockwise seen from above: built upside down (see heli.flipped).
        main = heli.flipped(heli.rotor(level, 5, ROTOR_R, ROTOR_CHORD, BLADE, METAL, clockwise=False, phase=math.pi / 2))
        fan = heli.tail_rotor(level, 8 if level < 2 else 4, FAN_R, FAN_CHORD, BLADE, METAL)
        levels.append((dist, body(level), main, fan, (math.pi, 0.0, 0.0)))
    heli.build_levels(root, levels, K, MAST, FAN, (0.0, 0.0, 0.0), NAV, {RED: BEACONS})
    for level in (0, 1):
        lod = bpy.data.objects[f"{NAME}_lod{level}"]
        group(lod, [(5, 1)], gear(level))
        group(lod, [(5, 0)], gear_doors())
        for state, angle in enumerate(BAY_ANGLES):
            group(lod, [(8, state)], door(-1, angle).add(door(1, angle)))
            if state == 0:
                continue
            for sx, slot in ((-1, 23), (1, 24)):
                for kind in (0, 1):
                    group(lod, [(8, state), (slot, kind)], stores(sx, angle, kind, level))


mklib.load_palette(os.path.join(ROOT, "assets", "palettes", "tod_central_europe.ron"))
build()
bpy.ops.wm.save_mainfile()
