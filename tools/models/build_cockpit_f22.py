# Builds the F-22A Raptor cockpit master (assets-src/blend/cockpit_f22.blend) from the panel
# layout, after the F-22A's glass cockpit, with parts of the F-117A's modern cockpit:
#   blender -b assets-src/blend/cockpit_modern.blend --python-exit-code 1 \
#       --python tools/models/build_cockpit_f22.py -- \
#       assets-src/blend/cockpit_f22_layout.json assets-src/blend/cockpit_f22.blend
# then tools/models/export.sh cockpit_f22. The layout JSON is written by
# `UPDATE_LAYOUT=1 cargo test layout_files` from `PanelLayout::f22` (src/cockpit3d/layout.rs).
#
# Life size, following tools/models/build_cockpit_av8b.py (the F-117A's kept parts scaled down
# by F117_SCALE about the eye). Kept, with their materials, labels and control properties: the
# seat with the ejection handle, the gear handle and the five target buttons, the HUD glass, the
# approach indexer and the flood lamps. Built here, low-poly: the flat main panel with the
# up-front controller (the integrated control panel between the up-front display and the
# square standby flight display) raised along its top under the HUD; the large centre MFD with a
# smaller one either side and the engine display under it in square bezels; the low glare shield with the HUD body and its wide
# combiner; the consoles with their switch panels, the side stick (as the F-16's) on the right
# one with its arm rest, the two throttles on the left one; the knee wall with the footwell, the
# walls up to the canopy sills with cheeks from them up to the glare shield's sides.
#
# It fits the exterior's one-piece bubble (`F22_CANOPY` in tools/models/aircraft.py, the eye at
# s 4.35, z 0.92): the walls follow the glass' sill line (`canopy`) a little inside and under
# it; the glare shield stays inside the glass and under the line over the nose (15.4 degrees
# down: 0.62 m under the eye 2.25 m ahead), the HUD body under it, the combiner over it.
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
# The exterior's canopy (tools/models/aircraft.py `F22_CANOPY`: s aft of the nose, the sill's z,
# the half width at the sill, the top's z) and the eye in its frame.
F22_CANOPY = [(2.60, 0.42, 0.03, 0.45), (2.80, 0.44, 0.22, 0.60), (3.00, 0.46, 0.34, 0.70), (3.30, 0.50, 0.44, 0.85),
              (3.60, 0.54, 0.47, 0.96), (4.00, 0.59, 0.49, 1.15), (4.40, 0.64, 0.50, 1.22), (4.80, 0.72, 0.49, 1.17),
              (5.20, 0.80, 0.44, 1.08), (5.45, 0.92, 0.12, 1.01)]
EYE_S, EYE_Z = 4.35, 0.92


def canopy(y):
    """The glass at y in front of the eye: (the sill's z, the half width at the sill, the top's z)."""
    s = EYE_S - y
    for a, b in zip(F22_CANOPY, F22_CANOPY[1:]):
        if a[0] <= s <= b[0]:
            t = (s - a[0]) / (b[0] - a[0])
            sill, hw, top = (a[k] + (b[k] - a[k]) * t for k in (1, 2, 3))
            return sill - EYE_Z, hw, top - EYE_Z
    raise ValueError(y)


# The main panel: 20 degrees off upright, its top edge (layout y PANEL_TOP) 0.30 m under the eye
# 0.80 m ahead.
PANEL_TILT = math.radians(20.0)
PANEL_TOP = -180.0
RIGHT = Vector((1.0, 0.0, 0.0))
UP = Vector((0.0, math.sin(PANEL_TILT), math.cos(PANEL_TILT)))
NORMAL = Vector((0.0, -math.cos(PANEL_TILT), math.sin(PANEL_TILT)))
TOP_EDGE = Vector((0.0, 0.80, -0.30))
PANEL_CENTRE = TOP_EDGE + UP * (PANEL_TOP / 1000)
FLOOR = -1.0
# The walls: their inner faces WALL_IN inside the glass' sill (at most WALL_X out), their tops
# WALL_UNDER under it, from the rear bulkhead (y REAR) to the glare shield's back edge.
WALL_X = 0.46
WALL_IN = 0.02
WALL_UNDER = 0.006
WALL_T = 0.012
REAR = -0.62
WALL_Y = [REAR, -0.45, -0.25, 0.0, 0.35, 0.60, 0.75, 0.92, 1.06]
# Consoles (the throttles on the left one, the stick on the right one): x, y and their top.
CONSOLE_X0 = 0.21
CONSOLE_Y = (-0.35, 0.60)
CONSOLE_TOP = -0.64
# The panel's outline (layout mm): out to WING, its top corners bevelled down BEVEL (45
# degrees), down to BOTTOM.
WING = 330.0
BEVEL = 40.0
BOTTOM = 270.0
# The up-front controller (|x| <= UFC_HALF, y < UFC_LINE) standing UFC_RAISE m off the panel.
UFC_HALF = 172.0
UFC_LINE = -80.0
UFC_RAISE = 0.02
# The glare shield: from the panel's top forward to COAMING_BACK in the middle (narrowing to
# the windscreen's sides), dropping COAMING_DROP; its sides' outline (half width, y).
COAMING_BACK = 1.06
COAMING_DROP = 0.04
SHIELD_SIDE = [(0.40, 0.79), (0.36, 0.93), (0.26, COAMING_BACK)]
# HUD combiner glass: half width, bottom and top (m), at HUD_Y in front of the eye
# (AircraftSpec::hud_scale fits the symbology into it); its body under it from HUD_BODY_LOW over
# the controller up to HUD_BODY_TOP, sloping down to HUD_BODY_BACK_TOP at HUD_BODY_BACK (under
# the line over the nose).
HUD_Y = 0.66
HUD_GLASS = (0.16, -0.19, 0.03)
HUD_BAR = 0.012
HUD_BODY_TOP = -0.19
HUD_BODY_LOW = -0.235
HUD_BODY_BACK = 0.90
HUD_BODY_BACK_TOP = -0.26
# Seat: its cushion's top, the eye over the hips.
SEAT_TOP = -0.80
# Side stick on the right console, the throttles' idle and full along the left one.
STICK = Vector((0.30, 0.20, CONSOLE_TOP))
THROTTLE = Vector((-0.29, 0.10, CONSOLE_TOP + 0.085))
THROTTLE_TRAVEL = 0.20
# Segments of the round parts (bezels, buttons, knobs): low-poly.
ROUND = 16
KNOB = 8


def wall_x(y):
    """The walls' inner face at y."""
    return min(WALL_X, canopy(y)[1] - WALL_IN - WALL_T)


def sill(y):
    """The walls' top at y."""
    return canopy(y)[0] - WALL_UNDER


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
    "seat_", "cushion_", "metal_98", "metal_99", "control_Action_Eject_", "hud_glass", "approach", "lamp",
    "control_Action_Gear_.001", "button_NEAR", "button_NEXT", "button_NONE", "button_WPN", "button_DTL",
)
SEAT = ("seat_", "cushion_", "metal_98", "metal_99", "control_Action_Eject_")


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


for n, like in (
    ("ded", "adi"), ("screen_third", "screen_right"), ("screen_lmpd", "screen_right"),
    ("screen_rmpd", "screen_right"), ("trackcam_left", "trackcam"), ("trackcam_third", "trackcam"),
):
    material(n, like)
# Guard covers and red buttons (not a shared material: the game takes its colour from the model).
red = material("red", "yellow")
red.diffuse_color = (0.55, 0.04, 0.03, 1.0)
for nd in red.node_tree.nodes if red.use_nodes else ():
    if nd.type == "BSDF_PRINCIPLED":
        nd.inputs["Base Color"].default_value = red.diffuse_color


# --- building blocks (as the AV-8B's)
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


def finish(bm, me, mat):
    uv = bm.loops.layers.uv.new()
    for f in bm.faces:
        for l in f.loops:
            l[uv].uv = (0.0, 0.0)
    bm.to_mesh(me)
    bm.free()
    me.materials.append(mats[mat])
    return me


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
        "-X": Matrix.Rotation(math.radians(-90.0), 4, "Y"),
    }[axis]
    m = Matrix.Translation(centre) @ rot @ Matrix.Translation((0.0, 0.0, h / 2))
    bmesh.ops.create_cone(bm, cap_ends=True, cap_tris=False, segments=n, radius1=r, radius2=r, depth=h, matrix=m)
    return finish(bm, me, mat)


def loft_mesh(name, rings, mat, n=KNOB):
    """A closed tube through rings (centre, a, b): each ring the points centre + a cos t + b sin t
    (the grips of the stick and the throttles)."""
    me = bpy.data.meshes.new(name)
    bm = bmesh.new()
    loops = []
    for c, a, b in rings:
        loops.append([bm.verts.new(Vector(c) + Vector(a) * math.cos(2 * math.pi * i / n) + Vector(b) * math.sin(2 * math.pi * i / n)) for i in range(n)])
    for l0, l1 in zip(loops, loops[1:]):
        for i in range(n):
            j = (i + 1) % n
            bm.faces.new((l0[i], l0[j], l1[j], l1[i]))
    bm.faces.new(loops[0][::-1])
    bm.faces.new(loops[-1])
    bmesh.ops.recalc_face_normals(bm, faces=bm.faces[:])
    return finish(bm, me, mat)


def section_mesh(name, sections, mat):
    """A closed solid lofted through convex sections (lists of points, the same count each)."""
    me = bpy.data.meshes.new(name)
    bm = bmesh.new()
    loops = [[bm.verts.new(p) for p in s] for s in sections]
    n = len(sections[0])
    for l0, l1 in zip(loops, loops[1:]):
        for i in range(n):
            j = (i + 1) % n
            bm.faces.new((l0[i], l0[j], l1[j], l1[i]))
    bm.faces.new(loops[0][::-1])
    bm.faces.new(loops[-1])
    bmesh.ops.recalc_face_normals(bm, faces=bm.faces[:])
    return finish(bm, me, mat)


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


def screen(mat, r, offset, behind=None):
    """A display over the rectangle r: a quad, a dark one (`behind`) just behind it."""
    rot = rect_rotation(r)
    x, y, w, h = r
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
    return mesh_object(name, section_mesh(name, [[(x, y, z) for y, z in yz] for x in (x0, x1)], mat), (0, 0, 0))


def slab(name, mat, outline, thickness):
    """A convex outline of (x, y, z) points (m), `thickness` m thick downwards."""
    top = [Vector(p) for p in outline]
    return mesh_object(name, section_mesh(name, [top, [p - Vector((0.0, 0.0, thickness)) for p in top]], mat), (0.0, 0.0, 0.0))


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


# --- glare shield: from the panel's top edge forward, narrowing along the windscreen's sides,
# dropping towards the windscreen; a lip over the panel's face
THICK = 0.016
LIP = 0.010
edge = [(-WING, PANEL_TOP + BEVEL), (-(WING - BEVEL), PANEL_TOP), (WING - BEVEL, PANEL_TOP), (WING, PANEL_TOP + BEVEL)]
front = [point(u, v) + UP * THICK + NORMAL * LIP for u, v in edge]
# The controller's raised strip: the lip comes out over it.
top_z = front[1].z
back_z = top_z - COAMING_DROP


def shield_z(y):
    return top_z + (back_z - top_z) * max(0.0, (y - front[1].y) / (COAMING_BACK - front[1].y))


side = [Vector((x, y, shield_z(y))) for x, y in SHIELD_SIDE]
shield = [front[0], front[1], front[2], front[3]] + side + [Vector((-p.x, p.y, p.z)) for p in reversed(side)]
# The middle and two convex sides.
slab("black", "black", [front[1], front[2], shield[6], shield[7]], THICK)
for s in (-1, 1):
    i = (0, 1, 7, 8, 9) if s < 0 else (2, 3, 4, 5, 6)
    slab("black", "black", [shield[k] for k in i], THICK)
# A bulkhead under its back edge down to the floor, as wide as the glass allows: the walls'
# width up to their tops, the glare shield's above.
wx = wall_x(COAMING_BACK)
box("dark", "dark", (0.0, COAMING_BACK + 0.01, (FLOOR + sill(COAMING_BACK)) / 2), (2 * wx, 0.02, sill(COAMING_BACK) - FLOOR))
box("dark", "dark", (0.0, COAMING_BACK + 0.01, (sill(COAMING_BACK) + back_z) / 2), (2 * SHIELD_SIDE[-1][0], 0.02, back_z - sill(COAMING_BACK)))
# Cheeks from the glare shield's sides down to the walls' tops.
for s in (-1, 1):
    ys = [p.y for p in side]
    for (a, b) in zip(range(len(ys)), range(1, len(ys))):
        pa, pb = side[a], side[b]
        qa = Vector((wall_x(pa.y) + WALL_T, pa.y, sill(pa.y)))
        qb = Vector((wall_x(pb.y) + WALL_T, pb.y, sill(pb.y)))
        quad = [pa - Vector((0, 0, THICK)), pb - Vector((0, 0, THICK)), qb, qa]
        quad = [Vector((s * p.x, p.y, p.z)) for p in quad]
        mesh_object("dark", section_mesh("dark", [quad, [p - Vector((0, 0, 0.01)) for p in quad]], "dark"), (0, 0, 0))
    # From the panel's top corner out to the first side point and down to the wall.
    pa = Vector((s * front[3].x, front[3].y, front[3].z - THICK)) if s > 0 else front[0] - Vector((0, 0, THICK))
    pb = Vector((s * side[0].x, side[0].y, side[0].z - THICK))
    y0 = pa.y - 0.02
    qa = Vector((s * (wall_x(y0) + WALL_T), y0, sill(y0)))
    qb = Vector((s * (wall_x(pb.y) + WALL_T), pb.y, sill(pb.y)))
    quad = [pa, pb, qb, qa]
    mesh_object("dark", section_mesh("dark", [quad, [p - Vector((0, 0, 0.01)) for p in quad]], "dark"), (0, 0, 0))

# --- panel: one plate, the controller's body raised along its top, rims round its edges
scr = layout["screens"]
faces = {f["key"]: f["rect"] for f in layout["faces"]}
plate("panel_grey", "panel_grey", [[
    (-(WING - BEVEL), PANEL_TOP), (WING - BEVEL, PANEL_TOP), (WING, PANEL_TOP + BEVEL),
    (WING, BOTTOM), (-WING, BOTTOM), (-WING, PANEL_TOP + BEVEL),
]], 0.03)
box("dark", "dark", PANEL_CENTRE - UP * ((PANEL_TOP + UFC_LINE) / 2000) + NORMAL * (UFC_RAISE / 2),
    (2 * UFC_HALF / 1000, UFC_RAISE, (UFC_LINE - PANEL_TOP) / 1000), rotation())
# Dark recesses out to the walls beside the panel, under the glare shield's cheeks (their top
# under the canopy's sills).
RECESS_TOP = -95.0
for s_ in (-1, 1):
    u0, u1 = sorted((s_ * WING, s_ * (wall_x(0.7) + WALL_T) * 1000))
    plane_box("dark", "dark", (u0, RECESS_TOP, u1 - u0, BOTTOM - RECESS_TOP), -0.04, -0.02)
panel_box("dark", "dark", (-WING, BOTTOM - 6.0, 2 * WING, 6.0), 0.007)
for s_ in (-1, 1):
    panel_box("dark", "dark", (s_ * (WING - 3.0) - 3.0, PANEL_TOP + BEVEL, 6.0, BOTTOM - 6.0 - PANEL_TOP - BEVEL), 0.007)

# --- tub: floor, consoles with fillers down to the panel's lower edge, walls up to the sills,
# rear bulkhead, knee wall with the footwell
box("dark", "dark", (0.0, (REAR + COAMING_BACK) / 2, FLOOR - 0.01), (2 * WALL_X, COAMING_BACK - REAR, 0.02))
low = point(0, BOTTOM)
CONSOLE_FACE = CONSOLE_TOP + 0.004
for s in (-1, 1):
    y0, y1 = CONSOLE_Y
    x1 = min(wall_x(y) for y in (y0, y1, 0.0)) + 0.002
    cx = s * (CONSOLE_X0 + x1) / 2
    box("panel_grey", "panel_grey", (cx, (y0 + y1) / 2, (CONSOLE_TOP + FLOOR) / 2), (x1 - CONSOLE_X0, y1 - y0, CONSOLE_TOP - FLOOR))
    box("dark", "dark", (cx, (y0 + y1) / 2, CONSOLE_TOP + 0.002), (x1 - CONSOLE_X0 - 0.01, y1 - y0 - 0.02, 0.004))
    yz = [(y1, CONSOLE_TOP), (low.y + 0.03, CONSOLE_TOP - 0.06), (low.y + 0.03, FLOOR), (y1, FLOOR)]
    prism("dark", "dark", yz, *sorted((s * CONSOLE_X0, s * x1)))
# Walls: lofted along WALL_Y, the inner face at wall_x up to the sill, a frame rail along the top.
for s in (-1, 1):
    secs, rails = [], []
    for y in WALL_Y:
        x, z = wall_x(y), sill(y)
        secs.append([(s * x, y, FLOOR), (s * (x + WALL_T), y, FLOOR), (s * (x + WALL_T), y, z - 0.02), (s * x, y, z - 0.02)])
        rails.append([(s * (x - 0.012), y, z - 0.024), (s * (x + WALL_T), y, z - 0.024), (s * (x + WALL_T), y, z), (s * (x - 0.004), y, z)])
    mesh_object("panel_grey", section_mesh("panel_grey", secs, "panel_grey"), (0, 0, 0))
    mesh_object("frame", section_mesh("frame", rails, "frame"), (0, 0, 0))
xr = wall_x(REAR) + WALL_T
box("panel_grey", "panel_grey", (0.0, REAR - 0.01, (sill(REAR) + FLOOR) / 2), (2 * xr, 0.02, sill(REAR) - FLOOR))
LEGS = (0.20, -0.78)  # the knee wall's opening: half width, top (m)
wall = low.y + 0.06
for s_ in (-1, 1):
    box("panel_grey", "panel_grey", (s_ * (LEGS[0] + WALL_X) / 2, wall, (low.z + FLOOR) / 2), (WALL_X - LEGS[0], 0.02, low.z - FLOOR))
box("panel_grey", "panel_grey", (0.0, wall, (low.z + LEGS[1]) / 2), (2 * LEGS[0], 0.02, low.z - LEGS[1]))
WELL_DEPTH = 0.20
yc = wall + WELL_DEPTH / 2
for s_ in (-1, 1):
    box("dark", "dark", (s_ * (LEGS[0] + 0.01), yc, (LEGS[1] + FLOOR) / 2), (0.02, WELL_DEPTH, LEGS[1] - FLOOR))
box("dark", "dark", (0.0, yc, LEGS[1] + 0.01), (2 * LEGS[0] + 0.04, WELL_DEPTH, 0.02))
box("dark", "dark", (0.0, wall + WELL_DEPTH, (LEGS[1] + FLOOR) / 2), (2 * LEGS[0] + 0.04, 0.02, LEGS[1] - FLOOR))
box("dark", "dark", (0.0, (low.y + wall) / 2, low.z - 0.01), (2 * WALL_X, wall - low.y + 0.02, 0.02))
# Rudder pedals in the footwell.
for s_ in (-1, 1):
    box("metal", "metal", (s_ * 0.11, wall + 0.08, FLOOR + 0.12), (0.08, 0.02, 0.16), Euler((math.radians(-20.0), 0, 0)))
    box("dark", "dark", (s_ * 0.11, wall + 0.12, FLOOR + 0.06), (0.03, 0.08, 0.10))

# --- HUD: its body on the glare shield, its front hanging over the controller; the combiner
# glass over it between two posts with a bar across the top
half_x = HUD_GLASS[0]
ufc_top = point(0.0, PANEL_TOP, 0.0)
prism("dark", "dark", [
    (HUD_Y - 0.004, HUD_BODY_LOW), (ufc_top.y - 0.01, top_z - 0.01), (HUD_BODY_BACK, shield_z(HUD_BODY_BACK) + 0.002),
    (HUD_BODY_BACK, HUD_BODY_BACK_TOP), (HUD_Y - 0.004, HUD_BODY_TOP),
], -half_x - 0.01, half_x + 0.01)

# --- plates, bezels, faces and displays
PLATE = 0.004
plates = [
    ("black", (-76.0, -178.0, 152.0, 96.0)),  # integrated control panel
    ("dark", (-288.0, -178.0, 106.0, 100.0)),  # master caution, threat lights, master arm, bay
    ("dark", (178.0, -178.0, 116.0, 76.0)),  # warning and caution lights
    ("black", (-324.0, 118.0, 210.0, 126.0)),  # gear, flaps, brake, standby instruments
    ("black", (118.0, 118.0, 176.0, 116.0)),  # fuel, EMV, countermeasures
    ("black", (118.0, 238.0, 176.0, 28.0)),  # target buttons
]
for mat, r in plates:
    panel_box(mat, mat, r, PLATE)
# The controller: rockers and knobs either side of the scratchpad.
for u in (-62.0, 62.0):
    knob(u, -165.0, 0.006, 0.010, "dark", offset=PLATE)
    knob(u, -143.0, 0.006, 0.010, offset=PLATE)
# The master arm switch on its stripes.
x, y, w, h = faces["Item(Hazard)"]
toggle(x + w / 2, y + h / 2, PLATE + 0.001)
# MFD bezels: a housing standing BEZEL off the panel round the screen, the screen back in a
# black recess; twenty push buttons round it (layout faces) with ridges between them, rocker
# switches at the corners, brightness knobs along the bottom.
BEZEL = 0.012
for key in ("left", "right", "third"):
    x, y, w, h = scr[key]
    outer = (x - 16.0, y - 20.0, w + 32.0, h + 50.0)
    hole = (x - 3.0, y - 3.0, w + 6.0, h + 6.0)
    frame("dark", outer, hole, -0.002, BEZEL - 0.002)
    frame("black", (outer[0] + 1.5, outer[1] + 1.5, outer[2] - 3.0, outer[3] - 3.0), hole, BEZEL - 0.002, BEZEL)
    frame("black", hole, (x - 0.5, y - 0.5, w + 1.0, h + 1.0), -0.006, BEZEL - 0.001)
    for k in range(1, 5):
        for by in (y - 6.5, y + h + 6.5):
            panel_box("black", "black", (x + w * k / 5 - 1.0, by - 4.0, 2.0, 8.0), 0.003, 0.0, BEZEL)
    for k in range(1, 5):
        for bx in (x - 9.0, x + w + 9.0):
            panel_box("black", "black", (bx - 4.0, y + h * k / 5 - 1.0, 8.0, 2.0), 0.003, 0.0, BEZEL)
    for bx in (x - 9.0, x + w + 9.0):
        for by in (y - 9.0, y + h + 9.0):
            panel_box("dark", "dark", (bx - 4.0, by - 5.0, 8.0, 10.0), 0.003, 0.0, BEZEL)
            panel_box("black", "black", (bx - 1.5, by - 4.0, 3.0, 8.0), 0.003, 0.0, BEZEL + 0.003)
    kv = y + h + 21.0
    knob(x + 18.0, kv, 0.005, 0.008, offset=BEZEL)
    knob(x + w / 2, kv, 0.004, 0.008, "dark", offset=BEZEL)
    knob(x + w - 18.0, kv, 0.005, 0.008, offset=BEZEL)
# The up-front display, the standby flight display and the engine display: plain bezels, the
# screens recessed.
for key, side_ in (("lmpd", 5.0), ("adi", 5.0), ("rmpd", 8.0)):
    x, y, w, h = scr[key]
    outer = (x - side_, y - side_, w + 2 * side_, h + 2 * side_)
    hole = (x - 1.5, y - 1.5, w + 3.0, h + 3.0)
    base = UFC_RAISE if in_ufc(*centre(scr[key])) else 0.0
    frame("dark", outer, hole, base, base + 0.008)
    frame("black", hole, (x - 0.5, y - 0.5, w + 1.0, h + 1.0), base - 0.004, base + 0.007)
screen("screen_left", scr["left"], BEZEL - 0.006, "trackcam_left")
screen("screen_right", scr["right"], BEZEL - 0.006, "trackcam")
screen("screen_third", scr["third"], BEZEL - 0.006, "trackcam_third")
screen("screen_lmpd", scr["lmpd"], 0.004, "black")
screen("screen_rmpd", scr["rmpd"], 0.004, "black")
screen("adi", scr["adi"], 0.004, "black")
screen("ded", scr["ded"], PLATE + 0.0005)
x, y, w, h = scr["ded"]
frame("black", (x - 3.0, y - 3.0, w + 6.0, h + 6.0), (x, y, w, h), UFC_RAISE, UFC_RAISE + PLATE + 0.004)
for f in layout["faces"]:
    if f["control"] is None:
        face(f["key"], f["rect"], PLATE + 0.0005, f["round"])
        if f["key"].startswith("Item(Dial(") and not f["round"]:
            x, y, w, h = f["rect"]
            frame("black", (x - 2.0, y - 2.0, w + 4.0, h + 4.0), (x, y, w, h), 0.0, PLATE + 0.004)
    elif f["kind"] == "osb":
        control(f["key"], f["rect"], BEZEL, f["control"], "osb")
    else:
        control(f["key"], f["rect"], PLATE, f["control"], f["kind"])


# --- side console panels: black plates on the consoles' tops round the throttles and the
# stick, switch panels (dressing; none of them is a control)
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


for rows, rect in (
    # The left console: behind the throttles, outboard of them, ahead of them.
    (["kKkk", "tgtt"], (-0.44, -0.22, -0.33, -0.22)),
    (["ttttt", "rgrgk"], (-0.44, -0.22, -0.21, -0.10)),
    (["kkk", "TTk", "krk"], (-0.44, -0.22, -0.09, 0.03)),
    (["k", "t", "r", "k"], (-0.44, -0.37, 0.05, 0.38)),
    (["kkkk", "tttt", "kTkk"], (-0.44, -0.22, 0.42, 0.58)),
    # The right console: behind the arm rest, ahead of the stick.
    (["kKkk", "tgtt"], (0.22, 0.44, -0.33, -0.22)),
    (["ttttt", "kTkk"], (0.22, 0.44, -0.21, -0.14)),
    (["kKk", "ttt", "rr"], (0.22, 0.44, 0.30, 0.40)),
    (["bbbbb", "bbbbb", "bbbbb"], (0.22, 0.34, 0.42, 0.58)),
    (["kk", "tt", "kk"], (0.35, 0.44, 0.42, 0.58)),
):
    console_panel(*rect, rows)
# Yellow handles down the consoles' inboard sides.
for s, y in ((-1, 0.45), (1, 0.45)):
    box("yellow", "yellow", (s * (CONSOLE_X0 - 0.022), y, CONSOLE_TOP - 0.07), (0.012, 0.012, 0.08))
    for dz in (-0.03, 0.03):
        box("yellow", "yellow", (s * (CONSOLE_X0 - 0.011), y, CONSOLE_TOP - 0.07 + dz), (0.022, 0.01, 0.01))

# --- side stick (right console), as the F-16's: a base plate and a bellows boot on the console,
# the pivot above it; a contoured grip leaning forward with the trigger and its guard in front,
# the weapon release (red), trim hat and switches on top
box("dark", "dark", STICK + Vector((0.0, 0.0, 0.006)), (0.10, 0.10, 0.004))
X, Y, Z = Vector((1.0, 0.0, 0.0)), Vector((0.0, 1.0, 0.0)), Vector((0.0, 0.0, 1.0))
boot = [(0.008, 0.042), (0.014, 0.036), (0.019, 0.037), (0.024, 0.029), (0.028, 0.030), (0.033, 0.020), (0.036, 0.014)]
mesh_object("rubber", loft_mesh("rubber", [(STICK + Z * z, X * r, Y * r) for z, r in boot], "rubber", n=12), (0, 0, 0))
stick = empty("stick", STICK + Vector((0.0, 0.0, 0.025)), props={"stick": True})
mesh_object("metal", cylinder_mesh("metal", 0.011, 0.05, "metal", axis="Z"), (0, 0, 0), parent=stick)


def spine(z):
    """The grip's centre line at height z above the pivot, leaning forward."""
    return Vector((0.0, 0.006 + (z - 0.045) * 0.25, z))


grip = [(0.042, 0.014, 0.016), (0.06, 0.018, 0.022), (0.085, 0.021, 0.026), (0.115, 0.020, 0.025), (0.14, 0.022, 0.028), (0.162, 0.024, 0.031), (0.178, 0.021, 0.027), (0.186, 0.014, 0.018)]
mesh_object("rubber", loft_mesh("rubber", [(spine(z), X * rx, Y * ry) for z, rx, ry in grip], "rubber", n=10), (0, 0, 0), parent=stick)
lean = Euler((math.radians(-14.0), 0.0, 0.0)).to_quaternion()
front_of = lambda z, d=0.0: spine(z) + Y * (0.026 + d)  # noqa: E731
mesh_object("metal", box_mesh("metal", (0.010, 0.010, 0.026), "metal", front_of(0.108, 0.004), lean), (0, 0, 0), parent=stick)
mesh_object("metal", box_mesh("metal", (0.006, 0.005, 0.052), "metal", front_of(0.105, 0.020), lean), (0, 0, 0), parent=stick)
for z in (0.081, 0.13):
    mesh_object("metal", box_mesh("metal", (0.006, 0.022, 0.005), "metal", front_of(z, 0.008)), (0, 0, 0), parent=stick)
mesh_object("dark", box_mesh("dark", (0.012, 0.010, 0.022), "dark", spine(0.07) - Y * 0.026), (0, 0, 0), parent=stick)
head = spine(0.184)
mesh_object("red", cylinder_mesh("red", 0.0055, 0.008, "red", axis="Z", centre=head + Vector((-0.009, 0.014, -0.004))), (0, 0, 0), parent=stick)
mesh_object("dark", cylinder_mesh("dark", 0.0055, 0.007, "dark", axis="Z", centre=head + Vector((0.007, -0.002, -0.002))), (0, 0, 0), parent=stick)
mesh_object("metal", box_mesh("metal", (0.009, 0.003, 0.003), "metal", head + Vector((0.007, -0.002, 0.006))), (0, 0, 0), parent=stick)
mesh_object("metal", box_mesh("metal", (0.003, 0.009, 0.003), "metal", head + Vector((0.007, -0.002, 0.006))), (0, 0, 0), parent=stick)
for z, r in ((0.162, 0.0045), (0.142, 0.004)):
    mesh_object("dark", cylinder_mesh("dark", r, 0.006, "dark", axis="-X", centre=spine(z) - X * 0.021), (0, 0, 0), parent=stick)
# Arm rest behind the stick.
box("cushion", "cushion", (STICK.x + 0.01, STICK.y - 0.19, CONSOLE_TOP + 0.015), (0.09, 0.26, 0.03))

# --- throttles (left console): two levers side by side moving together on their rails in a
# quadrant (its cheeks with the idle, military and afterburner marks); each a slider block, a
# post and a grip across the top, the inboard one carrying the switches
for x in (-0.335, -0.305, -0.275, -0.245):
    box("metal", "metal", (x, THROTTLE.y + THROTTLE_TRAVEL / 2, CONSOLE_TOP + 0.007), (0.005, THROTTLE_TRAVEL + 0.05, 0.006))
for x in (-0.352, -0.228):
    box("dark", "dark", (x, THROTTLE.y + THROTTLE_TRAVEL / 2, CONSOLE_FACE + 0.012), (0.008, THROTTLE_TRAVEL + 0.11, 0.024))
for y in (THROTTLE.y, THROTTLE.y + 0.14, THROTTLE.y + 0.17):
    box("metal", "metal", (-0.352, y, CONSOLE_FACE + 0.0245), (0.009, 0.003, 0.002))
box("black", "black", (-0.29, THROTTLE.y + THROTTLE_TRAVEL / 2, CONSOLE_FACE + 0.001), (0.13, THROTTLE_TRAVEL + 0.11, 0.002))
throttle = empty("throttle", THROTTLE, props={"throttle": True})
empty("throttle_full", THROTTLE + Vector((0.0, THROTTLE_TRAVEL, 0.0)), props={"anchor": "throttle_full"})
for dx in (-0.03, 0.03):
    mesh_object("dark", box_mesh("dark", (0.026, 0.06, 0.03), "dark", (dx, 0.0, -0.07)), (0, 0, 0), parent=throttle)
    mesh_object("metal", loft_mesh("metal", [(Vector((dx, 0.0, -0.056)), X * 0.009, Y * 0.014), (Vector((dx, 0.0, -0.014)), X * 0.008, Y * 0.011)], "metal"), (0, 0, 0), parent=throttle)
# The two grips joined into one handle across the levers, rising inboard.
rings = [(-0.052, 0.012, 0.012), (-0.045, 0.020, 0.017), (-0.02, 0.026, 0.021), (0.015, 0.027, 0.022), (0.045, 0.024, 0.022), (0.058, 0.018, 0.018), (0.064, 0.010, 0.011)]
mid = lambda x: Vector((x, 0.004, 0.004 + x * 0.15))  # noqa: E731
mesh_object("rubber", loft_mesh("rubber", [(mid(x), Y * ry, Z * rz) for x, ry, rz in rings], "rubber", n=10), (0, 0, 0), parent=throttle)
mesh_object("dark", box_mesh("dark", (0.003, 0.05, 0.046), "dark", mid(0.0)), (0, 0, 0), parent=throttle)
for mat, me in (
    ("dark", cylinder_mesh("dark", 0.006, 0.006, "dark", axis="Z", centre=mid(0.035) + Z * 0.020)),
    ("dark", box_mesh("dark", (0.010, 0.010, 0.007), "dark", mid(0.012) + Vector((0.0, -0.010, 0.022)))),
    ("metal", cylinder_mesh("metal", 0.007, 0.006, "metal", axis="X", centre=mid(0.045) + Vector((0.0, 0.018, 0.010)), n=12)),
    ("dark", box_mesh("dark", (0.008, 0.008, 0.012), "dark", mid(-0.005) + Y * 0.028)),
    ("dark", box_mesh("dark", (0.012, 0.008, 0.008), "dark", mid(0.025) - Y * 0.026)),
):
    mesh_object(mat, me, (0, 0, 0), parent=throttle)

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
# Target buttons in a row under the countermeasures; the gear handle under the gear lights.
for i, n in enumerate(["button_NEAR", "button_NEXT", "button_NONE", "button_WPN", "button_DTL"]):
    o = objs[n]
    u, v = 142.0 + 32.0 * i, 252.0
    o.location = point(u, v, PLATE)
    o.rotation_mode = "QUATERNION"
    o.rotation_quaternion = rotation(u, v)
g = objs["control_Action_Gear_.001"]
g.location = point(-298.0, 204.0, PLATE)
g.rotation_mode = "QUATERNION"
g.rotation_quaternion = rotation(-298.0, 204.0)
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
ind.location = Vector((-(post_x + HUD_BAR / 2 + 0.016), HUD_Y, -0.15))
box("dark", "dark", (ind.location.x, HUD_Y + 0.007, ind.location.z), (0.028, 0.010, 0.068))
box("dark", "dark", (ind.location.x, HUD_Y + 0.007, (ind.location.z - 0.034 + HUD_BODY_LOW) / 2), (0.010, 0.010, ind.location.z - 0.034 - HUD_BODY_LOW))
# Flood lamps: the ones behind the pilot's head inside the walls, one under the HUD's hood.
for o in old:
    if o.name.startswith("lamp") and abs(o.location.x) > wall_x(o.location.y) - 0.05:
        o.location.x = math.copysign(wall_x(o.location.y) - 0.05, o.location.x)
objs["lamp.002"].location = Vector((0.0, 0.62, -0.24))

bpy.data.orphans_purge(do_recursive=True)
bpy.context.view_layer.update()
print(f"throttle {throttle.location} -> {throttle.location + Vector((0.0, THROTTLE_TRAVEL, 0.0))}; "
      f"glare shield top {top_z:.3f} .. {back_z:.3f} at y {front[1].y:.3f} .. {COAMING_BACK}; seat bottom {bottom:.3f}; "
      f"panel bottom {low}; HUD combiner x +-{HUD_GLASS[0]}, z {HUD_GLASS[1]} .. {HUD_GLASS[2]} at y {HUD_Y}")
bpy.ops.wm.save_as_mainfile(filepath=out)
print("built", out, "objects", len(bpy.data.objects))
