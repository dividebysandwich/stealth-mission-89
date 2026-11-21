"""Builds the Ukraine theater's own models in Blender and exports them.

    blender -b --factory-startup --python tools/theater/models.py -- ukraine <out.glb>

Writes <out.glb> and <out.glb>.json (model numbers, attributes and detail level distances,
merged into assets/models/ukraine.glb by assemble.py together with Central Europe's models).

- Sea, lake and land overlays per LOD4 cell (128 km): the coasts, rivers and reservoirs of
  regions/ukraine.py cut into triangles by a constrained Delaunay triangulation, each triangle coloured by
  what lies under it (sea OR-overlay 49, inland water AND-overlay 57, land map overlay 50).
- Forest patches (LOD2), nuclear power plants (LOD2), ruined town blocks, a rubble pile and
  Donbas slag heaps (LOD1).

Model space: x east, y north, z up, in the units of the level the model is placed on.
"""

import json
import math
import os
import random
import sys

import bpy
from mathutils import Vector
from mathutils.geometry import delaunay_2d_cdt

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.abspath(os.path.join(HERE, "..", ".."))
sys.path.insert(0, HERE)
sys.path.insert(0, os.path.join(ROOT, "tools", "models"))

import common  # noqa: E402
import kerch  # noqa: E402

geo = common.load(sys.argv[sys.argv.index("--") + 1])
import mklib  # noqa: E402
from mklib import Geo, Ramp, block, frustum, hexa, tube  # noqa: E402

LEVEL_M = {1: geo.WU, 2: 4 * geo.WU, 3: 16 * geo.WU, 4: 64 * geo.WU}
CELL_M = {lod: 4096 * m for lod, m in LEVEL_M.items()}

SEA, LAKE = 49, 57
# The land's map overlay colour (fields 50, sand 54).
LAND = getattr(geo, "LAND_COLOUR", 50)

# Model numbers: Central Europe's 0..93, then these. LOD1/LOD2 models stay below 128 (the
# simulation masks tile shapes with 0x7F); the LOD4 water cells follow from 128.
NPP_SHAPES = {"znpp": 94, "rostov": 95, "south": 96, "kursk": 97, "novovoronezh": 98}
RUINS = [99, 100, 101]
RUBBLE = 102
TERRIKON = 103
FORESTS = [104, 105, 106, 107]
PLAIN_LAND, PLAIN_SEA = 108, 109  # for a library without a plain land or sea ground
MOUNTAIN_BASE = 110
WATER_BASE = 128
LIBRARY = json.load(open(os.path.join(HERE, "libraries", geo.LIBRARY + ".json")))
# The library's plain land and open sea ground of a LOD4 cell (or the region's own choice).
KINDS = dict(LIBRARY["kinds"], **getattr(geo, "KINDS", {}))
LAND_TILE, SEA_TILE = KINDS.get("land", PLAIN_LAND), KINDS.get("sea", PLAIN_SEA)

CONCRETE = Ramp(0xE1, 0xE6)
WHITE = Ramp(0xE0, 0xE2)
GREY = Ramp(0xE3, 0xEA)
DARK = Ramp(0xE9, 0xEF)
BLUEGREY = Ramp(0xF1, 0xF6)
RUBBLE_C = Ramp(0xE6, 0xEE)
BRICK = Ramp(0xD5, 0xDB)
SOOT = 0xEE
STRIPE = 0xD7


def models_root():
    for o in list(bpy.data.objects):
        bpy.data.objects.remove(o, do_unlink=True)


def empty(name, parent=None):
    ob = bpy.data.objects.new(name, None)
    bpy.context.scene.collection.objects.link(ob)
    ob.parent = parent
    return ob


def mesh_obj(name, geo_, parent):
    ob = bpy.data.objects.new(name, mklib.faces_mesh(name, geo_))
    bpy.context.scene.collection.objects.link(ob)
    ob.parent = parent
    for c, segs in sorted(geo_.lines.items()):
        ln = "%s_lines_pal_%03d" % (name, c)
        lo = bpy.data.objects.new(ln, mklib.lines_mesh(ln, segs, c))
        bpy.context.scene.collection.objects.link(lo)
        lo.parent = ob
    return ob


META = []


def no_floors(g):
    """Drops the faces under buildings (on the ground, facing down): never seen, and the
    scenery would take a long one for a runway."""
    out = Geo()
    out.lines = g.lines
    out.faces = [(p, c) for p, c in g.faces if not (all(abs(v.z) < 1e-3 for v in p) and mklib.normal(p).z < -0.9)]
    return out


def hull_planes(g):
    """The convex hull of a model's points as collision planes in the exported frame (y up):
    [nx, ny, nz, d] with a Q14 normal, inside where n.p + d < 0."""
    import bmesh

    bm = bmesh.new()
    for f, _ in g.faces:
        for p in f:
            bm.verts.new(p)
    bmesh.ops.convex_hull(bm, input=bm.verts)
    planes = set()
    for f in bm.faces:
        n = f.normal
        d = -n.dot(f.verts[0].co)
        # Blender z up -> glTF y up: (x, y, z) -> (x, z, -y).
        planes.add((round(n.x * 16384), round(n.z * 16384), round(-n.y * 16384), round(d * 16384)))
    bm.free()
    return [list(p) for p in sorted(planes)]


def model(index, lods, attr, child_suffix="", hull=False, wrecks=None):
    """A root `<theater>_NNN` with one node per detail level: [(geo, max_dist)]. With a suffix,
    the geometry hangs below each level in a child node of that name (`_s1v0`: a building,
    `_forest`). With `hull`, each level collides with its convex hull (mountains). `wrecks`
    (one geo per level) is the destroyed state, shown in a `_s1v1` node instead of `_s1v0`."""
    lods = [(no_floors(g), d) for g, d in lods]
    name = "%s_%03d" % (geo.ID, index)
    root = empty(name)
    info = {"index": index, "name": name, "attr": attr, "lods": []}
    for k, (g, max_dist) in enumerate(lods):
        ln = "%s_lod%d" % (name, k)
        pts = [p for f, _ in g.faces for p in f]
        radius = max((Vector((p.x, p.y, p.z)).length for p in pts), default=0.0)
        if child_suffix:
            node = empty(ln, root)
            mesh_obj(ln + child_suffix, g, node)
            if wrecks:
                mesh_obj(ln + "_s1v1", no_floors(wrecks[k]), node)
        else:
            mesh_obj(ln, g, root)
        info["lods"].append({"name": ln, "max_dist": max_dist, "radius": int(math.ceil(radius))})
        if hull:
            info["lods"][-1]["planes"] = hull_planes(g)
    META.append(info)


def flat(pts, c):
    """Upward facing flat polygon (counter-clockwise from above)."""
    pts = [Vector((p[0], p[1], p[2] if len(p) > 2 else 0.0)) for p in pts]
    area = sum(pts[i].x * pts[(i + 1) % len(pts)].y - pts[(i + 1) % len(pts)].x * pts[i].y for i in range(len(pts)))
    if area < 0:
        pts.reverse()
    return pts, c


# ---------------------------------------------------------------------------------------------
# Water and land overlays per LOD4 cell


def clip(a, b, x0, y0, x1, y1):
    """Liang-Barsky: the part of segment a-b inside the rectangle, or None."""
    t0, t1 = 0.0, 1.0
    dx, dy = b[0] - a[0], b[1] - a[1]
    for p, q in ((-dx, a[0] - x0), (dx, x1 - a[0]), (-dy, a[1] - y0), (dy, y1 - a[1])):
        if p == 0:
            if q < 0:
                return None
        else:
            t = q / p
            if p < 0:
                if t > t1:
                    return None
                t0 = max(t0, t)
            else:
                if t < t0:
                    return None
                t1 = min(t1, t)
    return (a[0] + dx * t0, a[1] + dy * t0), (a[0] + dx * t1, a[1] + dy * t1)


def water_cells(water):
    """Overlay geometry of each LOD4 cell (cols/rows -2..5): shape number per cell."""
    cell = CELL_M[4]
    unit = LEVEL_M[4]
    polys = water.polygons()
    shapes = {}
    next_shape = WATER_BASE
    for row in range(-2, 6):
        for col in range(-2, 6):
            x0, y0 = col * cell, row * cell
            x1, y1 = x0 + cell, y0 + cell
            cx, cy = x0 + cell / 2, y0 + cell / 2
            segs = []
            for _, p in polys:
                b = geo.bbox(p)
                if b[2] < x0 or b[0] > x1 or b[3] < y0 or b[1] > y1:
                    continue
                for i in range(len(p)):
                    s = clip(p[i], p[(i + 1) % len(p)], x0, y0, x1, y1)
                    if s and (abs(s[0][0] - s[1][0]) + abs(s[0][1] - s[1][1])) > 1.0:
                        segs.append(s)
            if not segs:
                kind = water.at(cx, cy)
                shapes[(col, row)] = SEA_TILE if kind == "sea" else LAND_TILE
                continue
            loc = lambda q: Vector(((q[0] - cx) / unit, (q[1] - cy) / unit))
            verts, edges, index = [], [], {}

            def vid(q):
                v = loc(q)
                key = (round(v.x, 3), round(v.y, 3))
                if key not in index:
                    index[key] = len(verts)
                    verts.append(v)
                return index[key]

            corners = [vid((x0, y0)), vid((x1, y0)), vid((x1, y1)), vid((x0, y1))]
            for i in range(4):
                edges.append((corners[i], corners[(i + 1) % 4]))
            for a, b in segs:
                ia, ib = vid(a), vid(b)
                if ia != ib:
                    edges.append((ia, ib))
            out = delaunay_2d_cdt(verts, edges, [], 0, 1e-4)
            vco, faces = out[0], out[2]
            g = Geo()
            kinds = set()
            for f in faces:
                pts = [vco[i] for i in f]
                area = (pts[1] - pts[0]).cross(pts[2] - pts[0]) if len(pts) == 3 else 1.0
                if abs(area) < 1e-6:
                    continue
                c = sum(pts, Vector((0, 0))) / len(pts)
                kind = water.at(cx + c.x * unit, cy + c.y * unit)
                kinds.add(kind)
                colour = SEA if kind == "sea" else LAKE if kind == "lake" else LAND
                g.faces.append(flat([(p.x, p.y) for p in pts], colour))
            if kinds == {"sea"}:
                shapes[(col, row)] = SEA_TILE
                continue
            if kinds == {None}:
                shapes[(col, row)] = LAND_TILE
                continue
            shape = next_shape
            next_shape += 1
            model(shape, [(merge_flat(g), 32769)], 7)
            shapes[(col, row)] = shape
            print("water cell", col, row, "->", shape, len(g.faces), "triangles", flush=True)
    return shapes


def merge_flat(g):
    """Drops nothing; keeps the faces (Blender merges nothing either). Hook for tuning."""
    return g


# ---------------------------------------------------------------------------------------------
# Forest patches (LOD2: 1.95 m units, cell +-2048)


def blob(cx, cy, r, rng, n=20):
    pts = []
    phase = [rng.uniform(0, math.tau) for _ in range(3)]
    for i in range(n):
        a = math.tau * i / n
        k = 1.0 + 0.22 * math.sin(2 * a + phase[0]) + 0.14 * math.sin(3 * a + phase[1]) + 0.08 * math.sin(5 * a + phase[2])
        pts.append((cx + math.cos(a) * r * k, cy + math.sin(a) * r * k * rng.uniform(0.85, 1.0)))
    return pts


def forest_patches():
    """Four patch sets of rising cover. The colour mix gives woods some texture from afar."""
    for k, shape in enumerate(FORESTS):
        rng = random.Random(4100 + k)
        g = Geo()
        count = [3, 5, 7, 9][k]
        for i in range(count):
            r = rng.uniform(150, 420) * (1.0 + 0.35 * k)
            cx, cy = rng.uniform(-1700 + r * 0.5, 1700 - r * 0.5), rng.uniform(-1700 + r * 0.5, 1700 - r * 0.5)
            g.faces.append(flat(blob(cx, cy, r, rng), rng.choice([0xAB, 0xAC, 0xAC, 0xAD])))
        model(shape, [(g, 32769)], 6, "_forest")


# ---------------------------------------------------------------------------------------------
# Mountains, in metres (scaled to LOD4 units): a range along y


SNOW = Ramp(0xE0, 0xE2)
# Slopes by height up to the rock: green, or on sand (the desert theaters, whose 0xA0 ramp is a
# pure green and whose browns a bright orange) the sand ground's own ramp, darker higher up.
if LAND == 54:
    ROCK = Ramp(0xE5, 0xEB)
    SLOPES = [(0.3, Ramp(0x78, 0x7E)), (1.0, Ramp(0x7B, 0x7F))]
else:
    ROCK = Ramp(0xE4, 0xEA)
    SLOPES = [(1.0, Ramp(0xA5, 0xAE))]
HEIGHT_SCALE = 1.6  # the game's mountains stand out of the squeezed theater


def mountain(seed, length, width, height, bearing):
    """A mountain range `length` long and `width` wide, its long axis `bearing` degrees from
    north: rings of the same number of points round an ellipse, rising to a ridge with two or
    three peaks along the long axis, joined ring to ring (a closed surface without slivers),
    coloured green (earth in the desert), rock and snow by height."""
    rng = random.Random(seed)
    top = height * HEIGHT_SCALE
    n = 24
    rings = [1.0, 0.82, 0.64, 0.46, 0.3, 0.16]
    peaks = [(rng.uniform(-0.55, -0.2), rng.uniform(0.75, 0.95)), (rng.uniform(-0.1, 0.1), 1.0),
             (rng.uniform(0.2, 0.55), rng.uniform(0.75, 0.95))]

    def z(u, v, s):
        """Height at (u, v) (along and across, -1..1) on ring `s`."""
        if s >= 1.0:
            return 0.0
        ridge = max(f * math.exp(-((u - c) / 0.35) ** 2) for c, f in peaks)
        return top * (1.0 - s) ** 0.85 * (0.55 + 0.45 * ridge) * rng.uniform(0.85, 1.1)

    b = math.radians(bearing)

    def place(x, y, h):
        return (x * math.cos(b) + y * math.sin(b), -x * math.sin(b) + y * math.cos(b), h)

    grid = []
    for s_ in rings:
        ring = []
        for k in range(n):
            a = k * math.tau / n
            jitter = rng.uniform(0.94, 1.04) if s_ < 1.0 else 1.0
            u, v = math.sin(a) * s_ * jitter, math.cos(a) * s_ * jitter
            ring.append(place(v * width / 2, u * length / 2, z(u, v, s_)))
        grid.append(ring)
    summit = place(0.0, peaks[1][0] * length / 2 * 0.2, top * rng.uniform(0.95, 1.05))
    g = Geo()

    def tri(a, b_, c):
        # Bands by height, their edges a little ragged; the rock comes down further on the
        # steep faces.
        mean = (a[2] + b_[2] + c[2]) / 3 / top + rng.uniform(-0.04, 0.04)
        steep = 1.0 - abs(mklib.normal([Vector(a), Vector(b_), Vector(c)]).z)
        slope = next(ramp for limit, ramp in SLOPES if mean <= limit)
        rock = mean > 0.6 or (mean > 0.45 and steep > 0.6)
        paint = SNOW if mean > 0.72 and height > 1800 else ROCK if rock else slope
        g.face([a, b_, c], paint, (0, 0, -top))

    for outer, inner in zip(grid, grid[1:]):
        for k in range(n):
            k1 = (k + 1) % n
            tri(outer[k], outer[k1], inner[k1])
            tri(outer[k], inner[k1], inner[k])
    for k in range(n):
        tri(grid[-1][k], grid[-1][(k + 1) % n], summit)
    return g


# ---------------------------------------------------------------------------------------------
# Buildings, in metres (scaled to the level's unit at the end)


def scaled(g, unit):
    return g.scaled(1.0 / unit)


def dome_unit(x, y, g, lod):
    """VVER containment: cylinder with a shallow dome, on its auxiliary building."""
    n = 16 if lod == 0 else 8
    g.add(block(x - 34, x + 34, y - 30, y + 30, 0, 22, GREY))
    g.add(frustum((x, y, 0), (x, y, 46), 23, 23, WHITE, n, caps=False))
    rings = [mklib.ring((x, y, 46 + 14 * math.sin(math.radians(a))), 23 * math.cos(math.radians(a)), 23 * math.cos(math.radians(a)), n,
                        axis=(0, 0, 1)) for a in (0, 30, 60, 85)]
    g.add(tube(rings, WHITE, cap0=False, cap1=True))


def turbine_hall(x0, x1, y0, y1, h, g, paint=BLUEGREY):
    g.add(block(x0, x1, y0, y1, 0, h, paint))
    g.add(block(x0 + 4, x1 - 4, y0 + 4, y1 - 4, h, h + 3, GREY))


def stack(x, y, h, g, r=3.5, striped=True, n=8):
    g.add(frustum((x, y, 0), (x, y, h * 0.8), r * 1.3, r, CONCRETE, n, caps=False))
    g.add(frustum((x, y, h * 0.8), (x, y, h), r, r * 0.9, STRIPE if striped else CONCRETE, n, caps=True))


def cooling_tower(x, y, h, g, lod, r=None):
    """Hyperboloid shell, open at the top, its inside showing from above."""
    r = r or h * 0.4
    n = 16 if lod == 0 else 8
    prof = [(0.0, 1.0), (0.25, 0.86), (0.55, 0.68), (0.8, 0.62), (1.0, 0.66)]
    rings = [mklib.ring((x, y, h * t), r * s, r * s, n, axis=(0, 0, 1)) for t, s in prof]
    shell = tube(rings, Ramp(0xE1, 0xE7), cap0=False, cap1=False)
    g.add(shell)
    inner = Geo()
    inner.faces = [(list(reversed(p)), 0xE9) for p, _ in shell.faces]
    g.add(inner)
    # Water basin at the foot, seen through the opening.
    g.faces.append(flat(mklib.ring((x, y, h * 0.3), r * 0.7, r * 0.7, n, axis=(0, 0, 1)), 0xEB))


def pad(x0, y0, x1, y1, g, c=0xE4):
    g.faces.append(flat([(x0, y0, 0.05), (x1, y0, 0.05), (x1, y1, 0.05), (x0, y1, 0.05)], c))


def pond(pts, g):
    g.faces.append(flat([(p[0], p[1], 0.06) for p in pts], LAKE))


def spray_pools(x, y, g, n=4):
    for i in range(n):
        px = x + (i % 2) * 140
        py = y + (i // 2) * 110
        g.faces.append(flat([(px, py, 0.08), (px + 120, py, 0.08), (px + 120, py + 90, 0.08), (px, py + 90, 0.08)], LAKE))


def npp(kind, lod):
    g = Geo()
    if kind == "znpp":
        # Six VVER-1000 units in a row along the turbine hall, two cooling towers, spray
        # pools, and the cooling pond reaching to the old reservoir bed.
        pad(-560, -260, 560, 260, g)
        for i in range(6):
            x = -450 + i * 180
            dome_unit(x, 70, g, lod)
            stack(x + 45, 40, 100, g, striped=False)
        turbine_hall(-520, 520, -110, -30, 34, g)
        cooling_tower(-380, -420, 150, g, lod)
        cooling_tower(-140, -420, 150, g, lod)
        if lod == 0:
            spray_pools(120, -480, g, 6)
            block_row(g, -520, 210, 8, 60, 30, 14)
        pond([(-1500, 300), (-600, 380), (-600, 1700), (-1500, 1500)], g)
    elif kind == "rostov":
        pad(-420, -240, 420, 240, g)
        for i in range(4):
            x = -300 + i * 200
            dome_unit(x, 60, g, lod)
            stack(x + 45, 30, 100, g, striped=False)
        turbine_hall(-380, 380, -110, -30, 32, g)
        for i in range(2):
            cooling_tower(-200 + i * 400, -420, 170, g, lod)
        pond([(-1400, 400), (1400, 450), (1400, 1600), (-1400, 1600)], g)
    elif kind == "south":
        pad(-360, -220, 360, 220, g)
        for i in range(3):
            x = -220 + i * 220
            dome_unit(x, 60, g, lod)
            stack(x + 45, 30, 100, g, striped=False)
        turbine_hall(-320, 320, -110, -30, 32, g)
        cooling_tower(-120, -420, 150, g, lod)
        cooling_tower(160, -420, 150, g, lod)
        if lod == 0:
            spray_pools(-420, -480, g, 4)
        pond([(500, -700), (1600, -600), (1700, 400), (520, 300)], g)
    elif kind == "kursk":
        # RBMK: four boxy reactor buildings in two pairs, striped stacks, a long turbine
        # hall, the cooling pond south.
        pad(-640, -240, 640, 260, g)
        for i in range(4):
            x = -480 + i * 320
            g.add(block(x - 55, x + 55, 30, 140, 0, 72, GREY))
            g.add(block(x - 40, x + 40, 50, 120, 72, 82, CONCRETE))
            if i % 2 == 0:
                stack(x + 160, 190, 150, g, r=4.5)
        turbine_hall(-600, 600, -100, 10, 40, g, GREY)
        pond([(-1800, -500), (1800, -500), (1800, -1800), (-1800, -1800)], g)
    elif kind == "novovoronezh":
        # Two VVER-1200 with their tall cooling towers, the older blocks with smaller ones.
        pad(-560, -300, 560, 280, g)
        for i in range(2):
            x = 260 + i * 200
            dome_unit(x, 80, g, lod)
        turbine_hall(180, 540, -110, -20, 34, g)
        cooling_tower(260, -460, 171, g, lod)
        cooling_tower(500, -460, 171, g, lod)
        for i in range(3):
            x = -440 + i * 170
            g.add(block(x - 50, x + 50, 20, 120, 0, 48, GREY))
            g.add(block(x - 36, x + 36, 36, 104, 48, 60, CONCRETE))
        turbine_hall(-520, 0, -110, -10, 30, g)
        for i in range(4):
            cooling_tower(-480 + i * 140, -380, 95, g, lod)
        stack(-120, 160, 120, g)
    return g


def block_row(g, x0, y, n, w, d, h):
    for i in range(n):
        x = x0 + i * (w + 20)
        g.add(block(x, x + w, y, y + d, 0, h, GREY))


# ---------------------------------------------------------------------------------------------
# Ruins (LOD1: 0.49 m units)


def ruined_block(g, cx, cy, L, W, H, rng, lod, angle=0.0):
    """A gutted apartment block: broken wall panels with jagged tops and gaps, slabs hanging
    from the floors that still stand, a debris spill along the walls, soot over the windows."""
    ca, sa = math.cos(angle), math.sin(angle)
    P = lambda x, y, z: (cx + x * ca - y * sa, cy + x * sa + y * ca, z)
    t = 0.5
    floors = max(1, int(H / 3.0))
    sides = [((-L / 2, -W / 2), (L / 2, -W / 2)), ((L / 2, -W / 2), (L / 2, W / 2)), ((L / 2, W / 2), (-L / 2, W / 2)),
             ((-L / 2, W / 2), (-L / 2, -W / 2))]
    # Collapse profile along the block: one end may stand, the middle often fell in.
    standing = rng.uniform(0.25, 1.0)
    for (ax, ay), (bx, by) in sides:
        length = math.hypot(bx - ax, by - ay)
        n = max(1, int(length / (4.0 if lod == 0 else 9.0)))
        dx, dy = (bx - ax) / length, (by - ay) / length
        nx, ny = dy, -dx
        for i in range(n):
            if rng.random() < 0.18:
                continue  # blown out
            s0, s1 = i * length / n, (i + 1) * length / n
            mid = (ax + dx * (s0 + s1) / 2) / max(L, 1) + 0.5
            hmax = H * (0.25 + 0.75 * standing * (1.0 - 0.6 * math.sin(mid * math.pi) * rng.random()))
            h0 = hmax * rng.uniform(0.35, 1.0)
            h1 = hmax * rng.uniform(0.35, 1.0)
            p0 = (ax + dx * s0, ay + dy * s0)
            p1 = (ax + dx * s1, ay + dy * s1)
            wall_c = rng.choice([0xE3, 0xE4, 0xE5, 0xE6]) if rng.random() < 0.8 else SOOT
            pts = [P(p0[0], p0[1], 0), P(p1[0], p1[1], 0), P(p1[0], p1[1], h1), P(p0[0], p0[1], h0)]
            g.face(pts, wall_c, P(-nx * 5 + (p0[0] + p1[0]) / 2, -ny * 5 + (p0[1] + p1[1]) / 2, 0))
            inside = [P(p0[0] - nx * t, p0[1] - ny * t, 0), P(p1[0] - nx * t, p1[1] - ny * t, 0),
                      P(p1[0] - nx * t, p1[1] - ny * t, h1), P(p0[0] - nx * t, p0[1] - ny * t, h0)]
            g.face(inside, 0xE9, P(nx * 5 + (p0[0] + p1[0]) / 2, ny * 5 + (p0[1] + p1[1]) / 2, 0))
            # Soot over the windows.
            if lod == 0 and rng.random() < 0.5 and min(h0, h1) > 3.5:
                z = rng.uniform(1.5, min(h0, h1) - 2.0)
                sx0, sx1 = 0.2, 0.8
                q = lambda f, zz: P(p0[0] + (p1[0] - p0[0]) * f + nx * 0.05, p0[1] + (p1[1] - p0[1]) * f + ny * 0.05, zz)
                g.face([q(sx0, z), q(sx1, z), q(sx1, z + 1.8), q(sx0, z + 1.8)], SOOT,
                       P(-nx * 5 + (p0[0] + p1[0]) / 2, -ny * 5 + (p0[1] + p1[1]) / 2, z))
    # Floor slabs: some still level, some hanging down at an angle.
    if lod == 0:
        for f in range(1, floors):
            z = f * 3.0
            if z > H * standing:
                break
            if rng.random() < 0.55:
                x0 = rng.uniform(-L / 2, 0)
                x1 = x0 + rng.uniform(L * 0.2, L * 0.5)
                drop = rng.uniform(0, 2.5) if rng.random() < 0.5 else 0
                slab = [P(x0, -W / 2 + 0.5, z), P(x1, -W / 2 + 0.5, z - drop), P(x1, W / 2 - 0.5, z - drop * 0.3),
                        P(x0, W / 2 - 0.5, z)]
                g.face(slab, 0xE7, P((x0 + x1) / 2, 0, z + 10))
                g.face(list(reversed(slab)), 0xEA, P((x0 + x1) / 2, 0, z - 10))
    debris(g, P, L, W, H * (1.0 - standing) * 0.35 + 1.2, rng, lod)


def debris(g, P, L, W, h, rng, lod):
    """Rubble mounds inside and around a footprint."""
    n = 5 if lod == 0 else 2
    for _ in range(n):
        x, y = rng.uniform(-L / 2, L / 2), rng.uniform(-W / 2 - 3, W / 2 + 3)
        r = rng.uniform(3, 7) * (1.4 if lod else 1.0)
        hh = h * rng.uniform(0.5, 1.2)
        ring = [P(x + math.cos(a) * r * rng.uniform(0.7, 1.2), y + math.sin(a) * r * rng.uniform(0.7, 1.2), 0)
                for a in [k * math.tau / 6 for k in range(6)]]
        top = P(x + rng.uniform(-1, 1), y + rng.uniform(-1, 1), hh)
        for k in range(6):
            g.face([ring[k], ring[(k + 1) % 6], top], RUBBLE_C, P(x, y, -5))


def ruined_house(g, cx, cy, rng, lod):
    L, W = rng.uniform(8, 12), rng.uniform(7, 9)
    a = rng.uniform(0, math.pi)
    ruined_block(g, cx, cy, L, W, rng.uniform(2.5, 5.5), rng, lod, a)
    # The roof came down inside.
    ca, sa = math.cos(a), math.sin(a)
    P = lambda x, y, z: (cx + x * ca - y * sa, cy + x * sa + y * ca, z)
    roof = [P(-L / 2 + 1, -W / 2 + 1, 0.4), P(L / 2 - 1, -W / 2 + 1, 2.2), P(L / 2 - 1, W / 2 - 1, 1.2), P(-L / 2 + 1, W / 2 - 1, 0.3)]
    g.face(roof, rng.choice([0xD8, 0xD9, 0xE8]), P(0, 0, 20))


def wrecked_vehicle(g, cx, cy, rng):
    a = rng.uniform(0, math.pi)
    body = block(-3.2, 3.2, -1.5, 1.5, 0.3, 1.9, Ramp(0xEB, 0xEF)).rotated_z(a)
    g.add(body.scaled(1.0, (cx, cy, 0)))


def ruin_cluster(seed, lod):
    """A town quarter in ruins, about 300 m across."""
    rng = random.Random(seed)
    g = Geo()
    blocks = [(-70, 40, 0.0), (45, 60, math.pi / 2), (-20, -60, 0.1), (80, -40, 0.0)]
    for k, (x, y, a) in enumerate(blocks):
        if seed % 3 == 2 and k % 2:
            continue
        ruined_block(g, x + rng.uniform(-10, 10), y + rng.uniform(-10, 10), rng.uniform(40, 70), rng.uniform(12, 15),
                     rng.uniform(12, 27), rng, lod, a + rng.uniform(-0.1, 0.1))
    houses = 6 if lod == 0 else 3
    for _ in range(houses):
        ruined_house(g, rng.uniform(-140, 140), rng.uniform(-140, 140), rng, lod)
    if lod == 0:
        for _ in range(3):
            wrecked_vehicle(g, rng.uniform(-120, 120), rng.uniform(-120, 120), rng)
        # Scorched ground and craters.
        for _ in range(6):
            x, y, r = rng.uniform(-150, 150), rng.uniform(-150, 150), rng.uniform(4, 9)
            g.faces.append(flat([(x + math.cos(a) * r, y + math.sin(a) * r, 0.1) for a in [k * math.tau / 7 for k in range(7)]],
                                rng.choice([0xEC, 0xED, 0xCF])))
    return scaled(g, LEVEL_M[1])


def rubble_pile(lod):
    rng = random.Random(77)
    g = Geo()
    P = lambda x, y, z: (x, y, z)
    debris(g, P, 30, 22, 4.0, rng, lod)
    debris(g, P, 18, 14, 6.0, rng, lod)
    for _ in range(3):
        a = rng.uniform(0, math.pi)
        slab = block(-6, 6, -0.3, 0.3, 0, 4, GREY).rotated_x(rng.uniform(0.3, 1.0)).rotated_z(a)
        g.add(slab.scaled(1.0, (rng.uniform(-8, 8), rng.uniform(-8, 8), 0)))
    return scaled(g, LEVEL_M[1])


def terrikon(lod):
    """Slag heap of the Donbas mines: a dark cone, the top a little flattened."""
    rng = random.Random(12)
    n = 12 if lod == 0 else 7
    h, r = 60.0, 140.0
    rings = []
    for t, s in ((0.0, 1.0), (0.45, 0.62), (0.85, 0.22), (1.0, 0.1)):
        rings.append([(math.cos(a) * r * s * rng.uniform(0.92, 1.06), math.sin(a) * r * s * rng.uniform(0.92, 1.06), h * t)
                      for a in [k * math.tau / n for k in range(n)]])
    g = tube(rings, Ramp(0xEC, 0xEF), cap0=False, cap1=True)
    return scaled(g, LEVEL_M[1])


# ---------------------------------------------------------------------------------------------
# The Crimean bridge over the Kerch Strait (layout in kerch.py), built in bridge coordinates:
# s metres along, n metres to the left (north), z metres up from the sea


STEEL = Ramp(0xE0, 0xE4)
GIRDER = Ramp(0xE2, 0xE9)
ASPHALT = 0xEC
BALLAST = 0xE9
MARKING = 0xE1
RAILS = 0xEF
SAND = Ramp(0xC1, 0xC5)
SCRUB = Ramp(0x72, 0x76)
GRAVEL = 0xE7


class Raw:
    """Faces and lines in bridge coordinates, mapped into a model's space at the end (the
    wrecked arches tip their halves before that)."""

    def __init__(self):
        self.faces = []  # (points, paint, centre or None)
        self.lines = []  # (a, b, colour)

    def face(self, pts, paint, centre=None):
        self.faces.append(([Vector(p) for p in pts], paint, Vector(centre) if centre is not None else None))

    def line(self, a, b, c):
        self.lines.append((Vector(a), Vector(b), c))

    def prism(self, bottom, top, paint, top_paint=None, ends=True, caps=(True, True)):
        """Solid between two 4-point rings (bottom, top; same winding); `caps` leaves out the
        bottom or top face where nothing can see it."""
        c = sum((Vector(p) for p in bottom + top), Vector()) / 8
        if caps[1]:
            self.face(top, top_paint if top_paint is not None else paint, c)
        if caps[0]:
            self.face(bottom, paint, c)
        for k in range(4):
            if not ends and k % 2 == 1:
                continue
            k1 = (k + 1) % 4
            self.face([bottom[k], bottom[k1], top[k1], top[k]], paint, c)

    def box(self, s0, s1, n0, n1, z0, z1, paint, top_paint=None, caps=(True, True)):
        self.prism([(s0, n0, z0), (s1, n0, z0), (s1, n1, z0), (s0, n1, z0)],
                   [(s0, n0, z1), (s1, n0, z1), (s1, n1, z1), (s0, n1, z1)], paint, top_paint, caps=caps)

    def add(self, other):
        self.faces += other.faces
        self.lines += other.lines

    def moved(self, fn):
        out = Raw()
        out.faces = [([fn(p) for p in pts], paint, fn(c) if c is not None else None) for pts, paint, c in self.faces]
        out.lines = [(fn(a), fn(b), c) for a, b, c in self.lines]
        return out

    def geo(self, f, unit):
        """The model geometry: `f` maps bridge coordinates to model metres, `unit` metres per
        model unit."""
        g = Geo()
        m = lambda p: Vector(f(p.x, p.y, p.z)) / unit
        for pts, paint, c in self.faces:
            g.face([m(p) for p in pts], paint, m(c) if c is not None else None)
        for a, b, c in self.lines:
            g.line(m(a), m(b), c)
        return g


def clip_s(pts, s0, s1):
    """A convex (or planar) polygon cut to s0 <= s <= s1 (Sutherland-Hodgman); [] if nothing
    is left."""
    pts = [Vector(p) for p in pts]
    for bound, keep in ((s0, lambda p: p.x >= s0), (s1, lambda p: p.x <= s1)):
        out = []
        for i in range(len(pts)):
            a, b = pts[i], pts[(i + 1) % len(pts)]
            if keep(a):
                out.append(a)
            if keep(a) != keep(b):
                out.append(a.lerp(b, (bound - a.x) / (b.x - a.x)))
        pts = out
        if len(pts) < 3:
            return []
    return pts


def clip_line(a, b, s0, s1):
    a, b = Vector(a), Vector(b)
    if a.x > b.x:
        a, b = b, a
    if b.x < s0 or a.x > s1:
        return None
    lerp = lambda s: a.lerp(b, (s - a.x) / (b.x - a.x))
    return (lerp(max(s0, a.x)), lerp(min(s1, b.x)))


def girder(raw, br, sa, sb, nc, w, depth, top, lod, rail):
    """One deck span between piers at sa and sb: a box girder, its deck, and (close up) the
    parapets, the road's median, lane lines and lamps or the railway's tracks and masts."""
    za, zb = br.deck(sa), br.deck(sb)
    wb = w * 0.55
    raw.prism([(sa, nc - wb / 2, za - depth), (sb, nc - wb / 2, zb - depth), (sb, nc + wb / 2, zb - depth),
               (sa, nc + wb / 2, za - depth)],
              [(sa, nc - w / 2, za), (sb, nc - w / 2, zb), (sb, nc + w / 2, zb), (sa, nc + w / 2, za)], GIRDER, top, ends=False)
    if lod > 0:
        return
    # Parapets: a wall on each edge, seen from outside and from the deck.
    for side in (-1, 1):
        n = nc + side * w / 2
        wall = [(sa, n, za), (sb, n, zb), (sb, n, zb + 1.2), (sa, n, za + 1.2)]
        raw.face(wall, GREY, (sa, nc, za))
        raw.face(wall, GREY, (sa, n + side * 10, za))
    if not rail:
        median = [(sa, nc, za), (sb, nc, zb), (sb, nc, zb + 0.9), (sa, nc, za + 0.9)]
        raw.face(median, GREY, (sa, nc - 10, za))
        raw.face(median, GREY, (sa, nc + 10, za))
        for off in (-7.4, -3.7, 3.7, 7.4):
            raw.line((sa, nc + off, za + 0.05), (sb, nc + off, zb + 0.05), MARKING)
        # Lamp posts on the median, their arms over both carriageways.
        raw.line((sa, nc, za + 0.9), (sa, nc, za + 11.0), 0xE5)
        raw.line((sa, nc - 2.5, za + 11.5), (sa, nc + 2.5, za + 11.5), 0xE5)
    else:
        for track in (-2.3, 2.3):
            for rail_n in (-0.76, 0.76):
                raw.line((sa, nc + track + rail_n, za + 0.25), (sb, nc + track + rail_n, zb + 0.25), RAILS)
        # Catenary masts at both edges with their cross span.
        for side in (-1, 1):
            raw.line((sa, nc + side * (w / 2 - 0.6), za), (sa, nc + side * (w / 2 - 0.6), za + 8.5), 0xE6)
        raw.line((sa, nc - w / 2 + 0.6, za + 8.0), (sa, nc + w / 2 - 0.6, za + 8.0), 0xE6)
        for track in (-2.3, 2.3):
            raw.line((sa, nc + track, za + 6.2), (sb, nc + track, zb + 6.2), RAILS)


def pier(raw, br, s, spec, rail, lod, main=False):
    """A pier under one bridge at `s`: the road's twin columns, the railway's single one,
    thicker the taller they stand, with a cap beam under the girder; `main`: an arch's."""
    nc, w, depth = spec
    top = br.deck(s) - depth
    t = max(0.0, top) / kerch.ARCH_DECK
    if main:
        raw.box(s - 6.0, s + 6.0, nc - w / 2 - 2.0, nc + w / 2 + 2.0, -4.0, top, CONCRETE, caps=(False, True))
        raw.box(s - 7.0, s + 7.0, nc - w / 2 - 3.0, nc + w / 2 + 3.0, -4.0, 1.5, CONCRETE, caps=(False, True))
        return
    cap = 1.6
    if rail:
        cs, cn = 1.4 * (1 + 0.5 * t), 2.3 * (1 + 0.4 * t)
        raw.box(s - cs, s + cs, nc - cn, nc + cn, -4.0, top - cap, CONCRETE, caps=(False, False))
        raw.box(s - 1.8, s + 1.8, nc - w / 2 + 1.0, nc + w / 2 - 1.0, top - cap, top, CONCRETE, caps=(True, False))
    else:
        cs, cn = 1.2 * (1 + 0.5 * t), 1.8 * (1 + 0.4 * t)
        cols = (-6.0, 6.0) if lod == 0 else (0.0,)
        for c in cols:
            wide = cn if lod == 0 else 7.0
            raw.box(s - cs, s + cs, nc + c - wide, nc + c + wide, -4.0, top - cap, CONCRETE, caps=(False, False))
        raw.box(s - 1.6, s + 1.6, nc - w / 2 * 0.7, nc + w / 2 * 0.7, top - cap, top, CONCRETE, caps=(True, False))


def abutment(raw, br, s):
    """Where the bridges come ashore: one block under both decks, buried in the shore."""
    top = br.deck(s) - 3.0
    sign = 1 if s < 1 else -1
    raw.box(min(s, s + sign * 25), max(s, s + sign * 25), kerch.RAIL[0] - 9, kerch.ROAD[0] + 13, -6.0, top, CONCRETE)


def approach(raw, br, s, lod):
    """The embankments up to both decks at end `s` of the bridge: from the deck's height there
    down into the ground `kerch.RAMP` metres outwards, the road's with its lane lines."""
    out = -1.0 if s < 1 else 1.0
    foot = s + out * kerch.RAMP
    for spec, rail in ((kerch.ROAD, False), (kerch.RAIL, True)):
        nc, w, _ = spec
        z = br.deck(s)
        hw, sw = w / 2, w / 2 + kerch.RAMP_SLOPE
        top = [(s, nc - hw, z), (foot, nc - hw, -0.6), (foot, nc + hw, -0.6), (s, nc + hw, z)]
        bottom = [(s, nc - sw, -1.5), (foot, nc - hw - 1.0, -1.5), (foot, nc + hw + 1.0, -1.5), (s, nc + sw, -1.5)]
        raw.prism(bottom, top, SCRUB, BALLAST if rail else ASPHALT, caps=(False, True))
        if lod == 0 and not rail:
            for off in (-7.4, -3.7, 0.0, 3.7, 7.4):
                raw.line((s, nc + off, z + 0.05), (foot, nc + off, -0.55), MARKING)


def island_rings(br):
    """Tuzla island's outline (bridge coordinates): the top ring above the sea and the beach's
    foot below it, as matching convex rings."""
    i0, i1 = br.island
    sc, half = (i0 + i1) / 2, (i1 - i0) / 2
    nc, hw = -10.0, 170.0
    top, foot = [], []
    n = 40
    for k in range(n):
        a = k * math.tau / n
        ca, sa = math.cos(a), math.sin(a)
        # A superellipse (power 4, still convex): long straight shores, rounded ends.
        u = math.copysign(abs(ca) ** 0.5, ca)
        v = math.copysign(abs(sa) ** 0.5, sa)
        foot.append((sc + u * (half + 25), nc + v * (hw + 25), -2.0))
        top.append((sc + u * (half - 25), nc + v * (hw - 25), 2.4))
    return top, foot


def dam_rings(br):
    """The dam on the Tuzla spit, beside the road bridge from the island to the Taman shore."""
    s0, s1 = br.island[1] - 150.0, br.length + 40.0
    lo, hi = kerch.ROAD[0] + 26.0, kerch.ROAD[0] + 60.0
    top = [(s0, lo + 8, 2.2), (s1, lo + 8, 2.2), (s1, hi - 8, 2.2), (s0, hi - 8, 2.2)]
    foot = [(s0, lo, -2.0), (s1, lo, -2.0), (s1, hi, -2.0), (s0, hi, -2.0)]
    return top, foot


def ground_piece(raw, top, foot, s0, s1, top_paint, side_paint, lines=None):
    """The part of an island (or dam) between s0 and s1: its top and the beach all round."""
    face = clip_s(top, s0, s1)
    if face:
        raw.face(face, top_paint, (sum(p[0] for p in face) / len(face), sum(p[1] for p in face) / len(face), -50.0))
    n = len(top)
    # The beach faces away from a point under the middle.
    mid = sum((Vector(p) for p in top), Vector()) / n
    mid.z = 0.0
    for k in range(n):
        k1 = (k + 1) % n
        for tri in ([foot[k], foot[k1], top[k1]], [foot[k], top[k1], top[k]]):
            cut = clip_s(tri, s0, s1)
            if cut:
                raw.face(cut, side_paint, mid)
    for a, b, c in lines or []:
        cut = clip_line(a, b, s0, s1)
        if cut:
            raw.line(cut[0], cut[1], c)


def viaduct(br, s0, s1, lod):
    """A segment of both bridges between piers s0 and s1, with whatever island or dam lies
    under it."""
    raw = Raw()
    a0, a1 = kerch.ARCH_S - kerch.ARCH_HALF, kerch.ARCH_S + kerch.ARCH_HALF
    piers = [p for p in br.piers() if s0 - 1e-6 <= p <= s1 + 1e-6]
    step = 1 if lod == 0 else 2
    for spec, rail in ((kerch.ROAD, False), (kerch.RAIL, True)):
        nc, w, depth = spec
        top = BALLAST if rail else ASPHALT
        for sa, sb in zip(piers, piers[1:]):
            girder(raw, br, sa, sb, nc, w, depth, top, lod, rail)
        for k, p in enumerate(piers):
            if abs(p - a0) < 1e-6 or abs(p - a1) < 1e-6:
                continue  # the arch's main pier
            if p < 1e-6 or p > br.length - 1e-6:
                continue
            if lod > 0 and k % step:
                continue
            pier(raw, br, p, spec, rail, lod)
    for end in (0.0, br.length):
        if s0 - 1e-6 <= end <= s1 + 1e-6:
            abutment(raw, br, end)
            approach(raw, br, end, lod)
    top, foot = island_rings(br)
    if s1 > br.island[0] - 60 and s0 < br.island[1] + 60:
        ground_piece(raw, top, foot, s0, s1, SCRUB, SAND)
    top, foot = dam_rings(br)
    if s1 > top[0][0]:
        mid = kerch.ROAD[0] + 43.0
        ground_piece(raw, top, foot, s0, s1, GRAVEL, SAND,
                     [((top[0][0], mid, 2.25), (top[1][0], mid, 2.25), 0xEB)] if lod == 0 else None)
    return raw


def arch_half(spec, rail, lod, side):
    """One half of an arch span (side -1 west, +1 east of the middle): deck slab, tie girders,
    rib halves, bracing and hangers. The halves meet at the crown; a destroyed arch breaks
    there."""
    raw = Raw()
    nc, w, depth = spec
    sm, half, deck, rise = kerch.ARCH_S, kerch.ARCH_HALF, kerch.ARCH_DECK, kerch.ARCH_RISE
    s_end = sm + side * half
    sa, sb = min(sm, s_end), max(sm, s_end)
    # Deck slab between the tie girders, with cross girders under it.
    raw.box(sa, sb, nc - w / 2 + 1.0, nc + w / 2 - 1.0, deck - 1.2, deck, GIRDER, BALLAST if rail else ASPHALT)
    for side_n in (-1, 1):
        n0 = nc + side_n * (w / 2 - 1.0)
        n1 = nc + side_n * (w / 2 + 0.6)
        raw.box(sa, sb, min(n0, n1), max(n0, n1), deck - depth, deck + 1.6, STEEL)
    if lod == 0:
        if rail:
            for track in (-2.3, 2.3):
                for rn in (-0.76, 0.76):
                    raw.line((sa, nc + track + rn, deck + 0.25), (sb, nc + track + rn, deck + 0.25), RAILS)
                raw.line((sa, nc + track, deck + 6.2), (sb, nc + track, deck + 6.2), RAILS)
        else:
            raw.box(sa, sb, nc - 0.4, nc + 0.4, deck, deck + 0.9, GREY)
            for off in (-7.4, -3.7, 3.7, 7.4):
                raw.line((sa, nc + off, deck + 0.05), (sb, nc + off, deck + 0.05), MARKING)
    # The ribs: a parabola rising `rise` over the deck at the crown, slender boxes leaning in
    # a little towards each other.
    stations = 12 if lod == 0 else 6
    xs = [side * half * k / stations for k in range(stations + 1)]
    xs.sort()
    lean = 2.5
    rib_n = w / 2 - 0.2

    def rib(x, side_n):
        t = 1 - (x / half) ** 2
        zc = deck + 1.0 + rise * t
        n = nc + side_n * (rib_n - lean * t)
        return zc, n

    for side_n in (-1, 1):
        for xa, xb in zip(xs, xs[1:]):
            za, na = rib(xa, side_n)
            zb, nb = rib(xb, side_n)
            d, h = 1.2, 1.7
            raw.prism([(sm + xa, na - d, za - h), (sm + xb, nb - d, zb - h), (sm + xb, nb + d, zb - h), (sm + xa, na + d, za - h)],
                      [(sm + xa, na - d, za + h), (sm + xb, nb - d, zb + h), (sm + xb, nb + d, zb + h), (sm + xa, na + d, za + h)],
                      STEEL, ends=False)
    # Bracing between the ribs where they stand high enough over the deck: struts, and
    # diagonals between them close up.
    braced = [x for x in xs if rib(x, 1)[0] > deck + 14]
    for k, x in enumerate(braced):
        if lod > 0 and k % 2:
            continue
        z, n = rib(x, 1)
        _, n2 = rib(x, -1)
        raw.box(sm + x - 0.6, sm + x + 0.6, n2, n, z - 0.6, z + 0.6, STEEL)
        if lod == 0 and k + 1 < len(braced):
            zb_, nb = rib(braced[k + 1], 1)
            _, nb2 = rib(braced[k + 1], -1)
            raw.line((sm + x, n2, z), (sm + braced[k + 1], nb, zb_), 0xE2)
            raw.line((sm + x, n, z), (sm + braced[k + 1], nb2, zb_), 0xE2)
    if lod == 0:
        # Hangers from the ribs down to the tie girders.
        count = 11
        for k in range(1, count + 1):
            x = side * half * (k - 0.5) / count
            for side_n in (-1, 1):
                z, n = rib(x, side_n)
                raw.line((sm + x, n, z - 1.7), (sm + x, nc + side_n * (w / 2 - 0.2), deck + 1.6), 0xE2)
    return raw


def tipped(raw, hinge_s, hinge_z, angle):
    """`raw` turned in the s-z plane about a hinge (positive: the side beyond the hinge in +s
    drops)."""
    c, s_ = math.cos(angle), math.sin(angle)

    def fn(p):
        ds, dz = p.x - hinge_s, p.z - hinge_z
        return Vector((hinge_s + ds * c + dz * s_, p.y, hinge_z - ds * s_ + dz * c))

    return raw.moved(fn)


def arch(br, spec, rail, lod, wrecked):
    """An arch span with its two main piers; wrecked, broken at the crown, each half hanging
    from its pier into the water, some debris round the break."""
    raw = Raw()
    a0, a1 = kerch.ARCH_S - kerch.ARCH_HALF, kerch.ARCH_S + kerch.ARCH_HALF
    for s in (a0, a1):
        pier(raw, br, s, spec, rail, lod, main=True)
    west, east = arch_half(spec, rail, lod, -1), arch_half(spec, rail, lod, 1)
    if not wrecked:
        raw.add(west)
        raw.add(east)
        return raw
    hz = kerch.ARCH_DECK - spec[2]
    # West half drops its crown end into the sea, the east one hangs at a steeper angle.
    tip_w = math.asin((kerch.ARCH_DECK + 7.0) / (kerch.ARCH_HALF - 4.0))
    tip_e = math.asin((kerch.ARCH_DECK + 2.0) / (kerch.ARCH_HALF - 4.0)) * 0.92
    raw.add(tipped(west, a0 + 4.0, hz, tip_w))
    raw.add(tipped(east, a1 - 4.0, hz, -tip_e))
    rng = random.Random(8900 + int(spec[0]))
    for _ in range(6 if lod == 0 else 2):
        s = kerch.ARCH_S + rng.uniform(-25, 25)
        n = spec[0] + rng.uniform(-spec[1], spec[1])
        piece = Raw()
        piece.box(-rng.uniform(4, 9), rng.uniform(4, 9), -rng.uniform(2, 5), rng.uniform(2, 5), -1.0, rng.uniform(1.0, 3.0),
                  rng.choice([GIRDER, STEEL]))
        a = rng.uniform(-0.5, 0.5)
        ca, sa = math.cos(a), math.sin(a)
        raw.add(piece.moved(lambda p: Vector((s + p.x * ca - p.y * sa, n + p.x * sa + p.y * ca, p.z))))
    return raw


def kerch_bridge():
    """The bridge's models: both arches (targets with a wrecked state) and the viaduct
    segments, each in metres around its snapped LOD2 origin."""
    br = kerch.Bridge(geo)
    unit = LEVEL_M[kerch.LEVEL]
    for shape, s0, s1, origin in br.pieces():
        f = br.local(origin)
        if shape in (kerch.ROAD_ARCH, kerch.RAIL_ARCH):
            rail = shape == kerch.RAIL_ARCH
            spec = kerch.RAIL if rail else kerch.ROAD
            lods = [(arch(br, spec, rail, k, False).geo(f, unit), d) for k, d in ((0, 900), (1, 32769))]
            wrecks = [arch(br, spec, rail, k, True).geo(f, unit) for k in (0, 1)]
            model(shape, lods, kerch.ARCH_ATTR, "_s1v0", wrecks=wrecks)
        else:
            lods = [(viaduct(br, s0, s1, k).geo(f, unit), d) for k, d in ((0, 700), (1, 32769))]
            model(shape, lods, 6, "_s1v0")


def main():
    out = sys.argv[sys.argv.index("--") + 2]
    mklib.load_palette(os.path.join(ROOT, "assets", "palettes", "tod_central_europe.ron"))
    models_root()
    forest_patches()
    for shape, colour in ((PLAIN_LAND, LAND), (PLAIN_SEA, SEA)):
        if shape in (LAND_TILE, SEA_TILE):
            g = Geo()
            g.faces.append(flat([(-2048, -2048), (2048, -2048), (2048, 2048), (-2048, 2048)], colour))
            model(shape, [(g, 32769)], 7)
    extras = geo.EXTRA_MODELS
    if "npp" in extras:
        for kind, shape in NPP_SHAPES.items():
            unit = LEVEL_M[2]
            model(shape, [(scaled(npp(kind, 0), unit), 6000), (scaled(npp(kind, 1), unit), 32769)], 6, "_s1v0")
    if "ruins" in extras:
        for k, shape in enumerate(RUINS):
            model(shape, [(ruin_cluster(500 + k, 0), 2500), (ruin_cluster(500 + k, 1), 32769)], 5, "_s1v0")
    if "rubble" in extras:
        model(RUBBLE, [(rubble_pile(0), 2048), (rubble_pile(1), 32769)], 3, "_s1v0")
    if "terrikon" in extras:
        model(TERRIKON, [(terrikon(0), 6000), (terrikon(1), 32769)], 6, "_s1v0")
    assert MOUNTAIN_BASE + len(geo.MOUNTAINS) <= WATER_BASE, "at most 18 mountain ranges"
    for k, (_, _, length, width, height, bearing) in enumerate(geo.MOUNTAINS):
        g = mountain(7100 + k, length * 1000.0, width * 1000.0, height, bearing)
        model(MOUNTAIN_BASE + k, [(scaled(g, LEVEL_M[4]), 32769)], 149, hull=True)
    if "kerch_bridge" in extras:
        assert MOUNTAIN_BASE + len(geo.MOUNTAINS) <= min(kerch.MODELS), "the bridge's model numbers"
        kerch_bridge()
    shapes = water_cells(geo.Water())
    bpy.ops.export_scene.gltf(
        filepath=out, export_format="GLB", export_extras=True, export_yup=True, export_texcoords=False,
        export_normals=False, export_materials="EXPORT", use_visible=False, export_animations=False,
        use_mesh_edges=True, use_mesh_vertices=True,
    )
    with open(out + ".json", "w") as f:
        json.dump({"models": META, "lod4": {"%d,%d" % k: v for k, v in shapes.items()}}, f, indent=1)


main()
