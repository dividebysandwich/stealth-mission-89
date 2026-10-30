"""Builds the remake's ground units in Blender: the low-level air defences (ZSU-23-4 Shilka, S-60 gun,
MANPADS team, and the friendly Patriot launcher and radar, the Gepard and the drone command center)
and the armour (T-90A and BMP-2 for the red side, Leopard 2A6 and M2A2 Bradley for the blue), for
the air defence option and the campaign (crates/stealth-sim/src/aaa.rs):

    blender -b --factory-startup --python-exit-code 1 --python tools/models/air_defence.py -- assets/models/aaa.glb

Nodes follow aircraft.glb's conventions: a root `aaa_NNN` with `attr` and `_lodN` levels. In the
detailed level the intact model is `_s1v0` and the wreck `_s1v1` (state slot 1, the destroyed
variant); the intact model's moving parts are `_part1` (turret or crew, yaw, state slot 21),
`_part2` (guns or launcher, elevation, slot 22, a child of part 1) and on the Shilka `_part3`
(the Gun Dish radar, or the Gepard's search radar, turning with the global radar angle, slot 16).
The tanks and IFVs have just the turret (part 1) and the main gun (part 2, pivoting at its
trunnions); their wrecks may have the turret blown off beside the hull. Model space: x right,
y forward, z up, aircraft model units (WU / 8 m); built in metres and scaled.
"""

import math
import os
import sys

import bpy

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.abspath(os.path.join(HERE, "..", ".."))
sys.path.insert(0, HERE)

import mklib  # noqa: E402
from mklib import Geo, Ramp, block, frustum, wheel  # noqa: E402

# Model numbers (after the drones' 100 and 101).
ZSU = 110
S60 = 111
MANPADS = 112
PATRIOT_LN = 113
PATRIOT_RADAR = 114
GEPARD = 115
DRONES = 116
T90 = 117
LEOPARD2 = 118
BRADLEY = 119
BMP2 = 120

# Model units per metre (a unit is WU / 8 = 0.061 m).
K = 8.0 * 1_048_576 / 512_000

OLIVE = Ramp(0xA6, 0xAF)
OLIVE_DARK = Ramp(0xAA, 0xAF)
KHAKI = Ramp(0xA1, 0xA5)
STEEL = Ramp(0xEB, 0xEF)
RUBBER = Ramp(0xED, 0xEF)
BURNT = Ramp(0xEC, 0xEF)
SKIN = Ramp(0xC2, 0xC5)
GLASS = 0x0B
CONCRETE = Ramp(0xE3, 0xEA)
EARTH = Ramp(0xA4, 0xA9)
# NATO green: the darker end of the olive ramp, so the blue armour reads a shade deeper than the red.
NATO = Ramp(0xA9, 0xAF)
NATO_DARK = Ramp(0xAC, 0xAF)
LENS = 0xDF
HOLE = 0x00

TURRET_SLOT, GUN_SLOT, RADAR_SLOT = 21, 22, 16


def empty(name, parent=None, loc=(0, 0, 0)):
    ob = bpy.data.objects.new(name, None)
    bpy.context.scene.collection.objects.link(ob)
    ob.parent = parent
    ob.location = tuple(c * K for c in loc)
    return ob


def mesh_obj(name, g, parent, loc=(0, 0, 0)):
    ob = bpy.data.objects.new(name, mklib.faces_mesh(name, g.scaled(K)))
    bpy.context.scene.collection.objects.link(ob)
    ob.parent = parent
    ob.location = tuple(c * K for c in loc)
    return ob


def part(name, g, parent, loc, slot, axis):
    ob = mesh_obj(name, g, parent, loc)
    ob["state_slot"] = slot
    ob["axis"] = axis
    return ob


def barrel(y0, y1, r, x=0.0, z=0.0, paint=STEEL, n=6):
    return frustum((x, y0, z), (x, y1, z), r, r, paint, n=n)


def sandbag_ring(r, h, n, gap=None, z0=0.0):
    """A ring of sandbags (a low wall), open over the arc `gap` (radians, around +y)."""
    g = Geo()
    for k in range(n):
        a0, a1 = 2 * math.pi * k / n, 2 * math.pi * (k + 1) / n
        mid = (a0 + a1) / 2
        if gap is not None and abs(math.atan2(math.sin(mid), math.cos(mid))) < gap / 2:
            continue
        inner, outer = r - 0.25, r + 0.25
        pts = lambda rr, a: (rr * math.sin(a), rr * math.cos(a))  # noqa: E731
        (x0, y0), (x1, y1) = pts(inner, a0), pts(inner, a1)
        (x2, y2), (x3, y3) = pts(outer, a1), pts(outer, a0)
        g.add(mklib.hexa(
            [(x0, y0, z0), (x1, y1, z0), (x2, y2, z0), (x3, y3, z0),
             (x0 * 0.98, y0 * 0.98, z0 + h), (x1 * 0.98, y1 * 0.98, z0 + h), (x2 * 0.96, y2 * 0.96, z0 + h * 0.9), (x3 * 0.96, y3 * 0.96, z0 + h * 0.9)],
            KHAKI,
        ))
    return g


# ------------------------------------------------------------------------------------ ZSU-23-4


def zsu_hull(paint=OLIVE, wheels=RUBBER, dark=OLIVE_DARK, glass=GLASS, steel=STEEL):
    """Hull 6.5 x 3.1 m on six road wheels a side, tracks round the drive sprocket in front and
    the idler behind, track skirts, the sloped glacis in front."""
    g = Geo()
    g.add(mklib.side_prism([(-3.2, 0.45), (2.9, 0.45), (3.25, 0.95), (3.0, 1.45), (-3.25, 1.45)], -1.3, 1.3, paint))
    for sx in (-1, 1):
        x = sx * 1.4
        g.add(running_gear(x, 0.32, [-2.35 + k * 0.94 for k in range(6)], 0.32, (2.95, 0.55, 0.27), (-2.95, 0.5, 0.26),
                           dark, track=wheels, hub=steel))
        # The skirt over the track's upper run.
        g.add(block(x + sx * 0.19 - 0.03, x + sx * 0.19 + 0.03, -3.05, 3.0, 0.75, 1.15, paint))
    # Engine deck louvres at the rear, a hatch and the driver's vision block in front.
    g.add(block(-1.0, 1.0, -3.0, -1.6, 1.45, 1.5, dark))
    g.add(block(-1.05, -0.45, 2.15, 2.6, 1.45, 1.55, paint))
    g.add(block(-0.95, -0.55, 2.55, 2.62, 1.47, 1.53, glass))
    return g


def zsu_turret(paint=OLIVE, dark=OLIVE_DARK, glass=GLASS):
    """Flat, wide turret around its pivot (the turret ring's centre)."""
    g = Geo()
    g.add(block(-1.45, 1.45, -1.3, 1.5, 0.0, 0.72, paint, taper=0.18, taper_y=0.12))
    # Gun housings either side, commander's hatch and periscopes.
    for sx in (-1, 1):
        g.add(block(sx * 1.45 - 0.2, sx * 1.45 + 0.2, -0.2, 1.55, 0.12, 0.6, paint))
    g.add(frustum((0.6, -0.4, 0.72), (0.6, -0.4, 0.84), 0.32, 0.28, dark, n=8))
    g.add(block(-0.3, 0.1, 1.2, 1.45, 0.72, 0.82, glass))
    return g


def zsu_guns():
    """Four 23 mm guns in two pairs around the trunnions (the part origin)."""
    g = Geo()
    for sx in (-1, 1):
        for dz in (-0.09, 0.09):
            g.add(barrel(0.0, 2.2, 0.035, x=sx * 1.45, z=dz))
            g.add(barrel(1.7, 2.25, 0.06, x=sx * 1.45, z=dz))
        g.add(block(sx * 1.45 - 0.12, sx * 1.45 + 0.12, -0.3, 0.35, -0.18, 0.18, OLIVE_DARK))
    return g


def zsu_radar():
    """The Gun Dish: a dish on a short mast (part origin at the mast foot)."""
    g = Geo()
    g.add(frustum((0, 0, 0), (0, 0, 0.35), 0.08, 0.08, STEEL, n=6))
    g.add(frustum((0, 0.05, 0.75), (0, 0.22, 0.75), 0.62, 0.42, OLIVE_DARK, n=10, rot=0.0))
    g.add(block(-0.08, 0.08, -0.05, 0.1, 0.35, 0.8, STEEL))
    return g


def zsu():
    root, lod0, lod1 = levels(ZSU, 16000)
    intact = mesh_obj("aaa_110_lod0_s1v0", zsu_hull(), lod0)
    turret = part("aaa_110_lod0_s1v0_part1", zsu_turret(), intact, (0, 0.3, 1.45), TURRET_SLOT, 0)
    part("aaa_110_lod0_s1v0_part2", zsu_guns(), turret, (0, 0.9, 0.36), GUN_SLOT, 1)
    part("aaa_110_lod0_s1v0_part3", zsu_radar(), turret, (0, -1.0, 0.72), RADAR_SLOT, 0)
    # The wreck: burnt hull, the turret knocked askew, the guns drooping.
    w = zsu_hull(BURNT, BURNT, BURNT, BURNT, BURNT)
    w.add(zsu_turret(BURNT, BURNT, BURNT).rotated_z(0.45).transformed(lambda p: p + mklib.v3((0.2, 0.3, 1.4))))
    guns = zsu_guns().recolour(BURNT).rotated_x(-0.25)
    w.add(guns.rotated_z(0.45).transformed(lambda p: p + mklib.v3((0.0, 1.3, 1.65))))
    mesh_obj("aaa_110_lod0_s1v1", w, lod0)
    far(lod1, "aaa_110", [(-1.5, 1.5, -3.2, 3.2, 0.2, 1.45), (-1.45, 1.45, -1.0, 1.8, 1.45, 2.15)])


# ----------------------------------------------------------------------------------------- S-60


def s60_carriage(paint=OLIVE, wheels=RUBBER):
    """Four-wheeled carriage on its jacks, outriggers out, inside a sandbag ring."""
    g = Geo()
    g.add(block(-0.75, 0.75, -2.0, 2.0, 0.55, 0.85, paint))
    for sx in (-1, 1):
        for y in (-1.3, 1.3):
            g.add(wheel((sx * 1.0, y, 0.48), 0.45, 0.3, wheels, n=8))
    # Outriggers to the ground, each with a jack pad.
    for a in (45, 135, 225, 315):
        t = math.radians(a)
        dx, dy = math.sin(t), math.cos(t)
        ex, ey = dx * 2.6, dy * 2.6
        g.add(frustum((dx * 0.6, dy * 0.6, 0.7), (ex, ey, 0.15), 0.08, 0.08, paint, n=4))
        g.add(block(ex - 0.2, ex + 0.2, ey - 0.2, ey + 0.2, 0.0, 0.12, STEEL))
    # Towbar and the ready ammunition crate beside the gun.
    g.add(frustum((0, 2.0, 0.65), (0, 3.0, 0.35), 0.05, 0.05, paint, n=4))
    g.add(block(1.4, 2.0, -0.6, 0.2, 0.0, 0.45, OLIVE_DARK))
    g.add(sandbag_ring(4.0, 0.7, 14, gap=0.9))
    return g


def s60_mount(paint=OLIVE):
    """Cradle and the shield around the traverse pivot."""
    g = Geo()
    g.add(frustum((0, 0, 0), (0, 0, 0.25), 0.55, 0.5, paint, n=8))
    for sx in (-1, 1):
        g.add(block(sx * 0.25 - 0.08, sx * 0.25 + 0.08, -0.4, 0.4, 0.2, 0.75, paint))
        # Gunner's seats and sights.
        g.add(block(sx * 0.75 - 0.18, sx * 0.75 + 0.18, -0.5, -0.1, 0.35, 0.5, OLIVE_DARK))
    # Shield: a front plate with angled wings.
    g.face2([(-0.6, 0.55, 0.35), (0.6, 0.55, 0.35), (0.6, 0.55, 1.25), (-0.6, 0.55, 1.25)], paint)
    for sx in (-1, 1):
        g.face2([(sx * 0.6, 0.55, 0.35), (sx * 1.0, 0.25, 0.35), (sx * 1.0, 0.25, 1.1), (sx * 0.6, 0.55, 1.25)], paint)
    return g


def s60_gun():
    """The 57 mm barrel with its muzzle brake and recoil cylinders (pivot at the trunnions)."""
    g = Geo()
    g.add(block(-0.16, 0.16, -0.9, 0.5, -0.14, 0.14, OLIVE_DARK))
    g.add(barrel(0.5, 4.2, 0.055))
    g.add(barrel(4.1, 4.45, 0.09, n=6))
    for sx in (-1, 1):
        g.add(barrel(0.0, 1.6, 0.04, x=sx * 0.12, z=0.17, paint=OLIVE_DARK))
    return g


def s60():
    root, lod0, lod1 = levels(S60, 16000)
    intact = mesh_obj("aaa_111_lod0_s1v0", s60_carriage(), lod0)
    mount = part("aaa_111_lod0_s1v0_part1", s60_mount(), intact, (0, 0, 0.85), TURRET_SLOT, 0)
    part("aaa_111_lod0_s1v0_part2", s60_gun(), mount, (0, 0.1, 0.8), GUN_SLOT, 1)
    w = s60_carriage(BURNT, BURNT).rotated_x(0.05)
    w.add(s60_mount(BURNT).rotated_z(-0.6).transformed(lambda p: p + mklib.v3((0, 0, 0.8))))
    w.add(s60_gun().recolour(BURNT).rotated_x(-0.12).rotated_z(-0.6).transformed(lambda p: p + mklib.v3((0, 0, 1.55))))
    mesh_obj("aaa_111_lod0_s1v1", w, lod0)
    far(lod1, "aaa_111", [(-1.2, 1.2, -2.0, 2.0, 0.1, 0.9), (-0.1, 0.1, 0.0, 4.4, 1.5, 1.8)])


# -------------------------------------------------------------------------------------- MANPADS


def soldier(kneel, paint=OLIVE):
    """A low-poly soldier facing +y, standing or kneeling, around his feet."""
    g = Geo()
    hip = 0.55 if kneel else 0.95
    if kneel:
        # Kneeling: one shin on the ground, the other leg up.
        g.add(block(-0.22, -0.04, -0.45, 0.05, 0.0, 0.16, paint))
        g.add(block(-0.22, -0.04, -0.05, 0.12, 0.0, hip, paint))
        g.add(block(0.04, 0.22, 0.15, 0.45, 0.0, 0.5, paint))
        g.add(block(0.04, 0.22, -0.05, 0.45, 0.42, hip, paint))
    else:
        for sx in (-1, 1):
            g.add(block(sx * 0.13 - 0.08, sx * 0.13 + 0.08, -0.08, 0.1, 0.0, hip, paint))
    g.add(block(-0.22, 0.22, -0.13, 0.13, hip, hip + 0.6, paint, taper=0.03))
    g.add(frustum((0, 0, hip + 0.6), (0, 0, hip + 0.86), 0.1, 0.11, SKIN, n=6))
    # Helmet.
    g.add(frustum((0, 0, hip + 0.86), (0, 0, hip + 0.98), 0.15, 0.1, OLIVE_DARK, n=6))
    return g


def launcher():
    """SA-14 tube with its grip stock and sight, the shooter's shoulder at the origin."""
    g = Geo()
    g.add(barrel(-0.45, 1.05, 0.045, paint=OLIVE_DARK))
    g.add(frustum((0, 1.05, 0), (0, 1.12, 0), 0.06, 0.05, OLIVE_DARK, n=6))
    g.add(block(-0.04, 0.04, 0.05, 0.3, -0.25, -0.04, STEEL))
    g.add(block(-0.12, -0.04, 0.3, 0.42, 0.03, 0.12, STEEL))
    return g


def manpads_position(burnt=False):
    g = sandbag_ring(1.6, 0.5, 9, gap=2.2)
    if burnt:
        g = g.recolour(BURNT)
    return g


def manpads():
    root, lod0, _ = levels(MANPADS, 32769, far_level=False)
    intact = mesh_obj("aaa_112_lod0_s1v0", manpads_position().add(
        soldier(False).rotated_z(2.6).transformed(lambda p: p + mklib.v3((-0.9, -0.6, 0)))
        .add(barrel(-0.5, 0.5, 0.045, x=0.0, z=0.0, paint=OLIVE_DARK).rotated_x(1.2).transformed(lambda p: p + mklib.v3((-0.95, -0.45, 1.2))))
    ), lod0)
    shooter = part("aaa_112_lod0_s1v0_part1", soldier(True), intact, (0.3, 0.2, 0), TURRET_SLOT, 0)
    part("aaa_112_lod0_s1v0_part2", launcher(), shooter, (0.16, -0.05, 1.32), GUN_SLOT, 1)
    mesh_obj("aaa_112_lod0_s1v1", manpads_position(True), lod0)


# ------------------------------------------------------------------------------ Patriot launcher


def semitrailer(length, paint=OLIVE, wheels=RUBBER):
    """A flatbed semitrailer (front at +y, deck at 1.3 m) on two rear axles, legs down."""
    g = Geo()
    y0, y1 = -length / 2, length / 2
    g.add(block(-1.25, 1.25, y0, y1, 1.05, 1.3, paint))
    for sx in (-1, 1):
        g.add(block(sx * 0.6 - 0.12, sx * 0.6 + 0.12, y0 + 0.3, y1 - 0.3, 0.8, 1.05, OLIVE_DARK))
        for y in (y0 + 0.9, y0 + 2.0):
            g.add(wheel((sx * 1.0, y, 0.5), 0.5, 0.45, wheels, n=8))
        # Levelling jacks at the corners.
        for y in (y0 + 0.25, y1 - 0.4):
            g.add(block(sx * 1.35 - 0.1, sx * 1.35 + 0.1, y - 0.1, y + 0.1, 0.0, 1.1, STEEL))
            g.add(block(sx * 1.35 - 0.25, sx * 1.35 + 0.25, y - 0.25, y + 0.25, 0.0, 0.06, STEEL))
    return g


def tractor(paint=OLIVE, wheels=RUBBER):
    """A heavy truck tractor (HEMTT-ish) with its cab forward, around its rear axles."""
    g = Geo()
    g.add(block(-1.2, 1.2, -2.6, 3.4, 0.9, 1.25, OLIVE_DARK))
    g.add(block(-1.25, 1.25, 1.6, 3.6, 1.25, 2.9, paint, taper=0.1, taper_y=0.15))
    g.add(block(-1.05, 1.05, 3.55, 3.62, 2.1, 2.75, GLASS))
    for sx in (-1, 1):
        g.add(block(sx * 1.25 - 0.02 * sx, sx * 1.26, 2.3, 3.3, 2.1, 2.7, GLASS))
        for y in (-2.0, -0.7, 2.6):
            g.add(wheel((sx * 1.05, y, 0.6), 0.6, 0.5, wheels, n=8))
    g.add(block(-1.0, 1.0, 0.2, 1.5, 1.25, 1.9, paint))
    return g


def canisters(paint=OLIVE):
    """Four launch canisters in a 2 x 2 box, pivot at the rear lower edge, pointing +y."""
    g = Geo()
    g.add(block(-1.05, 1.05, 0.0, 5.8, 0.0, 2.1, paint))
    for sx in (-1, 1):
        for z in (0.5, 1.6):
            # The canister lids (darker squares on the front face).
            g.face([(sx * 0.95, 5.81, z - 0.42), (sx * 0.1, 5.81, z - 0.42), (sx * 0.1, 5.81, z + 0.42), (sx * 0.95, 5.81, z + 0.42)], OLIVE_DARK, (0, 0, 1.05))
    # Stiffening ribs along the sides.
    for y in (1.0, 2.9, 4.8):
        g.add(block(-1.1, 1.1, y, y + 0.2, -0.05, 2.15, OLIVE_DARK))
    return g


def patriot_turntable(paint=OLIVE):
    """Traverse ring and the elevation cradle on the trailer bed."""
    g = Geo()
    g.add(frustum((0, 0, 0), (0, 0, 0.3), 1.0, 0.95, paint, n=8))
    for sx in (-1, 1):
        g.add(block(sx * 1.0 - 0.1, sx * 1.0 + 0.1, -0.5, 0.4, 0.2, 0.8, OLIVE_DARK))
    return g


def launcher_body(paint=OLIVE, wheels=RUBBER):
    """Trailer and tractor, the cab to the rear (-y): the canisters fire over the trailer's tail."""
    g = semitrailer(9.0, paint, wheels).add(tractor(paint, wheels).transformed(lambda p: p + mklib.v3((0, 7.2, 0))))
    # A generator set behind the gooseneck.
    g.add(block(-1.0, 1.0, 3.2, 4.4, 1.3, 2.3, OLIVE_DARK if paint is OLIVE else paint))
    return g.rotated_z(math.pi)


def patriot_launcher():
    root, lod0, lod1 = levels(PATRIOT_LN, 20000)
    intact = mesh_obj("aaa_113_lod0_s1v0", launcher_body(), lod0)
    table = part("aaa_113_lod0_s1v0_part1", patriot_turntable(), intact, (0, 1.2, 1.3), TURRET_SLOT, 0)
    part("aaa_113_lod0_s1v0_part2", canisters(), table, (0, -0.9, 0.7), GUN_SLOT, 1)
    w = launcher_body(BURNT, BURNT)
    w.add(canisters(BURNT).rotated_x(0.15).rotated_z(0.3).transformed(lambda p: p + mklib.v3((0.4, 0.0, 1.1))))
    mesh_obj("aaa_113_lod0_s1v1", w, lod0)
    far(lod1, "aaa_113", [(-1.25, 1.25, -10.8, 4.5, 0.4, 1.3), (-1.25, 1.25, -10.8, -8.8, 1.3, 2.9), (-1.05, 1.05, 0.3, 5.5, 1.6, 4.3)])


# --------------------------------------------------------------------------------- Patriot radar


def phased_array(paint=OLIVE):
    """The AN/MPQ-53 shelter with its big slanted array face to the front (+y)."""
    g = Geo()
    g.add(block(-1.2, 1.2, -2.2, 1.0, 0.0, 2.3, paint))
    # The array: an octagonal plate leaning back 22 degrees, on a frame on the front of the shelter.
    face = Geo()
    n = 8
    pts = [(1.25 * math.cos(2 * math.pi * (k + 0.5) / n), 0.0, 1.25 * math.sin(2 * math.pi * (k + 0.5) / n)) for k in range(n)]
    back = [(x, -0.25, z) for x, _, z in pts]
    face.add(mklib.tube([[mklib.v3(p) for p in back], [mklib.v3(p) for p in pts]], OLIVE_DARK))
    g.add(face.rotated_x(-0.38).transformed(lambda p: p + mklib.v3((0, 1.35, 1.9))))
    # Three small arrays below the main one, and the IFF bar along its top.
    for x in (-0.8, 0.0, 0.8):
        g.add(block(x - 0.25, x + 0.25, 1.0, 1.2, 0.35, 0.65, OLIVE_DARK))
    g.add(block(-1.0, 1.0, 1.75, 1.9, 3.15, 3.3, STEEL))
    return g


def patriot_radar():
    root, lod0, lod1 = levels(PATRIOT_RADAR, 20000)
    intact = mesh_obj("aaa_114_lod0_s1v0", semitrailer(7.0), lod0)
    part("aaa_114_lod0_s1v0_part1", phased_array(), intact, (0, 0.2, 1.3), TURRET_SLOT, 0)
    w = semitrailer(7.0, BURNT, BURNT)
    w.add(phased_array(BURNT).recolour(BURNT).rotated_z(-0.25).rotated_x(-0.1).transformed(lambda p: p + mklib.v3((0, 0.2, 1.2))))
    mesh_obj("aaa_114_lod0_s1v1", w, lod0)
    far(lod1, "aaa_114", [(-1.25, 1.25, -3.5, 3.5, 0.4, 1.3), (-1.2, 1.2, -2.0, 1.2, 1.3, 3.6), (-1.2, 1.2, 1.2, 1.7, 2.0, 4.4)])


# ---------------------------------------------------------------------------------------- Gepard


def gepard_hull(paint=OLIVE, wheels=RUBBER, dark=OLIVE_DARK, steel=STEEL):
    """Leopard 1 hull, 7.1 x 3.3 m, seven road wheels and four return rollers a side, the
    tracks round the rear sprocket and front idler, the wedge glacis in front."""
    g = Geo()
    g.add(mklib.side_prism([(-3.5, 0.5), (3.0, 0.5), (3.55, 1.05), (3.2, 1.55), (-3.5, 1.6)], -1.4, 1.4, paint))
    for sx in (-1, 1):
        x = sx * 1.5
        g.add(running_gear(x, 0.36, [-2.75 + k * 0.85 for k in range(7)], 0.33, (3.28, 0.6, 0.28), (-3.35, 0.62, 0.3),
                           dark, rollers=(-2.0, -0.6, 0.8, 2.2), track=wheels, hub=steel))
        # Side skirts.
        g.add(block(x + sx * 0.21 - 0.03, x + sx * 0.21 + 0.03, -3.3, 3.25, 0.7, 1.25, paint))
    g.add(block(-1.2, 1.2, -3.45, -1.8, 1.6, 1.7, dark))
    g.add(block(-1.1, -0.5, 2.3, 2.8, 1.55, 1.65, paint))
    return g


def gepard_turret(paint=OLIVE, dark=OLIVE_DARK, glass=GLASS):
    """The big boxy turret with its gun housings to the sides and the tracking radar in front."""
    g = Geo()
    g.add(block(-1.25, 1.25, -1.6, 1.5, 0.0, 1.15, paint, taper=0.1, taper_y=0.2))
    # Turret bustle and the cheek housings that carry the guns.
    g.add(block(-1.1, 1.1, -2.1, -1.5, 0.15, 1.0, dark))
    for x0 in (-1.33, 1.25):
        g.add(block(x0, x0 + 0.08, -0.6, 0.9, 0.3, 1.0, dark))
    # Tracking radar: a round dish on the turret front.
    g.add(frustum((0, 1.45, 0.75), (0, 1.75, 0.75), 0.5, 0.42, dark, n=10))
    g.add(block(0.5, 0.9, 0.4, 0.9, 1.15, 1.35, glass))
    return g


def gepard_guns():
    """The two 35 mm Oerlikons in their housings outside the turret (pivot at the trunnions)."""
    g = Geo()
    for sx in (-1, 1):
        x = sx * 1.6
        g.add(block(x - 0.22, x + 0.22, -1.0, 0.9, -0.3, 0.3, OLIVE))
        g.add(barrel(0.9, 3.4, 0.05, x=x))
        g.add(barrel(3.2, 3.55, 0.08, x=x))
    # The cross shaft through the turret.
    g.add(barrel(-0.1, 0.1, 0.12, x=0.0).rotated_z(math.pi / 2))
    return g


def gepard_search():
    """The search radar folded up on the turret's rear: a wide flat dish on a mast."""
    g = Geo()
    g.add(frustum((0, 0, 0), (0, 0, 0.45), 0.1, 0.08, STEEL, n=6))
    g.add(block(-1.0, 1.0, -0.1, 0.05, 0.45, 0.95, OLIVE_DARK, taper=0.25))
    return g


def gepard():
    root, lod0, lod1 = levels(GEPARD, 16000)
    intact = mesh_obj("aaa_115_lod0_s1v0", gepard_hull(), lod0)
    turret = part("aaa_115_lod0_s1v0_part1", gepard_turret(), intact, (0, 0.2, 1.6), TURRET_SLOT, 0)
    part("aaa_115_lod0_s1v0_part2", gepard_guns(), turret, (0, 0.2, 0.65), GUN_SLOT, 1)
    part("aaa_115_lod0_s1v0_part3", gepard_search(), turret, (0, -1.8, 1.0), RADAR_SLOT, 0)
    w = gepard_hull(BURNT, BURNT, BURNT, BURNT)
    w.add(gepard_turret(BURNT, BURNT, BURNT).rotated_z(-0.5).transformed(lambda p: p + mklib.v3((-0.2, 0.2, 1.55))))
    guns = gepard_guns().recolour(BURNT).rotated_x(-0.2)
    w.add(guns.rotated_z(-0.5).transformed(lambda p: p + mklib.v3((-0.1, 0.4, 2.15))))
    mesh_obj("aaa_115_lod0_s1v1", w, lod0)
    far(lod1, "aaa_115", [(-1.6, 1.6, -3.5, 3.5, 0.2, 1.6), (-1.8, 1.8, -1.6, 1.8, 1.6, 2.8)])


# ----------------------------------------------------------------------- drone command center

# The launch rail (crates/stealth-sim/src/uav.rs RAMP_*): its rear end where a drone sits, metres
# forward of the bunker's centre and up, its length and its rise (radians).
RAMP_Y = 7.0
RAMP_Z = 1.3
RAMP_LEN = 7.0
RAMP_RISE = 0.21


def bunker(paint=CONCRETE, earth=EARTH, door=STEEL):
    """An earth-covered concrete bunker, 9 x 7 m, its blast door and wing walls to the front
    (+y), the vents and the antenna mast on the mound."""
    g = Geo()
    # The mound: earth heaped over the roof and against the sides and back.
    g.add(mklib.hexa(
        [(-7.5, -7.0, 0.0), (7.5, -7.0, 0.0), (7.5, 3.4, 0.0), (-7.5, 3.4, 0.0),
         (-4.6, -4.2, 3.4), (4.6, -4.2, 3.4), (4.6, 2.6, 3.4), (-4.6, 2.6, 3.4)],
        earth,
    ))
    # The front wall with its blast door, the wing walls splaying out to the ramp.
    g.add(block(-4.6, 4.6, 2.5, 3.6, 0.0, 3.1, paint))
    g.add(block(-1.1, 1.1, 3.6, 3.75, 0.0, 2.2, door))
    g.add(block(-1.3, 1.3, 3.6, 3.7, 2.2, 2.4, paint))
    g.add(block(2.0, 3.2, 3.6, 3.65, 1.7, 1.9, GLASS))
    for sx in (-1, 1):
        # A wall 0.5 m thick from the front wall's corner out to the side, falling to the front.
        a, b = sorted((sx * 4.6, sx * 5.1))
        c, d = sorted((sx * 6.3, sx * 6.8))
        g.add(mklib.hexa(
            [(a, 3.2, 0.0), (b, 3.2, 0.0), (d, 6.0, 0.0), (c, 6.0, 0.0),
             (a, 3.2, 3.1), (b, 3.2, 3.1), (d, 6.0, 0.6), (c, 6.0, 0.6)],
            paint,
        ))
    # Vents on the mound, a generator set to the side.
    for x in (-2.5, 2.5):
        g.add(frustum((x, -1.0, 3.3), (x, -1.0, 4.1), 0.25, 0.25, STEEL, n=6))
        g.add(frustum((x, -1.0, 4.1), (x, -1.0, 4.3), 0.4, 0.1, STEEL, n=6))
    g.add(block(7.0, 8.4, 0.5, 2.6, 0.0, 1.5, OLIVE))
    g.add(block(7.15, 8.25, 2.6, 2.65, 0.3, 1.2, OLIVE_DARK))
    return g


def antenna(paint=STEEL):
    """The mast on the mound: a lattice-like pole, the datalink dish and two whips."""
    g = Geo()
    g.add(frustum((0, 0, 0), (0, 0, 9.0), 0.16, 0.07, paint, n=6))
    for z in (2.5, 5.0, 7.5):
        g.add(block(-0.35, 0.35, -0.04, 0.04, z, z + 0.08, paint))
    # The datalink dish facing forward, and the whips either side.
    g.add(frustum((0, 0.1, 8.0), (0, 0.45, 8.0), 0.15, 0.6, OLIVE_DARK, n=10))
    for sx in (-1, 1):
        g.add(frustum((sx * 0.35, 0, 7.5), (sx * 0.35, 0, 10.5), 0.03, 0.015, paint, n=4))
    return g


def ramp(paint=STEEL, legs=OLIVE_DARK):
    """The pneumatic launch rail: a beam rising RAMP_RISE over RAMP_LEN on an A-frame, the
    cylinder under it and the cradle at its rear end (its origin)."""
    rail = Geo()
    rail.add(block(-0.12, 0.12, -0.3, RAMP_LEN, -0.22, 0.0, paint))
    for sx in (-1, 1):
        rail.add(block(sx * 0.2 - 0.04, sx * 0.2 + 0.04, -0.3, RAMP_LEN, -0.05, 0.06, paint))
    rail.add(block(-0.3, 0.3, 0.0, 0.8, 0.0, 0.12, OLIVE_DARK))
    rail.add(frustum((0, 0.3, -0.38), (0, RAMP_LEN - 1.2, -0.38), 0.13, 0.13, OLIVE, n=8))
    g = rail.rotated_x(RAMP_RISE).transformed(lambda p: p + mklib.v3((0, RAMP_Y, RAMP_Z)))
    # The A-frame at the front, a trestle at the back, the compressor beside it.
    top = RAMP_Z + (RAMP_LEN - 1.0) * math.sin(RAMP_RISE) - 0.2
    yf = RAMP_Y + (RAMP_LEN - 1.0) * math.cos(RAMP_RISE)
    for sx in (-1, 1):
        g.add(frustum((sx * 1.3, yf, 0.0), (sx * 0.1, yf, top), 0.07, 0.06, legs, n=4))
        g.add(frustum((sx * 0.9, RAMP_Y + 0.5, 0.0), (sx * 0.1, RAMP_Y + 0.5, RAMP_Z - 0.1), 0.07, 0.06, legs, n=4))
    g.add(block(-1.5, 1.5, yf - 0.05, yf + 0.05, 0.4, 0.5, legs))
    g.add(block(1.2, 2.2, RAMP_Y - 0.5, RAMP_Y + 0.9, 0.0, 0.9, OLIVE))
    return g


def drone_center():
    root, lod0, lod1 = levels(DRONES, 24000)
    g = bunker()
    g.add(antenna().transformed(lambda p: p + mklib.v3((0, -2.5, 3.4))))
    g.add(ramp())
    mesh_obj("aaa_116_lod0_s1v0", g, lod0)
    # Wrecked: the door blown in, the mast down across the mound, the rail fallen off its legs.
    w = bunker(BURNT, EARTH, BURNT)
    w.add(block(-1.1, 1.1, 3.7, 3.8, 0.0, 2.2, 0x00))
    w.add(antenna(BURNT).rotated_x(-1.35).transformed(lambda p: p + mklib.v3((0.5, -2.5, 3.6))))
    w.add(ramp(BURNT, BURNT).rotated_z(0.25).rotated_x(-0.12).transformed(lambda p: p + mklib.v3((0.8, -0.6, -0.6))))
    mesh_obj("aaa_116_lod0_s1v1", w, lod0)
    far(lod1, "aaa_116", [(-6.5, 6.5, -6.0, 3.6, 0.0, 2.6), (-0.4, 0.4, RAMP_Y, RAMP_Y + RAMP_LEN, 0.5, 2.6)])


# ---------------------------------------------------------------------------------- armour common


def moved(g, d):
    return g.transformed(lambda p: p + mklib.v3(d))


def rotated_y(g, a):
    """Roll about the y axis (positive tips +x down)."""
    c, s = math.cos(a), math.sin(a)
    return g.transformed(lambda p: mklib.v3((p.x * c + p.z * s, p.y, -p.x * s + p.z * c)))


def on_ground(g, x, y, sink=0.05):
    """`g` moved to (x, y), resting on the ground (sunk in a little)."""
    lo, _ = g.bounds()
    return moved(g, (x, y, -lo.z - sink))


def inset(poly, t):
    """A convex (y, z) polygon shrunk by `t` (each edge moved inwards)."""
    n = len(poly)
    area = sum(poly[i][0] * poly[(i + 1) % n][1] - poly[(i + 1) % n][0] * poly[i][1] for i in range(n))
    sgn = 1.0 if area > 0 else -1.0
    lines = []
    for i in range(n):
        (ay, az), (by, bz) = poly[i], poly[(i + 1) % n]
        dy, dz = by - ay, bz - az
        ln = math.hypot(dy, dz)
        ny, nz = -dz / ln * sgn, dy / ln * sgn
        lines.append(((ay + ny * t, az + nz * t), (dy, dz)))
    out = []
    for i in range(n):
        (p, d), (q, e) = lines[i - 1], lines[i]
        den = d[0] * e[1] - d[1] * e[0]
        if abs(den) < 1e-9:
            out.append(q)
            continue
        u = ((q[0] - p[0]) * e[1] - (q[1] - p[1]) * e[0]) / den
        out.append((p[0] + d[0] * u, p[1] + d[1] * u))
    return out


def track_belt(x, w, ys, r, front, rear, t=0.07, paint=RUBBER):
    """A track loop centred on x (width w): the bottom run under the road wheels at `ys` (radius
    r), round the front and rear wheels ((y, z, radius) each) and the top run between them."""
    pts = [(min(ys), 0.0), (max(ys), 0.0)]
    for (cy, cz, cr), angles in ((front, (-35, 25, 80)), (rear, (100, 155, 215))):
        pts += [(cy + (cr + t) * math.cos(math.radians(a)), cz + (cr + t) * math.sin(math.radians(a))) for a in angles]
    inner = inset(pts, t)
    g = Geo()
    x0, x1 = x - w / 2, x + w / 2
    for i in range(len(pts)):
        j = (i + 1) % len(pts)
        quad = [pts[i], pts[j], inner[j], inner[i]]
        g.add(mklib.hexa([(x0, y, z) for y, z in quad] + [(x1, y, z) for y, z in quad], paint))
    return g


def running_gear(x, w, ys, r, front, rear, paint, rollers=(), t=0.07, track=RUBBER, hub=STEEL):
    """Track, road wheels with their hubs, drive sprocket / idler and return rollers on one side."""
    sx = 1 if x > 0 else -1
    g = track_belt(x, w, ys, r, front, rear, t, track)
    for y in ys:
        g.add(wheel((x, y, r + t), r, w * 0.85, paint, n=8))
        g.add(frustum((x + sx * w * 0.42, y, r + t), (x + sx * (w * 0.42 + 0.05), y, r + t), r * 0.35, r * 0.3, hub, n=6))
    for cy, cz, cr in (front, rear):
        g.add(wheel((x, cy, cz), cr, w * 0.75, hub, n=8))
    (fy, fz, fr), (ry, rz, rr) = front, rear
    for y in rollers:
        top = rz + rr + (fz + fr - rz - rr) * (y - ry) / (fy - ry)
        g.add(wheel((x, y, top - 0.11), 0.11, w * 0.6, paint, n=6))
    return g


def plate(x0, x1, a, b, th, paint):
    """A plate of thickness `th` lying on the sloped line a-b ((y, z) points), across x0..x1."""
    (ya, za), (yb, zb) = a, b
    dy, dz = yb - ya, zb - za
    ln = math.hypot(dy, dz)
    ny, nz = -dz / ln, dy / ln
    if nz < 0:
        ny, nz = -ny, -nz
    q = [(ya, za), (yb, zb), (yb + ny * th, zb + nz * th), (ya + ny * th, za + nz * th)]
    return mklib.hexa([(x0, y, z) for y, z in q] + [(x1, y, z) for y, z in q], paint)


def plan_tube(plan, levels, paint):
    """A turret from a plan-view outline ((x, y) points) at levels (z, x scale, y scale, y shift)."""
    return mklib.tube([[(x * kx, y * ky + dy, z) for x, y in plan] for z, kx, ky, dy in levels], paint)


def mirror_half(half):
    """Full outline from its right half, front centre first, running clockwise to the rear."""
    return half + [(-x, y) for x, y in reversed(half[1:])]


def both_sides(f):
    """Geometry built by f(sx) for the right (sx=1) and the left (sx=-1) side."""
    g = Geo()
    for sx in (-1, 1):
        g.add(f(sx))
    return g


def x_span(sx, a, b):
    return (sx * a, sx * b) if sx > 0 else (sx * b, sx * a)


# ----------------------------------------------------------------------------------------- T-90A


def t90_glacis(y):
    return 1.0 + (3.43 - y) * 0.45 / 1.23


def t90_hull():
    """Hull 6.9 x 3.8 m: six road wheels a side under rubber-flapped skirts, the long flat glacis
    with its Kontakt-5 blocks, the fuel drums and unditching log on the rear."""
    g = Geo()
    g.add(mklib.side_prism([(-3.43, 0.6), (-3.25, 0.42), (2.85, 0.42), (3.43, 0.95), (-3.43, 0.95)], -1.12, 1.12, OLIVE))
    g.add(mklib.side_prism([(-3.43, 0.92), (3.43, 0.92), (3.43, 1.0), (2.2, 1.45), (-3.3, 1.45), (-3.43, 1.3)], -1.62, 1.62, OLIVE))

    def side(sx):
        s = Geo()
        s.add(running_gear(sx * 1.5, 0.58, [2.45 - k * 0.97 for k in range(6)], 0.37, (3.05, 0.55, 0.26), (-3.12, 0.55, 0.28), OLIVE_DARK))
        s.add(block(*x_span(sx, 1.62, 1.87), -3.35, 3.3, 0.92, 0.97, OLIVE))
        # Skirt: rubber panels with flaps along the bottom, armoured panels in front.
        s.add(block(*x_span(sx, 1.83, 1.87), -3.0, 1.9, 0.65, 0.95, OLIVE))
        for k in range(7):
            y0 = -3.0 + k * 0.7
            s.add(block(*x_span(sx, 1.84, 1.86), y0 + 0.04, y0 + 0.64, 0.5, 0.65, RUBBER))
        s.add(block(*x_span(sx, 1.81, 1.92), 1.9, 3.3, 0.52, 0.95, OLIVE))
        # Fender stowage: boxes on the right, the fuel tanks on the left.
        if sx > 0:
            s.add(block(1.62, 1.84, -2.7, -0.9, 0.97, 1.25, OLIVE_DARK))
        else:
            for y0 in (-2.9, -1.9, -0.9):
                s.add(block(-1.84, -1.62, y0, y0 + 0.9, 0.97, 1.32, OLIVE))
        # Glacis ERA either side of the driver's vision block.
        s.add(plate(*x_span(sx, 0.18, 1.4), (3.32, t90_glacis(3.32)), (2.35, t90_glacis(2.35)), 0.12, OLIVE_DARK))
        # Fuel drums across the rear plate.
        s.add(wheel((sx * 0.62, -3.66, 1.18), 0.28, 0.95, OLIVE_DARK, n=8))
        return s

    g.add(both_sides(side))
    g.add(wheel((0, -3.62, 1.6), 0.12, 3.1, KHAKI, n=6))
    g.add(block(-1.25, 1.25, -3.2, -1.6, 1.45, 1.5, OLIVE_DARK))
    g.add(block(-0.3, 0.3, 1.75, 2.15, 1.45, 1.53, OLIVE_DARK))
    g.add(block(-0.2, 0.2, 2.12, 2.17, 1.47, 1.52, GLASS))
    return g


def t90_turret():
    """The low, rounded welded turret (pivot at the ring's centre on the roof), Kontakt-5
    wedges and Shtora emitters on its front, the cupola right, the sight left, the stowage box and
    snorkel on the rear."""
    plan = mirror_half([(0.0, 1.35), (0.8, 1.3), (1.35, 0.85), (1.52, 0.1), (1.42, -0.7), (1.1, -1.25), (0.5, -1.5)])
    g = plan_tube(plan, [(0.0, 1.0, 1.0, 0.0), (0.38, 1.0, 1.0, 0.0), (0.6, 0.9, 0.92, -0.03), (0.75, 0.72, 0.75, -0.08)], OLIVE)

    def side(sx):
        s = Geo()
        # The ERA "brows": wedges sloping down to the front.
        s.add(mklib.hexa([(sx * 0.28, 1.3, 0.08), (sx * 1.3, 0.95, 0.08), (sx * 1.48, 1.45, 0.08), (sx * 0.28, 2.0, 0.08),
                          (sx * 0.28, 1.3, 0.7), (sx * 1.3, 0.95, 0.62), (sx * 1.48, 1.45, 0.36), (sx * 0.28, 2.0, 0.4)], OLIVE))
        # Shtora emitter boxes with their red lenses.
        s.add(block(*x_span(sx, 0.45, 0.9), 1.5, 1.92, 0.42, 0.74, OLIVE_DARK))
        s.add(block(*x_span(sx, 0.5, 0.85), 1.92, 1.95, 0.47, 0.7, LENS))
        # Smoke grenade tubes on the turret sides.
        for k in range(3):
            s.add(frustum((sx * 1.3, 0.4 - k * 0.22, 0.5), (sx * 1.5, 0.55 - k * 0.22, 0.7), 0.06, 0.06, OLIVE_DARK, n=6))
        return s

    g.add(both_sides(side))
    g.add(block(-0.98, -0.55, 0.55, 1.0, 0.68, 0.98, OLIVE_DARK))
    g.add(block(-0.93, -0.6, 1.0, 1.03, 0.75, 0.93, GLASS))
    g.add(frustum((-0.55, -0.35, 0.68), (-0.55, -0.35, 0.8), 0.32, 0.3, OLIVE_DARK, n=8))
    g.add(frustum((0.6, -0.2, 0.66), (0.6, -0.2, 0.9), 0.4, 0.34, OLIVE_DARK, n=8))
    g.add(barrel(-0.1, 1.0, 0.03, x=0.6, z=1.0))
    g.add(block(0.5, 0.7, -0.1, 0.3, 0.9, 1.05, OLIVE_DARK))
    g.add(block(-1.05, 1.05, -2.0, -1.4, 0.12, 0.55, OLIVE_DARK))
    g.add(frustum((-0.95, -1.75, 0.66), (0.95, -1.75, 0.66), 0.11, 0.11, OLIVE, n=6))
    return g


def t90_gun(length=5.1):
    """The 125 mm gun in its thermal sleeve (pivot at the trunnions)."""
    g = Geo()
    g.add(block(-0.26, 0.26, -0.3, 0.55, -0.24, 0.24, OLIVE_DARK))
    g.add(barrel(0.5, length, 0.1, paint=OLIVE))
    g.add(barrel(2.1, 2.8, 0.135, paint=OLIVE))
    for y in (1.3, 3.5, 4.3):
        if y < length - 0.2:
            g.add(barrel(y, y + 0.12, 0.12, paint=OLIVE_DARK))
    g.add(barrel(length - 0.15, length + 0.05, 0.085))
    return g


def t90():
    root, lod0, lod1 = levels(T90, 16000)
    intact = mesh_obj("aaa_117_lod0_s1v0", t90_hull(), lod0)
    turret = part("aaa_117_lod0_s1v0_part1", t90_turret(), intact, (0, 0.1, 1.45), TURRET_SLOT, 0)
    part("aaa_117_lod0_s1v0_part2", t90_gun(), turret, (0, 1.05, 0.38), GUN_SLOT, 1)
    # The wreck: the ammunition cooked off, the turret blown off and lying upside down beside the
    # hull with its gun snapped, the turret ring a black hole.
    w = t90_hull().recolour(BURNT)
    w.add(frustum((0, 0.1, 1.45), (0, 0.1, 1.48), 0.95, 0.95, HOLE, n=10))
    tur = t90_turret().add(moved(t90_gun(3.2).rotated_x(0.1), (0, 1.05, 0.38))).recolour(BURNT)
    w.add(on_ground(rotated_y(tur, 2.75).rotated_z(-2.2), 3.8, 0.2))
    mesh_obj("aaa_117_lod0_s1v1", w, lod0)
    far(lod1, "aaa_117", [(-1.87, 1.87, -3.45, 3.45, 0.1, 1.45), (-1.45, 1.45, -1.4, 2.0, 1.45, 2.2), (-0.1, 0.1, 2.0, 6.2, 1.73, 1.93)])


# ------------------------------------------------------------------------------- Leopard 2A6


def leopard2_hull():
    """Hull 7.7 x 3.7 m: seven road wheels and four return rollers a side behind the skirts
    (heavy armoured panels in front), the glacis, the long engine deck."""
    g = Geo()
    g.add(mklib.side_prism([(-3.86, 0.75), (-3.6, 0.52), (3.25, 0.52), (3.86, 1.02), (-3.86, 1.02)], -1.1, 1.1, NATO))
    g.add(mklib.side_prism([(-3.86, 1.0), (3.86, 1.0), (3.86, 1.22), (2.75, 1.75), (-3.72, 1.75), (-3.86, 1.55)], -1.62, 1.62, NATO))

    def side(sx):
        s = Geo()
        s.add(running_gear(sx * 1.5, 0.635, [2.75 - k * 0.88 for k in range(7)], 0.35, (3.3, 0.62, 0.3), (-3.35, 0.62, 0.32),
                           NATO_DARK, rollers=(2.2, 0.75, -0.7, -2.1)))
        s.add(block(*x_span(sx, 1.62, 1.88), -3.75, 3.6, 1.0, 1.04, NATO))
        s.add(block(*x_span(sx, 1.84, 1.88), -3.55, 1.8, 0.58, 1.0, NATO))
        for k in range(1, 5):
            y = -3.55 + k * 1.07
            s.add(block(*x_span(sx, 1.875, 1.9), y - 0.03, y + 0.03, 0.6, 1.0, NATO_DARK))
        s.add(block(*x_span(sx, 1.82, 1.95), 1.8, 3.6, 0.5, 1.0, NATO))
        s.add(block(*x_span(sx, 1.9, 1.96), 2.68, 2.72, 0.52, 0.98, NATO_DARK))
        # Exhaust grilles on the rear plate, headlight guards on the glacis corners.
        s.add(block(*x_span(sx, 0.85, 1.5), -3.9, -3.86, 1.05, 1.45, NATO_DARK))
        s.add(block(*x_span(sx, 1.25, 1.5), 3.25, 3.45, 1.4, 1.6, NATO_DARK))
        return s

    g.add(both_sides(side))
    g.add(block(-1.45, 1.45, -3.65, -2.0, 1.75, 1.8, NATO_DARK))
    g.add(block(-1.0, 1.0, -1.9, -1.2, 1.75, 1.79, NATO_DARK))
    g.add(block(-0.95, -0.4, 2.15, 2.6, 1.75, 1.82, NATO_DARK))
    g.add(block(-0.9, -0.45, 2.6, 2.63, 1.72, 1.8, GLASS))
    return g


def leopard2_turret():
    """Slab-sided turret with the arrowhead wedge armour in front and the long bustle (pivot at
    the ring's centre), the EMES sight and PERI periscope right, the loader's hatch left."""
    g = Geo()
    g.add(block(-1.62, 1.62, -1.45, 1.25, 0.0, 0.88, NATO, taper=0.08, taper_y=0.03))
    g.add(block(-1.5, 1.5, -3.0, -1.4, 0.1, 0.86, NATO, taper=0.06, taper_y=0.06))

    def side(sx):
        s = Geo()
        # The wedge: a pointed spaced-armour module running from the turret's front corner
        # forward and in to the mantlet, its top sloping down to the point.
        s.add(mklib.hexa([(sx * 0.4, 1.25, 0.1), (sx * 1.62, 1.25, 0.1), (sx * 0.8, 2.25, 0.1), (sx * 0.4, 2.45, 0.1),
                          (sx * 0.4, 1.25, 0.86), (sx * 1.56, 1.25, 0.84), (sx * 0.8, 2.25, 0.36), (sx * 0.4, 2.45, 0.34)], NATO))
        # Add-on side armour on the turret cheeks, and the smoke dischargers behind it.
        s.add(block(*x_span(sx, 1.58, 1.7), -0.3, 1.25, 0.12, 0.82, NATO))
        for k in range(4):
            s.add(frustum((sx * 1.55, -0.55 - k * 0.14, 0.62), (sx * 1.72, -0.45 - k * 0.14, 0.82), 0.055, 0.055, NATO_DARK, n=6))
        return s

    g.add(both_sides(side))
    # EMES 15 sight box with its window, the PERI R17 on its stalk, the hatches.
    g.add(block(0.6, 1.2, 0.5, 1.15, 0.86, 1.2, NATO_DARK))
    g.add(block(0.7, 1.1, 1.15, 1.18, 0.93, 1.12, GLASS))
    g.add(frustum((0.55, 0.0, 0.86), (0.55, 0.0, 1.3), 0.12, 0.12, NATO_DARK, n=6))
    g.add(block(0.4, 0.7, -0.15, 0.25, 1.3, 1.5, NATO_DARK))
    g.add(block(0.45, 0.65, 0.25, 0.28, 1.35, 1.45, GLASS))
    g.add(frustum((0.75, -0.75, 0.84), (0.75, -0.75, 0.94), 0.36, 0.33, NATO_DARK, n=8))
    g.add(frustum((-0.7, -0.65, 0.84), (-0.7, -0.65, 0.92), 0.35, 0.33, NATO_DARK, n=8))
    g.add(barrel(-0.4, 0.5, 0.03, x=-0.7, z=1.05))
    # Stowage basket on the bustle.
    g.add(block(-1.4, 1.4, -3.35, -3.0, 0.25, 0.78, NATO_DARK))
    return g


def leopard2_gun(length=5.5):
    """The 120 mm L55 with its slab mantlet, bore evacuator and muzzle reference (pivot at the trunnions)."""
    g = Geo()
    g.add(block(-0.38, 0.38, -0.25, 0.85, -0.32, 0.36, NATO_DARK))
    g.add(barrel(0.85, length, 0.085, paint=NATO))
    g.add(barrel(2.5, 3.1, 0.125, paint=NATO))
    for y in (1.8, 4.0):
        if y < length - 0.2:
            g.add(barrel(y, y + 0.1, 0.1, paint=NATO_DARK))
    g.add(barrel(length - 0.15, length + 0.02, 0.08))
    if length > 5:
        g.add(block(-0.04, 0.04, length - 0.35, length - 0.15, 0.08, 0.16, NATO_DARK))
    return g


def leopard2():
    root, lod0, lod1 = levels(LEOPARD2, 16000)
    intact = mesh_obj("aaa_118_lod0_s1v0", leopard2_hull(), lod0)
    turret = part("aaa_118_lod0_s1v0_part1", leopard2_turret(), intact, (0, 0.5, 1.75), TURRET_SLOT, 0)
    part("aaa_118_lod0_s1v0_part2", leopard2_gun(), turret, (0, 1.1, 0.45), GUN_SLOT, 1)
    # The wreck: burnt out, the turret knocked round on its ring and the gun drooping.
    w = leopard2_hull().recolour(BURNT)
    tur = leopard2_turret().add(moved(leopard2_gun().rotated_x(-0.22), (0, 1.1, 0.45))).recolour(BURNT)
    w.add(moved(rotated_y(tur, 0.06).rotated_z(0.7), (0.15, 0.4, 1.72)))
    mesh_obj("aaa_118_lod0_s1v1", w, lod0)
    far(lod1, "aaa_118", [(-1.88, 1.88, -3.86, 3.86, 0.1, 1.75), (-1.62, 1.62, -2.5, 2.9, 1.75, 2.63), (-0.1, 0.1, 2.9, 7.1, 2.1, 2.3)], NATO)


# --------------------------------------------------------------------------------- M2A2 Bradley


def bradley_glacis(y):
    return 1.25 + (3.28 - y) * 0.73 / 1.13


def bradley_hull():
    """Hull 6.5 x 3.6 m: six road wheels a side, the tall slab sides with their bolted-on
    armour, the sloped upper glacis with the trim vane, the rear ramp."""
    g = Geo()
    g.add(mklib.side_prism([(-3.28, 0.6), (-3.1, 0.45), (2.8, 0.45), (3.28, 1.0), (-3.28, 1.0)], -1.1, 1.1, NATO))
    g.add(mklib.side_prism([(-3.28, 0.97), (3.28, 0.97), (3.28, 1.25), (2.15, 1.98), (-3.28, 1.98)], -1.6, 1.6, NATO))

    def side(sx):
        s = Geo()
        s.add(running_gear(sx * 1.4, 0.53, [2.35 - k * 0.95 for k in range(6)], 0.3, (2.95, 0.62, 0.26), (-2.95, 0.58, 0.27),
                           NATO_DARK, rollers=(1.6, 0.1, -1.4)))
        # The spaced side armour: a tall slab from the roof edge down over the top of the track,
        # its bolt strips, and the rubber skirt along the bottom.
        s.add(block(*x_span(sx, 1.6, 1.68), -3.2, 2.7, 0.75, 1.9, NATO))
        for k in range(6):
            y = -3.0 + k * 1.1
            s.add(block(*x_span(sx, 1.68, 1.7), y - 0.04, y + 0.04, 0.8, 1.85, NATO_DARK))
        s.add(block(*x_span(sx, 1.62, 1.66), -3.2, 2.7, 0.6, 0.75, RUBBER))
        # Headlight guards on the nose.
        s.add(block(*x_span(sx, 1.2, 1.45), 3.28, 3.4, 1.0, 1.3, NATO_DARK))
        return s

    g.add(both_sides(side))
    # Trim vane folded on the glacis, the engine exhaust grille on the roof (engine front right).
    g.add(plate(-1.45, 0.9, (3.15, bradley_glacis(3.15)), (2.4, bradley_glacis(2.4)), 0.05, NATO_DARK))
    g.add(block(0.55, 1.4, 1.2, 2.1, 1.98, 2.03, NATO_DARK))
    # Driver's hatch with its periscopes (front left).
    g.add(frustum((-0.85, 1.75, 1.98), (-0.85, 1.75, 2.1), 0.3, 0.28, NATO_DARK, n=8))
    g.add(block(-1.15, -0.55, 2.0, 2.08, 1.98, 2.08, GLASS))
    # The troop compartment's cargo hatch, and the ramp with its door on the rear.
    g.add(block(-0.95, 0.95, -3.0, -1.7, 1.98, 2.04, NATO_DARK))
    g.add(block(-0.9, 0.9, -3.34, -3.28, 0.5, 1.85, NATO_DARK))
    g.add(block(-0.75, -0.1, -3.37, -3.34, 0.65, 1.7, NATO))
    return g


def bradley_turret():
    """The small angular two-man turret (pivot at the ring's centre), the TOW launcher box on its
    left, the ISU sight on the roof, smoke dischargers on the front corners."""
    g = Geo()
    g.add(mklib.hexa([(-1.0, -1.05, 0.0), (1.0, -1.05, 0.0), (1.0, 0.95, 0.0), (-1.0, 0.95, 0.0),
                      (-0.82, -0.95, 0.7), (0.82, -0.95, 0.7), (0.82, 0.5, 0.7), (-0.82, 0.5, 0.7)], NATO))
    g.add(block(-0.85, 0.85, -1.45, -1.0, 0.12, 0.62, NATO_DARK))
    # TOW launcher, stowed level along the left side on its arm, the two tube mouths in front.
    g.add(block(-1.62, -1.06, -0.75, 0.75, 0.2, 0.68, NATO))
    g.add(block(-1.12, -0.9, -0.3, 0.3, 0.3, 0.5, NATO_DARK))
    for x0 in (-1.56, -1.32):
        g.add(block(x0, x0 + 0.2, 0.75, 0.77, 0.3, 0.58, RUBBER))
    # ISU sight box, the commander's hatch (right), periscopes.
    g.add(block(-0.62, -0.18, 0.1, 0.6, 0.7, 1.0, NATO_DARK))
    g.add(block(-0.55, -0.25, 0.6, 0.62, 0.76, 0.94, GLASS))
    g.add(frustum((0.4, -0.45, 0.7), (0.4, -0.45, 0.82), 0.33, 0.3, NATO_DARK, n=8))
    g.add(frustum((-0.35, -0.55, 0.7), (-0.35, -0.55, 0.77), 0.28, 0.27, NATO_DARK, n=8))
    for k in range(4):
        g.add(frustum((0.85, 0.5 - k * 0.12, 0.35), (1.0, 0.62 - k * 0.12, 0.5), 0.05, 0.05, NATO_DARK, n=6))
    return g


def bradley_gun():
    """The 25 mm Bushmaster with its shroud and muzzle brake, the coaxial MG (pivot at the trunnions)."""
    g = Geo()
    g.add(block(-0.22, 0.22, -0.2, 0.42, -0.2, 0.22, NATO_DARK))
    g.add(barrel(0.4, 0.95, 0.07, paint=NATO_DARK))
    g.add(barrel(0.9, 2.65, 0.038))
    g.add(barrel(2.5, 2.72, 0.06))
    g.add(barrel(0.4, 0.75, 0.025, x=0.3, z=-0.05))
    return g


def bradley():
    root, lod0, lod1 = levels(BRADLEY, 16000)
    intact = mesh_obj("aaa_119_lod0_s1v0", bradley_hull(), lod0)
    turret = part("aaa_119_lod0_s1v0_part1", bradley_turret(), intact, (0.25, 0.35, 1.98), TURRET_SLOT, 0)
    part("aaa_119_lod0_s1v0_part2", bradley_gun(), turret, (0.1, 0.62, 0.38), GUN_SLOT, 1)
    # The wreck: burnt, the turret knocked askew, the gun drooping, the ramp down onto the ground.
    w = bradley_hull().recolour(BURNT)
    tur = bradley_turret().add(moved(bradley_gun().rotated_x(-0.2), (0.1, 0.62, 0.38))).recolour(BURNT)
    w.add(moved(rotated_y(tur, -0.08).rotated_z(-0.55), (0.3, 0.3, 1.97)))
    w.add(block(-0.85, 0.85, -3.36, -3.34, 0.55, 1.8, HOLE))
    w.add(plate(-0.9, 0.9, (-3.32, 0.5), (-4.95, 0.0), 0.08, BURNT))
    mesh_obj("aaa_119_lod0_s1v1", w, lod0)
    far(lod1, "aaa_119", [(-1.7, 1.7, -3.3, 3.3, 0.1, 1.98), (-1.37, 1.25, -1.1, 1.3, 1.98, 2.7)], NATO)


# ---------------------------------------------------------------------------------------- BMP-2

BMP_NOSE = (3.36, 0.88)
BMP_ROOF = (0.9, 1.72)


def bmp_glacis(y):
    (y1, z1), (y0, z0) = BMP_NOSE, BMP_ROOF
    return z0 + (y - y0) * (z1 - z0) / (y1 - y0)


def bmp_section(y, b, f, r, bw, sw, rw):
    """Hull cross-section at y: the lower hull (half width bw from b up to the fenders at f),
    the fenders out to sw and the upper hull's sides leaning in to the roof (half width rw at r)."""
    half = [(bw, b), (bw, f), (sw, f), (rw, r)]
    pts = [(-bw, b)] + half + [(-x, z) for x, z in reversed(half[1:])]
    return [(x, y, z) for x, z in pts]


def bmp2_hull():
    """Hull 6.7 x 3.15 m: the low pointed nose with its very flat ribbed glacis, the engine
    louvres front right, six road wheels and three return rollers a side (no skirts), the leaning
    upper sides, the troop hatches and the two rear doors."""
    g = mklib.tube([
        bmp_section(-3.36, 0.62, 0.95, 1.55, 1.05, 1.57, 1.18),
        bmp_section(-3.2, 0.45, 0.95, 1.66, 1.05, 1.57, 1.18),
        bmp_section(BMP_ROOF[0], 0.42, 0.95, BMP_ROOF[1], 1.05, 1.57, 1.2),
        bmp_section(2.6, 0.42, 0.95, bmp_glacis(2.6), 1.05, 1.57, 1.36),
        bmp_section(BMP_NOSE[0], 0.8, 0.85, BMP_NOSE[1], 1.05, 1.57, 1.45),
    ], OLIVE)

    def side(sx):
        s = Geo()
        s.add(running_gear(sx * 1.34, 0.3, [2.25 - k * 0.93 for k in range(6)], 0.35, (2.95, 0.62, 0.25), (-3.0, 0.58, 0.26),
                           OLIVE_DARK, rollers=(1.5, 0.0, -1.5)))
        # Rear doors (the fuel tanks), bulging out of the rear plate.
        s.add(block(*x_span(sx, 0.08, 0.75), -3.48, -3.36, 0.68, 1.45, OLIVE, taper_y=0.06))
        s.add(block(*x_span(sx, 0.38, 0.48), -3.5, -3.48, 1.15, 1.27, GLASS))
        # Troop hatches on the rear roof, two a side.
        for y0 in (-1.75, -2.85):
            s.add(block(*x_span(sx, 0.12, 0.78), y0, y0 + 0.85, 1.6, 1.7, OLIVE_DARK))
        return s

    g.add(both_sides(side))
    # Ribs across the glacis, the engine louvres right, the driver's hatch left.
    for y in (1.95, 2.3, 2.65, 3.0):
        g.add(plate(-1.3, 1.3, (y - 0.05, bmp_glacis(y - 0.05)), (y + 0.05, bmp_glacis(y + 0.05)), 0.06, OLIVE_DARK))
    g.add(plate(0.25, 1.15, (1.05, bmp_glacis(1.05)), (1.75, bmp_glacis(1.75)), 0.03, OLIVE_DARK))
    g.add(frustum((-0.7, 1.3, bmp_glacis(1.3) - 0.05), (-0.7, 1.3, bmp_glacis(1.3) + 0.08), 0.28, 0.26, OLIVE_DARK, n=8))
    g.add(block(-0.95, -0.45, 1.52, 1.56, bmp_glacis(1.55) - 0.02, bmp_glacis(1.55) + 0.08, GLASS))
    g.add(frustum((-0.7, 0.55, 1.68), (-0.7, 0.55, 1.78), 0.26, 0.25, OLIVE_DARK, n=8))
    return g


def bmp2_turret():
    """The small conical two-man turret (pivot at the ring's centre), the Konkurs launcher on its
    roof, the hatches and the gunner's sight."""
    plan = [(math.sin(2 * math.pi * k / 10), math.cos(2 * math.pi * k / 10)) for k in range(10)]
    g = plan_tube(plan, [(0.0, 1.0, 1.0, 0.0), (0.22, 1.0, 1.0, 0.0), (0.52, 0.74, 0.74, -0.05)], OLIVE)
    g.add(block(-0.3, 0.3, 0.75, 1.02, 0.1, 0.45, OLIVE))
    g.add(block(-0.72, -0.38, 0.15, 0.55, 0.45, 0.72, OLIVE_DARK))
    g.add(block(-0.68, -0.42, 0.55, 0.57, 0.5, 0.66, GLASS))
    for sx in (-1, 1):
        g.add(frustum((sx * 0.38, -0.3, 0.5), (sx * 0.38, -0.3, 0.58), 0.27, 0.25, OLIVE_DARK, n=8))
        for k in range(3):
            g.add(frustum((sx * 0.85, -0.3 - k * 0.15, 0.25), (sx * 1.0, -0.2 - k * 0.15, 0.42), 0.05, 0.05, OLIVE_DARK, n=6))
    # The Konkurs (AT-5) launch tube on its bracket.
    g.add(block(0.0, 0.18, -0.15, 0.15, 0.5, 0.78, OLIVE_DARK))
    g.add(frustum((0.09, -0.65, 0.87), (0.09, 0.6, 0.87), 0.09, 0.09, OLIVE_DARK, n=8))
    return g


def bmp2_gun(length=3.1):
    """The long thin 30 mm 2A42 with its recoil jacket and flash hider, the coaxial PKT (pivot at
    the trunnions)."""
    g = Geo()
    g.add(block(-0.24, 0.24, -0.2, 0.25, -0.18, 0.18, OLIVE_DARK))
    g.add(barrel(0.25, 0.9, 0.065, paint=OLIVE_DARK))
    g.add(barrel(0.9, length - 0.2, 0.033))
    g.add(barrel(length - 0.3, length, 0.05))
    g.add(barrel(0.25, 0.6, 0.022, x=0.17, z=-0.04))
    return g


def bmp2():
    root, lod0, lod1 = levels(BMP2, 16000)
    intact = mesh_obj("aaa_120_lod0_s1v0", bmp2_hull(), lod0)
    turret = part("aaa_120_lod0_s1v0_part1", bmp2_turret(), intact, (0, 0.1, 1.71), TURRET_SLOT, 0)
    part("aaa_120_lod0_s1v0_part2", bmp2_gun(), turret, (0, 0.85, 0.28), GUN_SLOT, 1)
    # The wreck: gutted, the turret blown off and lying on its side by the left track.
    w = bmp2_hull().recolour(BURNT)
    w.add(frustum((0, 0.1, 1.71), (0, 0.1, 1.74), 0.8, 0.8, HOLE, n=10))
    tur = bmp2_turret().add(moved(bmp2_gun().rotated_x(-0.3), (0, 0.85, 0.28))).recolour(BURNT)
    w.add(on_ground(rotated_y(tur, -1.9).rotated_z(0.5), -2.9, -0.6))
    mesh_obj("aaa_120_lod0_s1v1", w, lod0)
    far(lod1, "aaa_120", [(-1.57, 1.57, -3.36, 3.36, 0.1, 1.7), (-1.0, 1.0, -0.9, 1.1, 1.7, 2.3)])


# -------------------------------------------------------------------------------------- common


def levels(index, near, far_level=True):
    """The root, its detailed level (within `near` model units) and a far level (boxes)."""
    name = "aaa_%03d" % index
    root = empty(name)
    root["attr"] = 0x01
    dists = [near, 32769] if far_level else [near]
    root["lod_dists"] = dists
    lod0 = empty(name + "_lod0", root)
    lod0["max_dist"] = near
    lod0["radius"] = int(8 * K)
    lod1 = None
    if far_level:
        lod1 = empty(name + "_lod1", root)
        lod1["max_dist"] = 32769
        lod1["radius"] = int(8 * K)
    return root, lod0, lod1


def far(lod, name, boxes, colour=OLIVE):
    """Far level: a few boxes, olive (or `colour`), and black once destroyed."""
    for variant, paint in (("s1v0", colour), ("s1v1", BURNT)):
        g = Geo()
        for b in boxes:
            g.add(block(*b, paint))
        mesh_obj("%s_lod1_%s" % (name, variant), g, lod)


def main():
    out = sys.argv[sys.argv.index("--") + 1]
    mklib.load_palette(os.path.join(ROOT, "assets", "palettes", "tod_central_europe.ron"))
    for o in list(bpy.data.objects):
        bpy.data.objects.remove(o, do_unlink=True)
    zsu()
    s60()
    manpads()
    patriot_launcher()
    patriot_radar()
    gepard()
    drone_center()
    t90()
    leopard2()
    bradley()
    bmp2()
    bpy.ops.export_scene.gltf(
        filepath=out, export_format="GLB", export_extras=True, export_yup=True, export_texcoords=False,
        export_normals=False, export_materials="EXPORT", use_visible=False, export_animations=False,
        use_mesh_edges=True, use_mesh_vertices=True,
    )


main()
