# Builds the MH-60R Seahawk's sensor operator's console master (assets-src/blend/cockpit_asw.blend):
# the anti-submarine warfare station in the helicopter's cabin, from nothing:
#   blender -b --factory-startup --python-exit-code 1 --python tools/models/build_console_asw.py -- \
#       assets-src/blend/cockpit_asw.blend
# then tools/models/export.sh cockpit_asw. See src/cockpit3d/model.rs for what the game binds to.
#
# Built: the cabin round the seat (quilted soundproofing on the walls, roof and bulkheads, frames,
# open window holes in the left wall and the sliding door's window, the floor with its tie-downs,
# the sonobuoy launcher's 25 tubes along the aft left wall, the dipping sonar's reeling machine
# behind the seat, stowage, a dim cabin lamp), the forward facing crew seat, the console bolted to
# the cockpit's rear bulkhead: a frame with grey avionics boxes, the big tactical display
# (`screen_asw`) with its page soft keys under it, the tall keyset right of it (sonobuoys, sonar,
# MAD / nav, weapons, RELEASE in a striped guard), the status screen (`screen_info`) over the
# keyset, a keyboard shelf, and right of the seat the hand controller (`stick`) that slews the
# map cursor.
#
# Blender frame: eye at the origin, x right, y forward, z up, metres. A control's face points
# along its local -Y (towards the operator), local Z is up its panel.
import math
import os
import sys

import bmesh
import bpy
from mathutils import Euler, Matrix, Vector

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

import console_lib as lib  # noqa: E402
from console_lib import Surface, box, column, screen  # noqa: E402

out = sys.argv[sys.argv.index("--") + 1]

# The cabin: floor, left and right walls, the roof (ROOF_EDGE at the walls, ROOF_TOP over the
# centreline CL), the cockpit's rear bulkhead (FORE) and the aft end (AFT).
FLOOR = -1.00
LEFT, RIGHT = -0.70, 1.60
ROOF_EDGE, ROOF_TOP, CL = 0.50, 0.95, 0.45
FORE, AFT = 1.30, -2.90
PAD = 0.03  # quilted soundproofing's thickness
# Holes in the left wall, (y0, y1, z0, z1): two windows by the seat, the sliding door's window.
WINDOWS = ((0.20, 0.47, -0.53, 0.13), (0.56, 0.80, -0.53, 0.13), (-0.97, -0.37, -0.53, 0.13))
DOOR = (-1.15, -0.17)
# The frames (ribs) across the cabin; the left wall keeps only those clear of its holes.
RIBS = (1.05, 0.515, 0.0, -1.30, -1.95, -2.55)
# The tactical display (4:3), its centre and its lean back; the status screen (4:3).
ASW = (0.42, 0.315)
ASW_AT = (-0.05, 0.72, -0.05)
ASW_LEAN = -10.0
INFO = (0.20, 0.15)
# The keyset: its face's centre, turn towards the eye and lean back; its face's size.
KEY_AT = (0.32, 0.58, -0.08)
KEY_TURN, KEY_LEAN = -28.0, -8.0
KEY_SIZE = (0.24, 0.56)
# The hand controller's pivot, on its pedestal right of the seat.
STICK = Vector((0.37, 0.20, -0.42))

lib.setup(("screen_asw", "screen_info"))
lib.material("yellow", (0.85, 0.65, 0.04))
lib.material("insulation", (0.27, 0.28, 0.25))
lib.material("cabin", (0.17, 0.19, 0.18))
lib.material("floor", (0.06, 0.06, 0.06))
lib.material("olive", (0.17, 0.19, 0.10))
lib.material("white", (0.75, 0.75, 0.72))
lib.material("lamp_lens", (0.6, 0.65, 0.7))


def roof(x):
    """The roof's height over x."""
    t = (x - CL) / (RIGHT - CL if x > CL else CL - LEFT)
    return ROOF_EDGE + (ROOF_TOP - ROOF_EDGE) * math.sqrt(max(0.0, 1.0 - t * t))


def merged(name, mat, parts):
    """One mesh of many parts: ("box", centre, size, rot) or ("cyl", centre, radius, length,
    rot), a cylinder along its local Y."""
    me = bpy.data.meshes.new(name)
    bm = bmesh.new()
    for kind, centre, a, b, rot in parts:
        m = Matrix.Translation(centre) @ rot.to_matrix().to_4x4()
        if kind == "box":
            bmesh.ops.create_cube(bm, size=1.0, matrix=m @ Matrix.Diagonal((*a, 1.0)))
        else:
            m = m @ Matrix.Rotation(math.pi / 2, 4, "X")
            bmesh.ops.create_cone(bm, cap_ends=True, segments=16, radius1=a, radius2=a, depth=b,
                                  matrix=m)
    uv = bm.loops.layers.uv.new()
    for f in bm.faces:
        for i, l in enumerate(f.loops):
            l[uv].uv = ((0, 0), (1, 0), (1, 1), (0, 1))[i % 4]
    bm.to_mesh(me)
    bm.free()
    me.materials.append(lib.mats[mat])
    return lib.mesh_object(name, me, (0.0, 0.0, 0.0))


def B(centre, size, rot=Euler((0, 0, 0))):
    return ("box", Vector(centre), size, None, rot)


def C(centre, r, length, rot=Euler((0, 0, 0))):
    return ("cyl", Vector(centre), r, length, rot)


def cyl(name, mat, centre, r, length, rot=Euler((0, 0, 0))):
    return merged(name, mat, [C(centre, r, length, rot)])


def tiles(rect, size=(0.24, 0.22), gap=0.012):
    """Pads tiling a rectangle (a0, a1, b0, b1): (centre a, centre b, size a, size b)."""
    a0, a1, b0, b1 = rect
    if a1 - a0 < 0.05 or b1 - b0 < 0.05:
        return []
    na = max(1, round((a1 - a0) / size[0]))
    nb = max(1, round((b1 - b0) / size[1]))
    da, db = (a1 - a0) / na, (b1 - b0) / nb
    return [(a0 + (i + 0.5) * da, b0 + (j + 0.5) * db, da - gap, db - gap)
            for i in range(na) for j in range(nb)]


# --- the cabin
def walls():
    pads = []
    # The left wall: the skin behind (with the holes), quilting round the holes and the door.
    skin = []
    edges = sorted({AFT, FORE, *[w[0] for w in WINDOWS], *[w[1] for w in WINDOWS]})
    for y0, y1 in zip(edges, edges[1:]):
        hole = next((w for w in WINDOWS if w[0] == y0 and w[1] == y1), None)
        spans = [(FLOOR, hole[2]), (hole[3], ROOF_EDGE)] if hole else [(FLOOR, ROOF_EDGE)]
        for z0, z1 in spans:
            skin.append(B((LEFT - 0.015, (y0 + y1) / 2, (z0 + z1) / 2), (0.03, y1 - y0, z1 - z0)))
    merged("cabin", "cabin", skin)
    m = 0.045
    regions = [
        (AFT, DOOR[0], FLOOR, ROOF_EDGE),
        (DOOR[1], FORE, FLOOR, WINDOWS[0][2] - m),
        (DOOR[1], FORE, WINDOWS[0][3] + m, ROOF_EDGE),
        (DOOR[1], WINDOWS[0][0] - m, WINDOWS[0][2] - m, WINDOWS[0][3] + m),
        (WINDOWS[1][1] + m, FORE, WINDOWS[0][2] - m, WINDOWS[0][3] + m),
    ]
    for r in regions:
        for a, b, sa, sb in tiles(r):
            pads.append(B((LEFT + PAD / 2, a, b), (PAD, sa, sb)))
    # The right wall, quilted all over.
    for a, b, sa, sb in tiles((AFT, FORE, FLOOR, ROOF_EDGE)):
        pads.append(B((RIGHT - PAD / 2, a, b), (PAD, sa, sb)))
    # The roof: panels along its curve, quilted.
    n = 12
    xs = [LEFT + (RIGHT - LEFT) * k / n for k in range(n + 1)]
    for x0, x1 in zip(xs, xs[1:]):
        z0, z1 = roof(x0), roof(x1)
        ang = math.atan2(z1 - z0, x1 - x0)
        chord = math.hypot(x1 - x0, z1 - z0)
        rot = Euler((0.0, -ang, 0.0))
        nrm = Vector((-math.sin(ang), 0.0, math.cos(ang)))
        mid = Vector(((x0 + x1) / 2, 0.0, (z0 + z1) / 2))
        for a, _, sa, _ in tiles((AFT, FORE, 0, 1), (0.26, 1)):
            c = mid - nrm * (PAD / 2) + Vector((0, a, 0))
            pads.append(B(c, (chord - 0.012, sa, PAD), rot))
        merged("cabin", "cabin", [B(mid + nrm * 0.015, (chord + 0.01, FORE - AFT, 0.03), rot)])
    # The rear bulkhead (the cockpit's back) round a curtained pass-through, the aft bulkhead.
    pass_x, pass_z = (0.78, 1.32), (-0.12, 0.42)
    for y, d in ((FORE, -1), (AFT, 1)):
        regions = [(LEFT, RIGHT, FLOOR, 1.0)]
        if y == FORE:
            regions = [(LEFT, pass_x[0] - m, FLOOR, 1.0), (pass_x[1] + m, RIGHT, FLOOR, 1.0),
                       (pass_x[0] - m, pass_x[1] + m, FLOOR, pass_z[0] - m),
                       (pass_x[0] - m, pass_x[1] + m, pass_z[1] + m, 1.0)]
        for r in regions:
            for a, b, sa, sb in tiles(r):
                top = min(roof(a - sa / 2), roof(a + sa / 2)) - 0.01
                lo, hi = b - sb / 2, min(b + sb / 2, top)
                if hi - lo > 0.05:
                    pads.append(B((a, y + d * PAD / 2, (lo + hi) / 2), (sa, PAD, hi - lo)))
        merged("cabin", "cabin", [B(((LEFT + RIGHT) / 2, y - d * 0.015, (FLOOR + ROOF_TOP) / 2),
                                    (RIGHT - LEFT, 0.03, ROOF_TOP - FLOOR))])
    merged("insulation", "insulation", pads)
    # The pass-through: a frame round a dark curtain hanging in folds.
    frame = []
    w, h = pass_x[1] - pass_x[0], pass_z[1] - pass_z[0]
    cx, cz = (pass_x[0] + pass_x[1]) / 2, (pass_z[0] + pass_z[1]) / 2
    for c, s in (((cx, pass_z[1] + 0.02), (w + 0.08, 0.04)), ((cx, pass_z[0] - 0.02), (w + 0.08, 0.04)),
                 ((pass_x[0] - 0.02, cz), (0.04, h)), ((pass_x[1] + 0.02, cz), (0.04, h))):
        frame.append(B((c[0], FORE - 0.03, c[1]), (s[0], 0.05, s[1])))
    merged("frame", "frame", frame)
    folds = []
    k = 8
    for i in range(k):
        x = pass_x[0] + (i + 0.5) * w / k
        folds.append(B((x, FORE - 0.02 - 0.012 * (i % 2), cz), (w / k + 0.004, 0.012, h)))
    merged("black", "black", folds)


def door_and_windows():
    """The sliding door's panel round its window, its rails; frames round the holes."""
    y0, y1 = DOOR
    win = WINDOWS[2]
    parts = [
        B((LEFT + 0.015, (y0 + y1) / 2, (FLOOR + win[2]) / 2), (0.03, y1 - y0, win[2] - FLOOR)),
        B((LEFT + 0.015, (y0 + y1) / 2, (win[3] + ROOF_EDGE - 0.08) / 2), (0.03, y1 - y0, ROOF_EDGE - 0.08 - win[3])),
        B((LEFT + 0.015, (y0 + win[0]) / 2, (win[2] + win[3]) / 2), (0.03, win[0] - y0, win[3] - win[2])),
        B((LEFT + 0.015, (win[1] + y1) / 2, (win[2] + win[3]) / 2), (0.03, y1 - win[1], win[3] - win[2])),
    ]
    merged("cabin", "cabin", parts)
    # The door's edges, its handle, the rails it slides aft on.
    trim = [B((LEFT + 0.035, y, (FLOOR + ROOF_EDGE - 0.08) / 2), (0.02, 0.03, ROOF_EDGE - 0.08 - FLOOR))
            for y in (y0 + 0.015, y1 - 0.015)]
    trim.append(B((LEFT + 0.035, (y0 + y1) / 2, ROOF_EDGE - 0.095), (0.02, y1 - y0, 0.03)))
    trim.append(B((LEFT + 0.035, (y0 + y1) / 2, FLOOR + 0.015), (0.02, y1 - y0, 0.03)))
    trim.append(B((LEFT + 0.03, (AFT + 0.2 + y1) / 2, ROOF_EDGE - 0.04), (0.05, y1 - AFT - 0.2, 0.04)))
    merged("dark", "dark", trim)
    merged("metal", "metal", [B((LEFT + 0.06, y1 - 0.09, -0.62), (0.03, 0.03, 0.16)),
                              B((LEFT + 0.045, y1 - 0.09, -0.55), (0.03, 0.03, 0.02)),
                              B((LEFT + 0.045, y1 - 0.09, -0.69), (0.03, 0.03, 0.02))])
    # Rubber frames round every hole, standing out of the wall.
    rub = []
    t, d = 0.03, 0.045
    for (a0, a1, b0, b1) in WINDOWS:
        x = LEFT + d / 2
        rub += [B((x, (a0 + a1) / 2, b1 + t / 2), (d, a1 - a0 + 2 * t, t)),
                B((x, (a0 + a1) / 2, b0 - t / 2), (d, a1 - a0 + 2 * t, t)),
                B((x, a0 - t / 2, (b0 + b1) / 2), (d, t, b1 - b0)),
                B((x, a1 + t / 2, (b0 + b1) / 2), (d, t, b1 - b0))]
    merged("rubber", "rubber", rub)


def ribs():
    """The frames: up the walls and over the roof; the left wall's only where clear of the
    holes and the door. Stringers along the walls."""
    parts = []
    for y in RIBS:
        clear = not any(w[0] - 0.06 < y < w[1] + 0.06 for w in WINDOWS) and not DOOR[0] - 0.03 < y < DOOR[1] + 0.03
        if clear:
            parts.append(B((LEFT + 0.04, y, (FLOOR + ROOF_EDGE) / 2), (0.08, 0.05, ROOF_EDGE - FLOOR)))
        parts.append(B((RIGHT - 0.04, y, (FLOOR + ROOF_EDGE) / 2), (0.08, 0.05, ROOF_EDGE - FLOOR)))
        n = 12
        xs = [LEFT + (RIGHT - LEFT) * k / n for k in range(n + 1)]
        for x0, x1 in zip(xs, xs[1:]):
            z0, z1 = roof(x0), roof(x1)
            ang = math.atan2(z1 - z0, x1 - x0)
            nrm = Vector((-math.sin(ang), 0.0, math.cos(ang)))
            c = Vector(((x0 + x1) / 2, y, (z0 + z1) / 2)) - nrm * 0.04
            parts.append(B(c, (math.hypot(x1 - x0, z1 - z0) + 0.02, 0.05, 0.08), Euler((0, -ang, 0))))
    # Stringers along the right wall and the left wall's ends.
    for z in (-0.6, -0.1, 0.35):
        parts.append(B((RIGHT - 0.035, (AFT + FORE) / 2, z), (0.04, FORE - AFT, 0.035)))
    parts.append(B((LEFT + 0.035, (AFT + DOOR[0]) / 2, -0.25), (0.04, DOOR[0] - AFT, 0.035)))
    parts.append(B((LEFT + 0.035, (AFT + FORE) / 2, 0.30), (0.04, FORE - AFT, 0.035)))
    merged("frame", "frame", parts)
    # Grab handles and a cable run under the roof.
    handles = []
    for y in (-2.2, -1.6, -0.5):
        z = roof(0.95) - 0.10
        handles += [B((0.95, y, z), (0.03, 0.30, 0.025)), B((0.95, y - 0.14, z + 0.04), (0.025, 0.025, 0.08)),
                    B((0.95, y + 0.14, z + 0.04), (0.025, 0.025, 0.08))]
    merged("metal", "metal", handles)
    merged("dark", "dark", [B((-0.05, (AFT + FORE) / 2, roof(-0.05) - 0.06), (0.10, FORE - AFT, 0.05))])


def floor():
    box("floor", "floor", ((LEFT + RIGHT) / 2, (AFT + FORE) / 2, FLOOR - 0.03), (RIGHT - LEFT, FORE - AFT, 0.06))
    rings, strips = [], []
    for x in (-0.45, 0.45, 1.35):
        for y in (-2.5, -1.8, -0.2, 0.6):
            rings.append(B((x, y, FLOOR + 0.005), (0.07, 0.07, 0.01)))
    for x in (-0.3, 0.0, 0.9, 1.2):
        strips.append(B((x, (AFT + FORE) / 2, FLOOR + 0.002), (0.12, FORE - AFT - 0.1, 0.004)))
    merged("metal", "metal", rings)
    merged("dark", "dark", strips)


def sonobuoys():
    """The launcher along the aft left wall: a grey frame with 5 x 5 tubes, their caps facing
    the cabin."""
    x0, x1 = LEFT + 0.05, LEFT + 0.40
    y0, y1 = -2.62, -1.48
    z0, z1 = FLOOR, -0.08
    box("panel_grey", "panel_grey", ((x0 + x1) / 2 - 0.01, (y0 + y1) / 2, (z0 + z1) / 2), (x1 - x0 - 0.02, y1 - y0, z1 - z0))
    tubes, caps, bands = [], [], []
    along = Euler((0, 0, math.pi / 2))  # cylinder axis along x
    for i in range(5):
        for j in range(5):
            y = y0 + 0.13 + i * (y1 - y0 - 0.26) / 4
            z = z0 + 0.18 + j * (z1 - z0 - 0.30) / 4
            tubes.append(C((x1 + 0.01, y, z), 0.068, 0.04, along))
            caps.append(C((x1 + 0.032, y, z), 0.056, 0.01, along))
            bands.append(B((x1 + 0.038, y, z + 0.035), (0.004, 0.06, 0.008)))
    merged("metal", "metal", tubes)
    merged("dark", "dark", caps)
    merged("yellow", "yellow", bands)
    merged("frame", "frame", [B((x1 + 0.01, (y0 + y1) / 2, z1 + 0.02), (0.06, y1 - y0 + 0.04, 0.04)),
                              B((x1 + 0.01, y0 - 0.01, (z0 + z1) / 2), (0.06, 0.04, z1 - z0)),
                              B((x1 + 0.01, y1 + 0.01, (z0 + z1) / 2), (0.06, 0.04, z1 - z0))])
    # A spare buoy in its stowage strap on top.
    cyl("olive", "olive", ((x0 + x1) / 2, (y0 + y1) / 2, z1 + 0.10), 0.062, 0.9)
    merged("black", "black", [B(((x0 + x1) / 2, (y0 + y1) / 2 + d, z1 + 0.10), (0.14, 0.03, 0.14)) for d in (-0.25, 0.25)])


def reeling_machine():
    """The dipping sonar's reeling machine on the floor behind the seat: a grey housing, the
    drum with its cable, side plates, a guard over it; the dome's well in the floor beside it."""
    cx, cy = 0.45, -1.0
    box("panel_grey", "panel_grey", (cx, cy, -0.80), (0.80, 0.70, 0.40))
    box("dark", "dark", (cx, cy, -0.995), (0.88, 0.78, 0.02))
    along = Euler((0, 0, math.pi / 2))
    cyl("black", "black", (cx, cy, -0.36), 0.25, 0.56, along)
    merged("metal", "metal", [C((cx + s * 0.30, cy, -0.36), 0.31, 0.04, along) for s in (-1, 1)])
    merged("panel_grey", "panel_grey",
           [B((cx + s * 0.30, cy, -0.52), (0.04, 0.50, 0.30)) for s in (-1, 1)] +
           [C((cx + 0.39, cy, -0.36), 0.12, 0.16, along)])
    # Guard rails over the drum, a placard and the cable's fairlead.
    rails = [B((cx, cy + d, -0.06), (0.66, 0.025, 0.025)) for d in (-0.22, 0.0, 0.22)]
    rails += [B((cx + s * 0.32, cy + d, -0.21), (0.025, 0.025, 0.31)) for s in (-1, 1) for d in (-0.22, 0.22)]
    merged("yellow", "yellow", rails)
    box("black", "black", (cx, cy + 0.351, -0.75), (0.30, 0.004, 0.12))
    mesh_label("SONAR", (cx, cy + 0.354, -0.75), 0.12)
    box("metal", "metal", (cx - 0.20, cy + 0.40, -0.92), (0.14, 0.12, 0.14))


def mesh_label(text, at, w, rot=Euler((0, 0, 0)), parent=None):
    """An engraved placard (`label_TEXT`), facing -Y."""
    name = "label_" + text
    lib.material(name, (0.1, 0.4, 0.1))
    return lib.mesh_object(name, lib.quad_mesh(name, w, w * 11 / 28, name), at, rot, parent=parent,
                           props={"no_shadow": True})


def stowage():
    """Bags and a box strapped to the right wall, a fire extinguisher and a first aid kit, the
    folded troop seat."""
    merged("olive", "olive", [B((RIGHT - 0.20, -2.3, FLOOR + 0.17), (0.34, 0.60, 0.34)),
                              B((RIGHT - 0.18, -1.75, FLOOR + 0.13), (0.30, 0.40, 0.26)),
                              B((RIGHT - 0.22, -2.3, FLOOR + 0.42), (0.28, 0.46, 0.16))])
    merged("black", "black", [B((RIGHT - 0.20, y, FLOOR + 0.20), (0.36, 0.03, 0.42)) for y in (-2.45, -2.15, -1.75)])
    cyl("red", "red", (1.05, FORE - 0.10, -0.55), 0.06, 0.40, Euler((math.pi / 2, 0, 0)))
    box("black", "black", (1.05, FORE - 0.10, -0.33), (0.05, 0.05, 0.05))
    box("metal", "metal", (1.05, FORE - 0.045, -0.55), (0.10, 0.02, 0.05))
    box("white", "white", (-0.15, AFT + 0.10, -0.05), (0.32, 0.12, 0.22))
    box("red", "red", (-0.15, AFT + 0.161, -0.05), (0.10, 0.004, 0.03))
    box("red", "red", (-0.15, AFT + 0.161, -0.05), (0.03, 0.004, 0.10))
    # The troop seat folded up against the right wall, its webbing.
    box("frame", "frame", (RIGHT - 0.08, -0.55, -0.45), (0.06, 0.90, 0.04))
    box("olive", "olive", (RIGHT - 0.06, -0.55, -0.70), (0.04, 0.86, 0.48))
    merged("black", "black", [B((RIGHT - 0.035, y, -0.70), (0.01, 0.05, 0.50)) for y in (-0.85, -0.55, -0.25)])
    # A cargo net on the aft bulkhead.
    net = [B((x, AFT + 0.06, -0.25), (0.02, 0.01, 1.4)) for x in (0.6, 0.8, 1.0, 1.2, 1.4)]
    net += [B((1.0, AFT + 0.06, z), (0.82, 0.01, 0.02)) for z in (-0.9, -0.6, -0.3, 0.0, 0.3)]
    merged("olive", "olive", net)


def lamp():
    """The dim cabin lamp under the roof, between the seat and the reeling machine."""
    x, y = 0.25, -0.30
    z = roof(x) - 0.04
    lib.empty("lamp", (x, y, z - 0.08), props={"lamp": [0.32, 0.34, 0.38], "range": 3.5})
    box("metal", "metal", (x, y, z), (0.22, 0.12, 0.03))
    box("lamp_lens", "lamp_lens", (x, y, z - 0.02), (0.18, 0.08, 0.01))


def seat():
    """The crew seat: the shared chair, a tube frame on floor tracks, a headrest, a harness."""
    lib.chair()
    merged("metal", "metal", [B((s * 0.22, -0.05, -0.93), (0.03, 0.50, 0.03)) for s in (-1, 1)] +
           [B((s * 0.22, -0.30, -0.88), (0.03, 0.03, 0.14)) for s in (-1, 1)])
    merged("dark", "dark", [B((s * 0.22, -0.05, FLOOR + 0.01), (0.06, 0.9, 0.02)) for s in (-1, 1)])
    box("cushion", "cushion", (0.0, -0.32, 0.05), (0.28, 0.08, 0.18))
    box("seat", "seat", (0.0, -0.35, -0.04), (0.06, 0.04, 0.10))
    merged("black", "black", [B((s * 0.09, -0.257, -0.36), (0.05, 0.006, 0.56)) for s in (-1, 1)] +
           [B((0.0, -0.257, -0.52), (0.30, 0.006, 0.05))])
    box("metal", "metal", (0.0, -0.25, -0.52), (0.07, 0.01, 0.07))


walls()
door_and_windows()
ribs()
floor()
sonobuoys()
reeling_machine()
stowage()
lamp()
seat()
lib.hand_controller(STICK, FLOOR)

# --- the console's frame, bolted to the bulkhead: two uprights, crossbars, the avionics boxes
FX = (-0.44, 0.53)
FY = 0.98
frame = []
for x in FX:
    frame.append(B((x, FY, (FLOOR + 0.42) / 2), (0.04, 0.04, 0.42 - FLOOR)))
    for z in (FLOOR + 0.06, 0.40):
        frame.append(B((x, (FY + FORE) / 2, z), (0.04, FORE - FY, 0.04)))
for z in (FLOOR + 0.06, -0.44, 0.40):
    frame.append(B(((FX[0] + FX[1]) / 2, FY, z), (FX[1] - FX[0], 0.04, 0.04)))
merged("frame", "frame", frame)
bolts = [B((x + dx, FORE - 0.035, z + dz), (0.012, 0.01, 0.012))
         for x in FX for z in (FLOOR + 0.06, 0.40) for dx in (-0.035, 0.035) for dz in (-0.02, 0.02)]
merged("metal", "metal", bolts)

# The avionics boxes under the screen: grey cases, black front panels, handles, connectors.
grey, black, metal = [], [], []
for (x, w, z, h) in ((-0.20, 0.44, -0.75, 0.42), (0.27, 0.46, -0.75, 0.42), (0.03, 0.92, -0.46, 0.12)):
    grey.append(B((x, (0.80 + FORE) / 2, z), (w - 0.01, FORE - 0.80, h - 0.01)))
    black.append(B((x, 0.795, z), (w - 0.04, 0.01, h - 0.04)))
    for s in (-1, 1):
        metal.append(B((x + s * (w / 2 - 0.035), 0.77, z), (0.02, 0.02, min(0.16, h - 0.06))))
        metal.append(B((x + s * (w / 2 - 0.035), 0.78, z + min(0.08, h / 2 - 0.03)), (0.02, 0.02, 0.02)))
        metal.append(B((x + s * (w / 2 - 0.035), 0.78, z - min(0.08, h / 2 - 0.03)), (0.02, 0.02, 0.02)))
    for k in range(3 if h > 0.2 else 0):
        black.append(C((x - 0.08 + k * 0.08, 0.785, z - 0.10), 0.022, 0.03, Euler((0, 0, 0))))
        metal.append(C((x - 0.08 + k * 0.08, 0.78, z - 0.10), 0.016, 0.025, Euler((0, 0, 0))))
merged("panel_grey", "panel_grey", grey)
merged("black", "black", black)
merged("metal", "metal", metal)
for x, text in ((-0.20, "DPU"), (0.27, "ACU")):
    mesh_label(text, (x, 0.789, -0.62), 0.07)

# The keyboard shelf: a tray on arms, a keyboard and a trackball pad on it.
SHELF_Z = -0.40
box("panel_grey", "panel_grey", (-0.13, 0.64, SHELF_Z), (0.64, 0.30, 0.025))
merged("frame", "frame", [B((x, 0.72, SHELF_Z - 0.03), (0.03, 0.16, 0.04)) for x in (-0.30, 0.14)])
box("dark", "dark", (-0.10, 0.62, SHELF_Z + 0.022), (0.40, 0.17, 0.02))
keys = []
for r in range(4):
    for k in range(13):
        keys.append(B((-0.28 + k * 0.03, 0.56 + r * 0.033, SHELF_Z + 0.036), (0.024, 0.026, 0.01)))
merged("black", "black", keys)
box("dark", "dark", (0.15, 0.62, SHELF_Z + 0.02), (0.07, 0.10, 0.016))

# --- the tactical display: a deep unit leaning back, its bezel, the page soft keys under it
asw = Surface(ASW_AT, Euler((math.radians(ASW_LEAN), 0.0, 0.0)))
box("dark", "dark", asw.at(-0.05, -0.03, -0.20), (0.62, 0.40, 0.52), asw.rot)
box("panel_grey", "panel_grey", asw.at(-0.05, -0.03, -0.21), (0.64, 0.36, 0.54), asw.rot)
screen(asw, "screen_asw", 0.0, 0.0, ASW)
column(asw, 0.0, [("Console(AswPage, 1)", "WPN", -0.206)])
column(asw, -0.12, [("Console(AswPage, 0)", "SONAR", -0.206)])
column(asw, 0.12, [("Console(AswPage, 2)", "MAP", -0.206)])
# Blank soft keys at the row's ends and down the sides, as on the real display.
blank = []
for u in (-0.20, 0.20):
    blank.append(B(asw.at(u, -0.206, 0.004), (0.036, 0.008, 0.02), asw.rot))
for v in (0.10, 0.03, -0.04, -0.11):
    blank.append(B(asw.at(0.248, v, 0.004), (0.022, 0.008, 0.036), asw.rot))
merged("dark", "dark", blank)
# Brightness and contrast knobs, status lamps, handles.
knobs = [C(asw.at(u, -0.245, 0.01), 0.013, 0.02, asw.rot) for u in (-0.20, 0.20)]
merged("black", "black", knobs)
merged("white", "white", [B(asw.at(u, -0.235, 0.021), (0.002, 0.002, 0.008), asw.rot) for u in (-0.20, 0.20)])
merged("yellow", "yellow", [B(asw.at(-0.19 + k * 0.03, 0.215, 0.002), (0.012, 0.004, 0.008), asw.rot) for k in range(3)])
merged("metal", "metal", [B(asw.at(s * 0.29 - 0.05, 0.12, 0.03), (0.02, 0.02, 0.14), asw.rot) for s in (-1, 1)] +
       [B(asw.at(s * 0.29 - 0.05, 0.12 + d, 0.015), (0.02, 0.03, 0.02), asw.rot) for s in (-1, 1) for d in (-0.06, 0.06)])
# LEAVE on its own sloping block at the shelf's left front corner, within reach.
leave = Surface((-0.37, 0.54, SHELF_Z + 0.05), Euler((math.radians(-55.0), 0.0, math.radians(25.0))))
box("dark", "dark", leave.at(0.0, 0.0, -0.03), (0.10, 0.06, 0.08), leave.rot)
box("black", "black", leave.at(0.0, 0.0, 0.0), (0.09, 0.004, 0.07), leave.rot)
column(leave, 0.0, [("Console(Leave)", "LEAVE", 0.0)])

# --- the keyset: a tall black panel on a grey case, two columns of keys by group, rockers and
# knobs down its edges, RELEASE in a yellow and black striped guard
keyset = Surface(KEY_AT, Euler((math.radians(KEY_LEAN), 0.0, math.radians(KEY_TURN))))
kw, kh = KEY_SIZE
box("panel_grey", "panel_grey", keyset.at(0.0, 0.0, -0.16), (kw + 0.02, 0.30, kh + 0.04), keyset.rot)
box("black", "black", keyset.at(0.0, 0.0, -0.004), (kw, 0.008, kh), keyset.rot)
L, R = -0.045, 0.045
for text, v in (("BUOY", 0.245), ("SONAR", 0.115), ("MAD/NAV", -0.055), ("WEAPONS", -0.180)):
    mesh_label(text, keyset.at(0.0, v, 0.001), 0.05, keyset.rot)
column(keyset, L, [
    ("Console(Buoy, 1)", "DIFAR", 0.205),
    ("Console(Buoy, 3)", "BT", 0.160),
    ("Console(Ping)", "PING", 0.075),
    ("Console(Depth, 1)", "DEPTH-", 0.030),
    ("Console(Audio)", "AUDIO", -0.015),
    ("Console(Mad)", "MAD", -0.095),
    ("Console(RangeOut)", "ZOOM-", -0.140),
    ("Console(Arm)", "ARM", -0.225),
])
column(keyset, R, [
    ("Console(Buoy, 2)", "DICASS", 0.205),
    ("Console(Dip)", "DIP", 0.075),
    ("Console(Depth, 2)", "DEPTH+", 0.030),
    ("Console(Mark)", "MARK", -0.095),
    ("Console(RangeIn)", "ZOOM+", -0.140),
    ("Console(Release)", "RELEASE", -0.225),
])
# Group rules between the key groups.
merged("white", "white", [B(keyset.at(0.0, v, 0.001), (kw - 0.03, 0.002, 0.002), keyset.rot)
                          for v in (0.138, -0.035, -0.160)])
# The guard round RELEASE: a raised ring of alternating yellow and black blocks.
yel, blk = [], []
hw, hh, g, t = 0.024 + 0.010, 0.011 + 0.010, 0.008, 0.012
ring = []
n = 6
for i in range(n):
    u = -hw - g + (i + 0.5) * (2 * (hw + g)) / n
    ring.append(((u, hh + g / 2), ((2 * (hw + g)) / n, g)))
    ring.append(((u, -hh - g / 2), ((2 * (hw + g)) / n, g)))
for i in range(2):
    v = -hh + (i + 0.5) * hh
    ring.append(((-hw - g / 2, v), (g, hh)))
    ring.append(((hw + g / 2, v), (g, hh)))
for k, ((u, v), (su, sv)) in enumerate(ring):
    part = B(keyset.at(R + u, -0.225 + v, 0.005), (su, 0.010, sv), keyset.rot)
    (yel if (k // 2 + k) % 2 == 0 else blk).append(part)
merged("yellow", "yellow", yel)
merged("black", "black", blk)
# Rocker switches down the left edge, knobs down the right, between the keys' rows.
rock_base, rock, knob, idx = [], [], [], []
for k, v in enumerate((0.205, 0.160, 0.075, 0.030, -0.015, -0.095, -0.140)):
    rock_base.append(B(keyset.at(-0.097, v, 0.003), (0.018, 0.006, 0.030), keyset.rot))
    tilt = Euler((math.radians(12.0 if k % 2 else -12.0), 0, 0))
    rot = (keyset.rot.to_matrix() @ tilt.to_matrix()).to_euler()
    rock.append(B(keyset.at(-0.097, v, 0.010), (0.012, 0.008, 0.024), rot))
for v in (0.205, 0.120, 0.030, -0.055, -0.140):
    knob.append(C(keyset.at(0.097, v, 0.010), 0.012, 0.02, keyset.rot))
    idx.append(B(keyset.at(0.097, v + 0.006, 0.0205), (0.002, 0.001, 0.008), keyset.rot))
merged("dark", "dark", rock_base)
merged("panel_grey", "panel_grey", rock)
merged("black", "black", knob)
merged("white", "white", idx)
for text, v in (("VOL", 0.245), ("BRT", -0.180)):
    mesh_label(text, keyset.at(0.097, v, 0.001), 0.03, keyset.rot)

# --- the status screen over the keyset, in its own case, turned and tilted towards the eye
top = keyset.at(0.0, kh / 2, 0.0)
info = Surface(top + Vector((0.0, 0.0, 0.11)), Euler((math.radians(14.0), 0.0, math.radians(KEY_TURN))))
box("dark", "dark", info.at(0.0, 0.0, -0.06), (INFO[0] + 0.07, 0.12, INFO[1] + 0.07), info.rot)
screen(info, "screen_info", 0.0, 0.0, INFO)
merged("frame", "frame", [B(keyset.at(s * 0.08, kh / 2 + 0.02, -0.12), (0.03, 0.05, 0.06), keyset.rot) for s in (-1, 1)])

bpy.ops.wm.save_as_mainfile(filepath=out)
