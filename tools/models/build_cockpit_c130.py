# Builds the AC-130 gunship's flight deck master (assets-src/blend/cockpit_c130.blend) from the
# panel layout, after the C-130J's glass flight deck, as the C-5M's (build_cockpit_c5.py) with
# parts of the F-117A's modern cockpit:
#   blender -b assets-src/blend/cockpit_modern.blend --python-exit-code 1 \
#       --python tools/models/build_cockpit_c130.py -- \
#       assets-src/blend/cockpit_c130_layout.json assets-src/blend/cockpit_c130.blend
# then tools/models/export.sh cockpit_c130. The layout JSON is written by
# `UPDATE_LAYOUT=1 cargo test layout_files` from `PanelLayout::c130` (src/cockpit3d/layout.rs).
#
# Life size (the F-117A's kept parts scaled down by F117_SCALE about the eye). Kept, with their
# materials, labels and control properties: the gear handle, the five target buttons, the HUD
# glass, the approach indexer and the flood lamps. Built here, low-poly: one narrow main panel
# across both seats under the glare shield, following the nose's skin; each pilot's attitude
# display (the copilot's dark) with the yoke in front of it, the standby instruments and the
# warnings display inboard of the pilot's, the two MFDs between the pilots, the gear column
# right of them; the autopilot panel with the gun select buttons on the glare shield's edge; the
# pilot's overhead HUD (its combiner in front of the eye) and the copilot's folded up; the
# pedestal between the seats with the four throttles between the two CDUs, the countermeasures
# and the target buttons behind them; the side consoles (the nosewheel steering tiller on the
# pilot's), the overhead panel with the engine fire handles, both crew seats, both yokes (the
# copilot's a plain model), the rudder pedals, the floor, the footwell (open to the lower nose
# windows) and the bulkhead behind the seats closing off the empty fuselage.
#
# The exterior (tools/models/c130.py, model 76) is drawn round the flight deck from inside
# (src/aircraft.rs): its skin round the windscreen's panes is the windscreen's frame, so no walls
# or frames here. Where a part meets the skin it follows `skin()`, the exterior's own sections
# (c130.py's `skin_x`), a little inside it.
#
# Blender frame: eye at the origin (the pilot in the left seat, the exterior's (-0.62, 12.50,
# 0.95), station 2.30 aft of the nose tip; the cabin's centre line is CABIN_X to the right),
# x right, y forward, z up, metres. A control's face points along its local -Y (towards the
# pilot), local Z is up the panel.
import json
import math
import os
import sys

import bmesh
import bpy
from mathutils import Euler, Matrix, Quaternion, Vector

layout_path, out = sys.argv[sys.argv.index("--") + 1:][:2]
layout = json.load(open(layout_path))
root = bpy.data.objects["cockpit"]

F117_SCALE = 1.4706
# The eye in the exterior's frame (its station aft of the nose tip), the cabin's centre line in
# the eye's.
EYE = (-0.62, 12.50, 0.95)
EYE_D = 2.30
CABIN_X = 0.62
CABIN_U = CABIN_X * 1000.0

# --- the exterior's fuselage (tools/models/c130.py): its sections and stations
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import c130  # noqa: E402


def skin(y, z):
    """The skin's half width (from the cabin's centre line) at eye-frame y, z; 0 above the roof,
    under the belly or ahead of the nose."""
    d, z = EYE_D - y, z + EYE[2]
    if d < c130.BODY[0][0] or d > c130.BODY[-1][0]:
        return 0.0
    top, belly, _ = c130.station(d)
    if not belly <= z <= top:
        return 0.0
    return c130.skin_x(d, z)


def roof(y, x=CABIN_X):
    """The skin's top over eye-frame (x, y)."""
    z = 0.0
    while skin(y, z + 0.01) > abs(x - CABIN_X):
        z += 0.01
    return z


# --- the flight deck
# Floor (the seated floor, z -0.25 in the exterior's frame), the bulkhead behind the seats (the
# rest of the fuselage is empty behind it). The floor ends at FOOT_Y; the footwell's floor runs on
# down to NOSE_FLOOR at its front wall (NOSE_Y), under the lower nose windows; its ceiling (the
# panel housing's underside) rises aft from CEIL_Z there to the panel's bottom edge, over them.
FLOOR = -1.20
BULK_Y = -1.55
FOOT_Y = 1.00
NOSE_Y = 1.40
NOSE_FLOOR = -1.42
CEIL_Z = -0.85
# Inside the skin.
SKIN_IN = 0.015
INSET = 0.02
# The glare shield: a deck from its aft edge (LIP_Y, at GLARE_Z, lower outboard, under the
# sight lines over it to the windscreen panes' lower edge) sloping down forward to the skin
# just under the panes (SILL_Z; the panes' lower edge is PANE_Z), the lip's face down to the
# panel's top edge.
GLARE_Z = -0.18
PANE_Z = -0.29
SILL_Z = -0.32
LIP_Y = 0.60
# The main panel: its plane leaning back PANEL_TILT, its top edge (layout y PANEL_TOP) under
# the lip at TOP_EDGE, out to the skin either side; down to SIDE_BOTTOM, its centre part between
# the pilots (CENTRE_U) down to the pedestal (CENTRE_BOTTOM).
PANEL_TILT = math.radians(10.0)
PANEL_TOP = -215.0
TOP_EDGE = Vector((0.0, 0.64, -0.25))
CENTRE_U = (380.0, 860.0)
SIDE_BOTTOM, CENTRE_BOTTOM = 130.0, 330.0
RIGHT = Vector((1.0, 0.0, 0.0))
UP = Vector((0.0, math.sin(PANEL_TILT), math.cos(PANEL_TILT)))
NORMAL = Vector((0.0, -math.cos(PANEL_TILT), math.sin(PANEL_TILT)))
PANEL_CENTRE = TOP_EDGE + UP * (PANEL_TOP / 1000)
# The pedestal between the seats (layout x 2500..4000, centred on PED_U): its top from under the
# panel's centre part (layout y 0) aft to PED_REAR, PED_W wide.
PED_U = 3000.0
PED_W = 0.46
PED_FRONT = PANEL_CENTRE + RIGHT * CABIN_X - UP * (CENTRE_BOTTOM / 1000) - NORMAL * 0.03
PED_REAR = Vector((CABIN_X, -0.55, PED_FRONT.z - 0.03))
SLOPE = (PED_REAR - PED_FRONT).normalized()  # aft along the pedestal
P_NORMAL = Vector((0.0, SLOPE.z, -SLOPE.y))  # up out of it
# The autopilot panel on the lip (layout x >= 4500, centred on GLARE_U, its middle at AFCS_Z),
# standing AFCS_D off it.
GLARE_U = 5000.0
AFCS_U = (4600.0, 5400.0)
AFCS_D = 0.03
AFCS_Z = -0.215
# Side consoles: from INNER_X (mirrored for the copilot) out to the skin, their top at
# CONSOLE_TOP, from the bulkhead forward to the panel.
INNER_X = -0.36
CONSOLE_TOP = -0.68
CONSOLE_FRONT = 0.55
# Overhead panel on the centre line: its face from OVERHEAD_Y[0] (aft) to [1], at OVERHEAD_Z
# (aft, fore), OVERHEAD_W wide, up to the roof.
OVERHEAD_Y = (-0.55, 0.28)
OVERHEAD_Z = (0.66, 0.44)
OVERHEAD_W = 0.52
# Seats: the pilot's at x 0, the copilot's mirrored; the cushion's top (the eye over the hips).
SEAT_TOP = -0.78
COPILOT_X = 2 * CABIN_X
# The yoke: its column's pivot on the floor in front of the seat, the wheel's hub at chest
# height; the wheel's grips' half span.
YOKE_PIVOT = Vector((0.0, 0.45, FLOOR + 0.02))
YOKE_HUB = Vector((0.0, 0.40, -0.52))
WHEEL_SPAN = 0.16
# The quadrant between the CDUs (its length along the pedestal, layout mm; its half width, m);
# the throttles: four levers swinging forward about THROTTLE_PIVOT (in the quadrant), their
# knobs' middle at THROTTLE_IDLE, mirrored forward at full thrust.
QUADRANT = (60.0, 300.0)
QUAD_W = 0.07
QUAD_H = 0.055
HP = None  # set below, once point() exists
# HUD: overhead, its combiner glass HUD_GLASS (half width, bottom, top) at HUD_Y in front of the
# eye (AircraftSpec::hud_scale fits the symbology into it; as the C-5M's), hanging from the
# projector over the pilot's head; the copilot's folded up.
HUD_Y = 0.32
HUD_GLASS = (0.075, -0.065, 0.045)
HUD_BAR = 0.010
PROJECTOR = (Vector((0.0, 0.17, 0.215)), Vector((0.13, 0.20, 0.10)))
# Pedals: either side of each pilot's centre line.
PEDAL_X = 0.13
PEDAL_Y = 0.80
# MFD bezels standing BEZEL off the panel; plates under the lights.
BEZEL = 0.012
PLATE = 0.004
ROUND = 16
KNOB = 8
# The pedestal's CDU keys (layout y range).
CDU_KEYS = (150.0, 200.0)


# --- the surfaces: the main panel, the pedestal, the autopilot panel
def surface(u):
    if u >= 4500.0:
        return "glare"
    if u >= 2500.0:
        return "pedestal"
    return "panel"


def basis(u):
    """(across, normal towards the eye, up the face) of the surface at layout x u."""
    s = surface(u)
    if s == "pedestal":
        return RIGHT, P_NORMAL, -SLOPE
    if s == "glare":
        return RIGHT, Vector((0.0, -1.0, 0.0)), Vector((0.0, 0.0, 1.0))
    return RIGHT, NORMAL, UP


def point(u, v, offset=0.0):
    """A point of a surface: layout millimetres (u right, v down / aft), `offset` m off it."""
    s = surface(u)
    if s == "pedestal":
        p = PED_FRONT + RIGHT * ((u - PED_U) / 1000) + SLOPE * (v / 1000)
        return p + P_NORMAL * offset
    if s == "glare":
        p = Vector((CABIN_X + (u - GLARE_U) / 1000, LIP_Y - AFCS_D, AFCS_Z - v / 1000))
        return p + Vector((0.0, -1.0, 0.0)) * offset
    return PANEL_CENTRE + RIGHT * (u / 1000) - UP * (v / 1000) + NORMAL * offset


def rotation(u):
    d, n, up = basis(u)
    return Matrix((d, -n, up)).transposed().to_quaternion()


def centre(r):
    x, y, w, h = r
    return x + w / 2.0, y + h / 2.0


def rect_point(r, offset=0.0):
    return point(*centre(r), offset)


def mirror(r):
    """A main panel rectangle mirrored onto the copilot's side."""
    x, y, w, h = r
    return (2 * CABIN_U - x - w, y, w, h)


def panel_half(v):
    """The main panel's half width (m, from the centre line) at layout y v: inside the skin, its
    plate's back too."""
    p = point(CABIN_U, v)
    q = p - NORMAL * 0.03
    return min(skin(p.y, p.z), skin(q.y, q.z)) - INSET


THROTTLE_PIVOT = point(PED_U, 190.0, -0.03)
THROTTLE_IDLE = THROTTLE_PIVOT + Vector((0.0, -0.12, 0.20))
THROTTLE_FULL = THROTTLE_PIVOT + Vector((0.0, 0.12, 0.20))

# --- keep, drop
KEEP = (
    "hud_glass", "approach", "lamp", "control_Action_Gear_.001",
    "button_NEAR", "button_NEXT", "button_NONE", "button_WPN", "button_DTL",
)


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


def material(name, like, colour=None):
    m = mats.get(name)
    if m is None:
        m = mats[like].copy()
        m.name = name
    if colour is not None:
        m.diffuse_color = colour
        for nd in m.node_tree.nodes if m.use_nodes else ():
            if nd.type == "BSDF_PRINCIPLED":
                nd.inputs["Base Color"].default_value = colour
    return m


for n, like in (("ded", "adi"), ("screen_lmpd", "screen_right"), ("trackcam_left", "trackcam")):
    material(n, like)
# Not shared materials: the game takes their colours from the model. The fire handles red, the
# crew seats' grey fabric.
material("red", "yellow", (0.55, 0.04, 0.03, 1.0))
material("fabric", "seat", (0.13, 0.135, 0.14, 1.0))


# --- building blocks (as the C-5M's)
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
    m = Matrix.Translation(centre) @ (quat(rot).to_matrix().to_4x4() if rot is not None else Matrix.Identity(4)) @ Matrix.Diagonal((*size, 1.0))
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
        "Y": Matrix.Rotation(math.radians(-90.0), 4, "X"),
        "X": Matrix.Rotation(math.radians(90.0), 4, "Y"),
        "-X": Matrix.Rotation(math.radians(-90.0), 4, "Y"),
    }[axis]
    m = Matrix.Translation(centre) @ rot @ Matrix.Translation((0.0, 0.0, h / 2))
    bmesh.ops.create_cone(bm, cap_ends=True, cap_tris=False, segments=n, radius1=r, radius2=r, depth=h, matrix=m)
    return finish(bm, me, mat)


def loft_mesh(name, rings, mat, n=KNOB):
    """A closed tube through rings (centre, a, b): each ring the points centre + a cos t + b sin t."""
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


def prism(name, mat, yz, x0, x1):
    """A convex outline in the y-z plane (m) from x0 to x1."""
    return mesh_object(name, section_mesh(name, [[(x, y, z) for y, z in yz] for x in (x0, x1)], mat), (0, 0, 0))


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


def panel_box(name, mat, r, raise_m, margin=0.0, base=0.0):
    """A plate from `base` to `base + raise_m` off the surface over the rectangle r (mm) grown by
    `margin` mm."""
    x, y, w, h = r
    c = rect_point(r, base + raise_m / 2)
    return box(name, mat, c, ((w + 2 * margin) / 1000, abs(raise_m), (h + 2 * margin) / 1000), rotation(x + w / 2))


def frame(mat, outer, inner, d0, d1):
    """A rectangular frame (mm, outer and inner rectangles) from d0 to d1 m off the surface."""
    ox, oy, ow, oh = outer
    ix, iy, iw, ih = inner
    for r in ((ox, oy, ow, iy - oy), (ox, iy + ih, ow, oy + oh - iy - ih), (ox, iy, ix - ox, ih), (ix + iw, iy, ox + ow - ix - iw, ih)):
        panel_box(mat, mat, r, d1 - d0, base=d0)


def knob(u, v, r=0.006, h=0.012, mat="metal", offset=0.0):
    mesh_object(mat, cylinder_mesh(mat, r, h, mat), point(u, v, offset), rotation(u))


def toggle(u, v, offset=0.0, up=True):
    mesh_object("dark", cylinder_mesh("dark", 0.005, 0.004, "dark"), point(u, v, offset), rotation(u))
    lean = Matrix.Rotation(math.radians(-35.0 if up else 35.0), 3, "X")
    me = box_mesh("metal", (0.004, 0.016, 0.004), "metal", (0.0, -0.011, 0.0), lean.to_quaternion())
    mesh_object("metal", me, point(u, v, offset + 0.003), rotation(u))


def bezel(r, offset, height=0.008, lip=0.003):
    """A round black bezel standing `height` out of the surface round the rectangle r's
    inscribed circle, the instrument's octagonal case behind it."""
    x, y, w, h = r
    me = cylinder_mesh("black", min(w, h) / 2000 + lip, height, "black", n=ROUND)
    mesh_object("black", me, rect_point(r, offset), rotation(x + w / 2))
    flat = min(w, h) / 2000 + lip + 0.002
    me = cylinder_mesh("dark", flat / math.cos(math.pi / 8), offset + 0.003, "dark", n=8)
    me.transform(Matrix.Rotation(math.pi / 8, 4, "Y"))
    mesh_object("dark", me, rect_point(r, 0.0), rotation(x + w / 2))


def face(key, r, offset, round=False):
    """An atlas face over the rectangle r; round ones are discs in a bezel (the instruments)."""
    x, y, w, h = r
    if round:
        bezel(r, offset)
        me = disc_mesh("atlas", min(w, h) / 2000, "atlas")
        offset += 0.0085
    else:
        me = quad_mesh("atlas", w / 1000, h / 1000, "atlas")
    mesh_object("atlas", me, rect_point(r, offset), rotation(x + w / 2), props={"atlas_face": key, "atlas_uv": [0.0, 0.0, 1.0, 1.0], "no_shadow": True})


def control(key, r, offset, effect, kind):
    """A push button: origin on the surface, its face 6 mm out; a dark cap body under the face
    and a highlight frame."""
    x, y, w, h = r
    hw, hh = w / 2000, h / 2000
    height, travel = 0.006, 0.004
    name = "control_" + "".join(ch if ch.isalnum() else "_" for ch in effect)
    c = empty(name, rect_point(r, offset), rotation(x + w / 2), props={"control": effect, "kind": kind, "half": [hw, hh], "height": height, "travel": travel})
    moving = empty("moving", (0, 0, 0), parent=c, props={"moving": True})
    mesh_object("atlas", quad_mesh("atlas", 2 * hw, 2 * hh, "atlas", -height), (0, 0, 0), parent=moving, props={"atlas_face": key, "atlas_uv": [0.0, 0.0, 1.0, 1.0], "no_shadow": True})
    mesh_object("dark", box_mesh("dark", (2 * hw, height - 0.0004, 2 * hh), "dark", (0, -(height - 0.0004) / 2, 0)), (0, 0, 0), parent=moving)
    hl = empty("highlight", (0, 0, 0), parent=c, props={"highlight": True})
    t = 0.002
    for (cx, cz, sx, sz) in ((0, hh + t, 2 * hw + 2 * t, t), (0, -hh - t, 2 * hw + 2 * t, t), (hw + t, 0, t, 2 * hh), (-hw - t, 0, t, 2 * hh)):
        mesh_object("highlight", box_mesh("highlight", (sx, t, sz), "highlight"), (cx, -height, cz), parent=hl, props={"no_shadow": True})


def screen(mat, r, offset, behind=None):
    """A display over the rectangle r: a quad, a dark one (`behind`) just behind it."""
    rot = rotation(r[0] + r[2] / 2)
    x, y, w, h = r
    mesh_object(mat, quad_mesh(mat, w / 1000, h / 1000, mat), rect_point(r, offset), rot, props={"no_shadow": True})
    if behind:
        mesh_object(behind, quad_mesh(behind, w / 1000, h / 1000, behind), rect_point(r, offset - 0.003), rot, props={"no_shadow": True})


def place(o, u, v, offset=0.0):
    """Moves a kept control onto a surface."""
    o.location = point(u, v, offset)
    o.rotation_mode = "QUATERNION"
    o.rotation_quaternion = rotation(u)


def strip(name, mat, a, b):
    """Quads between two polylines (one face each, facing the eye)."""
    me = bpy.data.meshes.new(name)
    bm = bmesh.new()
    va = [bm.verts.new(p) for p in a]
    vb = [bm.verts.new(p) for p in b]
    for i in range(len(a) - 1):
        bm.faces.new((va[i], va[i + 1], vb[i + 1], vb[i]))
    bmesh.ops.recalc_face_normals(bm, faces=bm.faces[:])
    for f in bm.faces:
        if f.normal.dot(Vector((0.0, 0.0, 0.0)) - f.calc_center_median()) < 0.0:
            f.normal_flip()
    return mesh_object(name, finish(bm, me, mat), (0, 0, 0))


def across(name, mat, path, inset=INSET, lo=None, hi=None):
    """A surface across the cabin from skin to skin (`inset` inside it) along a path of (y, z)
    points, optionally clipped to x lo..hi."""
    left, right = [], []
    for y, z in path:
        w = max(skin(y, z) - inset, 0.0)
        x0, x1 = CABIN_X - w, CABIN_X + w
        if lo is not None:
            x0 = max(x0, lo)
        if hi is not None:
            x1 = min(x1, hi)
        left.append(Vector((x0, y, z)))
        right.append(Vector((x1, y, z)))
    return strip(name, mat, left, right)


def lerp_path(a, b, n):
    return [(a[0] + (b[0] - a[0]) * k / n, a[1] + (b[1] - a[1]) * k / n) for k in range(n + 1)]


scr = layout["screens"]
faces = {f["key"]: f["rect"] for f in layout["faces"]}

# --- floor, side consoles, the footwell (its floor down under the lower nose windows, its front
# wall, its ceiling over them), the pedestal's foot running forward between the footwells, the
# bulkhead behind the seats
across("dark", "dark", lerp_path((BULK_Y, FLOOR), (FOOT_Y, FLOOR), 16))
across("dark", "dark", lerp_path((FOOT_Y, FLOOR), (NOSE_Y, NOSE_FLOOR), 6))
low_side = point(0.0, SIDE_BOTTOM, -0.03)
across("dark", "dark", lerp_path((low_side.y, low_side.z - 0.004), (NOSE_Y, CEIL_Z), 8))
# The front wall: from the footwell's floor up to its ceiling.
across("dark", "dark", [(NOSE_Y, NOSE_FLOOR + (CEIL_Z - NOSE_FLOOR) * k / 8) for k in range(9)])
low_mid = point(CABIN_U, CENTRE_BOTTOM, -0.002)
c0, c1 = CENTRE_U[0] / 1000 + 0.002, CENTRE_U[1] / 1000 - 0.002
prism("panel_grey", "panel_grey", [(low_mid.y, low_mid.z), (NOSE_Y - 0.01, low_mid.z), (NOSE_Y - 0.01, NOSE_FLOOR), (low_mid.y, NOSE_FLOOR)], c0, c1)
YS = [BULK_Y + (CONSOLE_FRONT - BULK_Y) * k / 10 for k in range(11)]
for s in (-1, 1):
    secs = []
    for y in YS:
        inner = CABIN_X + s * (CABIN_X - INNER_X)
        out_top = CABIN_X + s * (skin(y, CONSOLE_TOP) - SKIN_IN)
        out_low = CABIN_X + s * (skin(y, FLOOR) - SKIN_IN)
        secs.append([(inner, y, CONSOLE_TOP), (out_top, y, CONSOLE_TOP), (out_low, y, FLOOR + 0.002), (inner, y, FLOOR + 0.002)])
    mesh_object("panel_grey", section_mesh("panel_grey", secs, "panel_grey"), (0, 0, 0))
    # A black rim along the console's inboard edge.
    x = CABIN_X + s * (CABIN_X - INNER_X - 0.01)
    box("black", "black", (x, (BULK_Y + CONSOLE_FRONT) / 2, CONSOLE_TOP + 0.003), (0.02, CONSOLE_FRONT - BULK_Y, 0.006))


def ring(y, z0, out):
    """The skin's section at y above z0, `out` m outside it (a convex outline, x and z)."""
    left, right = [], []
    z = z0
    while True:
        w = skin(y, z)
        if w <= 0.0:
            break
        left.append((CABIN_X - w - out, z))
        right.append((CABIN_X + w + out, z))
        z += 0.05
    top = z - 0.05 + out
    return left + [(CABIN_X, top)] + right[::-1]


# The bulkhead: the whole section behind the seats, a door in its middle, black circuit breaker
# panels either side.
sec = ring(BULK_Y, FLOOR - 0.02, -SKIN_IN)
mesh_object("panel_grey", section_mesh("panel_grey", [[(x, BULK_Y, z) for x, z in sec], [(x, BULK_Y - 0.03, z) for x, z in sec]], "panel_grey"), (0, 0, 0))
DOOR = (CABIN_X - 0.32, CABIN_X + 0.32, FLOOR, 0.62)
x0, x1, z0, z1 = DOOR
box("dark", "dark", ((x0 + x1) / 2, BULK_Y + 0.004, (z0 + z1) / 2), (x1 - x0, 0.008, z1 - z0))
for x in (x0, x1):
    box("frame", "frame", (x, BULK_Y + 0.01, (z0 + z1) / 2), (0.03, 0.02, z1 - z0))
box("frame", "frame", ((x0 + x1) / 2, BULK_Y + 0.01, z1), (x1 - x0 + 0.03, 0.02, 0.03))
box("metal", "metal", (x1 - 0.07, BULK_Y + 0.02, -0.25), (0.02, 0.02, 0.12))
box("black", "black", ((x0 + x1) / 2, BULK_Y + 0.009, 0.25), (0.30, 0.004, 0.20))
for s in (-1, 1):
    xc = CABIN_X + s * 0.75
    box("black", "black", (xc, BULK_Y + 0.006, -0.05), (0.60, 0.012, 0.70))
    for i in range(6):
        for j in range(5):
            box("dark", "dark", (xc - 0.25 + 0.1 * j + 0.05 * (i % 2), BULK_Y + 0.016, -0.32 + 0.11 * i), (0.012, 0.012, 0.012))

# --- glare shield: the deck from the lip forward to the skin under the windscreen, the lip's
# face down to the panel's top edge, the autopilot panel on it


def sill_y(x, z):
    """The most forward y where the skin at height z is at least as wide as x."""
    y = 2.0
    while y > LIP_Y and skin(y, z) < abs(x - CABIN_X) + 0.01:
        y -= 0.005
    return y


def deck_aft_z(x):
    """The deck's aft edge's height at x: under the sight line to the panes' lower edge there."""
    return min(GLARE_Z, PANE_Z * LIP_Y / sill_y(x, PANE_Z) - 0.015)


top_half = min(panel_half(PANEL_TOP), skin(LIP_Y, GLARE_Z) - INSET, skin(LIP_Y, TOP_EDGE.z) - INSET)
XS = [CABIN_X - top_half + 2 * top_half * k / 40 for k in range(41)]
aft = [Vector((x, LIP_Y, deck_aft_z(x))) for x in XS]
fore = [Vector((x, sill_y(x, SILL_Z) - SKIN_IN, SILL_Z)) for x in XS]
lip = [Vector((x, LIP_Y, point(x * 1000, PANEL_TOP).z - 0.004)) for x in XS]
under = [point(x * 1000, PANEL_TOP, 0.002) for x in XS]

strip("black", "black", aft, fore)
strip("black", "black", aft, lip)
strip("dark", "dark", lip, under)

# The autopilot panel: a box on the lip's middle, its keys (layout faces), windows and knobs; the
# gun select buttons on a black plate at its left end.
a0, a1 = AFCS_U
box("dark", "dark", point((a0 + a1) / 2, 0.0, -AFCS_D / 2), ((a1 - a0) / 1000, AFCS_D, 0.07))
for u in (4860.0, 5140.0, 5240.0, 5340.0):
    panel_box("black", "black", (u - 32.0, -28.0, 44.0, 12.0), 0.002)
    knob(u + 26.0, -22.0, 0.007, 0.012, "metal", 0.0)
    for k in range(3):
        panel_box("dark", "dark", (u - 30.0 + 15.0 * k, 12.0, 12.0, 9.0), 0.004)
panel_box("black", "black", (4628.0, -14.0, 108.0, 28.0), 0.002)

# --- the main panel: one plate across the cabin out to the skin (in strips down it), its centre
# part down to the pedestal; rims


def plate(name, mat, pieces, depth):
    """A panel plate `depth` m thick behind the panel plane, from convex outlines (layout mm)
    that together make its shape."""
    me = bpy.data.meshes.new(name)
    bm = bmesh.new()
    for loop in pieces:
        front = [bm.verts.new(point(u, v)) for u, v in loop]
        back = [bm.verts.new(f.co - NORMAL * depth) for f in front]
        bm.faces.new(front)
        bm.faces.new(back[::-1])
        for i in range(len(loop)):
            j = (i + 1) % len(loop)
            bm.faces.new((front[i], back[i], back[j], front[j]))
    bmesh.ops.recalc_face_normals(bm, faces=bm.faces[:])
    return mesh_object(name, finish(bm, me, mat), (0, 0, 0))


c0, c1 = CENTRE_U
VS = [PANEL_TOP + (SIDE_BOTTOM - PANEL_TOP) * k / 12 for k in range(13)]
pieces = []
for va, vb in zip(VS, VS[1:]):
    wa, wb = panel_half(va) * 1000, panel_half(vb) * 1000
    pieces.append([(CABIN_U - wa, va), (CABIN_U + wa, va), (CABIN_U + wb, vb), (CABIN_U - wb, vb)])
pieces.append([(c0, SIDE_BOTTOM - 1.0), (c1, SIDE_BOTTOM - 1.0), (c1, CENTRE_BOTTOM), (c0, CENTRE_BOTTOM)])
plate("panel_grey", "panel_grey", pieces, 0.03)
wb = panel_half(SIDE_BOTTOM) * 1000
for a, b in ((CABIN_U - wb, c0), (c1, CABIN_U + wb)):
    panel_box("dark", "dark", (a, SIDE_BOTTOM - 6.0, b - a, 6.0), 0.008)
for a in (c0, c1 - 6.0):
    panel_box("dark", "dark", (a, SIDE_BOTTOM - 6.0, 6.0, CENTRE_BOTTOM - SIDE_BOTTOM), 0.008)
# Radio and lighting panels low on the centre part (dressing).
for k in range(4):
    v = 50.0 + 70.0 * k
    panel_box("black", "black", (c0 + 25.0, v, c1 - c0 - 50.0, 50.0), PLATE)
    for j in range(6):
        knob(c0 + 60.0 + 75.0 * j, v + 25.0, 0.008, 0.012, "metal", PLATE)
        panel_box("dark", "dark", (c0 + 77.0 + 75.0 * j, v + 18.0, 34.0, 14.0), 0.002, base=PLATE)


# --- MFDs: a housing standing BEZEL off the panel round the screen, the screen back in a black
# recess; twenty push buttons round it (layout faces) with ridges between them, rocker switches
# at the corners, brightness knobs along the bottom.
def mfd_bezel(r):
    x, y, w, h = r
    outer = (x - 16.0, y - 20.0, w + 32.0, h + 46.0)
    hole = (x - 3.0, y - 3.0, w + 6.0, h + 6.0)
    frame("dark", outer, hole, 0.0, BEZEL - 0.002)
    frame("black", (outer[0] + 1.5, outer[1] + 1.5, outer[2] - 3.0, outer[3] - 3.0), hole, BEZEL - 0.002, BEZEL)
    frame("black", hole, (x - 0.5, y - 0.5, w + 1.0, h + 1.0), -0.006, BEZEL - 0.001)
    for k in range(1, 5):
        for by in (y - 6.5, y + h + 6.5):
            panel_box("black", "black", (x + w * k / 5 - 1.0, by - 4.0, 2.0, 8.0), 0.003, base=BEZEL)
    for k in range(1, 5):
        for bx in (x - 9.0, x + w + 9.0):
            panel_box("black", "black", (bx - 4.0, y + h * k / 5 - 1.0, 8.0, 2.0), 0.003, base=BEZEL)
    for bx in (x - 9.0, x + w + 9.0):
        for by in (y - 9.0, y + h + 9.0):
            panel_box("dark", "dark", (bx - 4.0, by - 5.0, 8.0, 10.0), 0.003, base=BEZEL)
            panel_box("black", "black", (bx - 1.5, by - 4.0, 3.0, 8.0), 0.003, base=BEZEL + 0.003)
    kv = y + h + 19.0
    knob(x + 18.0, kv, 0.005, 0.008, offset=BEZEL)
    knob(x + w / 2, kv, 0.004, 0.008, "dark", offset=BEZEL)
    knob(x + w - 18.0, kv, 0.005, 0.008, offset=BEZEL)


for key in ("left", "right"):
    mfd_bezel(scr[key])
screen("screen_left", scr["left"], BEZEL - 0.006, "trackcam_left")
screen("screen_right", scr["right"], BEZEL - 0.006, "trackcam")


# The attitude displays (the pilot's, the copilot's dark): a display unit standing off the panel,
# the screen recessed, a row of keys and two knobs under it.
def display_unit(r):
    x, y, w, h = r
    outer = (x - 12.0, y - 12.0, w + 24.0, h + 36.0)
    hole = (x - 1.5, y - 1.5, w + 3.0, h + 3.0)
    frame("dark", outer, hole, 0.0, 0.010)
    frame("black", hole, (x - 0.5, y - 0.5, w + 1.0, h + 1.0), -0.004, 0.009)
    for k in range(5):
        panel_box("black", "black", (x + 22.0 + 20.0 * k, y + h + 8.0, 14.0, 8.0), 0.003, base=0.010)
    knob(x + 8.0, y + h + 12.0, 0.005, 0.008, offset=0.010)
    knob(x + w - 8.0, y + h + 12.0, 0.005, 0.008, offset=0.010)


display_unit(scr["adi"])
screen("adi", scr["adi"], 0.005, "black")
display_unit(mirror(scr["adi"]))
screen("black", mirror(scr["adi"]), 0.005)
# The warnings display: a plain bezel, the screen recessed.
x, y, w, h = scr["lmpd"]
frame("dark", (x - 6.0, y - 6.0, w + 12.0, h + 12.0), (x - 1.5, y - 1.5, w + 3.0, h + 3.0), 0.0, 0.008)
frame("black", (x - 1.5, y - 1.5, w + 3.0, h + 3.0), (x - 0.5, y - 0.5, w + 1.0, h + 1.0), -0.004, 0.007)
screen("screen_lmpd", scr["lmpd"], 0.004, "black")
# The copilot's standby instruments: dark round cases.
for key in ("Item(Dial(Airspeed))", "Item(Dial(Altimeter))"):
    bezel(mirror(faces[key]), PLATE)

# --- plates under the light groups (the copilot's mirrored, dark), faces and buttons
plates = [
    ("dark", (-102.0, -150.0, 188.0, 24.0)),  # master caution, stall, threat lights
    ("dark", (434.0, -202.0, 152.0, 56.0)),  # warning and caution lights
    ("dark", (652.0, -200.0, 160.0, 56.0)),  # fuel, thrust, EMV
    ("dark", (94.0, -120.0, 150.0, 104.0)),  # standby instruments
    ("dark", (872.0, -118.0, 56.0, 112.0)),  # gear, flaps, brake
]
for mat, r in plates:
    panel_box(mat, mat, r, PLATE)
panel_box("dark", "dark", mirror(plates[0][1]), PLATE)
panel_box("dark", "dark", mirror(plates[3][1]), PLATE)
for k in range(5):
    panel_box("black", "black", mirror((-96.0 + 36.0 * k, -146.0, 32.0, 15.0)), 0.006, base=PLATE)
for f in layout["faces"]:
    fx, fy, fw, fh = f["rect"]
    on_cdu = surface(fx) == "pedestal" and CDU_KEYS[0] <= fy < CDU_KEYS[1]
    if f["control"] is None:
        # The CDU's blank keys stand on its housing (6 mm) as its live ones.
        face(f["key"], f["rect"], 0.0065 if on_cdu else PLATE + 0.0005, f["round"])
        if f["key"].startswith("Item(Dial(") and not f["round"]:
            frame("black", (fx - 2.0, fy - 2.0, fw + 4.0, fh + 4.0), (fx, fy, fw, fh), 0.0, PLATE + 0.004)
    elif f["kind"] == "osb":
        control(f["key"], f["rect"], BEZEL, f["control"], "osb")
    elif surface(fx) == "panel":
        control(f["key"], f["rect"], PLATE, f["control"], f["kind"])
    else:
        # On the pedestal: the CDU keys stand on its housing (6 mm), the rest on their panels;
        # on the glare shield on the autopilot panel's face.
        control(f["key"], f["rect"], 0.006 if on_cdu else 0.004, f["control"], f["kind"])
# Setting knobs at the standby instruments' corners.
for r in (faces["Item(Dial(Airspeed))"], faces["Item(Dial(Altimeter))"]):
    knob(r[0] + r[2] - 3.0, r[1] + r[3] - 3.0, 0.0045, 0.009, "dark", offset=PLATE)
# The gear handle under the gear lights.
place(objs["control_Action_Gear_.001"], 900.0, -18.0, PLATE)
# Dressing under the attitude displays (behind the yokes): switch rows.
for s_ in (0, 1):
    for k in range(5):
        u = -40.0 + 30.0 * k
        if s_:
            u = 2 * CABIN_U - u
        toggle(u, 70.0, 0.0, up=(k % 2 == 0))

# --- pedestal: its top from under the panel aft, filled to the floor; the throttle quadrant in
# its middle between the CDUs, the countermeasures and the target buttons behind them
half = PED_W / 2
f0, f1 = PED_FRONT, PED_REAR
prism("panel_grey", "panel_grey", [(f0.y, f0.z), (f1.y, f1.z), (f1.y, FLOOR), (f0.y, FLOOR)], CABIN_X - half, CABIN_X + half)
# Its front panel (dressing).
panel_box("black", "black", (2780.0, 8.0, 440.0, 44.0), 0.004)
for j in range(5):
    knob(2820.0 + 90.0 * j, 30.0, 0.007, 0.012, "metal", 0.004)
    panel_box("dark", "dark", (2840.0 + 90.0 * j, 23.0, 40.0, 14.0), 0.002, base=0.004)
q0, q1 = QUADRANT
qa, qb = point(PED_U, q0), point(PED_U, q1)
yz = [(qa.y, qa.z), (qb.y, qb.z), (qb.y + 0.03 * SLOPE.y, qb.z + QUAD_H - 0.01), (qa.y - 0.01, qa.z + QUAD_H)]
prism("dark", "dark", yz, CABIN_X - QUAD_W, CABIN_X + QUAD_W)
# The slots: a black top with a slot per lever.
top = [(qa.y - 0.01, qa.z + QUAD_H + 0.002), (qb.y + 0.03 * SLOPE.y, qb.z + QUAD_H - 0.008)]
LEVERS = (-0.045, -0.015, 0.015, 0.045)
for x in LEVERS:
    p0 = Vector((CABIN_X + x, top[0][0] - 0.03, top[0][1]))
    p1 = Vector((CABIN_X + x, top[1][0] + 0.03, top[1][1]))
    rod("black", p0, p1, 0.005, "black", n=6)
# The throttles: one group of four levers swinging about their pivot together, each a lever
# up from the quadrant and a knob across its top; the origin is the knobs' middle at idle.
throttle = empty("throttle", THROTTLE_IDLE, props={"throttle": True})
empty("throttle_full", THROTTLE_FULL, props={"anchor": "throttle_full"})
empty("throttle_pivot", THROTTLE_PIVOT, props={"anchor": "throttle_pivot"})
pivot = THROTTLE_PIVOT - THROTTLE_IDLE
for x in LEVERS:
    # A flat blade from the pivot up to the knob, the knob a block with a rounded top.
    d = -pivot.normalized()
    rot = Vector((0.0, 0.0, 1.0)).rotation_difference(d)
    length = pivot.length - 0.02
    mesh_object("metal", box_mesh("metal", (0.007, 0.022, length), "metal", pivot + Vector((x, 0.0, 0.0)) + d * (length / 2), rot), (0, 0, 0), parent=throttle)
    mesh_object("black", box_mesh("black", (0.024, 0.045, 0.028), "black", (x, 0.0, -0.004)), (0, 0, 0), parent=throttle)
    mesh_object("black", loft_mesh("black", [(Vector((x - 0.012, 0.0, 0.010)), Vector((0, 0.0225, 0)), Vector((0, 0, 0.012))),
                                              (Vector((x + 0.012, 0.0, 0.010)), Vector((0, 0.0225, 0)), Vector((0, 0, 0.012)))], "black", n=10),
                (0, 0, 0), parent=throttle)
    mesh_object("metal", box_mesh("metal", (0.004, 0.02, 0.002), "metal", (x, -0.005, 0.0225)), (0, 0, 0), parent=throttle)
# The CDUs either side of the quadrant: the pilot's display (the `ded`) in a black housing over
# its keys, the copilot's the same dark; then the countermeasure panel, the target buttons'
# panel and the radio panels.
x, y, w, h = scr["ded"]
housing = (x - 8.0, y - 8.0, w + 16.0, 125.0)
panel_box("black", "black", housing, 0.006)
panel_box("black", "black", (2 * PED_U - housing[0] - housing[2], housing[1], housing[2], housing[3]), 0.006)
screen("ded", scr["ded"], 0.0065)
screen("black", (2 * PED_U - x - w, y, w, h), 0.0065)
for f in layout["faces"]:
    fx, fy, fw, fh = f["rect"]
    if surface(fx) == "pedestal" and CDU_KEYS[0] <= fy < CDU_KEYS[1]:
        panel_box("dark", "dark", (2 * PED_U - fx - fw, fy, fw, fh), 0.005, base=0.006)
panel_box("dark", "dark", (2782.0, 228.0, 136.0, 62.0), 0.004)
panel_box("dark", "dark", (3082.0, 228.0, 136.0, 62.0), 0.004)
for u in (3110.0, 3150.0, 3190.0):
    knob(u, 259.0, 0.007, 0.012, "metal", 0.004)
panel_box("dark", "dark", (2782.0, 300.0, 436.0, 46.0), 0.004)
for i, n in enumerate(["button_NEAR", "button_NEXT", "button_NONE", "button_WPN", "button_DTL"]):
    place(objs[n], 2852.0 + 32.0 * i, 323.0, 0.004)
# Radio panels aft (dressing).
for k in range(3):
    v = 356.0 + 70.0 * k
    panel_box("black", "black", (2782.0, v, 436.0, 60.0), 0.004)
    for j in range(4):
        knob(2830.0 + 110.0 * j, v + 30.0, 0.008, 0.012, "metal", 0.004)
        panel_box("dark", "dark", (2855.0 + 110.0 * j, v + 22.0, 50.0, 14.0), 0.002, base=0.004)

# --- side consoles' switch panels; the nosewheel steering tiller on the pilot's


def fitting(c, p):
    """A console fitting at p (on a panel's top): k / K small / large knob, t / T toggle, r red
    button, b a key."""
    if c in "kK":
        r, h = (0.006, 0.010) if c == "k" else (0.009, 0.013)
        mesh_object("dark", cylinder_mesh("dark", r + 0.003, 0.002, "dark", axis="Z", n=KNOB), p)
        mesh_object("metal", cylinder_mesh("metal", r, h, "metal", axis="Z", n=KNOB), p + Vector((0.0, 0.0, 0.002)))
    elif c in "tT":
        mesh_object("dark", cylinder_mesh("dark", 0.005, 0.004, "dark", axis="Z", n=KNOB), p)
        lean = Euler((math.radians(-30.0 if c == "t" else 30.0), 0.0, 0.0)).to_quaternion()
        mesh_object("metal", box_mesh("metal", (0.004, 0.004, 0.016), "metal", (0.0, 0.0, 0.008), lean), p + Vector((0.0, 0.0, 0.003)))
    elif c == "r":
        mesh_object("red", cylinder_mesh("red", 0.005, 0.006, "red", axis="Z", n=KNOB), p)
    elif c == "b":
        mesh_object("dark", box_mesh("dark", (0.011, 0.009, 0.005), "dark", (0.0, 0.0, 0.0025)), p)


def console_panel(x0, x1, y0, y1, rows, h=0.006):
    """A plate over x0..x1, y0..y1 (m) on a console, its fittings in rows from the front."""
    box("black", "black", ((x0 + x1) / 2, (y0 + y1) / 2, CONSOLE_TOP + h / 2), (x1 - x0, y1 - y0, h))
    for i, row in enumerate(rows):
        y = y1 - (y1 - y0) * (i + 0.5) / len(rows)
        for j, c in enumerate(row):
            fitting(c, Vector((x0 + (x1 - x0) * (j + 0.5) / len(row), y, CONSOLE_TOP + h)))


for rows, (y0, y1) in (
    (["ttttt", "krkk"], (-0.08, 0.12)),
    (["kkk", "TTk", "bbb"], (-0.40, -0.14)),
    (["kKk", "ttt"], (-1.20, -0.50)),
):
    x0 = CABIN_X - skin(y1, CONSOLE_TOP) + 0.04
    x1 = INNER_X - 0.025
    console_panel(x0, x1, y0, y1, rows)
    console_panel(2 * CABIN_X - x1, 2 * CABIN_X - x0, y0, y1, rows)
# The tiller: a small wheel on a post, leaning towards the pilot, a knob on its rim.
TILLER = Vector((-0.455, 0.26, CONSOLE_TOP))
TR = 0.055
box("black", "black", TILLER + Vector((0.0, 0.0, 0.004)), (0.12, 0.13, 0.008))
rod("metal", TILLER, TILLER + Vector((0.0, -0.01, 0.06)), 0.010, "metal")
lean = Euler((math.radians(-25.0), 0.0, 0.0))
hub = TILLER + Vector((0.0, -0.01, 0.065))
for i in range(12):
    a0, a1 = 2 * math.pi * i / 12, 2 * math.pi * (i + 1) / 12
    p0 = hub + lean.to_matrix() @ Vector((TR * math.cos(a0), TR * math.sin(a0), 0.0))
    p1 = hub + lean.to_matrix() @ Vector((TR * math.cos(a1), TR * math.sin(a1), 0.0))
    rod("black", p0, p1, 0.007, "black", n=6)
for a in (0.0, 2 * math.pi / 3, 4 * math.pi / 3):
    rod("metal", hub, hub + lean.to_matrix() @ Vector(((TR - 0.005) * math.cos(a), (TR - 0.005) * math.sin(a), 0.0)), 0.004, "metal", n=6)
knob_p = hub + lean.to_matrix() @ Vector((TR, 0.0, 0.0))
rod("dark", knob_p, knob_p + lean.to_matrix() @ Vector((0.0, 0.0, 0.035)), 0.007, "dark", n=6)

# --- overhead panel: hanging from the roof on the centre line, its face sloping down forward,
# rows of switch panels on it, the engine fire handles on a striped plate in its front row
oy0, oy1 = OVERHEAD_Y
oz0, oz1 = OVERHEAD_Z
ow = OVERHEAD_W / 2
r0 = min(roof(oy0, CABIN_X - ow), roof(oy0, CABIN_X + ow)) - SKIN_IN
r1 = min(roof(oy1, CABIN_X - ow), roof(oy1, CABIN_X + ow)) - SKIN_IN
prism("panel_grey", "panel_grey", [(oy0, r0), (oy1, r1), (oy1, oz1), (oy0, oz0)], CABIN_X - ow, CABIN_X + ow)
O_UP = Vector((0.0, oy1 - oy0, oz1 - oz0)).normalized()  # forward along the face
O_NORMAL = Vector((0.0, O_UP.z, -O_UP.y))  # down out of it
O_ROT = Matrix((RIGHT, -O_NORMAL, O_UP)).transposed().to_quaternion()
O_LENGTH = (Vector((oy1, oz1)) - Vector((oy0, oz0))).length


def o_point(x, s, offset=0.0):
    return Vector((CABIN_X + x, oy0, oz0)) + O_UP * s + O_NORMAL * offset


ROW = (O_LENGTH - 0.10) / 4
for k in range(4):
    s0 = 0.05 + k * ROW
    for i, x in enumerate((-0.165, 0.0, 0.165)):
        mesh_object("dark", box_mesh("dark", (0.155, 0.004, ROW - 0.02), "dark"), o_point(x, s0 + ROW / 2, 0.002), O_ROT)
        if k == 3 and i == 1:
            continue
        for j, dx in enumerate((-0.045, 0.0, 0.045)):
            for v in (0.3, 0.7):
                p = o_point(x + dx, s0 + ROW * v, 0.004)
                if (k + i + j) % 4 == 0:
                    mesh_object("metal", cylinder_mesh("metal", 0.008, 0.012, "metal"), p, O_ROT)
                else:
                    mesh_object("dark", cylinder_mesh("dark", 0.005, 0.004, "dark"), p, O_ROT)
                    lean_m = Matrix.Rotation(math.radians(-35.0 if (k + j) % 2 else 35.0), 3, "X")
                    me = box_mesh("metal", (0.004, 0.016, 0.004), "metal", (0.0, -0.011, 0.0), lean_m.to_quaternion())
                    mesh_object("metal", me, p + O_NORMAL * 0.003, O_ROT)
# The fire handles: four red T handles on a yellow and black striped plate.
s_fire = 0.05 + 3 * ROW + ROW / 2
mesh_object("black", box_mesh("black", (0.15, 0.003, ROW - 0.03), "black"), o_point(0.0, s_fire, 0.0055), O_ROT)
for k in range(9):
    me = box_mesh("yellow", (0.010, 0.002, ROW - 0.04), "yellow", rot=Euler((0.0, math.radians(35.0), 0.0)))
    mesh_object("yellow", me, o_point(-0.064 + 0.016 * k, s_fire, 0.0075), O_ROT)
for k in range(4):
    x = (k - 1.5) * 0.034
    mesh_object("dark", box_mesh("dark", (0.008, 0.012, 0.008), "dark"), o_point(x, s_fire, 0.014), O_ROT)
    mesh_object("red", box_mesh("red", (0.028, 0.012, 0.014), "red"), o_point(x, s_fire, 0.024), O_ROT)
for x in (-ow + 0.01, ow - 0.01):
    mesh_object("black", box_mesh("black", (0.02, 0.006, O_LENGTH), "black"), o_point(x, O_LENGTH / 2, 0.003), O_ROT)

# --- pedals: on a heel plate, either side of each pilot's centre line, their arms up into the
# footwell's ceiling
for x0 in (0.0, COPILOT_X):
    box("dark", "dark", (x0, PEDAL_Y - 0.16, FLOOR + 0.02), (0.40, 0.22, 0.04))
    for s in (-1, 1):
        p = Vector((x0 + s * PEDAL_X, PEDAL_Y, FLOOR + 0.16))
        box("dark", "dark", p, (0.08, 0.025, 0.22), Euler((math.radians(-20.0), 0.0, 0.0)))
        rod("metal", p + Vector((0, 0.03, 0.06)), Vector((x0 + s * PEDAL_X, 0.95, -0.70)), 0.010, "metal", n=6)


# --- seats: crew seats, grey fabric, armrests, on a pedestal; the copilot's the same
def seat(x0):
    y_front, y_back = 0.27, -0.22
    box("dark", "dark", (x0, 0.0, (FLOOR + SEAT_TOP - 0.10) / 2), (0.30, 0.36, SEAT_TOP - 0.10 - FLOOR))
    box("dark", "dark", (x0, 0.02, FLOOR + 0.02), (0.42, 0.56, 0.04))
    box("dark", "dark", (x0, (y_front + y_back) / 2, SEAT_TOP - 0.09), (0.50, y_front - y_back, 0.04))
    box("fabric", "fabric", (x0, (y_front + y_back) / 2 + 0.01, SEAT_TOP - 0.035), (0.48, y_front - y_back - 0.02, 0.07))
    # The backrest leaning back (its top aft) from the cushion's rear edge, the headrest on top
    # of it behind the head.
    lean = Euler((math.radians(10.0), 0.0, 0.0))
    tilt = lean.to_matrix()
    foot = Vector((x0, y_back + 0.06, SEAT_TOP - 0.02))
    back = foot + tilt @ Vector((0.0, 0.0, 0.38))
    box("fabric", "fabric", back, (0.48, 0.10, 0.76), lean)
    box("dark", "dark", back + tilt @ Vector((0.0, -0.06, 0.0)), (0.46, 0.03, 0.74), lean)
    box("fabric", "fabric", foot + tilt @ Vector((0.0, 0.0, 0.85)), (0.28, 0.10, 0.18), lean)
    # The armrests, folded down, on posts from the backrest.
    for s in (-1, 1):
        x = x0 + s * 0.27
        box("fabric", "fabric", (x, 0.02, -0.56), (0.06, 0.36, 0.05))
        box("dark", "dark", (x, -0.18, -0.64), (0.04, 0.06, 0.12))


seat(0.0)
seat(COPILOT_X)


# --- yokes: a column from the floor (a boot round its foot), a U-shaped wheel on top with a
# round hub badge; the pilot's moves (`yoke` with its `yoke_wheel`), the copilot's is the same
# model moved across
def yoke_parts(column, wheel, hub):
    """The column (in `column`'s frame from its pivot) and the wheel (in `wheel`'s frame from the
    hub)."""
    rod("dark", Vector((0.0, 0.0, 0.0)), hub + Vector((0.0, 0.03, -0.05)), 0.028, "dark", parent=column)
    rod("dark", hub + Vector((0.0, 0.03, -0.05)), hub + Vector((0.0, 0.06, 0.0)), 0.024, "dark", parent=column)
    # Wheel: the hub, a bar across under it, the horns up either side with their grips.
    mesh_object("dark", cylinder_mesh("dark", 0.045, 0.05, "dark", axis="-Y", centre=(0.0, 0.06, 0.0), n=12), (0, 0, 0), parent=wheel)
    mesh_object("metal", disc_mesh("metal", 0.03, "metal", y=0.0095, n=12), (0, 0, 0), parent=wheel)
    mesh_object("dark", disc_mesh("dark", 0.036, "dark", y=0.0098, n=12), (0, 0.0003, 0), parent=wheel)
    rod("dark", Vector((-WHEEL_SPAN, 0.02, -0.04)), Vector((WHEEL_SPAN, 0.02, -0.04)), 0.016, "dark", parent=wheel, n=8)
    for s in (-1, 1):
        rod("dark", Vector((s * 0.03, 0.02, -0.01)), Vector((s * WHEEL_SPAN, 0.02, -0.045)), 0.014, "dark", parent=wheel, n=8)
        rod("rubber", Vector((s * WHEEL_SPAN, 0.02, -0.05)), Vector((s * (WHEEL_SPAN + 0.015), 0.01, 0.10)), 0.019, "rubber", parent=wheel, n=10)
        mesh_object("dark", cylinder_mesh("dark", 0.006, 0.012, "dark", axis="Z", centre=(s * (WHEEL_SPAN + 0.017), 0.01, 0.10), n=8), (0, 0, 0), parent=wheel)
        mesh_object("red", box_mesh("red", (0.012, 0.010, 0.008), "red", (s * (WHEEL_SPAN + 0.005), -0.012, 0.075)), (0, 0, 0), parent=wheel)


yoke = empty("yoke", YOKE_PIVOT, props={"yoke": True})
wheel = empty("yoke_wheel", YOKE_HUB - YOKE_PIVOT, parent=yoke, props={"yoke_wheel": True})
yoke_parts(yoke, wheel, YOKE_HUB - YOKE_PIVOT)
for x0 in (0.0, COPILOT_X):
    box("rubber", "rubber", (x0, YOKE_PIVOT.y + 0.01, FLOOR + 0.03), (0.14, 0.18, 0.06))
    rod("rubber", Vector((x0, YOKE_PIVOT.y, FLOOR)), Vector((x0, YOKE_PIVOT.y, FLOOR + 0.12)), 0.045, "rubber", n=10)
bpy.context.view_layer.update()
shift = Matrix.Translation((COPILOT_X, 0.0, 0.0))
for o in [o for o in yoke.children_recursive if o.type == "MESH"]:
    c = bpy.data.objects.new("copilot_" + o.name, o.data)
    bpy.context.scene.collection.objects.link(c)
    c.parent = root
    c.matrix_parent_inverse = root.matrix_world.inverted()
    c.matrix_world = shift @ o.matrix_world

# --- HUDs: the projector over each pilot's head; the pilot's combiner hanging in front of the eye
# in a frame of bars, the arms up to the projector, the approach indexer beside it; the
# copilot's combiner folded up forward under the roof (its frame only)
p, size = PROJECTOR
for x0 in (0.0, COPILOT_X):
    box("dark", "dark", p + Vector((x0, 0.0, 0.0)), size)
    box("black", "black", p + Vector((x0, size.y / 2, -size.z / 2 + 0.02)), (size.x - 0.02, 0.01, 0.03))
hud = next(o for o in old if o.name.startswith("hud_glass"))
lo = Vector((min(v.co.x for v in hud.data.vertices), 0.0, min(v.co.z for v in hud.data.vertices)))
hi = Vector((max(v.co.x for v in hud.data.vertices), 0.0, max(v.co.z for v in hud.data.vertices)))
hud.location = (0.0, 0.0, 0.0)
hud.rotation_mode = "QUATERNION"
hud.rotation_quaternion = Quaternion()
for v in hud.data.vertices:
    tx, tz = (v.co.x - lo.x) / (hi.x - lo.x), (v.co.z - lo.z) / (hi.z - lo.z)
    v.co = Vector((-HUD_GLASS[0] + 2 * HUD_GLASS[0] * tx, HUD_Y, HUD_GLASS[1] + (HUD_GLASS[2] - HUD_GLASS[1]) * tz))
half_x, glass_lo, glass_hi = HUD_GLASS
post_x = half_x + HUD_BAR / 2
outline = [
    Vector((-post_x, glass_hi + HUD_BAR / 2)), Vector((-post_x, glass_lo - HUD_BAR / 2)),
    Vector((post_x, glass_lo - HUD_BAR / 2)), Vector((post_x, glass_hi + HUD_BAR / 2)),
]
for a, b in zip(outline, outline[1:] + outline[:1]):
    d = b - a
    c = (a + b) / 2
    rot = Euler((0.0, -math.atan2(d.y, d.x), 0.0))
    box("frame", "frame", (c.x, HUD_Y, c.y), (d.length + HUD_BAR, HUD_BAR, HUD_BAR), rot)
for s in (-1, 1):
    rod("frame", Vector((s * post_x, HUD_Y, glass_hi + HUD_BAR / 2)), p + Vector((s * (size.x / 2 - 0.01), size.y / 2 - 0.02, -size.z / 2)), 0.006, "frame", n=6)
# The copilot's: the frame swung up about the projector's front edge, lying under the roof.
hinge = p + Vector((COPILOT_X, size.y / 2 - 0.02, -size.z / 2))
folded = Euler((math.radians(-80.0), 0.0, 0.0)).to_matrix()
glass_mid = Vector((0.0, HUD_Y, (glass_lo + glass_hi) / 2)) + Vector((COPILOT_X, 0.0, 0.0))
for a, b in zip(outline, outline[1:] + outline[:1]):
    pa = Vector((COPILOT_X + a.x, HUD_Y, a.y)) - glass_mid
    pb = Vector((COPILOT_X + b.x, HUD_Y, b.y)) - glass_mid
    arm = glass_mid - hinge
    pa, pb = hinge + folded @ (arm + pa), hinge + folded @ (arm + pb)
    rod("frame", pa, pb, HUD_BAR / 2, "frame", n=6)
for s in (-1, 1):
    tip = hinge + folded @ (Vector((COPILOT_X + s * post_x, HUD_Y, glass_hi + HUD_BAR / 2)) - hinge)
    rod("frame", tip, hinge + Vector((s * (size.x / 2 - 0.01), 0.0, 0.0)), 0.006, "frame", n=6)
ind = next(o for o in old if o.name.startswith("approach"))
ind.data.transform(Matrix.Scale(0.55, 4))
ind.location = Vector((-(post_x + HUD_BAR / 2 + 0.009), HUD_Y, glass_hi - 0.02))
box("dark", "dark", (ind.location.x, HUD_Y + 0.004, ind.location.z), (0.016, 0.006, 0.038))

# --- flood lamps: one over each seat under the roof, one under the glare shield's lip
lamps = sorted((o for o in old if o.name.startswith("lamp")), key=lambda o: o.name)
for o, at in zip(lamps, (Vector((0.0, -0.30, 0.45)), Vector((COPILOT_X, -0.30, 0.45)), Vector((CABIN_X, LIP_Y - 0.05, TOP_EDGE.z - 0.03)))):
    o.location = at

bpy.data.orphans_purge(do_recursive=True)
bpy.context.view_layer.update()
print(f"throttle {THROTTLE_IDLE} -> {THROTTLE_FULL} about {THROTTLE_PIVOT}; yoke {YOKE_PIVOT} hub {YOKE_HUB}; "
      f"panel top {TOP_EDGE} half {top_half:.3f}, bottom half {panel_half(SIDE_BOTTOM):.3f}, centre bottom {low_mid}, "
      f"pedestal {PED_FRONT} -> {PED_REAR}; HUD combiner x +-{HUD_GLASS[0]}, z {HUD_GLASS[1]} .. {HUD_GLASS[2]} at y {HUD_Y}; "
      f"deck aft edge {[round(p.z, 3) for p in aft[::5]]}, sill {[round(p.y, 2) for p in fore[::5]]}")
bpy.ops.wm.save_as_mainfile(filepath=out)
print("built", out, "objects", len(bpy.data.objects))
