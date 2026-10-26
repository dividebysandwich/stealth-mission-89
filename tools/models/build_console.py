# Builds the air defence console master (assets-src/blend/cockpit_console.blend): the operator's
# seat in the dark shelter of a Patriot battery's engagement control station (also used for the
# Gepard's), from nothing:
#   blender -b --factory-startup --python-exit-code 1 --python tools/models/build_console.py -- \
#       assets-src/blend/cockpit_console.blend
# then tools/models/export.sh cockpit_console. See src/station/mod.rs and src/cockpit3d/model.rs
# for what the game binds to.
#
# Built: the shelter (walls, floor, ceiling, a dim red dome light), the operator's chair, the
# console: an upright panel with the camera screen (`trackcam` picture behind the `screen_eo`
# overlay) and its buttons, a sloping desk with the air picture (`screen_scope`), the track list
# (`screen_board`) and button columns beside them. Each is one flat surface, placed so a 65
# degree desktop view sees them all. Right of the camera's buttons a screen with the locked
# target's data (`screen_info`) turned towards the operator. Right of the chair a pedestal with the hand controller
# (`stick`) that slews the Gepard's turret, its trigger in front.
#
# Blender frame: eye at the origin, x right, y forward, z up, metres. A control's face points
# along its local -Y (towards the operator), local Z is up its panel.
import math
import sys

import bmesh
import bpy
from mathutils import Euler, Matrix, Vector

out = sys.argv[sys.argv.index("--") + 1]

# Shelter: inside walls (x left/right, y back/front, z floor/ceiling).
ROOM_X = 1.25
ROOM_BACK, ROOM_FRONT = -1.1, 1.15
FLOOR, CEILING = -1.25, 0.75
WALL = 0.03
# The upright panel: a vertical surface this far ahead of the eye, its centre this high.
PANEL_Y = 0.90
PANEL_Z = 0.20
# The desk: one flat surface sloping back DESK_TILT from the vertical (degrees), its centre
# DESK_D from the eye at DESK_EL degrees below the eye's level.
DESK_D = 0.70
DESK_EL = -20.0
DESK_TILT = 40.0
# The camera screen (77:60, the tracking camera's picture), the air picture, the track list, the
# target data (4:3) (m).
EO = (0.46, 0.46 * 60.0 / 77.0)
INFO = (0.24, 0.18)
# The target data screen's centre and its turn towards the operator (degrees).
INFO_AT = (0.58, PANEL_Y - 0.08, PANEL_Z)
INFO_TURN = -30.0
SCOPE = (0.28, 0.28)
BOARD = (0.34, 0.255)
# The hand controller's pivot, on its pedestal right of the chair (its grip GRIP_HEIGHT,
# src/cockpit3d/levers.rs, above it).
STICK = Vector((0.34, 0.20, -0.50))
# Push buttons: face half size, face height over the panel, travel.
BUTTON = (0.024, 0.011)
HEIGHT, TRAVEL = 0.006, 0.004

bpy.ops.wm.read_factory_settings(use_empty=True)
scene = bpy.context.scene
root = bpy.data.objects.new("cockpit", None)
scene.collection.objects.link(root)


def material(name, rgb=(0.1, 0.1, 0.1)):
    m = bpy.data.materials.get(name) or bpy.data.materials.new(name)
    m.diffuse_color = (*rgb, 1.0)
    m.use_nodes = True
    bsdf = m.node_tree.nodes.get("Principled BSDF")
    if bsdf:
        bsdf.inputs["Base Color"].default_value = (*rgb, 1.0)
    return m


# The shared cockpit materials (the game's own), the screens, the camera picture.
for name in ("panel_grey", "dark", "black", "frame", "seat", "cushion", "rubber", "metal",
             "highlight", "trackcam", "screen_scope", "screen_board", "screen_eo", "screen_info"):
    material(name)
material("red", (0.6, 0.05, 0.05))
mats = bpy.data.materials


def link(o, parent, loc, rot, props):
    scene.collection.objects.link(o)
    o.parent = parent
    o.rotation_mode = "QUATERNION"
    o.location = loc
    o.rotation_quaternion = rot.to_quaternion()
    for k, v in (props or {}).items():
        o[k] = v
    return o


def mesh_object(name, mesh, loc, rot=Euler((0, 0, 0)), parent=root, props=None):
    return link(bpy.data.objects.new(name, mesh), parent, loc, rot, props)


def empty(name, loc, rot=Euler((0, 0, 0)), parent=root, props=None):
    return link(bpy.data.objects.new(name, None), parent, loc, rot, props)


def box_mesh(name, size, mat, centre=(0.0, 0.0, 0.0)):
    me = bpy.data.meshes.new(name)
    bm = bmesh.new()
    m = Matrix.Translation(centre) @ Matrix.Diagonal((*size, 1.0))
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


def box(name, mat, centre, size, rot=Euler((0, 0, 0)), parent=root):
    return mesh_object(name, box_mesh(name, size, mat), centre, rot, parent=parent)


class Surface:
    """A flat console surface: its centre and rotation (its -Y faces the operator, local Z is
    up the surface); parts are placed on it at (u right, v up) metres from its centre."""

    def __init__(self, centre, rot):
        self.centre, self.rot = Vector(centre), rot

    def at(self, u, v, out=0.0):
        """A point on the surface, `out` metres in front of it."""
        return self.centre + self.rot.to_matrix() @ Vector((u, -out, v))


def screen(surface, mat, u, v, size, behind=None):
    """A display on a surface in a dark bezel; the camera picture 3 mm behind it."""
    p, rot = surface.at(u, v, 0.012), surface.rot
    w, h = size
    mesh_object(mat, quad_mesh(mat, w, h, mat), p, rot, props={"no_shadow": True})
    if behind:
        mesh_object(behind, quad_mesh(behind, w, h, behind), surface.at(u, v, 0.009), rot,
                    props={"no_shadow": True})
    # The bezel: a frame round the screen, standing out of the surface.
    bezel = empty("bezel", surface.at(u, v), rot)
    m = 0.018
    for (cx, cz, sx, sz) in ((0, h / 2 + m / 2, w + 2 * m, m), (0, -h / 2 - m / 2, w + 2 * m, m),
                             (w / 2 + m / 2, 0, m, h), (-w / 2 - m / 2, 0, m, h)):
        box("dark", "dark", (cx, -0.01, cz), (sx, 0.02, sz), parent=bezel)
    box("black", "black", (0, -0.004, 0), (w + 2 * m, 0.008, h + 2 * m), parent=bezel)


def button(surface, effect, label, u, v):
    """A labelled push button on a surface: origin on it, its face HEIGHT out."""
    p, rot = surface.at(u, v), surface.rot
    hw, hh = BUTTON
    name = "control_" + "".join(ch if ch.isalnum() else "_" for ch in effect)
    c = empty(name, p, rot, props={"control": effect, "kind": "side", "half": [hw, hh],
                                   "height": HEIGHT, "travel": TRAVEL})
    moving = empty("moving", (0, 0, 0), parent=c, props={"moving": True})
    lab = "label_" + label
    material(lab, (0.1, 0.4, 0.1))
    mesh_object(lab, quad_mesh(lab, 2 * hw, 2 * hh, lab, -HEIGHT), (0, 0, 0), parent=moving,
                props={"no_shadow": True})
    mesh_object("dark", box_mesh("dark", (2 * hw, HEIGHT - 0.0004, 2 * hh), "dark",
                                 (0, -(HEIGHT - 0.0004) / 2, 0)), (0, 0, 0), parent=moving)
    hl = empty("highlight", (0, 0, 0), parent=c, props={"highlight": True})
    t = 0.002
    for (cx, cz, sx, sz) in ((0, hh + t, 2 * hw + 2 * t, t), (0, -hh - t, 2 * hw + 2 * t, t),
                             (hw + t, 0, t, 2 * hh), (-hw - t, 0, t, 2 * hh)):
        mesh_object("highlight", box_mesh("highlight", (sx, t, sz), "highlight"),
                    (cx, -HEIGHT, cz), parent=hl, props={"no_shadow": True})
    # Its well in the surface.
    box("black", "black", surface.at(u, v, 0.001), (2 * hw + 0.008, 0.004, 2 * hh + 0.008), rot)


def column(surface, u, items):
    """Buttons down a column: (effect, label, v)."""
    for effect, label, v in items:
        button(surface, effect, label, u, v)


# --- the shelter: walls, floor and ceiling (their inner faces), a dome light
box("black", "black", (0, ROOM_FRONT + WALL / 2, (FLOOR + CEILING) / 2), (2 * ROOM_X, WALL, CEILING - FLOOR))
box("black", "black", (0, ROOM_BACK - WALL / 2, (FLOOR + CEILING) / 2), (2 * ROOM_X, WALL, CEILING - FLOOR))
for s in (-1, 1):
    box("black", "black", (s * (ROOM_X + WALL / 2), (ROOM_BACK + ROOM_FRONT) / 2, (FLOOR + CEILING) / 2),
        (WALL, ROOM_FRONT - ROOM_BACK, CEILING - FLOOR))
box("rubber", "rubber", (0, (ROOM_BACK + ROOM_FRONT) / 2, FLOOR - WALL / 2), (2 * ROOM_X, ROOM_FRONT - ROOM_BACK, WALL))
box("black", "black", (0, (ROOM_BACK + ROOM_FRONT) / 2, CEILING + WALL / 2), (2 * ROOM_X, ROOM_FRONT - ROOM_BACK, WALL))
# Cable trays and an equipment rack along the walls.
for s in (-1, 1):
    box("frame", "frame", (s * (ROOM_X - 0.06), 0.0, CEILING - 0.08), (0.10, ROOM_FRONT - ROOM_BACK - 0.1, 0.05))
    box("dark", "dark", (s * (ROOM_X - 0.25), -0.75, -0.35), (0.48, 0.55, 1.8))
    for k in range(6):
        box("black", "black", (s * (ROOM_X - 0.25), -0.75 + 0.28, -1.0 + k * 0.28), (0.42, 0.01, 0.2))
# The dome light: a red lamp over the console (the local lights option).
empty("lamp", (0.0, 0.25, CEILING - 0.05), props={"lamp": [0.9, 0.08, 0.05], "range": 3.0})
box("metal", "metal", (0.0, 0.25, CEILING - 0.02), (0.16, 0.16, 0.03))

# --- the chair under the eye
box("seat", "seat", (0.0, -0.05, -0.80), (0.5, 0.48, 0.10))
box("cushion", "cushion", (0.0, -0.05, -0.74), (0.46, 0.44, 0.03))
box("seat", "seat", (0.0, -0.30, -0.42), (0.48, 0.08, 0.66))
box("metal", "metal", (0.0, -0.05, -1.05), (0.06, 0.06, 0.40))

# --- the hand controller: a pedestal from the floor, a boot on its top, the stick over the
# pivot: a shaft and a grip leaning forward, a trigger in front, a thumb button on top
box("dark", "dark", (STICK.x, STICK.y, (FLOOR + STICK.z - 0.03) / 2), (0.14, 0.22, STICK.z - 0.03 - FLOOR))
box("frame", "frame", (STICK.x, STICK.y, STICK.z - 0.025), (0.15, 0.23, 0.01))
box("rubber", "rubber", (STICK.x, STICK.y, STICK.z - 0.012), (0.06, 0.06, 0.016))
stick = empty("stick", STICK, props={"stick": True})
box("metal", "metal", (0.0, 0.0, 0.03), (0.016, 0.016, 0.06), parent=stick)
lean = Euler((math.radians(-12.0), 0.0, 0.0))
box("panel_grey", "panel_grey", (0.0, 0.008, 0.11), (0.036, 0.044, 0.11), lean, parent=stick)
box("panel_grey", "panel_grey", (0.0, 0.016, 0.175), (0.04, 0.05, 0.03), lean, parent=stick)
box("metal", "metal", (0.0, 0.036, 0.125), (0.012, 0.012, 0.026), lean, parent=stick)
box("red", "red", (0.0, 0.02, 0.192), (0.014, 0.014, 0.008), parent=stick)

# --- the console: the upright panel and the desk, each one flat surface, where they meet,
# and the body down to the floor
panel = Surface((0.0, PANEL_Y, PANEL_Z), Euler((0.0, 0.0, 0.0)))
e = math.radians(DESK_EL)
desk = Surface((0.0, DESK_D * math.cos(e), DESK_D * math.sin(e)),
               Euler((-math.radians(DESK_TILT), 0.0, 0.0)))
box("dark", "dark", panel.at(0.0, 0.0, -0.02), (1.5, 0.04, 0.58), panel.rot)
box("dark", "dark", desk.at(0.0, 0.0, -0.02), (1.5, 0.04, 0.46), desk.rot)
top, foot = desk.at(0.0, 0.23, -0.02), panel.at(0.0, -0.29)
box("black", "black", ((top + foot) / 2), (1.5, abs(foot.y - top.y) + 0.04, 0.03))
front = desk.at(0.0, -0.23, -0.02)
box("black", "black", (0.0, front.y + 0.12, (front.z + FLOOR) / 2), (1.5, 0.24, front.z - FLOOR))
box("frame", "frame", desk.at(0.0, -0.235, 0.0), (1.52, 0.03, 0.03), desk.rot)

# --- the camera screen; the leave button and the launchers' block left of it, the camera's
# buttons right of it
screen(panel, "screen_eo", 0.0, 0.0, EO, behind="trackcam")
column(panel, -0.44, [("Console(Leave)", "LEAVE", 0.16)])
for k in (1, 2, 3):
    v = 0.06 - (k - 1) * 0.05
    column(panel, -0.44, [(f"Console(Launcher, {k})", f"L{k}", v)])
    column(panel, -0.34, [(f"Console(Refill, {k})", f"RFL{k}", v)])
column(panel, 0.36, [
    ("Console(Flir)", "FLIR", 0.16),
    ("Console(ZoomIn)", "ZOOM+", 0.09),
    ("Console(ZoomOut)", "ZOOM-", 0.04),
])

# --- the target data screen in its own housing, turned towards the operator
info = Surface(INFO_AT, Euler((0.0, 0.0, math.radians(INFO_TURN))))
box("dark", "dark", info.at(0.0, 0.0, -0.03), (INFO[0] + 0.07, 0.06, INFO[1] + 0.07), info.rot)
screen(info, "screen_info", 0.0, 0.0, INFO)

# --- the desk: the air picture left, the track list right, button columns outside them
screen(desk, "screen_scope", -0.18, 0.0, SCOPE)
screen(desk, "screen_board", 0.21, 0.0, BOARD)
column(desk, -0.43, [
    ("Console(RangeOut)", "RNG+", 0.10),
    ("Console(RangeIn)", "RNG-", 0.06),
    ("Console(Tracks)", "TRACKS", -0.02),
    ("Console(Site)", "SITE", -0.06),
])
column(desk, 0.50, [
    ("Console(Lock)", "LOCK", 0.10),
    ("Console(Drop)", "DROP", 0.06),
    ("Console(Guided)", "GUIDED", -0.01),
    ("Console(Semi)", "SEMI", -0.05),
    ("Console(Fire)", "FIRE", -0.12),
])

bpy.ops.wm.save_as_mainfile(filepath=out)
