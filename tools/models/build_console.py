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
import os
import sys

import bpy
from mathutils import Euler, Vector

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

import console_lib as lib  # noqa: E402
from console_lib import Surface, box, column, screen  # noqa: E402

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

lib.setup(("screen_scope", "screen_board", "screen_eo", "screen_info"))

# --- the shelter, the chair under the eye, the hand controller right of it
lib.shelter(ROOM_X, ROOM_BACK, ROOM_FRONT, FLOOR, CEILING, WALL)
lib.chair()
lib.hand_controller(STICK, FLOOR)

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
    column(panel, -0.34, [(f"Console(Refill, {k})", f"RLD{k}", v)])
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
