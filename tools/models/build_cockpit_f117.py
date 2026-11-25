# Builds the F-117A's modern cockpit master (assets-src/blend/cockpit_modern.blend) from the
# contributed cockpit model and the panel layout:
#   blender -b assets-src/blend/cockpit_base.blend --python-exit-code 1 \
#       --python tools/models/build_cockpit_f117.py -- \
#       assets-src/blend/cockpit_modern_layout.json assets-src/blend/cockpit_modern.blend
# then tools/models/export.sh cockpit_modern.
#
# The cockpit itself is the contributed model's (assets-src/blend/f117a.blend, which
# tools/models/f117_source.py makes of the Sketchfab "F-117 Nighthawk (War Thunder)" by Köjf,
# CC BY 4.0): the tub with its consoles, the panel with its bezels, cases, knobs and switches,
# the coaming with the HUD's mount and the canopy's frame, in the game's cockpit materials and
# without the source's pictures; and its stick, which becomes the game's (without the box it
# stands in: its column goes down to the floor). The seat and what was behind it are left out
# there.
#
# What the game works is put on it here, from the faces and displays of `PanelLayout::modern`
# (src/cockpit3d/layout.rs; the layout JSON is written by `UPDATE_LAYOUT=1 cargo test
# layout_files`). The panel has three flat parts: its middle and its two wings, which stand at
# an angle to it; the right console's top is a fourth (`SECTIONS`; the layout's x says which:
# `section`). Each face is stood on whatever the model has under it, on the highest of it, as
# that surface lies (`seat`): a display on its glass, the data entry display in the up-front
# controls' glass, a bezel button on the model's button, the warning lights in the cells of
# the model's light panel and the engine and fuel instruments on their plates (turned as these
# are: `SPIN`; the plates show through the instruments: `CUT`), the dials on the panel with a
# thin rim, a few groups of faces on a plate (`PLATES`; the countermeasures' on the right
# console). Where the model's own parts would be in the way, or are only clutter, they are
# taken out first (`CLEAR`: the up-front display's keys, the countermeasures' case, the
# dashboard's fittings over the MFDs, the switches of the panel under the left MFD, three knobs
# under the centre display, a lever by the gear lights' first place, a button and the pod they
# stood in, the RPM indicator's pod and the plate under it, the pods under the EMV gauge), or
# made one plain box (`BOXED`: the up-front controls' plate under the data entry display, the
# panel low on the left wing with its gear lever, a handle, switches and buttons, the round
# instruments' pods with their bezels, cases and knobs), or made flat (`FLATTEN`: a few
# recesses, and the fuel quantity indicator's plate with its round pocket, one face).
#
# From the parts file (cockpit_base.blend, the F-117A's first modern cockpit, which the other
# cockpits' builders also take their seat and levers from) come, with their materials, labels
# and control properties, brought to life size: the seat with the ejection handle (simple,
# as the game's others: the model's seat frame and the machinery behind it are gone), the
# throttle with its rails forward on the left console, the gear handle low on the panel's left
# wing where the model's gear lever was, the five target buttons on the left console behind the
# throttle, the approach indexer and the flood lamps; and the HUD glass's material, on the
# model's HUD glass (the HUD shows only through it).
#
# The F-117A's cockpits are built larger than life (`AircraftSpec::cockpit_size`, after the
# classic one): everything here is life size, the contributed model's metres, and scaled by
# SCALE about the eye at the end (but the indexer and the lamps, which stay where the parts
# file has them). Blender frame: eye at the origin, x right, y forward, z up,
# metres. A control's face points along its local -Y (towards the pilot), local Z is up the
# panel.
import json
import math
import os
import sys

import bmesh
import numpy
import bpy
from mathutils import Euler, Matrix, Quaternion, Vector
from mathutils.bvhtree import BVHTree

layout_path, out = sys.argv[sys.argv.index("--") + 1:][:2]
layout = json.load(open(layout_path))
root = bpy.data.objects["cockpit"]
SOURCE = os.path.join(os.path.dirname(os.path.abspath(layout_path)), "f117a.blend")

# The F-117A cockpit's size over life size (`AircraftSpec::cockpit_size`).
SCALE = 1.4706
# Segments of the round parts (bezels, buttons, knobs): low-poly.
ROUND = 16
KNOB = 8

# The panel's flat parts, as the contributed model has them (m about the eye): the plane's
# normal towards the pilot and its distance (normal . point), and the x of the line the
# layout's x is counted from, on the level Z0 under the eye where the layout's y is 0.
Z0 = -0.45


def plane(normal, d, x0):
    """(origin, across to the right, normal towards the pilot, up the panel)."""
    n = Vector(normal).normalized()
    h = Vector((-n.y, n.x, 0.0)).normalized()
    if h.x < 0:
        h = -h
    up = h.cross(n)
    if up.z < 0:
        up = -up
    return Vector((x0, (d - n.x * x0 - n.z * Z0) / n.y, Z0)), h, n, up


SECTIONS = {
    0: plane((0.0, -0.9279, 0.3728), -1.0948, 0.0),
    -1: plane((0.4217, -0.8758, 0.2346), -1.0766, -0.10),
    1: plane((-0.4217, -0.8758, 0.2346), -1.0766, 0.10),
}
# The consoles' tops fall 15 degrees towards the seat: z at x.
CONSOLE = (0.2588, 0.9195, 0.9659)


def console_z(x):
    return (-CONSOLE[1] + CONSOLE[0] * abs(x)) / CONSOLE[2]


# The right console's top as a part of the panel (2): the layout's x 3000 mm at its inner edge
# (CONSOLE_X), outwards up its slope; its y aft from its front end (CONSOLE_Y), "up" forward.
CONSOLE_X, CONSOLE_Y = 0.335, 0.68
_tilt = math.atan2(CONSOLE[0], CONSOLE[2])
SECTIONS[2] = (
    Vector((CONSOLE_X, CONSOLE_Y, console_z(CONSOLE_X))),
    Vector((math.cos(_tilt), 0.0, math.sin(_tilt))),
    Vector((-math.sin(_tilt), 0.0, math.cos(_tilt))),
    Vector((0.0, 1.0, 0.0)),
)
# Where each part's x starts (mm).
SECTION_X = {0: 0.0, -1: -1000.0, 1: 1000.0, 2: 3000.0}


def section(u):
    """-1 / 1 the left / right wing (the layout's x 1000 mm out from the wing's inner edge), 0
    the middle, 2 the right console's top (x 3000 mm and more)."""
    return 2 if u >= 3000.0 else -1 if u <= -1000.0 else 1 if u >= 1000.0 else 0


def point(u, v, offset=0.0):
    """A panel point: layout millimetres (u right, v down), `offset` m off its part's plane."""
    sec = section(u)
    o, h, n, up = SECTIONS[sec]
    return o + h * ((u - SECTION_X[sec]) / 1000) - up * (v / 1000) + n * offset


def rotation(u):
    o, h, n, up = SECTIONS[section(u)]
    return Matrix((h, -n, up)).transposed().to_quaternion()


def rect_point(r, offset=0.0):
    x, y, w, h = r
    return point(x + w / 2.0, y + h / 2.0, offset)


# --- the parts file: keep, drop
KEEP = (
    "seat_", "cushion_", "metal_98", "metal_99", "control_Action_Eject_", "throttle",
    "black_549", "metal_550", "metal_551", "hud_glass", "approach", "lamp", "control_Action_Gear_.001",
    "button_NEAR", "button_NEXT", "button_NONE", "button_WPN", "button_DTL",
)
# Of those, left where and as large as they are.
AS_IS = ("approach", "lamp")
SEAT = ("seat_", "cushion_", "metal_98", "metal_99", "control_Action_Eject_")
THROTTLE = ("throttle", "black_549", "metal_550", "metal_551")


def kept(o):
    return o.name.startswith(KEEP)


def drop(o):
    for c in list(o.children):
        drop(c)
    bpy.data.objects.remove(o, do_unlink=True)


for o in list(root.children):
    if not kept(o):
        drop(o)
fixed = {o for o in root.children if o.name.startswith(AS_IS)}

# --- materials
mats = bpy.data.materials


def material(name, like):
    m = mats.get(name)
    if m is None:
        m = mats[like].copy()
        m.name = name
    return m


for n, like in (
    ("ded", "adi"), ("screen_third", "screen_right"), ("trackcam_left", "trackcam"),
    ("trackcam_third", "trackcam"),
):
    material(n, like)


def coloured(name, like, rgb):
    """A material of its own colour (linear RGB), not one of the game's shared ones: the game
    takes its colour from the model."""
    m = material(name, like)
    m.diffuse_color = (*rgb, 1.0)
    for nd in m.node_tree.nodes if m.use_nodes else ():
        if nd.type == "BSDF_PRINCIPLED":
            nd.inputs["Base Color"].default_value = m.diffuse_color
    return m


# Guard covers and red buttons.
coloured("red", "yellow", (0.55, 0.04, 0.03))
# The panel's face, lighter than the shared panel grey (the game's own `panel_light`, which
# darkens to the grey at night): the F-117A's panel lies deep in the
# shade of its glare shield (about sRGB 0.30 / 0.32 / 0.34).
coloured("panel_light", "panel_grey", (0.073, 0.084, 0.095))

# --- the contributed cockpit and its stick, in this file's materials
with bpy.data.libraries.load(SOURCE) as (src, dst):
    dst.objects = ["cockpit", "cockpit_stick", "hud"]
shell, stick_mesh, hud_source = dst.objects
for o in (shell, stick_mesh):
    bpy.context.scene.collection.objects.link(o)
    for slot in o.material_slots:
        name = slot.material.name.split(".")[0]
        own = mats.get(name) if mats.get(name) is not slot.material else None
        if own is None:
            # Not in the parts file: a copy of the nearest one that is.
            slot.material.name = name + "_source"
            own = material(name, "dark")
        slot.material = own
shell.name = "tub"
shell.parent = root


def loose_parts(bm):
    """A mesh's loose parts (lists of faces)."""
    seen, parts = set(), []
    for f in bm.faces:
        if f in seen:
            continue
        seen.add(f)
        part, todo = [], [f]
        while todo:
            g = todo.pop()
            part.append(g)
            for e in g.edges:
                for h in e.link_faces:
                    if h not in seen:
                        seen.add(h)
                        todo.append(h)
        parts.append(part)
    return parts


# The model's stick stands in a box (its boot) from the floor: the box goes and the stick's
# column is drawn down to the floor, where it tilts; what the source had beside the boot,
# lower than its top (the round cap of a pull handle), is the cockpit's.
bm = bmesh.new()
bm.from_mesh(stick_mesh.data)
parts = loose_parts(bm)
boot = next(p for p in parts if min(v.co.z for f in p for v in f.verts) < 0.01)
boot_top = max(v.co.z for f in boot for v in f.verts)
beside = [f for p in parts if p is not boot and max(v.co.z for f in p for v in f.verts) < boot_top for f in p]
cap = bmesh.new()
cap.from_mesh(stick_mesh.data)
cap.faces.ensure_lookup_table()
keep = {f.index for f in beside}
bmesh.ops.delete(cap, geom=[f for f in cap.faces if f.index not in keep], context="FACES")
loose = bpy.data.meshes.new("stick_beside")
cap.to_mesh(loose)
cap.free()
loose.transform(Matrix.Translation(stick_mesh.location - shell.location))
tub = bmesh.new()
tub.from_mesh(shell.data)
tub.from_mesh(loose)
bpy.data.meshes.remove(loose)
# The rudder pedals with their mechanism on the floor (the loose parts wholly inside one of
# PEDAL_ASSEMBLY: the floor under the panel, and above it, clear of the panel's lowest
# controls, where the pedals stand) go: a flat upright board stands for each pedal (PEDALS,
# where its upright frame was: x0, y0, z0 and x1, y1, z1).
PEDAL_ASSEMBLY = [((-0.3, 0.77, -1.2), (0.3, 1.4, -1.0)), ((-0.3, 0.9, -1.2), (0.3, 1.4, -0.84))]
PEDALS = [((0.106, 1.160, -1.036), (0.261, 1.175, -0.880)), ((-0.261, 1.160, -1.036), (-0.106, 1.175, -0.880))]
bmesh.ops.delete(tub, geom=[
    f for p in loose_parts(tub)
    if any(all(lo[i] <= v.co[i] <= hi[i] for f in p for v in f.verts for i in range(3)) for lo, hi in PEDAL_ASSEMBLY)
    for f in p
], context="FACES")
dark = [s.material for s in shell.material_slots].index(mats["dark"])
for lo, hi in PEDALS:
    lo, hi = Vector(lo), Vector(hi)
    board = bmesh.ops.create_cube(tub, size=1.0, matrix=Matrix.Translation((lo + hi) / 2) @ Matrix.Diagonal((hi - lo).to_4d()))
    for f in {f for v in board["verts"] for f in v.link_faces}:
        f.material_index = dark
        f.smooth = False
tub.to_mesh(shell.data)
tub.free()
bmesh.ops.delete(bm, geom=boot + beside, context="FACES")
# The column's open foot, extended straight down to the floor and closed.
low = min(v.co.z for v in bm.verts)
ring = [e for e in bm.edges if e.is_boundary and all(v.co.z < low + 0.005 for v in e.verts)]
new = bmesh.ops.extrude_edge_only(bm, edges=ring)["geom"]
feet = [g for g in new if isinstance(g, bmesh.types.BMVert)]
for v in feet:
    v.co.z = 0.0
for g in new:
    if isinstance(g, bmesh.types.BMFace):
        g.smooth = True
    elif isinstance(g, bmesh.types.BMEdge):
        g.smooth = True
for e in ring:
    e.smooth = True
bmesh.ops.contextual_create(bm, geom=[g for g in new if isinstance(g, bmesh.types.BMEdge) and g.is_boundary])
bm.normal_update()
# The stick tilts about the middle of the column's foot.
foot = sum((v.co for v in feet), Vector()) / len(feet)
foot.z = 0.0
bmesh.ops.translate(bm, verts=bm.verts, vec=-foot)
bm.to_mesh(stick_mesh.data)
bm.free()
stick_mesh.location += foot
# The HUD is drawn in the model's HUD glass.
hud = next(o for o in root.children if o.name.startswith("hud_glass"))
glass = hud.data.materials[0]
hud.data = hud_source.data
hud.data.materials.clear()
hud.data.materials.append(glass)
hud.location = (0.0, 0.0, 0.0)
hud.rotation_mode = "QUATERNION"
hud.rotation_quaternion = (1.0, 0.0, 0.0, 0.0)

# Loose parts of the model that are in the way of the game's own, or only clutter: those
# wholly inside a box (the layout's x0, y0, x1, y1, mm, in the box's part of the panel), their
# middle within CLEAR_DEPTH m of its plane, and no larger than the box's limit (m) go. What
# else stays is the box's `keeps`: PLATES_STAY, plates (simple boxes of at most 10 faces,
# PLATE_M across or more), on which things stand, and single flat faces (an instrument's face,
# which closes its pod); FACES_STAY, the flat faces only (the instruments' plates are their
# cases' backs, which go with the cases); NOTHING_STAYS (the dashboard's shoulders, where the
# flat faces and small plates are what is left of its fittings, and the holes they stood in
# are closed).
PLATES_STAY, FACES_STAY, NOTHING_STAYS = "plates", "faces", "nothing"
CLEAR = [
    ((-76.0, -136.0, 76.0, -112.0), 0.13, PLATES_STAY),  # the up-front display's upper keys
    ((-76.0, -90.0, 76.0, -68.0), 0.13, PLATES_STAY),  # its lower keys
    ((-62.0, 150.0, 48.0, 192.0), 0.13, PLATES_STAY),  # three knobs under the centre display
    ((-1335.0, -50.0, -1265.0, 80.0), 0.13, PLATES_STAY),  # a lever by the gear lights
    # A button by the airspeed indicator, and the pod over the attitude indicator (the gear
    # lights' first place) with its face.
    ((-1416.0, 38.0, -1383.0, 71.0), 0.13, NOTHING_STAYS),
    ((-1390.0, -20.0, -1310.0, 56.0), 0.13, NOTHING_STAYS),
    # The dashboard's switches, lights and fittings over the MFDs, either side of the
    # up-front display and the HUD's mount.
    ((-620.0, -320.0, -95.0, -64.0), 0.12, NOTHING_STAYS),
    ((95.0, -320.0, 620.0, -64.0), 0.12, NOTHING_STAYS),
    ((-322.0, 174.0, -142.0, 270.0), 0.10, PLATES_STAY),  # the switches of the panel under the left MFD
    ((1380.0, 85.0, 1505.0, 230.0), 0.19, PLATES_STAY),  # the countermeasures' case and its knob
    # The three round pods and the plate with its screws under the EMV gauge's row.
    ((140.0, 130.0, 330.0, 176.0), 0.1, NOTHING_STAYS),
    # The RPM indicator's pod and the plate under it with its two round recesses.
    ((1255.0, -60.0, 1385.0, 90.0), 0.16, NOTHING_STAYS),
]
# The angle of attack and vertical velocity tapes (layout mm, both).
TAPES = (-1354.0, 156.0, 48.0, 60.0)
# The model's pod for them stands a little off them (up and outboard of the other
# instruments' grid): their box reaches this much further (mm).
TAPES_POD_REACH = 8.0
# Kept all the same (the layout's bounds of the part, mm): the countermeasures' plate.
UNCLEARED = ((1392.4, 101.6, 1492.6, 217.1),)
CLEAR_DEPTH = 0.36
PLATE_M = 0.05
# The most sides of a hole closed where nothing stays.
HOLE_SIDES = 16
# Clutter that becomes one box (as CLEAR, the box's limit): the loose parts inside go, and the
# largest of them (its plate, or its pod) is made a plain box, as it lies: the up-front
# controls' plate under the data entry display, with its knobs, switches and screws; the
# panel low on the left wing; the round instruments' (and the tapes') pods with their bezels, cases and knobs, this far (mm)
# round them.
# The dials that stand flush on the panel instead (their pods go: `CLEAR`).
FLUSH = ("Item(Dial(Rpm))",)
BOXED = [
    ((-92.0, -70.0, 110.0, -14.0), 0.2),
    # The panel low on the left wing, with its gear lever, a yellow handle, switches and
    # buttons.
    ((-1505.0, 220.0, -1255.0, 430.0), 0.35),
]
CASE_REACH = 35.0
# The pods on the left wing are made all alike: squares round their faces' middles (BOXED's
# index, the face's rect), POD_SIZE across (layout mm): the largest face's and POD_RIM round
# it.
POD_RIM = 7.0
SIZED = {}
for r in [f["rect"] for f in layout["faces"] if f["round"] and f["key"] not in FLUSH] + [layout["screens"]["adi"], TAPES]:
    x, y, w, h = r
    if section(x) == -1:
        SIZED[len(BOXED)] = r
    reach = CASE_REACH + (TAPES_POD_REACH if r == TAPES else 0.0)
    BOXED.append(((x - reach, y - reach, x + w + reach, y + h + reach), 0.2))
POD_SIZE = max(max(w, h) for _, _, w, h in SIZED.values()) + 2 * POD_RIM
# A box is at least this thick (m): a pod that is only its face.
BOX_MIN = 0.003
# Recesses in the model's panel made flat (layout mm, as CLEAR): what is inside a box is
# brought out (or in) along its part's normal onto the plane where its edge meets the panel
# round it (of the points inside with an edge going out), and its faces made one: the gear
# lights' first pod's and the button's beside it, the centre display's sunken glass, and the
# fuel quantity indicator's plate (with a round pocket in it): a box can lie turned as its
# plate (degrees, as `SPIN`), about its middle.
FLATTEN = [
    ((-1390.0, -22.0, -1310.0, 58.0), 0.0),
    ((-1416.0, 38.0, -1385.0, 71.0), 0.0),
    ((-114.0, 4.0, 114.0, 173.0), 0.0),
    ((1395.5, 103.0, 1489.5, 215.0), 6.4),
]
# How far off its part's plane (m) a point can be and still be flattened, and the most sides
# of the face it makes.
FLATTEN_DEPTH = 0.08
FLATTEN_SIDES = 32


def layout_bounds(vs, sec):
    o, h, n, up = SECTIONS[sec]
    us = [(v.co - o).dot(h) * 1000 + SECTION_X[sec] for v in vs]
    ws = [-(v.co - o).dot(up) * 1000 for v in vs]
    return min(us), min(ws), max(us), max(ws)


def hull(vs, sec):
    """The points of vs round their outline in the layout's plane, in order."""
    pts = sorted(vs, key=lambda v: layout_bounds([v], sec)[:2])

    def cross(a, b, c):
        (ax, ay), (bx, by), (cx, cy) = (layout_bounds([v], sec)[:2] for v in (a, b, c))
        return (bx - ax) * (cy - ay) - (by - ay) * (cx - ax)

    def half(ps):
        out = []
        for p in ps:
            while len(out) >= 2 and cross(out[-2], out[-1], p) <= 0:
                out.pop()
            out.append(p)
        return out[:-1]

    return half(pts) + half(pts[::-1])


def boxify(part, sec, face=None):
    """The box a part makes (a unit cube's matrix) and its material: turned as its largest
    face, across as its part of the panel, over the part's extent (across and up, POD_SIZE
    square round a face's middle instead); in its material with the most area."""
    o, h, n, up = SECTIONS[sec]
    fronts = [f for f in part if abs(f.normal.dot(n)) > 0.7]
    big = max(fronts, key=lambda f: f.calc_area()) if fronts else None
    normal = n if big is None else big.normal.copy() if big.normal.dot(n) > 0 else -big.normal
    across = (h - normal * h.dot(normal)).normalized()
    upward = normal.cross(across)
    if upward.dot(up) < 0:
        upward = -upward
    axes = (across, normal, upward)
    vs = [v.co.copy() for v in {v for f in part for v in f.verts}]
    lo = [min(v.dot(a) for v in vs) for a in axes]
    hi = [max(v.dot(a) for v in vs) for a in axes]
    if face is not None:
        x, y, w, h = face
        mx, my, r = x + w / 2, y + h / 2, POD_SIZE / 2
        corners = [point(u, v) for u in (mx - r, mx + r) for v in (my - r, my + r)]
        for i in (0, 2):
            lo[i], hi[i] = min(c.dot(axes[i]) for c in corners), max(c.dot(axes[i]) for c in corners)
    centre = sum(((l + u) / 2 * a for l, u, a in zip(lo, hi, axes)), Vector())
    rot = Matrix((across, -normal, upward)).transposed().to_4x4()
    m = Matrix.Translation(centre) @ rot @ Matrix.Diagonal((*(max(u - l, BOX_MIN) for l, u in zip(lo, hi)), 1.0))
    area = {}
    for f in part:
        area[f.material_index] = area.get(f.material_index, 0.0) + f.calc_area()
    return m, max(area, key=area.get)


def clear():
    bm = bmesh.new()
    bm.from_mesh(shell.data)
    seen, gone, plates_of = set(), [], {}
    for f in bm.faces:
        if f in seen:
            continue
        part, stack = [], [f]
        seen.add(f)
        while stack:
            g = stack.pop()
            part.append(g)
            for e in g.edges:
                for k in e.link_faces:
                    if k not in seen:
                        seen.add(k)
                        stack.append(k)
        vs = {v for g in part for v in g.verts}
        lo = Vector([min(v.co[i] for v in vs) for i in range(3)])
        hi = Vector([max(v.co[i] for v in vs) for i in range(3)])
        size = (hi - lo).length
        plate = len(part) <= 10 and size >= PLATE_M
        face_plate = len(part) <= 2
        c = (lo + hi) / 2
        boxed = False
        for k, ((x0, y0, x1, y1), limit) in enumerate(BOXED):
            sec = section(x0)
            if size <= limit and abs((c - SECTIONS[sec][0]).dot(SECTIONS[sec][2])) < CLEAR_DEPTH:
                u0, v0, u1, v1 = layout_bounds(vs, sec)
                if x0 <= u0 and u1 <= x1 and y0 <= v0 and v1 <= y1:
                    if k not in plates_of or size > plates_of[k][0]:
                        plates_of[k] = (size, part)
                    gone += part
                    boxed = True
                    break
        if boxed:
            continue
        for (x0, y0, x1, y1), limit, keeps in CLEAR:
            if (plate and keeps == PLATES_STAY) or (face_plate and keeps != NOTHING_STAYS):
                continue
            sec = section(x0)
            o, h, n, up = SECTIONS[sec]
            if size > limit or abs((c - o).dot(n)) >= CLEAR_DEPTH:
                continue
            u0, v0, u1, v1 = layout_bounds(vs, sec)
            if not (x0 <= u0 and u1 <= x1 and y0 <= v0 and v1 <= y1):
                continue
            if any(max(abs(a - b) for a, b in zip((u0, v0, u1, v1), k)) < 3.0 for k in UNCLEARED):
                continue
            gone += part
            break
    blocks = [boxify(plates_of[k][1], section(BOXED[k][0][0]), SIZED.get(k)) for k in sorted(plates_of)]
    bmesh.ops.delete(bm, geom=gone, context="FACES")
    for m, mat in blocks:
        made = bmesh.ops.create_cube(bm, size=1.0, matrix=m)
        for f in {f for v in made["verts"] for f in v.link_faces}:
            f.material_index = mat
    # Where nothing stays, the holes the fittings stood in are closed.
    def inside(v, box):
        x0, y0, x1, y1 = box
        u0, v0, u1, v1 = layout_bounds([v], section(x0))
        return x0 <= u0 <= x1 and y0 <= v0 <= y1
    rims = [e for e in bm.edges if e.is_boundary and any(keeps == NOTHING_STAYS and all(inside(v, box) for v in e.verts) for box, _, keeps in CLEAR)]
    for f in bmesh.ops.holes_fill(bm, edges=rims, sides=HOLE_SIDES)["faces"]:
        # In the material round it.
        f.material_index = next(k.material_index for e in f.edges for k in e.link_faces if k is not f)
    def inside_turned(v, box, deg):
        x0, y0, x1, y1 = box
        mx, my = (x0 + x1) / 2, (y0 + y1) / 2
        u0, v0, _, _ = layout_bounds([v], section(x0))
        a = math.radians(deg)
        du, dv = u0 - mx, v0 - my
        x, y = du * math.cos(a) - dv * math.sin(a), du * math.sin(a) + dv * math.cos(a)
        return abs(x) <= (x1 - x0) / 2 and abs(y) <= (y1 - y0) / 2
    for box, deg in FLATTEN:
        o, h, n, up = SECTIONS[section(box[0])]
        ins = {v for v in bm.verts if abs((v.co - o).dot(n)) < FLATTEN_DEPTH and inside_turned(v, box, deg)}
        edge = [v for v in ins if any(e.other_vert(v) not in ins for e in v.link_edges)]
        loose = not edge
        if loose:
            # A plate of its own: its outline (the hull of its points, as the pilot sees them).
            edge = hull(list(ins), section(box[0]))
        if len(edge) >= 3:
            # The edge's plane (a least squares fit).
            mid = sum((v.co for v in edge), Vector()) / len(edge)
            pn = min(
                (Vector(r) for r in numpy.linalg.svd(numpy.array([v.co - mid for v in edge]))[2]),
                key=lambda r: sum(((v.co - mid).dot(r)) ** 2 for v in edge),
            )
            for v in ins:
                v.co += n * ((mid - v.co).dot(pn) / n.dot(pn))
            # Its faces made one, in the material round it.
            inner = [f for f in bm.faces if all(v in ins for v in f.verts)]
            mat = next((f.material_index for v in edge for f in v.link_faces if f not in inner), 0)
            if loose:
                # In its own material with the most area, one face over its outline.
                areas = {}
                for f in inner:
                    areas[f.material_index] = areas.get(f.material_index, 0.0) + f.calc_area()
                mat = max(areas, key=areas.get)
                bmesh.ops.delete(bm, geom=[v for v in ins if v not in edge], context="VERTS")
                bmesh.ops.delete(bm, geom=[f for v in edge for f in v.link_faces], context="FACES_ONLY")
                f = bm.faces.new(edge)
                f.material_index = mat
                f.normal_update()
                if f.normal.dot(n) < 0:
                    f.normal_flip()
                continue
            bmesh.ops.delete(bm, geom=inner, context="FACES")
            rim = [e for e in bm.edges if e.is_boundary and all(v in ins for v in e.verts)]
            for f in bmesh.ops.holes_fill(bm, edges=rim, sides=FLATTEN_SIDES)["faces"]:
                f.material_index = mat
                if f.normal.dot(n) < 0:
                    f.normal_flip()
    bm.to_mesh(shell.data)
    bm.normal_update()
    tree = BVHTree.FromBMesh(bm)
    bm.free()
    return tree


surface = clear()

# --- the panel's colours: the model paints the panel, its bezels and cases nearly all one
# dark grey. The panel's face (what lies flat in its part's plane, up to PANEL_FACE m out of
# it) is made light grey (`panel_light`); the displays' bezels standing further out
# (BEZEL_FACE) and the round instruments' pods (`BOXED` but its first two) standing more
# than POD_FACE m out black, as the plates and pods in SUNK_PLATES.
PANEL_FACE = (-0.012, 0.045)
BEZEL_FACE = (0.045, 0.08)
POD_FACE = 0.015
# The centre display's bezel is lower over its top (its button row stands on a sloping
# shoulder): over its top edge, BEZEL_REACH mm round it, all of it from CENTRE_TOP m out to
# the bezel's front is black, its steps and slopes too.
CENTRE_TOP = 0.010
BEZEL_REACH = 45.0
_cx, _cy, _cw, _ch = layout["screens"]["right"]
CENTRE_BEZEL = (_cx - BEZEL_REACH, _cy - BEZEL_REACH, _cx + _cw + BEZEL_REACH, _cy)
PODS = BOXED[2:]
# The engine instruments' and the fuel quantity indicator's sunken plates show through their
# faces (`CUT`): black as the displays' bezels, the plates (turned: `SPIN`) and the frames'
# rims round them, what lies wholly within SUNK_REACH mm round a face; the warning lights'
# panel with its frame, round all of its cells, too.
SUNK_REACH = 30.0


def around(keys, reach=SUNK_REACH):
    """The box (layout mm) reach round the faces whose keys start with one of keys."""
    rects = [f["rect"] for f in layout["faces"] if f["key"].startswith(keys)]
    return (
        min(x for x, _, _, _ in rects) - reach,
        min(y for _, y, _, _ in rects) - reach,
        max(x + w for x, _, w, _ in rects) + reach,
        max(y + h for _, y, _, h in rects) + reach,
    )


# The warning lights' panel: its cells (`SPIN`'s first box), the lamps' upper rows and the
# unused below.
LIGHT_PANEL = (1290.0, 265.0, 1420.0, 400.0)
SUNK_PLATES = [
    around(("Item(Dial(Engines))",)),
    around(("Item(Dial(FuelQuantity))",)),
    (
        LIGHT_PANEL[0] - SUNK_REACH,
        LIGHT_PANEL[1] - SUNK_REACH,
        LIGHT_PANEL[2] + SUNK_REACH,
        LIGHT_PANEL[3] + SUNK_REACH,
    ),
]
# The round instruments' pods on the left wing (and the attitude indicator's, and the tapes'
# beside them) black too, up to CASE_REACH round their faces.
LEFT_DIALS = ("Item(Dial(Airspeed))", "Item(Dial(Clock))", "Item(Dial(Altimeter))")
SUNK_PLATES += [around((key,), CASE_REACH) for key in LEFT_DIALS] + [
    (x - CASE_REACH, y - CASE_REACH, x + w + CASE_REACH, y + h + CASE_REACH)
    for x, y, w, h in (layout["screens"]["adi"], TAPES)
]
# The plate under the left display that the flaps, brake, auto and bay buttons stand on.
SUNK_PLATES.append(around(("Button(Flaps", "Button(Brake", "Button(Auto", "Button(Bay")))


# How nearly parallel (the cosine of the angle between their normals) faces lie in one plane.
COPLANAR = 0.9995


def colour_panel():
    shell.data.materials.append(mats["panel_light"])
    slot = {s.material.name: i for i, s in enumerate(shell.material_slots)}
    grey, black = slot["panel_light"], slot["black"]
    repaint = {slot[m] for m in ("dark", "black", "frame") if m in slot}
    counts = [0, 0]
    left = set()
    for f in shell.data.polygons:
        if f.material_index not in repaint:
            continue
        o, h, n, up = SECTIONS[0]
        c = f.center
        u, v, d = (c - o).dot(h) * 1000, -(c - o).dot(up) * 1000, (c - o).dot(n)
        x0, y0, x1, y1 = CENTRE_BEZEL
        if x0 <= u <= x1 and y0 <= v <= y1 and CENTRE_TOP <= d < BEZEL_FACE[1]:
            f.material_index = black
            counts[1] += 1
            continue
        # The engine and fuel instruments' plates and frames, their walls too (all of the face
        # inside: the panel's big faces round them reach in).
        def sunk(plate):
            x0, y0, x1, y1 = plate
            o, h, n, up = SECTIONS[section(x0)]
            return all(
                x0 <= (p - o).dot(h) * 1000 + SECTION_X[section(x0)] <= x1
                and y0 <= -(p - o).dot(up) * 1000 <= y1
                and PANEL_FACE[0] <= (p - o).dot(n) < BEZEL_FACE[1]
                for p in (shell.data.vertices[i].co for i in f.vertices)
            )

        if any(sunk(p) for p in SUNK_PLATES):
            f.material_index = black
            counts[1] += 1
            continue
        sec = max((0, -1, 1), key=lambda k: f.normal.dot(SECTIONS[k][2]))
        o, h, n, up = SECTIONS[sec]
        if f.normal.dot(n) < 0.9:
            continue
        d = (c - o).dot(n)
        u, v = (c - o).dot(h) * 1000 + SECTION_X[sec], -(c - o).dot(up) * 1000
        pod = any(section(x0) == sec and x0 <= u <= x1 and y0 <= v <= y1 for (x0, y0, x1, y1), _ in PODS)
        if pod and d >= POD_FACE:
            f.material_index = black
            counts[1] += 1
        elif PANEL_FACE[0] <= d < PANEL_FACE[1]:
            f.material_index = grey
            counts[0] += 1
        elif BEZEL_FACE[0] <= d < BEZEL_FACE[1]:
            f.material_index = black
            counts[1] += 1
        else:
            left.add(f.index)
    # What is left of a light grey face's plane, though it stands further out (the centre
    # column's front, sloping steeper than the panel below its controls), light grey with it.
    polys = shell.data.polygons
    by_edge = {}
    for f in polys:
        for k in f.edge_keys:
            by_edge.setdefault(k, []).append(f.index)
    grown = True
    while grown:
        grown = False
        for i in sorted(left):
            f = polys[i]
            for k in f.edge_keys:
                if any(
                    polys[j].material_index == grey
                    and polys[j].normal.dot(f.normal) > COPLANAR
                    and abs((f.center - polys[j].center).dot(polys[j].normal)) < 0.001
                    for j in by_edge[k]
                ):
                    f.material_index = grey
                    left.discard(i)
                    counts[0] += 1
                    grown = True
                    break
    print(f"panel: {counts[0]} faces light grey, {counts[1]} bezel black")


def even_tones():
    """The model's dark tones (`DARK_TONES`) are taken from its picture face by face, which
    shades its flat parts unevenly (the coaming's shoulders round the up-front display a
    patchwork of black and dark triangles): each flat part (faces joined by their edges in one
    plane) of mostly a dark tone is all that tone."""
    slot = {s.material.name: i for i, s in enumerate(shell.material_slots)}
    tones = {slot[m] for m in DARK_TONES if m in slot}
    bm = bmesh.new()
    bm.from_mesh(shell.data)
    seen, changed = set(), 0
    for f in bm.faces:
        if f in seen:
            continue
        seen.add(f)
        part, todo = [f], [f]
        while todo:
            g = todo.pop()
            c = g.calc_center_median()
            for e in g.edges:
                for k in e.link_faces:
                    if (
                        k not in seen
                        and k.normal.dot(g.normal) > COPLANAR
                        and abs((k.calc_center_median() - c).dot(g.normal)) < 0.001
                    ):
                        seen.add(k)
                        part.append(k)
                        todo.append(k)
        area = {}
        for g in part:
            area[g.material_index] = area.get(g.material_index, 0.0) + g.calc_area()
        most = max(area, key=area.get)
        if most not in tones:
            continue
        for g in part:
            if g.material_index in tones and g.material_index != most:
                g.material_index = most
                changed += 1
    bm.to_mesh(shell.data)
    bm.free()
    print(f"tones: {changed} faces evened")


DARK_TONES = ("black", "dark", "frame")
even_tones()
colour_panel()
# How far off its plane the model can stand under a face (m): what is further out is in front
# of the panel, not on it (the up-front display stands a good way out of the panel's middle).
PROUD = (-0.15, 0.36)
REACH = 0.5


class Seat:
    """Where a face stands on the model: a point, the surface's normal there (towards the
    pilot) and the directions across and up it."""

    def __init__(self, at, h, n, up):
        self.at, self.h, self.n, self.up = at, h, n, up
        self.rot = Matrix((h, -n, up)).transposed().to_quaternion()

    def point(self, du=0.0, dv=0.0, off=0.0):
        """`du` mm right and `dv` mm down of the point, `off` m off the surface."""
        return self.at + self.h * (du / 1000) - self.up * (dv / 1000) + self.n * off


# The model's surface under a face is what stands within TOP m of its highest point there.
TOP = 0.003


def seat(r, inset=1.0, step=5.0, round=False):
    """The model's surface under the rectangle r (layout mm, less `inset`; under its inscribed
    circle for a round face): the highest of what is there, as it lies (the model's cases
    and bezels do not all lie in their part's plane)."""
    x, y, w, h = r
    o, hh, n, up = SECTIONS[section(x + w / 2)]
    hits = []
    nu = max(2, int((w - 2 * inset) / step) + 1)
    nv = max(2, int((h - 2 * inset) / step) + 1)
    for i in range(nu):
        for j in range(nv):
            fu, fv = i / (nu - 1) - 0.5, j / (nv - 1) - 0.5
            if round and fu * fu + fv * fv > 0.12:
                continue
            p = point(x + w / 2 + (w - 2 * inset) * fu, y + h / 2 + (h - 2 * inset) * fv, REACH)
            hit, normal, _, _ = surface.ray_cast(p, -n, REACH - PROUD[0])
            if hit is not None and (hit - o).dot(n) <= PROUD[1]:
                hits.append(((hit - o).dot(n), hit, normal))
    centre = rect_point(r)
    if not hits:
        return Seat(centre, hh, n, up)
    top = max(d for d, _, _ in hits)
    layer = [(q, m) for d, q, m in hits if d >= top - TOP]
    normal = sum((m for _, m in layer), Vector()).normalized()
    if normal.dot(n) < 0.8:
        normal = n
    mid = sum((q for q, _ in layer), Vector()) / len(layer)
    # The rectangle's middle, carried along its part's normal onto that surface.
    at = centre + n * ((mid - centre).dot(normal) / n.dot(normal))
    across = (hh - normal * hh.dot(normal)).normalized()
    upward = across.cross(normal)
    return Seat(at, across, normal, upward if upward.dot(up) > 0 else -upward)


# --- building blocks (as the F-16's)
def quat(rot):
    return rot if isinstance(rot, Quaternion) else rot.to_quaternion()


def mesh_object(name, mesh, loc, rot=Euler((0, 0, 0)), parent=root, props=None):
    o = bpy.data.objects.new(name, mesh)
    bpy.context.scene.collection.objects.link(o)
    o.parent = parent
    o.rotation_mode = "QUATERNION"
    o.location = loc
    o.rotation_quaternion = quat(rot)
    for k, v in (props or {}).items():
        o[k] = v
    return o


def empty(name, loc, rot=Euler((0, 0, 0)), parent=root, props=None):
    o = bpy.data.objects.new(name, None)
    bpy.context.scene.collection.objects.link(o)
    o.parent = parent
    o.rotation_mode = "QUATERNION"
    o.location = loc
    o.rotation_quaternion = quat(rot)
    for k, v in (props or {}).items():
        o[k] = v
    return o


def box_mesh(name, size, mat, centre=(0.0, 0.0, 0.0), rot=None):
    me = bpy.data.meshes.new(name)
    bm = bmesh.new()
    m = Matrix.Translation(centre) @ (rot.to_matrix().to_4x4() if rot is not None else Matrix.Identity(4)) @ Matrix.Diagonal((*size, 1.0))
    bmesh.ops.create_cube(bm, size=1.0, matrix=m)
    uv = bm.loops.layers.uv.new()
    for f in bm.faces:
        for i, l in enumerate(f.loops):
            l[uv].uv = ((0, 0), (1, 0), (1, 1), (0, 1))[i % 4]
    bm.to_mesh(me)
    bm.free()
    me.materials.append(mats[mat])
    return me


def quad_mesh(name, w, h, mat, y=0.0):
    """A quad in the local XZ plane at local y (faces -Y), UVs 0..1 with (0, 0) bottom left."""
    me = bpy.data.meshes.new(name)
    v = [(-w / 2, y, -h / 2), (w / 2, y, -h / 2), (w / 2, y, h / 2), (-w / 2, y, h / 2)]
    me.from_pydata(v, [], [(0, 1, 2, 3)])
    uv = me.uv_layers.new()
    for i, d in enumerate(uv.data):
        d.uv = ((0, 0), (1, 0), (1, 1), (0, 1))[i]
    me.materials.append(mats[mat])
    return me


def disc_mesh(name, r, mat, y=0.0, n=ROUND):
    """A ROUND-gon in the local XZ plane at local y (faces -Y); its UVs map the 0..1 square's
    inscribed circle onto it, so a square texture face shows round."""
    me = bpy.data.meshes.new(name)
    v, uvs = [], []
    for i in range(n):
        a = 2 * math.pi * i / n
        v.append((r * math.cos(a), y, r * math.sin(a)))
        uvs.append((0.5 + 0.5 * math.cos(a), 0.5 + 0.5 * math.sin(a)))
    me.from_pydata(v, [], [tuple(range(n))])
    uv = me.uv_layers.new()
    for i, d in enumerate(uv.data):
        d.uv = uvs[i]
    me.materials.append(mats[mat])
    return me


def cylinder_mesh(name, r, h, mat, axis="-Y", centre=(0.0, 0.0, 0.0), n=KNOB):
    """A closed cylinder of radius r and height h from `centre` along `axis` (local)."""
    me = bpy.data.meshes.new(name)
    bm = bmesh.new()
    rot = {"Z": Matrix.Identity(4), "-Y": Matrix.Rotation(math.radians(90.0), 4, "X")}[axis]
    m = Matrix.Translation(centre) @ rot @ Matrix.Translation((0.0, 0.0, h / 2))
    bmesh.ops.create_cone(bm, cap_ends=True, cap_tris=False, segments=n, radius1=r, radius2=r, depth=h, matrix=m)
    uv = bm.loops.layers.uv.new()
    for f in bm.faces:
        for l in f.loops:
            l[uv].uv = (0.0, 0.0)
    bm.to_mesh(me)
    bm.free()
    me.materials.append(mats[mat])
    return me


def box(name, mat, centre, size, rot=Euler((0, 0, 0)), parent=root):
    return mesh_object(name, box_mesh(name, size, mat), centre, rot, parent=parent)


# Faces drawn over the model's own plate: their atlas texels' see-through index left out.
CUT = ("Item(Dial(Engines))", "Item(Dial(FuelQuantity))")


def face(key, r, pos, rot, n, round=False):
    """An atlas face the size of the rectangle r at `pos`; round ones are discs (the dials)
    with a thin black rim."""
    x, y, w, h = r
    if round:
        me = disc_mesh("atlas", min(w, h) / 2000, "atlas")
        mesh_object("black", disc_mesh("black", min(w, h) / 2000 + RIM, "black"), pos - n * FACE_LIFT, rot, props={"no_shadow": True})
    else:
        me = quad_mesh("atlas", w / 1000, h / 1000, "atlas")
    props = {"atlas_face": key, "atlas_uv": [0.0, 0.0, 1.0, 1.0], "no_shadow": True}
    if key in CUT:
        props["atlas_cut"] = True
    mesh_object("atlas", me, pos, rot, props=props)


def control(key, r, pos, rot, effect, kind):
    """A push button: origin on the surface, its face 6 mm out; a dark cap body under the face
    and a highlight frame."""
    x, y, w, h = r
    hw, hh = w / 2000, h / 2000
    height, travel = 0.006, 0.004
    name = "control_" + "".join(ch if ch.isalnum() else "_" for ch in effect)
    c = empty(name, pos, rot, props={"control": effect, "kind": kind, "half": [hw, hh], "height": height, "travel": travel})
    moving = empty("moving", (0, 0, 0), parent=c, props={"moving": True})
    mesh_object("atlas", quad_mesh("atlas", 2 * hw, 2 * hh, "atlas", -height), (0, 0, 0), parent=moving, props={"atlas_face": key, "atlas_uv": [0.0, 0.0, 1.0, 1.0], "no_shadow": True})
    mesh_object("dark", box_mesh("dark", (2 * hw, height - 0.0004, 2 * hh), "dark", (0, -(height - 0.0004) / 2, 0)), (0, 0, 0), parent=moving)
    hl = empty("highlight", (0, 0, 0), parent=c, props={"highlight": True})
    t = 0.002
    for (cx, cz, sx, sz) in ((0, hh + t, 2 * hw + 2 * t, t), (0, -hh - t, 2 * hw + 2 * t, t), (hw + t, 0, t, 2 * hh), (-hw - t, 0, t, 2 * hh)):
        mesh_object("highlight", box_mesh("highlight", (sx, t, sz), "highlight"), (cx, -height, cz), parent=hl, props={"no_shadow": True})


def screen(mat, r, at, behind=None, round=False):
    """A display the size of the rectangle r on the surface `at`: a quad, or a disc (the
    attitude indicator, in the model's case); the tracking camera's picture just behind it."""
    x, y, w, h = r
    if round:
        mesh_object(mat, disc_mesh(mat, min(w, h) / 2000, mat), at.point(off=LIFT), at.rot, props={"no_shadow": True})
        return
    mesh_object(mat, quad_mesh(mat, w / 1000, h / 1000, mat), at.point(off=2 * LIFT), at.rot, props={"no_shadow": True})
    if behind:
        mesh_object(behind, quad_mesh(behind, w / 1000, h / 1000, behind), at.point(off=LIFT), at.rot, props={"no_shadow": True})


# --- plates: groups of faces that do not fit what the model has there stand on a plate over
# it (material, rectangle in layout mm)
PLATES = (
    ("black", TAPES),  # angle of attack and vertical velocity tapes
    ("dark", (-1399.0, 317.0, 48.0, 40.0)),  # gear lights
    ("black", (3025.0, 18.0, 130.0, 95.0)),  # the countermeasures, on the right console
)
PLATE = 0.006  # thickness
LIFT = 0.0015  # a plate's front, a display or a face over the model's surface
FACE_LIFT = 0.0005  # a face over its plate
RIM = 0.0015  # a dial's rim round its face
# The model's light panel's cells and some plates lie turned (degrees, anticlockwise as the
# pilot sees them): the faces in these boxes (layout mm) are turned with them, about their
# middles.
SPIN = (
    ((1290.0, 265.0, 1420.0, 400.0), 6.3),
    # The engine instruments' and the fuel quantity indicator's sunken plates.
    ((1270.0, 85.0, 1392.0, 250.0), 6.0),
    ((1392.0, 100.0, 1495.0, 220.0), 6.4),
)


def spun(r, at):
    """The rotation of a face over r: the surface's, turned with the model's part under it."""
    x, y, w, h = r
    for (x0, y0, x1, y1), deg in SPIN:
        if x0 <= x and x + w <= x1 and y0 <= y and y + h <= y1:
            return at.rot @ Quaternion((0.0, 1.0, 0.0), math.radians(-deg))
    return at.rot


plates = []
for mat, r in PLATES:
    at = seat(r)
    x, y, w, h = r
    box(mat, mat, at.point(off=LIFT - PLATE / 2), (w / 1000, PLATE, h / 1000), at.rot)
    plates.append((r, at))


# Faces that fill a plate sunk in a frame (layout key, mm): they stand on the plate, as it
# lies, looking this far in from their edges (not on the frame's rim round it).
SUNK = {"Item(Dial(Engines))": 12.0, "Item(Dial(FuelQuantity))": 12.0}


def stand(r, round=False, key=None):
    """Where a face over r stands: (the surface under its middle, m off it): on its plate, or
    on the model."""
    x, y, w, h = r
    for (px, py, pw, ph), at in plates:
        if px <= x and x + w <= px + pw and py <= y and y + h <= py + ph:
            return Seat(at.point(x + w / 2 - px - pw / 2, y + h / 2 - py - ph / 2), at.h, at.n, at.up), LIFT
    return seat(r, inset=SUNK.get(key, 1.0), round=round), 0.0


# --- displays and faces
scr = layout["screens"]
screen("screen_left", scr["left"], seat(scr["left"]), "trackcam_left")
screen("screen_right", scr["right"], seat(scr["right"]), "trackcam")
screen("screen_third", scr["third"], seat(scr["third"]), "trackcam_third")
screen("adi", scr["adi"], seat(scr["adi"], round=True), round=True)
at, off = stand(scr["ded"])
mesh_object("ded", quad_mesh("ded", scr["ded"][2] / 1000, scr["ded"][3] / 1000, "ded"), at.point(off=off + FACE_LIFT), at.rot, props={"no_shadow": True})
for f in layout["faces"]:
    r = f["rect"]
    at, off = stand(r, f["round"], f["key"])
    if f["control"] is None:
        face(f["key"], r, at.point(off=off + (FACE_LIFT if off else LIFT)), spun(r, at), at.n, f["round"])
    else:
        control(f["key"], r, at.point(off=off), spun(r, at), f["control"], f["kind"])

# --- the stick: the model's, tilting about the foot of its column on the floor
grip = max(v.co.z for v in stick_mesh.data.vertices) - 0.08
stick = empty("stick", stick_mesh.location.copy(), props={"stick": True})
stick_mesh.name = "stick_grip"
stick_mesh.parent = stick
stick_mesh.location = (0.0, 0.0, 0.0)
stick["grip"] = grip * SCALE

# --- the parts file's parts, which it has at the F-117A cockpit's size: to life size first
SIZE_PROPS = ("height", "travel")


def scale(o, k, done):
    """Scales an object and its children about the eye: their places, meshes and sizes."""
    o.location = o.location * k
    if o.type == "MESH" and o.data.name not in done:
        done.add(o.data.name)
        o.data.transform(Matrix.Scale(k, 4))
    for p in SIZE_PROPS:
        if p in o.keys():
            o[p] = o[p] * k
    if "half" in o.keys():
        o["half"] = [h * k for h in o["half"]]
    for c in o.children:
        scale(c, k, done)


objs = {o.name: o for o in root.children}
life = set()
for o in objs.values():
    if o not in fixed and o is not hud and o.name.startswith(KEEP):
        scale(o, 1.0 / SCALE, life)


def named(prefixes):
    return [o for n, o in objs.items() if n.startswith(prefixes)]


# The seat: larger, to sit between the consoles with the pilot's eye 0.8 m over its cushion,
# on a pedestal from the floor.
SEAT_SIZE = 1.25
FLOOR = -1.13
seated = set()
for o in named(SEAT):
    scale(o, SEAT_SIZE, seated)
bottom = min((o.matrix_world @ Vector(c)).z for o in named(("seat_",)) for c in o.bound_box)
pedestal = box("dark", "dark", (0.0, -0.12, (FLOOR + bottom) / 2), (0.34, 0.46, bottom - FLOOR))
# The throttle and its rails forward on the left console, the handle at idle level with the
# stick's grip.
THROTTLE_AT = Vector((-0.425, 0.52))  # x, y of the handle at idle
handle = objs["throttle"]
move = Vector((THROTTLE_AT.x - handle.location.x, THROTTLE_AT.y - handle.location.y, 0.0))
move.z = console_z(THROTTLE_AT.x) + 0.004 - objs["black_549"].location.z
for o in named(THROTTLE):
    o.location += move
# The target buttons on the left console behind the throttle, faces up; the gear handle low
# on the panel's left wing where the model's lever was.
tilt = math.atan2(CONSOLE[0], CONSOLE[2])
across = Vector((math.cos(tilt), 0.0, -math.sin(tilt)))
normal = Vector((math.sin(tilt), 0.0, math.cos(tilt)))
console_rot = Matrix((across, -normal, Vector((0.0, 1.0, 0.0)))).transposed().to_quaternion()
for i, n in enumerate(["button_NEAR", "button_NEXT", "button_NONE", "button_WPN", "button_DTL"]):
    o = objs[n]
    col_, row = i // 3, i % 3
    x = -0.385 - 0.08 * col_
    o.location = Vector((x, 0.37 - 0.045 * row, console_z(x))) + normal * 0.002
    o.rotation_mode = "QUATERNION"
    o.rotation_quaternion = console_rot
g = objs["control_Action_Gear_.001"]
GEAR = (-1329.0, 337.0)
at = seat((GEAR[0] - 15.0, GEAR[1] - 15.0, 30.0, 30.0))
g.location = at.point(off=0.002)
g.rotation_mode = "QUATERNION"
g.rotation_quaternion = at.rot

# --- the consoles' switch panels: the model's are flat (their switches were in its picture),
# so plates with knobs, toggle switches and guarded buttons lie on them, round the throttle
# and the target buttons left (dressing: none of them works)


def console_frame(sx):
    """A console's top (sx -1 left, 1 right): (normal, rotation of a part lying on it)."""
    n = Vector((-sx * math.sin(tilt), 0.0, math.cos(tilt)))
    return n, Euler((0.0, -sx * tilt, 0.0))


def fitting(c, p, n, rot):
    """A console fitting at p: k / K small / large knob, t / T toggle leaning forward / back, g
    red guard, r red button."""
    if c in "kK":
        r, h = (0.006, 0.010) if c == "k" else (0.009, 0.013)
        mesh_object("dark", cylinder_mesh("dark", r + 0.003, 0.002, "dark", axis="Z", n=KNOB), p, rot)
        mesh_object("metal", cylinder_mesh("metal", r, h, "metal", axis="Z", n=KNOB), p + n * 0.002, rot)
    elif c in "tT":
        mesh_object("dark", cylinder_mesh("dark", 0.005, 0.004, "dark", axis="Z", n=KNOB), p, rot)
        lean = Euler((math.radians(-30.0 if c == "t" else 30.0), 0.0, 0.0)).to_quaternion()
        mesh_object("metal", box_mesh("metal", (0.004, 0.004, 0.016), "metal", (0.0, 0.0, 0.008), lean), p + n * 0.003, rot)
    elif c == "g":
        mesh_object("red", box_mesh("red", (0.013, 0.022, 0.012), "red", (0.0, 0.0, 0.006)), p, rot)
    elif c == "r":
        mesh_object("red", cylinder_mesh("red", 0.005, 0.006, "red", axis="Z", n=KNOB), p, rot)


def console_panel(x0, x1, y0, y1, rows, h=0.005):
    """A plate over x0..x1, y0..y1 (m) on a console, its fittings in rows from the front."""
    sx = 1 if x0 > 0 else -1
    n, rot = console_frame(sx)
    at = lambda x, y, off: Vector((x, y, console_z(x))) + n * off
    w = (x1 - x0) / math.cos(tilt)
    mesh_object("black", box_mesh("black", (w, y1 - y0, h), "black"), at((x0 + x1) / 2, (y0 + y1) / 2, h / 2), rot)
    for i, row in enumerate(rows):
        y = y1 - (y1 - y0) * (i + 0.5) / len(rows)
        for j, c in enumerate(row):
            fitting(c, at(x0 + (x1 - x0) * (j + 0.5) / len(row), y, h), n, rot)


# The consoles' tops reach from |x| 0.335 to 0.515 m, from y -0.17 m to 0.68 m.
for rows, rect in (
    (["kKk", "tgt", "tttt"], (-0.515, -0.335, -0.17, 0.06)),  # behind the target buttons
    (["kk", "TT", "kr"], (-0.515, -0.335, 0.08, 0.22)),
    (["t", "k", "t"], (-0.395, -0.338, 0.49, 0.67)),  # inboard of its rails
    (["k", "t", "r", "k"], (-0.512, -0.455, 0.49, 0.67)),  # outboard of them
    (["kKk", "tTt"], (0.335, 0.515, -0.10, 0.06)),
    (["tttt", "kTk"], (0.335, 0.515, 0.09, 0.28)),
    (["kk", "tt", "rg"], (0.335, 0.515, 0.31, 0.50)),
):
    console_panel(*rect, rows)

# The approach indexer left of the HUD's mount, in a housing behind it standing on the
# glare shield (it is not scaled: its place at the cockpit's size).
ind = next(o for o in root.children if o.name.startswith("approach"))
INDEXER = Vector((-0.19, 0.88))  # x, y (m)
HOUSING = (0.040, 0.014, 0.097)
# Down onto it from under the canopy's frame.
shield = surface.ray_cast(Vector((INDEXER.x, INDEXER.y, -0.12)), Vector((0.0, 0.0, -1.0)), 1.0)[0]
ind.location = Vector((INDEXER.x * SCALE, INDEXER.y * SCALE, shield.z * SCALE + HOUSING[2] / 2))
housing = box("dark", "dark", ind.location + Vector((0.0, 0.009, 0.0)), HOUSING)
fixed.add(housing)

# --- the pilot sits EYE_DROP m lower than the contributed model's pilot (`AircraftSpec::eye`
# is as much under tools/models/f117_source.py `EYE`), to look through the HUD's glass, not
# over it: the cockpit stands as much higher about the eye, but the seat, which goes down
# with him
EYE_DROP = 0.10
seat_parts = set(named(SEAT)) | {pedestal}
for o in list(root.children):
    if o not in seat_parts:
        o.location.z += EYE_DROP * (SCALE if o in fixed else 1.0)

# --- everything to the F-117A cockpit's size: positions and meshes scaled about the eye, the
# controls' sizes with them
done = set()
for o in list(root.children):
    if o not in fixed:
        scale(o, SCALE, done)

bpy.data.orphans_purge(do_recursive=True)
bpy.ops.wm.save_as_mainfile(filepath=out)
print("built", out, "objects", len(bpy.data.objects))
