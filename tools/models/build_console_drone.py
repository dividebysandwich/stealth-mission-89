# Builds the drone command center's console (assets-src/blend/cockpit_drone.blend): the operator's
# seat inside the bunker, from nothing:
#   blender -b --factory-startup --python-exit-code 1 --python tools/models/build_console_drone.py -- \
#       assets-src/blend/cockpit_drone.blend
# then tools/models/export.sh cockpit_drone. See src/station/drone/ and src/cockpit3d/model.rs for
# what the game binds to.
#
# Built: the shelter and the chair of the air defence console (tools/models/console_lib.py), the
# console: an upright panel with the drone's camera across it as wide as the map
# (`trackcam_wide` behind the `screen_feed` overlay), the drone list (`screen_info`) in its own housing turned
# towards the operator, and on the sloping desk one wide ground control station map
# (`screen_map`) with the camera's picture-in-picture (`trackcam_pip`) behind its top right
# corner and the buttons that fly the drone either side of it. Right of the chair the stick on its pedestal, left of it the throttle quadrant
# (`throttle` at idle, the `throttle_full` anchor ahead of it).
#
# Blender frame: eye at the origin, x right, y forward, z up, metres.
import math
import os
import sys

import bpy
from mathutils import Euler, Vector

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

import console_lib as lib  # noqa: E402
from console_lib import Surface, box, column, empty, mesh_object, quad_mesh, screen  # noqa: E402

out = sys.argv[sys.argv.index("--") + 1]

# The bunker's room: inside walls (x left/right, y back/front, z floor/ceiling).
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
# The map (src/cockpit3d/displays.rs MAP_CANVAS, 640 x 244 canvas pixels) and the picture in
# picture in its top right corner (`drone::PIP`: 154 x 65 pixels, 6 in from the edges).
MAP = (0.84, 0.84 * 244.0 / 640.0)
MAP_PX = MAP[0] / 640.0
PIP = (154 * MAP_PX, 65 * MAP_PX)
PIP_AT = (MAP[0] / 2 - (6 + 77) * MAP_PX, MAP[1] / 2 - (6 + 32.5) * MAP_PX)
# The camera screen as wide as the map, the shape of its picture (`trackcam::WIDE`, 180:76);
# the drone list (4:3) right of it.
FEED = (MAP[0], MAP[0] * 76.0 / 180.0)
INFO = (0.24, 0.18)
INFO_AT = (0.64, PANEL_Y - 0.10, PANEL_Z)
INFO_TURN = -32.0
# The button columns either side of the map.
BUTTONS_U = 0.50
# The stick's pivot right of the chair, the throttle's handle at idle left of it and how far
# forward it goes to full (its grip GRIP_HEIGHT, src/cockpit3d/levers.rs, above the stick's).
STICK = Vector((0.34, 0.20, -0.50))
THROTTLE = Vector((-0.34, 0.12, -0.44))
THROTTLE_TRAVEL = 0.16

lib.setup(("screen_feed", "screen_info", "screen_map", "trackcam_wide", "trackcam_pip"))

# --- the bunker's room, the chair under the eye, the stick right of it
lib.shelter(ROOM_X, ROOM_BACK, ROOM_FRONT, FLOOR, CEILING, WALL)
lib.chair()
lib.hand_controller(STICK, FLOOR)

# --- the throttle quadrant left of the chair: a pedestal, the slot, the lever sliding along it
q = THROTTLE + Vector((0.0, THROTTLE_TRAVEL / 2, -0.06))
box("dark", "dark", (q.x, q.y, (FLOOR + q.z) / 2), (0.14, 0.30, q.z - FLOOR))
box("frame", "frame", (q.x, q.y, q.z + 0.005), (0.15, 0.31, 0.01))
box("black", "black", (q.x, q.y, q.z + 0.011), (0.025, THROTTLE_TRAVEL + 0.03, 0.004))
for k in range(5):
    y = THROTTLE.y + k * THROTTLE_TRAVEL / 4
    box("panel_grey", "panel_grey", (q.x + 0.035, y, q.z + 0.011), (0.03, 0.003, 0.002))
lever = empty("throttle", THROTTLE, props={"throttle": True})
box("metal", "metal", (0.0, 0.0, -0.035), (0.014, 0.014, 0.06), parent=lever)
box("panel_grey", "panel_grey", (0.0, 0.0, 0.01), (0.07, 0.04, 0.035), parent=lever)
box("red", "red", (0.03, 0.0, 0.03), (0.012, 0.012, 0.006), parent=lever)
empty("throttle_full", THROTTLE + Vector((0.0, THROTTLE_TRAVEL, 0.0)),
      props={"anchor": "throttle_full"})

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

# --- the drone's camera across the panel
screen(panel, "screen_feed", 0.0, 0.02, FEED, behind="trackcam_wide")

# --- the drone list in its own housing, turned towards the operator
info = Surface(INFO_AT, Euler((0.0, 0.0, math.radians(INFO_TURN))))
box("dark", "dark", info.at(0.0, 0.0, -0.03), (INFO[0] + 0.07, 0.06, INFO[1] + 0.07), info.rot)
screen(info, "screen_info", 0.0, 0.0, INFO)

# --- the desk: the ground control station map across it, the picture in picture behind its
# corner (the map's canvas is clear there)
screen(desk, "screen_map", 0.0, 0.0, MAP)
# The flying buttons left of it, the camera's right of it.
column(desk, -BUTTONS_U, [
    ("Console(Leave)", "LEAVE", 0.14),
    ("Console(Auto)", "AUTO", 0.05),
    ("Console(Manual)", "MAN", 0.0),
    ("Console(View)", "FWD", -0.08),
])
column(desk, BUTTONS_U, [
    ("Console(Flir)", "FLIR", 0.14),
    ("Console(ZoomIn)", "ZOOM+", 0.06),
    ("Console(ZoomOut)", "ZOOM-", 0.01),
    ("Console(Track)", "TRACK", -0.07),
    ("Console(Slave)", "SLAVE", -0.12),
])
mesh_object("trackcam_pip", quad_mesh("trackcam_pip", PIP[0], PIP[1], "trackcam_pip"),
            desk.at(PIP_AT[0], PIP_AT[1], 0.009), desk.rot, props={"no_shadow": True})

bpy.ops.wm.save_as_mainfile(filepath=out)
