"""Builds the MH-60R Seahawk (`aircraft_078`) in aircraft.blend with three detail levels, and
saves it:

    blender -b assets-src/blend/aircraft.blend --python-exit-code 1 --python tools/models/seahawk.py
    tools/models/export.sh aircraft

The airframe is the UH-60's (uh60.py's stations, rotors and pylon) in navy grey, changed where
the Seahawk differs: the tail wheel moved forward under the end of the cabin (wheelbase 4.83 m,
twin wheels), the main wheels on short sponsons (track 2.79 m), the FLIR ball under the nose,
the search radar's flat radome under the forward belly, the dipping sonar's hatch in the cabin
floor, the sonobuoy launcher on the left side of the cabin, and a weapons pylon on each side
behind the cabin door. Origin on the ground under the mast, metres scaled by uh60.K; x right,
y forward, z up.

Levels 0 and 1 also carry groups by state slot (aircraft::SEAHAWK_* in the game):
- slot 26: the left outer station's Hellfire launcher, value = missiles left (0 the bare
  launcher, -1 nothing);
- slots 21 / 22: the left / right inner station's Mk-54 (1 hung, 0 gone);
- slot 30: the MAD towed body, 0 stowed in its cradle on the right pylon, 1 towed below and
  behind on its cable;
- slot 31: the dipping sonar, 1 its transducer down on its cable out of the cabin floor.
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
import uh60  # noqa: E402
from mklib import Geo, Ramp, frustum, loft, v3  # noqa: E402

NAME = "aircraft_078"
K = uh60.K
LOD_DISTS = uh60.LOD_DISTS

# Navy light grey; the UH-60's builders read these module globals, so they are swapped in.
BODY = Ramp(0xE2, 0xEA)
DECK = Ramp(0xE3, 0xEB)
METAL = Ramp(0xE8, 0xEC)
DARK = uh60.DARK
STORE = Ramp(0xE4, 0xEA)
SENSOR = Ramp(0xEA, 0xEF)
LENS = Ramp(157, 159)
TYRE = uh60.TYRE
SEAM = uh60.SEAM
BLADE = uh60.BLADE
RED, GREEN = uh60.RED, uh60.GREEN
uh60.BODY, uh60.DECK, uh60.METAL = BODY, DECK, METAL

STATIONS = uh60.STATIONS
MAST, TAIL_HUB = uh60.MAST, uh60.TAIL_HUB
MAIN_WHEEL = (1.395, 1.06, 0.34)
TAIL_WHEEL = (0.0, 1.06 - 4.83, 0.21)

# The weapons pylon: a short stub out of the cabin side behind the door, a station under it at
# INNER_X and one at OUTER_X; the stores hang with their top at STORE_Z.
PYLON_Y, PYLON_CHORD, PYLON_Z = -1.35, 0.75, 1.62
INNER_X, OUTER_X, PYLON_TIP = 1.55, 2.08, 2.26
STORE_Z = 1.30
# The dipping sonar's hatch in the cabin floor (y), the MAD bird's cradle (right outer station).
HATCH = (0.0, -0.35)
MAD_LEN, MAD_R = 1.85, 0.11
# How far the cables pay out in the model (m): the bird trails behind and below, the sonar
# hangs straight down; the sea hides what is under it.
MAD_TOW = (0.0, -14.0, -22.0)
DIP_DOWN = 40.0


def side_x(y, z):
    return heli.side_x(STATIONS, y, z)


def gear(full=True):
    """Main wheels on sponson struts, twin tail wheels forward under the end of the cabin."""
    g = Geo()
    n = 8 if full else 6
    x, y, z = MAIN_WHEEL
    for sx in (-1, 1):
        g.add(mklib.wheel((sx * x, y, z), z, 0.22, TYRE, n=n))
        # The sponson and its oleo down to the axle.
        g.add(mklib.block(sx * 0.90, sx * 1.30, y - 0.45, y + 0.40, 0.62, 0.86, BODY, taper=0.0))
        if full:
            g.add(frustum((sx * 1.20, y, 0.70), (sx * (x - 0.10), y, z + 0.05), 0.07, 0.06, DARK, n=4))
    x, y, z = TAIL_WHEEL
    for dx in (-0.15, 0.15):
        g.add(mklib.wheel((dx, y, z), z, 0.10, TYRE, n=n))
    g.add(frustum((0, y + 0.05, z + 0.08), (0, y + 0.35, 0.66), 0.06, 0.07, DARK, n=4))
    if full:
        g.add(frustum((-0.20, y, z), (0.20, y, z), 0.03, 0.03, DARK, n=4))
    return g


def sensors(level):
    """The FLIR ball under the nose, the radome under the forward belly, the sonar hatch."""
    g = Geo()
    n = 8 if level == 0 else 6
    # FLIR: a short mast down from the chin, the ball with its window facing forward.
    g.add(frustum((0, 4.05, 0.74), (0, 4.05, 0.60), 0.10, 0.10, METAL, n=6))
    g.add(loft([(4.30, 0.48, 0.02, 0.02), (4.25, 0.48, 0.13, 0.13), (4.05, 0.48, 0.21, 0.21), (3.85, 0.48, 0.13, 0.13), (3.80, 0.48, 0.02, 0.02)], SENSOR, n=n))
    if level == 0:
        g.face([(-0.08, 4.262, 0.55), (0.08, 4.262, 0.55), (0.08, 4.262, 0.41), (-0.08, 4.262, 0.41)], LENS.hi, Vector((0, 4.0, 0.48)))
    # Radome: a flat dish under the belly between the main wheels' sponsons.
    g.add(frustum((0, 2.05, 0.57), (0, 2.05, 0.42), 0.62, 0.50, SENSOR, n=n))
    # The sonar hatch's frame on the cabin floor.
    if level == 0:
        y0, y1 = HATCH
        g.face([(-0.42, y0 + 0.42, 0.575), (0.42, y0 + 0.42, 0.575), (0.42, y1 - 0.42, 0.575), (-0.42, y1 - 0.42, 0.575)], DARK.hi, Vector((0, 0, 2)))
    return g


def buoy_launcher(level):
    """The sonobuoy launcher on the left of the cabin behind the door: a box faired onto the side,
    its tubes' ends showing as rows of dark squares."""
    g = Geo()
    y0, y1, z0, z1 = -1.20, -2.60, 0.90, 1.80
    x = -side_x((y0 + y1) / 2, (z0 + z1) / 2)
    g.add(mklib.block(x - 0.16, x + 0.02, y1, y0, z0, z1, BODY, taper=0.02, taper_y=0.03))
    if level == 0:
        rows, cols = 5, 5
        for i in range(rows):
            for j in range(cols):
                cy = y0 - 0.14 - j * (y0 - y1 - 0.28) / (cols - 1)
                cz = z0 + 0.12 + i * (z1 - z0 - 0.24) / (rows - 1)
                s = 0.055
                g.face([(x - 0.161, cy - s, cz - s), (x - 0.161, cy + s, cz - s), (x - 0.161, cy + s, cz + s), (x - 0.161, cy - s, cz + s)], DARK.hi, Vector((0, cy, cz)))
    return g


def pylons(level):
    """A stub on each side behind the cabin door, a hanger under each station."""
    g = Geo()
    for sx in (-1, 1):
        x0 = side_x(PYLON_Y, PYLON_Z) - 0.05
        g.add(mklib.wing((sx * x0, PYLON_Y, PYLON_Z), PYLON_CHORD, (sx * PYLON_TIP, PYLON_Y - 0.05, PYLON_Z - 0.04), PYLON_CHORD * 0.85, BODY, t=0.16, mirror=False))
        for x in (INNER_X, OUTER_X):
            g.add(mklib.block(sx * x - 0.04, sx * x + 0.04, PYLON_Y - 0.55, PYLON_Y - 0.10, STORE_Z + 0.08, PYLON_Z, METAL))
    # The MAD reeling machine on the right outer station: a drum fairing under the stub.
    if level == 0:
        g.add(frustum((OUTER_X - 0.20, PYLON_Y - 0.32, STORE_Z + 0.02), (OUTER_X + 0.20, PYLON_Y - 0.32, STORE_Z + 0.02), 0.16, 0.16, METAL, n=8))
    else:
        g.add(mklib.block(OUTER_X - 0.20, OUTER_X + 0.20, PYLON_Y - 0.48, PYLON_Y - 0.16, STORE_Z - 0.14, STORE_Z + 0.18, METAL))
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


def launcher(left, level):
    """The M299 four-rail launcher on the left outer station with `left` missiles on it."""
    g = Geo()
    x = -OUTER_X
    top = STORE_Z
    g.add(mklib.block(x - 0.22, x + 0.22, PYLON_Y - 0.95, PYLON_Y + 0.25, top - 0.08, top, METAL))
    g.add(mklib.block(x - 0.03, x + 0.03, PYLON_Y - 0.85, PYLON_Y + 0.15, top - 0.42, top - 0.08, METAL))
    # Rails: upper outer, upper inner, lower outer, lower inner (fired in that order).
    rails = [(x - 0.17, top - 0.17), (x + 0.17, top - 0.17), (x - 0.17, top - 0.40), (x + 0.17, top - 0.40)]
    for k, (rx, rz) in enumerate(rails):
        if k < 4 - left:
            continue
        g.add(hellfire((rx, PYLON_Y + 0.80, rz), level))
    return g


def torpedo(x, level):
    """An Mk-54 under a station: rounded nose, the body, the tail cone with its fins and the air
    launch's parachute pack on the end (2.72 m, 324 mm)."""
    g = Geo()
    n = 8 if level == 0 else 6
    r = 0.162
    z = STORE_Z - r
    y0 = PYLON_Y + 1.15
    g.add(loft([(y0, z, 0.02, 0.02), (y0 - 0.06, z, 0.11, 0.11), (y0 - 0.20, z, r, r), (y0 - 2.05, z, r, r), (y0 - 2.40, z, 0.11, 0.11), (y0 - 2.48, z, 0.10, 0.10)], STORE, n=n, x=x))
    # The parachute pack (air launch accessory) on the tail.
    g.add(frustum((x, y0 - 2.48, z), (x, y0 - 2.72, z), 0.12, 0.12, DARK, n=n))
    if level == 0:
        for a in (0, 90, 180, 270):
            d = Vector((math.cos(math.radians(a)), 0, math.sin(math.radians(a))))
            b = Vector((x, y0, z))
            g.face2([b + Vector((0, -2.05, 0)) + d * 0.15, b + Vector((0, -2.42, 0)) + d * 0.11, b + Vector((0, -2.40, 0)) + d * 0.22, b + Vector((0, -2.10, 0)) + d * 0.22], METAL)
        # Its band and the nose cap's seam.
        g.line((x - r, y0 - 1.10, z), (x + r, y0 - 1.10, z), DARK.hi)
    return g


def mad_bird(level, towed):
    """The MAD's towed body: stowed in its cradle under the right outer station, or towed on its
    cable, aft and below."""
    g = Geo()
    n = 8 if level == 0 else 6
    x, y, z = OUTER_X, PYLON_Y + 0.55, STORE_Z - 0.18
    if towed:
        reel = Vector((x, PYLON_Y - 0.32, STORE_Z - 0.14))
        x, y, z = reel.x + MAD_TOW[0], reel.y + MAD_TOW[1], reel.z + MAD_TOW[2]
        g.line(reel, (x, y + MAD_LEN * 0.6, z), DARK.hi)
    g.add(loft([(y, z, 0.01, 0.01), (y - 0.25, z, MAD_R, MAD_R), (y - MAD_LEN + 0.40, z, MAD_R, MAD_R), (y - MAD_LEN, z, 0.05, 0.05)], STORE, n=n, x=x))
    if level == 0:
        for a in (45, 135, 225, 315):
            d = Vector((math.cos(math.radians(a)), 0, math.sin(math.radians(a))))
            b = Vector((x, y - MAD_LEN, z))
            g.face2([b + Vector((0, 0.40, 0)) + d * 0.10, b + Vector((0, 0.02, 0)) + d * 0.05, b + Vector((0, 0.02, 0)) + d * 0.24, b + Vector((0, 0.25, 0)) + d * 0.24], DARK)
    if not towed:
        g.add(mklib.block(x - 0.05, x + 0.05, y - 1.30, y - 0.40, z + MAD_R - 0.02, STORE_Z + 0.02, METAL))
    return g


def dipping_sonar(level):
    """The AQS-13F's transducer on its cable straight down out of the cabin floor's hatch."""
    g = Geo()
    n = 8 if level == 0 else 6
    y = sum(HATCH) / 2
    g.line((0, y, 0.57), (0, y, 0.57 - DIP_DOWN), DARK.hi)
    z = 0.57 - DIP_DOWN
    g.add(frustum((0, y, z), (0, y, z - 1.1), 0.24, 0.24, STORE, n=n))
    g.add(frustum((0, y, z - 1.1), (0, y, z - 1.3), 0.24, 0.10, DARK, n=n))
    return g


def body(level):
    g = Geo()
    if level == 0:
        g.add(heli.fuselage(STATIONS, BODY, uh60.GLASS, True, uh60.GLAZED))
        g.add(uh60.chin_frame(level))
        g.add(uh60.nacelles(True))
        uh60.details(g)
    elif level == 1:
        st = [STATIONS[i] for i in (0, 1, 3, 4, 5, 6, 8, 11, 12, 14, 15)]
        g.add(heli.fuselage(st, BODY, uh60.GLASS, True, {1: uh60.SIDES, 2: uh60.GLAZED[uh60.WINDSHIELD], 3: uh60.GLAZED[uh60.ROOF], 4: uh60.GLAZED[uh60.COCKPIT]}))
        g.add(uh60.chin_frame(level))
        g.add(uh60.nacelles(False))
    else:
        st = [STATIONS[i] for i in (1, 3, 4, 8, 11, 12, 15)]
        g.add(heli.fuselage(st, BODY, uh60.GLASS, False, {1: {1, 2, 3}}))
    g.add(uh60.pylon())
    g.add(uh60.stabilator(0.08 if level < 2 else 0.05))
    g.add(uh60.upper_deck(level == 0))
    g.add(uh60.tail_gearbox(level == 0))
    if level < 2:
        g.add(gear(level == 0))
        g.add(sensors(level))
        g.add(buoy_launcher(level))
        g.add(pylons(level))
    return g


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
        main = heli.flipped(heli.rotor(level, 4, uh60.ROTOR_R, uh60.ROTOR_CHORD, BLADE, METAL, clockwise=False, phase=math.pi / 4))
        tail = heli.tail_rotor(level, 4 if level < 2 else 2, uh60.TAIL_R, uh60.TAIL_CHORD, BLADE, METAL)
        levels.append((dist, body(level), main, tail, (math.pi, 0.0, 0.0)))
    heli.build_levels(root, levels, K, MAST, TAIL_HUB, (0.0, -uh60.TAIL_CANT, 0.0), uh60.NAV, {RED: uh60.BEACONS})
    for level in (0, 1):
        lod = bpy.data.objects[f"{NAME}_lod{level}"]
        for left in range(5):
            group(lod, [(26, left)], launcher(left, level))
        group(lod, [(21, 1)], torpedo(-INNER_X, level))
        group(lod, [(22, 1)], torpedo(INNER_X, level))
        group(lod, [(30, 0)], mad_bird(level, False))
        group(lod, [(30, 1)], mad_bird(level, True))
        group(lod, [(31, 1)], dipping_sonar(level))


if __name__ == "__main__":
    mklib.load_palette(os.path.join(ROOT, "assets", "palettes", "tod_central_europe.ron"))
    build()
    bpy.ops.wm.save_mainfile()
