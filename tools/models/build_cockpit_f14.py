# Builds the F-14B Tomcat's two cockpit masters from the panel layouts, with parts of the F-117A's
# modern cockpit: the pilot's (front seat)
#   blender -b assets-src/blend/cockpit_base.blend --python-exit-code 1 \
#       --python tools/models/build_cockpit_f14.py -- \
#       assets-src/blend/cockpit_f14_layout.json assets-src/blend/cockpit_f14.blend
# and the radar intercept officer's (rear seat)
#   blender -b assets-src/blend/cockpit_base.blend --python-exit-code 1 \
#       --python tools/models/build_cockpit_f14.py -- \
#       assets-src/blend/cockpit_f14_rio_layout.json assets-src/blend/cockpit_f14_rio.blend \
#       --seat rio
# then tools/models/export.sh cockpit_f14 cockpit_f14_rio. The layout JSONs are written by
# `UPDATE_LAYOUT=1 cargo test layout_files` from `PanelLayout::f14` / `f14_rio`
# (src/cockpit3d/f14_layout.rs). Each seat's cockpit has its own panel live; the other seat's
# panel is its plate only (from the front seat the RIO's panel is behind the pilot, from the
# back seat the pilot's is behind the pilot's seat and under its glare shield).
#
# One cabin for both: the geometry is laid out in the exterior's frame (tools/models/aircraft.py
# `f14`: x right of the centre line, s aft of the radome's tip, z up from it, metres) and built
# round the seat's eye (`AircraftSpec::eye` / its `gunner_seat`): the pilot's (0, 4.75, 1.36), the
# RIO's (0, 6.35, 1.42), 1.6 m aft and a little higher. Life size (the F-117A's cockpit is built
# larger than life: its kept parts are scaled down by F117_SCALE about the eye).
#
# Kept, with their materials and control properties: the seat with the ejection handle (copied
# for the other seat), the stick (the pilot's centre stick on the centre line; in the RIO's
# cockpit his hand controller on its pedestal on the centre line is the `stick`, a tilt
# sensing one: `tilt`), the gear handle on the pilot's left knee panel,
# the HUD glass (the pilot's combiner on the glare shield), the approach indexer and the flood
# lamps. Built here, low-poly: the throttle on the pilot's left console (the RIO has none): the
# two engines' levers swinging together about a pivot sunk in the console (the `throttle_full`
# and `throttle_pivot` anchors), up through the slots of the quadrant, a grip on each with its
# switches; the tub under the one-piece canopy (walls up to the glass's
# sill, the canopy's sill rails, the windscreen's bow and posts, the turtledeck behind the RIO),
# the windscreen's front pane (an oval window in its heavy frame), both instrument panels (the
# pilot's under its glare shield with the HUD's combiner; the RIO's with its radar section
# standing out under a shroud), the knee panels sloping down from them towards the crew (the
# pilot's gear and DISPLAYS panels, the RIO's armament and ECM panels: `PARTS` moves those parts
# of the flat layouts there), the consoles, the rudder pedals; on the live panel its displays
# (the square tubes of the VDI, the HSD, the DDD and the ECMD in deep bezels, the round TID, the
# round standby instruments), the round instruments and tapes, the steering pushbuttons (`twist`), the lit
# pushbuttons (`lamp`), the toggle switches (`switch`), the knobs (`knob`) and keys (`cap`).
#
# Blender frame: eye at the origin, x right, y forward, z up, metres. A control's face points
# along its local -Y (towards the crew), local Z is up the panel.
import json
import math
import sys

import bmesh
import bpy
from mathutils import Euler, Matrix, Quaternion, Vector

args = sys.argv[sys.argv.index("--") + 1:]
layout_path, out = args[:2]
layout = json.load(open(layout_path))
RIO = "--seat" in args and args[args.index("--seat") + 1] == "rio"
root = bpy.data.objects["cockpit"]

F117_SCALE = 1.4706
# The seats' eyes (x, s, z).
PILOT_EYE = (0.0, 4.75, 1.36)
RIO_EYE = (0.0, 6.35, 1.42)
EYE = RIO_EYE if RIO else PILOT_EYE
OTHER_EYE = PILOT_EYE if RIO else RIO_EYE


def C(x, s, z):
    """A point of the cabin (x right of the centre line, s aft of the radome's tip, z up) in the
    eye's frame."""
    return Vector((x - EYE[0], EYE[1] - s, z - EYE[2]))


# The canopy's glass (tools/models/aircraft.py F14_CANOPY, F14_GLASS): (s, sill z, half width at
# the sill, top) and the section's shape (fraction of the height, of the half width).
CANOPY = [(2.95, 0.97, 0.20, 0.99), (3.30, 0.97, 0.36, 1.14), (3.60, 0.97, 0.44, 1.29),
          (3.88, 0.97, 0.48, 1.44), (4.30, 0.97, 0.50, 1.62), (4.80, 0.97, 0.51, 1.74),
          (5.40, 0.97, 0.51, 1.79), (6.20, 0.97, 0.51, 1.79), (6.80, 0.97, 0.50, 1.74),
          (7.20, 0.97, 0.48, 1.63), (7.55, 0.97, 0.46, 1.49)]
GLASS = [(0.0, 1.0), (0.25, 0.99), (0.45, 0.95), (0.62, 0.87), (0.76, 0.75), (0.87, 0.59), (0.95, 0.36),
         (1.0, 0.0)]
BOW_S = 3.88


def interp(table, x):
    """Linear through a table of (x, y...) rows at x."""
    for a, b in zip(table, table[1:]):
        if x <= b[0]:
            t = min(max((x - a[0]) / (b[0] - a[0]), 0.0), 1.0)
            return tuple(p + (q - p) * t for p, q in zip(a[1:], b[1:]))
    return tuple(table[-1][1:])


def canopy(s):
    """(sill z, half width at the sill, top z) of the glass at s."""
    return interp(CANOPY, s)


def glass_half(s, z):
    """The glass's half width at height z at s (0 over its top)."""
    sill, hw, top = canopy(s)
    t = (z - sill) / (top - sill)
    if t <= 0.0:
        return hw
    return hw * interp(GLASS, min(t, 1.0))[0]


# --- the instrument panels: the layout's origin on the centre line, the top leaning away
class Panel:
    def __init__(self, origin, tilt_deg):
        t = math.radians(tilt_deg)
        self.origin = origin
        self.up = Vector((0.0, math.sin(t), math.cos(t)))
        self.normal = Vector((0.0, -math.cos(t), math.sin(t)))
        self.rot = Matrix((RIGHT, -self.normal, self.up)).transposed().to_quaternion()

    def point(self, u, v, offset=0.0):
        """A panel point: layout millimetres (u right, v down), raised `offset` m off it."""
        return self.origin + RIGHT * (u / 1000) - self.up * (v / 1000) + self.normal * offset

    def cabin(self, u, v):
        """A panel point's (s, z) in the cabin."""
        p = self.point(u, v)
        return EYE[1] - p.y, p.z + EYE[2]


class Raised:
    """A part of a panel standing `d` m out of it towards the crew (the RIO's radar section)."""

    def __init__(self, base, d):
        self.base, self.d = base, d
        self.up, self.normal, self.rot = base.up, base.normal, base.rot

    def point(self, u, v, offset=0.0):
        return self.base.point(u, v, offset + self.d)


class Shift:
    """A part of a panel moved over it: layout (u, v) is built at (u + du, v + dv)."""

    def __init__(self, base, du, dv):
        self.base, self.du, self.dv = base, du, dv
        self.up, self.normal, self.rot = base.up, base.normal, base.rot

    def point(self, u, v, offset=0.0):
        return self.base.point(u + self.du, v + self.dv, offset)


class Slope:
    """A knee panel: it leaves the panel along the row v = fold and slopes down towards the
    crew, `angle` degrees off the panel. The layout's rows from v = top on run down it (its
    columns moved du). Its head stands `out` m out of the panel (on a ledge). Canted: turned
    `cant` degrees about its own length to face the centre line, about its edge at u = pivot
    (the side wall's: the inboard edge lies deeper)."""

    def __init__(self, base, top, fold, angle, du=0.0, out=0.0, cant=0.0, pivot=0.0):
        a, c = math.radians(angle), math.radians(cant)
        self.base, self.top, self.fold, self.du, self.out, self.pivot = base, top, fold, du, out, pivot
        self.down = -base.up * math.cos(a) + base.normal * math.sin(a)
        flat = base.normal * math.cos(a) + base.up * math.sin(a)
        side = 1.0 if pivot > 0.0 else -1.0
        self.right = RIGHT * math.cos(c) + flat * (side * math.sin(c))
        self.normal = flat * math.cos(c) - RIGHT * (side * math.sin(c))
        self.up = -self.down
        self.rot = Matrix((self.right, -self.normal, self.up)).transposed().to_quaternion()

    def point(self, u, v, offset=0.0):
        head = self.base.point(self.pivot, self.fold, self.out)
        return head + self.right * ((u + self.du - self.pivot) / 1000) + self.down * ((v - self.top) / 1000) + self.normal * offset


RIGHT = Vector((1.0, 0.0, 0.0))
PILOT_PANEL = Panel(C(0.0, 4.13, 0.86), 15.0)
RIO_PANEL = Panel(C(0.0, 5.80, 0.98), 10.0)
MAIN = RIO_PANEL if RIO else PILOT_PANEL
# The panel the layout's helpers build on (`use` picks a control's).
PANEL = MAIN
# The knee panels slope down towards the crew from the foot of the vertical panel, this many
# degrees off it.
KNEE = 40.0
# The RIO's radar section (the DDD and its panel: layout v above this) stands out of the panel
# under its shroud.
RIO_RADAR_V = -116.0
RIO_RADAR_OUT = 0.06
SHROUD = 0.08
# The pilot's centre stack stands out of his panel in steps: the ACM panel's box under the glare
# shield furthest, the VDI's housing (wider than its tube: a strip of knobs either side) under
# it, the HSD's least: (layout rectangle, m out of the panel).
ACM_BOX = ((-152.0, -272.0, 264.0, 78.0), 0.070)
VDI_BOX = ((-114.0, -196.0, 228.0, 182.0), 0.045)
HSD_BOX = ((-85.0, -12.0, 170.0, 164.0), 0.025)
STACK = [ACM_BOX, VDI_BOX, HSD_BOX]
# The fire handles (FUEL SHUT OFF, PULL) either side of the ACM panel: layout (u, v).
FIRE_HANDLES = ((-232.0, -197.0), (252.0, -197.0))
# The pilot's knee panels (fitted to a photograph of the cockpit from the pilot's eye): each a
# box against its side wall, its head on a ledge KNEE_OUT m out of the panel along the row
# KNEE_FOLD (a little over the HSD's top), its face sloping KNEE_PILOT degrees off the panel down
# to the consoles' fronts, KNEE_LEN mm long, and turned KNEE_CANT degrees to face the centre
# line; a little wider at its foot: its inboard edge KNEE_IN mm from the centre line (head,
# foot), its outboard one on the wall (KNEE_WALL).
KNEE_PILOT = 33.0
KNEE_CANT = 8.0
KNEE_OUT = 0.085
KNEE_FOLD = -25.0
KNEE_LEN = 308.0
KNEE_IN = (289.0, 276.0)
KNEE_WALL = 445.0
# The arresting hook's panel on the right knee panel over the DISPLAYS panel: where the layout
# draws it (`f14_layout::HOOK_PANEL`: off the instrument panel; left, top, width, height) and
# its left edge on the knee panel in the DISPLAYS panel's columns (centred over that panel). The
# handle swings about a pivot HOOK_PIVOT m behind the panel's face (a toggle switch's lean:
# 0.5 rad from UP to DN), a flat bar up through the slot with the grip HOOK_REACH m out.
HOOK_PANEL = (500.0, 14.0, 66.0, 80.0)
HOOK_U = 138.0
HOOK_PIVOT = 0.085
HOOK_REACH = 0.055
# The RIO's: the TID's housing (layout rectangle, m out of the panel) and the hood round its
# tube (m out of the housing: low and thin, clear of the buttons round it and of the eye's
# line to the knobs under it); his knee boxes (see `rio_knee`).
TID_BOX = ((-162.0, -112.0, 324.0, 298.0), 0.045)
TID_HOOD = 0.012
RIO_KNEES = {-1: (-100.0, 52.0), 1: (-102.0, 16.0)}
RIO_KNEE_OUT = 0.04
RIO_KNEE_IN = 180.0
RIO_KNEE_LEN = 250.0
RIO_WALL = 398.0
# The parts of the layout built elsewhere than where it draws them: (u from, to, v from, to) of
# the layout and their panel. The pilot's: the right knee panel (the DISPLAYS panel), the left
# one (the knee panel with the gear handle). The RIO's: the flight instruments go up beside the
# TID and the armament panel slopes under them; the LANTIRN panel goes up on the right and the
# ECM panel with the ECMD slopes under it.
if RIO:
    RADAR = Raised(MAIN, RIO_RADAR_OUT)
    # His knee panels are boxes like the pilot's (RIO_KNEES: the layout's row at the box's
    # head, the row it leaves the panel along), from RIO_KNEE_IN mm off the centre line out to
    # the wall, RIO_KNEE_LEN mm down to the consoles' fronts: wider and longer than the layout's
    # parts on them (the rest of their faces is dressing).
    def rio_knee(side, du=0.0):
        top, fold = RIO_KNEES[side]
        return Slope(MAIN, top, fold, KNEE_PILOT, du, RIO_KNEE_OUT, KNEE_CANT, side * RIO_WALL)

    PARTS = [
        ((-346.0, -140.0, 118.0, 256.0), Shift(MAIN, -52.0, -204.0)),
        ((-348.0, -230.0, -104.0, 100.0), rio_knee(-1, -48.0)),
        ((124.0, 348.0, 110.0, 256.0), Shift(MAIN, 52.0, -194.0)),
        ((186.0, 348.0, -106.0, 110.0), rio_knee(1, 52.0)),
        # The TID's housing stands out of the panel with its buttons and knobs.
        ((TID_BOX[0][0], TID_BOX[0][0] + TID_BOX[0][2], TID_BOX[0][1], TID_BOX[0][1] + TID_BOX[0][3]), Raised(MAIN, TID_BOX[1])),
    ]
    KNEES = []
else:
    RADAR = None
    # The layout's rows run down the knee panels as they are drawn (v = mm down the slope): the
    # DISPLAYS panel moved out onto the right one under the emergency gear handle's plate (the
    # model's); the knee panel's column and the gear handle's plate under it on the left one.
    def knee_part(du, side):
        return Slope(MAIN, 0.0, KNEE_FOLD, KNEE_PILOT, du, KNEE_OUT, KNEE_CANT, side * KNEE_WALL)

    PARTS = [
        ((HOOK_PANEL[0] - 4.0, HOOK_PANEL[0] + HOOK_PANEL[2] + 4.0, HOOK_PANEL[1] - 4.0, HOOK_PANEL[1] + HOOK_PANEL[3] + 4.0),
         knee_part(192.0 + HOOK_U - HOOK_PANEL[0], 1)),
        ((90.0, 300.0, 100.0, 300.0), knee_part(192.0, 1)),
        ((-356.0, -272.0, 156.0, 268.0), knee_part(-34.0, -1)),
        ((-356.0, -272.0, -2.0, 156.0), knee_part(-46.0, -1)),
    ] + [((r[0], r[0] + r[2], r[1], r[1] + r[3]), Raised(MAIN, d)) for r, d in STACK]
    KNEES = []


def panel_at(u, v):
    """The panel a layout point is on: the radar section, a moved or sloped part, else the flat
    one."""
    if RADAR is not None and v < RIO_RADAR_V:
        return RADAR
    for (u0, u1, v0, v1), part in PARTS:
        if u0 < u < u1 and v0 < v < v1:
            return part
    return MAIN


def use(r=None, at=None):
    """Builds on the panel the layout rectangle r (or point `at`) is on; nothing: the flat one."""
    global PANEL
    if r is not None:
        at = (r[0] + r[2] / 2.0, r[1] + r[3] / 2.0)
    PANEL = MAIN if at is None else panel_at(*at)
# The panels' outlines (layout mm, the right half; the left one mirrored).
PILOT_OUTLINE = [(0.0, -275.0), (280.0, -275.0), (330.0, -235.0), (372.0, -190.0), (440.0, -120.0), (445.0, 268.0), (0.0, 268.0)]
# (Wider under the radar section: the flight instruments and the LANTIRN panel stand outboard
# of the TID's buttons.)
RIO_OUTLINE = [(0.0, -306.0), (300.0, -306.0), (344.0, -262.0), (348.0, -116.0), (398.0, -110.0), (398.0, 256.0), (0.0, 256.0)]
OUTLINE = RIO_OUTLINE if RIO else PILOT_OUTLINE
# How deep a panel's body is behind its face.
PANEL_DEPTH = 0.22
# Floors: the pilot's, the RIO's (raised), the step between them.
PILOT_FLOOR = 0.30
RIO_FLOOR = 0.42
STEP_S = 5.45
# The tub: from under the glare shield's front back to the bulkhead behind the RIO; the walls
# stand WALL_IN inside the glass's sill.
FRONT_S = 3.45
REAR_S = 7.12
WALL_IN = 0.035
# The turtledeck behind the RIO under the glass, to the exterior's spine.
DECK_Z = 1.45
DECK_END = 7.62
# Consoles: (inner and outer x, s from and to, top z) of each seat's.
# (They start behind the knee panels.)
PILOT_CONSOLE = (0.27, 0.45, 4.50, 5.40, 0.72)
RIO_CONSOLE = (0.25, 0.45, 6.02, 6.95, 0.75)
# Seat: its cushion's top under the eye.
SEAT_TOP = -0.80
# The pilot's centre stick: its pivot on the floor, the kept grip raised on a shaft.
STICK_S = 4.42
STICK_RAISE = 0.36
# The throttle on the pilot's left console: where the hand holds its grips at idle, its travel
# forward (the chord of its arc); the pivot THROTTLE_PIVOT under the console's top, under the
# middle of the travel. The quadrant: a well in the console (half width, s from and to), its
# slotted cover QUADRANT_SUNK under the top.
THROTTLE = (-0.36, 4.74, PILOT_CONSOLE[4] + 0.089)
THROTTLE_TRAVEL = 0.17
THROTTLE_PIVOT = 0.12
QUADRANT = (0.048, 4.555, 4.765)
QUADRANT_SUNK = 0.012
# The levers either side of the throttle's middle, their slots' width.
LEVER_X = 0.024
SLOT = 0.012
# The pilot's HUD combiner on the glare shield: its distance ahead of the eye and its half
# width, bottom and top off the eye: the Tornado's combiner at the same angular size
# (AircraftSpec::hud_scale 2080 fits the symbology into it).
HUD_Y = 0.78
# The glass reaches lower than the Tornado's, down to its body on the shield (its frame only
# clips the picture, the scale is the angular size across; the clipped corners are outside the
# symbology's field, src/hud/modern.rs FIELD).
# Its width is the oval front pane's as the 2D camera sees it (PANE_X at the pane, some 1.25 m
# ahead of the eye; the camera sits `Settings::seat_offset`'s lean nearer, src/settings.rs
# F14_DESK_VIEW): wide enough there for the symbology's field (src/hud/vdig.rs HALF).
HUD_GLASS = (0.110, -0.165, 0.0926)
# The combiner's clipped top corners (share of its half width, of its height).
HUD_CLIP = (0.30, 0.22)
HUD_BODY = (0.16, 0.09)
HUD_BAR = 0.010
# The glare shield's front edge (s) and its height there (centre, at its side).
SHADE_FRONT = 3.45
# Segments of the round parts (bezels, buttons, knobs): low-poly.
ROUND = 16
KNOB = 8
PLATE = 0.004

# --- keep, drop
KEEP = (
    "seat_", "cushion_", "metal_98", "metal_99", "control_Action_Eject_", "stick", "throttle",
    "hud_glass", "approach", "lamp", "control_Action_Gear_.001",
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


SCREENS = ("hsi", "screen_rwr", "screen_f14vdi", "screen_f14hsd", "screen_ddd", "screen_tid", "screen_ecmd")
for n in SCREENS:
    material(n, "adi")
coloured("red", "yellow", (0.55, 0.04, 0.03))
coloured("white", "yellow", (0.80, 0.80, 0.76))
# The pilot's panel's light grey face (the black instruments' cases stand on it).
coloured("panel_light", "panel_grey", (0.72, 0.74, 0.72))
# The VDI's pale knobs.
coloured("knob_pale", "yellow", (0.55, 0.56, 0.54))
# The hook handle's grip: worn blue grey.
coloured("hook_grip", "panel_grey", (0.33, 0.37, 0.44), 0.6)
# A dark tube's glass (the other seat's dressing).
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
    """An n-gon in the local XZ plane at local y (faces -Y); its UVs map the 0..1 square's
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


def loft(name, mat, sections, smooth=False):
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
    return mesh_object(name, finish(name, bm, mat, smooth), (0, 0, 0))


def rod(name, a, b, radius, mat, parent=root, n=KNOB):
    """A cylinder from a to b (parent frame)."""
    a, b = Vector(a), Vector(b)
    bm = bmesh.new()
    bmesh.ops.create_cone(bm, cap_ends=True, segments=n, radius1=radius, radius2=radius, depth=(b - a).length)
    me = finish(name, bm, mat)
    rot = Vector((0, 0, 1)).rotation_difference((b - a).normalized()).to_matrix().to_4x4()
    me.transform(Matrix.Translation((a + b) / 2) @ rot)
    return mesh_object(name, me, (0, 0, 0), parent=parent)


# --- panel helpers (the live panel's layout)
# The flight instruments are in square cases.
SQUARE = True


def point(u, v, offset=0.0):
    return PANEL.point(u, v, offset)


def rotation(u=0.0, v=0.0):
    return PANEL.rot


def centre(r):
    x, y, w, h = r
    return x + w / 2.0, y + h / 2.0


def rect_point(r, offset=0.0):
    return point(*centre(r), offset)


def panel_box(name, mat, r, raise_m, margin=0.0, base=0.0):
    """A plate from `base` to `base + raise_m` off the panel over the rectangle r (mm) grown by
    `margin` mm."""
    x, y, w, h = r
    c = rect_point(r, base + raise_m / 2)
    return box(name, mat, c, ((w + 2 * margin) / 1000, abs(raise_m), (h + 2 * margin) / 1000), PANEL.rot)


def bezel(r, offset, height=0.008, lip=0.003, case="dark"):
    """A round black bezel standing `height` out of the panel at `offset` round the rectangle r's
    inscribed circle, the instrument's case behind it: octagonal, or (the pilot's, `SQUARE`) a
    square black one with a screw in each corner."""
    x, y, w, h = r
    me = cylinder_mesh("black", min(w, h) / 2000 + lip, height, "black", n=ROUND)
    mesh_object("black", me, rect_point(r, offset), PANEL.rot)
    flat = min(w, h) / 2000 + lip + 0.002
    if SQUARE:
        deep = offset + 0.004
        mesh_object("black", box_mesh("black", (2 * flat, deep, 2 * flat), "black", (0.0, -deep / 2, 0.0)), rect_point(r, 0.0), PANEL.rot)
        for sx in (-1, 1):
            for sz in (-1, 1):
                p = Vector((sx * (flat - 0.0045), -deep, sz * (flat - 0.0045)))
                mesh_object("metal", cylinder_mesh("metal", 0.0022, 0.001, "metal", n=KNOB), rect_point(r, 0.0) + PANEL.rot @ p, PANEL.rot)
        return
    me = cylinder_mesh(case, flat / math.cos(math.pi / 8), offset + 0.003, case, n=8)
    me.transform(Matrix.Rotation(math.pi / 8, 4, "Y"))
    mesh_object(case, me, rect_point(r, 0.0), PANEL.rot)


def face(key, r, offset, round=False):
    """An atlas face over the rectangle r; round ones are discs in a bezel (the instruments)."""
    x, y, w, h = r
    if round:
        bezel(r, offset)
        me = disc_mesh("atlas", min(w, h) / 2000, "atlas")
        offset += 0.0085
    else:
        me = quad_mesh("atlas", w / 1000, h / 1000, "atlas")
    mesh_object("atlas", me, rect_point(r, offset), PANEL.rot, props={"atlas_face": key, "atlas_uv": [0.0, 0.0, 1.0, 1.0], "no_shadow": True})


def highlight(c, hw, hh, height):
    """The frame shown round a control while a fingertip touches it."""
    hl = empty("highlight", (0, 0, 0), parent=c, props={"highlight": True})
    t = 0.002
    for (cx, cz, sx, sz) in ((0, hh + t, 2 * hw + 2 * t, t), (0, -hh - t, 2 * hw + 2 * t, t), (hw + t, 0, t, 2 * hh), (-hw - t, 0, t, 2 * hh)):
        mesh_object("highlight", box_mesh("highlight", (sx, t, sz), "highlight"), (cx, -height, cz), parent=hl, props={"no_shadow": True})


def control_name(effect):
    return "control_" + "".join(ch if ch.isalnum() else "_" for ch in effect)


def control(key, r, offset, effect, kind):
    """A push button or key: origin on the surface, its face `height` out; a cap body under the
    face and a highlight frame. A `twist` one (the steering pushbutton that latches in turned 45
    degrees) stands taller with a grey body; a `lamp` one (a lit pushbutton) a dark one."""
    x, y, w, h = r
    hw, hh = w / 2000, h / 2000
    height, travel = {"twist": (0.008, 0.004), "lamp": (0.007, 0.004)}.get(kind, (0.006, 0.004))
    body = "frame" if kind == "twist" else "dark"
    c = empty(control_name(effect), rect_point(r, offset), PANEL.rot, props={"control": effect, "kind": kind, "half": [hw, hh], "height": height, "travel": travel})
    moving = empty("moving", (0, 0, 0), parent=c, props={"moving": True})
    mesh_object("atlas", quad_mesh("atlas", 2 * hw, 2 * hh, "atlas", -height), (0, 0, 0), parent=moving, props={"atlas_face": key, "atlas_uv": [0.0, 0.0, 1.0, 1.0], "no_shadow": True})
    mesh_object(body, box_mesh(body, (2 * hw, height - 0.0004, 2 * hh), body, (0, -(height - 0.0004) / 2, 0)), (0, 0, 0), parent=moving)
    highlight(c, hw, hh, height)


def switch(key, r, offset, effect):
    """A toggle switch: its plate (the atlas face) on the surface, a collar and the lever (the
    `moving` part, leaning to its position about the control's X)."""
    x, y, w, h = r
    hw, hh = w / 2000, h / 2000
    c = empty(control_name(effect), rect_point(r, offset), PANEL.rot, props={"control": effect, "kind": "switch", "half": [hw, hh], "height": 0.014, "travel": 0.0})
    mesh_object("atlas", quad_mesh("atlas", 2 * hw, 2 * hh, "atlas", -0.0004), (0, 0, 0), parent=c, props={"atlas_face": key, "atlas_uv": [0.0, 0.0, 1.0, 1.0], "no_shadow": True})
    mesh_object("metal", cylinder_mesh("metal", 0.0045, 0.003, "metal", n=KNOB), (0, 0, 0), parent=c)
    moving = empty("moving", (0, 0, 0), parent=c, props={"moving": True})
    mesh_object("metal", box_mesh("metal", (0.0036, 0.017, 0.0036), "metal", (0.0, -0.0085, 0.0)), (0, 0, 0), parent=moving)
    mesh_object("metal", cylinder_mesh("metal", 0.0028, 0.004, "metal", centre=(0.0, -0.015, 0.0), n=KNOB), (0, 0, 0), parent=moving)
    highlight(c, hw, hh, 0.001)


def hook_handle(r, offset, effect):
    """The arresting hook's handle in its slot (the layout rectangle r): a `switch` whose `moving`
    part is the whole handle, swinging about a pivot behind the panel (the control's origin),
    up the slot with the hook up and down it with the hook down. A flat bar out through the
    slot, on its end the grip: a small hook."""
    x, y, w, h = r
    hw, hh = w / 2000, h / 2000
    out = HOOK_PIVOT + HOOK_REACH
    # The slot: a dark well with a raised rim.
    frame("dark", (x - 1.5, y - 1.5, w + 3.0, h + 3.0), (x, y, w, h), offset, offset + 0.003)
    c = empty(control_name(effect), rect_point(r, offset - HOOK_PIVOT), PANEL.rot,
              props={"control": effect, "kind": "switch", "half": [hw + 0.012, hh + 0.006], "height": out + 0.03, "travel": 0.0})
    moving = empty("moving", (0, 0, 0), parent=c, props={"moving": True})
    mesh_object("metal", box_mesh("metal", (0.005, out, 0.017), "metal", (0.0, -out / 2, 0.0)), (0, 0, 0), parent=moving)
    # The grip is a small arresting hook, as on the aircraft: in side view (mm out along the
    # handle from the bar's end, mm up the panel) a slim shank and a toe that curls down and
    # back round a throat. A thin slab, its edges chamfered.
    profile = [(-4, 4), (12, 5), (28, 7), (42, 7), (52, 3), (57, -4), (57, -12), (52, -18), (45, -19),
               (41, -15), (45, -11), (46, -6), (41, -2), (28, -3), (12, -4), (-4, -4)]
    ca = sum(a for a, _ in profile) / len(profile)
    cb = sum(b for _, b in profile) / len(profile)
    bm = bmesh.new()
    rings = []
    for x_, k in ((-0.006, 0.86), (-0.004, 1.0), (0.004, 1.0), (0.006, 0.86)):
        rings.append([bm.verts.new((x_, -(out + (ca + (a - ca) * k) / 1000), (cb + (b - cb) * k) / 1000)) for a, b in profile])
    n_ = len(profile)
    for r0, r1 in zip(rings, rings[1:]):
        for q in range(n_):
            bm.faces.new((r0[q], r0[(q + 1) % n_], r1[(q + 1) % n_], r1[q]))
    caps = [bm.faces.new(rings[0]), bm.faces.new(list(reversed(rings[-1])))]
    bmesh.ops.triangulate(bm, faces=caps)
    bmesh.ops.recalc_face_normals(bm, faces=bm.faces)
    mesh_object("hook_grip", finish("hook_grip", bm, "hook_grip"), (0, 0, 0), parent=moving)
    # The control's face in the atlas: on the slot's floor.
    mesh_object("atlas", quad_mesh("atlas", 2 * hw, 2 * hh, "atlas", -HOOK_PIVOT - 0.0008), (0, 0, 0), parent=c,
                props={"atlas_face": "Item(Tomcat(Hook))", "atlas_uv": [0.0, 0.0, 1.0, 1.0], "no_shadow": True})
    highlight(c, hw + 0.012, hh + 0.006, HOOK_PIVOT)


def knob_control(key, r, offset, effect):
    """A rotary knob: a dark skirt, the knob (the `moving` part, turned to its position about
    the control's face normal) with its top the atlas face (its index)."""
    x, y, w, h = r
    rad = min(w, h) / 2000
    height = 0.014
    c = empty(control_name(effect), rect_point(r, offset), PANEL.rot, props={"control": effect, "kind": "knob", "half": [rad, rad], "height": height, "travel": 0.0})
    mesh_object("black", cylinder_mesh("black", rad + 0.002, 0.003, "black", n=ROUND), (0, 0, 0), parent=c)
    moving = empty("moving", (0, 0, 0), parent=c, props={"moving": True})
    mesh_object("dark", cylinder_mesh("dark", rad, height - 0.0004, "dark", n=ROUND), (0, 0, 0), parent=moving)
    mesh_object("atlas", disc_mesh("atlas", rad, "atlas", -height), (0, 0, 0), parent=moving, props={"atlas_face": key, "atlas_uv": [0.0, 0.0, 1.0, 1.0], "no_shadow": True})
    highlight(c, rad, rad, 0.001)


def plane_box(name, mat, r, d0, d1):
    """A box over the rectangle r (layout mm) from d0 to d1 m off the panel plane."""
    x, y, w, h = r
    c = point(x + w / 2, y + h / 2, (d0 + d1) / 2)
    return box(name, mat, c, (w / 1000, abs(d1 - d0), h / 1000), PANEL.rot)


def frame(mat, outer, inner, d0, d1):
    """A rectangular frame (layout mm, outer and inner rectangles) from d0 to d1 m off the
    panel plane."""
    ox, oy, ow, oh = outer
    ix, iy, iw, ih = inner
    for r in ((ox, oy, ow, iy - oy), (ox, iy + ih, ow, oy + oh - iy - ih), (ox, iy, ix - ox, ih), (ix + iw, iy, ox + ow - ix - iw, ih)):
        plane_box(mat, mat, r, d0, d1)


def knob(u, v, r=0.006, h=0.012, mat="metal", offset=0.0):
    """A round knob standing out of the panel at layout (u, v) (dressing)."""
    mesh_object(mat, cylinder_mesh(mat, r, h, mat, n=KNOB), point(u, v, offset), PANEL.rot)


def panel_body(name, mat, panel, outline, depth):
    """A panel's plate: its face on the panel plane from the outline (right half, mirrored), its
    body `depth` m deep behind it."""
    half = [(u, v) for u, v in outline]
    loop = [(-u, v) for u, v in reversed(half[1:-1])] + half[1:-1]
    loop = [(-half[0][0], half[0][1])] + loop if half[0][0] != 0.0 else loop
    bm = bmesh.new()
    front = [bm.verts.new(panel.point(u, v)) for u, v in loop]
    back = [bm.verts.new(panel.point(u, v) - panel.normal * depth) for u, v in loop]
    bm.faces.new(front)
    bm.faces.new(back[::-1])
    for i in range(len(loop)):
        j = (i + 1) % len(loop)
        bm.faces.new((front[i], back[i], back[j], front[j]))
    bmesh.ops.recalc_face_normals(bm, faces=bm.faces[:])
    return mesh_object(name, finish(name, bm, mat), (0, 0, 0))


def glass_section(s, zmax, inset, floor):
    """The cabin's section at s inside the glass: from the left floor up the wall and the glass
    (inset `inset` m) to zmax, across, down the right side to the floor."""
    sill, hw, top = canopy(s)
    zmax = min(zmax, top - inset)
    right = [(hw - inset, floor), (hw - inset, sill)]
    for t in (0.25, 0.45, 0.62, 0.76, 0.87, 0.95):
        z = sill + (top - sill) * t
        if z >= zmax:
            break
        right.append((glass_half(s, z) - inset, z))
    right.append((max(glass_half(s, zmax) - inset, 0.02), zmax))
    left = [(-x, z) for x, z in reversed(right)]
    return [C(x, s, z) for x, z in left + right]


# --- the tub: floors, walls up to the glass's sill, the canopy's rails
cbox("dark", (-0.47, 0.47), (FRONT_S, STEP_S), (PILOT_FLOOR - 0.02, PILOT_FLOOR))
cbox("dark", (-0.47, 0.47), (STEP_S, REAR_S + 0.02), (RIO_FLOOR - 0.02, RIO_FLOOR))
cbox("dark", (-0.47, 0.47), (STEP_S - 0.02, STEP_S), (PILOT_FLOOR, RIO_FLOOR))
STATIONS = [FRONT_S, 3.60, 3.88, 4.30, 4.80, 5.40, 6.20, 6.80, REAR_S]
for side in (-1, 1):
    wall, rail = [], []
    for s in STATIONS:
        sill, hw, _ = canopy(s)
        xi = hw - WALL_IN
        wall.append([C(side * a, s, b) for a, b in ((xi, PILOT_FLOOR - 0.02), (xi, sill + 0.004), (xi + 0.025, sill + 0.004), (xi + 0.025, PILOT_FLOOR - 0.02))])
    for s in [2.95, 3.30] + STATIONS[1:] + [7.20, 7.55]:
        sill, hw, _ = canopy(s)
        xi = hw - WALL_IN
        rail.append([C(side * a, s, b) for a, b in ((xi - 0.012, sill - 0.024), (xi - 0.012, sill + 0.012), (xi + 0.03, sill + 0.012), (xi + 0.03, sill - 0.024))])
    loft("panel_grey", "panel_grey", wall)
    loft("frame", "frame", rail)

# The turtledeck behind the RIO: a bulkhead at REAR_S up to DECK_Z under the glass, its lid back
# to the exterior's spine.
loft("panel_grey", "panel_grey", [glass_section(s, DECK_Z - (s - REAR_S) * 0.05, 0.03, RIO_FLOOR) for s in (REAR_S, 7.30, 7.50, DECK_END)])

# --- the canopy's frames from inside (the exterior's are cut away round the cockpit,
# `CockpitView::cut` in src/aircraft.rs): the windscreen's bow, its two posts.
# (A heavy arch: the real one carries the mirrors.)
BOW_W = 0.085
BOW_DEPTH = 0.042


def bow(s, width, depth, inset=0.012):
    """An arch from sill to sill just inside the glass at s, `width` m along the cabin."""
    sill, hw, top = canopy(s)
    # The glass's section from the right sill over the top to the left one.
    pts = []
    for i in range(25):
        t = i / 24
        z = sill + 0.012 + (top - sill - 0.012) * (1.0 - abs(1.0 - 2.0 * t)) ** 0.6
        x = glass_half(s, z) * (1.0 if t < 0.5 else -1.0)
        pts.append((x, z))
    secs = []
    for ds in (-width / 2, width / 2):
        outer, inner = [], []
        for x, z in pts:
            d = Vector((x, 0.0, z - (sill + 0.05))).normalized() if abs(x) > 1e-6 or z > sill else Vector((0, 0, 1))
            o = Vector((x, 0.0, z)) - d * inset
            q = o - d * depth
            outer.append(C(o.x, s + ds, o.z))
            inner.append(C(q.x, s + ds, q.z))
        secs.append((outer, inner))
    bm = bmesh.new()
    vs = [[[bm.verts.new(p) for p in row] for row in pair] for pair in secs]
    m = len(pts)
    for i in range(m - 1):
        (fo, fi), (bo, bi) = vs
        for quad in ((fo[i], fo[i + 1], bo[i + 1], bo[i]), (fi[i], bi[i], bi[i + 1], fi[i + 1]),
                     (fo[i], fi[i], fi[i + 1], fo[i + 1]), (bo[i], bo[i + 1], bi[i + 1], bi[i])):
            bm.faces.new(quad)
    bmesh.ops.recalc_face_normals(bm, faces=bm.faces[:])
    return mesh_object("frame", finish("frame", bm, "frame"), (0, 0, 0))


bow(BOW_S + 0.03, BOW_W, BOW_DEPTH)
# The windscreen's posts off the centre line: at the front pane's front, at the bow.
POST_X = (0.195, 0.225)
# The windscreen's posts between its front pane and its side panes, along the glass.
for side in (-1, 1):
    a = (POST_X[0], 3.02)
    b = (POST_X[1], BOW_S)
    pts = []
    for k in range(5):
        f = k / 4
        x, s = a[0] + (b[0] - a[0]) * f, a[1] + (b[1] - a[1]) * f
        sill, hw, top = canopy(s)
        # The height where the glass is x wide, a little inside it.
        lo, hi = sill, top
        for _ in range(30):
            mid = (lo + hi) / 2
            if glass_half(s, mid) > x:
                lo = mid
            else:
                hi = mid
        pts.append(C(side * (x - 0.006), s, lo - 0.008))
    for p, q in zip(pts, pts[1:]):
        rod("frame", p, q, 0.011, "frame", n=6)

# The windscreen's front pane: an oval window in the heavy frame between the posts, from the
# glare shield's front up to the bow (the frame lies on the inside of the glass).
PANE_S = (3.50, 0.375)
PANE_X = 0.19
PANE_ROUND = 2.6
PANE_THICK = 0.022


def glass_z(s, x):
    """The glass's height at s where it is x from the centre line (its sill where narrower)."""
    sill, hw, top = canopy(s)
    lo, hi = sill, top
    for _ in range(30):
        mid = (lo + hi) / 2
        if glass_half(s, mid) > abs(x):
            lo = mid
        else:
            hi = mid
    return lo


def pane_frame():
    edge_s = (3.02, BOW_S + 0.02)

    def edge_x(s):
        return POST_X[0] + (POST_X[1] - POST_X[0]) * (s - 3.02) / (BOW_S - 3.02)

    n = 40
    inner, outer = [], []
    for k in range(n):
        a = 2 * math.pi * k / n
        c, sn = math.cos(a), math.sin(a)
        r = (abs(c) ** PANE_ROUND + abs(sn) ** PANE_ROUND) ** (-1.0 / PANE_ROUND)
        inner.append((PANE_X * r * c, PANE_S[0] + PANE_S[1] * r * sn))
        # Out along the same ray to the posts, the front or the bow.
        t = 10.0
        if abs(sn) > 1e-6:
            t = min(t, ((edge_s[1] if sn > 0 else edge_s[0]) - PANE_S[0]) / (PANE_S[1] * sn))
        if abs(c) > 1e-6:
            # The posts' lines widen aft: step to where the ray meets them.
            lo, hi = 0.0, t
            for _ in range(30):
                mid = (lo + hi) / 2
                if abs(PANE_X * mid * c) < edge_x(PANE_S[0] + PANE_S[1] * mid * sn):
                    lo = mid
                else:
                    hi = mid
            t = lo
        outer.append((PANE_X * t * c, PANE_S[0] + PANE_S[1] * t * sn))
    bm = bmesh.new()

    def ring(loop, drop_):
        return [bm.verts.new(C(x, s_, glass_z(s_, x) - 0.010 - drop_)) for x, s_ in loop]

    ti, to, bi, bo = ring(inner, 0.0), ring(outer, 0.0), ring(inner, PANE_THICK), ring(outer, PANE_THICK)
    for k in range(n):
        m = (k + 1) % n
        for quad in ((ti[k], ti[m], to[m], to[k]), (bi[k], bo[k], bo[m], bi[m]), (ti[k], bi[k], bi[m], ti[m]), (to[k], to[m], bo[m], bo[k])):
            bm.faces.new(quad)
    bmesh.ops.recalc_face_normals(bm, faces=bm.faces[:])
    mesh_object("frame", finish("frame", bm, "frame"), (0, 0, 0))


pane_frame()

# --- the pilot's panel and glare shield (in both cockpits: from the back seat its glare shield
# shows over the RIO's panel)
panel_body("panel_light", "panel_light", PILOT_PANEL, PILOT_OUTLINE, PANEL_DEPTH)
THICK = 0.014
LIP = 0.035
xs = [-0.44, -0.40, -0.366, -0.33, -0.28, -0.14, 0.0, 0.14, 0.28, 0.33, 0.366, 0.40, 0.44]


def pilot_top_v(u):
    """The pilot's panel's top edge (layout v) at layout u."""
    a = abs(u)
    half = PILOT_OUTLINE[1:-2]
    if a <= half[0][0]:
        return half[0][1]
    return interp(half, a)[0]


shade_back = [PILOT_PANEL.point(x * 1000, pilot_top_v(x * 1000)) + PILOT_PANEL.up * THICK + PILOT_PANEL.normal * LIP for x in xs]
lip_low = [PILOT_PANEL.point(x * 1000, pilot_top_v(x * 1000)) + PILOT_PANEL.normal * LIP - PILOT_PANEL.up * 0.006 for x in xs]
front_half = glass_half(SHADE_FRONT, 0.99) - 0.03
shade_front = []
for x in xs:
    xf = x * front_half / 0.44
    shade_front.append(C(xf, SHADE_FRONT, 0.99 + 0.05 * (1.0 - (xf / front_half) ** 2)))
down = Vector((0.0, 0.0, -THICK))
loft("black", "black", [[b, f, f + down, l] for b, f, l in zip(shade_back, shade_front, lip_low)])
# Its cheeks down to the tub's walls.
for side in (0, -1):
    b, f = shade_back[side], shade_front[side]
    cheek = [b, f, Vector((f.x, f.y, C(0, 0, 0.90).z)), Vector((b.x, b.y, C(0, 0, 0.90).z))]
    loft("black", "black", [cheek, [p + Vector((0.012 if side == 0 else -0.012, 0, 0)) for p in cheek]])


def shade_z(x, y):
    """The glare shield's top at eye-frame (x, y)."""
    X = x + EYE[0]
    for i in range(len(xs) - 1):
        if xs[i] <= X <= xs[i + 1]:
            t = (X - xs[i]) / (xs[i + 1] - xs[i])
            b = shade_back[i].lerp(shade_back[i + 1], t)
            f = shade_front[i].lerp(shade_front[i + 1], t)
            k = min(max((y - b.y) / (f.y - b.y), 0.0), 1.0)
            return b.z + (f.z - b.z) * k
    return shade_back[0].z


# Under the pilot's panel: the knee wall and its lid; the rudder pedals.
low_s, low_z = PILOT_PANEL.cabin(0.0, PILOT_OUTLINE[-1][1])
cbox("dark", (-0.44, 0.44), (low_s - 0.12, low_s - 0.10), (PILOT_FLOOR, low_z))
cbox("dark", (-0.44, 0.44), (low_s - 0.12, low_s + 0.004), (low_z - 0.02, low_z))
for dx in (-0.12, 0.12):
    cbox("black", (dx - 0.045, dx + 0.045), (low_s - 0.08, low_s - 0.065), (PILOT_FLOOR + 0.08, PILOT_FLOOR + 0.20))
    cbox("frame", (dx - 0.008, dx + 0.008), (low_s - 0.095, low_s - 0.08), (PILOT_FLOOR, PILOT_FLOOR + 0.14))

# --- the RIO's panel under its hood (in both cockpits: from the front seat its back shows)
panel_body("panel_light", "panel_light", RIO_PANEL, RIO_OUTLINE, PANEL_DEPTH + 0.05)
top_v = RIO_OUTLINE[0][1]
# The radar section standing out of it (the DDD and its controls) and the shroud over its top
# and down its sides: deep at the top, shallower at the radar section's foot.
RP = RIO_PANEL


def rio_slab(mat, corners):
    """A solid through layout corners (u, v, m off the panel) given as sections of points."""
    loft(mat, mat, [[RP.point(u, v, d) for u, v, d in sec] for sec in corners])


ru, rt, rb = 344.0, top_v, RIO_RADAR_V
rio_slab("panel_grey", [[(-ru, rt, 0.0), (-ru, rb, 0.0), (-ru, rb, RIO_RADAR_OUT), (-ru, rt, RIO_RADAR_OUT)],
                        [(ru, rt, 0.0), (ru, rb, 0.0), (ru, rb, RIO_RADAR_OUT), (ru, rt, RIO_RADAR_OUT)]])
deep, shallow, thick = RIO_RADAR_OUT + SHROUD, RIO_RADAR_OUT + 0.03, 14.0
# Its top, a little longer in the middle, and its sides.
rio_slab("black", [[(u, rt - thick, 0.0), (u, rt, 0.0), (u, rt, deep + k), (u, rt - thick, deep + k)]
                   for u, k in ((-ru - thick, 0.0), (-ru * 0.5, 0.025), (ru * 0.5, 0.025), (ru + thick, 0.0))])
for side in (-1, 1):
    rio_slab("black", [[(side * u, rt - thick, 0.0), (side * u, rb + 30.0, 0.0), (side * u, rb + 30.0, shallow), (side * u, rt - thick, deep)]
                       for u in (ru, ru + thick)])
rlow_s, rlow_z = RIO_PANEL.cabin(0.0, RIO_OUTLINE[-1][1])
cbox("dark", (-0.40, 0.40), (rlow_s - 0.10, rlow_s - 0.08), (RIO_FLOOR, rlow_z))
cbox("dark", (-0.40, 0.40), (rlow_s - 0.10, rlow_s + 0.004), (rlow_z - 0.02, rlow_z))
# The RIO's foot rests.
for dx in (-0.12, 0.12):
    cbox("black", (dx - 0.05, dx + 0.05), (rlow_s - 0.06, rlow_s - 0.04), (RIO_FLOOR, RIO_FLOOR + 0.10))

# The live panel's knee panels: a wedge from the vertical panel under each sloped face.
for knee, u0, u1, length in KNEES:
    v0 = knee.top
    secs = []
    for u in (u0, u1):
        a_, b_ = knee.point(u, v0), knee.point(u, v0 + length)
        # Straight down the vertical panel under the fold to the level of the slope's foot.
        foot = a_ + (-MAIN.up) * ((a_.z - b_.z) / MAIN.up.z)
        secs.append([a_, b_, foot])
    loft("panel_grey", "panel_grey", secs)
# The knee boxes: from the panel out to the ledge, down the sloping face, straight down to the
# level of the panel's foot and back to it; the inboard side follows the face's taper.
foot_v = OUTLINE[-1][1]
for side in (-1, 1):
    if RIO:
        slope, fold, length, edges = rio_knee(side), RIO_KNEES[side][1], RIO_KNEE_LEN, ((RIO_WALL, RIO_WALL), (RIO_KNEE_IN, RIO_KNEE_IN))
    else:
        slope, fold, length, edges = knee_part(0.0, side), KNEE_FOLD, KNEE_LEN, ((KNEE_WALL, KNEE_WALL), KNEE_IN)
    secs = []
    for head, foot in edges:
        a_, b_ = slope.point(side * head, slope.top), slope.point(side * foot, slope.top + length)
        low = MAIN.point(side * foot, foot_v)
        secs.append([MAIN.point(side * head, fold), a_, b_, Vector((b_.x, b_.y, low.z)), low])
    loft("panel_light", "panel_light", secs)

# --- the consoles either side of each seat, with their fittings
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


def console_panel(x0, x1, s0, s1, top, rows, h=0.006):
    """A plate over x0..x1, s0..s1 (cabin, m) on a console, its fittings in rows from the front."""
    cbox("black", (x0, x1), (s0, s1), (top, top + h))
    for i, row in enumerate(rows):
        s = s0 + (s1 - s0) * (i + 0.5) / len(rows)
        for j, c in enumerate(row):
            fitting(c, C(x0 + (x1 - x0) * (j + 0.5) / len(row), s, top + h))


for (xin, xout, s0, s1, top), floor in ((PILOT_CONSOLE, PILOT_FLOOR), (RIO_CONSOLE, RIO_FLOOR)):
    for side in (-1, 1):
        x = sorted((side * xin, side * xout))
        if top == PILOT_CONSOLE[4] and side < 0:
            # Round the throttle quadrant's well.
            qx, q0, q1 = THROTTLE[0] - QUADRANT[0], QUADRANT[1], QUADRANT[2]
            qx = (qx, qx + 2 * QUADRANT[0])
            well = top - THROTTLE_PIVOT - 0.04
            for xs, ss, z1 in (((x[0], qx[0]), (s0, s1), top), ((qx[1], x[1]), (s0, s1), top),
                               (qx, (s0, q0), top), (qx, (q1, s1), top), (qx, (q0, q1), well)):
                cbox("panel_grey", xs, ss, (floor, z1))
            for xs, ss in (((x[0] + 0.006, qx[0]), (s0 + 0.01, s1 - 0.01)), ((qx[1], x[1] - 0.006), (s0 + 0.01, s1 - 0.01)),
                           (qx, (s0 + 0.01, q0)), (qx, (q1, s1 - 0.01))):
                cbox("dark", xs, ss, (top, top + 0.004))
            continue
        cbox("panel_grey", x, (s0, s1), (floor, top))
        cbox("dark", (x[0] + 0.006, x[1] - 0.006), (s0 + 0.01, s1 - 0.01), (top, top + 0.004))
PC = PILOT_CONSOLE[4] + 0.004
RC = RIO_CONSOLE[4] + 0.004
# The pilot's left console: the panels round the throttle; his right console: radios, lights.
console_panel(-0.44, -0.28, 4.96, 5.16, PC, ["kKk", "tttt", "ww"])
console_panel(-0.44, -0.41, 4.54, 4.92, PC, ["k", "t", "r"])
console_panel(-0.44, -0.28, 5.20, 5.36, PC, ["tTtg", "kkk"])
console_panel(0.28, 0.44, 4.54, 4.98, PC, ["ww", "kKKk", "tttt", "bbbbb"])
console_panel(0.28, 0.44, 5.02, 5.36, PC, ["tgtg", "KkkK", "kttk"])
# The RIO's: his left console's radios and navigation panels, the right one's.
console_panel(-0.44, -0.26, 6.05, 6.24, RC, ["kKKk", "tttt"])
console_panel(-0.44, -0.26, 6.28, 6.56, RC, ["kkk", "bbbbb", "bbbbb"])
console_panel(-0.44, -0.26, 6.60, 6.90, RC, ["tTtg", "kkKk"])
console_panel(0.26, 0.44, 6.05, 6.36, RC, ["tttt", "kKk", "bbbbb"])
console_panel(0.26, 0.44, 6.40, 6.90, RC, ["kKk", "tttt", "ww", "kkk"])

# The RIO's hand controller on its pedestal on the centre line under the TID: the pedestal's
# box from the floor with its keys' plate, a boot, and the grip standing straight on it (no
# shaft: in his cockpit it is the `stick`, sensing the hand's tilt).
PED = (0.10, 5.87, 6.12, 0.70)
cbox("panel_grey", (-PED[0], PED[0]), (PED[1], PED[2]), (RIO_FLOOR, PED[3]))
cbox("black", (-PED[0] + 0.008, PED[0] - 0.008), (PED[1] + 0.008, PED[2] - 0.008), (PED[3], PED[3] + 0.004))
for j_, c in enumerate("bbbb"):
    fitting(c, C(-0.054 + 0.036 * j_, PED[1] + 0.035, PED[3] + 0.004))
for j_, c in enumerate("tt"):
    fitting(c, C(-0.07 + 0.14 * j_, PED[1] + 0.085, PED[3] + 0.004))
hc = C(0.0, PED[2] - 0.085, PED[3] + 0.004)
mesh_object("rubber", cylinder_mesh("rubber", 0.030, 0.022, "rubber", axis="Z", n=12), hc)
HC_PIVOT = hc + Vector((0, 0, 0.018))
hc_tip = hc + Vector((0.0, -0.006, 0.070))
hc_parts = [
    rod("rubber", hc + Vector((0.0, 0.0, 0.018)), hc_tip + Vector((0.0, -0.006, 0.032)), 0.019, "rubber", n=10),
    mesh_object("red", cylinder_mesh("red", 0.005, 0.005, "red", axis="Z", n=KNOB), hc_tip + Vector((0.0, -0.006, 0.032))),
]
HC_GRIP = (hc_tip + Vector((0.0, -0.006, 0.005)) - HC_PIVOT).z

# --- the live panel's displays
scr = layout["screens"]


def superellipse(theta, a, b, n):
    c, s_ = abs(math.cos(theta)), abs(math.sin(theta))
    return ((c / a) ** n + (s_ / b) ** n) ** (-1.0 / n)


TUBE_N = 4.2
TUBE_BULGE = 0.006
SEG = 56


def tube_mesh(name, a, b, mat, bulge=TUBE_BULGE, rings=7):
    """A square tube's glass: a pillowed face inside the superellipse (half axes a, b, m) in the
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


def square_tube(mat, rect, margin, front, trackcam=False):
    """A square tube (the VDI, the DDD): its pillowed face in a deep bezel `margin` mm wide
    standing `front` m out of the panel, screws at its corners; the pod's picture (`trackcam`)
    just behind the face when it shows through."""
    x, y, w, h = rect
    a, b = w / 2000, h / 2000
    origin = rect_point(rect, 0.0)
    mesh_object(mat, tube_mesh(mat, a, b, mat), rect_point(rect, PLATE + 0.004), PANEL.rot, props={"no_shadow": True})
    if trackcam:
        mesh_object("trackcam", quad_mesh("trackcam", w / 1000, h / 1000, "trackcam"), rect_point(rect, PLATE + 0.002), PANEL.rot, props={"no_shadow": True})
    inner = at_angle(a + 0.0015, b + 0.0015, TUBE_N)
    ha, hb = a + margin / 1000, b + margin / 1000
    outer = at_angle(ha, hb, 12.0)
    mesh_object("black", bezel_ring("black", "black", inner, outer, front, PLATE + 0.002, -0.001), origin, PANEL.rot)
    rim = at_angle(ha + 0.004, hb + 0.004, 12.0)
    mesh_object("dark", bezel_ring("dark", "dark", outer, rim, front - 0.006, 0.0, -0.001), origin, PANEL.rot)
    for sx in (-1, 1):
        for sz in (-1, 1):
            p = Vector((sx * (ha - 0.007), -front, sz * (hb - 0.007)))
            mesh_object("metal", cylinder_mesh("metal", 0.003, 0.0015, "metal", n=KNOB), origin + PANEL.rot @ p, PANEL.rot)


def round_screen(mat, r, offset, trackcam=False, lip=0.004, height=0.010):
    """A round display (a CRT, a standby instrument): a disc in a bezel; the pod's picture
    (`trackcam`, its circle) just behind it when it shows through (in front of the bezel's
    closed cylinder, whose cap would hide it)."""
    bezel(r, offset, height, lip)
    rad = min(r[2], r[3]) / 2000
    face = offset + height + (0.0012 if trackcam else 0.0005)
    mesh_object(mat, disc_mesh(mat, rad, mat, n=40), rect_point(r, face), PANEL.rot, props={"no_shadow": True})
    if trackcam:
        mesh_object("trackcam", disc_mesh("trackcam", rad, "trackcam", n=40), rect_point(r, offset + height + 0.0006), PANEL.rot, props={"no_shadow": True})


# The plates under the panel's groups.
if RIO:
    plates = [
        ("black", (-344.0, -302.0, 688.0, 186.0)),  # the radar panel round the DDD
        ("black", (-344.0, -100.0, 110.0, 196.0)),  # armament
        ("black", (190.0, -102.0, 150.0, 210.0)),  # ECM
        ("black", (126.0, 114.0, 218.0, 92.0)),  # LANTIRN
        ("black", (-124.0, 136.0, 244.0, 44.0)),  # the TID's knobs
    ]
else:
    plates = [
        ("black", (-152.0, -272.0, 264.0, 78.0)),  # the ACM panel
        ("black", (126.0, -272.0, 90.0, 104.0)),  # wing sweep
        ("black", (94.0, 104.0, 154.0, 186.0)),  # DISPLAYS
        ("black", HOOK_PANEL),  # the hook handle's, over it
        ("dark", (-276.0, -268.0, 68.0, 53.0)),  # cautions
        ("dark", (222.0, -268.0, 68.0, 53.0)),  # threats
        ("black", (-352.0, 0.0, 66.0, 152.0)),  # the left knee panel
        ("black", (-350.0, 158.0, 44.0, 106.0)),  # the gear handle's, low on the knee panel
        ("black", (-256.0, 2.0, 174.0, 186.0)),  # the engine tapes and keys
        ("black", (84.0, 0.0, 212.0, 88.0)),  # fuel
    ]
for mat, r in plates:
    use(r)
    panel_box(mat, mat, r, PLATE)
use()
if RIO:
    # The TID's housing out of the panel, down to the hand controller's pedestal; the knee
    # boxes' black faces.
    plane_box("dark", "dark", TID_BOX[0], -0.002, TID_BOX[1])
    use(TID_BOX[0])
    panel_box("black", "black", TID_BOX[0], PLATE - 0.001)
    use()
    foot = TID_BOX[0][1] + TID_BOX[0][3]
    loft("dark", "dark", [[RP.point(sx * 95.0, foot, 0.0), RP.point(sx * 95.0, foot, TID_BOX[1]), C(sx * PED[0], PED[1], PED[3]), C(sx * PED[0], PED[1], PED[3] - 0.08)]
                          for sx in (-1, 1)])
    for side in (-1, 1):
        PANEL = rio_knee(side)
        x0 = min(side * RIO_KNEE_IN, side * RIO_WALL) + 4.0
        plane_box("black", "black", (x0, PANEL.top + 4.0, RIO_WALL - RIO_KNEE_IN - 8.0, RIO_KNEE_LEN - 8.0), 0.0, PLATE - 0.001)
    use()
if not RIO:
    # The centre stack's boxes out of the panel (their faces carry the parts built on them).
    for r, d in STACK:
        plane_box("dark", "dark", r, -0.002, d)
    for r, d in (VDI_BOX, HSD_BOX):
        use(r)
        panel_box("black", "black", r, PLATE)
    use()


def screen(build, name, *a, **k):
    """A display built on its rectangle's panel."""
    use(scr[name.removeprefix("screen_")])
    build(name, scr[name.removeprefix("screen_")], *a, **k)


if RIO:
    screen(square_tube, "screen_ddd", 12.0, 0.020)
    screen(round_screen, "screen_tid", PLATE, trackcam=True, lip=0.008, height=0.012)
    # The hood round the TID's tube.
    tid_r = min(scr["tid"][2], scr["tid"][3]) / 2000
    ring = lambda r: (lambda th: (r * math.cos(th), r * math.sin(th)))
    use(scr["tid"])
    mesh_object("black", bezel_ring("black", "black", ring(tid_r + 0.004), ring(tid_r + 0.010), PLATE + 0.012 + TID_HOOD, PLATE + 0.010, 0.0), rect_point(scr["tid"], 0.0), PANEL.rot)
    use()
    # The ECMD: a square tube at the head of the right knee panel.
    screen(square_tube, "screen_ecmd", 8.0, 0.014)
    screen(round_screen, "adi", PLATE)
else:
    screen(square_tube, "screen_f14vdi", 14.0, 0.018, trackcam=True)
    # The HSD is a square tube like the VDI (its bezel clear of the plates either side).
    screen(square_tube, "screen_f14hsd", 11.0, 0.018)
    screen(round_screen, "adi", PLATE)
    screen(round_screen, "hsi", PLATE)
    screen(round_screen, "screen_rwr", PLATE)

# --- faces and controls
for f in layout["faces"]:
    r = f["rect"]
    use(r)
    off = PLATE
    key = f["key"]
    if f["control"] is None:
        if key.startswith(("Item(Light(", "Item(Readout(", "Item(Lamp(", "Item(Led(", "Item(Gauge", "LedBars", "Item(GearLights", "Item(TomcatLight(", "Item(TomcatReadout(")):
            # A light or window: a black frame round it.
            x, y, w, h = r
            frame("black", (x - 1.5, y - 1.5, w + 3.0, h + 3.0), (x, y, w, h), off - 0.001, off + 0.002)
        # (The hook's emergency placard's legend lies on its striped plate.)
        face(key, r, off + (0.0030 if "EMERG" in key else 0.0005), f["round"])
        if key.startswith("Item(Dial(") and not f["round"]:
            # The tapes' and the fuel quantity's black window frames.
            x, y, w, h = r
            frame("black", (x - 3.0, y - 3.0, w + 6.0, h + 6.0), (x, y, w, h), off - 0.001, off + 0.004)
    elif f["control"] == "Tomcat(Hook)":
        hook_handle(r, off, f["control"])
    elif f["kind"] == "switch":
        switch(key, r, off, f["control"])
    elif f["kind"] == "knob":
        knob_control(key, r, off, f["control"])
    else:
        control(key, r, off, f["control"], f["kind"])

use()
# --- dressing: what the layouts do not carry
if RIO:
    # The DDD's and TID's brightness knobs, the TID's cursor and offset knobs.
    for u, v in ((-146.0, -106.0), (146.0, -106.0), (-146.0, 104.0), (146.0, 104.0)):
        use(at=(u, v))
        knob(u, v, 0.006, 0.010, "dark", offset=PLATE)

    def toggle(u, v):
        """A toggle switch with a pale tip (dressing)."""
        knob(u, v, 0.0045, 0.003, "metal", offset=PLATE)
        knob(u, v, 0.0022, 0.013, "metal", offset=PLATE)
        knob(u, v, 0.0032, 0.005, "white", offset=PLATE + 0.012)

    # The armament panel's face inboard of and under the layout's part: the attack mode and fuze
    # knobs, the delivery switches, the interval and quantity counters, the station selects.
    PANEL = rio_knee(-1)
    for v in (-62.0, -8.0):
        knob(-232.0, v, 0.013, 0.004, "dark", offset=PLATE)
        knob(-232.0, v, 0.010, 0.016, "knob_pale", offset=PLATE)
    # (Clear of the WPN SEL button beside the master caution light.)
    for u in (-232.0, -208.0):
        toggle(u, 34.0)
    for u in (-262.0, -224.0):
        plane_box("tube", "tube", (u, 76.0, 30.0, 14.0), PLATE, PLATE + 0.002)
    for i_ in range(6):
        toggle(-378.0 + 36.0 * i_, 118.0)
    # The ECM panel's: switches inboard of the ECMD, a keyboard under the layout's part.
    PANEL = rio_knee(1)
    for v in (-80.0, -50.0, -20.0):
        toggle(210.0, v)
    for v in (20.0, 60.0):
        knob(210.0, v, 0.009, 0.014, "dark", offset=PLATE)
    for row in range(2):
        for i_ in range(7):
            plane_box("dark", "dark", (196.0 + 27.0 * i_, 112.0 + 17.0 * row, 20.0, 13.0), PLATE, PLATE + 0.004)
    use()
else:
    # On the VDI's housing: the three pale knobs down the strip left of the tube under a
    # placard, the HUD's and VDI's trim knobs on the strip right of it.
    use(VDI_BOX[0])
    for v in (-96.0, -62.0, -28.0):
        knob(-102.0, v, 0.007, 0.013, "knob_pale", offset=PLATE)
    plane_box("metal", "metal", (-110.0, -186.0, 16.0, 62.0), PLATE, PLATE + 0.002)
    for v in (-106.0, -52.0):
        knob(102.0, v, 0.006, 0.011, "dark", offset=PLATE)
    # The hook panel's emergency release placard left of the slot: a plate in yellow and black
    # stripes round a black field (the layout's EMERG DOWN PULL TURN legend lies on it) with the
    # turn's arrow under the words, and two screws.
    hx, hy = HOOK_PANEL[:2]
    use(HOOK_PANEL)
    pl = (hx + 4.0, hy + 20.0, 32.0, 50.0)
    plane_box("yellow", "yellow", pl, PLATE, PLATE + 0.0010)
    bm = bmesh.new()
    bar, lean, n = 4.0, pl[3], 9
    for k in range(n):
        x0 = pl[0] - lean + (pl[2] + lean) * (k + 0.2) / n
        xs_ = [min(max(x, pl[0]), pl[0] + pl[2]) for x in (x0, x0 + bar, x0 + bar + lean, x0 + lean)]
        if xs_[1] - xs_[0] < 0.1 and xs_[2] - xs_[3] < 0.1:
            continue
        quad = ((xs_[0], pl[1] + pl[3]), (xs_[1], pl[1] + pl[3]), (xs_[2], pl[1]), (xs_[3], pl[1]))
        bm.faces.new([bm.verts.new(point(u, v, PLATE + 0.0014)) for u, v in quad])
    mesh_object("black", finish("black", bm, "black"), (0, 0, 0), props={"no_shadow": True})
    plane_box("black", "black", (pl[0] + 4.0, pl[1] + 4.0, pl[2] - 8.0, pl[3] - 8.0), PLATE, PLATE + 0.0024)
    # The arrow: an arc over the field's foot, its head at the left end.
    bm = bmesh.new()
    ac, ar, aw = (pl[0] + pl[2] / 2, pl[1] + pl[3] - 9.0), 7.0, 0.9
    arc = [math.radians(a) for a in range(200, 341, 20)]
    for a0, a1 in zip(arc, arc[1:]):
        ring_ = [(ac[0] + r_ * math.cos(a), ac[1] + r_ * math.sin(a)) for a, r_ in ((a0, ar - aw), (a1, ar - aw), (a1, ar + aw), (a0, ar + aw))]
        bm.faces.new([bm.verts.new(point(u, v, PLATE + 0.0030)) for u, v in ring_])
    tip = (ac[0] + ar * math.cos(arc[0]), ac[1] + ar * math.sin(arc[0]))
    head = (tip, (tip[0] + 3.4, tip[1] - 0.6), (tip[0] + 0.4, tip[1] - 3.4))
    bm.faces.new([bm.verts.new(point(u, v, PLATE + 0.0030)) for u, v in head])
    mesh_object("white", finish("white", bm, "white"), (0, 0, 0), props={"no_shadow": True})
    for u, v in ((hx + 5.0, hy + 5.0), (hx + 8.0, hy + 75.0)):
        knob(u, v, 0.003, 0.0015, "metal", offset=PLATE)
    # The fire handles either side of the ACM panel: a stem out of the panel and a yellow grip
    # across it with black stripes, turned a little towards the pilot.
    use()
    for u, v in FIRE_HANDLES:
        turn = Euler((0.0, 0.0, math.radians(-14.0 if u < 0.0 else 14.0))).to_quaternion()
        rot = PANEL.rot @ turn
        at = point(u, v)
        gw, gh, reach, deep = 0.046, 0.020, 0.038, 0.014
        mesh_object("dark", box_mesh("dark", (0.026, reach, 0.012), "dark", (0.0, -reach / 2, 0.0)), at, rot)
        mesh_object("yellow", box_mesh("yellow", (gw, deep, gh), "yellow", (0.0, -reach - deep / 2, 0.0)), at, rot)
        # The stripes: slanted bars on the grip's face, cut off at its ends.
        bm = bmesh.new()
        y = -reach - deep - 0.0004
        bar, lean, n = 0.0046, 0.010, 6
        for k in range(n):
            x0 = -gw / 2 - lean + (gw + lean) * (k + 0.25) / n
            xs_ = [min(max(x, -gw / 2), gw / 2) for x in (x0, x0 + bar, x0 + bar + lean, x0 + lean)]
            if xs_[1] - xs_[0] < 1e-4 and xs_[2] - xs_[3] < 1e-4:
                continue
            bm.faces.new([bm.verts.new(p) for p in ((xs_[0], y, -gh / 2), (xs_[1], y, -gh / 2), (xs_[2], y, gh / 2), (xs_[3], y, gh / 2))])
        mesh_object("black", finish("black", bm, "black"), at, rot, props={"no_shadow": True})

use()
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
floors = {EYE: RIO_FLOOR if RIO else PILOT_FLOOR, OTHER_EYE: PILOT_FLOOR if RIO else RIO_FLOOR}
for shift, eye in ((Vector((0.0, 0.0, 0.0)), EYE), (to_other, OTHER_EYE)):
    floor_z = floors[eye] - EYE[2]
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

stick = objs["stick"]
throttle = objs["throttle"]
full = objs["throttle_full"]
if RIO:
    # The RIO has no flight controls: his hand controller is the `stick` (it slews the pod and
    # the TID's cursor).
    for c in list(stick.children):
        drop(c)
    stick.location = HC_PIVOT
    for o in hc_parts:
        o.parent = stick
        o.location = o.location - HC_PIVOT
    stick["grip"] = HC_GRIP
    stick["tilt"] = True
    for o in [throttle, full, objs["control_Action_Gear_.001"]]:
        drop(o)
else:
    # Centre stick: pivot on the floor on the centre line, the grip raised on a shaft, a boot.
    pivot = C(0.0, STICK_S, PILOT_FLOOR + 0.01)
    stick.location = pivot
    for c in stick.children:
        c.location.z += STICK_RAISE
    stick["grip"] = float(stick["grip"]) + STICK_RAISE
    rod("metal", (0, 0, 0), (0, 0, STICK_RAISE + 0.02), 0.011, "metal", stick)
    rod("rubber", pivot, pivot + Vector((0, 0, 0.06)), 0.04, "rubber")
    box("dark", "dark", pivot + Vector((0.0, 0.0, -0.004)), (0.12, 0.12, 0.008))
    # Throttle: the two engines' levers swinging together about a pivot sunk in the left
    # console, up through the slots of the quadrant's cover; a grip on each (after the F-14's:
    # a rounded body lying along the hand, nose up, on the lever under its aft end). The
    # inboard grip's inboard face carries the thumb's switches: the wing sweep hat aft, the
    # speed brake switch, the microphone hat ahead; a button on each grip's edge under the
    # fingers (two on the outboard one: PLM, CAGE), the lights' toggle on its outboard face.
    # Built upright in millimetres (x inboard, y forward, z up the levers, the hand's place at
    # the origin), then leant aft to idle about the pivot.
    for c in list(throttle.children):
        drop(c)
    throttle.location = C(*THROTTLE)
    full.location = throttle.location + Vector((0.0, THROTTLE_TRAVEL, 0.0))
    top_z = PILOT_CONSOLE[4]
    up = THROTTLE[2] - top_z + THROTTLE_PIVOT
    pivot = Vector((0.0, THROTTLE_TRAVEL / 2, -up))
    swing = math.atan2(THROTTLE_TRAVEL / 2, up)
    arm = math.hypot(THROTTLE_TRAVEL / 2, up)
    empty("throttle_pivot", throttle.location + pivot, props={"anchor": "throttle_pivot"})
    IDLE = Matrix.Translation(pivot) @ Matrix.Rotation(swing, 4, "X") @ Matrix.Translation((0.0, 0.0, arm)) @ Matrix.Scale(0.001, 4)

    def part(name, me, m=Matrix.Identity(4), smooth=False):
        """A throttle part from its mesh in the upright frame (mm), moved by m first."""
        me.transform(IDLE @ m)
        for pl in me.polygons:
            pl.use_smooth = smooth
        return mesh_object(name, me, (0, 0, 0), parent=throttle)

    def hull(points):
        """The convex hull of points in a plane, anticlockwise."""
        pts = sorted(set(points))

        def half(seq):
            h = []
            for q in seq:
                while len(h) >= 2 and (h[-1][0] - h[-2][0]) * (q[1] - h[-2][1]) - (h[-1][1] - h[-2][1]) * (q[0] - h[-2][0]) <= 0:
                    h.pop()
                h.append(q)
            return h[:-1]

        return half(pts) + half(pts[::-1])

    # The grip's side view (y, z): the aft lobe under the thumb, the nose, the palm's swell
    # over them; its long axis GRIP_RISE nose up.
    GRIP_RISE = math.radians(30.0)
    GRIP_LONG = 59.0
    along = Vector((math.cos(GRIP_RISE), math.sin(GRIP_RISE)))
    below = Vector((along.y, -along.x))
    aft, nose = -along * GRIP_LONG / 2, along * GRIP_LONG / 2
    lobes = [(aft, 21.0), (nose, 13.0), (-below * 7.0 - along * 7.0, 21.0)]
    side_view = hull([
        (round(c.x + r * math.cos(2 * math.pi * i / 20), 2), round(c.y + r * math.sin(2 * math.pi * i / 20), 2))
        for c, r in lobes for i in range(20)
    ])
    # Half the grips' thickness: the outboard (left engine's) one slimmer, both close to the gap
    # between them.
    GRIP_HALF = {-1: 14.0, 1: 21.0}
    GRIP_GAP = 3.0

    def yz(p, x=0.0):
        return Vector((x, p.x, p.y))

    def lathe(name, mat, c, d, steps, n=KNOB):
        """A round part standing on c along d: (radius, height) steps, closed on top."""
        bm = bmesh.new()
        rot = Vector((0, 0, 1)).rotation_difference(d.normalized()).to_matrix()
        rings = []
        for r, h in steps:
            rings.append([bm.verts.new(c + rot @ Vector((r * math.cos(2 * math.pi * i / n), r * math.sin(2 * math.pi * i / n), h))) for i in range(n)])
        for a_, b_ in zip(rings, rings[1:]):
            for i in range(n):
                j = (i + 1) % n
                bm.faces.new((a_[i], a_[j], b_[j], b_[i]))
        bm.faces.new(rings[-1])
        bmesh.ops.recalc_face_normals(bm, faces=bm.faces[:])
        return part(name, finish(name, bm, mat))

    def slab(name, mat, c, x_axis, y_axis, size):
        """A box at c: `size` along x_axis, y_axis and their normal (standing on c along it)."""
        z_axis = x_axis.cross(y_axis).normalized()
        m = Matrix((x_axis.normalized(), z_axis.cross(x_axis).normalized(), z_axis)).transposed().to_4x4()
        me = box_mesh(name, size, mat, (0.0, 0.0, size[2] / 2))
        return part(name, me, Matrix.Translation(c) @ m)

    coloured("throttle_grip", "panel_grey", (0.70, 0.71, 0.69), 0.55)
    INBOARD, AHEAD, UPWARD = Vector((1, 0, 0)), yz(along), yz(-below)
    for sx in (-1, 1):
        cx = sx * LEVER_X * 1000
        gh = GRIP_HALF[sx]
        gx = sx * (GRIP_GAP + gh)
        # The grip: its side view through its thickness, the edges rounded.
        bm = bmesh.new()
        rings = []
        for x, k in ((-gh, 0.80), (-gh + 2.5, 0.94), (-gh + 7.0, 1.0), (gh - 7.0, 1.0), (gh - 2.5, 0.94), (gh, 0.80)):
            rings.append([bm.verts.new((gx + x, y * k, z * k)) for y, z in side_view])
        n = len(side_view)
        for a_, b_ in zip(rings, rings[1:]):
            for i in range(n):
                j = (i + 1) % n
                bm.faces.new((a_[i], a_[j], b_[j], b_[i]))
        bm.faces.new(rings[0][::-1])
        bm.faces.new(rings[-1])
        bmesh.ops.recalc_face_normals(bm, faces=bm.faces[:])
        part("throttle_grip", finish("throttle_grip", bm, "throttle_grip"), smooth=True)
        # Its lever: a blade from the pivot up into the grip's aft end.
        foot, head = Vector((cx, 0.0, -arm * 1000)), yz(aft + below * 6.0, cx)
        d = (head - foot).normalized()
        me = box_mesh("throttle_lever", (7.0, 20.0, (head - foot).length), "frame", (0.0, 0.0, (head - foot).length / 2))
        part("throttle_lever", me, Matrix.Translation(foot) @ Vector((0, 0, 1)).rotation_difference(d).to_matrix().to_4x4())
        # The buttons under the fingers, on the grip's lower edge ahead.
        for k in ((-9.0, 9.0) if sx < 0 else (0.0,)):
            at_ = yz(along * (6.0 + k) + below * 15.0, gx)
            lathe("throttle_button", "dark", at_, yz(below), [(6.5, 0.0), (6.5, 5.0), (4.5, 5.0), (4.5, 9.0)])
    face_x = (GRIP_GAP + 2 * GRIP_HALF[1]) * 0.999
    # The wing sweep hat on its black dial.
    lathe("throttle_sweep", "black", yz(aft, face_x), INBOARD, [(16.0, 0.0), (16.0, 1.5), (10.5, 2.0), (9.5, 8.0), (4.0, 8.5), (3.5, 11.5)], n=12)
    # The microphone hat.
    lathe("throttle_mic", "black", yz(nose, face_x), INBOARD, [(11.5, 0.0), (11.5, 1.5), (7.5, 2.0), (6.5, 7.0), (3.0, 7.5), (2.5, 10.0)], n=10)
    # The speed brake switch: a ribbed slide between its two rails.
    mid = yz(along * 2.0, face_x)
    slab("throttle_brake", "black", mid, AHEAD, UPWARD, (15.0, 20.0, 6.0))
    for k in (-6.0, -2.0, 2.0, 6.0):
        slab("throttle_brake", "dark", mid + AHEAD * k + INBOARD * 6.0, AHEAD, UPWARD, (1.6, 20.0, 1.6))
    for k in (-10.5, 10.5):
        slab("throttle_rail", "metal", mid + AHEAD * k, AHEAD, UPWARD, (3.0, 25.0, 4.0))
    # The exterior lights' toggle on the outboard grip's outboard face.
    out_ = yz(nose - along * 4.0, -(GRIP_GAP + 2 * GRIP_HALF[-1]) * 0.999)
    lathe("throttle_toggle", "dark", out_, -INBOARD, [(6.0, 0.0), (6.0, 3.0), (4.0, 3.5)])
    lathe("throttle_toggle", "metal", out_ - INBOARD * 3.0, -INBOARD + AHEAD * 0.35, [(1.8, 0.0), (2.2, 12.0)], n=6)
    # The pivot's axle across the well, a hub on it for each lever.
    part("throttle_axle", cylinder_mesh("throttle_axle", 8.0, 2000 * QUADRANT[0], "metal", axis="Z", n=10),
         Matrix.Translation((-1000 * QUADRANT[0], 0.0, -arm * 1000)) @ Matrix.Rotation(math.radians(90.0), 4, "Y"))
    # The quadrant's cover sunk in the console round the levers' slots: the strips between and
    # beside them, the ends; the slots as long as the levers swing (with a finger's clearance).
    cover = top_z - QUADRANT_SUNK
    cross = []
    for t_ in (0.0, 1.0):
        rot = Matrix.Rotation(swing * (1.0 - 2.0 * t_), 3, "X")
        a_ = rot @ Vector((0.0, 0.0, 0.0))
        b_ = rot @ ((yz(aft + below * 6.0) / 1000) + Vector((0.0, 0.0, arm)))
        k = (THROTTLE_PIVOT - QUADRANT_SUNK) / (b_.z - a_.z)
        cross.append((a_ + (b_ - a_) * k).y)
    pivot_s = THROTTLE[1] - THROTTLE_TRAVEL / 2
    slot0, slot1 = pivot_s - max(cross) - 0.022, pivot_s - min(cross) + 0.022
    qx0, qx1 = THROTTLE[0] - QUADRANT[0], THROTTLE[0] + QUADRANT[0]
    edges = [qx0, THROTTLE[0] - LEVER_X - SLOT / 2, THROTTLE[0] - LEVER_X + SLOT / 2,
             THROTTLE[0] + LEVER_X - SLOT / 2, THROTTLE[0] + LEVER_X + SLOT / 2, qx1]
    for x0, x1 in zip(edges[::2], edges[1::2]):
        cbox("black", (x0, x1), (QUADRANT[1], QUADRANT[2]), (cover - 0.003, cover))
    for s0_, s1_ in ((QUADRANT[1], slot0), (slot1, QUADRANT[2])):
        cbox("black", (qx0, qx1), (s0_, s1_), (cover - 0.003, cover))
    # The well's dark lining under the cover.
    cbox("black", (qx0 + 0.001, qx1 - 0.001), (QUADRANT[1] + 0.001, QUADRANT[2] - 0.001), (top_z - THROTTLE_PIVOT - 0.04, top_z - THROTTLE_PIVOT - 0.038))
    print(f"throttle: swing {math.degrees(2 * swing):.1f} deg on a {arm:.3f} m arm, slots s {slot0:.3f}..{slot1:.3f}")
    # The gear handle on the left knee panel.
    g = objs["control_Action_Gear_.001"]
    use(at=(-328.0, 211.0))
    g.location = point(-328.0, 211.0, 0.0)
    g.rotation_mode = "QUATERNION"
    g.rotation_quaternion = PANEL.rot
    use()

# The HUD on the glare shield: its body sunk in the shield, the combiner glass (the pilot's
# cockpit's HUD glass; from the back seat its frame only), the posts and the top bar round it.
hud = next(o for n, o in objs.items() if n.startswith("hud_glass"))
hud_y = C(0, PILOT_EYE[1], 0).y + HUD_Y
hud_z = PILOT_EYE[2] - EYE[2]
half_x, glass_lo, glass_hi = HUD_GLASS


def hud_outline(grow=0.0):
    """The combiner's outline (x, z) anticlockwise from the lower left as the pilot sees it: a
    plate with its top corners clipped; `grow` m outside it."""
    hx = half_x + grow
    lo_z, hi_z = hud_z + glass_lo - grow, hud_z + glass_hi + grow
    cx, cz = hx * HUD_CLIP[0], (hi_z - lo_z) * HUD_CLIP[1]
    return [(-hx, lo_z), (hx, lo_z), (hx, hi_z - cz), (hx - cx, hi_z), (-hx + cx, hi_z), (-hx, hi_z - cz)]


if RIO:
    drop(hud)
else:
    # The kept glass's picture (its UVs across its rectangle) on the new outline.
    me = hud.data
    xs_ = [v.co.x for v in me.vertices]
    zs_ = [v.co.z for v in me.vertices]
    # (Without UVs the game lays the picture over the mesh's bounds: the outline's are the
    # rectangle's.)
    uvs = me.uv_layers.active.data if me.uv_layers.active else None
    corner = {}
    for l in me.loops if uvs else ():
        co = me.vertices[l.vertex_index].co
        corner[(co.x > (min(xs_) + max(xs_)) / 2, co.z > (min(zs_) + max(zs_)) / 2)] = uvs[l.index].uv.copy()
    facing = sum((pl.normal.y for pl in me.polygons), 0.0)

    def hud_uv(x, z):
        tx = (x + half_x) / (2 * half_x)
        tz = (z - hud_z - glass_lo) / (glass_hi - glass_lo)
        low = corner[(False, False)].lerp(corner[(True, False)], tx)
        high = corner[(False, True)].lerp(corner[(True, True)], tx)
        return low.lerp(high, tz)

    bm = bmesh.new()
    outline = hud_outline()
    vs = [bm.verts.new((x, hud_y, z)) for x, z in outline]
    face_ = bm.faces.new(vs)
    face_.normal_update()
    if face_.normal.y * facing < 0.0:
        face_.normal_flip()
    if uvs:
        uv = bm.loops.layers.uv.new()
        for l in face_.loops:
            l[uv].uv = hud_uv(l.vert.co.x, l.vert.co.z)
    bmesh.ops.triangulate(bm, faces=bm.faces[:])
    new = bpy.data.meshes.new(me.name)
    bm.to_mesh(new)
    bm.free()
    for m in me.materials:
        new.materials.append(m)
    hud.location = (0.0, 0.0, 0.0)
    hud.data = new
# Its arms: a thin strut up each side from the projector's body to the clipped corner.
post_x = half_x + HUD_BAR / 2
for sx in (-1, 1):
    foot = shade_z(sx * post_x, hud_y)
    top = hud_z + glass_hi - (glass_hi - glass_lo) * HUD_CLIP[1]
    rod("frame", (sx * post_x, hud_y, foot), (sx * post_x, hud_y, top), HUD_BAR / 2, "frame", n=6)
    rod("frame", (sx * post_x, hud_y, hud_z + glass_lo + 0.02), (sx * post_x * 0.9, hud_y + 0.07, foot), HUD_BAR / 2, "frame", n=6)
hud_low = shade_z(0.0, hud_y + 0.03)
body_top = hud_z + glass_lo - 0.004
box("dark", "dark", (0.0, hud_y + HUD_BODY[1] / 2 - 0.01, (hud_low - 0.01 + body_top) / 2), (HUD_BODY[0], HUD_BODY[1], body_top - hud_low + 0.01))
# The approach indexer left of the HUD on a stalk (the pilot's only).
for o in [o for n, o in objs.items() if n.startswith("approach")]:
    if RIO:
        drop(o)
        continue
    at = Vector((-(post_x + 0.045), hud_y + 0.01, 0.0))
    at.z = shade_z(at.x, at.y) + 0.06
    o.location = at
    box("dark", "dark", at + Vector((0, 0.007, 0)), (0.028, 0.010, 0.068))
    box("dark", "dark", at + Vector((0, 0.007, -0.047)), (0.010, 0.010, 0.03))
# Flood lamps: either side behind the head, one under the hood over the panel.
objs["lamp"].location = Vector((-0.25, -0.05, -0.12))
objs["lamp.001"].location = Vector((0.25, -0.05, -0.12))
objs["lamp.002"].location = point(0.0, -150.0, 0.14)

bpy.data.orphans_purge(do_recursive=True)
bpy.context.view_layer.update()
print(f"seat {'rio' if RIO else 'pilot'}: stick grip {stick['grip']:.3f} m at {tuple(round(c, 3) for c in stick.location)}; "
      f"pilot panel bottom s {low_s:.3f} z {low_z:.3f}; RIO panel bottom s {rlow_s:.3f} z {rlow_z:.3f}; "
      f"HUD y {hud_y:.3f} on the shield at z {hud_low + EYE[2]:.3f}")
bpy.ops.wm.save_as_mainfile(filepath=out)
print("built", out, "objects", len(bpy.data.objects))
