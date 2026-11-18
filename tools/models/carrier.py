"""Builds the aircraft carrier (a Nimitz-class ship) into every theater master that has one, and
saves them:

    blender -b --python-exit-code 1 --python tools/models/carrier.py

then splices each carrier model into its theater file (never export the theater masters whole,
the theater builder's output would be lost):

    blender -b assets-src/blend/<theater>.blend --python-exit-code 1 --python tools/models/export.py -- /tmp/<theater>.glb
    python3 tools/models/splice_model.py <theater> <NNN> /tmp/<theater>.glb

The ship is "USS Nimitz class aircraft carrier" by Larcenie Grant (CC BY 4.0,
https://skfb.ly/pvWrZ), reduced to assets-src/blend/nimitz.blend by tools/models/nimitz.py:
the hull, flight deck, island and masts, the deck's painted lines and the light fixtures. It is
drawn at the game's carrier size, about 1.5 times its real one: 256 model units bow to stern
with the flight deck 17 up, which is what landing on it, its deck altitude and the take-off
spawn expect (stealth_sim::catapult `WU_PER_M`, `DECK_WU` and `LANDING_BOX`). Lengths and
breadths scale alike (`U` units per metre); heights scale by `H` up to the flight deck (its
height above the water fixed at 17) and by `U` above it, so the island keeps its proportions.

Each carrier node keeps its name, its root's `attr` and the way it faces (bow along +y or -y),
and gets one detail level drawn at every distance (the ship and a dark hangar inside it). Children of the detail level: the wake behind
the stern (`_lines_pal_057`), the red masthead light, `_s10v1` the deck and island lights (lights
on), `_s11v0` / `_s11v1` the deck's painted lines by day and lit at night. The collision hull
(`planes`) is the convex hull of the ship up to the flight deck, in the glTF frame the game tests
the pilot's eye in (Q14 normals, inside when n.p + d < 0).
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

SOURCE = os.path.join(ROOT, "assets-src", "blend", "nimitz.blend")

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

LENGTH_UNITS = 256.0
DECK = 17.0  # flight deck height (model units)
DECK_M = 17.17  # its height above the waterline in nimitz.blend (m)
MAX_DIST = 32769

# Paint roles of nimitz.blend (its materials' colour bytes) -> paints.
PAINTS = {
    226: Ramp(225, 232),  # hull and island haze grey
    230: Ramp(229, 235),  # mid grey: fittings, doors
    236: Ramp(236, 238),  # flight deck
    239: 239,  # black
    219: Ramp(218, 222),  # red antifouling
    224: Ramp(224, 227),  # white
    199: Ramp(199, 201),  # yellow
    150: 150,  # window glass: the classic window colour (dark by day, amber at night)
}
# The deck's painted lines by day, and lit at night (the palette's ramps go dark at night; these
# fixed colours do not). The hull-grey paint (the stern's round-down, the bow catapults) and black
# are left out at night.
MARK_DAY = {224: 224, 226: 226, 230: 230, 199: 199, 219: 215, 239: 239}
MARK_NIGHT = {224: 7, 230: 7, 199: 19, 219: 13}
BLACK_ROLE, HULL_ROLE = 239, 226
WAKE = 57
RED = 146  # masthead lights, always lit
LIGHT = 148  # deck and island lights
# The hangar inside the hull (m: half breadth, from and to aft-fore, from and to up): the source's
# hangar walls face away from the openings (the renderer culls back faces), so a dark box inside
# them is what shows through the elevator openings.
HANGAR = (15.5, -128.0, 48.0, 5.0, 16.6)
HANGAR_PAINT = Ramp(236, 239)

class Frame:
    """nimitz.blend's metres (bow +y, starboard +x, waterline z 0) to model units."""

    def __init__(self, length_m):
        self.u = LENGTH_UNITS / length_m
        self.h = DECK / DECK_M

    def __call__(self, p):
        z = p.z * self.h if p.z <= DECK_M else DECK + (p.z - DECK_M) * self.u
        return Vector((p.x * self.u, p.y * self.u, z))


def load():
    """The source objects (appended from nimitz.blend): {name: object}."""
    with bpy.data.libraries.load(SOURCE) as (src, dst):
        dst.objects = [n for n in src.objects if n.startswith("nimitz")]
    return {ob.name: ob for ob in dst.objects}


def faces(ob, frame, paints, lift=0.0):
    """Geo of a source mesh's faces, each painted by its material's role."""
    g = Geo()
    me = ob.data
    roles = [int(m.name[4:7]) for m in me.materials]
    for p in me.polygons:
        paint = paints.get(roles[p.material_index])
        if paint is None:
            continue
        pts = [frame(me.vertices[i].co) + Vector((0, 0, lift)) for i in p.vertices]
        g.face(pts, paint)
    return g


def hangar(frame):
    x, y0, y1, z0, z1 = HANGAR
    return mklib.block(-x, x, y0, y1, z0, z1, HANGAR_PAINT).transformed(frame)


# The angled deck's painted lines the landing lights follow: a rough centre line (m, stern end
# and forward end) and the lines' offsets from it (m to starboard: the port foul line, the
# centre line, the starboard foul line). `landing_lines` fits each to the paint.
LANDING_AXIS = (Vector((4.5, -148.0)), Vector((-27.0, 55.0)))
LANDING_OFFSETS = (-8.5, 1.0, 11.5)
LIGHTS_PER_LINE = 16


def landing_lines(marks):
    """The landing area's painted lines (stern end, forward end; m, at the deck), fitted to the
    deck's paint: each line's mean offset from the rough axis, over the length it is painted."""
    a, b = LANDING_AXIS
    along = (b - a).normalized()
    side = Vector((along.y, -along.x))  # to starboard
    pts = [(marks.matrix_world @ v.co).xy for v in marks.data.vertices]
    lines = []
    for off in LANDING_OFFSETS:
        near = [((p - a).dot(along), (p - a).dot(side)) for p in pts]
        near = [(t, o) for t, o in near if abs(o - off) < 2.0 and t > -10.0]
        o = sum(o for _, o in near) / len(near)
        t0, t1 = min(t for t, _ in near), max(t for t, _ in near)
        ends = [a + along * t + side * o for t in (t0 + 2.0, t1 - 2.0)]
        lines.append(tuple(Vector((e.x, e.y, DECK_M)) for e in ends))
    return lines


# The catapults' tracks (m: the shuttle's start, the track's forward end): the bow's two as the
# source paints them, the waist's two along the angled deck (the source paints only the bow's).
# Keep in step with stealth_sim::catapult's `TRACKS`. Each is a dark groove between steel strips,
# drawn over the source's thin paint so it shows from the cockpit.
CATAPULTS = (
    ((16.7, 28.2), (5.8, 148.0)),
    ((-5.1, 27.7), (-5.1, 148.0)),
    ((-9.3, -33.4), (-22.3, 50.6)),
    ((-17.6, -44.9), (-30.6, 39.1)),
)
# (offset to starboard, half width, paint role) of a track's strips (m).
TRACK_STRIPS = ((0.0, 0.2, BLACK_ROLE), (-0.36, 0.16, HULL_ROLE), (0.36, 0.16, HULL_ROLE))


def catapult_tracks(frame, paints, lift):
    """The catapults' tracks, as the deck's marks (`paints`: the day's or the
    night's colours of the roles; a role without one is not painted)."""
    g = Geo()
    for a, b in CATAPULTS:
        a, b = Vector(a), Vector(b)
        d = (b - a).normalized()
        n = Vector((d.y, -d.x))
        for off, w, role in TRACK_STRIPS:
            paint = paints.get(role)
            if paint is None:
                continue
            pts = [a + n * (off - w), a + n * (off + w), b + n * (off + w), b + n * (off - w)]
            pts = [frame(Vector((p.x, p.y, DECK_M))) + Vector((0, 0, lift)) for p in pts]
            below = sum(pts, Vector()) / 4 - Vector((0, 0, 1))
            g.face(pts, paint, below)
    return g


def deck_lights(frame, marks):
    pts = []
    for p, q in landing_lines(marks):
        for k in range(LIGHTS_PER_LINE):
            pts.append(frame(p.lerp(q, k / (LIGHTS_PER_LINE - 1))) + Vector((0, 0, 0.1)))
    return pts


def wake(stern):
    """The wake behind the stern (drawn for a 480 unit ship, scaled to this one)."""
    k = LENGTH_UNITS / 480.0
    g = Geo()
    for sx in (-1, 1):
        g.line((sx * 17 * k, stern, 0.0), (sx * 49 * k, stern - 270 * k, 0.0), WAKE)
        g.line((sx * 9 * k, stern, 0.0), (sx * 20 * k, stern - 215 * k, 0.0), WAKE)
    return g


def planes(geo, flip):
    """Collision planes of the ship up to the flight deck (glTF frame: x, z up, -y)."""
    bm = bmesh.new()
    for pts, _ in geo.faces:
        for p in pts:
            if p.z <= DECK + 0.01:
                q = flip(p)
                bm.verts.new((q.x, q.z, -q.y))
    bmesh.ops.remove_doubles(bm, verts=bm.verts, dist=0.5)
    bmesh.ops.convex_hull(bm, input=bm.verts)
    # Merge the hull's nearly coplanar facets: a few dozen planes are enough.
    bmesh.ops.dissolve_limit(bm, angle_limit=math.radians(6.0), verts=bm.verts, edges=bm.edges)
    bm.faces.ensure_lookup_table()
    out, seen = [], set()
    for f in bm.faces:
        n = f.normal.normalized()
        d = -n.dot(f.calc_center_median())
        key = tuple(round(v, 3) for v in (*n, d / 100))
        if key in seen:
            continue
        seen.add(key)
        out.append([round(n.x * 16384), round(n.y * 16384), round(n.z * 16384), round(d * 16384)])
    bm.free()
    return out


def orientation(lod):
    """+1 if the old model's bow points along +y (its wake trails to -y, reaching farther from
    the middle than the bow), else -1."""
    ys = []
    for ob in [lod] + list(lod.children_recursive):
        if ob.type == "MESH":
            ys += [(ob.matrix_world @ v.co).y for v in ob.data.vertices]
    return 1 if -min(ys) > max(ys) else -1


def group(lod, name, conds):
    g = bpy.data.objects.new(name, None)
    g["conds"] = conds
    for col in lod.users_collection:
        col.objects.link(g)
    g.parent = lod
    return g


def build(blend):
    mklib.load_palette(os.path.join(ROOT, "assets", "palettes", f"tod_{blend}.ron"))
    bpy.ops.wm.open_mainfile(filepath=os.path.join(ROOT, "assets-src", "blend", f"{blend}.blend"))
    src = load()
    body, marks = src["nimitz"], src["nimitz_marks"]
    ys = [v.co.y for v in body.data.vertices]
    frame = Frame(max(ys) - min(ys))
    ship = faces(body, frame, PAINTS)
    ship.add(hangar(frame))
    day = faces(marks, frame, MARK_DAY, lift=0.06).add(catapult_tracks(frame, MARK_DAY, 0.1))
    night = faces(marks, frame, MARK_NIGHT, lift=0.06).add(catapult_tracks(frame, MARK_NIGHT, 0.1))
    lights = {LIGHT: deck_lights(frame, marks)}
    for name, ob in src.items():
        if name.startswith("nimitz_lights_"):
            lights.setdefault(int(name[-3:]), []).extend(frame(v.co) for v in ob.data.vertices)
    # The red light on the masthead (burning by day too): the ship's highest point.
    masthead = [max((p for f, _ in ship.faces for p in f), key=lambda p: p.z) + Vector((0, 0, 0.5))]
    stern = min(p.y for f, _ in ship.faces for p in f)
    trail = wake(stern)
    radius = math.ceil(max(heli.radius(ship), max(Vector(p).length for segs in trail.lines.values() for s in segs for p in s)))
    for name in CARRIERS[blend]:
        root = bpy.data.objects[name]
        old = bpy.data.objects[f"{name}_lod0"]
        bow = orientation(old)
        flip = (lambda p: Vector(p)) if bow > 0 else (lambda p: Vector((-p[0], -p[1], p[2])))
        # One detail level, drawn at every distance.
        for c in list(root.children):
            mklib.delete_tree(c)
        root["lod_dists"] = [MAX_DIST]
        lod = heli.mesh_with_lines(root, f"{name}_lod0", ship.transformed(flip))
        lod["max_dist"] = MAX_DIST
        lod["radius"] = radius
        lod["planes"] = planes(ship, flip)
        for c, segs in trail.transformed(flip).lines.items():
            ln = f"{lod.name}_lines_pal_{c:03}"
            mklib.link_child(lod, ln, mklib.lines_mesh(ln, segs, c))
        dn = f"{lod.name}_dots_pal_{RED:03}"
        mklib.link_child(lod, dn, mklib.points_mesh(dn, [flip(p) for p in masthead], RED))
        heli.lights(lod, 10, {c: [flip(p) for p in pts] for c, pts in lights.items()})
        for value, geo in ((0, day), (1, night)):
            g = group(lod, f"{lod.name}_s11v{value}", [[11, value]])
            mn = f"{g.name}_marks"
            mklib.link_child(g, mn, mklib.faces_mesh(mn, geo.transformed(flip)))
        print(f"{name}: bow {'+' if bow > 0 else '-'}y, {len(ship.faces)} faces, {len(day.faces)} marks, "
              f"{len(lod['planes'])} planes, radius {radius}")
    for ob in src.values():
        data = ob.data
        bpy.data.objects.remove(ob)
        if data is not None and data.users == 0:
            bpy.data.meshes.remove(data)
    for m in list(bpy.data.materials):
        if m.users == 0 and not m.name.startswith("pal_"):
            bpy.data.materials.remove(m)
    bpy.ops.wm.save_mainfile()


for blend in sys.argv[sys.argv.index("--") + 1:] if "--" in sys.argv else CARRIERS:
    build(blend)
