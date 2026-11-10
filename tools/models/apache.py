"""Builds the AH-64D Apache (`aircraft_077`) in aircraft.blend with three detail levels, and saves it:

    blender -b assets-src/blend/aircraft.blend --python-exit-code 1 --python tools/models/apache.py
    tools/models/export.sh aircraft

The narrow tandem fuselage under the flat-plate canopy (the gunner in front, low, the pilot behind
and higher), the wide avionics bays along the lower sides, the TADS turret on the nose with the
PNVS above it, the M230 chin gun under the gunner, the engine nacelles on the shoulders behind the
cockpit with their exhaust suppressors, the stub wings with two pylons each and the wingtip
Stinger launchers, the deep tail boom with the swept fin, the stabilator at its foot and the
scissor tail rotor on the left, the trailing arm main gear and the tail wheel. Proportions are
taken from a three-view drawing (41.2 px per metre; fuselage 15.06 m, rotor 14.63 m, wingspan
5.23 m). The drawing's side view shows the fuselage level; on its wheels the Apache sits nose up
(`TILT`), and so does the model: it is built in the drawing's frame and turned about the main
wheels' contact. The origin is on the ground under the rotor hub.

The canopy's glass is cut into the skin exactly along the drawing's window outlines (`WINDOWS`):
each window is the part of the skin whose side view (and, for the roof panes, front view) falls
inside the drawn shapes; the frames are what is left.

Each level carries its rotors (`_part1` four-blade main rotor about the hub, built upside down so
it turns anticlockwise seen from above, `_part2` the scissor tail rotor; both on slot 17),
navigation lights (`_s12v1`) and red beacons (`_s20v1`). Levels 0 and 1 also have:
- the chin gun turning in azimuth (`_part3`, slot 30) and the gun in elevation inside it
  (`_part4`, slot 31), both about the turret's pivot, square to the fuselage;
- the TADS turret turning in azimuth (`_part5`, slot 9);
- the stores on the four wing pylons, slots 21 (left outer), 22 (left inner), 23 (right inner),
  24 (right outer): 0 nothing, 1 an empty M299 launcher, 2..5 the launcher with 1..4 Hellfires;
- the wingtip Stinger launchers, slots 26 (left) and 27 (right): 0 nothing, 1 the empty
  launcher, 2..3 the launcher with 1..2 Stingers.
Dark olive drab. Built in metres and scaled to aircraft units (K per metre). Model space: x right,
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
from mklib import Geo, Ramp, area, cut, frustum, v3  # noqa: E402

NAME = "aircraft_077"
K = 16.38
# max_dist of each level (the game draws them 4x further): about 600 m and 2.2 km.
LOD_DISTS = [2500, 9000, 32769]

BODY = Ramp(0xAB, 0xAF)
DECK = Ramp(0xAC, 0xAF)
GLASS = Ramp(144, 145)
DARK = Ramp(0xEB, 0xEF)
METAL = Ramp(0xE9, 0xEE)
BLADE = Ramp(0xEC, 0xEF)
SENSOR = Ramp(0xEA, 0xEF)
LENS = Ramp(157, 159)
STORE = Ramp(0xE4, 0xEA)
TYRE = 0xEF
SEAM = 0xAF
RED, GREEN = 147, 151

# --- The drawing ----------------------------------------------------------------------------------
PX = 41.2
MAST_PX = 307.0
# The main wheels' contact (the side view's row 245) and the tail wheel's (row 207, 0.92 m up).
GROUND_ROW = 245.0


def side(px, row):
    """A side view drawing point (column, row) as (y, z) metres in the drawing's frame."""
    return ((MAST_PX - px) / PX, (GROUND_ROW - row) / PX)


def sy(px):
    return (MAST_PX - px) / PX


def sz(row):
    return (GROUND_ROW - row) / PX


def interp(table, t):
    """Piecewise linear through (t, value) pairs sorted by t; clamped at the ends."""
    if t <= table[0][0]:
        return table[0][1]
    for (t0, v0), (t1, v1) in zip(table, table[1:]):
        if t <= t1:
            return v0 + (v1 - v0) * (t - t0) / (t1 - t0)
    return table[-1][1]


# On its wheels the fuselage is pitched up by the tail wheel's height over the wheelbase; the
# model turns about the main wheels' contact, then moves the hub over the origin.
MAIN_WHEEL = side(272.5, 245)
TAIL_WHEEL = side(705, 207)
TILT = math.atan2(TAIL_WHEEL[1] - MAIN_WHEEL[1], MAIN_WHEEL[0] - TAIL_WHEEL[0])
HUB_D = side(MAST_PX, 77)


def tilt(p):
    """A drawing frame point (x, y, z) turned nose up about the main wheels' contact."""
    x, y, z = p
    y0 = MAIN_WHEEL[0]
    c, s = math.cos(TILT), math.sin(TILT)
    return Vector((x, y0 + (y - y0) * c - z * s, (y - y0) * s + z * c))


SHIFT = -tilt((0.0, *HUB_D)).y


def place(p):
    """A drawing frame point in model space (metres)."""
    q = tilt(p)
    return Vector((q.x, q.y + SHIFT, q.z))


def placed(geo):
    return geo.transformed(place)


# --- Fuselage ---------------------------------------------------------------------------------
# A loft from the nose (behind the TADS) to the tail boom's end through stations at drawing
# columns. Each section, from the belly's centre up: the flat belly, its rounded corner, the
# avionics bays' side (`LOW` half width) up to their top (`CHEEK`), a step in to the upper side
# (`UP`), up to the sill or shoulder (`SILL`), the canopy (or fairing) leaning in to the roof's
# edge (`ROOF` half width) at the top (`TOP`), the flat roof. Rows and half widths in metres by
# drawing column.
TOP = [(137, 161), (145, 159.5), (152, 158), (160, 156), (163, 154), (165, 150.5), (168, 146.5), (172, 143), (180, 138.5),
       (190, 133.5), (200, 129.5), (210, 126), (216, 124.5), (222, 123), (232, 121.5), (245, 120.5), (262, 119.5), (270, 116),
       (283, 107.5), (300, 106), (345, 105), (400, 121), (450, 136), (500, 150.5), (520, 153), (560, 153.5), (640, 155),
       (692, 158)]
SILL = [(137, 164), (152, 162), (160, 165), (162, 171), (190, 161.5), (216, 152), (222, 154.5), (250, 151), (262, 150), (270, 122),
        (283, 119.5), (345, 119.5), (400, 126), (450, 140), (500, 156), (520, 158), (560, 159), (640, 161), (692, 163)]
CHEEK = [(137, 190), (150, 191), (155, 187.5), (162, 184), (175, 181), (200, 179), (250, 176), (300, 173), (350, 171), (395, 170), (420, 172),
         (450, 176), (500, 180), (560, 181), (640, 180), (692, 178)]
BOTTOM = [(137, 197), (145, 199), (152, 200), (395, 200), (420, 203), (465, 205), (490, 201.5), (530, 198), (560, 196),
          (640, 190), (692, 188)]
LOW = [(137, 0.36), (150, 0.42), (155, 0.50), (162, 0.62), (170, 0.74), (180, 0.82), (195, 0.85), (390, 0.85), (410, 0.80),
       (430, 0.70), (460, 0.55), (500, 0.45), (560, 0.38), (640, 0.27), (692, 0.24)]
UP = [(137, 0.36), (150, 0.42), (162, 0.50), (175, 0.56), (200, 0.58), (262, 0.60), (345, 0.60), (400, 0.58), (430, 0.52),
      (460, 0.48), (500, 0.43), (560, 0.36), (640, 0.25), (692, 0.22)]
ROOF = [(137, 0.26), (150, 0.30), (160, 0.28), (165, 0.27), (175, 0.30), (200, 0.34), (216, 0.36), (262, 0.40), (283, 0.40),
        (345, 0.42), (400, 0.38), (450, 0.35), (500, 0.32), (560, 0.27), (640, 0.18), (692, 0.15)]
COLUMNS = [137, 145, 152, 157, 162, 165, 168, 172, 180, 190, 200, 210, 216, 219, 222, 232, 245, 255, 262, 270, 283, 300, 320,
           345, 370, 395, 420, 450, 475, 500, 520, 560, 600, 640, 670, 692]
COLUMNS1 = [137, 152, 162, 168, 180, 200, 216, 222, 245, 262, 270, 283, 345, 395, 420, 450, 500, 560, 640, 692]
COLUMNS2 = [137, 162, 180, 216, 262, 283, 345, 420, 500, 692]


def section(px):
    """The fuselage's ring at a drawing column: points round from the belly's centre (drawing
    frame)."""
    y = sy(px)
    zb, zc, zs, zt = (sz(interp(t, px)) for t in (BOTTOM, CHEEK, SILL, TOP))
    wl, wu, wt = (interp(t, px) for t in (LOW, UP, ROOF))
    r = min(0.12, (zc - zb) * 0.4, wl * 0.3)
    step = min(0.05, (zs - zc) * 0.3)
    right = [(wl - r, zb), (wl, zb + r), (wl, zc), (wu, zc + step), (wu, zs), (wt, zt)]
    pts = [(0.0, zb)] + right + [(0.0, zt)] + [(-x, z) for x, z in reversed(right)]
    return [Vector((x, y, z)) for x, z in pts]


# --- Windows ------------------------------------------------------------------------------------
# A window is a list of constraints, all of which a point of the skin must meet: a convex outline
# in the side view ((y, z), either side) or in the front view ((x, z)). The frames: 0.06 m along
# the roof's edge, 0.035 m along the sill, the posts between them.


def roof_line(px, drop):
    return side(px, interp(TOP, px) + drop)


def sill_line(px, rise):
    return side(px, interp(SILL, px) - rise)


def front_rect(hw, z0=0.0, z1=6.0):
    return ("front", [(-hw, z0), (hw, z0), (hw, z1), (-hw, z1)])


# The gunner's side window: from the windshield's frame along the canopy's top to the post
# between the seats, down to the sill sloping up from the nose.
GUNNER_SIDE = ("side", [sill_line(163.5, 1.5), roof_line(163.5, 2.5)] + [roof_line(px, 2.5) for px in (168, 172, 180, 190, 200, 208)]
               + [roof_line(214.5, 2.5), sill_line(214.5, 1.5)])
# The pilot's side window: behind the post to the rear frame leaning forward down to the sill.
PILOT_SIDE = ("side", [sill_line(223.5, 1.5), roof_line(223.5, 2.5), roof_line(240, 2.5), roof_line(259, 2.5), side(249, 148.5)])
# The roof panes over each seat (the gunner's runs down into the windshield).
GUNNER_ROOF = [("side", [side(163.6, 90), side(214.5, 90), side(214.5, 200), side(163.6, 200)]), front_rect(0.21)]
PILOT_ROOF = [("side", [side(223.5, 90), side(258, 90), side(258, 200), side(223.5, 200)]), front_rect(0.30)]
WINDOWS = [[GUNNER_SIDE], [PILOT_SIDE], GUNNER_ROOF, PILOT_ROOF]
# Windows lie between these drawing columns.
GLASS_PX = (160, 265)


def fuselage(columns, glazed=True):
    """The fuselage's skin through the stations (drawing frame), the canopy cut into it."""
    g = Geo()
    rings = [section(px) for px in columns]
    m = len(rings[0])
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
                if glazed and GLASS_PX[0] <= columns[i + 1] and columns[i] <= GLASS_PX[1]:
                    gl, bd = cut(t, WINDOWS)
                else:
                    gl, bd = [], [t]
                for p in gl:
                    if area(p) > 1e-6:
                        g.face(p, GLASS)
                for p in bd:
                    if area(p) > 1e-6:
                        g.face(p, BODY)
    for r, other in ((rings[0], rings[1]), (rings[-1], rings[-2])):
        c = sum(other, Vector()) / m
        g.face(r, BODY, c)
    return g


# --- Nose: TADS and PNVS ------------------------------------------------------------------------
# The TADS turret: its azimuth axis, the drum's radius, the sensor housings either side (centre
# out from the axis, radius) and their windows; the PNVS turret on top of the nose.
TADS_AXIS = (0.0, sy(122), sz(186))
TADS_DRUM = (0.24, sz(200), sz(171))
TADS_HOUSING = (0.29, 0.31)
TADS_HOUSING_Y = sy(118)
PNVS = (0.0, sy(116), sz(166))


def sphere_x(cx, cy, cz, r, n, x0, x1):
    """A housing along x from x0 to x1 (outward), round in section, its outer end domed."""
    rings = []
    span = x1 - x0
    for t, s in ((0.0, 0.92), (0.15, 1.0), (0.7, 1.0), (0.88, 0.85), (1.0, 0.45)):
        x = x0 + span * t
        rings.append(mklib.ring((cx + x, cy, cz), r * s, r * s, n, axis=(1, 0, 0)))
    if x1 < x0:
        rings = [list(reversed(r)) for r in rings]
    return mklib.tube(rings, SENSOR)


def tads(level):
    """The TADS turret about its azimuth pivot (drawing frame, relative to `TADS_AXIS`): the
    drum, the day sensor housing on the right and the night sensor's on the left with their
    windows."""
    g = Geo()
    n = (12, 8, 6)[level]
    ax, ay, az = TADS_AXIS
    r, z0, z1 = TADS_DRUM
    g.add(frustum((0, 0, z0 - az), (0, 0, z1 - az), r, r * 0.9, SENSOR, n=n))
    dy = TADS_HOUSING_Y - ay
    off, hr = TADS_HOUSING
    for sxn in (-1, 1):
        g.add(sphere_x(0.0, dy, 0.0, hr, n, sxn * 0.04, sxn * (off + hr * 0.85)))
        if level < 2:
            # The windows: tall on the day side (right), square on the night side.
            x = sxn * 0.22
            front = dy + hr
            w, h = (0.07, 0.13) if sxn > 0 else (0.075, 0.08)
            g.face([(x - w, front + 0.01, -h), (x + w, front + 0.01, -h), (x + w, front + 0.01, h), (x - w, front + 0.01, h)], LENS,
                   (x, front - 0.3, 0))
    return g


def pnvs(level):
    """The PNVS turret on top of the nose and its fairing back to the fuselage."""
    g = Geo()
    n = (10, 8, 6)[level]
    x, y, z = PNVS
    g.add(frustum((x, y, z - 0.17), (x, y, z + 0.17), 0.16, 0.15, SENSOR, n=n))
    g.add(mklib.block(-0.15, 0.15, sy(137), y, z - 0.17, z + 0.15, SENSOR))
    if level < 2:
        g.face([(-0.06, y + 0.165, z - 0.07), (0.06, y + 0.165, z - 0.07), (0.06, y + 0.165, z + 0.07), (-0.06, y + 0.165, z + 0.07)],
               LENS, (0, y, z))
    # The TADS's mount down from the nose.
    g.add(mklib.block(-0.13, 0.13, sy(132), sy(137.5), sz(172), sz(166), SENSOR))
    return g


# --- Chin gun -----------------------------------------------------------------------------------
GUN_PIVOT = (0.0, sy(213), sz(218))
GUN_MOUNT_Z = sz(200)
BARREL_TIP = sy(155)


def gun_turret(level):
    """The turret's drum hanging from the belly (relative to the pivot)."""
    g = Geo()
    n = (10, 8, 6)[level]
    _, _, pz = GUN_PIVOT
    g.add(frustum((0, 0, GUN_MOUNT_Z - pz + 0.02), (0, 0, 0.12), 0.20, 0.17, METAL, n=n))
    g.add(frustum((0, 0, 0.12), (0, 0, 0.06), 0.17, 0.12, METAL, n=n))
    # The cradle's arms either side of the gun.
    g.add(mklib.block(-0.14, -0.09, -0.16, 0.16, -0.10, 0.06, METAL))
    g.add(mklib.block(0.09, 0.14, -0.16, 0.16, -0.10, 0.06, METAL))
    return g


def gun(level):
    """The M230: the receiver, the barrel and its muzzle brake (relative to the pivot)."""
    g = Geo()
    n = (8, 6, 4)[level]
    py = GUN_PIVOT[1]
    g.add(mklib.block(-0.08, 0.08, -0.32, 0.36, -0.10, 0.06, DARK))
    tip = BARREL_TIP - py
    g.add(frustum((0, 0.36, -0.02), (0, tip, -0.02), 0.045, 0.04, DARK, n=n))
    if level == 0:
        g.add(frustum((0, tip - 0.12, -0.02), (0, tip, -0.02), 0.055, 0.055, DARK, n=n))
        # The feed chute up into the turret.
        g.add(mklib.block(-0.05, 0.05, -0.30, -0.10, 0.06, 0.12, DARK))
    return g


# --- Engines ------------------------------------------------------------------------------------
NACELLE_X, NACELLE_Z = 0.97, sz(137.5)
# Sections along the nacelle: (column, half width, half height).
NACELLE = [(327, 0.22, 0.20), (329, 0.31, 0.29), (334, 0.37, 0.35), (395, 0.38, 0.36), (412, 0.35, 0.33), (424, 0.30, 0.28)]


def nacelles(level):
    """The engine nacelles on the fuselage's shoulders, the intakes in front, the exhaust
    suppressors at the back turned out and up."""
    g = Geo()
    n = (14, 10, 6)[level]
    st = NACELLE if level == 0 else [NACELLE[i] for i in (0, 2, 3, 5)]
    for sxn in (-1, 1):
        x = sxn * NACELLE_X
        rings = [mklib.ring((x, sy(px), NACELLE_Z), hw, hh, n, power=2.6) for px, hw, hh in st]
        paints = [DARK] + [DECK] * (len(st) - 2)
        g.add(mklib.tube(rings, DECK, paints=paints))
        if level < 2:
            # The intake's dark mouth and the starter's bullet in it.
            y = sy(327) + 0.005
            g.add(frustum((x, y, NACELLE_Z), (x, y + 0.01, NACELLE_Z), 0.18, 0.18, DARK, n=n))
            # The exhaust: a box angled out and back from the nacelle's tail, its louvres dark.
            y0, y1 = sy(398), sy(422)
            xi, xo = sxn * 0.50, sxn * 1.28
            z0, z1 = NACELLE_Z - 0.26, NACELLE_Z + 0.30
            pts = [(xi, y0, z0), (xo, y0 + 0.10, z0), (xo, y1, z0), (xi, y1 + 0.15, z0),
                   (xi, y0, z1), (xo, y0 + 0.10, z1 - 0.06), (xo, y1, z1 - 0.06), (xi, y1 + 0.15, z1)]
            if sxn < 0:
                pts = [pts[i] for i in (1, 0, 3, 2, 5, 4, 7, 6)]
            g.add(mklib.hexa(pts, DARK))
    return g


# --- Mast, rotor head, upper fairings -------------------------------------------------------------
MAST_FOOT = sz(105.5)
ROTOR_R, ROTOR_CHORD = 7.315, 0.53
TAIL_HUB_D = (-0.36, *side(683, 85))
TAIL_R, TAIL_CHORD = 1.40, 0.25


def mast(level):
    """The mast from the transmission fairing up to under the hub (built vertical in model space,
    so in the drawing's frame it leans forward by `TILT`)."""
    g = Geo()
    n = 8 if level == 0 else 6
    hy, hz = HUB_D
    foot = (0.0, hy + (hz - MAST_FOOT) * math.sin(TILT), MAST_FOOT)
    top = (0.0, hy + 0.12 * math.sin(TILT), hz - 0.12)
    g.add(frustum(foot, top, 0.17, 0.13, METAL, n=n))
    if level < 2:
        # The swashplate's housing on the fairing.
        g.add(frustum((0.0, foot[1], MAST_FOOT - 0.05), (0.0, foot[1] + 0.02, MAST_FOOT + 0.18), 0.36, 0.30, METAL, n=n))
    return g


def rotor_head(level):
    """The rotor head's bulk round the hub and the rod standing on it (about the hub, before the
    rotor is flipped upside down)."""
    g = Geo()
    if level == 2:
        return g
    n = 8 if level == 0 else 6
    g.add(frustum((0, 0, -0.18), (0, 0, 0.14), 0.30, 0.26, METAL, n=n))
    for k in range(4):
        a = math.pi / 4 + k * math.pi / 2
        d = Vector((math.cos(a), math.sin(a), 0))
        g.add(frustum(d * 0.25, d * 0.70, 0.12, 0.09, METAL, n=4))
    if level == 0:
        g.add(frustum((0, 0, 0.14), (0, 0, 0.90), 0.035, 0.025, METAL, n=4))
        g.add(frustum((0, 0, 0.50), (0, 0, 0.62), 0.07, 0.06, METAL, n=6))
    return g


def wire_cutter(level):
    """The upper wire strike cutter on the canopy's top behind the pilot's windshield."""
    if level == 2:
        return Geo()
    y0, z0 = side(262, 119.5)
    y1, z1 = side(256, 106)
    return mklib.hexa([(-0.02, y0, z0), (0.02, y0, z0), (0.02, y0 - 0.12, z0), (-0.02, y0 - 0.12, z0),
                       (-0.02, y1, z1), (0.02, y1, z1), (0.02, y1 - 0.05, z1), (-0.02, y1 - 0.05, z1)], METAL)


# --- Wings and stores -----------------------------------------------------------------------------
WING_Z = sz(159.5)
WING_LE = sy(298)
WING_CHORD = 0.98
WING_ROOT, WING_TIP = 0.55, 2.62
PYLONS = (1.62, 2.35)
PYLON_FOOT = WING_Z - 0.40
HELLFIRE_NOSE = sy(276)


def wings(level):
    g = Geo()
    g.add(mklib.wing((WING_ROOT, WING_LE, WING_Z), WING_CHORD, (WING_TIP, WING_LE - 0.03, WING_Z), WING_CHORD * 0.95, BODY, t=0.15))
    if level < 2:
        for sxn in (-1, 1):
            # The tips' fairings.
            x = sxn * WING_TIP
            g.add(mklib.block(min(x, x + sxn * 0.06), max(x, x + sxn * 0.06), WING_LE - WING_CHORD, WING_LE, WING_Z - 0.06, WING_Z + 0.06,
                              BODY))
            for px in PYLONS:
                xp = sxn * px
                # The pylon's fairing down from the wing.
                g.add(mklib.hexa([(xp - 0.06, WING_LE - 0.05, PYLON_FOOT), (xp + 0.06, WING_LE - 0.05, PYLON_FOOT),
                                  (xp + 0.06, WING_LE - 0.85, PYLON_FOOT), (xp - 0.06, WING_LE - 0.85, PYLON_FOOT),
                                  (xp - 0.07, WING_LE - 0.02, WING_Z), (xp + 0.07, WING_LE - 0.02, WING_Z),
                                  (xp + 0.07, WING_LE - 0.92, WING_Z), (xp - 0.07, WING_LE - 0.92, WING_Z)], BODY))
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
            g.face2([base + Vector((0, -1.25, 0)) + d * 0.08, base + Vector((0, -1.60, 0)) + d * 0.08, base + Vector((0, -1.58, 0)) + d * 0.17],
                    METAL)
    return g


# The four rails of an M299, in the order they fire: (x off the pylon, z under its foot).
HELLFIRE_RAILS = [(0.19, -0.42), (-0.19, -0.42), (0.19, -0.14), (-0.19, -0.14)]


def m299(x, count, level):
    """An M299 launcher under the pylon at `x` (drawing frame) with `count` Hellfires: the spine
    down the middle, the rails either side."""
    g = Geo()
    y0, y1 = HELLFIRE_NOSE - 0.25, HELLFIRE_NOSE - 1.55
    g.add(mklib.block(x - 0.04, x + 0.04, y1, y0, PYLON_FOOT - 0.52, PYLON_FOOT, METAL))
    g.add(mklib.block(x - 0.27, x + 0.27, y1 + 0.25, y0 - 0.15, PYLON_FOOT - 0.06, PYLON_FOOT, METAL))
    if level == 0:
        g.add(mklib.block(x - 0.27, x + 0.27, y1 + 0.25, y0 - 0.15, PYLON_FOOT - 0.34, PYLON_FOOT - 0.30, METAL))
    # The rails left: the last `count` in firing order.
    for dx, dz in HELLFIRE_RAILS[len(HELLFIRE_RAILS) - count:]:
        g.add(hellfire((x + (dx if x > 0 else -dx), HELLFIRE_NOSE, PYLON_FOOT + dz - 0.10), level))
    return g


STINGER_Z = WING_Z - 0.22


def atas(x, count, level):
    """The wingtip's two-round Stinger launcher (`x` the tip, drawing frame): a beam under the tip
    and the launch tubes either side of it; `count` of them loaded."""
    g = Geo()
    n = 6 if level == 0 else 4
    sxn = 1 if x > 0 else -1
    y0 = WING_LE + 0.20
    g.add(mklib.block(x - 0.035, x + 0.035, y0 - 1.30, y0 - 0.20, STINGER_Z, WING_Z - 0.05, METAL))
    for k, dx in enumerate((0.09, -0.09)):
        if k >= count:
            continue
        xt = x + sxn * dx
        g.add(frustum((xt, y0, STINGER_Z), (xt, y0 - 1.52, STINGER_Z), 0.065, 0.065, STORE, n=n))
        if level == 0:
            g.add(frustum((xt, y0 + 0.01, STINGER_Z), (xt, y0, STINGER_Z), 0.055, 0.055, DARK, n=n))
    return g


# --- Tail ---------------------------------------------------------------------------------------
FIN_ROOT = (sy(637), sy(692), sz(156), 0.20)
FIN_TOP = (sy(687), sy(725), sz(53), 0.11)
STAB_LE, STAB_CHORD, STAB_SPAN, STAB_Z = sy(668), 1.07, 1.70, sz(158.5)


def tail(level):
    """The swept fin, the tail rotor's gearbox on its left, the stabilator at its foot."""

    def fin_section(le, te, z, t):
        le, te = Vector((0, le, z)), Vector((0, te, z))
        a, b = le + (te - le) * 0.25, le + (te - le) * 0.7
        return [le, a + Vector((t / 2, 0, 0)), b + Vector((t * 0.35, 0, 0)), te, b - Vector((t * 0.35, 0, 0)), a - Vector((t / 2, 0, 0))]

    g = Geo()
    g.add(mklib.tube([fin_section(*FIN_ROOT), fin_section(*FIN_TOP)], BODY))
    g.add(mklib.wing((0.0, STAB_LE, STAB_Z), STAB_CHORD, (STAB_SPAN, STAB_LE - 0.04, STAB_Z), STAB_CHORD * 0.92, BODY, t=0.10))
    if level < 2:
        x, y, z = TAIL_HUB_D
        n = 8 if level == 0 else 6
        g.add(frustum((-0.05, y, z), (x + 0.10, y, z), 0.18, 0.14, METAL, n=n))
    return g


def scissor_rotor(level):
    """The tail rotor in its own frame (it turns about local x): two two-blade rotors stacked
    at 55 degrees to each other."""
    g = Geo()
    if level < 2:
        g.add(frustum((-0.10, 0, 0), (0.10, 0, 0), 0.12, 0.10, METAL, n=6))
    side_ax = Vector((1, 0, 0))
    for k, a in enumerate((0, 55, 180, 235)):
        a = math.radians(a)
        d = Vector((0, math.cos(a), math.sin(a)))
        c = Vector((0, -math.sin(a), math.cos(a)))
        dx = Vector((0.05 if k % 2 else -0.05, 0, 0))
        if level == 0:
            st = [(0.12, TAIL_CHORD * 0.7, 0.0, 0.0, 0.05), (TAIL_R - 0.08, TAIL_CHORD, 0.0, 0.0, 0.035), (TAIL_R, TAIL_CHORD * 0.9, 0.0, 0.0, 0.03)]
            g.add(mklib.tube(heli.blade_rings(d, c, side_ax, st), BLADE).transformed(lambda p, dx=dx: p + dx))
        else:
            p0, p1 = d * 0.1 + dx, d * TAIL_R + dx
            g.face2([p0 + c * TAIL_CHORD * 0.3, p1 + c * TAIL_CHORD * 0.3, p1 - c * TAIL_CHORD * 0.7, p0 - c * TAIL_CHORD * 0.7], BLADE)
    return g


# --- Gear ---------------------------------------------------------------------------------------
WHEEL_R, WHEEL_X = 0.35, 0.98
TAIL_WHEEL_R = 0.22


def gear(level):
    """The trailing arm main gear from the avionics bays' undersides, the tail wheel on its fork
    under the fin."""
    g = Geo()
    n = 10 if level == 0 else 6
    wy, _ = MAIN_WHEEL
    wz = WHEEL_R + MAIN_WHEEL[1]
    belly = sz(200)
    for sxn in (-1, 1):
        x = sxn * WHEEL_X
        g.add(mklib.wheel((x, wy, wz), WHEEL_R, 0.22, TYRE, n=n))
        # The arm from its pivot under the bay back to the axle, the shock strut down to it.
        g.add(frustum((sxn * 0.70, sy(248), belly + 0.02), (x - sxn * 0.13, wy, wz), 0.06, 0.05, METAL, n=4))
        g.add(frustum((sxn * 0.72, sy(258), belly + 0.02), (x - sxn * 0.13, wy + 0.05, wz + 0.05), 0.05, 0.05, METAL, n=4))
        if level == 0:
            g.add(frustum((x - sxn * 0.13, wy, wz), (x - sxn * 0.02, wy, wz), 0.06, 0.06, METAL, n=4))
    ty, tz = TAIL_WHEEL
    tz += TAIL_WHEEL_R
    g.add(mklib.wheel((0.0, ty, tz), TAIL_WHEEL_R, 0.12, TYRE, n=n))
    fy, fz = side(682, 172)
    for sxn in (-1, 1):
        g.add(frustum((sxn * 0.08, fy, fz), (sxn * 0.08, ty, tz), 0.035, 0.03, METAL, n=4))
    return g


# --- Seams and details ----------------------------------------------------------------------------


def details(level):
    g = Geo()
    if level == 2:
        return g
    # The bays' rounded fronts: a cap ahead of the stations' sharp corner.
    if level == 0:
        # Antenna mast on the boom, the ADF loop under it.
        g.add(frustum((0, sy(470), sz(140)), (0, sy(470), sz(126)), 0.02, 0.015, METAL, n=4))
        # Steps on the bays' sides.
        for sxn in (-1, 1):
            for px in (232, 284):
                y, z = side(px, 190)
                x = sxn * 0.87
                g.add(mklib.block(min(x, x + sxn * 0.03), max(x, x + sxn * 0.03), y - 0.12, y + 0.12, z - 0.02, z + 0.02, METAL))
    return g


def body(level):
    """The airframe without its moving parts and stores, in the drawing's frame."""
    g = Geo()
    cols = (COLUMNS, COLUMNS1, COLUMNS2)[level]
    g.add(fuselage(cols))
    g.add(pnvs(level))
    g.add(nacelles(level))
    g.add(mast(level))
    g.add(wings(level))
    g.add(tail(level))
    g.add(wire_cutter(level))
    g.add(details(level))
    if level < 2:
        g.add(gear(level))
    else:
        # Far away the gun and the TADS are part of the body.
        g.add(gun_turret(level).transformed(lambda p: p + Vector(GUN_PIVOT)))
        g.add(gun(level).transformed(lambda p: p + Vector(GUN_PIVOT)))
        g.add(tads(level).transformed(lambda p: p + Vector(TADS_AXIS)))
    return g


# Lights in the drawing's frame: navigation lights on the wing tips, beacons on the aft fairing
# and under the belly.
NAV_Y = WING_LE - 0.45
NAV = {RED: [(-WING_TIP - 0.08, NAV_Y, WING_Z)], GREEN: [(WING_TIP + 0.08, NAV_Y, WING_Z)]}
BEACONS = [(0.0, sy(420), sz(126) + 0.05), (0.0, sy(330), sz(200) - 0.04)]


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


def part(parent, name, geo, pivot, slot, axis, rest):
    """An articulated part: `geo` about its pivot (metres, part frame), `pivot` in its parent's
    frame (model units), turned by state slot `slot` about local `axis` from the rest rotation."""
    ob = heli.mesh_with_lines(parent, name, geo.scaled(K), tuple(pivot))
    ob.rotation_euler = rest
    ob["state_slot"] = slot
    ob["axis"] = axis
    return ob


def build():
    root = root_object()
    root["attr"] = 0x83
    for c in list(root.children):
        mklib.delete_tree(c)
    root["lod_dists"] = LOD_DISTS
    hub = place((0.0, *HUB_D)) * K
    tail_hub = place(TAIL_HUB_D) * K
    for level, dist in enumerate(LOD_DISTS):
        geo = placed(body(level)).scaled(K)
        lod = heli.mesh_with_lines(root, f"{NAME}_lod{level}", geo)
        lod["max_dist"] = dist
        # The main rotor turns anticlockwise seen from above: built upside down (see heli.flipped).
        rotor = heli.rotor(level, 4, ROTOR_R, ROTOR_CHORD, BLADE, METAL, clockwise=False, phase=math.pi / 4)
        rotor.add(rotor_head(level))
        main = heli.flipped(rotor).scaled(K)
        p1 = heli.mesh_with_lines(lod, f"{lod.name}_part1", main, tuple(hub))
        p1.rotation_euler = (math.pi, 0.0, 0.0)
        p1["state_slot"] = 17
        p1["axis"] = 0
        tail_rot = scissor_rotor(level).scaled(K)
        p2 = heli.mesh_with_lines(lod, f"{lod.name}_part2", tail_rot, tuple(tail_hub))
        p2["state_slot"] = 17
        p2["axis"] = 1
        radius = heli.radius(geo, main.transformed(lambda p: p + hub))
        lod["radius"] = radius
        heli.lights(lod, 12, {c: [place(p) * K for p in pts] for c, pts in NAV.items()})
        heli.lights(lod, 20, {RED: [place(p) * K for p in BEACONS]})
        faces = len(geo.faces) + len(main.faces) + len(tail_rot.faces)
        if level < 2:
            # The chin gun and the TADS turn square to the fuselage: their rest pose is the tilt.
            rest = (TILT, 0.0, 0.0)
            turret = part(lod, f"{lod.name}_part3", gun_turret(level), place(GUN_PIVOT) * K, 30, 0, rest)
            part(turret, f"{turret.name}_part4", gun(level), (0.0, 0.0, 0.0), 31, 1, (0.0, 0.0, 0.0))
            part(lod, f"{lod.name}_part5", tads(level), place(TADS_AXIS) * K, 9, 0, rest)
            for slot, px in ((21, -PYLONS[1]), (22, -PYLONS[0]), (23, PYLONS[0]), (24, PYLONS[1])):
                for count in range(5):
                    group(lod, [(slot, count + 1)], placed(m299(px, count, level)))
            for slot, x in ((26, -WING_TIP), (27, WING_TIP)):
                for count in range(3):
                    group(lod, [(slot, count + 1)], placed(atas(x, count, level)))
        print(f"{lod.name}: {faces} faces, radius {radius}")
    print(f"tilt {math.degrees(TILT):.2f} deg, hub {tuple(round(c / K, 3) for c in hub)} m, "
          f"tail hub {tuple(round(c / K, 3) for c in tail_hub)} m, gun pivot {tuple(round(c, 3) for c in place(GUN_PIVOT))} m, "
          f"TADS {tuple(round(c, 3) for c in place(TADS_AXIS))} m")


if __name__ == "__main__":
    mklib.load_palette(os.path.join(ROOT, "assets", "palettes", "tod_central_europe.ron"))
    build()
    bpy.ops.wm.save_mainfile()
