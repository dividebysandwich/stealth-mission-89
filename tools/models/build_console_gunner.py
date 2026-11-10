# Builds the AC-130's gunner's console master (assets-src/blend/cockpit_gunner.blend): the
# fire control officer's seat in the booth in the cargo hold, from nothing:
#   blender -b --factory-startup --python-exit-code 1 --python tools/models/build_console_gunner.py -- \
#       assets-src/blend/cockpit_gunner.blend
# then tools/models/export.sh cockpit_gunner. See src/gunner.rs and src/cockpit3d/model.rs for
# what the game binds to.
#
# Built: the hold round the booth (the fuselage's ribbed inside, a curved roof, the floor with
# its rollers, bulkheads fore and aft, a dim blue-green dome light), the operator's chair, the
# console: an upright panel with the big targeting pod screen (`trackcam` picture behind the
# `screen_eo` overlay), the gun and camera buttons beside it, the aircraft's data on a small
# screen (`screen_info`) turned towards the operator on the right, a shelf below. Right of the
# chair a pedestal with the hand controller (`stick`) that slews the pod, its trigger fires.
#
# Blender frame: eye at the origin, x right, y forward, z up, metres. A control's face points
# along its local -Y (towards the operator), local Z is up its panel.
import math
import os
import sys

import bpy
from mathutils import Euler, Vector

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

import console_lib as lib  # noqa: E402
from console_lib import Surface, box, column, screen  # noqa: E402

out = sys.argv[sys.argv.index("--") + 1]

# The hold: the fuselage's inside a circle of this radius round (HOLD_X, HOLD_Z) above the
# floor, from AFT to FORE (y); the booth on the hold's left side.
HOLD_R = 1.65
HOLD_X, HOLD_Z = 0.55, 0.15
FLOOR = -1.20
AFT, FORE = -2.6, 1.6
SKIN = 0.03
SIDES = 20
RIB_EVERY = 0.5
# The upright panel: a vertical surface this far ahead of the eye, its centre this high.
PANEL_Y = 0.95
PANEL_Z = -0.02
# The big screen (77:60, the tracking camera's picture) and the aircraft's data screen (4:3).
EO = (0.80, 0.80 * 60.0 / 77.0)
INFO = (0.24, 0.18)
INFO_AT = (0.68, PANEL_Y - 0.14, PANEL_Z - 0.12)
INFO_TURN = -32.0
# The hand controller's pivot, on its pedestal right of the chair.
STICK = Vector((0.34, 0.24, -0.48))

lib.setup(("screen_eo", "screen_info"))
lib.material("hold", (0.16, 0.18, 0.15))
lib.material("insulation", (0.22, 0.21, 0.17))
lib.material("floor", (0.07, 0.07, 0.07))


def hold():
    """The fuselage's inside above the floor: skin panels round the circle, ribs, stringers."""
    length = FORE - AFT
    mid = (AFT + FORE) / 2
    for k in range(SIDES):
        a0 = math.pi * 2 * k / SIDES
        a1 = math.pi * 2 * (k + 1) / SIDES
        am = (a0 + a1) / 2
        x = HOLD_X + math.cos(am) * (HOLD_R + SKIN / 2)
        z = HOLD_Z + math.sin(am) * (HOLD_R + SKIN / 2)
        if z < FLOOR - 0.05:
            continue
        chord = 2 * HOLD_R * math.sin(math.pi / SIDES) + 0.01
        rot = Euler((0.0, -(am - math.pi / 2), 0.0))
        box("insulation", "insulation", (x, mid, z), (chord, length, SKIN), rot)
        # A stringer along each panel's edge.
        xe = HOLD_X + math.cos(a0) * (HOLD_R - 0.03)
        ze = HOLD_Z + math.sin(a0) * (HOLD_R - 0.03)
        if ze > FLOOR:
            box("hold", "hold", (xe, mid, ze), (0.04, length, 0.04), Euler((0.0, -a0, 0.0)))
    # The ribs: rings of short boxes every RIB_EVERY metres.
    y = AFT + RIB_EVERY / 2
    while y < FORE:
        for k in range(SIDES):
            a = math.pi * 2 * (k + 0.5) / SIDES
            x = HOLD_X + math.cos(a) * (HOLD_R - 0.05)
            z = HOLD_Z + math.sin(a) * (HOLD_R - 0.05)
            if z < FLOOR:
                continue
            chord = 2 * HOLD_R * math.sin(math.pi / SIDES)
            box("hold", "hold", (x, y, z), (chord, 0.05, 0.08), Euler((0.0, -(a - math.pi / 2), 0.0)))
        y += RIB_EVERY
    # The floor with its roller tracks, the bulkheads fore and aft.
    box("floor", "floor", (HOLD_X, mid, FLOOR - 0.03), (2 * HOLD_R, length, 0.06))
    for s in (-1.1, -0.4, 0.4, 1.1):
        box("metal", "metal", (HOLD_X + s, mid, FLOOR + 0.01), (0.06, length, 0.02))
    for y, n in ((FORE + 0.02, "black"), (AFT - 0.02, "dark")):
        box(n, n, (HOLD_X, y, HOLD_Z), (2 * HOLD_R + 0.2, 0.04, 2 * HOLD_R + 0.2))
    # Cable runs along the roof, a dome light (blue-green, dim) over the booth.
    box("frame", "frame", (HOLD_X - 0.5, mid, HOLD_Z + HOLD_R - 0.18), (0.10, length, 0.05))
    box("frame", "frame", (HOLD_X + 0.5, mid, HOLD_Z + HOLD_R - 0.18), (0.10, length, 0.05))
    lib.empty("lamp", (0.0, 0.3, HOLD_Z + HOLD_R - 0.3), props={"lamp": [0.10, 0.45, 0.40], "range": 4.0})
    box("metal", "metal", (0.0, 0.3, HOLD_Z + HOLD_R - 0.22), (0.18, 0.18, 0.03))


hold()
lib.chair()
lib.hand_controller(STICK, FLOOR)

# --- the booth: the upright panel, a shelf under it, its sides and the body down to the floor
panel = Surface((0.0, PANEL_Y, PANEL_Z), Euler((0.0, 0.0, 0.0)))
box("dark", "dark", panel.at(0.0, 0.0, -0.02), (1.6, 0.04, 0.95), panel.rot)
box("frame", "frame", panel.at(0.0, 0.49, 0.0), (1.62, 0.04, 0.03), panel.rot)
box("frame", "frame", panel.at(0.0, -0.49, 0.0), (1.62, 0.04, 0.03), panel.rot)
shelf_z = PANEL_Z - 0.55
box("panel_grey", "panel_grey", (0.0, PANEL_Y - 0.25, shelf_z), (1.6, 0.5, 0.04))
box("black", "black", (0.0, PANEL_Y - 0.05, (shelf_z + FLOOR) / 2), (1.6, 0.2, shelf_z - FLOOR))
for s in (-1, 1):
    box("dark", "dark", (s * 0.82, PANEL_Y - 0.3, PANEL_Z - 0.2), (0.04, 0.6, 1.3))

# --- the big screen; the guns' buttons left of it, the pod's right of it
screen(panel, "screen_eo", 0.0, 0.08, EO, behind="trackcam")
column(panel, -0.62, [
    ("Console(Gun, 1)", "40MM", 0.30),
    ("Console(Gun, 2)", "105MM", 0.24),
    ("Console(Leave)", "LEAVE", -0.30),
])
column(panel, 0.62, [
    ("Console(Lock)", "TRACK", 0.30),
    ("Console(Flir)", "FLIR", 0.20),
    ("Console(ZoomIn)", "ZOOM+", 0.12),
    ("Console(ZoomOut)", "ZOOM-", 0.07),
])

# --- the aircraft's data screen in its own housing on the shelf, turned towards the operator
info = Surface(INFO_AT, Euler((math.radians(-20.0), 0.0, math.radians(INFO_TURN))))
box("dark", "dark", info.at(0.0, 0.0, -0.03), (INFO[0] + 0.07, 0.06, INFO[1] + 0.07), info.rot)
screen(info, "screen_info", 0.0, 0.0, INFO)

bpy.ops.wm.save_as_mainfile(filepath=out)
