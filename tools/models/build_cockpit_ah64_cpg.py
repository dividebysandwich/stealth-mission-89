# Builds the AH-64D copilot/gunner's cockpit master (assets-src/blend/cockpit_ah64_cpg.blend) from
# the panel layout, the gunner's (front, lower) seat after the AH-64D's, with parts of the
# F-117A's modern cockpit and the UH-60's collective:
#   blender -b assets-src/blend/cockpit_modern.blend --python-exit-code 1 \
#       --python tools/models/build_cockpit_ah64_cpg.py -- \
#       assets-src/blend/cockpit_ah64_cpg_layout.json assets-src/blend/cockpit_ah64_cpg.blend
# then tools/models/export.sh cockpit_ah64_cpg. The layout JSON is written by
# `UPDATE_LAYOUT=1 cargo test layout_files` from `PanelLayout::ah64_cpg`
# (src/cockpit3d/layout.rs). The pilot's (rear) cockpit is tools/models/build_cockpit_ah64.py.
#
# Life size, as the AH-1Z's (tools/models/build_cockpit_ah1z.py, which this follows). Kept from
# the F-117A's cockpit: the seat with the ejection handle (no ejection: the handle stays), the
# stick (a side-stick cyclic on the right console), the throttle (the UH-60's collective on its
# arm from a pivot left of the seat), the five target buttons (on the left console), the HUD
# glass (a helmet display type: kept, low on the glareshield) and the
# flood lamps. Built here: a low panel leaning well back under the flat windshield (the gunner
# looks out over it) with three displays across it, the TEDAC's (the third MFD) between the two
# MFDs, the up-front display over the TEDAC under a shallow glareshield, the warning lights over
# the MFDs; under it the TEDAC's console with its two hand grips either side of the cautions and
# countermeasures; the side consoles, the floor and the pedals; the sill coamings and the dark
# bulkhead behind the gunner's head (the back of the pilot's panel). The canopy and its frames
# are the helicopter's own fuselage, drawn round the cockpit from inside
# (tools/models/apache.py, src/aircraft.rs).
#
# Blender frame: eye at the origin (on the cabin's centre line), x right, y forward, z up, metres.
# A control's face points along its local -Y (towards the gunner), local Z is up the panel.
import json
import math
import sys

import bmesh
import bpy
from mathutils import Euler, Matrix, Quaternion, Vector

layout_path, out = sys.argv[sys.argv.index("--") + 1:][:2]
layout = json.load(open(layout_path))
root = bpy.data.objects["cockpit"]
mats = bpy.data.materials

F117_SCALE = 1.4706
CABIN_X = 0.0
# The fuselage (tools/models/apache.py) in this frame: the flat windshield from the nose's top
# (y 0.9, z -0.22) leaning back to y 0.6, z 0.05 (half width 0.31 at z -0.2, 0.42 at z -0.3 at
# y 0.7); the sills falling from z -0.22 at the post (y -0.5) to -0.57 at the windshield; the
# sides (half width 0.52..0.58) straight down to the bays at z -1.0, the belly at z -1.33.
FLOOR = -1.17
# The main panel: its plane leaning back PANEL_TILT, the layout's origin at PANEL_CENTRE; its
# lower part (the MFDs) across PANEL_U, its upper part (the up-front display and the warning
# lights, above UPPER_V) across UPPER_U, TOP to BOTTOM (mm).
PANEL_TILT = math.radians(35.0)
PANEL_CENTRE = Vector((0.0, 0.56, -0.43))
PANEL_U = (-365.0, 365.0)
UPPER_U = (-365.0, 365.0)
GLARE_U = (-290.0, 290.0)
UPPER_V = -95.0
TOP, BOTTOM = -180.0, 172.0
# The glareshield over its top: its lip at HOOD_Y, its top RISE over the panel's top edge,
# forward to GLARE_FRONT (under the windshield). The HUD glass on it, scaled about the eye (the
# same angular size) to stand at HUD_Y, then shrunk about its foot to reach no higher than
# HUD_TOP (the windshield is 0.1 m over it).
HOOD_Y = 0.645
RISE = 0.04
GLARE_FRONT = 0.80
HUD_Y = 0.72
HUD_TOP = -0.14
# The TEDAC's console under the panel (layout x >= 1500, centred on 2000): its face sloping down
# aft from the panel's bottom edge, CONSOLE_V (mm) along it from CONSOLE_FRONT to CONSOLE_REAR;
# CONSOLE_W wide, SHELF thick, on a narrow pedestal to the floor. Its hand grips either side
# GRIP_U from the centre.
CONSOLE_U = 2000.0
CONSOLE_V = (15.0, 160.0)
CONSOLE_FRONT = Vector((CABIN_X, 0.45, -0.585))
CONSOLE_REAR = Vector((CABIN_X, 0.315, -0.64))
CONSOLE_W = 0.30
SHELF = 0.05
# The TEDAC: a deep box standing TEDAC_OUT out of the panel, its face (round the screen, mm:
# TEDAC_SIDE either side, TEDAC_TOP over it, TEDAC_BOTTOM under it) parallel to the panel; its
# hand grips on the box's sides, their heads GRIP_V down the panel.
TEDAC_OUT = 0.15
TEDAC_SIDE, TEDAC_TOP, TEDAC_BOTTOM = 40.0, 46.0, 40.0
GRIP_V = 118.0
TEDAC_KEY = TEDAC_OUT + 0.004
# Seat: its cushion's top (upright, the eye over the hips).
SEAT_TOP = -0.78
# Side consoles: inboard and outboard edges, aft and forward ends, top.
CONSOLE_X = (0.31, 0.47)
CONSOLE_Y = (-0.35, 0.44)
CONSOLE_TOP = -0.60
# Cyclic: a side stick, pivot on the right console, its grip raised on a short shaft.
CYCLIC_PIVOT = Vector((0.38, 0.14, CONSOLE_TOP))
CYCLIC_RAISE = 0.06
# Collective: handle (lowered) and pivot left of the seat, swing (degrees).
COLLECTIVE_HANDLE = Vector((-0.27, 0.12, -0.64))
COLLECTIVE_PIVOT = Vector((-0.27, -0.25, -0.72))
COLLECTIVE_SWING = 28.0
# Where the hand holds the collective's grip, in the grip's frame (mm, negated).
GRIP_HAND = Vector((0.0, -50.0, 4.0))
# The bulkhead behind the head (the back of the pilot's panel): its plane, its outline inside
# the canopy there (x, z; the roof at z 0.40, half width 0.39 at z 0.3, 0.50 at z 0, 0.58 under
# the sill at z -0.25).
BULKHEAD_Y = -0.42
BULKHEAD = [(0.55, FLOOR), (0.55, -0.30), (0.50, -0.10), (0.42, 0.10), (0.33, 0.27), (0.12, 0.35)]
# MFD bezels standing BEZEL out of the panel, their margins round the screen (mm).
BEZEL = 0.03
MFD_SIDE, MFD_TOP = 22.0, 25.0
PLATE = 0.004
ROUND = 16
KNOB = 8

RIGHT = Vector((1.0, 0.0, 0.0))
UP = Vector((0.0, math.sin(PANEL_TILT), math.cos(PANEL_TILT)))
NORMAL = Vector((0.0, -math.cos(PANEL_TILT), math.sin(PANEL_TILT)))
SLOPE = (CONSOLE_REAR - CONSOLE_FRONT).normalized()  # down the console
C_NORMAL = Vector((0.0, SLOPE.z, -SLOPE.y))  # up out of it


def console(u):
    return u >= 1500.0


def basis(u):
    """(across, normal towards the pilot, up) of the panel or the console."""
    return (RIGHT, C_NORMAL, -SLOPE) if console(u) else (RIGHT, NORMAL, UP)


def tedac_face():
    x, y, w, h = layout["screens"]["third"]
    return (x - TEDAC_SIDE, y - TEDAC_TOP, w + 2 * TEDAC_SIDE, h + TEDAC_TOP + TEDAC_BOTTOM)


def on_tedac(r):
    """The rectangle r (mm) is on the TEDAC's face."""
    x, y, w, h = tedac_face()
    return x <= r[0] + r[2] / 2 <= x + w and y <= r[1] + r[3] / 2 <= y + h


def point(u, v, offset=0.0):
    """A point of the panel or the console: layout millimetres (u right, v down), `offset` m
    off its surface."""
    if console(u):
        p = CONSOLE_FRONT + RIGHT * ((u - CONSOLE_U) / 1000) + SLOPE * ((v - CONSOLE_V[0]) / 1000)
        return p + C_NORMAL * offset
    return PANEL_CENTRE + RIGHT * (u / 1000) - UP * (v / 1000) + NORMAL * offset


def rotation(u):
    d, n, up = basis(u)
    return Matrix((d, -n, up)).transposed().to_quaternion()


def rect_point(r, offset=0.0):
    x, y, w, h = r
    return point(x + w / 2.0, y + h / 2.0, offset)


# --- keep, drop
KEEP = (
    "seat_", "cushion_", "metal_98", "metal_99", "control_Action_Eject_", "stick", "throttle",
    "hud_glass", "lamp",
    "button_NEAR", "button_NEXT", "button_NONE", "button_WPN", "button_DTL",
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


# --- building blocks (as the F-117A's)
def mesh_object(name, mesh, loc, rot=Euler((0, 0, 0)), parent=root, props=None):
    o = bpy.data.objects.new(name, mesh)
    bpy.context.scene.collection.objects.link(o)
    o.parent = parent
    o.rotation_mode = "QUATERNION"
    o.location = loc
    o.rotation_quaternion = rot if isinstance(rot, Quaternion) else rot.to_quaternion()
    for k, v in (props or {}).items():
        o[k] = v
    return o


def empty(name, loc, rot=Euler((0, 0, 0)), parent=root, props=None):
    o = bpy.data.objects.new(name, None)
    bpy.context.scene.collection.objects.link(o)
    o.parent = parent
    o.rotation_mode = "QUATERNION"
    o.location = loc
    o.rotation_quaternion = rot if isinstance(rot, Quaternion) else rot.to_quaternion()
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


def cylinder_mesh(name, r, h, mat, axis="-Y", n=KNOB):
    """A closed cylinder of radius r and height h from the origin along `axis` (local)."""
    me = bpy.data.meshes.new(name)
    bm = bmesh.new()
    rot = {"Z": Matrix.Identity(4), "-Y": Matrix.Rotation(math.radians(90.0), 4, "X")}[axis]
    m = rot @ Matrix.Translation((0.0, 0.0, h / 2))
    bmesh.ops.create_cone(bm, cap_ends=True, cap_tris=False, segments=n, radius1=r, radius2=r, depth=h, matrix=m)
    uv = bm.loops.layers.uv.new()
    for f in bm.faces:
        for l in f.loops:
            l[uv].uv = (0.0, 0.0)
    bm.to_mesh(me)
    bm.free()
    me.materials.append(mats[mat])
    return me


def outward(me):
    bm = bmesh.new()
    bm.from_mesh(me)
    bmesh.ops.recalc_face_normals(bm, faces=bm.faces[:])
    bm.to_mesh(me)
    bm.free()


def box(name, mat, centre, size, rot=Euler((0, 0, 0)), parent=root):
    return mesh_object(name, box_mesh(name, size, mat), centre, rot, parent=parent)


def rod(name, a, b, radius, mat, parent=root, n=8):
    """A cylinder from a to b (parent frame)."""
    a, b = Vector(a), Vector(b)
    me = bpy.data.meshes.new(name)
    bm = bmesh.new()
    bmesh.ops.create_cone(bm, cap_ends=True, segments=n, radius1=radius, radius2=radius, depth=(b - a).length)
    bm.to_mesh(me)
    bm.free()
    rot = Vector((0, 0, 1)).rotation_difference((b - a).normalized()).to_matrix().to_4x4()
    me.transform(Matrix.Translation((a + b) / 2) @ rot)
    for p in me.polygons:
        p.use_smooth = True
    me.materials.append(mats[mat])
    return mesh_object(name, me, (0, 0, 0), parent=parent)


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


def panel_box(name, mat, r, raise_m, margin=0.0, base=0.0):
    """A plate from `base` to `base + raise_m` off the panel over the rectangle r (mm) grown by
    `margin` mm."""
    x, y, w, h = r
    c = rect_point(r, base + raise_m / 2)
    return box(name, mat, c, ((w + 2 * margin) / 1000, abs(raise_m), (h + 2 * margin) / 1000), rotation(x + w / 2))


def frame(mat, outer, inner, d0, d1):
    """A rectangular frame (mm, outer and inner rectangles) from d0 to d1 m off the panel."""
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
    """A round black bezel standing `height` out of the panel at `offset` round the rectangle r's
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


def screen(mat, r, offset, behind=None, round=False):
    """A display over the rectangle r: a quad, or a disc in a bezel (the round instruments)."""
    x, y, w, h = r
    rot = rotation(x + w / 2)
    if round:
        bezel(r, offset, 0.010, 0.004)
        mesh_object(mat, disc_mesh(mat, min(w, h) / 2000, mat), rect_point(r, offset + 0.0105), rot, props={"no_shadow": True})
        return
    mesh_object(mat, quad_mesh(mat, w / 1000, h / 1000, mat), rect_point(r, offset), rot, props={"no_shadow": True})
    if behind:
        mesh_object(behind, quad_mesh(behind, w / 1000, h / 1000, behind), rect_point(r, offset - 0.003), rot, props={"no_shadow": True})


# --- the instrument panel, 3 cm thick, across the cabin
u0, u1 = PANEL_U
panel_box("panel_grey", "panel_grey", (u0, UPPER_V, u1 - u0, BOTTOM - UPPER_V), -0.03)
panel_box("panel_grey", "panel_grey", (UPPER_U[0], TOP, UPPER_U[1] - UPPER_U[0], UPPER_V - TOP + 1.0), -0.03)
# A rim along its bottom edge.
panel_box("dark", "dark", (u0, BOTTOM - 6.0, u1 - u0, 6.0), 0.008)

# --- glareshield: a deep hood over the panel's top edge, its lip reaching back towards the
# pilot, forward to the windshield
t = point(0.0, TOP)
glare_top = t.z + RISE
yz = [(t.y + 0.01, t.z - 0.01), (HOOD_Y, glare_top - 0.02), (HOOD_Y, glare_top - 0.008),
      (HOOD_Y + 0.015, glare_top), (GLARE_FRONT, glare_top + 0.01), (GLARE_FRONT, t.z - 0.01)]
prism("black", "black", yz, GLARE_U[0] / 1000, GLARE_U[1] / 1000)

# --- MFDs: a housing standing BEZEL out of the panel round each screen, the screen recessed in
# it; twenty labelled push buttons round it with ridges between them and rocker switches at the
# corners (as the F-16's).
scr = layout["screens"]
faces = {f["key"]: f["rect"] for f in layout["faces"]}
for key in ("left", "right"):
    x, y, w, h = scr[key]
    outer = (x - MFD_SIDE, y - MFD_TOP, w + 2 * MFD_SIDE, h + 2 * MFD_TOP)
    hole = (x - 3.0, y - 3.0, w + 6.0, h + 6.0)
    frame("dark", outer, hole, 0.0, BEZEL - 0.002)
    frame("black", (outer[0] + 1.0, outer[1] + 1.0, outer[2] - 2.0, outer[3] - 2.0), hole, BEZEL - 0.002, BEZEL)
    # The recess's back behind the targeting pod picture (3 mm behind the screen).
    frame("black", hole, (x - 0.5, y - 0.5, w + 1.0, h + 1.0), BEZEL - 0.016, BEZEL - 0.001)
    panel_box("black", "black", hole, 0.002, base=BEZEL - 0.016)
    for k in range(1, 5):
        for by in (y - 16.5, y + h + 6.5):
            panel_box("black", "black", (x + w * k / 5 - 1.0, by - 4.0, 2.0, 8.0), 0.003, base=BEZEL)
    for k in range(1, 5):
        for bx in (x - 9.0, x + w + 9.0):
            panel_box("black", "black", (bx - 4.0, y + h * k / 5 - 1.0, 8.0, 2.0), 0.003, base=BEZEL)
    for bx in (x - 10.0, x + w + 10.0):
        for by in (y - 16.0, y + h + 16.0):
            panel_box("dark", "dark", (bx - 4.5, by - 6.5, 9.0, 13.0), 0.003, base=BEZEL)
            panel_box("black", "black", (bx - 1.5, by - 5.0, 3.0, 10.0), 0.003, base=BEZEL + 0.003)
screen("screen_left", scr["left"], BEZEL - 0.008, "trackcam_left")
screen("screen_right", scr["right"], BEZEL - 0.008, "trackcam")


# --- the TEDAC (after the real unit): its box from inside the panel out to the face, a body
# under the face set back from it; the face's bezel round the screen, sunk in a sloping hood;
# the top keys and the side rockers in raised black housings, the knobs (LEV, GAIN; DAY / NT /
# OFF), the azimuth rocker and the square keys along the bottom, lines from these up the hood.
def chamfer(mat, outer, inner, d0, d1):
    """The sloping sides between the rectangle `outer` (mm) d0 m off the panel and `inner` d1
    off it, facing the gunner."""
    def corners(r, d):
        x, y, w, h = r
        return [point(x, y, d), point(x + w, y, d), point(x + w, y + h, d), point(x, y + h, d)]
    me = bpy.data.meshes.new(mat)
    bm = bmesh.new()
    vo = [bm.verts.new(p) for p in corners(outer, d0)]
    vi = [bm.verts.new(p) for p in corners(inner, d1)]
    for i in range(4):
        j = (i + 1) % 4
        bm.faces.new((vo[i], vo[j], vi[j], vi[i]))
    bm.normal_update()
    for f in bm.faces:
        if f.normal.dot(NORMAL) < 0:
            f.normal_flip()
    bm.to_mesh(me)
    bm.free()
    me.materials.append(mats[mat])
    return mesh_object(mat, me, (0, 0, 0))


def grown(r, m):
    x, y, w, h = r
    return (x - m, y - m, w + 2 * m, h + 2 * m)


tx, ty, tw, th = tedac_face()
sx_, sy_, sw_, sh_ = scr["third"]
panel_box("black", "black", (tx + 4.0, ty + 4.0, tw - 8.0, th - 8.0), TEDAC_OUT - 0.022 + 0.03, base=-0.03)
panel_box("black", "black", (tx + 10.0, ty + th - 4.0, tw - 20.0, 46.0), TEDAC_OUT - 0.03 + 0.03, base=-0.03)
hole = grown(scr["third"], 14.0)
frame("black", (tx, ty, tw, th), hole, TEDAC_OUT - 0.008, TEDAC_OUT)
chamfer("black", hole, grown(scr["third"], 1.0), TEDAC_OUT, TEDAC_OUT - 0.016)
screen("screen_third", scr["third"], TEDAC_OUT - 0.016, "trackcam_third")
keys = {int(f["key"].split(",")[1].strip(" )")): f["rect"] for f in layout["faces"] if f["key"].startswith("Osb(Third")}
for slot in (0, 1, 2, 3, 4, 5, 14, 19):
    panel_box("dark", "dark", grown(keys[slot], 2.5), TEDAC_KEY - TEDAC_OUT, base=TEDAC_OUT)
for a, b in ((10, 11), (12, 13), (15, 16), (17, 18)):
    x, y, w, h = keys[a]
    bottom = keys[b][1] + keys[b][3]
    panel_box("dark", "dark", (x - 3.0, y - 3.0, w + 6.0, bottom - y + 6.0), TEDAC_KEY - TEDAC_OUT, base=TEDAC_OUT)
for slot in (6, 7, 8, 9):
    x, y, w, h = keys[slot]
    panel_box("black", "black", grown(keys[slot], 1.5), TEDAC_KEY - TEDAC_OUT, base=TEDAC_OUT)
    # Its line up the hood to the screen's edge.
    u = x + w / 2
    rod("metal", point(u, sy_ + sh_ + 0.5, TEDAC_OUT - 0.0155), point(u, sy_ + sh_ + 14.0, TEDAC_OUT + 0.0003), 0.0006, "metal", n=4)
    rod("metal", point(u, sy_ + sh_ + 14.0, TEDAC_OUT + 0.0003), point(u, y - 1.0, TEDAC_OUT + 0.0003), 0.0006, "metal", n=4)
for u, v, r in ((sx_ - 24.0, sy_ - 28.0, 0.0065), (sx_ - 24.0, sy_ - 6.0, 0.0065), (sx_ + sw_ + 18.0, sy_ - 27.0, 0.007)):
    knob(u, v, r + 0.002, 0.004, "black", offset=TEDAC_OUT)
    knob(u, v, r, 0.016, "metal", offset=TEDAC_OUT + 0.004)
# The DAY / NT / OFF knob's pointer.
box("black", "black", point(sx_ + sw_ + 22.0, sy_ - 27.0, TEDAC_OUT + 0.0205), (0.008, 0.002, 0.002), rotation(0.0))
# Screws in the face's corners.
for u in (tx + 6.0, tx + tw - 6.0):
    for v in (ty + th - 8.0,):
        knob(u, v, 0.0025, 0.0015, "black", offset=TEDAC_OUT)

# --- plates under the instrument groups
plates = [
    ("black", (-78.0, -179.0, 156.0, 86.0)),  # up-front display over the TEDAC
    ("dark", (-363.0, -130.0, 76.0, 37.0)),  # warning lights over the left MFD
    ("dark", (287.0, -130.0, 76.0, 37.0)),  # and over the right one
    ("dark", (-213.0, -151.0, 131.0, 42.0)),  # FLAPS, BRAKE, BAY, AUTO, HOVER, HMD
    ("dark", (82.0, -171.0, 164.0, 78.0)),  # fuel, engine readouts, EMV
    ("dark", (1940.0, 25.0, 120.0, 128.0)),  # cautions and countermeasures
    ("dark", (1862.0, 32.0, 76.0, 76.0)),  # standby attitude indicator beside them
]
for mat, r in plates:
    panel_box(mat, mat, r, PLATE)
# The up-front display's knobs either side of it; switches on the countermeasure panel's side.
for u in (-70.0, 70.0):
    knob(u, -135.0, 0.005, 0.008, offset=PLATE)
for v in (85.0, 105.0, 125.0):
    toggle(2050.0, v, PLATE, up=(v != 105.0))


# Setting knobs at instrument corners.
def corner(r, inset=3.0):
    x, y, w, h = r
    return x + w - inset, y + h - inset


for r in (scr["adi"],):
    knob(*corner(r), 0.0045, 0.009, "dark", offset=PLATE)

screen("adi", scr["adi"], PLATE, round=True)
screen("ded", scr["ded"], PLATE + 0.0005)
for f in layout["faces"]:
    if f["control"] is None and on_tedac(f["rect"]):
        face(f["key"], f["rect"], TEDAC_KEY + 0.0005, f["round"])
    elif f["control"] is None:
        face(f["key"], f["rect"], PLATE + 0.0005, f["round"])
        if f["key"].startswith("Item(Dial(") and not f["round"]:
            # The tapes' black window frames.
            x, y, w, h = f["rect"]
            frame("black", (x - 3.0, y - 3.0, w + 6.0, h + 6.0), (x, y, w, h), 0.0, PLATE + 0.005)
    elif f["kind"] == "osb" and f["key"].startswith("Osb(Third"):
        control(f["key"], f["rect"], TEDAC_KEY, f["control"], "osb")
    elif f["kind"] == "osb":
        control(f["key"], f["rect"], BEZEL, f["control"], "osb")
    else:
        control(f["key"], f["rect"], PLATE, f["control"], f["kind"])

# --- the TEDAC's console: a shelf under the panel's bottom edge sloping down aft, on a narrow
# pedestal to the floor between the knees
half = CONSOLE_W / 2
f0 = CONSOLE_FRONT
f1 = CONSOLE_FRONT + SLOPE * ((CONSOLE_V[1] - CONSOLE_V[0]) / 1000)
b = point(CABIN_X * 1000, BOTTOM)
yz = [(b.y, b.z), (f0.y, f0.z - 0.002), (f1.y, f1.z - 0.002), (f1.y, f1.z - SHELF), (b.y + 0.03, b.z - SHELF - 0.03)]
prism("panel_grey", "panel_grey", yz, CABIN_X - half, CABIN_X + half)
prism("dark", "dark", [(f1.y + 0.03, f1.z - SHELF), (b.y + 0.03, b.z - SHELF - 0.03), (b.y + 0.03, FLOOR), (f1.y + 0.03, FLOOR)],
      CABIN_X - 0.07, CABIN_X + 0.07)


# --- the TEDAC's hand grips, on the box's sides: a mount plate, the head (its face to the
# gunner with the thumb switches round a socket) and the handle hanging from it, bending
# towards the gunner into a pointed foot; the trigger behind it.
def extrude(name, mat, front, back, shrink=0.9):
    """A slab from the outline `front` (points) to a copy moved by `back`, shrunk about its
    centre."""
    c = sum(front, Vector()) / len(front)
    rear = [c + (p - c) * shrink + back for p in front]
    me = bpy.data.meshes.new(name)
    n = len(front)
    f = [tuple(range(n)), tuple(range(n, 2 * n))[::-1]] + [(i, n + i, n + (i + 1) % n, (i + 1) % n) for i in range(n)]
    me.from_pydata(list(front) + rear, [], f)
    outward(me)
    me.materials.append(mats[mat])
    for p in me.polygons:
        p.use_smooth = False
    return mesh_object(name, me, (0, 0, 0))


def sweep(name, mat, path, sizes, n=10):
    """A tube along `path` (points), its elliptical sections (half width across, half depth)
    square to it, closed at both ends."""
    me = bpy.data.meshes.new(name)
    v = []
    for i, (p, (hw, hd)) in enumerate(zip(path, sizes)):
        t = (path[min(i + 1, len(path) - 1)] - path[max(i - 1, 0)]).normalized()
        d = t.cross(RIGHT).normalized()
        x = d.cross(t).normalized()
        for k in range(n):
            a = 2 * math.pi * k / n
            v.append(p + x * (hw * math.cos(a)) + d * (hd * math.sin(a)))
    f = [tuple(range(n))[::-1], tuple(range(n * (len(path) - 1), n * len(path)))]
    for s in range(len(path) - 1):
        for k in range(n):
            j = (k + 1) % n
            f.append((s * n + k, s * n + j, (s + 1) * n + j, (s + 1) * n + k))
    me.from_pydata(v, [], f)
    outward(me)
    for p in me.polygons:
        p.use_smooth = True
    me.materials.append(mats[mat])
    return mesh_object(name, me, (0, 0, 0))


GRIP_OFF = TEDAC_OUT - 0.035
# The head's outline (mm: out from the box's side, up), its socket and its thumb switches.
HEAD = [(4.0, -20.0), (4.0, 18.0), (16.0, 30.0), (40.0, 26.0), (58.0, 32.0), (82.0, 30.0), (104.0, 24.0), (118.0, 8.0),
        (116.0, -10.0), (102.0, -26.0), (84.0, -30.0), (62.0, -26.0), (40.0, -30.0), (18.0, -30.0)]
SWITCHES = [(22.0, 14.0), (52.0, 16.0), (82.0, 16.0), (106.0, 6.0), (24.0, -14.0), (82.0, -14.0), (104.0, -14.0)]
SOCKET = (54.0, -10.0)
# The handle's path (mm down the panel, out towards the gunner), HANDLE_SCALE of its drawn length.
HANDLE_SCALE = 0.75
HANDLE = [(d * HANDLE_SCALE, n * HANDLE_SCALE) for d, n in
          [(0.0, 0.0), (30.0, 0.0), (70.0, 3.0), (115.0, 8.0), (150.0, 18.0), (175.0, 38.0), (188.0, 62.0), (193.0, 86.0)]]
HANDLE_SIZE = [(30.0, 18.0), (24.0, 19.0), (20.0, 18.0), (21.0, 19.0), (19.0, 17.0), (14.0, 13.0), (8.0, 8.0), (3.0, 3.0)]
HANDLE_A = 66.0
side = tw / 2 - 4.0
for sx in (-1, 1):
    def at(a, b, off):
        return point(sx * (side + a), GRIP_V - b, off)
    rot = rotation(sx * side)
    box("dark", "dark", at(-1.0, 0.0, GRIP_OFF), (0.008, 0.06, 0.06), rot)
    box("dark", "dark", at(-1.0, 0.0, GRIP_OFF), (0.004, 0.064, 0.064), rot)
    extrude("tedac_grip", "black", [at(a, b, GRIP_OFF + 0.015) for a, b in HEAD], -NORMAL * 0.03)
    for a, b in SWITCHES:
        knob(sx * (side + a), GRIP_V - b, 0.0075, 0.003, "black", offset=GRIP_OFF + 0.015)
        knob(sx * (side + a), GRIP_V - b, 0.004, 0.006, "dark", offset=GRIP_OFF + 0.015)
    mesh_object("black", disc_mesh("black", 0.009, "black"), at(*SOCKET, GRIP_OFF + 0.0155), rot)
    top = Vector(at(HANDLE_A, -22.0, GRIP_OFF))
    path = [at(HANDLE_A, -22.0 - d, GRIP_OFF + n / 1000) for d, n in HANDLE]
    sweep("tedac_grip", "black", path, [(w / 1000, d / 1000) for w, d in HANDLE_SIZE])
    # The trigger behind the handle's top.
    box("black", "black", at(HANDLE_A, -62.0, GRIP_OFF - 0.018), (0.01, 0.012, 0.03), rot)
# --- side consoles, the target buttons on the left one (faces up), ahead of the collective
for sx in (-1, 1):
    x0, x1 = CONSOLE_X
    box("panel_grey", "panel_grey", (sx * (x0 + x1) / 2, sum(CONSOLE_Y) / 2, (CONSOLE_TOP + FLOOR) / 2),
        (x1 - x0, CONSOLE_Y[1] - CONSOLE_Y[0], CONSOLE_TOP - FLOOR))
    box("dark", "dark", (sx * (x0 + x1) / 2, sum(CONSOLE_Y) / 2, CONSOLE_TOP + 0.002), (x1 - x0 - 0.02, CONSOLE_Y[1] - CONSOLE_Y[0] - 0.02, 0.004))
up = Euler((math.radians(-90.0), 0.0, 0.0)).to_quaternion()
for i, n in enumerate(["button_NEAR", "button_NEXT", "button_NONE", "button_WPN", "button_DTL"]):
    o = objs[n]
    o.location = Vector((-0.41 + 0.06 * (i // 3), 0.30 + 0.035 * (i % 3), CONSOLE_TOP + 0.008))
    o.rotation_mode = "QUATERNION"
    o.rotation_quaternion = up

# --- floor, pedals
box("dark", "dark", (0.0, 0.15, FLOOR - 0.01), (0.90, 1.7, 0.02))
for s in (-1, 1):
    for x0 in (0.0,):
        p = Vector((x0 + s * 0.15, 0.72, FLOOR + 0.12))
        box("dark", "dark", p, (0.07, 0.025, 0.20), Euler((math.radians(-25.0), 0.0, 0.0)))
        rod("metal", p + Vector((0, 0.03, -0.08)), p + Vector((0, 0.10, -0.12)), 0.008, "metal")

# --- seat: upright, its cushion at SEAT_TOP, on a pedestal from the floor
seat = [o for o in old if o.name.startswith(SEAT)]
bpy.context.view_layer.update()
cushion = min((o for o in seat if o.name.startswith("cushion_")), key=lambda o: o.location.z)
cushion_top = max((cushion.matrix_world @ v.co).z for v in cushion.data.vertices)
for o in seat:
    o.location.z += SEAT_TOP - cushion_top
bpy.context.view_layer.update()
bottom = min((o.matrix_world @ v.co).z for o in seat if o.type == "MESH" for v in o.data.vertices)
box("dark", "dark", (0.0, -0.08, (bottom + FLOOR) / 2), (0.36, 0.40, bottom - FLOOR))

# --- cyclic on the right console
stick = objs["stick"]
stick.location = CYCLIC_PIVOT
for c in stick.children:
    c.location.z += CYCLIC_RAISE
stick["grip"] = float(stick["grip"]) + CYCLIC_RAISE
rod("stick_shaft", (0, 0, 0), (0, 0, CYCLIC_RAISE + 0.02), 0.012, "metal", stick)
# The boot at its foot (does not move with it).
rod("cyclic_boot", CYCLIC_PIVOT, CYCLIC_PIVOT + Vector((0, 0, 0.04)), 0.035, "rubber")

# --- collective: the arm from the pivot up to the grip, the UH-60's grip on it (after its
# drawings, mm; few sides to the round parts, it is seen small): a knurled ring on the arm's end, the hand grip, the switch box ahead of it
# with its face up (the hat, the rocker, the release button, the dimmer, the servo toggle, the
# landing light hat). Built in its own frame (ring's front at the origin, y along the grip, z
# up), then turned onto the arm with the hand at the lever's origin.
throttle = objs["throttle"]
for c in list(throttle.children):
    drop(c)
throttle.location = COLLECTIVE_HANDLE
arm = COLLECTIVE_PIVOT - COLLECTIVE_HANDLE
GRIP = Matrix.Rotation(math.atan2(-arm.z, -arm.y), 4, "X") @ Matrix.Scale(0.001, 4) @ Matrix.Translation(GRIP_HAND)


def grip_mat(name, rgba):
    m = mats.get(name) or mats.new(name)
    m.diffuse_color = rgba
    m.use_nodes = True
    bsdf = m.node_tree.nodes["Principled BSDF"]
    bsdf.inputs["Base Color"].default_value = rgba
    bsdf.inputs["Roughness"].default_value = 0.4
    return name


def grip(name, me, m=Matrix.Identity(4), smooth=False):
    """A grip part from its mesh in the grip frame (mm), moved by m first."""
    me.transform(GRIP @ m)
    for p in me.polygons:
        p.use_smooth = smooth
    return mesh_object(name, me, (0, 0, 0), parent=throttle)


def loft(name, mat, sections, n=16, rib=1.0):
    """Closed tube through elliptical sections (y, centre z, half width, half height); every
    other vertex at `rib` times the radius (knurling)."""
    me = bpy.data.meshes.new(name)
    v = []
    for y, z, hw, hh in sections:
        for i in range(n):
            a = 2 * math.pi * i / n
            k = rib if i % 2 else 1.0
            v.append((hw * k * math.cos(a), y, z + hh * k * math.sin(a)))
    f = [tuple(range(n))[::-1], tuple(range(n * (len(sections) - 1), n * len(sections)))]
    for s in range(len(sections) - 1):
        for i in range(n):
            j = (i + 1) % n
            f.append((s * n + i, s * n + j, (s + 1) * n + j, (s + 1) * n + i))
    me.from_pydata(v, [], f)
    outward(me)
    me.materials.append(mats[mat])
    return me


def slab(name, mat, top, z0, z1, shrink=1.0):
    """A plate under the outline `top` [(x, y)] from z1 down to z0, its bottom outline shrunk
    by `shrink` about its centre."""
    cx = sum(p[0] for p in top) / len(top)
    cy = sum(p[1] for p in top) / len(top)
    bottom = [(cx + (x - cx) * shrink, cy + (y - cy) * shrink, z0) for x, y in top]
    me = bpy.data.meshes.new(name)
    n = len(top)
    v = [(x, y, z1) for x, y in top] + bottom
    f = [tuple(range(n)), tuple(range(n, 2 * n))[::-1]]
    f += [(i, n + i, n + (i + 1) % n, (i + 1) % n) for i in range(n)]
    me.from_pydata(v, [], f)
    outward(me)
    me.materials.append(mats[mat])
    return me


def at(x, y, z, rz=0.0):
    return Matrix.Translation((x, y, z)) @ Matrix.Rotation(math.radians(rz), 4, "Z")


def stud(name, mat, r, h, m, n=KNOB):
    """A cylinder up the z of m (mm) standing on a surface: no bottom, smooth sides."""
    me = cylinder_mesh(name, r, h, mat, axis="Z", n=n)
    bm = bmesh.new()
    bm.from_mesh(me)
    bottom = [f for f in bm.faces if all(v.co.z < 1e-6 for v in f.verts)]
    bmesh.ops.delete(bm, geom=bottom, context="FACES_ONLY")
    bm.to_mesh(me)
    bm.free()
    o = grip(name, me, m)
    for p in o.data.polygons:
        p.use_smooth = len(p.vertices) == 4
    return o


def post(name, mat, x, y, z, r, h, n=KNOB):
    """A cylinder up from (x, y, z) mm."""
    stud(name, mat, r, h, at(x, y, z), n)


red = grip_mat("grip_red", (0.75, 0.02, 0.01, 1.0))
blue = grip_mat("grip_blue", (0.05, 0.45, 0.9, 1.0))
# The arm: from the ring's back to the pivot.
back = GRIP @ Vector((0.0, -19.0, 0.0))
rod("throttle_arm", back, arm, 0.016, "metal", throttle, n=6)
# Sides of the collective's round parts: large, small.
COLL_ROUND = 8
COLL_KNOB = 6
# Knurled ring, the hand grip narrowing to a waist and dropping onto the switch box's hub.
grip("throttle_ring", loft("throttle_ring", "rubber", [(-19.0, 0.0, 21.5, 21.5), (0.0, 0.0, 21.5, 21.5)], n=12, rib=0.94))
grip("throttle_grip", loft("throttle_grip", "dark", [
    (0.0, 0.0, 19.0, 19.0), (30.0, -2.0, 17.0, 19.0), (65.0, -6.0, 13.5, 16.0),
    (100.0, -11.0, 15.0, 15.0), (128.0, -14.0, 15.0, 10.0),
], n=COLL_ROUND), smooth=True)
# The two buttons on top and on the outboard side near the ring.
for y in (8.0, 38.0):
    stud("throttle_button", "metal", 3.5, 3.0, at(0.0, y, 18.0 - y / 15), COLL_KNOB)
    stud("throttle_button", "metal", 3.5, 3.0,
         at(-18.0 + y / 15, y, -2.0) @ Matrix.Rotation(math.radians(-90.0), 4, "Y"), COLL_KNOB)
# The web under the grip down to the box's back, the finger lever on it.
me = bpy.data.meshes.new("throttle_web")
web = [(55.0, -14.0), (98.0, -22.0), (98.0, -56.0), (86.0, -56.0)]
me.from_pydata([(x, y, z) for x in (-7.0, 7.0) for y, z in web], [], [
    (0, 1, 2, 3), (7, 6, 5, 4), (0, 4, 5, 1), (1, 5, 6, 2), (2, 6, 7, 3), (3, 7, 4, 0)])
outward(me)
me.materials.append(mats["dark"])
grip("throttle_web", me)
grip("throttle_lever", box_mesh("throttle_lever", (7.0, 6.0, 22.0), "black"),
     Matrix.Translation((9.0, 72.0, -40.0)) @ Matrix.Rotation(math.radians(-25.0), 4, "Y"))
# The switch box: its top outline (x across, inboard +, y along), the black face on it.
face_top = [(-44.0, 125.0), (-44.0, 178.0), (-38.0, 182.0), (6.0, 182.0), (53.0, 151.0), (53.0, 102.0), (35.0, 97.0)]
FACE = -15.0
grip("throttle_box", slab("throttle_box", "dark", face_top, -60.0, FACE - 2.0, 0.9))
inset = [(x * 0.97 + 0.5, (y - 140.0) * 0.97 + 140.0) for x, y in face_top]
grip("throttle_face", slab("throttle_face", "black", inset, FACE - 2.0, FACE))
# The hat on its collar where the grip meets the box.
post("throttle_collar", "metal", 0.0, 136.0, FACE, 15.0, 8.0, n=COLL_ROUND)
post("throttle_hat", "black", 0.0, 136.0, FACE + 8.0, 9.0, 8.0, n=COLL_ROUND)
post("throttle_hat", "black", 0.0, 136.0, FACE + 16.0, 4.0, 3.0, n=COLL_KNOB)
# The ribbed rocker in its housing (outboard, aft).
grip("throttle_rocker", box_mesh("throttle_rocker", (16.0, 18.0, 4.0), "dark", (-28.0, 140.0, FACE + 2.0)))
grip("throttle_rocker", loft("throttle_rocker", "black", [(-6.0, 0.0, 7.0, 7.0), (6.0, 0.0, 7.0, 7.0)], n=8, rib=0.88),
     Matrix.Translation((-28.0, 140.0, FACE + 3.0)))
# The release button: red, in its blue ring (outboard, ahead).
post("throttle_ring_lit", blue, -22.0, 165.0, FACE, 12.0, 1.5, n=COLL_ROUND)
post("throttle_release", red, -22.0, 165.0, FACE + 1.5, 8.5, 6.0, n=COLL_ROUND)
# The dimmer knob, star cut.
post("throttle_dimmer", "dark", 12.0, 167.0, FACE, 9.5, 3.0, n=COLL_ROUND)
grip("throttle_dimmer", loft("throttle_dimmer", "black", [(0.0, 0.0, 8.0, 8.0), (8.0, 0.0, 8.0, 8.0)], n=8, rib=0.75),
     Matrix.Translation((12.0, 167.0, FACE + 3.0)) @ Matrix.Rotation(math.radians(90.0), 4, "X"))
# The servo toggle in its slanted slot.
grip("throttle_slot", box_mesh("throttle_slot", (9.0, 26.0, 4.0), "dark", (30.0, 147.0, FACE + 2.0), Euler((0.0, 0.0, math.radians(-35.0)))))
grip("throttle_toggle", box_mesh("throttle_toggle", (3.5, 4.0, 12.0), "metal", (30.0, 147.0, FACE + 8.0), Euler((0.0, math.radians(15.0), 0.0))))
# The landing light hat (inboard, aft).
post("throttle_ldg", "dark", 39.0, 118.0, FACE, 9.5, 4.0, n=COLL_ROUND)
post("throttle_ldg", "black", 39.0, 118.0, FACE + 4.0, 6.5, 6.0, n=COLL_ROUND)
box("dark", "dark", COLLECTIVE_PIVOT + Vector((0, 0, -0.03)), (0.07, 0.07, 0.09))
box("dark", "dark", COLLECTIVE_PIVOT + Vector((0, 0, (FLOOR - COLLECTIVE_PIVOT.z) / 2 - 0.04)), (0.05, 0.05, COLLECTIVE_PIVOT.z - FLOOR - 0.08))
full = COLLECTIVE_PIVOT + Matrix.Rotation(math.radians(COLLECTIVE_SWING), 3, "X") @ -arm
objs["throttle_full"].location = full
empty("throttle_pivot", COLLECTIVE_PIVOT, props={"anchor": "throttle_pivot"})

# --- HUD: the glass on the glareshield in front of the gunner, its foot on it and shrunk to
# stay under the windshield. No frame round it: the gunner's symbols are a helmet display's, the
# glass only carries them.
hud = next(o for o in old if o.name.startswith("hud_glass"))
bpy.context.view_layer.update()
glass = [hud.matrix_world @ v.co for v in hud.data.vertices]
k = HUD_Y / (sum(v.y for v in glass) / len(glass))
hud.location *= k
hud.data.transform(Matrix.Scale(k, 4))
bpy.context.view_layer.update()
glass = [hud.matrix_world @ v.co for v in hud.data.vertices]
low = min(v.z for v in glass)
foot = glare_top + 0.012
k = min(1.0, (HUD_TOP - foot) / (max(v.z for v in glass) - low))
shrink = Matrix.Translation((0.0, 0.0, foot)) @ Matrix.Diagonal((k, 1.0, k, 1.0)) @ Matrix.Translation((0.0, 0.0, -low))
hud.data.transform(hud.matrix_world.inverted() @ shrink @ hud.matrix_world)
bpy.context.view_layer.update()
glass = [hud.matrix_world @ v.co for v in hud.data.vertices]
print(f"HUD glass shrunk {k:.2f}: foot {foot:.3f}")
# No approach indexer: a helicopter has no angle of attack to fly the approach by.
# --- flood lamps: two behind the pilot's head, one under the glareshield
objs["lamp.002"].location = Vector((0.0, HOOD_Y + 0.04, t.z))

# --- the sill coamings along the bottom of the side windows, inside the skin, from the post
# down to the windshield
SILL = [(0.555, -0.40, -0.28), (0.55, -0.10, -0.37), (0.535, 0.40, -0.50), (0.48, 0.78, -0.58)]
for sx in (-1, 1):
    for (x0, y0, z0), (x1, y1, z1) in zip(SILL, SILL[1:]):
        rod("dark", (sx * x0, y0, z0), (sx * x1, y1, z1), 0.02, "dark", n=6)

# --- the bulkhead behind the gunner's head: dark, from the floor to under the roof
me = bpy.data.meshes.new("dark")
outline = BULKHEAD + [(-x, z) for x, z in reversed(BULKHEAD)]
n = len(outline)
v = [(x, y, z) for y in (BULKHEAD_Y, BULKHEAD_Y - 0.02) for x, z in outline]
f = [tuple(range(n)), tuple(range(n, 2 * n))[::-1]] + [(i, (i + 1) % n, n + (i + 1) % n, n + i) for i in range(n)]
me.from_pydata(v, [], f)
outward(me)
me.materials.append(mats["dark"])
mesh_object("dark", me, (0, 0, 0))

bpy.data.orphans_purge(do_recursive=True)
bpy.context.view_layer.update()
print(f"cyclic grip {stick['grip']:.3f} m; collective {COLLECTIVE_HANDLE} -> {full} "
      f"({(full - COLLECTIVE_HANDLE).length:.3f} m); glareshield top {glare_top:.3f}; seat bottom {bottom:.3f}")
bpy.ops.wm.save_as_mainfile(filepath=out)
print("built", out, "objects", len(bpy.data.objects))
