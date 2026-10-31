# Builds the A-10C cockpit master (assets-src/blend/cockpit_a10.blend) from the panel layout,
# after the A-10C's front panel, with parts of the F-117A's modern cockpit:
#   blender -b assets-src/blend/cockpit_modern.blend --python-exit-code 1 \
#       --python tools/models/build_cockpit_a10.py -- \
#       assets-src/blend/cockpit_a10_layout.json assets-src/blend/cockpit_a10.blend
# then tools/models/export.sh cockpit_a10. The layout JSON is written by
# `UPDATE_LAYOUT=1 cargo test layout_files` from `PanelLayout::a10` (src/cockpit3d/layout.rs).
#
# Life size (the F-117A's cockpit is built larger than life: its kept parts are scaled down by
# F117_SCALE about the eye, as tools/models/build_cockpit_tornado.py does, which this follows).
# Kept, with their materials, labels and control properties: the seat with the ejection handle,
# the stick (made a centre stick on a shaft from the floor between the knees), the throttle with
# its rails on the left console, the gear handle and the five target buttons, the HUD glass, the
# approach indexer and the flood lamps. Built here, low-poly: the flat main panel (its centre
# column with the large attitude indicator, the HSI under it, the RWR scope and the round flight
# instruments either side; the wings with the two MFDs in raised bezels; the wider lower panel
# with the gear, armament, engine, caution and fuel groups), the round displays and instruments
# in their bezels (after build_cockpit_f16.py), the glare shield over it, the HUD on its tall
# body standing on the glare shield with the up-front controller and the data entry display on
# the body's face under the combiner, the consoles with their switch panels, the knee wall with
# the footwell and the centre pedestal, the walls up to the canopy sills.
#
# It fits the exterior's canopy (tools/models/aircraft.py): the sills slope from 0.27 m under
# the eye behind the seat to 0.31 m under it by the knees, about 0.45 m either side. The front
# (the panel, the glare shield and the up-front controller) stands 0.10 m higher than the
# exterior's sills ahead (the glare shield's top 0.255 m under the eye), the walls rising to it.
#
# Blender frame: eye at the origin, x right, y forward, z up, metres. A control's face points
# along its local -Y (towards the pilot), local Z is up the panel.
import json
import math
import sys

import bmesh
import bpy
from mathutils import Euler, Matrix, Quaternion, Vector

layout_path, out = sys.argv[sys.argv.index("--") + 1:][:2]
layout = json.load(open(layout_path))
root = bpy.data.objects["cockpit"]

F117_SCALE = 1.4706
# The main panel: the layout's origin (the attitude indicator's centre), the top leaning away.
PANEL_TILT = math.radians(20.0)
PANEL_CENTRE = Vector((0.0, 0.68, -0.42))
FLOOR = -1.0
# The walls' inner faces; their tops (the canopy sills) along SILL_LINE ((y, z), from the rear
# bulkhead to the glare shield's back), under the exterior's sills.
WALL_X = 0.43
SILL_REAR = -0.62
COAMING_BACK = 1.0
SILL_LINE = [(SILL_REAR, -0.285), (0.25, -0.31), (0.85, -0.26), (COAMING_BACK, -0.275)]
# Consoles (the throttle stands on the left one): x, y and their top (the kept throttle rails'
# own).
CONSOLE_X = (0.22, 0.43)
CONSOLE_Y = (-0.30, 0.42)
CONSOLE_TOP = -0.646
# The panel's outline (layout mm): the centre column (|x| <= HINGE) standing up to TOP_CENTRE,
# a slanted step STEP wide down to the wings' top TOP_WING, their outer corners bevelled BEVEL
# (45 degrees) out to WING; under SHOULDER the lower panel out to LOWER, down to BOTTOM.
HINGE = 165.0
STEP = 40.0
WING = 395.0
TOP_CENTRE = -160.0
TOP_WING = -135.0
BEVEL = 30.0
SHOULDER = 60.0
LOWER = 420.0
BOTTOM = 240.0
# The up-front controller (layout y < UFC_LINE) on the HUD body's face just over the glare
# shield, nearly upright; layout y UFC_V0 at UFC_CENTRE.
UFC_LINE = -170.0
UFC_V0 = -226.0
UFC_TILT = math.radians(5.0)
UFC_CENTRE = Vector((0.0, 0.735, -0.19))
# The glare shield reaches forward to COAMING_BACK, sloping down COAMING_DROP from the panel's top.
COAMING_DROP = 0.03
# HUD combiner glass: half width, bottom and top (m), at HUD_Y in front of the eye (the A-10C's
# large one; AircraftSpec::hud_scale fits the symbology into it). Its body under it from the
# glare shield up to HUD_BODY_TOP, back to HUD_BODY_BACK.
HUD_Y = 0.70
HUD_GLASS = (0.17, -0.13, 0.11)
HUD_BAR = 0.012
HUD_BODY_TOP = -0.145
HUD_BODY_BACK = 0.80
# Seat: its cushion's top, the eye over the hips.
SEAT_TOP = -0.80
# Centre stick: pivot on the floor between the knees, the kept grip raised on a shaft.
STICK_PIVOT = Vector((0.0, 0.32, FLOOR + 0.01))
STICK_RAISE = 0.28
# The throttle's travel along the console (the kept F-117A's, life size, is too short a slide).
THROTTLE_TRAVEL = 0.17
# Segments of the round parts (bezels, buttons, knobs): low-poly.
ROUND = 16
KNOB = 8

RIGHT = Vector((1.0, 0.0, 0.0))
UP = Vector((0.0, math.sin(PANEL_TILT), math.cos(PANEL_TILT)))
NORMAL = Vector((0.0, -math.cos(PANEL_TILT), math.sin(PANEL_TILT)))
UFC_UP = Vector((0.0, math.sin(UFC_TILT), math.cos(UFC_TILT)))
UFC_NORMAL = Vector((0.0, -math.cos(UFC_TILT), math.sin(UFC_TILT)))


def section(u, v):
    """1 the up-front controller, 0 the main panel."""
    return 1 if v < UFC_LINE else 0


def basis(sec):
    """(across, normal towards the pilot, up) of a section."""
    if sec == 1:
        return RIGHT, UFC_NORMAL, UFC_UP
    return RIGHT, NORMAL, UP


def point(u, v, offset=0.0):
    """A panel point: layout millimetres (u right, v down), raised `offset` m off its surface."""
    sec = section(u, v)
    _, n, up = basis(sec)
    if sec == 1:
        p = UFC_CENTRE + RIGHT * (u / 1000) - up * ((v - UFC_V0) / 1000)
    else:
        p = PANEL_CENTRE + RIGHT * (u / 1000) - up * (v / 1000)
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


# --- keep, drop
KEEP = (
    "seat_", "cushion_", "metal_98", "metal_99", "control_Action_Eject_", "stick", "throttle",
    "black_549", "metal_550", "metal_551", "hud_glass", "approach", "lamp", "control_Action_Gear_.001",
    "button_NEAR", "button_NEXT", "button_NONE", "button_WPN", "button_DTL",
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


for n, like in (("hsi", "adi"), ("screen_rwr", "adi"), ("ded", "adi"), ("trackcam_left", "trackcam")):
    material(n, like)
# Guard covers and red buttons (not a shared material: the game takes its colour from the model).
red = material("red", "yellow")
red.diffuse_color = (0.55, 0.04, 0.03, 1.0)
for nd in red.node_tree.nodes if red.use_nodes else ():
    if nd.type == "BSDF_PRINCIPLED":
        nd.inputs["Base Color"].default_value = red.diffuse_color


# --- building blocks (as the Tornado's)
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


def panel_box(name, mat, r, raise_m, margin=0.0, base=0.0):
    """A plate from `base` to `base + raise_m` off the panel over the rectangle r (mm, within one
    section) grown by `margin` mm."""
    x, y, w, h = r
    c = rect_point(r, base + raise_m / 2)
    return box(name, mat, c, ((w + 2 * margin) / 1000, abs(raise_m), (h + 2 * margin) / 1000), rect_rotation(r))


def bezel(r, offset, height=0.008, lip=0.003):
    """A round black bezel standing `height` out of the panel at `offset` round the rectangle r's
    inscribed circle, the instrument's octagonal case behind it."""
    x, y, w, h = r
    me = cylinder_mesh("black", min(w, h) / 2000 + lip, height, "black", n=ROUND)
    mesh_object("black", me, rect_point(r, offset), rect_rotation(r))
    flat = min(w, h) / 2000 + lip + 0.002
    me = cylinder_mesh("dark", flat / math.cos(math.pi / 8), offset + 0.003, "dark", n=8)
    me.transform(Matrix.Rotation(math.pi / 8, 4, "Y"))
    mesh_object("dark", me, rect_point(r, 0.0), rect_rotation(r))


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


def control(key, r, offset, effect, kind):
    """A push button: origin on the surface, its face 6 mm out; a dark cap body under the face
    and a highlight frame."""
    x, y, w, h = r
    hw, hh = w / 2000, h / 2000
    height, travel = 0.006, 0.004
    name = "control_" + "".join(ch if ch.isalnum() else "_" for ch in effect)
    c = empty(name, rect_point(r, offset), rect_rotation(r), props={"control": effect, "kind": kind, "half": [hw, hh], "height": height, "travel": travel})
    moving = empty("moving", (0, 0, 0), parent=c, props={"moving": True})
    mesh_object("atlas", quad_mesh("atlas", 2 * hw, 2 * hh, "atlas", -height), (0, 0, 0), parent=moving, props={"atlas_face": key, "atlas_uv": [0.0, 0.0, 1.0, 1.0], "no_shadow": True})
    mesh_object("dark", box_mesh("dark", (2 * hw, height - 0.0004, 2 * hh), "dark", (0, -(height - 0.0004) / 2, 0)), (0, 0, 0), parent=moving)
    hl = empty("highlight", (0, 0, 0), parent=c, props={"highlight": True})
    t = 0.002
    for (cx, cz, sx, sz) in ((0, hh + t, 2 * hw + 2 * t, t), (0, -hh - t, 2 * hw + 2 * t, t), (hw + t, 0, t, 2 * hh), (-hw - t, 0, t, 2 * hh)):
        mesh_object("highlight", box_mesh("highlight", (sx, t, sz), "highlight"), (cx, -height, cz), parent=hl, props={"no_shadow": True})


def screen(mat, r, offset, behind=None, round=False):
    """A display over the rectangle r: a quad, or a disc in a bezel (the round instruments)."""
    rot = rect_rotation(r)
    x, y, w, h = r
    if round:
        bezel(r, offset, 0.010, 0.004)
        mesh_object(mat, disc_mesh(mat, min(w, h) / 2000, mat), rect_point(r, offset + 0.0105), rot, props={"no_shadow": True})
        return
    mesh_object(mat, quad_mesh(mat, w / 1000, h / 1000, mat), rect_point(r, offset), rot, props={"no_shadow": True})
    if behind:
        mesh_object(behind, quad_mesh(behind, w / 1000, h / 1000, behind), rect_point(r, offset - 0.003), rot, props={"no_shadow": True})


def plate(name, mat, pieces, depth):
    """A panel plate `depth` m thick behind the panel plane, from convex outlines (layout mm, one
    section) that together make its shape."""
    me = bpy.data.meshes.new(name)
    bm = bmesh.new()
    n = basis(section(*pieces[0][0]))[1]
    for loop in pieces:
        front = [bm.verts.new(point(u, v)) for u, v in loop]
        back = [bm.verts.new(point(u, v) - n * depth) for u, v in loop]
        bm.faces.new(front)
        bm.faces.new(back[::-1])
        for i in range(len(loop)):
            j = (i + 1) % len(loop)
            bm.faces.new((front[i], back[i], back[j], front[j]))
    bmesh.ops.recalc_face_normals(bm, faces=bm.faces[:])
    bm.to_mesh(me)
    bm.free()
    me.materials.append(mats[mat])
    return mesh_object(name, me, (0, 0, 0))


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
    """A toggle switch: a round base and a lever leaning up or down the panel."""
    mesh_object("dark", cylinder_mesh("dark", 0.005, 0.004, "dark", n=KNOB), point(u, v, offset), rotation(u, v))
    lean = Matrix.Rotation(math.radians(-35.0 if up else 35.0), 3, "X")
    me = box_mesh("metal", (0.004, 0.016, 0.004), "metal", (0.0, -0.011, 0.0), lean.to_quaternion())
    mesh_object("metal", me, point(u, v, offset + 0.003), rotation(u, v))


def prism(name, mat, yz, x0, x1):
    """A convex outline in the y-z plane (m) from x0 to x1."""
    me = bpy.data.meshes.new(name)
    bm = bmesh.new()
    sides = [[bm.verts.new((x, y, z)) for y, z in yz] for x in (x0, x1)]
    bm.faces.new(sides[0])
    bm.faces.new(sides[1][::-1])
    n = len(yz)
    for i in range(n):
        j = (i + 1) % n
        bm.faces.new((sides[0][i], sides[1][i], sides[1][j], sides[0][j]))
    bmesh.ops.recalc_face_normals(bm, faces=bm.faces[:])
    bm.to_mesh(me)
    bm.free()
    me.materials.append(mats[mat])
    return mesh_object(name, me, (0, 0, 0))


def rod(name, a, b, radius, mat, parent=root, n=KNOB):
    """A cylinder from a to b (parent frame)."""
    a, b = Vector(a), Vector(b)
    me = bpy.data.meshes.new(name)
    bm = bmesh.new()
    bmesh.ops.create_cone(bm, cap_ends=True, segments=n, radius1=radius, radius2=radius, depth=(b - a).length)
    bm.to_mesh(me)
    bm.free()
    rot = Vector((0, 0, 1)).rotation_difference((b - a).normalized()).to_matrix().to_4x4()
    me.transform(Matrix.Translation((a + b) / 2) @ rot)
    me.materials.append(mats[mat])
    return mesh_object(name, me, (0, 0, 0), parent=parent)


def solid(name, mat, front, back, thickness, up):
    """A sheet `thickness` m thick under the surface between the polylines front and back (the
    glare shield)."""
    me = bpy.data.meshes.new(name)
    bm = bmesh.new()
    top_f = [bm.verts.new(v) for v in front]
    top_b = [bm.verts.new(v) for v in back]
    bot_f = [bm.verts.new(v - up * thickness) for v in front]
    bot_b = [bm.verts.new(v - up * thickness) for v in back]
    n = len(front)
    for i in range(n - 1):
        j = i + 1
        bm.faces.new((top_f[i], top_f[j], top_b[j], top_b[i]))
        bm.faces.new((bot_f[j], bot_f[i], bot_b[i], bot_b[j]))
        bm.faces.new((bot_f[i], bot_f[j], top_f[j], top_f[i]))
        bm.faces.new((top_b[i], top_b[j], bot_b[j], bot_b[i]))
    for i in (0, n - 1):
        bm.faces.new((top_f[i], top_b[i], bot_b[i], bot_f[i]))
    bmesh.ops.recalc_face_normals(bm, faces=bm.faces[:])
    uv = bm.loops.layers.uv.new()
    for f in bm.faces:
        for l in f.loops:
            l[uv].uv = (0.0, 0.0)
    bm.to_mesh(me)
    bm.free()
    me.materials.append(mats[mat])
    return mesh_object(name, me, (0.0, 0.0, 0.0))


def sill(y):
    """The canopy sill's height at y."""
    for (y0, z0), (y1, z1) in zip(SILL_LINE, SILL_LINE[1:]):
        if y <= y1:
            return z0 + (z1 - z0) * (y - y0) / (y1 - y0)
    return SILL_LINE[-1][1]


# --- glare shield: over the panel's top edge (the centre column's standing above the wings'),
# forward to COAMING_BACK sloping down a little towards the windscreen
THICK = 0.016
LIP = 0.008
edge = [
    point(-WING, TOP_WING + BEVEL), point(-(WING - BEVEL), TOP_WING), point(-(HINGE + STEP), TOP_WING),
    point(-HINGE, TOP_CENTRE), point(HINGE, TOP_CENTRE), point(HINGE + STEP, TOP_WING),
    point(WING - BEVEL, TOP_WING), point(WING, TOP_WING + BEVEL),
]
front = [e + UP * THICK + NORMAL * LIP for e in edge]
fall = Vector((0.0, 0.0, -COAMING_DROP))
back = [Vector((f.x, COAMING_BACK, f.z)) + fall for f in front]
solid("black", "black", front, back, THICK, Vector((0.0, 0.0, 1.0)))
# Out to the walls past the wings' outer edges.
for s in (-1, 1):
    f = front[0 if s < 0 else -1]
    yz = [(f.y, f.z), (COAMING_BACK, f.z - COAMING_DROP), (COAMING_BACK, f.z - COAMING_DROP - THICK), (f.y, f.z - THICK)]
    prism("black", "black", yz, *sorted((s * WING / 1000, s * WALL_X)))
# A bulkhead under its back edge.
z_back = max(b.z for b in back)
box("dark", "dark", (0.0, COAMING_BACK + 0.01, (FLOOR + z_back) / 2), (2 * WALL_X, 0.02, z_back - FLOOR))


def coaming_z(x, y):
    """The glare shield's top at (x, y) (between its front edge and the back)."""
    for a, b in zip(front, front[1:]):
        if a.x <= x <= b.x and b.x > a.x:
            f = a.lerp(b, (x - a.x) / (b.x - a.x))
            return f.z - COAMING_DROP * (y - f.y) / (COAMING_BACK - f.y)
    return z_back


# --- panel: the centre column, the wings round the MFDs' wells, the lower panel
scr = layout["screens"]
faces = {f["key"]: f["rect"] for f in layout["faces"]}


def well(r):
    x, y, w, h = r
    return (x - 19.0, y - 25.0, w + 38.0, h + 50.0)


wells = {key: well(scr[key]) for key in ("left", "right")}
bevel_v = TOP_WING + BEVEL
for key in ("left", "right"):
    x, y, w, h = wells[key]
    s_ = -1 if x < 0 else 1
    # In the right wing's terms (the left one mirrored): the step, above, outside, below and
    # inside the well.
    x0, x1 = (x, x + w) if s_ > 0 else (-(x + w), -x)
    y0, y1 = y, y + h
    bevel_u = WING - BEVEL + (y0 - TOP_WING)
    pieces = [
        [(HINGE, TOP_CENTRE), (HINGE + STEP, TOP_WING), (HINGE, TOP_WING)],
        [(HINGE, TOP_WING), (WING - BEVEL, TOP_WING), (bevel_u, y0), (HINGE, y0)],
        [(x1, y0), (bevel_u, y0), (WING, bevel_v), (WING, SHOULDER), (x1, SHOULDER)],
        [(x0, y1), (x1, y1), (x1, SHOULDER), (x0, SHOULDER)],
        [(HINGE, y0), (x0, y0), (x0, SHOULDER), (HINGE, SHOULDER)],
    ]
    plate("panel_grey", "panel_grey", [[(s_ * u, v) for u, v in piece] for piece in pieces], 0.03)
plate("panel_grey", "panel_grey", [
    [(-HINGE, TOP_CENTRE), (HINGE, TOP_CENTRE), (HINGE, SHOULDER), (-HINGE, SHOULDER)],
    [(-LOWER, SHOULDER), (LOWER, SHOULDER), (LOWER, BOTTOM), (-LOWER, BOTTOM)],
], 0.03)
# Dark recesses out to the walls beside the wings, under the glare shield.
for s_ in (-1, 1):
    u0, u1 = sorted((s_ * WING, s_ * WALL_X * 1000))
    plane_box("dark", "dark", (u0, TOP_WING, u1 - u0, SHOULDER - TOP_WING), -0.04, -0.02)
# Rims standing out along the panel's lower edge, down the wings' and the lower panel's outer
# edges.
panel_box("dark", "dark", (-LOWER, BOTTOM - 6.0, 2 * LOWER, 6.0), 0.007)
for s_ in (-1, 1):
    panel_box("dark", "dark", (s_ * (WING - 3.0) - 3.0, bevel_v, 6.0, SHOULDER - bevel_v), 0.007)
    panel_box("dark", "dark", (s_ * (LOWER - 3.0) - 3.0, SHOULDER, 6.0, BOTTOM - 6.0 - SHOULDER), 0.007)
    u0, u1 = sorted((s_ * WING, s_ * LOWER))
    panel_box("dark", "dark", (u0, SHOULDER, u1 - u0, 6.0), 0.007)

# --- tub: floor, consoles with fillers down to the panel's lower edge, walls up to the sills,
# knee wall with the footwell, centre pedestal
box("dark", "dark", (0.0, (SILL_REAR + COAMING_BACK) / 2, FLOOR - 0.01), (2 * WALL_X, COAMING_BACK - SILL_REAR, 0.02))
low = point(0, BOTTOM)
for s in (-1, 1):
    cx = s * sum(CONSOLE_X) / 2
    y0, y1 = CONSOLE_Y
    box("panel_grey", "panel_grey", (cx, (y0 + y1) / 2, (CONSOLE_TOP + FLOOR) / 2), (CONSOLE_X[1] - CONSOLE_X[0], y1 - y0, CONSOLE_TOP - FLOOR))
    box("dark", "dark", (cx, (y0 + y1) / 2, CONSOLE_TOP + 0.002), (CONSOLE_X[1] - CONSOLE_X[0] - 0.01, y1 - y0 - 0.02, 0.004))
    # From the console's end down to the panel's lower edge.
    yz = [(y1, CONSOLE_TOP), (low.y, low.z), (low.y, FLOOR), (y1, FLOOR)]
    prism("dark", "dark", yz, *sorted(s * x for x in CONSOLE_X))
# Walls up to the sills, the sill rails along them.
for s in (-1, 1):
    yz = [(SILL_REAR, FLOOR), (COAMING_BACK, FLOOR)] + [(y, z) for y, z in reversed(SILL_LINE)]
    prism("panel_grey", "panel_grey", yz, *sorted((s * WALL_X, s * (WALL_X + 0.02))))
    for (ya, za), (yb, zb) in zip(SILL_LINE, SILL_LINE[1:]):
        d = Vector((0.0, yb - ya, zb - za))
        rot = Euler((math.atan2(d.z, d.y), 0.0, 0.0))
        box("frame", "frame", (s * (WALL_X + 0.002), (ya + yb) / 2, (za + zb) / 2 - 0.012), (0.03, d.length + 0.02, 0.025), rot)
# Rear bulkhead behind the seat, up to the sills.
box("panel_grey", "panel_grey", (0.0, SILL_REAR - 0.01, (sill(SILL_REAR) + FLOOR) / 2), (2 * WALL_X + 0.04, 0.02, sill(SILL_REAR) - FLOOR))
# Knee wall under the panel with the opening for the legs, the footwell behind it.
LEGS = (0.20, -0.76)  # opening's half width, top (m)
wall = low.y + 0.08
for s_ in (-1, 1):
    box("panel_grey", "panel_grey", (s_ * (LEGS[0] + WALL_X) / 2, wall, (low.z + FLOOR) / 2), (WALL_X - LEGS[0], 0.02, low.z - FLOOR))
box("panel_grey", "panel_grey", (0.0, wall, (low.z + LEGS[1]) / 2), (2 * LEGS[0], 0.02, low.z - LEGS[1]))
WELL_DEPTH = 0.15
yc = wall + WELL_DEPTH / 2
for s_ in (-1, 1):
    box("dark", "dark", (s_ * (LEGS[0] + 0.01), yc, (LEGS[1] + FLOOR) / 2), (0.02, WELL_DEPTH, LEGS[1] - FLOOR))
box("dark", "dark", (0.0, yc, LEGS[1] + 0.01), (2 * LEGS[0] + 0.04, WELL_DEPTH, 0.02))
box("dark", "dark", (0.0, wall + WELL_DEPTH, (LEGS[1] + FLOOR) / 2), (2 * LEGS[0] + 0.04, 0.02, LEGS[1] - FLOOR))
# Under the panel's lower edge back to the knee wall.
box("dark", "dark", (0.0, (low.y + wall) / 2, low.z - 0.01), (2 * WALL_X, wall - low.y + 0.02, 0.02))
# Centre pedestal between the legs: a panel of knobs and toggles down its face.
col = point(0.0, BOTTOM, -0.03)
PEDESTAL = 0.07  # half width
pfront = col.y - 0.004
ptop = LEGS[1] - 0.01
box("panel_grey", "panel_grey", (0.0, (pfront + wall + WELL_DEPTH) / 2, (ptop + FLOOR) / 2), (2 * PEDESTAL, wall + WELL_DEPTH - pfront, ptop - FLOOR))
zt = ptop - 0.02
box("black", "black", (0.0, pfront - 0.002, zt - 0.06), (0.12, 0.004, 0.12))
for r in range(3):
    for c in range(3):
        p = Vector(((c - 1) * 0.034, pfront - 0.004, zt - 0.02 - r * 0.036))
        if (r + c) % 3 == 2:
            mesh_object("dark", cylinder_mesh("dark", 0.005, 0.004, "dark", n=KNOB), p)
            lean = Euler((math.radians(-30.0 if r % 2 else 30.0), 0, 0)).to_quaternion()
            mesh_object("metal", box_mesh("metal", (0.004, 0.016, 0.004), "metal", (0.0, -0.008, 0.0), lean), p - Vector((0, 0.003, 0)))
        else:
            mesh_object("metal", cylinder_mesh("metal", 0.009 if r == 0 else 0.007, 0.012, "metal", n=KNOB), p)

# --- HUD: its body standing on the glare shield, the up-front controller on its face; the
# combiner glass over it between two posts with a bar across the top
ufc_y = lambda z: UFC_CENTRE.y + (z - UFC_CENTRE.z) * math.tan(UFC_TILT)  # noqa: E731
hud_low = coaming_z(0.0, HUD_Y) - 0.02
mid = HUD_BODY_TOP - 0.026
half_x = HUD_GLASS[0]
prism("dark", "dark", [(ufc_y(hud_low), hud_low), (HUD_BODY_BACK, hud_low), (HUD_BODY_BACK, mid), (ufc_y(mid), mid)], -half_x, half_x)
# Its top reaching out under the glass.
prism("dark", "dark", [(HUD_Y - 0.01, HUD_BODY_TOP), (HUD_BODY_BACK, HUD_BODY_TOP), (HUD_BODY_BACK, mid), (ufc_y(mid), mid)], -half_x, half_x)

# --- plates, bezels, faces and displays
PLATE = 0.004
plates = [
    ("black", (-134.0, -262.0, 262.0, 68.0)),  # up-front controller
    ("dark", (-154.0, -161.0, 308.0, 18.0)),  # fire handles
    ("dark", (-161.0, -141.0, 82.0, 30.0)),  # threat lights
    ("dark", (79.0, -141.0, 82.0, 30.0)),  # engine and flight control lights
    ("black", (-70.0, -141.0, 140.0, 51.0)),  # countermeasures
    ("dark", (-416.0, 66.0, 48.0, 94.0)),  # gear lights, flaps, brakes, autopilot
    ("black", (-366.0, 76.0, 40.0, 86.0)),  # gear handle, downlock override
    ("black", (-322.0, 64.0, 154.0, 172.0)),  # armament panel
    ("dark", (-156.0, 54.0, 88.0, 64.0)),  # clock, AOA
    ("black", (-156.0, 122.0, 88.0, 112.0)),  # switches under them
    ("dark", (-86.0, 196.0, 172.0, 36.0)),  # target buttons
    ("dark", (77.0, 59.0, 104.0, 104.0)),  # engine instruments
    ("black", (77.0, 168.0, 104.0, 66.0)),  # switches under them
    ("dark", (192.0, 66.0, 124.0, 57.0)),  # caution lights
    ("black", (192.0, 128.0, 136.0, 106.0)),  # fuel display selector, switches
    ("dark", (332.0, 90.0, 66.0, 66.0)),  # fuel quantity
    ("black", (332.0, 160.0, 66.0, 74.0)),  # under it
]
for mat, r in plates:
    panel_box(mat, mat, r, PLATE)
# Up-front controller: blank function keys round the master caution and the HUD mode lamps,
# rocker switches either side of the keypad and the data entry display, two thumb wheels.
for k in range(12):
    col_, row = k % 3, k // 3
    if col_ < 2 and row < 2:
        continue
    panel_box("dark", "dark", (24.0 + 34.0 * col_, -256.0 + 15.0 * row, 30.0, 13.0), 0.005, 0.0, PLATE)
for u in (-130.0, 125.0):
    for v in (-250.0, -222.0):
        panel_box("dark", "dark", (u - 1.0, v - 6.0, 3.0, 20.0), 0.004, 0.0, PLATE)
for u in (-118.0, -84.0):
    mesh_object("metal", cylinder_mesh("metal", 0.006, 0.006, "metal", n=12), point(u, -272.0), rotation(u, -272.0))
# The HUD's own controls on the body's face over the up-front controller: brightness and
# contrast knobs, the standby reticle and test switches.
panel_box("black", "black", (-80.0, -306.0, 160.0, 30.0), PLATE)
for u in (-60.0, -30.0):
    knob(u, -291.0, 0.007, 0.010, offset=PLATE)
for u in (10.0, 34.0, 58.0):
    toggle(u, -291.0, PLATE, up=(u != 34.0))
# Fire handles: a red T handle under each light.
for x in (-150.0, -38.0, 74.0):
    panel_box("red", "red", (x + 18.0, -145.0, 40.0, 3.0), 0.010, 0.0, PLATE)
# Jettison button outboard of the left MFD, the fire extinguisher discharge switch under its red
# guard outboard of the right one (on the wings' strips between the wells and the rims, clear of
# the bezels).
mesh_object("red", cylinder_mesh("red", 0.006, 0.008, "red", n=12), point(-381.0, -80.0), rotation(-381.0, -80.0))
mesh_object("dark", cylinder_mesh("dark", 0.008, 0.003, "dark", n=12), point(-381.0, -80.0), rotation(-381.0, -80.0))
toggle(381.0, -80.0)
mesh_object("red", box_mesh("red", (0.014, 0.012, 0.022), "red", (0.0, -0.006, -0.002)), point(381.0, -80.0), rotation(381.0, -80.0))
# Downlock override: a red button on the stripes.
x, y, w, h = faces["Item(Hazard)"]
mesh_object("red", cylinder_mesh("red", 0.006, 0.010, "red", n=KNOB), point(x + w / 2, y + h / 2, PLATE + 0.001), rotation(x, y))
# Lights switches by the gear handle; the armament panel's switches (master arm, gun / pylons,
# laser, targeting pod, the HUD's modes) and knobs.
for v in (178.0, 204.0, 226.0):
    toggle(-400.0, v, PLATE, up=(v != 204.0))
    toggle(-380.0, v, PLATE, up=(v == 204.0))
for v in (190.0, 216.0):
    toggle(-346.0, v, PLATE)
for r_ in range(4):
    for c_ in range(4):
        u, v = -302.0 + 38.0 * c_, 86.0 + 40.0 * r_
        if (r_, c_) in ((2, 3), (3, 0)):
            knob(u, v, 0.007, 0.012, offset=PLATE)
        else:
            toggle(u, v, PLATE, up=(r_ + c_) % 2 == 0)
mesh_object("red", box_mesh("red", (0.014, 0.012, 0.022), "red", (0.0, -0.006, -0.002)), point(-302.0, 86.0, PLATE), rotation(-302.0, 86.0))
# Under the clock: the HARS switches and the slaving knobs.
for v in (140.0, 172.0, 204.0):
    toggle(-136.0, v, PLATE, up=(v != 172.0))
    knob(-100.0, v, 0.006, 0.010, offset=PLATE)
# Under the engine instruments: the engine start, fuel flow and APU switches.
for u in (92.0, 116.0, 140.0, 164.0):
    toggle(u, 186.0, PLATE, up=(u < 120.0))
    knob(u, 214.0, 0.005, 0.009, "dark", offset=PLATE)
# Fuel display selector (a large knob), test buttons and switches; the fuel test button.
knob(240.0, 170.0, 0.016, 0.016, offset=PLATE)
for u in (290.0, 310.0):
    toggle(u, 150.0, PLATE)
for u in (210.0, 240.0, 270.0, 300.0):
    toggle(u, 214.0, PLATE, up=(u != 240.0))
knob(365.0, 180.0, 0.007, 0.010, "dark", offset=PLATE)
toggle(365.0, 212.0, PLATE)
# MFD bezels: a housing standing out of the well to the bezel's face (BEZEL off the panel), the
# screen back in a black recess; twenty push buttons round the screen with ridges between them
# and rocker switches at the corners.
BEZEL = 0.006
for key in ("left", "right"):
    x, y, w, h = scr[key]
    wx, wy, ww, wh = wells[key]
    hole = (x - 3.0, y - 3.0, w + 6.0, h + 6.0)
    frame("dark", (wx - 3.0, wy - 3.0, ww + 6.0, wh + 6.0), hole, -0.03, BEZEL - 0.002)
    frame("black", (wx + 1.0, wy + 1.0, ww - 2.0, wh - 2.0), hole, BEZEL - 0.002, BEZEL)
    frame("black", hole, (x - 0.5, y - 0.5, w + 1.0, h + 1.0), -0.006, BEZEL - 0.001)
    for k in range(1, 5):
        for by in (y - 16.5, y + h + 6.5):
            panel_box("black", "black", (x + w * k / 5 - 1.0, by - 4.0, 2.0, 8.0), 0.003, 0.0, BEZEL)
    for k in range(1, 5):
        for bx in (x - 9.0, x + w + 9.0):
            panel_box("black", "black", (bx - 4.0, y + h * k / 5 - 1.0, 8.0, 2.0), 0.003, 0.0, BEZEL)
    for bx in (x - 9.0, x + w + 9.0):
        for by in (y - 16.0, y + h + 16.0):
            panel_box("dark", "dark", (bx - 4.5, by - 6.5, 9.0, 13.0), 0.003, 0.0, BEZEL)
            panel_box("black", "black", (bx - 1.5, by - 5.0, 3.0, 10.0), 0.003, 0.0, BEZEL + 0.003)


# Setting knobs at instrument corners: airspeed and altimeter, the attitude indicator, the HSI
# (heading and course) and the clock.
def corner(r, sx, sy, inset=3.0):
    x, y, w, h = r
    return (x + w - inset if sx > 0 else x + inset, y + h - inset if sy > 0 else y + inset)


for r, sx, sy in (
    (faces["Item(Dial(Airspeed))"], 1, 1),
    (faces["Item(Dial(Altimeter))"], -1, 1),
    (scr["adi"], 1, 1),
    (scr["hsi"], -1, 1),
    (scr["hsi"], 1, 1),
    (faces["Item(Dial(Clock))"], -1, 1),
    (faces["Item(Dial(Fuel))"], -1, 1),
):
    u, v = corner(r, sx, sy)
    knob(u, v, 0.0045, 0.009, "dark", offset=PLATE)
screen("screen_left", scr["left"], -0.004, "trackcam_left")
screen("screen_right", scr["right"], -0.004, "trackcam")
screen("adi", scr["adi"], 0.0, round=True)
screen("hsi", scr["hsi"], 0.0, round=True)
screen("screen_rwr", scr["rwr"], 0.0, round=True)
screen("ded", scr["ded"], PLATE + 0.0005)
# The data entry display's black frame.
x, y, w, h = scr["ded"]
frame("black", (x - 3.0, y - 3.0, w + 6.0, h + 6.0), (x, y, w, h), 0.0, PLATE + 0.004)
for f in layout["faces"]:
    if f["control"] is None:
        face(f["key"], f["rect"], PLATE + 0.0005, f["round"])
        if f["key"].startswith("Item(Dial(") and not f["round"]:
            # The tapes' black window frames.
            x, y, w, h = f["rect"]
            frame("black", (x - 3.0, y - 3.0, w + 6.0, h + 6.0), (x, y, w, h), 0.0, PLATE + 0.005)
    elif f["kind"] == "osb":
        control(f["key"], f["rect"], BEZEL, f["control"], "osb")
    else:
        control(f["key"], f["rect"], PLATE, f["control"], f["kind"])

# --- side console panels: black plates on the consoles' tops round the throttle, the control
# display unit and switch panels on the right one (dressing; none of them is a control)
CONSOLE_FACE = CONSOLE_TOP + 0.004


def fitting(c, p):
    """A console fitting at p (on a panel's top): k / K small / large knob, t / T toggle leaning
    forward / back, g red guard, r red button, b a keypad key."""
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


def console_panel(x0, x1, y0, y1, rows, h=0.006):
    """A plate over x0..x1, y0..y1 (m) on a console, its fittings in rows from the front."""
    box("black", "black", ((x0 + x1) / 2, (y0 + y1) / 2, CONSOLE_FACE + h / 2), (x1 - x0, y1 - y0, h))
    for i, row in enumerate(rows):
        y = y1 - (y1 - y0) * (i + 0.5) / len(rows)
        for j, c in enumerate(row):
            fitting(c, Vector((x0 + (x1 - x0) * (j + 0.5) / len(row), y, CONSOLE_FACE + h)))


for s in (-1, 1):
    def side(x0, x1, y0, y1, rows):
        """A panel given for the right console (x outwards), mirrored on the left."""
        console_panel(*sorted((s * x0, s * x1)), y0, y1, [r if s > 0 else r[::-1] for r in rows])
    # At the back, the width of the console.
    side(0.23, 0.42, -0.29, -0.19, ["kKk", "tgt"])
    side(0.23, 0.42, -0.18, -0.08, ["tttt", "kTk"] if s > 0 else ["tttt", "rgrg"])
for rows, rect in (
    (["kk", "TT", "kr"], (-0.42, -0.23, -0.07, 0.04)),  # behind the throttle
    (["k", "t", "r", "k"], (-0.42, -0.31, 0.06, 0.30)),  # outboard of its rails
    (["t", "k", "t"], (-0.245, -0.226, 0.06, 0.30)),  # inboard of them
    (["kkk", "ttt"], (-0.42, -0.23, 0.31, 0.41)),  # ahead of them
    # The right console: the control display unit's keys, switch panels.
    (["kKk", "ttt", "rr"], (0.23, 0.42, -0.07, 0.07)),
    (["bbbbb", "bbbbb", "bbbbb", "bbbbb"], (0.23, 0.33, 0.08, 0.20)),
    (["kk", "tt", "kk"], (0.34, 0.42, 0.08, 0.20)),
    (["tkt", "bbbbbb", "bbbbbb", "kKk"], (0.23, 0.42, 0.21, 0.41)),
):
    console_panel(*rect, rows)
# The control display unit's screen on the right console, dark.
box("black", "black", (0.33, 0.255, CONSOLE_FACE + 0.007), (0.15, 0.06, 0.004))
# Yellow handles on the consoles' inner faces by the pilot's knees.
for s, y in ((-1, 0.20), (1, 0.06)):
    box("yellow", "yellow", (s * (CONSOLE_X[0] - 0.022), y, CONSOLE_TOP - 0.07), (0.012, 0.012, 0.08))
    for dz in (-0.03, 0.03):
        box("yellow", "yellow", (s * (CONSOLE_X[0] - 0.011), y, CONSOLE_TOP - 0.07 + dz), (0.022, 0.01, 0.01))

# --- the kept parts
# Seat: its cushion's top at SEAT_TOP, on a pedestal from the floor.
seat = [o for o in old if o.name.startswith(SEAT)]
bpy.context.view_layer.update()
cushion = min((o for o in seat if o.name.startswith("cushion_")), key=lambda o: o.location.z)
cushion_top = max((cushion.matrix_world @ v.co).z for v in cushion.data.vertices)
for o in seat:
    o.location.z += SEAT_TOP - cushion_top
bpy.context.view_layer.update()
bottom = min((o.matrix_world @ v.co).z for o in seat if o.type == "MESH" for v in o.data.vertices)
box("dark", "dark", (0.0, -0.08, (bottom + FLOOR) / 2), (0.36, 0.40, bottom - FLOOR))
# Centre stick: pivot on the floor, the grip raised on a shaft, a boot at its foot.
stick = objs["stick"]
stick.location = STICK_PIVOT
for c in stick.children:
    c.location.z += STICK_RAISE
stick["grip"] = float(stick["grip"]) + STICK_RAISE
rod("metal", (0, 0, 0), (0, 0, STICK_RAISE + 0.02), 0.011, "metal", stick)
rod("rubber", STICK_PIVOT, STICK_PIVOT + Vector((0, 0, 0.06)), 0.04, "rubber")
box("dark", "dark", STICK_PIVOT + Vector((0.0, 0.0, -0.004)), (0.12, 0.12, 0.008))
# Throttle: the kept handle's travel lengthened to THROTTLE_TRAVEL, its rails along it.
throttle = objs["throttle"]
full = objs["throttle_full"]
full.location = throttle.location + Vector((0.0, THROTTLE_TRAVEL, 0.0))
y0, y1 = throttle.location.y - 0.03, full.location.y + 0.03
for n in RAILS:
    o = objs[n]
    o.data = o.data.copy()
    vs = [v.co.y for v in o.data.vertices]
    k = (y1 - y0) / (max(vs) - min(vs))
    o.data.transform(Matrix.Diagonal((1.0, k, 1.0, 1.0)))
    o.location.y = (y0 + y1) / 2
# Target buttons in a row under the HSI; the gear handle on the lower left panel right of the
# gear lights.
for i, n in enumerate(["button_NEAR", "button_NEXT", "button_NONE", "button_WPN", "button_DTL"]):
    o = objs[n]
    u, v = -64.0 + 32.0 * i, 214.0
    o.location = point(u, v, PLATE)
    o.rotation_mode = "QUATERNION"
    o.rotation_quaternion = rotation(u, v)
g = objs["control_Action_Gear_.001"]
g.location = point(-346.0, 100.0, PLATE)
g.rotation_mode = "QUATERNION"
g.rotation_quaternion = rotation(-346.0, 100.0)
# HUD glass: the combiner over the body (its vertices are absolute), a frame of bars along its
# outline (posts down to the glare shield, chamfered top corners, top), outside it by half a bar.
hud = next(o for o in old if o.name.startswith("hud_glass"))
lo = Vector((min(v.co.x for v in hud.data.vertices), 0.0, min(v.co.z for v in hud.data.vertices)))
hi = Vector((max(v.co.x for v in hud.data.vertices), 0.0, max(v.co.z for v in hud.data.vertices)))
hud.location = (0.0, 0.0, 0.0)
for v in hud.data.vertices:
    tx, tz = (v.co.x - lo.x) / (hi.x - lo.x), (v.co.z - lo.z) / (hi.z - lo.z)
    v.co = Vector((-HUD_GLASS[0] + 2 * HUD_GLASS[0] * tx, HUD_Y, HUD_GLASS[1] + (HUD_GLASS[2] - HUD_GLASS[1]) * tz))
glass = [v.co for v in hud.data.vertices]
top = max(v.z for v in glass)
shoulder = max(v.z for v in glass if abs(abs(v.x) - half_x) < 0.01)
top_x = min(abs(v.x) for v in glass if abs(v.z - top) < 0.002)
post_x = half_x + HUD_BAR / 2
foot = [coaming_z(s * post_x, HUD_Y) for s in (-1, 1)]
outline = [
    Vector((-post_x, foot[0])), Vector((-post_x, shoulder)),
    Vector((-top_x, top + HUD_BAR / 2)), Vector((top_x, top + HUD_BAR / 2)),
    Vector((post_x, shoulder)), Vector((post_x, foot[1])),
]
for a, b in zip(outline, outline[1:]):
    d = b - a
    c = (a + b) / 2
    rot = Euler((0.0, -math.atan2(d.y, d.x), 0.0))
    box("frame", "frame", (c.x, HUD_Y, c.y), (d.length + HUD_BAR, HUD_BAR, HUD_BAR), rot)
# The posts' side plates back along the body.
for s in (-1, 1):
    box("frame", "frame", (s * post_x, (HUD_Y + HUD_BODY_BACK) / 2, (foot[0] + HUD_BODY_TOP) / 2), (HUD_BAR, HUD_BODY_BACK - HUD_Y, HUD_BODY_TOP - foot[0]))
# The approach indexer just outside the left post, in a housing behind it, on a stalk.
ind = next(o for o in old if o.name.startswith("approach"))
ind.location = Vector((-(post_x + HUD_BAR / 2 + 0.016), HUD_Y, -0.07))
box("dark", "dark", (ind.location.x, HUD_Y + 0.007, ind.location.z), (0.028, 0.010, 0.068))
box("dark", "dark", (ind.location.x, HUD_Y + 0.007, (ind.location.z - 0.034 + foot[0]) / 2), (0.010, 0.010, ind.location.z - 0.034 - foot[0]))
# Flood lamps: two behind the pilot's head, one under the HUD's hood.
objs["lamp.002"].location = Vector((0.0, 0.62, -0.10))

bpy.data.orphans_purge(do_recursive=True)
bpy.context.view_layer.update()
print(f"stick grip {stick['grip']:.3f} m; throttle {throttle.location} -> {full.location}; "
      f"glare shield front z {max(f.z for f in front):.3f} at y {min(f.y for f in front):.3f}; "
      f"HUD posts on {foot}; seat bottom {bottom:.3f}; panel bottom {low}")
bpy.ops.wm.save_as_mainfile(filepath=out)
print("built", out, "objects", len(bpy.data.objects))
