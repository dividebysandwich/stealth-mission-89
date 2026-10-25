"""Builds the aircraft carrier (a Nimitz-class ship) into every theater master that has one, and
saves them:

    blender -b --python-exit-code 1 --python tools/models/carrier.py
    tools/models/export.sh libya persian_gulf desert_storm middle_east north_cape vietnam cuba korea

From the top view of the game's original carrier and a Nimitz-class plan: the flight deck with
the angled landing deck and its ramp jutting out to port,
the bow and waist catapults, four deck-edge elevators, the island on the starboard side with its
bridge windows and mast, the hull flaring out to the deck. The ship is drawn at the game's carrier
size, not its real one (333 m long): the hull is 480 model units long with the flight deck 32 up,
which is what landing on it, its deck altitude and the take-off spawn expect (src/flight.rs
`landing_corridor`, crates/stealth-mission generator).

Each carrier node keeps its name, its root's `attr` and the way it faces (bow along +y or -y),
and gets one detail level drawn at every distance. Children of the detail level:
the wake behind the stern (`_lines_pal_057`), the red mast lights, `_s10v1` the deck lights
(lights on), `_s11v0` / `_s11v1` the deck markings by day and lit at night. The collision
hull (`planes`) is the convex hull of the ship below the flight deck, in the glTF frame the game
tests the pilot's eye in (Q14 normals, inside when n.p + d < 0).
"""

import math
import os
import sys

import bmesh
import bpy
from mathutils import Vector

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.abspath(os.path.join(HERE, "..", ".."))
sys.path.insert(0, HERE)

import heli  # noqa: E402
import mklib  # noqa: E402
from mklib import Geo, Ramp  # noqa: E402

# The carrier nodes per theater master (both former carrier models of each theater).
CARRIERS = {
    "libya": ["libya_063", "libya_087"],
    "persian_gulf": ["persian_gulf_073", "persian_gulf_087"],
    "desert_storm": ["desert_storm_073", "desert_storm_087"],
    "middle_east": ["middle_east_073", "middle_east_087"],
    "north_cape": ["north_cape_061", "north_cape_072"],
    "vietnam": ["vietnam_075", "vietnam_090"],
    "cuba": ["cuba_075", "cuba_090"],
    "korea": ["korea_075", "korea_090"],
}

LENGTH = 333.0  # m, bow to stern
U = 480.0 / LENGTH  # model units per metre along the ship
DECK, DECK_UNDER, HULL_TOP = 32.0, 28.0, 28.0
H = DECK / 18.5  # model units per metre upwards (the deck stands 18.5 m above the water)

MAX_DIST = 32769
RADIUS = 514

HULL = Ramp(226, 233)
DECK_PAINT = 236
DECK_EDGE = Ramp(230, 236)
ISLAND = Ramp(225, 232)
GLASS = Ramp(144, 145)
ELEVATOR = 234
WAKE = 57
RED = 146
LIGHT = 148
# Deck markings by day and lit at night: centre lines, and the foul lines and catapults.
MARK_DAY, MARK_DAY2 = 230, 203
MARK_NIGHT, MARK_NIGHT2 = 7, 19

# Deck outline (m aft of the bow, m to starboard), starboard side then port side, bow to stern:
# a square bow, the angled deck's ramp jutting out to port a third of the way aft, the straight
# sides and the slanted stern.
DECK_STBD = [(0, 17), (71, 25), (103, 40), (268, 40), (319, 24)]
DECK_PORT = [(0, -17), (85, -25), (103, -50), (148, -40), (268, -40), (333, -24)]
# The hull at the waterline and under the deck: (m aft, half beam m), symmetric.
WATERLINE = [(4, 0), (22, 9), (65, 18), (110, 22), (230, 22), (290, 17), (318, 12)]
HULL_UNDER = [(1, 0), (20, 14), (75, 24), (110, 34), (260, 34), (300, 22), (325, 12)]
# Elevators: (m aft from, to, m from, to across).
ELEVATORS = [(120, 140, 28, 39), (160, 180, 28, 39), (240, 260, 28, 39), (160, 180, -40, -29)]
# Catapults: (m aft, m across) start and end.
CATAPULTS = [((8, -8), (90, -11)), ((8, 8), (90, 10)), ((110, -34), (185, -36)), ((115, -22), (190, -26))]
# The angled deck's centre line and its edges (landing area), stern to forward end.
LANDING = ((322, -2), (140, -32))


def _parallel(line, offset):
    """`line` moved `offset` m to the ship's starboard (negative: to port)."""
    (s0, x0), (s1, x1) = line
    ds, dx = s1 - s0, x1 - x0
    n = math.hypot(ds, dx)
    ps, px = dx / n, -ds / n  # to starboard for a line running forward
    return ((s0 + ps * offset, x0 + px * offset), (s1 + ps * offset, x1 + px * offset))


# The landing area's edges, parallel to its centre line.
FOUL_PORT = _parallel(LANDING, -9.0)
FOUL_STBD = _parallel(LANDING, 9.0)
ISLAND_AT = (193, 228, 31, 39)  # m aft from, to, m across from, to


def P(s, x, z):
    """A point: s m aft of the bow, x m to starboard, z model units up."""
    return Vector((x * U, 240.0 - s * U, z))


def ring(points, z, half=None):
    """Points of an outline at height z; with `half`, a symmetric half outline mirrored."""
    if half is not None:
        right = [P(s, b, z) for s, b in half]
        left = [P(s, -b, z) for s, b in reversed(half) if b > 0]
        return right + left
    return [P(s, x, z) for s, x in points]


def deck_outline(z):
    return ring(DECK_STBD + list(reversed(DECK_PORT)), z)


def ship():
    g = Geo()
    centre = P(160, 0, 14)
    # Hull: from the waterline flaring out to under the deck, a transom at the stern.
    lo = ring(None, 0.0, WATERLINE)
    hi = ring(None, HULL_TOP, HULL_UNDER)
    n = len(lo)
    for i in range(n):
        j = (i + 1) % n
        if (i, j) == (len(WATERLINE) - 1, len(WATERLINE)):
            g.face([lo[i], lo[j], hi[j], hi[i]], HULL.hi, centre)
            continue
        g.face([lo[i], lo[j], hi[j], hi[i]], HULL, centre)
    # The flight deck: a slab overhanging the hull, its edges and underside.
    top, under = deck_outline(DECK), deck_outline(DECK_UNDER)
    g.face(top, DECK_PAINT, centre)
    g.face(under, HULL.hi, P(160, 0, 40))
    for i in range(len(top)):
        j = (i + 1) % len(top)
        g.face([under[i], under[j], top[j], top[i]], DECK_EDGE, centre)
    # Elevators, a shade lighter, a hair above the deck.
    for s0, s1, x0, x1 in ELEVATORS:
        g.face([P(s0, x0, DECK + 0.05), P(s0, x1, DECK + 0.05), P(s1, x1, DECK + 0.05), P(s1, x0, DECK + 0.05)],
               ELEVATOR, centre)
    island(g)
    return g


def island(g):
    """The island: a base block, the bridge levels with their window bands, the mast."""
    s0, s1, x0, x1 = ISLAND_AT
    levels = [(s0, s1, x0, x1, 0, 9), (s0 + 4, s1 - 2, x0 + 1, x1 - 0.5, 9, 15),
              (s0 + 6, s1 - 6, x0 + 1.5, x1 - 1, 15, 19)]
    for a, b, c, d, z0, z1 in levels:
        z0, z1 = DECK + z0 * H, DECK + z1 * H
        box = [[P(a, c, z0), P(a, d, z0), P(a, d, z1), P(a, c, z1)],
               [P(a, d, z0), P(b, d, z0), P(b, d, z1), P(a, d, z1)],
               [P(b, d, z0), P(b, c, z0), P(b, c, z1), P(b, d, z1)],
               [P(b, c, z0), P(a, c, z0), P(a, c, z1), P(b, c, z1)],
               [P(a, c, z1), P(a, d, z1), P(b, d, z1), P(b, c, z1)]]
        mid = P((a + b) / 2, (c + d) / 2, (z0 + z1) / 2)
        for f in box:
            g.face(f, ISLAND, mid)
        # A window band round the top of each level.
        w0, w1 = z1 - (z1 - z0) * 0.35, z1 - (z1 - z0) * 0.12
        lift = 0.06
        for (p, q) in (((a, c), (a, d)), ((a, d), (b, d)), ((b, c), (a, c))):
            pa, pb = P(p[0], p[1], 0), P(q[0], q[1], 0)
            out = (pa + pb) / 2 - P((a + b) / 2, (c + d) / 2, 0)
            out = out.normalized() * lift
            band = [Vector((pa.x, pa.y, w0)) + out, Vector((pb.x, pb.y, w0)) + out,
                    Vector((pb.x, pb.y, w1)) + out, Vector((pa.x, pa.y, w1)) + out]
            g.face(band, GLASS, mid)
    # Mast with its yard and radar.
    top = DECK + 19 * H
    m = P((s0 + s1) / 2 + 2, (x0 + x1) / 2, top)
    g.line(m, m + Vector((0, 0, 14 * H)), ISLAND.hi)
    g.line(m + Vector((-5, 0, 10 * H)), m + Vector((5, 0, 10 * H)), ISLAND.hi)
    g.face2([m + Vector((-4, -1, 5 * H)), m + Vector((4, -1, 5 * H)), m + Vector((4, -1, 8 * H)), m + Vector((-4, -1, 8 * H))],
            ISLAND)


def markings(day):
    """Deck lines: the landing area's centre line, its foul lines and the catapult tracks."""
    g = Geo()
    a, b = (MARK_DAY, MARK_DAY2) if day else (MARK_NIGHT, MARK_NIGHT2)
    z = DECK + 0.08
    g.line(P(*LANDING[0], z), P(*LANDING[1], z), a)
    g.line(P(0, 0, z), P(140, 0, z), a)
    for p, q in (FOUL_PORT, FOUL_STBD) + tuple(CATAPULTS):
        g.line(P(*p, z), P(*q, z), b)
    for s0, s1, x0, x1 in ELEVATORS:
        for p, q in (((s0, x0), (s1, x0)), ((s0, x1), (s1, x1)), ((s0, x0), (s0, x1)), ((s1, x0), (s1, x1))):
            g.line(P(*p, z), P(*q, z), b)
    return g


def deck_lights():
    pts = []
    z = DECK + 0.1
    for (p, q) in (LANDING, FOUL_PORT, FOUL_STBD):
        for k in range(16):
            t = k / 15
            pts.append(P(p[0] + (q[0] - p[0]) * t, p[1] + (q[1] - p[1]) * t, z))
    return pts


def wake():
    g = Geo()
    for sx in (-1, 1):
        g.line(P(328, sx * 12, 0.0), P(328 + 190, sx * 34, 0.0), WAKE)
        g.line(P(328, sx * 6, 0.0), P(328 + 150, sx * 14, 0.0), WAKE)
    return g


def planes(geo, flip):
    """Collision planes of the ship below the flight deck (glTF frame: x, z up, -y)."""
    bm = bmesh.new()
    for pts, _ in geo.faces:
        for p in pts:
            if p.z <= DECK + 0.01:
                q = flip(p)
                bm.verts.new((q.x, q.z, -q.y))
    bmesh.ops.convex_hull(bm, input=bm.verts)
    bm.faces.ensure_lookup_table()
    out, seen = [], set()
    for f in bm.faces:
        n = f.normal.normalized()
        d = -n.dot(f.verts[0].co)
        key = tuple(round(v, 3) for v in (*n, d / 100))
        if key in seen:
            continue
        seen.add(key)
        out.append([round(n.x * 16384), round(n.y * 16384), round(n.z * 16384), round(d * 16384)])
    bm.free()
    return out


def orientation(lod):
    """+1 if the old model's bow points along +y (its wake trails to -y), else -1."""
    ys = []
    for ob in [lod] + list(lod.children_recursive):
        if ob.type == "MESH":
            ys += [(ob.matrix_world @ v.co).y for v in ob.data.vertices]
    return 1 if min(ys) < -300 else -1


def build(blend):
    mklib.load_palette(os.path.join(ROOT, "assets", "palettes", f"tod_{blend}.ron"))
    bpy.ops.wm.open_mainfile(filepath=os.path.join(ROOT, "assets-src", "blend", f"{blend}.blend"))
    for name in CARRIERS[blend]:
        root = bpy.data.objects[name]
        lod = bpy.data.objects[f"{name}_lod0"]
        bow = orientation(lod)
        flip = (lambda p: p) if bow > 0 else (lambda p: Vector((-p.x, -p.y, p.z)))
        # One detail level, drawn at every distance.
        for c in list(root.children):
            mklib.delete_tree(c)
        root["lod_dists"] = [MAX_DIST]
        keep = {"max_dist": MAX_DIST, "radius": RADIUS}
        geo = ship().transformed(flip)
        lod = heli.mesh_with_lines(root, f"{name}_lod0", geo)
        for k, v in keep.items():
            lod[k] = v
        lod["planes"] = planes(ship(), flip)
        w = wake().transformed(flip)
        for c, segs in w.lines.items():
            ln = f"{lod.name}_lines_pal_{c:03}"
            mklib.link_child(lod, ln, mklib.lines_mesh(ln, segs, c))
        mast = flip(P((ISLAND_AT[0] + ISLAND_AT[1]) / 2 + 2, (ISLAND_AT[2] + ISLAND_AT[3]) / 2, DECK + 33 * H))
        dn = f"{lod.name}_dots_pal_{RED:03}"
        mklib.link_child(lod, dn, mklib.points_mesh(dn, [mast], RED))
        heli.lights(lod, 10, {LIGHT: [flip(p) for p in deck_lights()]})
        for value, day in ((0, True), (1, False)):
            group = bpy.data.objects.new(f"{lod.name}_s11v{value}", None)
            group["conds"] = [[11, value]]
            for col in lod.users_collection:
                col.objects.link(group)
            group.parent = lod
            for c, segs in markings(day).transformed(flip).lines.items():
                ln = f"{group.name}_lines_pal_{c:03}"
                mklib.link_child(group, ln, mklib.lines_mesh(ln, segs, c))
        print(f"{name}: bow {'+' if bow > 0 else '-'}y, {len(geo.faces)} faces, {len(lod['planes'])} planes")
    bpy.ops.wm.save_mainfile()


for blend in CARRIERS:
    build(blend)
