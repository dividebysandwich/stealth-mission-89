# Builds the AV-8B Harrier II cockpit master (assets-src/blend/cockpit_av8b.blend) from the panel
# layout, after the AV-8B Night Attack's front panel, with parts of the F-117A's modern cockpit:
#   blender -b assets-src/blend/cockpit_modern.blend --python-exit-code 1 \
#       --python tools/models/build_cockpit_av8b.py -- \
#       assets-src/blend/cockpit_av8b_layout.json assets-src/blend/cockpit_av8b.blend
# then tools/models/export.sh cockpit_av8b. The layout JSON is written by
# `UPDATE_LAYOUT=1 cargo test layout_files` from `PanelLayout::av8b` (src/cockpit3d/layout.rs).
#
# Life size, following tools/models/build_cockpit_a10.py (the F-117A's kept parts scaled down
# by F117_SCALE about the eye). Kept, with their materials, labels and control properties: the
# seat with the ejection handle, the stick (a centre stick on a shaft from the floor between
# the knees), the throttle with its rails on the left console (moved outboard), the gear handle
# and the five target buttons, the HUD glass, the approach indexer and the flood lamps. Built
# here, low-poly: the flat main panel with the up-front controller raised in its middle under
# the HUD, the two MFDs in square bezels either side, the round standby instruments under the
# controller; the glare shield over it, kept low (the Harrier's view over the nose), the HUD
# body on it with the combiner; the consoles with their switch panels, the nozzle lever's
# quadrant just inboard of the throttle with the lever (`nozzle`, swinging about the
# `nozzle_pivot` anchor back to `nozzle_down`) and its STO and hover stops; the knee wall with
# the footwell and the centre pedestal, the walls up to the canopy sills.
#
# It fits the exterior's canopy (the eye 0.27 m under the bubble's top): the sills 0.40 m under
# the eye 0.40 m behind it, 0.43 m at the eye, 0.45 m at the frame bow 0.65 m ahead, the
# bubble 0.50 m either side; the windscreen's base 1.60 m ahead, 0.51 m under the eye, its top
# 0.18 m under the eye 1.10 m ahead. The walls and consoles stay under the sills, the glare
# shield and the HUD under the windscreen; the glare shield's back edge 0.35 m under the eye
# 0.95 m ahead (20 degrees down; the nose top is 17.8 degrees down).
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
# The main panel: the layout's origin, the top (layout y -150) leaning away 0.31 m under the eye.
PANEL_TILT = math.radians(20.0)
PANEL_TOP = -150.0
PANEL_CENTRE = Vector((0.0, 0.70 - 0.15 * math.sin(PANEL_TILT), -0.31 - 0.15 * math.cos(PANEL_TILT)))
FLOOR = -1.0
# The walls' inner faces; their tops (the canopy sills) along SILL_LINE ((y, z), from the rear
# bulkhead to the glare shield), a little under the exterior's sills.
WALL_X = 0.48
SILL_REAR = -0.62
SILL_LINE = [(SILL_REAR, -0.405), (-0.40, -0.415), (0.0, -0.445), (0.65, -0.465), (0.95, -0.48)]
# Consoles (the throttle and the nozzle lever on the left one): x, y and their top (the kept
# throttle rails' own).
CONSOLE_X = (0.22, WALL_X)
CONSOLE_Y = (-0.30, 0.42)
CONSOLE_TOP = -0.646
# The panel's outline (layout mm): out to WING, its top corners bevelled down BEVEL (45
# degrees), down to BOTTOM.
WING = 420.0
BEVEL = 40.0
BOTTOM = 250.0
# The up-front controller (|x| <= UFC_HALF, y < UFC_LINE) standing UFC_RAISE m off the panel.
UFC_HALF = 100.0
UFC_LINE = -10.0
UFC_RAISE = 0.02
# The glare shield: from the panel's top forward to COAMING_BACK in the middle (narrowing to
# the windscreen's sides), dropping COAMING_DROP.
COAMING_BACK = 0.95
COAMING_DROP = 0.04
# HUD combiner glass: half width, bottom and top (m), at HUD_Y in front of the eye, under the
# windscreen (AircraftSpec::hud_scale fits the symbology into it); its body under it from
# HUD_BODY_LOW over the controller up to HUD_BODY_TOP, back to HUD_BODY_BACK.
HUD_Y = 0.66
HUD_GLASS = (0.15, -0.13, 0.075)
HUD_BAR = 0.012
HUD_BODY_TOP = -0.19
HUD_BODY_LOW = -0.255
HUD_BODY_BACK = 0.88
# Seat: its cushion's top, the eye over the hips.
SEAT_TOP = -0.80
# Centre stick: pivot on the floor between the knees, the kept grip raised on a shaft.
STICK_PIVOT = Vector((0.0, 0.32, FLOOR + 0.01))
STICK_RAISE = 0.28
# The throttle: moved outboard THROTTLE_DX, its travel along the console.
THROTTLE_DX = -0.08
THROTTLE_TRAVEL = 0.17
# The nozzle lever: its pivot under the quadrant, the knob NOZZLE_ARM above it, leaning
# NOZZLE_FWD forward at lever 0 (nozzles aft) and NOZZLE_BACK back at 1 (the braking stop,
# 98.5 degrees); the STO stop at 50 and the hover stop at 90 degrees of nozzle.
NOZZLE_PIVOT = Vector((-0.25, 0.13, CONSOLE_TOP - 0.06))
NOZZLE_ARM = 0.16
NOZZLE_FWD = math.radians(40.0)
NOZZLE_BACK = math.radians(-35.0)
NOZZLE_TRAVEL = 98.5
NOZZLE_STOPS = ((50.0, "yellow"), (90.0, "red"))
# Segments of the round parts (bezels, buttons, knobs): low-poly.
ROUND = 16
KNOB = 8

RIGHT = Vector((1.0, 0.0, 0.0))
UP = Vector((0.0, math.sin(PANEL_TILT), math.cos(PANEL_TILT)))
NORMAL = Vector((0.0, -math.cos(PANEL_TILT), math.sin(PANEL_TILT)))


def in_ufc(u, v):
    return abs(u) <= UFC_HALF and v < UFC_LINE


def point(u, v, offset=0.0):
    """A panel point: layout millimetres (u right, v down), raised `offset` m off its surface
    (the controller's face UFC_RAISE off the panel)."""
    if in_ufc(u, v):
        offset += UFC_RAISE
    return PANEL_CENTRE + RIGHT * (u / 1000) - UP * (v / 1000) + NORMAL * offset


def rotation(u=0.0, v=0.0):
    return Matrix((RIGHT, -NORMAL, UP)).transposed().to_quaternion()


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


for n, like in (("ded", "adi"), ("trackcam_left", "trackcam")):
    material(n, like)
# Guard covers and red buttons (not a shared material: the game takes its colour from the model).
red = material("red", "yellow")
red.diffuse_color = (0.55, 0.04, 0.03, 1.0)
for nd in red.node_tree.nodes if red.use_nodes else ():
    if nd.type == "BSDF_PRINCIPLED":
        nd.inputs["Base Color"].default_value = red.diffuse_color


# --- building blocks (as the A-10's)
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
    rot = {
        "Z": Matrix.Identity(4),
        "-Y": Matrix.Rotation(math.radians(90.0), 4, "X"),
        "X": Matrix.Rotation(math.radians(90.0), 4, "Y"),
    }[axis]
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
    """A plate from `base` to `base + raise_m` off the panel over the rectangle r (mm) grown by
    `margin` mm."""
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
    """A panel plate `depth` m thick behind the panel plane, from convex outlines (layout mm)
    that together make its shape."""
    me = bpy.data.meshes.new(name)
    bm = bmesh.new()
    for loop in pieces:
        front = [bm.verts.new(point(u, v) if not in_ufc(u, v) else point(u, v) - NORMAL * UFC_RAISE) for u, v in loop]
        back = [bm.verts.new(f.co - NORMAL * depth) for f in front]
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
    """A box over the rectangle r (layout mm) from d0 to d1 m off the panel plane (the panel's,
    not the controller's)."""
    x, y, w, h = r
    c = PANEL_CENTRE + RIGHT * ((x + w / 2) / 1000) - UP * ((y + h / 2) / 1000) + NORMAL * ((d0 + d1) / 2)
    return box(name, mat, c, (w / 1000, abs(d1 - d0), h / 1000), rotation())


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


def slab(name, mat, outline, thickness):
    """A convex outline of (x, y, z) points (m), `thickness` m thick downwards."""
    me = bpy.data.meshes.new(name)
    bm = bmesh.new()
    top = [bm.verts.new(p) for p in outline]
    bot = [bm.verts.new(Vector(p) - Vector((0.0, 0.0, thickness))) for p in outline]
    bm.faces.new(top)
    bm.faces.new(bot[::-1])
    n = len(outline)
    for i in range(n):
        j = (i + 1) % n
        bm.faces.new((top[i], bot[i], bot[j], top[j]))
    bmesh.ops.recalc_face_normals(bm, faces=bm.faces[:])
    uv = bm.loops.layers.uv.new()
    for f in bm.faces:
        for l in f.loops:
            l[uv].uv = (0.0, 0.0)
    bm.to_mesh(me)
    bm.free()
    me.materials.append(mats[mat])
    return mesh_object(name, me, (0.0, 0.0, 0.0))


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


def sill(y):
    """The canopy sill's height at y."""
    for (y0, z0), (y1, z1) in zip(SILL_LINE, SILL_LINE[1:]):
        if y <= y1:
            return z0 + (z1 - z0) * (y - y0) / (y1 - y0)
    return SILL_LINE[-1][1]


# --- glare shield: from the panel's top edge forward, narrowing to the windscreen's sides,
# dropping towards the windscreen; a lip over the panel's face
THICK = 0.016
LIP = 0.010
edge = [(-WING, PANEL_TOP + BEVEL), (-(WING - BEVEL), PANEL_TOP), (WING - BEVEL, PANEL_TOP), (WING, PANEL_TOP + BEVEL)]
front = [point(u, v) + UP * THICK + NORMAL * LIP for u, v in edge]
top_z = front[1].z
back_z = top_z - COAMING_DROP
# Its outline seen from above: the panel's top edge, out to the walls' line, forward along the
# windscreen's sides (half width 0.45 at y 0.75, 0.40 at y 0.85) to the middle's back edge.
shield = [
    front[0], front[1], front[2], front[3],
    Vector((0.45, 0.75, top_z - 0.012)), Vector((0.40, 0.85, back_z + 0.01)),
    Vector((0.25, COAMING_BACK, back_z)), Vector((-0.25, COAMING_BACK, back_z)),
    Vector((-0.40, 0.85, back_z + 0.01)), Vector((-0.45, 0.75, top_z - 0.012)),
]
# Two convex halves (left, right of the middle) and the middle.
slab("black", "black", [front[1], front[2], shield[6], shield[7]], THICK)
for s in (-1, 1):
    i = (0, 1, 7, 8, 9) if s < 0 else (2, 3, 4, 5, 6)
    slab("black", "black", [shield[k] for k in i], THICK)
# A bulkhead under its back edge down to the floor.
box("dark", "dark", (0.0, COAMING_BACK + 0.01, (FLOOR + back_z) / 2), (2 * WALL_X, 0.02, back_z - FLOOR))
for s in (-1, 1):
    yz = [(0.70, sill(0.70)), (COAMING_BACK, sill(COAMING_BACK)), (COAMING_BACK, FLOOR), (0.70, FLOOR)]
    prism("dark", "dark", yz, *sorted((s * 0.40, s * WALL_X)))

# --- panel: one plate, the controller's body raised in its middle, rims round its edges
scr = layout["screens"]
faces = {f["key"]: f["rect"] for f in layout["faces"]}
plate("panel_grey", "panel_grey", [[
    (-(WING - BEVEL), PANEL_TOP), (WING - BEVEL, PANEL_TOP), (WING, PANEL_TOP + BEVEL),
    (WING, BOTTOM), (-WING, BOTTOM), (-WING, PANEL_TOP + BEVEL),
]], 0.03)
# The controller's body.
box("dark", "dark", PANEL_CENTRE + RIGHT * 0 - UP * ((PANEL_TOP + UFC_LINE) / 2000) + NORMAL * (UFC_RAISE / 2),
    (2 * UFC_HALF / 1000, UFC_RAISE, (UFC_LINE - PANEL_TOP) / 1000), rotation())
# Dark recesses out to the walls beside the panel, under the glare shield.
for s_ in (-1, 1):
    u0, u1 = sorted((s_ * WING, s_ * WALL_X * 1000))
    plane_box("dark", "dark", (u0, -60.0, u1 - u0, BOTTOM + 60.0), -0.04, -0.02)
panel_box("dark", "dark", (-WING, BOTTOM - 6.0, 2 * WING, 6.0), 0.007)
for s_ in (-1, 1):
    panel_box("dark", "dark", (s_ * (WING - 3.0) - 3.0, PANEL_TOP + BEVEL, 6.0, BOTTOM - 6.0 - PANEL_TOP - BEVEL), 0.007)

# --- tub: floor, consoles with fillers down to the panel's lower edge, walls up to the sills,
# knee wall with the footwell, centre pedestal
box("dark", "dark", (0.0, (SILL_REAR + COAMING_BACK) / 2, FLOOR - 0.01), (2 * WALL_X, COAMING_BACK - SILL_REAR, 0.02))
low = point(0, BOTTOM)
for s in (-1, 1):
    cx = s * sum(CONSOLE_X) / 2
    y0, y1 = CONSOLE_Y
    box("panel_grey", "panel_grey", (cx, (y0 + y1) / 2, (CONSOLE_TOP + FLOOR) / 2), (CONSOLE_X[1] - CONSOLE_X[0], y1 - y0, CONSOLE_TOP - FLOOR))
    box("dark", "dark", (cx, (y0 + y1) / 2, CONSOLE_TOP + 0.002), (CONSOLE_X[1] - CONSOLE_X[0] - 0.01, y1 - y0 - 0.02, 0.004))
    yz = [(y1, CONSOLE_TOP), (low.y, low.z), (low.y, FLOOR), (y1, FLOOR)]
    prism("dark", "dark", yz, *sorted(s * x for x in CONSOLE_X))
for s in (-1, 1):
    yz = [(SILL_REAR, FLOOR), (COAMING_BACK, FLOOR)] + [(y, z) for y, z in reversed(SILL_LINE)]
    prism("panel_grey", "panel_grey", yz, *sorted((s * WALL_X, s * (WALL_X + 0.02))))
    for (ya, za), (yb, zb) in zip(SILL_LINE, SILL_LINE[1:]):
        d = Vector((0.0, yb - ya, zb - za))
        rot = Euler((math.atan2(d.z, d.y), 0.0, 0.0))
        box("frame", "frame", (s * (WALL_X + 0.002), (ya + yb) / 2, (za + zb) / 2 - 0.012), (0.03, d.length + 0.02, 0.025), rot)
box("panel_grey", "panel_grey", (0.0, SILL_REAR - 0.01, (sill(SILL_REAR) + FLOOR) / 2), (2 * WALL_X + 0.04, 0.02, sill(SILL_REAR) - FLOOR))
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
box("dark", "dark", (0.0, (low.y + wall) / 2, low.z - 0.01), (2 * WALL_X, wall - low.y + 0.02, 0.02))
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

# --- HUD: its body on the glare shield, its front hanging over the controller; the combiner
# glass over it between two posts with a bar across the top
half_x = HUD_GLASS[0]
ufc_top = point(0.0, PANEL_TOP, 0.0)
prism("dark", "dark", [
    (HUD_Y - 0.004, HUD_BODY_LOW), (ufc_top.y + 0.02, top_z - 0.005), (HUD_BODY_BACK, back_z + 0.005),
    (HUD_BODY_BACK, HUD_BODY_TOP), (HUD_Y - 0.004, HUD_BODY_TOP),
], -half_x - 0.01, half_x + 0.01)

# --- plates, bezels, faces and displays
PLATE = 0.004
plates = [
    ("black", (-74.0, -148.0, 148.0, 134.0)),  # up-front controller's face
    ("black", (-100.0, -146.0, 24.0, 132.0)),  # left of it: the option window, the knobs
    ("black", (75.0, -146.0, 25.0, 132.0)),  # option display unit
    ("dark", (-78.0, -6.0, 156.0, 20.0)),  # caution and warning lights
    ("dark", (-342.0, -140.0, 64.0, 120.0)),  # master modes, master arm, launch flare
    ("black", (-418.0, 46.0, 52.0, 146.0)),  # gear lights, gear handle, brake
    ("dark", (-364.0, 46.0, 60.0, 92.0)),  # flaps
    ("black", (-288.0, 44.0, 184.0, 140.0)),  # armament
    ("black", (272.0, -148.0, 80.0, 128.0)),  # engine display
    ("dark", (352.0, -108.0, 64.0, 142.0)),  # RWR and ECM
    ("dark", (272.0, -4.0, 80.0, 50.0)),  # countermeasures
    ("black", (118.0, 48.0, 158.0, 80.0)),  # fuel
    ("dark", (280.0, 50.0, 136.0, 80.0)),  # caution lights
    ("black", (-100.0, 140.0, 200.0, 30.0)),  # target buttons
    ("black", (118.0, 140.0, 298.0, 90.0)),  # switches under the fuel and caution panels
    ("black", (-418.0, 196.0, 314.0, 40.0)),  # switches under the armament panel
]
for mat, r in plates:
    panel_box(mat, mat, r, PLATE)
# Up-front controller: the channel and volume knobs under the channel windows (two stacked),
# the brightness knob under the left option window.
for u in (-60.0, 60.0):
    knob(u, -116.0, 0.009, 0.008, "dark", offset=PLATE)
    knob(u, -116.0, 0.006, 0.014, offset=PLATE)
knob(-88.0, -70.0, 0.007, 0.010, offset=PLATE)
knob(-88.0, -40.0, 0.007, 0.010, "dark", offset=PLATE)
# The master arm switch on its stripes, the launch flare switch under its guard.
x, y, w, h = faces["Item(Hazard)"]
toggle(x + w / 2, y + h / 2, PLATE + 0.001)
mesh_object("yellow", box_mesh("yellow", (0.012, 0.010, 0.020), "yellow", (0.0, -0.005, 0.0)), point(-295.0, -42.0, PLATE), rotation())
toggle(-295.0, -42.0, PLATE + 0.010, up=False)
# The armament panel's switches and knobs under their legends, the jettison button and its
# selector knob, the master arm's own lights.
for k in range(5):
    u = -266.0 + 35.0 * k
    if k == 0:
        knob(u, 76.0, 0.008, 0.012, offset=PLATE)
    else:
        toggle(u, 76.0, PLATE, up=k % 2 == 0)
        panel_box("black", "black", (u - 10.0, 84.0, 20.0, 8.0), 0.002, 0.0, PLATE)
# The station select buttons 1-7 (blank caps), their lights under them.
for k in range(7):
    panel_box("dark", "dark", (-280.0 + 24.0 * k, 100.0, 20.0, 16.0), 0.006, 0.0, PLATE)
    panel_box("metal", "metal", (-277.0 + 24.0 * k, 101.5, 14.0, 4.0), 0.0005, 0.0, PLATE + 0.006)
    panel_box("dark", "dark", (-276.0 + 24.0 * k, 124.0, 12.0, 6.0), 0.002, 0.0, PLATE)
mesh_object("red", cylinder_mesh("red", 0.010, 0.010, "red", n=12), point(-120.0, 157.0, PLATE), rotation())
mesh_object("dark", cylinder_mesh("dark", 0.013, 0.004, "dark", n=12), point(-120.0, 157.0, PLATE), rotation())
knob(-230.0, 157.0, 0.012, 0.012, offset=PLATE)
knob(-200.0, 157.0, 0.007, 0.010, "dark", offset=PLATE)
# Flaps panel: the mode knob; under the gear lights the gear handle (kept) and the brake cap.
knob(-334.0, 126.0, 0.007, 0.012, offset=PLATE)
# Switch rows under the lower panels.
for k in range(12):
    toggle(-400.0 + 26.0 * k, 216.0, PLATE, up=k % 3 != 1)
for k in range(10):
    u = 134.0 + 30.0 * k
    if k % 4 == 3:
        knob(u, 185.0, 0.008, 0.012, offset=PLATE)
    else:
        toggle(u, 165.0, PLATE, up=k % 2 == 0)
        toggle(u, 205.0, PLATE, up=k % 2 == 1)
# Fuel: the selector and BINGO knobs.
knob(150.0, 104.0, 0.012, 0.012, offset=PLATE)
knob(212.0, 104.0, 0.007, 0.010, "dark", offset=PLATE)
knob(245.0, 104.0, 0.007, 0.010, "dark", offset=PLATE)
# The engine display's windows (black, unlit) right of their legends.
for k in range(4):
    panel_box("black", "black", (302.0, -71.0 + 14.0 * k, 40.0, 12.0), 0.001, 0.0, PLATE)
# MFD bezels: a square housing standing BEZEL off the panel round the screen (the canvas'
# shape in its upper part), the screen back in a black recess; twenty push buttons round it
# (layout faces) with ridges between them, rocker switches at the corners, the DAY / NIGHT /
# brightness knobs along the bottom.
BEZEL = 0.012
for key in ("left", "right"):
    x, y, w, h = scr[key]
    outer = (x - 20.0, y - 22.0, w + 40.0, w + 40.0)
    hole = (x - 3.0, y - 3.0, w + 6.0, h + 6.0)
    frame("dark", outer, hole, -0.002, BEZEL - 0.002)
    frame("black", (outer[0] + 1.5, outer[1] + 1.5, outer[2] - 3.0, outer[3] - 3.0), hole, BEZEL - 0.002, BEZEL)
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
    kv = y + h + 31.0
    knob(x + 20.0, kv, 0.008, 0.010, offset=BEZEL)
    knob(x + w / 2, kv, 0.006, 0.010, "dark", offset=BEZEL)
    knob(x + w - 20.0, kv, 0.008, 0.010, offset=BEZEL)


# Setting knobs at instrument corners: the airspeed indicator, the altimeter, the standby
# attitude indicator and the clock.
def corner(r, sx, sy, inset=3.0):
    x, y, w, h = r
    return (x + w - inset if sx > 0 else x + inset, y + h - inset if sy > 0 else y + inset)


for r, sx, sy in (
    (faces["Item(Dial(Airspeed))"], 1, 1),
    (faces["Item(Dial(Altimeter))"], -1, 1),
    (scr["adi"], 1, 1),
    (faces["Item(Dial(Clock))"], -1, 1),
):
    u, v = corner(r, sx, sy)
    knob(u, v, 0.0045, 0.009, "dark", offset=PLATE)
screen("screen_left", scr["left"], BEZEL - 0.006, "trackcam_left")
screen("screen_right", scr["right"], BEZEL - 0.006, "trackcam")
screen("adi", scr["adi"], 0.0, round=True)
screen("ded", scr["ded"], PLATE + 0.0005)
x, y, w, h = scr["ded"]
frame("black", (x - 3.0, y - 3.0, w + 6.0, h + 6.0), (x, y, w, h), UFC_RAISE, UFC_RAISE + PLATE + 0.004)
for f in layout["faces"]:
    if f["control"] is None:
        face(f["key"], f["rect"], PLATE + 0.0005, f["round"])
        if f["key"].startswith("Item(Dial(") and not f["round"]:
            x, y, w, h = f["rect"]
            frame("black", (x - 3.0, y - 3.0, w + 6.0, h + 6.0), (x, y, w, h), 0.0, PLATE + 0.005)
    elif f["kind"] == "osb":
        control(f["key"], f["rect"], BEZEL, f["control"], "osb")
    else:
        control(f["key"], f["rect"], PLATE, f["control"], f["kind"])

# --- side console panels: black plates on the consoles' tops round the throttle and the
# nozzle quadrant, switch panels on the right one (dressing; none of them is a control)
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
    side(0.23, 0.47, -0.29, -0.19, ["kKkk", "tgtt"])
    side(0.23, 0.47, -0.18, -0.08, ["ttttt", "kTkk"] if s > 0 else ["ttttt", "rgrgk"])
for rows, rect in (
    (["kkk", "TTk", "krk"], (-0.47, -0.23, -0.07, 0.01)),  # behind the throttle quadrant
    (["k", "t", "r", "k"], (-0.47, -0.40, 0.04, 0.30)),  # outboard of the throttle's rails
    (["kkkk", "tttt"], (-0.47, -0.23, 0.31, 0.41)),  # ahead of the quadrant
    # The right console: comms, lights, the environmental controls.
    (["kKk", "ttt", "rr"], (0.23, 0.47, -0.07, 0.07)),
    (["bbbbb", "bbbbb", "bbbbb", "bbbbb"], (0.23, 0.35, 0.08, 0.20)),
    (["kk", "tt", "kk"], (0.36, 0.47, 0.08, 0.20)),
    (["tkt", "KkK", "tttt", "kKk"], (0.23, 0.47, 0.21, 0.41)),
):
    console_panel(*rect, rows)
box("black", "black", (0.35, 0.255, CONSOLE_FACE + 0.007), (0.17, 0.06, 0.004))
for s, y in ((-1, 0.36), (1, 0.06)):
    box("yellow", "yellow", (s * (CONSOLE_X[0] - 0.022), y, CONSOLE_TOP - 0.07), (0.012, 0.012, 0.08))
    for dz in (-0.03, 0.03):
        box("yellow", "yellow", (s * (CONSOLE_X[0] - 0.011), y, CONSOLE_TOP - 0.07 + dz), (0.022, 0.01, 0.01))

# --- the nozzle lever's quadrant: a raised plate with the lever's slot along it, the STO and
# hover stops beside the slot, a scale of ticks every 10 degrees; the lever on its pivot under it
QH = 0.012  # the quadrant plate's height over the console
q_top = CONSOLE_FACE + QH
q_y = (NOZZLE_PIVOT.y - 0.09, NOZZLE_PIVOT.y + 0.11)
SLOT = 0.007  # half width
qx0, qx1 = NOZZLE_PIVOT.x - 0.03, NOZZLE_PIVOT.x + 0.025
box("dark", "dark", ((qx0 + qx1) / 2, sum(q_y) / 2, CONSOLE_FACE + QH / 2 - 0.004), (qx1 - qx0 + 0.006, q_y[1] - q_y[0] + 0.006, QH))
for xa, xb in ((qx0, NOZZLE_PIVOT.x - SLOT), (NOZZLE_PIVOT.x + SLOT, qx1)):
    box("black", "black", ((xa + xb) / 2, sum(q_y) / 2, q_top - 0.002), (xb - xa, q_y[1] - q_y[0], 0.004))


def lever_angle(t):
    """The lever's lean from upright (forward +) at lever t."""
    return NOZZLE_FWD + (NOZZLE_BACK - NOZZLE_FWD) * t


def slot_y(t):
    """Where the lever crosses the quadrant's top at lever t."""
    return NOZZLE_PIVOT.y + (q_top - NOZZLE_PIVOT.z) * math.tan(lever_angle(t))


for deg in range(0, 100, 10):
    y = slot_y(deg / NOZZLE_TRAVEL)
    w = 0.010 if deg % 30 == 0 else 0.006
    box("metal", "metal", (NOZZLE_PIVOT.x + SLOT + 0.002 + w / 2, y, q_top + 0.0004), (w, 0.0015, 0.001))
for deg, mat in NOZZLE_STOPS:
    y = slot_y(deg / NOZZLE_TRAVEL)
    # A block on the slot's inboard edge, its face where the lever stops.
    box(mat, mat, (NOZZLE_PIVOT.x - SLOT - 0.006, y - 0.004, q_top + 0.004), (0.012, 0.008, 0.008))
# The braking stop at the slot's back end, the forward stop at its front.
for t in (0.0, 1.0):
    y = slot_y(t) + (0.006 if t == 0.0 else -0.006)
    box("metal", "metal", (NOZZLE_PIVOT.x, y, q_top - 0.001), (2 * SLOT, 0.004, 0.004))
# The lever: its origin the knob at lever 0, the arm down to the pivot, a gaiter in the slot.
knob_at = lambda t: NOZZLE_PIVOT + NOZZLE_ARM * Vector((0.0, math.sin(lever_angle(t)), math.cos(lever_angle(t))))  # noqa: E731
aft = knob_at(0.0)
lever = empty("nozzle", aft, props={"nozzle": True})
mesh_object("dark", cylinder_mesh("dark", 0.013, 0.034, "dark", axis="X", centre=(-0.017, 0.0, 0.0), n=10), (0, 0, 0), parent=lever)
mesh_object("metal", cylinder_mesh("metal", 0.0135, 0.004, "metal", axis="X", centre=(0.017, 0.0, 0.0), n=10), (0, 0, 0), parent=lever)
rod("metal", (0.0, 0.0, 0.0), NOZZLE_PIVOT - aft, 0.0055, "metal", lever)
empty("nozzle_down", knob_at(1.0), props={"anchor": "nozzle_down"})
empty("nozzle_pivot", NOZZLE_PIVOT, props={"anchor": "nozzle_pivot"})

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
# Throttle: outboard by THROTTLE_DX, the kept handle's travel lengthened to THROTTLE_TRAVEL,
# its rails along it.
throttle = objs["throttle"]
full = objs["throttle_full"]
throttle.location.x += THROTTLE_DX
full.location = throttle.location + Vector((0.0, THROTTLE_TRAVEL, 0.0))
y0, y1 = throttle.location.y - 0.03, full.location.y + 0.03
for n in RAILS:
    o = objs[n]
    o.data = o.data.copy()
    vs = [v.co.y for v in o.data.vertices]
    k = (y1 - y0) / (max(vs) - min(vs))
    o.data.transform(Matrix.Diagonal((1.0, k, 1.0, 1.0)))
    o.location.y = (y0 + y1) / 2
    o.location.x += THROTTLE_DX
# A black plate under the rails, the quadrant's outboard half.
box("black", "black", (throttle.location.x, (y0 + y1) / 2, CONSOLE_FACE + 0.001), (0.07, y1 - y0 + 0.02, 0.002))
# Target buttons in a row under the standby instruments; the gear handle under the gear lights.
for i, n in enumerate(["button_NEAR", "button_NEXT", "button_NONE", "button_WPN", "button_DTL"]):
    o = objs[n]
    u, v = -64.0 + 32.0 * i, 155.0
    o.location = point(u, v, PLATE)
    o.rotation_mode = "QUATERNION"
    o.rotation_quaternion = rotation(u, v)
g = objs["control_Action_Gear_.001"]
g.location = point(-392.0, 120.0, PLATE)
g.rotation_mode = "QUATERNION"
g.rotation_quaternion = rotation(-392.0, 120.0)
# HUD glass: the combiner over the body (its vertices are absolute), a frame of bars along its
# outline (posts down to the body, chamfered top corners, top), outside it by half a bar.
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
outline = [
    Vector((-post_x, HUD_BODY_TOP)), Vector((-post_x, shoulder)),
    Vector((-top_x, top + HUD_BAR / 2)), Vector((top_x, top + HUD_BAR / 2)),
    Vector((post_x, shoulder)), Vector((post_x, HUD_BODY_TOP)),
]
for a, b in zip(outline, outline[1:]):
    d = b - a
    c = (a + b) / 2
    rot = Euler((0.0, -math.atan2(d.y, d.x), 0.0))
    box("frame", "frame", (c.x, HUD_Y, c.y), (d.length + HUD_BAR, HUD_BAR, HUD_BAR), rot)
# The approach indexer just outside the left post, in a housing on the body.
ind = next(o for o in old if o.name.startswith("approach"))
ind.location = Vector((-(post_x + HUD_BAR / 2 + 0.016), HUD_Y, -0.12))
box("dark", "dark", (ind.location.x, HUD_Y + 0.007, ind.location.z), (0.028, 0.010, 0.068))
box("dark", "dark", (ind.location.x, HUD_Y + 0.007, (ind.location.z - 0.034 + HUD_BODY_LOW) / 2), (0.010, 0.010, ind.location.z - 0.034 - HUD_BODY_LOW))
# Flood lamps: two behind the pilot's head, one under the HUD's hood.
objs["lamp.002"].location = Vector((0.0, 0.62, -0.16))

bpy.data.orphans_purge(do_recursive=True)
bpy.context.view_layer.update()
print(f"stick grip {stick['grip']:.3f} m; throttle {throttle.location} -> {full.location}; "
      f"nozzle knob {aft} -> {knob_at(1.0)} about {NOZZLE_PIVOT}; "
      f"glare shield top {top_z:.3f} .. {back_z:.3f}; seat bottom {bottom:.3f}; panel bottom {low}")
bpy.ops.wm.save_as_mainfile(filepath=out)
print("built", out, "objects", len(bpy.data.objects))
