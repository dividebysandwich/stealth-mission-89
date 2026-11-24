"""Builds the F-117A Nighthawk (`aircraft_024`) in aircraft.blend and saves it:

    blender -b assets-src/blend/aircraft.blend --python-exit-code 1 --python tools/models/f117.py
    tools/models/export.sh aircraft

The airframe is the contributed model's skin (assets-src/blend/f117a.blend, which
tools/models/f117_source.py makes of the Sketchfab "F-117 Nighthawk (War Thunder)" by Köjf,
CC BY 4.0): its facets in a thousand triangles, the openings closed. This builder paints it
(the game's greys by each facet's slope, the underside one colour, the exhaust slots glowing),
lays the canopy's panes, the intake grids and the two infrared turrets' windows over it, draws
the air data probes, and cuts the two weapon bays out of the underside.

Children of the one detail level:
- `_s8v0` / `_s8v1` / `_s8v2`: the two weapon bays' doors shut (the pieces cut out of the
  underside), half open, and open with the bay inside and a GBU-27 in each (state slot 8, the
  bay door animation). Each bay has one door, hinged on the keel between them.
- `_s5v1`: the landing gear down (slot 5), under the source's gear doors.
- `_s12v1` navigation lights on the wingtips, `_s20v1` the red beacons above and below.

In metres (x right, y forward, z up; the origin under the underside, 10.6 m behind the nose's
tip), scaled to aircraft units (K per metre). The wheels touch the ground 1.25 m below the origin
(`F117_LIFT` in src/aircraft.rs).
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

NAME = "aircraft_024"
SOURCE = os.path.join(ROOT, "assets-src", "blend", "f117a.blend")
# Unused variants of the F-117 left over in the file.
OBSOLETE = ["aircraft_031", "aircraft_033"]
K = 16.384
MAX_DIST = 32769

BODY = Ramp(152, 159)
DARK = Ramp(155, 159)
GLASS = Ramp(144, 145)
BAY = Ramp(157, 159)
METAL = Ramp(0xE8, 0xEC)
BOMB = Ramp(0xA8, 0xAE)
EXHAUST = 13
TYRE = 0xEF
SEAM = 159
UNDER = 158  # the underside, and its panel lines
UNDER_SEAM = 154
PROBE = 154
GRID = 156  # the intake grids, their bars BODY.lo
WINDOW = 159  # the infrared turrets' screens
RED, GREEN = 147, 151

NOSE = 10.6  # y of the nose tip; points are given aft of it (s)
GROUND = -1.25  # z of the ground under the wheels
# A facet this steep downwards is underside.
UNDER_Z = -0.3
# Overlays (panes, grids) lie this far off the skin.
LIFT = 0.012

# Weapon bays: either side of the keel, each door hinged at the keel (the source's doors: x
# 0.02 to 0.94 m, 6.24 to 11.16 m aft of the nose).
BAY_X = (0.04, 0.93)
BAY_S = (6.30, 11.10)
BAY_DEPTH = 0.8
# Landing gear: nose wheel and main wheels (x, s), wheel radius and width; under the source's
# gear doors, where the legs stand when down (they retract forward).
NOSE_GEAR = (0.0, 4.7, 0.30, 0.18)
MAIN_GEAR = (2.16, 10.3, 0.40, 0.26)
# The exhaust slots: the facets facing aft and outwards between these stations, outside this x,
# up to EXHAUST_H over their lower edge.
EXHAUST_S = (14.6, 16.9)
EXHAUST_X = 0.55
EXHAUST_H = 0.16
# Air data probes: (x, s of the root, s of the tip), z.
PROBES = ((0.0, 0.05, -0.30), (0.26, 0.60, -0.07), (0.478, 1.10, 0.43))
PROBE_Z = 0.29


def P(x, s, z):
    return Vector((x, NOSE - s, z))


def source(name):
    """The polygons (lists of points) of an object of the source file."""
    with bpy.data.libraries.load(SOURCE) as (src, dst):
        dst.objects = [name]
    ob = dst.objects[0]
    me = ob.data
    polys = [[me.vertices[i].co.copy() for i in p.vertices] for p in me.polygons]
    bpy.data.objects.remove(ob)
    bpy.data.meshes.remove(me)
    return polys


def in_bay():
    """> 0 inside a bay's opening, seen from below: the four edges' functions."""
    (x0, x1), (s0, s1) = BAY_X, BAY_S
    return [lambda q: abs(q.x) - x0, lambda q: x1 - abs(q.x), lambda q: (NOSE - q.y) - s0, lambda q: s1 - (NOSE - q.y)]


def split_bays(poly):
    """An underside polygon as (the pieces over the bays, the rest). The keel's line splits it
    first, so that each piece lies on one side."""
    doors, rest = [], []
    for side in (True, False):
        piece = mklib.clip(poly, lambda q: q.x, side)
        if not piece:
            continue
        inside = piece
        for f in in_bay():
            out = mklib.clip(inside, f, False)
            if out:
                rest.append(out)
            inside = mklib.clip(inside, f, True)
            if not inside:
                break
        if inside:
            doors.append(inside)
    return doors, rest


def airframe():
    """(the skin, open over the bays; the bays' shut doors: the pieces cut out of it)."""
    g, doors = Geo(), Geo()
    (x0, x1), (s0, s1) = BAY_X, BAY_S
    for poly in source("airframe"):
        n = mklib.normal(poly)
        s = [NOSE - p.y for p in poly]
        if n.z < UNDER_Z:
            c = sum(poly, Vector()) / len(poly)
            near = max(abs(p.x) for p in poly) > x0 and min(abs(p.x) for p in poly) < x1 and max(s) > s0 and min(s) < s1
            if near and c.z < 0.3:
                inside, rest = split_bays(poly)
                for piece in inside:
                    doors.face(piece, UNDER)
                for piece in rest:
                    g.face(piece, UNDER)
                continue
            g.face(poly, UNDER)
        elif n.y < -0.5 and abs(n.z) < 0.25 and EXHAUST_S[0] < min(s) and max(s) < EXHAUST_S[1] and min(abs(p.x) for p in poly) > EXHAUST_X:
            # The slot along the foot of the deck's rear wall.
            top = min(p.z for p in poly) + EXHAUST_H
            g.face(mklib.clip(poly, lambda q: top - q.z, True), EXHAUST)
            g.face(mklib.clip(poly, lambda q: top - q.z, False), DARK)
        else:
            g.face(poly, BODY)
    return g, doors


def overlays():
    """What lies on the skin: the canopy's panes, the intake grids, the turrets' screens, and
    the probes."""
    g = Geo()
    lifted = lambda poly, k=1.0: [p + mklib.normal(poly) * LIFT * k for p in poly]
    for poly in source("glass"):
        g.face(lifted(poly), GLASS)
    for poly in source("sensor"):
        g.face(lifted(poly), WINDOW)
    for poly in source("intake"):
        q = lifted(poly)
        g.face(q, GRID)
        if len(q) != 4:
            continue
        # Bars across the longer way, and two along it.
        a, b, c, d = lifted(poly, 2.0)
        if (b - a).length < (c - b).length:
            a, b, c, d = b, c, d, a
        for k in range(1, 6):
            g.line(a.lerp(b, k / 6), d.lerp(c, k / 6), BODY.lo)
        for k in range(1, 3):
            g.line(a.lerp(d, k / 3), b.lerp(c, k / 3), BODY.lo)
    for x, s0, s1 in PROBES:
        for sx in ((-1, 1) if x else (1,)):
            g.line(P(sx * x, s0, PROBE_Z), P(sx * x, s1, PROBE_Z), PROBE)
    return g


def door(hinge0, hinge1, width, angle, out):
    """A bay door hinged along `hinge0`-`hinge1` (on the belly), `width` m wide, folded down
    `angle` degrees from the belly towards `out` (+1: away from the keel)."""
    a = math.radians(angle)
    d = Vector((out * math.cos(a) * width, 0, -math.sin(a) * width))
    g = Geo()
    g.face2([hinge0, hinge1, hinge1 + d, hinge0 + d], BODY)
    return g


def bay_doors(state, shut, z):
    """Both bays' doors at `state` 0 shut (`shut`: the pieces cut out of the underside), 1 half
    open, 2 open, the bays' inside with 2. Each hangs from the keel's side of its bay, at `z`."""
    g = Geo()
    (x0, x1), (s0, s1) = BAY_X, BAY_S
    angle = (0, 45, 90)[state]
    if state == 0:
        g.add(shut)
    for sx in (-1, 1):
        inner = (P(sx * x0, s0, z), P(sx * x0, s1, z))
        outer = (P(sx * x1, s0, z), P(sx * x1, s1, z))
        if state == 0:
            for p, q in ((inner[0], inner[1]), (outer[0], outer[1]), (inner[0], outer[0]), (inner[1], outer[1])):
                g.line(p + Vector((0, 0, -0.03)), q + Vector((0, 0, -0.03)), UNDER_SEAM)
            continue
        g.add(door(inner[0], inner[1], x1 - x0, angle, sx))
        if state == 2:
            g.add(bay_inside(sx, z))
    return g


def bay_inside(sx, z0):
    """A bay's walls and roof, seen from below, and its bomb."""
    g = Geo()
    (x0, x1), (s0, s1) = BAY_X, BAY_S
    lo = [P(sx * x0, s0, z0), P(sx * x1, s0, z0), P(sx * x1, s1, z0), P(sx * x0, s1, z0)]
    hi = [p + Vector((0, 0, BAY_DEPTH)) for p in lo]
    inside = P(sx * (x0 + x1) / 2, (s0 + s1) / 2, z0 + BAY_DEPTH / 2)
    outside = lambda f: (sum(f, Vector()) / len(f)) * 2 - inside
    walls = [[lo[i], lo[(i + 1) % 4], hi[(i + 1) % 4], hi[i]] for i in range(4)] + [hi]
    for f in walls:
        g.face(f, BAY, outside(f))
    # GBU-27: a 2,000 lb bomb with a laser seeker and fins, hung from the bay's roof.
    x, z = sx * (x0 + x1) / 2, z0 + BAY_DEPTH - 0.32
    y0, y1 = NOSE - s0 - 0.25, NOSE - s1 + 0.3
    body = [(y0, 0.04), (y0 - 0.35, 0.16), (y0 - 0.9, 0.19), (y1 + 0.6, 0.19), (y1 + 0.15, 0.12), (y1, 0.06)]
    for (ya, ra), (yb, rb) in zip(body, body[1:]):
        g.add(frustum((x, ya, z), (x, yb, z), ra, rb, BOMB, n=6, caps=False))
    for a in (45, 135, 225, 315):
        d = Vector((math.cos(math.radians(a)), 0, math.sin(math.radians(a))))
        g.face2([Vector((x, y1 + 0.55, z)) + d * 0.16, Vector((x, y1 + 0.05, z)) + d * 0.16,
                 Vector((x, y1 + 0.15, z)) + d * 0.36], METAL)
    return g


def gear():
    """The gear down: legs, wheels and the open gear doors."""
    g = Geo()
    x, s, r, w = NOSE_GEAR
    axle = P(x, s, GROUND + r)
    g.add(mklib.wheel(tuple(axle), r, w, TYRE, n=8))
    g.add(frustum(tuple(P(x, s - 0.25, 0.1)), tuple(axle + Vector((0, 0.05, 0.1))), 0.07, 0.06, METAL, n=5))
    for sx in (-1, 1):
        g.face2([P(sx * 0.28, s - 2.0, 0.08), P(sx * 0.28, s + 0.1, 0.08), P(sx * 0.30, s + 0.1, -0.37), P(sx * 0.30, s - 2.0, -0.37)], BODY)
    x, s, r, w = MAIN_GEAR
    for sx in (-1, 1):
        axle = P(sx * x, s, GROUND + r)
        g.add(mklib.wheel(tuple(axle), r, w, TYRE, n=8))
        g.add(frustum(tuple(P(sx * (x - 0.1), s - 0.3, 0.1)), tuple(axle + Vector((-sx * 0.12, 0, 0.12))), 0.09, 0.08, METAL, n=5))
        g.add(frustum(tuple(P(sx * (x - 0.1), s - 1.2, 0.1)), tuple(axle + Vector((-sx * 0.12, 0, 0.2))), 0.05, 0.05, METAL, n=4))
        g.face2([P(sx * (x + 0.28), s - 2.4, 0.1), P(sx * (x + 0.28), s - 0.4, 0.1), P(sx * (x + 0.32), s - 0.4, -0.5), P(sx * (x + 0.32), s - 2.4, -0.5)], BODY)
    return g


def group(parent, slot, value, geo):
    name = f"{parent.name}_s{slot}v{value}"
    ob = bpy.data.objects.new(name, None)
    ob["conds"] = [[slot, value]]
    for col in parent.users_collection:
        col.objects.link(ob)
    ob.parent = parent
    if geo.faces:
        heli.mesh_with_lines(ob, f"{name}_body", geo.scaled(K))
    else:
        for c, segs in sorted(geo.scaled(K).lines.items()):
            ln = f"{name}_lines_pal_{c:03}"
            mklib.link_child(ob, ln, mklib.lines_mesh(ln, segs, c))
    return ob


def build():
    for name in OBSOLETE:
        if name in bpy.data.objects:
            mklib.delete_tree(bpy.data.objects[name])
    root = bpy.data.objects[NAME]
    for c in list(root.children):
        mklib.delete_tree(c)
    root["attr"] = 0x84
    root["lod_dists"] = [MAX_DIST]
    shell, shut = airframe()
    shell.add(overlays())
    geo = shell.scaled(K)
    lod = heli.mesh_with_lines(root, f"{NAME}_lod0", geo)
    lod["max_dist"] = MAX_DIST
    lod["radius"] = heli.radius(geo, gear().scaled(K))
    # Aircraft are never tested against their hull: one plane, like the other generated models.
    lod["planes"] = [[0, 16384, 0, 0]]
    z = min(p.z for pts, _ in shut.faces for p in pts)
    for state in range(3):
        group(lod, 8, state, bay_doors(state, shut, z))
    group(lod, 5, 1, gear())
    lo, hi = shell.bounds()
    pts = [p for f, _ in shell.faces for p in f]
    tip = max(pts, key=lambda p: (round(p.x, 2), -p.y))
    heli.lights(lod, 12, {RED: [Vector((-tip.x - 0.03, tip.y + 0.3, tip.z + 0.03)) * K], GREEN: [Vector((tip.x + 0.03, tip.y + 0.3, tip.z + 0.03)) * K]})
    spine = max((p for p in pts if abs(p.x) < 0.05 and 9.0 < NOSE - p.y < 12.0), key=lambda p: p.z, default=P(0, 10.0, 2.2))
    heli.lights(lod, 20, {RED: [Vector((0, spine.y, spine.z + 0.04)) * K, v3(P(0, 12.5, -0.04)) * K]})
    print(f"{lod.name}: {len(geo.faces)} faces, radius {lod['radius']}, bounds {tuple(round(c, 2) for c in lo)} .. {tuple(round(c, 2) for c in hi)}")


mklib.load_palette(os.path.join(ROOT, "assets", "palettes", "tod_central_europe.ron"))
build()
bpy.ops.wm.save_mainfile()
