# Builds the A-6 Intruder's two cockpit masters from the panel layouts, with parts of the F-117A's
# modern cockpit: the pilot's (left seat)
#   blender -b assets-src/blend/cockpit_base.blend --python-exit-code 1 \
#       --python tools/models/build_cockpit_a6.py -- \
#       assets-src/blend/cockpit_a6_layout.json assets-src/blend/cockpit_a6.blend
# and the bombardier/navigator's (right seat)
#   blender -b assets-src/blend/cockpit_base.blend --python-exit-code 1 \
#       --python tools/models/build_cockpit_a6.py -- \
#       assets-src/blend/cockpit_a6_bn_layout.json assets-src/blend/cockpit_a6_bn.blend \
#       --seat right
# then tools/models/export.sh cockpit_a6 cockpit_a6_bn. The layout JSONs are written by
# `UPDATE_LAYOUT=1 cargo test layout_files` from `PanelLayout::a6` / `a6_bn`
# (src/cockpit3d/a6_layout.rs). Both have the whole panel, live: the seats differ in the eye,
# the sight's glass, the approach indexer and the ejection handle's control.
#
# One cabin for both: the geometry is laid out in the exterior's frame (tools/models/aircraft.py
# `a6`: x right of the centre line, s aft of the nose, z up from the nose tip, metres) and built
# round the seat's eye (`AircraftSpec::eye` / its `gunner_seat`): the pilot's (-0.36, 3.05, 1.35),
# the bombardier/navigator's (0.36, 3.20, 1.30), 0.15 m aft of the pilot's and 0.05 m lower.
# Life size (the F-117A's cockpit is built larger than life: its kept parts are scaled down by
# F117_SCALE about the eye).
#
# Kept, with their materials and control properties: the seat with the ejection handle (copied
# for the other seat), the stick (a centre stick on the pilot's centre line), the throttle with
# its rails on the pilot's left console (in the right seat's cockpit the slew stick is the
# `stick`, the pilot's a still copy), the gear handle, the HUD glass (the optical sight's
# reflector plate), the approach indexer and the flood lamps. Built here, low-poly: the tub
# under the wide canopy (walls up to the glass's lower edge, the rear bulkhead under the
# canopy's arch), one panel across both seats under an arched glare shield, the three consoles,
# the pedestal with the computer keyboard and the slew stick between the bombardier's knees;
# on the panel the vertical display indicator (a pillowed tube face with rounded corners in its
# bezel with the bank indices), the round scopes in their heading rings (the horizontal display,
# the direct view radar indicator), the sensor scope, the round instruments and tapes, the
# square pushbuttons (`twist`), the toggle switches (`switch`) and knobs (`knob`).
#
# Blender frame: eye at the origin, x right, y forward, z up, metres. A control's face points
# along its local -Y (towards the crew), local Z is up the panel.
import json
import math
import os
import sys

import bmesh
import bpy
from mathutils import Euler, Matrix, Quaternion, Vector

args = sys.argv[sys.argv.index("--") + 1:]
layout_path, out = args[:2]
layout = json.load(open(layout_path))
RIGHT_SEAT = "--seat" in args and args[args.index("--seat") + 1] == "right"
root = bpy.data.objects["cockpit"]

F117_SCALE = 1.4706
# The seats' eyes (x, s, z).
PILOT_EYE = (-0.36, 3.05, 1.35)
BN_EYE = (0.36, 3.20, 1.30)
EYE = BN_EYE if RIGHT_SEAT else PILOT_EYE
OTHER_EYE = PILOT_EYE if RIGHT_SEAT else BN_EYE


def C(x, s, z):
    """A point of the cabin (x right of the centre line, s aft of the nose, z up) in the eye's
    frame."""
    return Vector((x - EYE[0], EYE[1] - s, z - EYE[2]))


# The main panel: the layout's origin (on the cabin's centre line), the top leaning away.
PANEL_TILT = math.radians(15.0)
PANEL_ORIGIN = C(0.0, 2.44, 0.942)
# Its outline (layout mm, the right half; the left one mirrored): the top edge an arch under
# the windscreen, down to the sills.
TOP = -200.0
BOTTOM = 240.0
ARCH = [(500.0, TOP), (620.0, -164.0), (720.0, -81.0), (800.0, 43.0), (860.0, 188.0), (885.0, BOTTOM)]
# The pedestal's sloping face between the bombardier's knees (layout y >= PED_LINE): its top
# edge (layout y PED_LINE) at PED_ORIGIN, facing up and aft.
PED_LINE = 300.0
PED_TILT = math.radians(35.0)
PED_ORIGIN = C(0.0, 2.55, 0.70)
PED_X = (0.283, 0.447)
FLOOR = 0.35
# The glass's lower edge along the cabin (s, half width, z): the walls stand up to it.
SILL = [
    (1.95, 0.74, 0.83), (2.10, 0.85, 0.76), (2.25, 0.89, 0.725), (2.45, 0.93, 0.72),
    (2.65, 0.955, 0.735), (2.80, 0.965, 0.79), (3.10, 0.985, 0.87), (3.40, 0.965, 0.98),
    (3.60, 0.95, 1.07), (3.80, 0.90, 1.19), (4.03, 0.885, 1.25),
]
WALL_IN = 0.03
# The canopy's arch at the rear bulkhead (half width, z), and the bulkhead's station.
REAR_S = 4.02
REAR_ARCH = [(0.0, 1.79), (0.25, 1.75), (0.37, 1.72), (0.49, 1.66), (0.69, 1.50), (0.76, 1.43), (0.87, 1.22)]
# The windscreen's foot (half width, z) at FOOT_S: the glare shield's back edge under it.
FOOT_S = 1.96
FOOT = [(0.0, 1.105), (0.37, 1.085), (0.44, 1.055), (0.55, 1.01), (0.64, 0.975), (0.69, 0.92), (0.73, 0.82)]
# Consoles: their top, the side ones' and the centre one's inner edges, their ends.
CONSOLE_Z = 0.705
SIDE_CONSOLE = (0.60, 0.93)
CENTRE_CONSOLE = 0.12
CONSOLE_S = (2.42, 3.70)
# Seat: its cushion's top under the eye.
SEAT_TOP = -0.80
# Centre stick: pivot on the floor on the pilot's centre line, the kept grip raised on a shaft.
STICK_S = 2.70
STICK_RAISE = 0.33
# The throttle on the pilot's left console: the handle at idle, its travel.
THROTTLE = (-0.77, 2.965, CONSOLE_Z + 0.089)
THROTTLE_TRAVEL = 0.17
# The optical sight on the glare shield on the pilot's centre line: the reflector plate's half
# width, its lower edge (s, z) and its upper one (leaning towards the pilot).
SIGHT_HALF = 0.065
SIGHT_LOW = (2.35, 1.265)
SIGHT_HIGH = (2.39, 1.395)
# Segments of the round parts (bezels, buttons, knobs): low-poly.
ROUND = 16
KNOB = 8

RIGHT = Vector((1.0, 0.0, 0.0))
UP = Vector((0.0, math.sin(PANEL_TILT), math.cos(PANEL_TILT)))
NORMAL = Vector((0.0, -math.cos(PANEL_TILT), math.sin(PANEL_TILT)))
PED_UP = Vector((0.0, math.cos(PED_TILT), math.sin(PED_TILT)))
PED_NORMAL = Vector((0.0, -math.sin(PED_TILT), math.cos(PED_TILT)))


def section(u, v):
    """1 the pedestal's face, 0 the main panel."""
    return 1 if v >= PED_LINE - 20.0 else 0


def basis(sec):
    """(across, normal towards the crew, up) of a section."""
    if sec == 1:
        return RIGHT, PED_NORMAL, PED_UP
    return RIGHT, NORMAL, UP


def point(u, v, offset=0.0):
    """A panel point: layout millimetres (u right, v down), raised `offset` m off its surface."""
    sec = section(u, v)
    _, n, up = basis(sec)
    if sec == 1:
        p = PED_ORIGIN + RIGHT * (u / 1000) - up * ((v - PED_LINE) / 1000)
    else:
        p = PANEL_ORIGIN + RIGHT * (u / 1000) - up * (v / 1000)
    return p + n * offset


def rotation(u, v):
    d, n, up = basis(section(u, v))
    return Matrix((d, -n, up)).transposed().to_quaternion()


def centre(r):
    x, y, w, h = r
    return x + w / 2.0, y + h / 2.0


def rect_point(r, offset=0.0):
    return point(*centre(r), offset)


def rect_rotation(r):
    return rotation(*centre(r))


def inside(r, zone):
    u, v = centre(r)
    x, y, w, h = zone
    return x <= u <= x + w and y <= v <= y + h


# --- keep, drop
KEEP = (
    "seat_", "cushion_", "metal_98", "metal_99", "control_Action_Eject_", "stick", "throttle",
    "black_549", "metal_550", "metal_551", "hud_glass", "approach", "lamp", "control_Action_Gear_.001",
)
SEAT = ("seat_", "cushion_", "metal_98", "metal_99", "control_Action_Eject_")
RAILS = ("black_549", "metal_550", "metal_551")


def drop(o):
    for c in list(o.children):
        drop(c)
    bpy.data.objects.remove(o, do_unlink=True)


for o in list(root.children):
    if not o.name.startswith(KEEP):
        drop(o)
old = set(root.children)

# The kept parts to life size about the eye: positions and meshes, the controls' sizes.
SIZE_PROPS = ("height", "travel", "grip")
scaled = set()


def scale(o, k):
    o.location = o.location * k
    if o.type == "MESH" and o.data.name not in scaled:
        o.data.transform(Matrix.Scale(k, 4))
        scaled.add(o.data.name)
    for p in SIZE_PROPS:
        if p in o.keys():
            o[p] = o[p] * k
    if "half" in o.keys():
        o["half"] = [h * k for h in o["half"]]
    for c in o.children:
        scale(c, k)


for o in old:
    scale(o, 1.0 / F117_SCALE)
objs = {o.name: o for o in root.children}

# --- materials
mats = bpy.data.materials


def material(name, like):
    m = mats.get(name)
    if m is None:
        m = mats[like].copy()
        m.name = name
    return m


def coloured(name, like, rgb, roughness=None):
    """A material of its own colour (not a shared one: the game takes its look from the model)."""
    m = material(name, like)
    m.diffuse_color = (*rgb, 1.0)
    for nd in m.node_tree.nodes if m.use_nodes else ():
        if nd.type == "BSDF_PRINCIPLED":
            nd.inputs["Base Color"].default_value = m.diffuse_color
            if roughness is not None:
                nd.inputs["Roughness"].default_value = roughness
    return m


for n, like in (("hsi", "adi"), ("screen_vdi", "adi"), ("screen_phd", "adi"), ("screen_dvri", "adi"), ("screen_sensor", "adi")):
    material(n, like)
coloured("red", "yellow", (0.55, 0.04, 0.03))
# The engraved white of the bank indices; a dark tube's glass (the dressing's gauges).
coloured("white", "yellow", (0.80, 0.80, 0.76))
coloured("tube", "yellow", (0.012, 0.022, 0.016), 0.2)


# --- building blocks
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


def finish(name, bm, mat, smooth=False):
    """The mesh of a bmesh (freed), with flat UVs where it has none."""
    me = bpy.data.meshes.new(name)
    uv = bm.loops.layers.uv.verify()
    del uv
    bm.to_mesh(me)
    bm.free()
    me.materials.append(mats[mat])
    if smooth:
        for p in me.polygons:
            p.use_smooth = True
    return me


def box_mesh(name, size, mat, centre=(0.0, 0.0, 0.0), rot=None):
    bm = bmesh.new()
    m = Matrix.Translation(centre) @ (rot.to_matrix().to_4x4() if rot is not None else Matrix.Identity(4)) @ Matrix.Diagonal((*size, 1.0))
    bmesh.ops.create_cube(bm, size=1.0, matrix=m)
    uv = bm.loops.layers.uv.new()
    for f in bm.faces:
        for i, l in enumerate(f.loops):
            l[uv].uv = ((0, 0), (1, 0), (1, 1), (0, 1))[i % 4]
    return finish(name, bm, mat)


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
    bm = bmesh.new()
    rot = {"Z": Matrix.Identity(4), "-Y": Matrix.Rotation(math.radians(90.0), 4, "X")}[axis]
    m = Matrix.Translation(centre) @ rot @ Matrix.Translation((0.0, 0.0, h / 2))
    bmesh.ops.create_cone(bm, cap_ends=True, cap_tris=False, segments=n, radius1=r, radius2=r, depth=h, matrix=m)
    return finish(name, bm, mat)


def box(name, mat, centre, size, rot=Euler((0, 0, 0)), parent=root):
    return mesh_object(name, box_mesh(name, size, mat), centre, rot, parent=parent)


def cbox(mat, x, s, z):
    """A box between two corners of the cabin: (x0, x1), (s0, s1), (z0, z1)."""
    a, b = C(x[0], s[0], z[0]), C(x[1], s[1], z[1])
    return box(mat, mat, (a + b) / 2, (abs(b.x - a.x), abs(b.y - a.y), abs(b.z - a.z)))


def panel_box(name, mat, r, raise_m, margin=0.0, base=0.0):
    """A plate from `base` to `base + raise_m` off the panel over the rectangle r (mm, within one
    section) grown by `margin` mm."""
    x, y, w, h = r
    c = rect_point(r, base + raise_m / 2)
    return box(name, mat, c, ((w + 2 * margin) / 1000, abs(raise_m), (h + 2 * margin) / 1000), rect_rotation(r))


def bezel(r, offset, height=0.008, lip=0.003, case="dark"):
    """A round black bezel standing `height` out of the panel at `offset` round the rectangle r's
    inscribed circle, the instrument's octagonal case behind it."""
    x, y, w, h = r
    me = cylinder_mesh("black", min(w, h) / 2000 + lip, height, "black", n=ROUND)
    mesh_object("black", me, rect_point(r, offset), rect_rotation(r))
    flat = min(w, h) / 2000 + lip + 0.002
    me = cylinder_mesh(case, flat / math.cos(math.pi / 8), offset + 0.003, case, n=8)
    me.transform(Matrix.Rotation(math.pi / 8, 4, "Y"))
    mesh_object(case, me, rect_point(r, 0.0), rect_rotation(r))


def face(key, r, offset, round=False):
    """An atlas face over the rectangle r; round ones are discs in a bezel (the instruments)."""
    x, y, w, h = r
    if round:
        bezel(r, offset)
        me = disc_mesh("atlas", min(w, h) / 2000, "atlas")
        offset += 0.0085
    else:
        me = quad_mesh("atlas", w / 1000, h / 1000, "atlas")
    mesh_object("atlas", me, rect_point(r, offset), rect_rotation(r), props={"atlas_face": key, "atlas_uv": [0.0, 0.0, 1.0, 1.0], "no_shadow": True})


def highlight(c, hw, hh, height):
    """The frame shown round a control while a fingertip touches it."""
    hl = empty("highlight", (0, 0, 0), parent=c, props={"highlight": True})
    t = 0.002
    for (cx, cz, sx, sz) in ((0, hh + t, 2 * hw + 2 * t, t), (0, -hh - t, 2 * hw + 2 * t, t), (hw + t, 0, t, 2 * hh), (-hw - t, 0, t, 2 * hh)):
        mesh_object("highlight", box_mesh("highlight", (sx, t, sz), "highlight"), (cx, -height, cz), parent=hl, props={"no_shadow": True})


def control_name(effect):
    return "control_" + "".join(ch if ch.isalnum() else "_" for ch in effect)


def control(key, r, offset, effect, kind):
    """A push button or key: origin on the surface, its face `height` out; a dark cap body under
    the face and a highlight frame. A `twist` one (the square pushbutton that latches in turned
    45 degrees) stands taller, its cap body grey."""
    x, y, w, h = r
    hw, hh = w / 2000, h / 2000
    height, travel = (0.008, 0.004) if kind == "twist" else (0.006, 0.004)
    body = "frame" if kind == "twist" else "dark"
    c = empty(control_name(effect), rect_point(r, offset), rect_rotation(r), props={"control": effect, "kind": kind, "half": [hw, hh], "height": height, "travel": travel})
    moving = empty("moving", (0, 0, 0), parent=c, props={"moving": True})
    mesh_object("atlas", quad_mesh("atlas", 2 * hw, 2 * hh, "atlas", -height), (0, 0, 0), parent=moving, props={"atlas_face": key, "atlas_uv": [0.0, 0.0, 1.0, 1.0], "no_shadow": True})
    mesh_object(body, box_mesh(body, (2 * hw, height - 0.0004, 2 * hh), body, (0, -(height - 0.0004) / 2, 0)), (0, 0, 0), parent=moving)
    highlight(c, hw, hh, height)


def switch(key, r, offset, effect):
    """A toggle switch: its plate (the atlas face) on the surface, a collar and the lever (the
    `moving` part, leaning to its position about the control's X). A plate wider than high is a
    switch lying across: the control turned so that its first position is to the left."""
    x, y, w, h = r
    rot = rect_rotation(r)
    if w > h:
        rot = rot @ Quaternion((0.0, -1.0, 0.0), math.radians(90.0))
        w, h = h, w
    hw, hh = w / 2000, h / 2000
    c = empty(control_name(effect), rect_point(r, offset), rot, props={"control": effect, "kind": "switch", "half": [hw, hh], "height": 0.014, "travel": 0.0})
    mesh_object("atlas", quad_mesh("atlas", 2 * hw, 2 * hh, "atlas", -0.0004), (0, 0, 0), parent=c, props={"atlas_face": key, "atlas_uv": [0.0, 0.0, 1.0, 1.0], "no_shadow": True})
    mesh_object("metal", cylinder_mesh("metal", 0.0045, 0.003, "metal", n=KNOB), (0, 0, 0), parent=c)
    moving = empty("moving", (0, 0, 0), parent=c, props={"moving": True})
    mesh_object("metal", box_mesh("metal", (0.0036, 0.017, 0.0036), "metal", (0.0, -0.0085, 0.0)), (0, 0, 0), parent=moving)
    mesh_object("metal", cylinder_mesh("metal", 0.0028, 0.004, "metal", centre=(0.0, -0.015, 0.0), n=KNOB), (0, 0, 0), parent=moving)
    highlight(c, hw, hh, 0.001)


def knob_control(key, r, offset, effect):
    """A rotary knob: a dark skirt, the knob (the `moving` part, turned to its position about
    the control's face normal) with its top the atlas face (its index)."""
    x, y, w, h = r
    rad = min(w, h) / 2000
    height = 0.014
    c = empty(control_name(effect), rect_point(r, offset), rect_rotation(r), props={"control": effect, "kind": "knob", "half": [rad, rad], "height": height, "travel": 0.0})
    mesh_object("black", cylinder_mesh("black", rad + 0.002, 0.003, "black", n=ROUND), (0, 0, 0), parent=c)
    moving = empty("moving", (0, 0, 0), parent=c, props={"moving": True})
    mesh_object("dark", cylinder_mesh("dark", rad, height - 0.0004, "dark", n=ROUND), (0, 0, 0), parent=moving)
    mesh_object("atlas", disc_mesh("atlas", rad, "atlas", -height), (0, 0, 0), parent=moving, props={"atlas_face": key, "atlas_uv": [0.0, 0.0, 1.0, 1.0], "no_shadow": True})
    highlight(c, rad, rad, 0.001)


def plate(name, mat, loop, depth):
    """A panel plate `depth` m thick behind the panel plane, from a convex outline (layout mm)."""
    bm = bmesh.new()
    n = basis(section(*loop[0]))[1]
    front = [bm.verts.new(point(u, v)) for u, v in loop]
    back = [bm.verts.new(point(u, v) - n * depth) for u, v in loop]
    bm.faces.new(front)
    bm.faces.new(back[::-1])
    for i in range(len(loop)):
        j = (i + 1) % len(loop)
        bm.faces.new((front[i], back[i], back[j], front[j]))
    bmesh.ops.recalc_face_normals(bm, faces=bm.faces[:])
    return mesh_object(name, finish(name, bm, mat), (0, 0, 0))


def plane_box(name, mat, r, d0, d1):
    """A box over the rectangle r (layout mm) from d0 to d1 m off the panel plane."""
    x, y, w, h = r
    c = point(x + w / 2, y + h / 2, (d0 + d1) / 2)
    return box(name, mat, c, (w / 1000, abs(d1 - d0), h / 1000), rect_rotation(r))


def frame(mat, outer, inner, d0, d1):
    """A rectangular frame (layout mm, outer and inner rectangles) from d0 to d1 m off the
    panel plane."""
    ox, oy, ow, oh = outer
    ix, iy, iw, ih = inner
    for r in ((ox, oy, ow, iy - oy), (ox, iy + ih, ow, oy + oh - iy - ih), (ox, iy, ix - ox, ih), (ix + iw, iy, ox + ow - ix - iw, ih)):
        plane_box(mat, mat, r, d0, d1)


def knob(u, v, r=0.006, h=0.012, mat="metal", offset=0.0):
    """A round knob standing out of the panel at layout (u, v)."""
    mesh_object(mat, cylinder_mesh(mat, r, h, mat, n=KNOB), point(u, v, offset), rotation(u, v))


def toggle(u, v, offset=0.0, up=True):
    """A toggle switch (dressing): a round base and a lever leaning up or down the panel."""
    mesh_object("dark", cylinder_mesh("dark", 0.005, 0.004, "dark", n=KNOB), point(u, v, offset), rotation(u, v))
    lean = Matrix.Rotation(math.radians(-35.0 if up else 35.0), 3, "X")
    me = box_mesh("metal", (0.004, 0.016, 0.004), "metal", (0.0, -0.011, 0.0), lean.to_quaternion())
    mesh_object("metal", me, point(u, v, offset + 0.003), rotation(u, v))


def loft(name, mat, sections):
    """A closed solid through equal-length outlines (lists of points), capped at both ends."""
    bm = bmesh.new()
    rings = [[bm.verts.new(p) for p in sec] for sec in sections]
    n = len(rings[0])
    for a, b in zip(rings, rings[1:]):
        for i in range(n):
            j = (i + 1) % n
            bm.faces.new((a[i], a[j], b[j], b[i]))
    bm.faces.new(rings[0][::-1])
    bm.faces.new(rings[-1])
    bmesh.ops.recalc_face_normals(bm, faces=bm.faces[:])
    return mesh_object(name, finish(name, bm, mat), (0, 0, 0))


def prism(name, mat, sz, x0, x1):
    """A convex outline in the cabin's s-z plane (m) from x0 to x1."""
    return loft(name, mat, [[C(x, s, z) for s, z in sz] for x in (x0, x1)])


def rod(name, a, b, radius, mat, parent=root, n=KNOB):
    """A cylinder from a to b (parent frame)."""
    a, b = Vector(a), Vector(b)
    bm = bmesh.new()
    bmesh.ops.create_cone(bm, cap_ends=True, segments=n, radius1=radius, radius2=radius, depth=(b - a).length)
    me = finish(name, bm, mat)
    rot = Vector((0, 0, 1)).rotation_difference((b - a).normalized()).to_matrix().to_4x4()
    me.transform(Matrix.Translation((a + b) / 2) @ rot)
    return mesh_object(name, me, (0, 0, 0), parent=parent)


def interp(table, x):
    """Linear through a table of (x, y...) rows at x."""
    for a, b in zip(table, table[1:]):
        if x <= b[0]:
            t = min(max((x - a[0]) / (b[0] - a[0]), 0.0), 1.0)
            return tuple(p + (q - p) * t for p, q in zip(a[1:], b[1:]))
    return tuple(table[-1][1:])


def arch_v(u):
    """The panel's top edge (layout y) at layout x."""
    a = abs(u)
    if a <= ARCH[0][0]:
        return TOP
    return interp(ARCH, a)[0]


# --- the tub: floor, walls up to the glass, bulkheads
cbox("dark", (-0.95, 0.95), (FOOT_S - 0.02, REAR_S + 0.02), (FLOOR - 0.02, FLOOR))
for side in (-1, 1):
    wall = []
    rail = []
    for s, x, z in SILL:
        xi = x - WALL_IN
        wall.append([C(side * a, s, b) for a, b in ((xi, FLOOR), (xi, z + 0.004), (xi + 0.025, z + 0.004), (xi + 0.025, FLOOR))])
        rail.append([C(side * a, s, b) for a, b in ((xi - 0.012, z - 0.024), (xi - 0.012, z + 0.006), (xi + 0.025, z + 0.006), (xi + 0.025, z - 0.024))])
    loft("panel_grey", "panel_grey", wall)
    loft("frame", "frame", rail)
# The rear bulkhead under the canopy's arch, behind the seats.
arch = [(x, z) for x, z in REAR_ARCH]
outline = [(-x, z) for x, z in reversed(arch[1:])] + arch + [(arch[-1][0], FLOOR), (-arch[-1][0], FLOOR)]
loft("panel_grey", "panel_grey", [[C(x, s, z) for x, z in outline] for s in (REAR_S, REAR_S + 0.02)])
# The front bulkhead under the windscreen's foot.
foot = [(x, z) for x, z in FOOT]
outline = [(-x, z) for x, z in reversed(foot[1:])] + foot + [(foot[-1][0] + 0.03, FLOOR), (-foot[-1][0] - 0.03, FLOOR)]
loft("dark", "dark", [[C(x, s, z - 0.012) for x, z in outline] for s in (FOOT_S - 0.02, FOOT_S)])

# --- the canopy's frames. From inside, the exterior model is drawn without its glass and without
# the skin round the cockpit (`CockpitView::cut` in src/aircraft.rs), which the panes lie on: the
# frames between the panes go with it. They are put back here, lying on the exterior's skin
# (`aircraft_027_lod0` of assets-src/blend/aircraft.blend) just inside it: a strip of frame
# along every pane's outline (tools/models/a6_canopy.json, as tools/models/aircraft.py
# `a6_panes` lays them on the skin: a point of a pane and the skin's point under it are at the
# same s and the same angle about the fuselage's axis at PANE_AXIS), and between the strips the
# skin itself in cells of FRAME_CELL (s, m; degrees about the axis) that touch no pane, from
# FRAME_S and to FRAME_ANGLE either side of straight up.
FRAME_WIDTH = 0.042
FRAME_INSET = 0.004
FRAME_STEP = 0.05
FRAME_CELL = (0.02, 1.2)
FRAME_S = (1.78, 4.06)
FRAME_ANGLE = 84.0
PANE_AXIS = 0.50
AIRCRAFT_K = 16.384
AIRCRAFT_MIDDLE = 8.19


def canopy_frames():
    from mathutils.bvhtree import BVHTree

    here = os.path.dirname(os.path.abspath(__file__))
    blend = os.path.join(os.path.dirname(os.path.abspath(out)), "aircraft.blend")
    with bpy.data.libraries.load(blend) as (src, dst):
        dst.objects = ["aircraft_027_lod0"]
    ext = dst.objects[0].data
    skin = BVHTree.FromPolygons(
        [(v.co.x / AIRCRAFT_K, AIRCRAFT_MIDDLE - v.co.y / AIRCRAFT_K, v.co.z / AIRCRAFT_K) for v in ext.vertices],
        [tuple(p.vertices) for p in ext.polygons],
    )

    def on_skin(p):
        """The skin's point under a pane's point (cabin frame), and the skin's normal there
        (outwards)."""
        axis = Vector((0.0, p.y, PANE_AXIS))
        d = (p - axis).normalized()
        hit, n, _, _ = skin.ray_cast(axis, d)
        if hit is None:
            return None, None
        return hit, (n if n.dot(d) > 0.0 else -n)

    with open(os.path.join(here, "a6_canopy.json")) as f:
        panes = [[Vector(v) for v in tri] for tri in json.load(f)["glass"]]
    key = lambda p: (round(p.x, 3), round(p.y, 3), round(p.z, 3))  # noqa: E731
    count = {}
    for tri in panes:
        for a, b in zip(tri, tri[1:] + tri[:1]):
            e = tuple(sorted((key(a), key(b))))
            count[e] = count.get(e, 0) + 1
    bm = bmesh.new()
    corners = {}

    welded = {}

    def vert(p):
        k = (round(p.x, 4), round(p.y, 4), round(p.z, 4))
        if k not in welded:
            welded[k] = bm.verts.new(C(*p))
        return welded[k]

    def face(*pts):
        """A face seen from inside the cabin (its normal towards the fuselage's axis)."""
        vs = [vert(p) for p in pts]
        if len(set(vs)) < 3:
            return
        mid = sum(pts, Vector()) / len(pts)
        n = (pts[1] - pts[0]).cross(pts[2] - pts[0])
        # (The eye's frame mirrors the cabin's: s runs aft, y forward.)
        if n.dot(Vector((0.0, mid.y, PANE_AXIS)) - mid) > 0.0:
            vs = vs[::-1]
        try:
            bm.faces.new(vs)
        except ValueError:
            pass

    for tri in panes:
        for k in range(3):
            a, b, c = tri[k], tri[(k + 1) % 3], tri[(k + 2) % 3]
            if count[tuple(sorted((key(a), key(b))))] != 1:
                continue
            # The outline's edge in steps along the skin, its strip away from the pane.
            steps = max(1, math.ceil((b - a).length / FRAME_STEP))
            inside, _ = on_skin((a + b + c) / 3)
            row = []
            for i in range(steps + 1):
                p, n = on_skin(a.lerp(b, i / steps))
                if p is not None:
                    row.append((p, n))
            if inside is None or len(row) < 2:
                continue
            along = (row[-1][0] - row[0][0]).normalized()
            strip = []
            for p, n in row:
                away = along.cross(n).normalized()
                if away.dot(inside - p) > 0.0:
                    away = -away
                p0 = p - n * FRAME_INSET
                strip.append((p0, p0 + away * FRAME_WIDTH))
            for (a0, a1), (b0, b1) in zip(strip, strip[1:]):
                face(a0, b0, b1, a1)
            # The wedges between the strips of the edges meeting at a corner.
            for v, (p0, p1) in ((a, strip[0]), (b, strip[-1])):
                if key(v) in corners:
                    face(p0, corners[key(v)], p1)
                else:
                    corners[key(v)] = p1
    # The skin between the strips: the cells whose corners and middle are in no pane.
    flat = []
    for tri in panes:
        q = [(v.y, math.atan2(v.x, v.z - PANE_AXIS)) for v in tri]
        flat.append((q, min(p[0] for p in q), max(p[0] for p in q), min(p[1] for p in q), max(p[1] for p in q)))

    def in_pane(sv, w):
        for q, s0, s1, w0, w1 in flat:
            if s0 <= sv <= s1 and w0 <= w <= w1:
                d = [(q[(i + 1) % 3][0] - q[i][0]) * (w - q[i][1]) - (q[(i + 1) % 3][1] - q[i][1]) * (sv - q[i][0]) for i in range(3)]
                if all(x >= 0 for x in d) or all(x <= 0 for x in d):
                    return True
        return False

    ds, dw = FRAME_CELL[0], math.radians(FRAME_CELL[1])
    ns = round((FRAME_S[1] - FRAME_S[0]) / ds)
    nw = round(math.radians(FRAME_ANGLE) / dw)
    free = {(i, j): not in_pane(FRAME_S[0] + i * ds / 2, j * dw / 2) for i in range(2 * ns + 1) for j in range(-2 * nw, 2 * nw + 1)}
    grid = {}

    def node(i, j):
        if (i, j) not in grid:
            sv, w = FRAME_S[0] + i * ds, j * dw
            p, n = on_skin(Vector((math.sin(w), sv, PANE_AXIS + math.cos(w))))
            grid[(i, j)] = None if p is None else p - n * (FRAME_INSET + 0.002)
        return grid[(i, j)]

    for i in range(ns):
        for j in range(-nw, nw):
            if not all(free[(2 * i + a, 2 * j + b)] for a in range(3) for b in range(3)):
                continue
            quad = [node(i, j), node(i + 1, j), node(i + 1, j + 1), node(i, j + 1)]
            if all(q is not None for q in quad):
                face(*quad)
    me = finish("frame", bm, "frame")
    bpy.data.objects.remove(dst.objects[0])
    bpy.data.meshes.remove(ext)
    return mesh_object("frame", me, (0, 0, 0))


canopy_frames()

# --- panel plate and glare shield
half = [(u, v) for u, v in ARCH]
outline = [(-u, v) for u, v in reversed(half)] + half
plate("panel_grey", "panel_grey", outline, 0.03)
# The glare shield: from a lip over the panel's top edge forward and down to the windscreen's
# foot, following the arch.
THICK = 0.014
LIP = 0.035
xs = [-0.885, -0.86, -0.80, -0.72, -0.62, -0.50, -0.25, 0.0, 0.25, 0.50, 0.62, 0.72, 0.80, 0.86, 0.885]
shade_front = [point(x * 1000, arch_v(x * 1000)) + UP * THICK + NORMAL * LIP for x in xs]
# The lip's lower edge, a little under the panel's top edge.
lip_low = [point(x * 1000, arch_v(x * 1000)) + NORMAL * LIP - UP * 0.006 for x in xs]
shade_back = []
for x in xs:
    xb = x * FOOT[-1][0] / 0.885
    shade_back.append(C(xb, FOOT_S, interp(FOOT, abs(xb))[0] - 0.012))
down = Vector((0.0, 0.0, -THICK))
loft("black", "black", [[f, b, b + down, l] for f, b, l in zip(shade_front, shade_back, lip_low)])


def shade_z(x, y):
    """The glare shield's top at eye-frame (x, y)."""
    X = x + EYE[0]
    for i in range(len(xs) - 1):
        if xs[i] <= X <= xs[i + 1]:
            t = (X - xs[i]) / (xs[i + 1] - xs[i])
            f = shade_front[i].lerp(shade_front[i + 1], t)
            b = shade_back[i].lerp(shade_back[i + 1], t)
            k = min(max((y - f.y) / (b.y - f.y), 0.0), 1.0)
            return f.z + (b.z - f.z) * k
    return shade_front[0].z


# --- under the panel: the knee wall and the lid back to it, the consoles
low = PANEL_ORIGIN - UP * (BOTTOM / 1000)
low_s, low_z = EYE[1] - low.y, low.z + EYE[2]
cbox("dark", (-0.95, 0.95), (low_s - 0.10, low_s - 0.08), (FLOOR, low_z))
cbox("dark", (-0.95, 0.95), (low_s - 0.10, low_s + 0.004), (low_z - 0.02, low_z))
for x0, x1 in ((-SIDE_CONSOLE[1], -SIDE_CONSOLE[0]), (-CENTRE_CONSOLE, CENTRE_CONSOLE), (SIDE_CONSOLE[0], SIDE_CONSOLE[1])):
    cbox("panel_grey", (x0, x1), CONSOLE_S, (FLOOR, CONSOLE_Z))
    cbox("dark", (x0 + 0.006, x1 - 0.006), (CONSOLE_S[0] + 0.01, CONSOLE_S[1] - 0.01), (CONSOLE_Z, CONSOLE_Z + 0.004))
CONSOLE_FACE = CONSOLE_Z + 0.004


def fitting(c, p):
    """A console fitting at p (on a panel's top): k / K small / large knob, t / T toggle leaning
    forward / back, g red guard, r red button, b a keypad key, w a counter's window."""
    if c in "kK":
        r, h = (0.006, 0.010) if c == "k" else (0.009, 0.013)
        mesh_object("dark", cylinder_mesh("dark", r + 0.003, 0.002, "dark", axis="Z", n=KNOB), p)
        mesh_object("metal", cylinder_mesh("metal", r, h, "metal", axis="Z", n=KNOB), p + Vector((0.0, 0.0, 0.002)))
    elif c in "tT":
        mesh_object("dark", cylinder_mesh("dark", 0.005, 0.004, "dark", axis="Z", n=KNOB), p)
        lean = Euler((math.radians(-30.0 if c == "t" else 30.0), 0.0, 0.0)).to_quaternion()
        mesh_object("metal", box_mesh("metal", (0.004, 0.004, 0.016), "metal", (0.0, 0.0, 0.008), lean), p + Vector((0.0, 0.0, 0.003)))
    elif c == "g":
        mesh_object("red", box_mesh("red", (0.013, 0.022, 0.012), "red", (0.0, 0.0, 0.006)), p)
    elif c == "r":
        mesh_object("red", cylinder_mesh("red", 0.005, 0.006, "red", axis="Z", n=KNOB), p)
    elif c == "b":
        mesh_object("dark", box_mesh("dark", (0.011, 0.009, 0.005), "dark", (0.0, 0.0, 0.0025)), p)
    elif c == "w":
        mesh_object("tube", box_mesh("tube", (0.026, 0.010, 0.002), "tube", (0.0, 0.0, 0.001)), p)


def console_panel(x0, x1, s0, s1, rows, h=0.006):
    """A plate over x0..x1, s0..s1 (cabin, m) on a console, its fittings in rows from the front."""
    cbox("black", (x0, x1), (s0, s1), (CONSOLE_FACE, CONSOLE_FACE + h))
    for i, row in enumerate(rows):
        s = s0 + (s1 - s0) * (i + 0.5) / len(rows)
        for j, c in enumerate(row):
            fitting(c, C(x0 + (x1 - x0) * (j + 0.5) / len(row), s, CONSOLE_FACE + h))


# The pilot's left console: fuel management, the panels round the throttle, lights at the back.
console_panel(-0.92, -0.62, 2.50, 2.62, ["kKkk", "tttt"])
console_panel(-0.92, -0.845, 2.64, 3.06, ["k", "t", "t", "r"])
console_panel(-0.695, -0.62, 2.64, 3.06, ["t", "k", "t", "k"])
console_panel(-0.92, -0.62, 3.08, 3.30, ["tTtg", "kkKk"])
console_panel(-0.92, -0.62, 3.32, 3.66, ["kKk", "tttt", "ww", "kkk"])
# The centre console: the radios, the autopilot, the IFF.
console_panel(-0.11, 0.11, 2.50, 2.66, ["ww", "kKKk", "tttt"])
console_panel(-0.11, 0.11, 2.68, 2.90, ["w", "kkkk", "bbbbb"])
console_panel(-0.11, 0.11, 2.92, 3.14, ["tgtg", "KkkK"])
console_panel(-0.11, 0.11, 3.16, 3.38, ["ww", "kttk"])
console_panel(-0.11, 0.11, 3.40, 3.66, ["kkk", "tTt", "rr"])
# The bombardier/navigator's right console: the doppler and platform panels, lights, oxygen.
console_panel(0.62, 0.92, 2.50, 2.66, ["kKk", "tttt"])
console_panel(0.62, 0.92, 2.68, 2.94, ["ww", "kkkk", "tTtT"])
console_panel(0.62, 0.92, 2.96, 3.22, ["KkK", "bbbbbb", "bbbbbb"])
console_panel(0.62, 0.92, 3.24, 3.44, ["tttg", "kkk"])
console_panel(0.62, 0.92, 3.46, 3.66, ["kKk", "tt"])

# --- the pedestal between the bombardier's knees: the keyboard on its sloping face, the slew
# stick on the shelf behind it
ped_low = PED_ORIGIN - PED_UP * 0.150
ped_top = (EYE[1] - PED_ORIGIN.y, PED_ORIGIN.z + EYE[2])
ped_end = (EYE[1] - ped_low.y, ped_low.z + EYE[2])
SHELF_S = ped_end[0] + 0.17
prism("panel_grey", "panel_grey", [(low_s - 0.09, FLOOR), (low_s - 0.09, ped_top[1] + 0.012), ped_top, ped_end, (SHELF_S, ped_end[1] - 0.01), (SHELF_S, FLOOR)], *PED_X)
# The slew control: a base plate, a boot and the stick with its grip, leaning a little aft.
slew = C(sum(PED_X) / 2, ped_end[0] + 0.09, ped_end[1] - 0.004)
box("black", "black", slew + Vector((0, 0, 0.003)), (0.11, 0.12, 0.006))
mesh_object("rubber", cylinder_mesh("rubber", 0.026, 0.02, "rubber", axis="Z", n=12), slew + Vector((0, 0, 0.006)))
tip = slew + Vector((0.0, -0.02, 0.12))
# Its pivot in the boot, and the parts that move about it (in the right seat's cockpit they are
# its `stick`: it slews the pod).
SLEW_PIVOT = slew + Vector((0, 0, 0.015))
slew_parts = [
    rod("metal", slew + Vector((0, 0, 0.02)), tip, 0.007, "metal"),
    rod("rubber", tip - Vector((0.0, -0.004, 0.025)), tip + Vector((0.0, -0.008, 0.05)), 0.016, "rubber", n=10),
    mesh_object("red", cylinder_mesh("red", 0.005, 0.005, "red", axis="Z", n=KNOB), tip + Vector((0.0, -0.008, 0.05))),
]
SLEW_GRIP = (tip + Vector((0.0, -0.006, 0.0125)) - SLEW_PIVOT).z

# --- plates under the panel's groups, housings of the displays
PLATE = 0.004
scr = layout["screens"]
# The displays' housings: (rectangle, its front's height off the panel).
VDI_HOUSING = ((-462.0, -194.0, 204.0, 174.0), 0.016)
PHD_HOUSING = ((-448.0, 4.0, 176.0, 194.0), 0.008)
DVRI_HOUSING = ((245.0, -164.0, 240.0, 274.0), 0.010)
SENSOR_HOUSING = ((506.0, -120.0, 152.0, 124.0), 0.018)
HOUSINGS = (VDI_HOUSING, PHD_HOUSING, DVRI_HOUSING, SENSOR_HOUSING)
shared_plates = [
    ("black", (4.0, -196.0, 148.0, 238.0)),  # armament panel
    ("black", (4.0, 46.0, 148.0, 192.0)),  # flight / navigation mode panel
    ("black", (156.0, -196.0, 80.0, 152.0)),  # countermeasures
    ("dark", (156.0, -40.0, 80.0, 82.0)),  # under them
    ("dark", (156.0, 46.0, 82.0, 192.0)),  # fuzing panel
]
pilot_plates = [
    ("black", (-668.0, -34.0, 182.0, 172.0)),  # engine tapes, the keys under them
    ("black", (-254.0, 42.0, 204.0, 196.0)),  # the pilot's control box
    ("black", (-46.0, 42.0, 44.0, 76.0)),  # the sight's modes
    ("dark", (-68.0, -44.0, 66.0, 82.0)),  # caution lights
    ("black", (-730.0, 94.0, 48.0, 40.0)),  # gear lights
    ("dark", (-668.0, 146.0, 182.0, 90.0)),  # hydraulic and trim gauges
    ("dark", (-46.0, 124.0, 44.0, 114.0)),  # master test
]
bn_plates = [
    ("black", (245.0, 112.0, 240.0, 126.0)),  # search radar controls
    ("black", (500.0, 8.0, 160.0, 26.0)),  # master caution and keys
    ("dark", (500.0, 40.0, 160.0, 198.0)),  # navigation panel
    ("black", (286.0, 300.0, 158.0, 148.0)),  # computer keyboard
]
for mat, r in shared_plates + pilot_plates + bn_plates:
    panel_box(mat, mat, r, PLATE)


def offset_of(r):
    """How far off the panel a face over the rectangle r lies: on a display's housing, or on its
    plate."""
    for zone, front in HOUSINGS:
        if inside(r, zone):
            return front
    return PLATE


# --- the vertical display indicator
def superellipse(theta, a, b, n):
    """The radius at angle theta of the superellipse with half axes a, b and exponent n."""
    c, s_ = abs(math.cos(theta)), abs(math.sin(theta))
    return ((c / a) ** n + (s_ / b) ** n) ** (-1.0 / n)


# The tube face's outline: a superellipse (rounded corners, bowed sides) in its rectangle; how
# far its middle stands out (the pillow).
TUBE_N = 4.2
TUBE_BULGE = 0.007
SEG = 56


def tube_mesh(name, a, b, mat, bulge=TUBE_BULGE, rings=7):
    """The tube's glass: a pillowed face inside the superellipse (half axes a, b, m) in the
    local XZ plane, its middle `bulge` towards the crew (-Y), UVs 0..1 across its rectangle."""
    bm = bmesh.new()
    uv = bm.loops.layers.uv.new()
    coords = {}

    def vert(k, i):
        f = k / rings
        th = 2 * math.pi * i / SEG
        r = f * superellipse(th, a, b, TUBE_N)
        v = bm.verts.new((r * math.cos(th), -bulge * (1.0 - f * f), r * math.sin(th)))
        coords[v] = (0.5 + r * math.cos(th) / (2 * a), 0.5 + r * math.sin(th) / (2 * b))
        return v

    mid = bm.verts.new((0.0, -bulge, 0.0))
    coords[mid] = (0.5, 0.5)
    loops = [[vert(k, i) for i in range(SEG)] for k in range(1, rings + 1)]
    for i in range(SEG):
        j = (i + 1) % SEG
        bm.faces.new((mid, loops[0][i], loops[0][j]))
        for a_, b_ in zip(loops, loops[1:]):
            bm.faces.new((a_[i], b_[i], b_[j], a_[j]))
    for f in bm.faces:
        for l in f.loops:
            l[uv].uv = coords[l.vert]
    return finish(name, bm, mat, smooth=True)


def bezel_ring(name, mat, inner, outer, front, inner_back, outer_back):
    """A bezel between two outlines (functions of the angle giving (x, z), m) in the local XZ
    plane: its front face at local y -front, its inner wall back to -inner_back and its outer
    wall back to -outer_back."""
    bm = bmesh.new()
    rows = []
    for i in range(SEG):
        th = 2 * math.pi * i / SEG
        (ix, iz), (ox, oz) = inner(th), outer(th)
        rows.append([bm.verts.new(p) for p in ((ix, -inner_back, iz), (ix, -front, iz), (ox, -front, oz), (ox, -outer_back, oz))])
    for i in range(SEG):
        a, b = rows[i], rows[(i + 1) % SEG]
        for k in range(3):
            bm.faces.new((a[k], a[k + 1], b[k + 1], b[k]))
    return finish(name, bm, mat)


def at_angle(a, b, n, cx=0.0, cz=0.0):
    return lambda th: (cx + superellipse(th, a, b, n) * math.cos(th), cz + superellipse(th, a, b, n) * math.sin(th))


def vdi(rect, live):
    """The vertical display: the tube's pillowed face low in a deep bezel, the bank indices
    engraved along the bezel's top, screws at its corners; the lights and the slip ball's window
    in its lower edge are faces of the layout."""
    x, y, w, h = rect
    a, b = w / 2000, h / 2000
    (hx, hy, hw, hh), front = VDI_HOUSING
    rot = rect_rotation(rect)
    origin = rect_point(rect, 0.0)
    # The housing's outline about the tube's centre.
    cu, cv = centre(rect)
    ox, oz = (hx + hw / 2 - cu) / 1000, -(hy + hh / 2 - cv) / 1000
    mesh_object("screen_vdi" if live else "tube", tube_mesh("screen_vdi" if live else "tube", a, b, "screen_vdi" if live else "tube"), rect_point(rect, 0.005), rot, props={"no_shadow": True})
    inner = at_angle(a + 0.0015, b + 0.0015, TUBE_N)
    outer = at_angle(hw / 2000, hh / 2000, 12.0, ox, oz)
    mesh_object("black", bezel_ring("black", "black", inner, outer, front, 0.002, -0.001), origin, rot)
    # A grey rim round the bezel's foot.
    rim = at_angle(hw / 2000 + 0.004, hh / 2000 + 0.004, 12.0, ox, oz)
    mesh_object("dark", bezel_ring("dark", "dark", outer, rim, 0.006, 0.0, -0.001), origin, rot)
    # Bank indices: at 0, 10, 20, 30, 45 and 60 degrees either side, radial about the middle.
    for deg in (-60, -45, -30, -20, -10, 0, 10, 20, 30, 45, 60):
        th = math.radians(90.0 - deg)
        r0 = superellipse(th, a, b, TUBE_N) + 0.004
        long = 0.009 if deg % 30 == 0 else 0.006
        r1 = r0 + long
        mid = Vector(((r0 + r1) / 2 * math.cos(th), -front - 0.0003, (r0 + r1) / 2 * math.sin(th)))
        tick = box_mesh("white", (0.0022 if deg == 0 else 0.0014, 0.0006, long), "white", rot=Euler((0.0, math.radians(deg), 0.0)).to_quaternion())
        mesh_object("white", tick, origin + rot @ mid, rot, props={"no_shadow": True})
    for sx in (-1, 1):
        for sz in (-1, 1):
            p = Vector((ox + sx * (hw / 2000 - 0.009), -front, oz + sz * (hh / 2000 - 0.009)))
            mesh_object("metal", cylinder_mesh("metal", 0.003, 0.0015, "metal", n=KNOB), origin + rot @ p, rot)


# --- the round scopes in their heading rings
# The tube's radius of the ring's (src/intruder/scope.rs `TUBE`), the lip between them.
TUBE_FRAC = 0.76
LIP_FRAC = (0.745, 0.79)


def scope_mesh(name, rad, mat, bulge, ring_y, rings=6, seg=48):
    """A round scope with its heading ring, one face (UVs 0..1 over the ring's square): the
    tube's glass inside TUBE_FRAC of the radius, bulging `bulge` towards the crew from its rim;
    the flat ring round it at local y -ring_y."""
    bm = bmesh.new()
    uv = bm.loops.layers.uv.new()
    coords = {}
    rt = rad * TUBE_FRAC

    def vert(r, y, i):
        th = 2 * math.pi * i / seg
        v = bm.verts.new((r * math.cos(th), y, r * math.sin(th)))
        coords[v] = (0.5 + 0.5 * r / rad * math.cos(th), 0.5 + 0.5 * r / rad * math.sin(th))
        return v

    mid = bm.verts.new((0.0, -bulge, 0.0))
    coords[mid] = (0.5, 0.5)
    loops = [[vert(rt * k / rings, -bulge * (1.0 - (k / rings) ** 2), i) for i in range(seg)] for k in range(1, rings + 1)]
    loops.append([vert(rt * 1.02, -ring_y, i) for i in range(seg)])
    loops.append([vert(rad, -ring_y, i) for i in range(seg)])
    for i in range(seg):
        j = (i + 1) % seg
        bm.faces.new((mid, loops[0][i], loops[0][j]))
        for a_, b_ in zip(loops, loops[1:]):
            bm.faces.new((a_[i], b_[i], b_[j], a_[j]))
    for f in bm.faces:
        for l in f.loops:
            l[uv].uv = coords[l.vert]
    return finish(name, bm, mat, smooth=True)


def annulus(name, mat, r0, r1, y0, y1, seg=48):
    """A ring between radii r0 and r1 from local y -y0 (its foot) to -y1 (its front)."""
    circle = lambda r: (lambda th: (r * math.cos(th), r * math.sin(th)))  # noqa: E731
    global SEG
    keep, SEG = SEG, seg
    me = bezel_ring(name, mat, circle(r0), circle(r1), y1, y0, y0)
    SEG = keep
    return me


def scope(mat, rect, housing, live, bulge):
    """A round scope: the housing's plate, the scope and its ring, a black lip between the
    glass and the ring and a rim round the ring."""
    zone, front = housing
    x, y, w, h = rect
    rad = min(w, h) / 2000
    rot = rect_rotation(rect)
    origin = rect_point(rect, 0.0)
    panel_box("black", "black", zone, front - 0.0008)
    name = mat if live else "tube"
    mesh_object(name, scope_mesh(name, rad, name, bulge, 0.0025), rect_point(rect, front), rot, props={"no_shadow": True})
    mesh_object("black", annulus("black", "black", rad * LIP_FRAC[0], rad * LIP_FRAC[1], front, front + 0.008), origin, rot)
    mesh_object("dark", annulus("dark", "dark", rad, rad + 0.006, front, front + 0.006), origin, rot)
    zx, zy, zw, zh = zone
    for sx in (0, 1):
        for sy in (0, 1):
            u, v = zx + 8.0 + sx * (zw - 16.0), zy + 8.0 + sy * (zh - 16.0)
            mesh_object("metal", cylinder_mesh("metal", 0.003, 0.0015, "metal", n=KNOB), point(u, v, front), rot)


def sensor(rect, live):
    """The sensor scope: the pod's picture behind its overlay, deep in a plain bezel with
    rounded inner corners under a short hood."""
    zone, front = SENSOR_HOUSING
    x, y, w, h = rect
    rot = rect_rotation(rect)
    origin = rect_point(rect, 0.0)
    a, b = w / 2000, h / 2000
    cu, cv = centre(rect)
    zx, zy, zw, zh = zone
    ox, oz = (zx + zw / 2 - cu) / 1000, -(zy + zh / 2 - cv) / 1000
    if live:
        mesh_object("screen_sensor", quad_mesh("screen_sensor", w / 1000, h / 1000, "screen_sensor"), rect_point(rect, 0.004), rot, props={"no_shadow": True})
        mesh_object("trackcam", quad_mesh("trackcam", w / 1000, h / 1000, "trackcam"), rect_point(rect, 0.001), rot, props={"no_shadow": True})
    else:
        mesh_object("tube", quad_mesh("tube", w / 1000, h / 1000, "tube"), rect_point(rect, 0.004), rot, props={"no_shadow": True})
    inner = at_angle(a - 0.001, b - 0.001, 7.0)
    outer = at_angle(zw / 2000, zh / 2000, 14.0, ox, oz)
    mesh_object("black", bezel_ring("black", "black", inner, outer, front, 0.003, -0.001), origin, rot)
    # The hood over its top.
    plane_box("black", "black", (zx, zy - 3.0, zw, 4.0), 0.0, front + 0.03)
    for u in (zx + 26.0, zx + 58.0, zx + 94.0, zx + 126.0):
        knob(u, zy + zh - 9.0, 0.005, 0.008, "dark", offset=front)


# --- displays
vdi(scr["vdi"], True)
scope("screen_phd", scr["phd"], PHD_HOUSING, True, 0.006)
scope("screen_dvri", scr["dvri"], DVRI_HOUSING, True, 0.008)
sensor(scr["sensor"], True)


def round_screen(mat, r, offset):
    """A round display (the standby attitude indicator, the HSI): a disc in a bezel."""
    bezel(r, offset, 0.010, 0.004)
    mesh_object(mat, disc_mesh(mat, min(r[2], r[3]) / 2000, mat), rect_point(r, offset + 0.0105), rect_rotation(r), props={"no_shadow": True})


round_screen("adi", scr["adi"], 0.0)
round_screen("hsi", scr["hsi"], 0.0)

# --- faces and controls
for f in layout["faces"]:
    r = f["rect"]
    off = offset_of(r)
    key = f["key"]
    if f["control"] is None:
        if key.startswith(("Item(Light(", "Item(Readout(", "Item(Lamp(", "Item(Led(", "Item(Gauge", "LedBars", "Item(GearLights")):
            # A light or window: a black frame round it.
            x, y, w, h = r
            frame("black", (x - 1.5, y - 1.5, w + 3.0, h + 3.0), (x, y, w, h), off - 0.001, off + 0.002)
        face(key, r, off + 0.0005, f["round"])
        if key.startswith("Item(Dial(") and not f["round"]:
            # The tapes' and the slip ball's black window frames.
            x, y, w, h = r
            frame("black", (x - 3.0, y - 3.0, w + 6.0, h + 6.0), (x, y, w, h), off - 0.001, off + 0.004)
    elif f["kind"] == "switch":
        switch(key, r, off, f["control"])
    elif f["kind"] == "knob":
        knob_control(key, r, off, f["control"])
    else:
        control(key, r, off, f["control"], f["kind"])

# --- dressing: what the layouts do not carry
# The pilot's side: the emergency stores jettison button, the small hydraulic and trim gauges
# under the keys, the master test panel's switches.
for u in (-644.0, -598.0, -552.0, -506.0):
    r = (u - 17.0, 160.0, 34.0, 34.0)
    bezel(r, PLATE, 0.006, 0.002)
    mesh_object("tube", disc_mesh("tube", 0.017, "tube"), rect_point(r, PLATE + 0.0065), rect_rotation(r))
for u in (-644.0, -552.0):
    toggle(u, 216.0, PLATE)
knob(-598.0, 216.0, 0.006, 0.010, "dark", offset=PLATE)
mesh_object("yellow", cylinder_mesh("yellow", 0.016, 0.004, "yellow", n=ROUND), point(-610.0, -86.0), rotation(-610.0, -86.0))
mesh_object("red", cylinder_mesh("red", 0.009, 0.010, "red", n=12), point(-610.0, -86.0), rotation(-610.0, -86.0))
for v in (140.0, 166.0, 192.0, 218.0):
    toggle(-24.0, v, PLATE, up=(v != 166.0))
# The control box's brightness and contrast knobs.
for u, v in ((-128.0, 78.0), (-128.0, 112.0), (-190.0, 226.0), (-78.0, 228.0)):
    knob(u, v, 0.006, 0.010, "dark", offset=PLATE)
# Between the seats: the fuzing panel's buttons and switches, the knobs under the
# countermeasures.
for i in range(8):
    panel_box("black", "black", (172.0, 60.0 + 20.0 * i, 12.0, 12.0), 0.006, 0.0, PLATE)
for i, v in enumerate((70.0, 110.0, 150.0, 190.0)):
    toggle(214.0, v, PLATE, up=(i % 2 == 0))
for u, v in ((176.0, -22.0), (214.0, -22.0), (176.0, 16.0)):
    knob(u, v, 0.008, 0.012, "dark", offset=PLATE)
toggle(214.0, 16.0, PLATE)
# The bombardier's side: the radar panel's knobs and its meter, the navigation panel's counters
# and switches.
for i, u in enumerate((270.0, 310.0, 350.0, 390.0, 430.0, 466.0)):
    knob(u, 214.0, 0.008 if i % 2 == 0 else 0.006, 0.012, "dark", offset=PLATE)
panel_box("tube", "tube", (296.0, 174.0, 30.0, 20.0), 0.002, 0.0, PLATE)
for v in (62.0, 86.0, 110.0):
    for u in (520.0, 560.0):
        panel_box("tube", "tube", (u, v, 34.0, 12.0), 0.002, 0.0, PLATE)
    toggle(626.0, v + 6.0, PLATE, up=(v != 86.0))
for u, v in ((530.0, 150.0), (580.0, 150.0), (630.0, 150.0), (555.0, 200.0), (610.0, 200.0)):
    knob(u, v, 0.009, 0.013, "dark", offset=PLATE)

# --- the kept parts
# Seats: the cushion's top at SEAT_TOP under each eye, on a pedestal from the floor; the other
# seat a copy without the handle's control.
seat = [o for o in old if o.name.startswith(SEAT)]
bpy.context.view_layer.update()
cushion = min((o for o in seat if o.name.startswith("cushion_")), key=lambda o: o.location.z)
cushion_top = max((cushion.matrix_world @ v.co).z for v in cushion.data.vertices)
for o in seat:
    o.location.z += SEAT_TOP - cushion_top
bpy.context.view_layer.update()
bottom = min((o.matrix_world @ v.co).z for o in seat if o.type == "MESH" for v in o.data.vertices)
to_other = Vector((OTHER_EYE[0] - EYE[0], EYE[1] - OTHER_EYE[1], OTHER_EYE[2] - EYE[2]))
floor_z = FLOOR - EYE[2]
for shift in (Vector((0.0, 0.0, 0.0)), to_other):
    box("dark", "dark", Vector((shift.x, shift.y - 0.06, (bottom + shift.z + floor_z) / 2)), (0.34, 0.30, bottom + shift.z - floor_z))


def copy_of(o, loc):
    """A copy of a mesh object (sharing its mesh) under the root at loc."""
    c = o.copy()
    bpy.context.scene.collection.objects.link(c)
    c.parent = root
    c.location = loc
    return c


for o in seat:
    if o.type == "MESH":
        copy_of(o, o.location + to_other)
handle = objs["control_Action_Eject_"]
for m in handle.children:
    for part in m.children:
        copy_of(part, handle.location + m.location + part.location + to_other)

# Centre stick: pivot on the floor on the pilot's centre line, the grip raised on a shaft, a
# boot at its foot.
stick = objs["stick"]
pivot = C(PILOT_EYE[0], STICK_S, FLOOR + 0.01)
stick.location = pivot
for c in stick.children:
    c.location.z += STICK_RAISE
stick["grip"] = float(stick["grip"]) + STICK_RAISE
rod("metal", (0, 0, 0), (0, 0, STICK_RAISE + 0.02), 0.011, "metal", stick)
rod("rubber", pivot, pivot + Vector((0, 0, 0.06)), 0.04, "rubber")
box("dark", "dark", pivot + Vector((0.0, 0.0, -0.004)), (0.12, 0.12, 0.008))
# In the right seat the pilot's stick stands still (a copy) and the slew stick between the
# bombardier's knees is the `stick`: taken in VR, it slews the pod.
if RIGHT_SEAT:
    for c in list(stick.children):
        copy_of(c, pivot + c.location)
        drop(c)
    stick.location = SLEW_PIVOT
    for o in slew_parts:
        o.parent = stick
        o.location = o.location - SLEW_PIVOT
    stick["grip"] = SLEW_GRIP
# Rudder pedals under the panel.
for dx in (-0.11, 0.11):
    cbox("black", (PILOT_EYE[0] + dx - 0.045, PILOT_EYE[0] + dx + 0.045), (low_s - 0.06, low_s - 0.045), (FLOOR + 0.08, FLOOR + 0.20))
    cbox("frame", (PILOT_EYE[0] + dx - 0.008, PILOT_EYE[0] + dx + 0.008), (low_s - 0.075, low_s - 0.06), (FLOOR, FLOOR + 0.14))
# Throttle: on the pilot's left console, the kept handle's travel lengthened to
# THROTTLE_TRAVEL, its rails along it.
throttle = objs["throttle"]
full = objs["throttle_full"]
move = C(*THROTTLE) - throttle.location
throttle.location = C(*THROTTLE)
full.location = throttle.location + Vector((0.0, THROTTLE_TRAVEL, 0.0))
y0, y1 = throttle.location.y - 0.03, full.location.y + 0.03
for n in RAILS:
    o = objs[n]
    o.data = o.data.copy()
    vs = [v.co.y for v in o.data.vertices]
    k = (y1 - y0) / (max(vs) - min(vs))
    o.data.transform(Matrix.Diagonal((1.0, k, 1.0, 1.0)))
    o.location = o.location + move
    o.location.y = (y0 + y1) / 2

# The optical sight on the glare shield on the pilot's centre line: its head (a box with the
# lens on top) and the reflector plate leaning towards the pilot in its thin frame between two
# arms. The pilot's cockpit has the plate as the HUD glass; from the right seat only its frame
# shows.
px = PILOT_EYE[0]
low_p, high_p = C(px, SIGHT_LOW[0], SIGHT_LOW[1]), C(px, SIGHT_HIGH[0], SIGHT_HIGH[1])
base_z = shade_z(low_p.x, low_p.y + 0.02)
head_y = (low_p.y - 0.035, low_p.y + 0.075)
box("dark", "dark", (low_p.x, sum(head_y) / 2, (base_z - 0.01 + low_p.z - 0.022) / 2), (2 * SIGHT_HALF + 0.01, head_y[1] - head_y[0], low_p.z - 0.022 - (base_z - 0.01)))
mesh_object("tube", cylinder_mesh("tube", 0.034, 0.004, "tube", axis="Z", n=ROUND), Vector((low_p.x, low_p.y + 0.02, low_p.z - 0.022)))
mesh_object("black", cylinder_mesh("black", 0.040, 0.003, "black", axis="Z", n=ROUND), Vector((low_p.x, low_p.y + 0.02, low_p.z - 0.0225)))
for sx in (-1, 1):
    arm = [Vector((low_p.x + sx * (SIGHT_HALF + 0.003), p.y, p.z)) for p in (low_p, high_p)]
    rod("frame", arm[0] + Vector((0, 0.004, -0.024)), arm[1], 0.003, "frame")
for p in (low_p, high_p):
    rod("frame", p - Vector((SIGHT_HALF + 0.003, 0, 0)), p + Vector((SIGHT_HALF + 0.003, 0, 0)), 0.0025, "frame")
glasses = [o for o in old if o.name.startswith("hud_glass")]
indexers = [o for o in old if o.name.startswith("approach")]
for o in glasses:
    if RIGHT_SEAT:
        drop(o)
        continue
    lo = Vector((min(v.co.x for v in o.data.vertices), 0.0, min(v.co.z for v in o.data.vertices)))
    hi = Vector((max(v.co.x for v in o.data.vertices), 0.0, max(v.co.z for v in o.data.vertices)))
    o.location = (0.0, 0.0, 0.0)
    for v in o.data.vertices:
        tx, tz = (v.co.x - lo.x) / (hi.x - lo.x), (v.co.z - lo.z) / (hi.z - lo.z)
        p = low_p.lerp(high_p, tz)
        v.co = Vector((low_p.x - SIGHT_HALF + 2 * SIGHT_HALF * tx, p.y, p.z))
# The approach indexer left of the sight, in a housing on a stalk (the pilot's only).
for o in indexers:
    if RIGHT_SEAT:
        drop(o)
        continue
    at = Vector((low_p.x - SIGHT_HALF - 0.05, low_p.y + 0.02, 0.0))
    foot_z = shade_z(at.x, at.y)
    at.z = foot_z + 0.065
    o.location = at
    box("dark", "dark", at + Vector((0, 0.007, 0)), (0.028, 0.010, 0.068))
    box("dark", "dark", at + Vector((0, 0.007, -0.047)), (0.010, 0.010, 0.03))
# The gear handle on the pilot's panel, left of the engine tapes.
g = objs["control_Action_Gear_.001"]
g.location = point(-705.0, 40.0, 0.0)
g.rotation_mode = "QUATERNION"
g.rotation_quaternion = rotation(-705.0, 40.0)
panel_box("black", "black", (-722.0, -10.0, 34.0, 96.0), 0.002)
# Flood lamps: one either side behind the head, one under the glare shield over the panel.
objs["lamp"].location = Vector((-0.25, -0.05, -0.12))
objs["lamp.001"].location = Vector((0.25, -0.05, -0.12))
objs["lamp.002"].location = point(EYE[0] * 1000, -120.0, 0.12)

bpy.data.orphans_purge(do_recursive=True)
bpy.context.view_layer.update()
print(f"seat {'right' if RIGHT_SEAT else 'left'}: stick grip {stick['grip']:.3f} m at {tuple(round(c, 3) for c in stick.location)}; "
      f"throttle {tuple(round(c, 3) for c in throttle.location)}; panel bottom s {low_s:.3f} z {low_z:.3f}; "
      f"pedestal {ped_top} -> {ped_end}; sight {tuple(round(c, 3) for c in low_p)} -> {tuple(round(c, 3) for c in high_p)}")
bpy.ops.wm.save_as_mainfile(filepath=out)
print("built", out, "objects", len(bpy.data.objects))
