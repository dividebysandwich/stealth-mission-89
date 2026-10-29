# Helpers shared by the ground console builders (build_console.py, build_console_drone.py): the
# Blender scene with its cockpit root, the shared materials, boxes and quads, console surfaces,
# screens and push buttons, and the shelter, chair and hand controller every console has.
#
# Blender frame: eye at the origin, x right, y forward, z up, metres. A control's face points
# along its local -Y (towards the operator), local Z is up its panel.
import math

import bmesh
import bpy
from mathutils import Euler, Matrix, Vector

# Push buttons: face half size, face height over the panel, travel.
BUTTON = (0.024, 0.011)
HEIGHT, TRAVEL = 0.006, 0.004

scene = None
root = None
mats = None


def setup(screens):
    """A new scene with the cockpit root, the shared materials and the screens' (and the camera
    picture's) materials."""
    global scene, root, mats
    bpy.ops.wm.read_factory_settings(use_empty=True)
    scene = bpy.context.scene
    root = bpy.data.objects.new("cockpit", None)
    scene.collection.objects.link(root)
    for name in ("panel_grey", "dark", "black", "frame", "seat", "cushion", "rubber", "metal",
                 "highlight", "trackcam", *screens):
        material(name)
    material("red", (0.6, 0.05, 0.05))
    mats = bpy.data.materials


def material(name, rgb=(0.1, 0.1, 0.1)):
    m = bpy.data.materials.get(name) or bpy.data.materials.new(name)
    m.diffuse_color = (*rgb, 1.0)
    m.use_nodes = True
    bsdf = m.node_tree.nodes.get("Principled BSDF")
    if bsdf:
        bsdf.inputs["Base Color"].default_value = (*rgb, 1.0)
    return m


def link(o, parent, loc, rot, props):
    scene.collection.objects.link(o)
    o.parent = parent or root
    o.rotation_mode = "QUATERNION"
    o.location = loc
    o.rotation_quaternion = rot.to_quaternion()
    for k, v in (props or {}).items():
        o[k] = v
    return o


def mesh_object(name, mesh, loc, rot=Euler((0, 0, 0)), parent=None, props=None):
    return link(bpy.data.objects.new(name, mesh), parent, loc, rot, props)


def empty(name, loc, rot=Euler((0, 0, 0)), parent=None, props=None):
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


def box(name, mat, centre, size, rot=Euler((0, 0, 0)), parent=None):
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


def shelter(room_x, back, front, floor, ceiling, wall=0.03):
    """The shelter's walls, floor and ceiling (their inner faces), cable trays and racks along the
    walls, a red dome light over the console."""
    ROOM_X, ROOM_BACK, ROOM_FRONT, FLOOR, CEILING, WALL = room_x, back, front, floor, ceiling, wall
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
    empty("lamp", (0.0, 0.25, CEILING - 0.05), props={"lamp": [0.9, 0.08, 0.05], "range": 4.0})
    box("metal", "metal", (0.0, 0.25, CEILING - 0.02), (0.16, 0.16, 0.03))
    box("red", "red", (0.0, 0.25, CEILING - 0.04), (0.12, 0.12, 0.01))


def chair():
    """The operator's chair under the eye."""
    box("seat", "seat", (0.0, -0.05, -0.80), (0.5, 0.48, 0.10))
    box("cushion", "cushion", (0.0, -0.05, -0.74), (0.46, 0.44, 0.03))
    box("seat", "seat", (0.0, -0.30, -0.42), (0.48, 0.08, 0.66))
    box("metal", "metal", (0.0, -0.05, -1.05), (0.06, 0.06, 0.40))


def hand_controller(STICK, FLOOR):
    """A pedestal from the floor, a boot on its top, the stick over the pivot `STICK`: a shaft
    and a grip leaning forward, a trigger in front, a thumb button on top."""
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
