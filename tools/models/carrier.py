"""Builds the aircraft carrier (a Nimitz-class ship) once, into its own model file
assets/models/carrier.glb, and makes each theater master's carrier a reference to it:

    blender -b --python-exit-code 1 --python tools/models/carrier.py [-- theater...]

then splices each referring node into its theater file (never export the theater masters whole,
the theater builder's output would be lost):

    blender -b assets-src/blend/<theater>.blend --python-exit-code 1 --python tools/models/export.py -- /tmp/<theater>.glb
    python3 tools/models/splice_model.py <theater> <NNN> /tmp/<theater>.glb

The ship is "USS Nimitz class aircraft carrier" by Larcenie Grant (CC BY 4.0,
https://skfb.ly/pvWrZ), converted to assets-src/blend/nimitz.blend by tools/models/nimitz.py in
full detail above the waterline, with the deck's painted lines and the light fixtures. It is
drawn at the game's carrier size, about 1.5 times its real one: 256 model units bow to stern
with the flight deck 17 up, which is what landing on it, its deck altitude and the take-off
spawn expect (stealth_sim::catapult `WU_PER_M`, `DECK_WU` and `LANDING_BOX`). Lengths and
breadths scale alike (`U` units per metre); heights scale by `H` up to the flight deck (its
height above the water fixed at 17) and by `U` above it, so the island keeps its proportions.

carrier.glb holds two models: `carrier_000` with the bow north (+y) and `carrier_001` the same
meshes turned about (its detail level's node turned half round). A theater's carrier node keeps
its name and its root's `attr`, loses its children and gets `ref`, the name of the model drawn
for it (the game's `world::ShapeHandles` follows it). Each model has one detail level drawn at
every distance (the ship and a dark hangar inside it). Children of the detail level: the wake
behind the stern (`_lines_pal_057`), the red masthead light, `_s10v1` the deck and island lights
(lights on), `_s11v0` / `_s11v1` the deck's painted lines by day and lit at night. The collision
hull (`planes`) is the convex hull of the ship up to the flight deck, in the detail level's glTF
frame the game tests the pilot's eye in (Q14 normals, inside when n.p + d < 0).
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

OUT = os.path.join(ROOT, "assets", "models", "carrier.glb")
# The models of carrier.glb by the way the bow points (+1 north, -1 south).
MODELS = {1: "carrier_000", -1: "carrier_001"}
ATTR = 165  # the classic carriers' attribute byte

# The carrier nodes per theater master (both former carrier models of each theater; the Cuba and
# Korea files ship only 075) and the way their ships face (the former models' bows).
CARRIERS = {
    "libya": (-1, ["libya_063", "libya_087"]),
    "persian_gulf": (1, ["persian_gulf_073", "persian_gulf_087"]),
    "desert_storm": (1, ["desert_storm_073", "desert_storm_087"]),
    "middle_east": (1, ["middle_east_073", "middle_east_087"]),
    "north_cape": (-1, ["north_cape_061", "north_cape_072"]),
    "vietnam": (-1, ["vietnam_075", "vietnam_090"]),
    "cuba": (-1, ["cuba_075", "cuba_090"]),
    "korea": (1, ["korea_075", "korea_090"]),
}

LENGTH_UNITS = 256.0
LENGTH_M = 304.0  # nimitz.blend's bow to stern (m): LENGTH_UNITS for it (catapult::WU_PER_M)
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

    def __init__(self):
        self.u = LENGTH_UNITS / LENGTH_M
        self.h = DECK / DECK_M

    def __call__(self, p):
        z = p.z * self.h if p.z <= DECK_M else DECK + (p.z - DECK_M) * self.u
        return Vector((p.x * self.u, p.y * self.u, z))


def load():
    """The source objects (appended from nimitz.blend): {name: object}."""
    with bpy.data.libraries.load(SOURCE) as (src, dst):
        dst.objects = [n for n in src.objects if n.startswith("nimitz")]
    return {ob.name: ob for ob in dst.objects}


def faces(ob, frame, paints, lift=0.0, skip=None):
    """Geo of a source mesh's faces, each painted by its material's role (but those `skip`
    takes: role, middle (m))."""
    g = Geo()
    me = ob.data
    roles = [int(m.name[4:7]) for m in me.materials]
    for p in me.polygons:
        paint = paints.get(roles[p.material_index])
        if paint is None or (skip and skip(roles[p.material_index], p.center)):
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


# The catapults' tracks (m: the shuttle's start, the track's forward end): the bow's two where
# the source paints them, the waist's two along the angled deck (the source paints only the
# bow's). Keep in step with stealth_sim::catapult's `TRACKS`. Each is a dark groove between steel
# strips, all four alike: the source's own paint of the bow's two is left out (`source_track`).
CATAPULTS = (
    ((16.7, 28.2), (5.8, 148.0)),
    ((-5.1, 27.7), (-5.1, 148.0)),
    ((-9.3, -33.4), (-22.3, 50.6)),
    ((-17.6, -44.9), (-30.6, 39.1)),
)
# (offset to starboard, half width, paint role) of a track's strips (m).
TRACK_STRIPS = ((0.0, 0.2, BLACK_ROLE), (-0.36, 0.16, HULL_ROLE), (0.36, 0.16, HULL_ROLE))
# The tracks at night: the grey and black ramps go darker than the deck then, these fixed
# colours (black, the dim grey) keep the groove and its steel strips apart from it.
TRACK_NIGHT = {BLACK_ROLE: 0, HULL_ROLE: 8}
# The source's own bow tracks (its hull-grey paint this close to theirs, m) give way to these.
SOURCE_TRACK_M = 1.5


def source_track(role, c):
    """Whether a deck mark (its role, middle) is the source's paint of a bow catapult's track."""
    if role != HULL_ROLE:
        return False
    for a, b in CATAPULTS[:2]:
        a, b = Vector(a), Vector(b)
        d = b - a
        t = max(0.0, min(1.0, (c.xy - a).dot(d) / d.length_squared))
        if (a + d * t - c.xy).length < SOURCE_TRACK_M:
            return True
    return False


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


def planes(geo):
    """Collision planes of the ship up to the flight deck (glTF frame: x, z up, -y)."""
    bm = bmesh.new()
    for pts, _ in geo.faces:
        for p in pts:
            if p.z <= DECK + 0.01:
                bm.verts.new((p.x, p.z, -p.y))
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


def group(lod, name, conds):
    g = bpy.data.objects.new(name, None)
    g["conds"] = conds
    for col in lod.users_collection:
        col.objects.link(g)
    g.parent = lod
    return g


def turned(root, name):
    """A copy of the model `root` named `name`, sharing its meshes, its detail level turned half
    round (the bow the other way)."""

    def copy(ob, parent):
        c = ob.copy()
        c.name = name + ob.name[len(root.name):]
        for col in ob.users_collection:
            col.objects.link(c)
        c.parent = parent
        for k in list(ob.children):
            if k is not c:
                copy(k, c)
        return c

    new = copy(root, None)
    for lod in new.children:
        lod.rotation_euler.z = math.pi
    return new


def build_models():
    """The ship, both ways round, in carrier.glb."""
    bpy.ops.wm.read_factory_settings(use_empty=True)
    # The colours of the materials as seen in Blender; the game paints by colour index.
    mklib.load_palette(os.path.join(ROOT, "assets", "palettes", "tod_cuba.ron"))
    src = load()
    body, marks = src["nimitz"], src["nimitz_marks"]
    frame = Frame()
    ship = faces(body, frame, PAINTS)
    ship.add(hangar(frame))
    day = faces(marks, frame, MARK_DAY, 0.06, source_track).add(catapult_tracks(frame, MARK_DAY, 0.1))
    night = faces(marks, frame, MARK_NIGHT, 0.06, source_track).add(catapult_tracks(frame, TRACK_NIGHT, 0.1))
    lights = {LIGHT: deck_lights(frame, marks)}
    for name, ob in src.items():
        if name.startswith("nimitz_lights_"):
            lights.setdefault(int(name[-3:]), []).extend(frame(v.co) for v in ob.data.vertices)
    # The red light on the masthead (burning by day too): the ship's highest point.
    masthead = [max((p for f, _ in ship.faces for p in f), key=lambda p: p.z) + Vector((0, 0, 0.5))]
    stern = min(p.y for f, _ in ship.faces for p in f)
    trail = wake(stern)
    radius = math.ceil(max(heli.radius(ship), max(Vector(p).length for segs in trail.lines.values() for s in segs for p in s)))
    for ob in src.values():
        bpy.data.objects.remove(ob)

    root = bpy.data.objects.new(MODELS[1], None)
    bpy.context.scene.collection.objects.link(root)
    root["attr"] = ATTR
    root["lod_dists"] = [MAX_DIST]
    # One detail level, drawn at every distance.
    lod = heli.mesh_with_lines(root, f"{root.name}_lod0", ship)
    lod["max_dist"] = MAX_DIST
    lod["radius"] = radius
    lod["planes"] = planes(ship)
    for c, segs in trail.lines.items():
        ln = f"{lod.name}_lines_pal_{c:03}"
        mklib.link_child(lod, ln, mklib.lines_mesh(ln, segs, c))
    dn = f"{lod.name}_dots_pal_{RED:03}"
    mklib.link_child(lod, dn, mklib.points_mesh(dn, masthead, RED))
    heli.lights(lod, 10, lights)
    for value, geo in ((0, day), (1, night)):
        g = group(lod, f"{lod.name}_s11v{value}", [[11, value]])
        mn = f"{g.name}_marks"
        mklib.link_child(g, mn, mklib.faces_mesh(mn, geo))
    turned(root, MODELS[-1])
    print(f"{OUT}: {len(ship.faces)} faces, {len(day.faces)} marks, {len(lod['planes'])} planes, radius {radius}")
    bpy.ops.export_scene.gltf(
        filepath=OUT, export_format="GLB", export_extras=True, export_yup=True, export_texcoords=False,
        export_normals=False, export_materials="EXPORT", use_visible=False, export_animations=False,
        use_mesh_edges=True, use_mesh_vertices=True,
    )


def refer(blend):
    """The theater master's carriers become references to carrier.glb's models."""
    bpy.ops.wm.open_mainfile(filepath=os.path.join(ROOT, "assets-src", "blend", f"{blend}.blend"))
    bow, names = CARRIERS[blend]
    for name in names:
        root = bpy.data.objects[name]
        for c in list(root.children):
            mklib.delete_tree(c)
        if "lod_dists" in root:
            del root["lod_dists"]
        root["ref"] = MODELS[bow]
        print(f"{name}: {MODELS[bow]}")
    for m in list(bpy.data.materials):
        if m.users == 0 and not m.name.startswith("pal_"):
            bpy.data.materials.remove(m)
    bpy.ops.wm.save_mainfile()


build_models()
for blend in sys.argv[sys.argv.index("--") + 1:] if "--" in sys.argv else CARRIERS:
    refer(blend)
