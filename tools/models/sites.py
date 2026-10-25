"""Builds two ground sites into the theater masters that use them, and saves them:

    blender -b --python-exit-code 1 --python tools/models/sites.py
    tools/models/export.sh desert_storm korea cuba

- The Scud launcher (desert_storm_094): a MAZ-543 transporter-erector-launcher with its R-17
  missile in the travel position, parked in a U-shaped sand revetment open to the front, a Ural
  support truck beside it, on a gravel hard-standing. Destroyed: the burnt-out TEL with the
  missile broken off its erector, the truck's hulk and scorched ground.
- The supply depot (korea_028, cuba_028): a vaulted warehouse with a loading dock, a flat-roofed
  vehicle shed, a small office, stacked crates and two trucks in a fenced concrete yard.
  Destroyed: the buildings fallen in to rubble on scorched concrete.

Each root keeps its name, `attr` and `lod_dists`. The finest level holds the ground (always
drawn), `_s1v0` the intact site, `_s1v1` the destroyed one, `_s10v1` its lights at night and
`_detail` the fence and tyre tracks drawn as lines; the next level is a copy of it without the
lines (tools/models/lower_lods.py); the depot has a block outline after that; the last level is
a dot. The collision hull is the convex hull of the main vehicle or building.

Model space: x east, y north (the TEL's nose), z up, in world units (the sites are placed on the
finest terrain level); built in metres.
"""

import math
import os
import random
import sys

import bmesh
import bpy
from mathutils import Vector

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.abspath(os.path.join(HERE, "..", ".."))
sys.path.insert(0, HERE)

import heli  # noqa: E402
import lower_lods  # noqa: E402
import mklib  # noqa: E402
from mklib import Geo, Ramp, block, frustum, hexa, wheel  # noqa: E402

K = 1_048_576 / 512_000.0  # world units per metre

VEHICLE = Ramp(0xE5, 0xEC)
MISSILE = Ramp(0xE1, 0xE6)
TYRE = Ramp(0xEC, 0xEF)
CANVAS = Ramp(0xE7, 0xED)
BURNT = Ramp(0xEC, 0xEF)
SAND = Ramp(0xC1, 0xC5)
GRAVEL = 0xE6
CONCRETE = 0xE4
WALL = Ramp(0xE3, 0xEA)
ROOF = Ramp(0xE5, 0xED)
DOOR = Ramp(0xEA, 0xEE)
CRATE = Ramp(0xE6, 0xEC)
GLASS = 0x9A
SCORCH = 0xEF
FIRE = 0x0D  # the burning patch every destroyed site shows
FENCE = 0xEB
TRACKS = 0xE9
LIGHT = 0x94
DOT = 0xBF


def flat(x0, x1, y0, y1, z, c):
    g = Geo()
    g.face([(x0, y0, z), (x1, y0, z), (x1, y1, z), (x0, y1, z)], c)
    return g


def blob(cx, cy, r, z, c, rng, n=9):
    g = Geo()
    pts = []
    for k in range(n):
        a = 2 * math.pi * k / n
        rr = r * rng.uniform(0.7, 1.15)
        pts.append((cx + rr * math.cos(a), cy + rr * math.sin(a), z))
    g.face(pts, c)
    return g


def placed(g, x, y, heading=0.0, tilt=0.0, z=0.0):
    """`g` turned by `heading` (degrees, clockwise from north), pitched by `tilt` and moved."""
    g = g.rotated_x(math.radians(tilt)) if tilt else g
    return g.rotated_z(-math.radians(heading)).scaled(1.0, (x, y, z))


# ---------------------------------------------------------------------------------------------
# Scud launcher site

def wheels(axles, half_track, r, w):
    g = Geo()
    for y in axles:
        for sx in (-1, 1):
            g.add(wheel((sx * half_track, y, r), r, w, TYRE, n=8))
    return g


def maz543(paint=VEHICLE, burnt=False):
    """The 8x8 MAZ-543 TEL, 13.4 m long and 3 m wide, nose to +y: two cabs either side of the
    missile's nose, the erector along the deck, the launch table at the back."""
    g = Geo()
    tyre = BURNT if burnt else TYRE
    for y in (5.0, 2.6, -1.7, -4.1):
        for sx in (-1, 1):
            g.add(wheel((sx * 1.15, y, 0.78), 0.78, 0.55, tyre, n=8))
    g.add(block(-1.05, 1.05, -6.5, 6.4, 0.75, 1.45, paint))
    # Mudguards and side lockers.
    for sx in (-1, 1):
        x0, x1 = sorted((sx * 0.9, sx * 1.52))
        g.add(block(x0, x1, -5.0, 3.6, 1.45, 1.95, paint))
        g.add(block(x0, x1, 3.6, 6.4, 1.45, 1.75, paint))
    # The two cabs, glazed at the front.
    for sx in (-1, 1):
        x0, x1 = sorted((sx * 0.5, sx * 1.52))
        g.add(block(x0, x1, 4.6, 6.6, 1.45, 3.0, paint, taper=0.08, taper_y=0.15))
        if not burnt:
            c = ((x0 + x1) / 2, 5.6, 2.2)
            g.face([(x0 + 0.12, 6.62, 2.2), (x1 - 0.12, 6.62, 2.2), (x1 - 0.18, 6.5, 2.85), (x0 + 0.18, 6.5, 2.85)], GLASS, c)
            xs = x1 + 0.01 if sx > 0 else x0 - 0.01
            g.face([(xs, 5.2, 2.2), (xs, 6.3, 2.2), (xs, 6.2, 2.8), (xs, 5.3, 2.8)], GLASS, c)
    # Equipment boxes behind the cabs, the erector beam and the launch table.
    g.add(block(-1.5, -0.55, 2.0, 4.4, 1.45, 2.5, paint))
    g.add(block(0.55, 1.5, 2.0, 4.4, 1.45, 2.4, paint))
    g.add(block(-0.35, 0.35, -6.2, 3.8, 1.45, 1.95, paint))
    g.add(block(-1.0, 1.0, -6.6, -5.6, 1.45, 2.3, paint))
    return g


def r17(burnt=False):
    """The R-17 missile, 11.2 m long, 0.88 m across, lying on the erector, nose forward."""
    paint = BURNT if burnt else MISSILE
    z = 2.45
    g = frustum((0, -5.9, z), (0, 3.6, z), 0.44, 0.44, paint, n=8)
    g.add(frustum((0, 3.6, z), (0, 5.3, z), 0.44, 0.0, paint, n=8, caps=False))
    for k in range(4):
        a = math.radians(45 + 90 * k)
        d = Vector((math.cos(a), 0, math.sin(a)))
        root0, root1 = Vector((0, -5.8, z)) + d * 0.42, Vector((0, -4.9, z)) + d * 0.42
        tip = Vector((0, -5.9, z)) + d * 0.95
        g.face2([root0, root1, tip], paint)
    return g


def ural(paint=VEHICLE, burnt=False):
    """A Ural-375 support truck, 7.4 m long, nose to +y."""
    g = Geo()
    tyre = BURNT if burnt else TYRE
    for y in (2.5, -1.3, -2.7):
        for sx in (-1, 1):
            g.add(wheel((sx * 1.0, y, 0.55), 0.55, 0.4, tyre, n=8))
    g.add(block(-0.8, 0.8, -3.6, 3.6, 0.6, 1.1, paint))
    g.add(block(-1.1, 1.1, 2.4, 3.7, 1.1, 1.65, paint, taper_y=0.2))
    g.add(block(-1.15, 1.15, 0.9, 2.4, 1.1, 2.6, paint, taper=0.05, taper_y=0.08))
    if not burnt:
        g.face([(-1.0, 2.41, 1.95), (1.0, 2.41, 1.95), (0.95, 2.36, 2.45), (-0.95, 2.36, 2.45)], GLASS, (0, 1.6, 1.8))
        g.add(block(-1.25, 1.25, -3.7, 0.8, 1.1, 2.9, CANVAS, taper=0.12))
    else:
        g.add(block(-1.2, 1.2, -3.7, 0.8, 1.1, 1.4, paint))
    return g


def berm(x0, x1, y0, y1, h, crest):
    """A sand wall filling the box, sloping up to a crest `crest` wide along its longer side."""
    inset = (min(x1 - x0, y1 - y0) - crest) / 2
    top = [(x0 + inset, y0 + inset), (x1 - inset, y0 + inset), (x1 - inset, y1 - inset), (x0 + inset, y1 - inset)]
    return hexa([(x0, y0, 0), (x1, y0, 0), (x1, y1, 0), (x0, y1, 0)] + [(x, y, h) for x, y in top], SAND)


def revetment():
    g = Geo()
    g.add(berm(-9.0, -3.6, -14.0, 9.0, 2.4, 0.8))
    g.add(berm(3.6, 9.0, -14.0, 9.0, 2.4, 0.8))
    g.add(berm(-9.0, 9.0, -15.5, -10.0, 2.4, 0.8))
    return g


def scud_ground():
    g = flat(-15, 15, -30, 30, 0.05, GRAVEL)
    return g


def scud_tracks():
    g = Geo()
    for sx in (-1.15, 1.15):
        g.line((sx, 9.0, 0.08), (sx + 1.0, 30.0, 0.08), TRACKS)
    for sx in (8.0, 10.0):
        g.line((sx, 2.0, 0.08), (sx + 3.0, 30.0, 0.08), TRACKS)
    return g


TEL_AT = (0.0, -3.0)
TRUCK_AT = (11.0, -6.0, 12.0)


def scud_intact():
    g = Geo()
    g.add(revetment())
    g.add(placed(maz543().add(r17()), *TEL_AT))
    g.add(placed(ural(), TRUCK_AT[0], TRUCK_AT[1], TRUCK_AT[2]))
    return g


def scud_destroyed():
    rng = random.Random(94)
    g = Geo()
    g.add(revetment())
    g.add(blob(0, -3, 7.5, 0.1, SCORCH, rng, 11))
    g.add(blob(11, -6, 4.5, 0.1, SCORCH, rng))
    g.add(blob(0, -1, 3.0, 0.15, FIRE, rng))
    hull = maz543(BURNT, burnt=True)
    g.add(placed(hull, TEL_AT[0], TEL_AT[1], 4.0, 2.0, -0.45))
    # The missile broken in two, one half fallen off to the left.
    m = r17(burnt=True)
    rear = Geo()
    rear.faces = [f for f in m.faces if min(p.y for p in f[0]) < -0.5]
    front = Geo()
    front.faces = [f for f in m.faces if min(p.y for p in f[0]) >= -0.5]
    g.add(placed(rear, TEL_AT[0] + 0.2, TEL_AT[1], 6.0, 0.0, -0.2))
    g.add(placed(front, TEL_AT[0] - 2.8, TEL_AT[1] + 2.0, -25.0, -4.0, -2.0))
    g.add(placed(ural(BURNT, burnt=True), TRUCK_AT[0], TRUCK_AT[1], TRUCK_AT[2] + 8.0, 0.0, -0.3))
    return g


def scud_lights():
    tx, ty = TEL_AT
    return [(tx - 1.0, ty + 6.65, 1.6), (tx + 1.0, ty + 6.65, 1.6), (tx, ty - 6.7, 1.2),
            (TRUCK_AT[0] - 0.6, TRUCK_AT[1] + 3.6, 1.3)]


# ---------------------------------------------------------------------------------------------
# Supply depot

YARD = (-30.0, 30.0, -30.0, 73.0)
STORE = (-26.0, -5.0, -12.0, 60.0, 7.0, 4.0)  # x0, x1, y0, y1, wall height, vault rise
SHED = (6.0, 27.0, 40.0, 66.0, 6.5)
OFFICE = (12.0, 26.0, -25.0, -15.0, 4.2)


def vault(x0, x1, y0, y1, h, rise, n=6, wall=WALL, roof=ROOF):
    """A barrel-vaulted building along y."""
    cx, hw = (x0 + x1) / 2, (x1 - x0) / 2
    arc = [(cx - hw * math.cos(math.pi * k / n), h + rise * math.sin(math.pi * k / n)) for k in range(n + 1)]
    g = Geo()
    c = (cx, (y0 + y1) / 2, h / 2)
    g.face([(x0, y0, 0), (x0, y1, 0), (x0, y1, h), (x0, y0, h)], wall, c)
    g.face([(x1, y0, 0), (x1, y1, 0), (x1, y1, h), (x1, y0, h)], wall, c)
    for y in (y0, y1):
        g.face([(x0, y, 0), (x1, y, 0)] + [(x, y, z) for x, z in reversed(arc)], wall, c)
    for (xa, za), (xb, zb) in zip(arc, arc[1:]):
        g.face([(xa, y0, za), (xb, y0, zb), (xb, y1, zb), (xa, y1, za)], roof, c)
    return g


def door(x0, x1, y, z1, face_y):
    """A door panel on a wall at `y`, facing `face_y` (+1 / -1)."""
    g = Geo()
    yy = y + 0.05 * face_y
    pts = [(x0, yy, 0.0), (x1, yy, 0.0), (x1, yy, z1), (x0, yy, z1)]
    g.face(pts if face_y < 0 else pts[::-1], DOOR)
    return g


def side_door(x, y0, y1, z1, face_x):
    g = Geo()
    xx = x + 0.05 * face_x
    pts = [(xx, y0, 0.0), (xx, y1, 0.0), (xx, y1, z1), (xx, y0, z1)]
    g.face(pts if face_x > 0 else pts[::-1], DOOR)
    return g


def windows(x0, x1, y, z0, z1, n, face_y):
    g = Geo()
    step = (x1 - x0) / n
    yy = y + 0.05 * face_y
    for k in range(n):
        a, b = x0 + step * (k + 0.25), x0 + step * (k + 0.75)
        pts = [(a, yy, z0), (b, yy, z0), (b, yy, z1), (a, yy, z1)]
        g.face(pts if face_y < 0 else pts[::-1], GLASS)
    return g


def crates(x0, y0, rows, cols, rng):
    g = Geo()
    for i in range(rows):
        for j in range(cols):
            x, y = x0 + j * 4.2, y0 + i * 3.4
            tiers = rng.choice((1, 2, 2, 3))
            g.add(block(x, x + 3.2, y, y + 2.4, 0.0, 1.3 * tiers, CRATE))
    return g


def depot_ground():
    x0, x1, y0, y1 = YARD
    return flat(x0, x1, y0, y1, 0.05, CONCRETE)


def depot_fence():
    x0, x1, y0, y1 = YARD
    g = Geo()
    corners = [(x0, y0), (x1, y0), (x1, y1), (x0, y1)]
    for (ax, ay), (bx, by) in zip(corners, corners[1:] + corners[:1]):
        if ay == by == y0:
            # The gate in the south fence.
            g.line((ax, ay, 2.2), (-5.0, ay, 2.2), FENCE)
            g.line((5.0, ay, 2.2), (bx, by, 2.2), FENCE)
            segs = [(ax, -5.0), (5.0, bx)]
        else:
            g.line((ax, ay, 2.2), (bx, by, 2.2), FENCE)
            segs = [(0.0, 1.0)]
        length = math.hypot(bx - ax, by - ay)
        n = int(length // 8)
        for k in range(n + 1):
            t = k / n
            px, py = ax + (bx - ax) * t, ay + (by - ay) * t
            if ay == by == y0 and -5.0 < px < 5.0:
                continue
            g.line((px, py, 0.0), (px, py, 2.2), FENCE)
    for x in (-5.0, 5.0):
        g.line((x, y0, 0.0), (x, y0, 2.6), FENCE)
    return g


def depot_intact():
    rng = random.Random(28)
    g = Geo()
    x0, x1, y0, y1, h, rise = STORE
    g.add(vault(x0, x1, y0, y1, h, rise))
    g.add(door(x0 + 6.0, x1 - 6.0, y0, 5.0, -1))
    for y in (6.0, 22.0, 38.0):
        g.add(side_door(x1, y, y + 6.0, 4.5, 1))
    g.add(block(x1, x1 + 3.0, 2.0, 46.0, 0.0, 1.2, CONCRETE))
    sx0, sx1, sy0, sy1, sh = SHED
    g.add(block(sx0, sx1, sy0, sy1, 0.0, sh, WALL))
    g.add(block(sx0 - 0.3, sx1 + 0.3, sy0 - 0.3, sy1 + 0.3, sh, sh + 0.5, ROOF))
    for k in range(3):
        y = sy0 + 2.0 + k * 8.0
        g.add(side_door(sx0, y, y + 6.0, 4.6, -1))
    ox0, ox1, oy0, oy1, oh = OFFICE
    g.add(block(ox0, ox1, oy0, oy1, 0.0, oh, WALL))
    g.add(block(ox0 - 0.4, ox1 + 0.4, oy0 - 0.4, oy1 + 0.4, oh, oh + 0.4, ROOF))
    g.add(windows(ox0, ox1, oy0, 1.4, 2.8, 4, -1))
    g.add(windows(ox0, ox1, oy1, 1.4, 2.8, 4, 1))
    g.add(crates(8.0, -6.0, 6, 4, rng))
    g.add(placed(ural(), 1.0, 20.0, 180.0))
    g.add(placed(ural(), -1.5, -18.0, 15.0))
    return g


def rubble(x0, x1, y0, y1, hmax, rng, n):
    g = Geo()
    for _ in range(n):
        w, d = rng.uniform(2.0, 6.0), rng.uniform(2.0, 6.0)
        x, y = rng.uniform(x0, x1 - w), rng.uniform(y0, y1 - d)
        h = rng.uniform(0.4, hmax)
        g.add(block(x, x + w, y, y + d, 0.0, h, BURNT, taper=min(w, d) * 0.3, taper_y=min(w, d) * 0.3))
    return g


def depot_destroyed():
    rng = random.Random(29)
    g = Geo()
    x0, x1, y0, y1, h, rise = STORE
    g.add(blob((x0 + x1) / 2, (y0 + y1) / 2, 30.0, 0.1, SCORCH, rng, 13))
    g.add(blob((x0 + x1) / 2, 30.0, 6.0, 0.15, FIRE, rng))
    g.add(rubble(x0, x1, y0, y1, 2.8, rng, 22))
    # The vault's end walls still standing, broken off.
    for y in (y0, y1 - 0.4):
        g.add(block(x0, x0 + 5.0, y, y + 0.4, 0.0, 5.5, BURNT))
        g.add(block(x1 - 7.0, x1, y, y + 0.4, 0.0, 3.5, BURNT))
    sx0, sx1, sy0, sy1, sh = SHED
    g.add(blob((sx0 + sx1) / 2, (sy0 + sy1) / 2, 15.0, 0.1, SCORCH, rng))
    g.add(rubble(sx0, sx1, sy0, sy1, 2.0, rng, 10))
    g.add(block(sx1 - 0.5, sx1, sy0, sy1, 0.0, 4.0, BURNT))
    ox0, ox1, oy0, oy1, oh = OFFICE
    g.add(rubble(ox0, ox1, oy0, oy1, 1.5, rng, 5))
    g.add(blob(17.0, 8.0, 12.0, 0.1, SCORCH, rng))
    g.add(placed(ural(BURNT, burnt=True), 1.0, 20.0, 160.0, 0.0, -0.3))
    return g


def depot_outline():
    """The far level: the buildings as plain blocks on the yard."""
    g = depot_ground()
    x0, x1, y0, y1, h, rise = STORE
    g.add(vault(x0, x1, y0, y1, h, rise, n=2))
    sx0, sx1, sy0, sy1, sh = SHED
    g.add(block(sx0, sx1, sy0, sy1, 0.0, sh, WALL))
    return g


def depot_lights():
    x0, x1, y0, y1 = YARD
    pts = [(x, y, 5.0) for x in (x0, x1) for y in (y0, (y0 + y1) / 2, y1)]
    pts += [(-5.0, y0, 2.8), (5.0, y0, 2.8), (STORE[1] + 0.3, 20.0, 5.0), (SHED[0] - 0.3, 52.0, 5.2)]
    return pts


# ---------------------------------------------------------------------------------------------

def hull_planes(g):
    """Collision planes of `g` (metres) in the exported frame, model units: [nx, ny, nz, d] with
    a Q14 normal, inside where n.p + d < 0."""
    bm = bmesh.new()
    for pts, _ in g.faces:
        for p in pts:
            q = p * K
            bm.verts.new((q.x, q.z, -q.y))
    bmesh.ops.convex_hull(bm, input=bm.verts)
    bmesh.ops.dissolve_limit(bm, angle_limit=math.radians(0.5), verts=bm.verts, edges=bm.edges)
    out, seen = [], set()
    for f in bm.faces:
        n = f.normal.normalized()
        d = -n.dot(f.verts[0].co)
        key = (round(n.x * 16384), round(n.y * 16384), round(n.z * 16384), round(d * 16384))
        if key not in seen:
            seen.add(key)
            out.append(list(key))
    bm.free()
    return out


def group(lod, suffix, conds, geo):
    """A state node below `lod` holding `geo` (metres)."""
    name = f"{lod.name}_{suffix}"
    ob = heli.mesh_with_lines(lod, name, geo.scaled(K))
    ob["conds"] = conds
    return ob


def build(name, site):
    root = bpy.data.objects[name]
    dists = list(root["lod_dists"])
    for c in list(root.children):
        mklib.delete_tree(c)
    lod0 = heli.mesh_with_lines(root, f"{name}_lod0", site["ground"].scaled(K))
    lod0["max_dist"] = dists[0]
    lod0["planes"] = hull_planes(site["hull"])
    lod0["radius"] = heli.radius(site["intact"].scaled(K), site["ground"].scaled(K))
    detail = bpy.data.objects.new(f"{lod0.name}_detail", None)
    for col in lod0.users_collection:
        col.objects.link(detail)
    detail.parent = lod0
    for c, segs in sorted(site["lines"].scaled(K).lines.items()):
        ln = f"{detail.name}_lines_pal_{c:03}"
        mklib.link_child(detail, ln, mklib.lines_mesh(ln, segs, c))
    group(lod0, "s1v0", [[1, 0]], site["intact"])
    group(lod0, "s1v1", [[1, 1]], site["destroyed"])
    heli.lights(lod0, 10, {LIGHT: [mklib.v3(p) * K for p in site["lights"]]})
    # The next level copies the finest one (lower_lods keeps its switch distance).
    near = bpy.data.objects.new(f"{name}_lod1", None)
    for col in lod0.users_collection:
        col.objects.link(near)
    near.parent = root
    near["max_dist"] = dists[1]
    lower_lods.rebuild(root, 1)
    k = 2
    if site.get("outline") is not None and len(dists) > 3:
        lod = heli.mesh_with_lines(root, f"{name}_lod{k}", site["outline"].scaled(K))
        lod["max_dist"] = dists[k]
        lod["planes"] = []
        lod["radius"] = heli.radius(site["outline"].scaled(K))
        k += 1
    last = bpy.data.objects.new(f"{name}_lod{k}", None)
    for col in lod0.users_collection:
        col.objects.link(last)
    last.parent = root
    last["max_dist"] = dists[k]
    last["planes"] = []
    last["radius"] = 0
    dn = f"{last.name}_dots_pal_{DOT:03}"
    mklib.link_child(last, dn, mklib.points_mesh(dn, [(0, 0, 0)], DOT))
    root["lod_dists"] = dists[:k + 1]
    print(f"{name}: {len(site['intact'].faces)} faces intact, {len(site['destroyed'].faces)} destroyed, "
          f"{len(lod0['planes'])} planes, radius {lod0['radius']}, {k + 1} levels")


SCUD = {"ground": scud_ground, "lines": scud_tracks, "intact": scud_intact, "destroyed": scud_destroyed,
        "lights": scud_lights, "hull": lambda: placed(maz543().add(r17()), *TEL_AT)}
DEPOT = {"ground": depot_ground, "lines": depot_fence, "intact": depot_intact, "destroyed": depot_destroyed,
         "lights": depot_lights, "hull": lambda: vault(*STORE), "outline": depot_outline}
SITES = {"desert_storm": {"desert_storm_094": SCUD}, "korea": {"korea_028": DEPOT}, "cuba": {"cuba_028": DEPOT}}


def main():
    only = sys.argv[sys.argv.index("--") + 1:] if "--" in sys.argv else []
    for blend, sites in SITES.items():
        if only and blend not in only:
            continue
        mklib.load_palette(os.path.join(ROOT, "assets", "palettes", f"tod_{blend}.ron"))
        bpy.ops.wm.open_mainfile(filepath=os.path.join(ROOT, "assets-src", "blend", f"{blend}.blend"))
        for name, site in sites.items():
            build(name, {k: (v() if callable(v) else v) for k, v in site.items()})
        bpy.ops.wm.save_mainfile()


main()
