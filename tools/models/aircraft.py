"""Builds the aircraft of aircraft.blend from their builders and saves it:

    blender -b assets-src/blend/aircraft.blend --python-exit-code 1 --python tools/models/aircraft.py
    blender -b assets-src/blend/aircraft.blend --python-exit-code 1 --python tools/models/anticollision.py
    tools/models/export.sh aircraft

Each aircraft builder returns a `Craft` built from three-view proportions in metres (nose along +y,
z up): the airframe, optional afterburner flames, the anchors for the navigation light dots (147
red / left, 151 green / right). The stores and the wreck are built the same way. Every model is
drawn at its real size (`K` model units per metre) with one detail level: the airframe, its hot
nozzle exits in `_exhaust` and the afterburner flames in `_s13v1` (the game flickers both), a
missile's motor flame in `_s14v1`, the landing gear down in `_s5v1` (the F-16's, F/A-18's and
MiG-23's), the navigation lights in `_s12v1`; anticollision.py adds the flashing lights
afterwards. The F-117A (24, f117.py), the UH-60 (25, uh60.py), the AH-1Z (43, ah1z.py), the RAH-66 (46,
comanche.py) and the OH-58D (48, kiowa.py) have generators of their own. The flyable F-16 (23), A-10 (32) and Tornado (45) set the eye, lift,
exhaust and the A-10's muzzle in src/aircraft.rs: change them with their builders; the gear's depth below the origin is in
`gear_depth` there too. The Tornado's outer wings are parts (`_part1` right, `_part2` left)
turning about their pivots with state slots 21 and 22 (the sweep). The flyable AV-8B Harrier
(47) sits nose up on its bicycle gear (`HARRIER_GROUND_PITCH`); its four nozzles are parts
(`_part1`..`_part4`) pitching down about their pivots with state slot 25 (the nozzle angle).
"""

import math
import os
import sys

import bpy
from mathutils import Vector

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.abspath(os.path.join(HERE, "..", ".."))
sys.path.insert(0, HERE)

import heli  # noqa: E402
import mklib  # noqa: E402

from mklib import Geo, Ramp, block, frustum, hexa, loft, ring, tube, wing  # noqa: E402

GREY_L = Ramp(224, 234)
GREY = Ramp(226, 236)
GREY_D = Ramp(229, 239)
GREY_B = Ramp(225, 235)
# The flyable jets' paint, darker: modern shading lights the ramp's sunlit end again.
F16_GREY = Ramp(231, 237)
TORNADO_GREY = Ramp(232, 238)
A10_GREY = Ramp(233, 239)
CANOPY = Ramp(144, 145)
DARK = Ramp(235, 239)
RADOME = Ramp(230, 236)
OLIVE = Ramp(165, 173)
FLAME = 13
BLACK = 239


class Craft:
    def __init__(self, body, red, green, white=(), flames=None, part=None, pivot=False, gear=None, parts=()):
        # `parts`: (geometry about its pivot, pivot, state slot[, axis]) of parts turning about
        # their up axis (the Tornado's outer wings), or axis 1 (pitch: the Harrier's nozzles).
        self.body, self.flames, self.part, self.gear, self.parts = body, flames, part, gear, parts
        # Light dots sit just outboard of the wingtips so the tip itself doesn't hide them.
        out = Vector((0.25, 0, 0))
        self.lights = {147: [Vector(red) - out], 151: [Vector(green) + out]}


def mirror(g):
    return Geo().add(g).add(g.mirrored())


def canopy(y0, y1, z, hw, hh, n=6):
    """Bubble from front windscreen (y0) to aft fairing (y1) sitting at height z."""
    L = y0 - y1
    return loft(
        [(y0, z, 0.02, 0.02), (y0 - 0.25 * L, z + 0.6 * hh, 0.8 * hw, 0.7 * hh), (y0 - 0.55 * L, z + 0.75 * hh, hw, hh),
         (y1, z, 0.5 * hw, 0.2)],
        CANOPY, n=n, power=2.0,
    )


def nozzle(x, y, z, r, length, n=8):
    """Engine nozzle with the hot exit glowing."""
    g = frustum((x, y, z), (x, y - length, z), r * 1.05, r * 0.9, DARK, n, caps=False)
    g.add(frustum((x, y, z), (x, y - length + 0.15, z), r * 0.75, r * 0.75, FLAME, n, caps=True))
    return g


def flame(x, y, z, r, length, n=6):
    """Exhaust glow behind a nozzle (drawn whenever the aircraft is airborne)."""
    return frustum((x, y + 0.3, z), (x, y - length * 0.45, z), r * 0.85, 0.05, FLAME, n, caps=True)


def missile(x, y, z, length=3.0, r=0.08):
    g = frustum((x, y, z), (x, y - length, z), r, r, GREY_L, 4)
    g.add(frustum((x, y + 0.4, z), (x, y, z), 0.01, r, GREY_L, 4))
    return g


def box_intake(x, y0, y1, z, hw, hh, slant=0.6, paint=GREY):
    """Rectangular intake trunk with a raked mouth (top lip ahead by `slant`)."""
    g = hexa([(x - hw, y0 - slant, z - hh), (x + hw, y0 - slant, z - hh), (x + hw, y1, z - hh), (x - hw, y1, z - hh),
              (x - hw, y0, z + hh), (x + hw, y0, z + hh), (x + hw, y1, z + hh), (x - hw, y1, z + hh)], paint)
    # dark mouth just inside the lip
    lip = lambda t: y0 - slant * t + 0.04
    g.face2([(x - hw * 0.85, lip(0.925), z - hh * 0.85), (x + hw * 0.85, lip(0.925), z - hh * 0.85),
             (x + hw * 0.85, lip(0.075), z + hh * 0.85), (x - hw * 0.85, lip(0.075), z + hh * 0.85)], BLACK)
    return g


def nacelle(x, y0, y1, z, r, paint=GREY_L, n=8, inlet=BLACK):
    """Podded engine: lip ring, body, tapered exhaust."""
    L = y0 - y1
    g = loft([(y0, z, r * 0.9, r * 0.9), (y0 - 0.15 * L, z, r, r), (y1 + 0.2 * L, z, r * 0.9, r * 0.9), (y1, z, r * 0.55, r * 0.55)],
             paint, n=n, x=x)
    g.add(frustum((x, y0 + 0.01, z), (x, y0 - 0.05, z), r * 0.75, r * 0.75, inlet, n))
    return g


def pylon(x, y0, y1, z0, z1, w=0.12, paint=GREY):
    return hexa([(x - w, y0, z0), (x + w, y0, z0), (x + w, y1, z0), (x - w, y1, z0),
                 (x - w, y0 - 0.3 * (y0 - y1), z1), (x + w, y0 - 0.3 * (y0 - y1), z1), (x + w, y1, z1), (x - w, y1, z1)], paint)


# ---------------------------------------------------------------------------------------------
# Fighters


def f15():
    g = loft([(9.7, 0.1, 0.0, 0.0), (8.9, 0.1, 0.32, 0.3), (7.6, 0.15, 0.55, 0.55), (6.0, 0.2, 0.65, 0.7), (4.4, 0.25, 0.75, 0.75),
              (2.8, 0.3, 1.1, 0.7), (0.0, 0.3, 1.55, 0.65), (-4.0, 0.25, 1.6, 0.6), (-7.6, 0.2, 1.45, 0.55), (-8.5, 0.15, 1.3, 0.5)],
             GREY, n=8, power=2.6)
    g.add(loft([(9.7, 0.1, 0, 0), (8.9, 0.1, 0.32, 0.3), (8.2, 0.12, 0.42, 0.4)], RADOME, n=8))
    for s in (-1, 1):
        g.add(box_intake(s * 1.75, 4.0, -1.5, -0.05, 0.55, 0.75, slant=0.9))
        g.add(nozzle(s * 0.65, -8.4, 0.1, 0.58, 1.3))
    g.add(canopy(6.8, 3.6, 0.75, 0.42, 0.5))
    g.add(wing((1.5, 1.6, 0.25), 6.6, (6.5, -3.4, 0.15), 1.9, GREY, t=0.05))
    g.add(wing((1.5, -6.3, 0.1), 3.0, (4.3, -8.4, 0.05), 1.3, GREY, t=0.05))
    for s in (-1, 1):
        g.add(wing((s * 1.45, -5.2, 0.75), 3.3, (s * 1.85, -7.4, 3.4), 1.3, GREY, t=0.06, vertical=True, mirror=False))
    fl = Geo().add(flame(0.65, -9.7, 0.1, 0.5, 3.0)).add(flame(-0.65, -9.7, 0.1, 0.5, 3.0))
    return Craft(g, (-6.5, -4.6, 0.15), (6.5, -4.6, 0.15), [(-1.85, -8.6, 3.4), (1.85, -8.6, 3.4)], fl)


# The F-16C from the three-view in "Aviatsiya i Vremya" 3/99 (Block 50/52): stations along y,
# heights above the chine (the plane of the strakes, wing and stabilizers, z 0).
# Fuselage: (y, top, chine z, chine half width, bottom, half width below the chine, the flanks'
# shape above and below the chine: superellipse powers, under 2 hollow, over 2 boxy). Forward
# the chine is the round nose's widest line; from the cockpit aft it is the strakes' edge,
# then the wing root's and the stabilizer booms'.
F16_BODY = [
    (7.30, -0.20, -0.20, 0.00, -0.20, 0.00, 2.0, 2.0),
    (6.96, 0.02, -0.15, 0.10, -0.31, 0.10, 2.0, 2.0),
    (6.33, 0.24, -0.10, 0.28, -0.37, 0.28, 2.0, 2.0),
    (5.71, 0.37, -0.06, 0.44, -0.39, 0.44, 2.0, 2.2),
    (4.96, 0.52, -0.03, 0.55, -0.38, 0.54, 1.9, 2.4),
    (4.46, 0.56, -0.01, 0.62, -0.36, 0.56, 1.7, 2.4),
    (3.84, 0.58, 0.0, 0.72, -0.34, 0.56, 1.5, 2.4),
    (3.20, 0.60, 0.0, 0.79, -0.36, 0.55, 1.4, 2.4),
    (2.60, 0.62, 0.0, 0.88, -0.62, 0.50, 1.3, 2.6),
    (2.00, 0.66, 0.0, 1.01, -0.80, 0.55, 1.3, 2.8),
    (1.40, 0.74, 0.0, 1.07, -0.92, 0.66, 1.3, 3.0),
    (0.80, 0.82, 0.0, 1.07, -0.92, 0.68, 1.4, 3.0),
    (0.10, 0.81, 0.0, 1.07, -0.91, 0.70, 1.4, 3.0),
    (-1.50, 0.79, 0.0, 1.07, -0.86, 0.70, 1.4, 3.0),
    (-2.70, 0.80, 0.0, 1.07, -0.83, 0.70, 1.4, 3.0),
    (-3.40, 0.79, 0.0, 1.05, -0.78, 0.68, 1.5, 2.8),
    (-4.60, 0.77, 0.02, 0.86, -0.72, 0.64, 1.7, 2.6),
    (-5.80, 0.70, 0.03, 0.70, -0.60, 0.58, 1.9, 2.4),
    (-6.25, 0.58, 0.04, 0.58, -0.50, 0.55, 2.0, 2.0),
]
# The canopy's glass and the fairing behind it: (y, sill z, half width, top).
F16_CANOPY = [(5.00, 0.50, 0.05, 0.52), (4.70, 0.53, 0.30, 0.74), (4.30, 0.55, 0.40, 0.98), (3.80, 0.57, 0.44, 1.16),
              (3.20, 0.59, 0.45, 1.25), (2.60, 0.61, 0.44, 1.20), (2.10, 0.63, 0.40, 1.08)]
F16_FAIRING = [(2.10, 0.63, 0.40, 1.08), (1.40, 0.70, 0.36, 0.97), (0.60, 0.76, 0.30, 0.87), (-0.20, 0.76, 0.18, 0.81)]
# The fin's base: (y, top, half width), from the spine to the drag chute housing past the nozzle.
F16_DORSAL = [(-1.20, 0.80, 0.12), (-2.40, 0.89, 0.20), (-3.60, 1.12, 0.23), (-6.40, 1.17, 0.21), (-7.10, 1.12, 0.15)]


def f16_section(y, top, zc, wc, bottom, wl, pu, pl, m=4):
    """A fuselage ring: hollow or full flanks from the spine out to the chine, a ledge in to the
    lower flanks and round to the keel."""
    right = []
    for k in range(m + 1):
        a = math.pi / 2 * (1 - k / m)
        right.append((wc * math.cos(a) ** (2 / pu), y, zc + (top - zc) * math.sin(a) ** (2 / pu)))
    zl = zc - 0.03 if wl < wc else zc
    for k in range(m + 1):
        a = math.pi / 2 * k / m
        right.append((wl * math.cos(a) ** (2 / pl), y, zl - (zl - bottom) * math.sin(a) ** (2 / pl)))
    left = [(-x, y, z) for x, y, z in reversed(right[1:-1])]
    return right + left


def f16_arch(y, sill, hw, top, n=8):
    """An arch over a flat base (the canopy and its fairing)."""
    return [(hw * math.cos(math.pi * k / n), y, sill + (top - sill) * math.sin(math.pi * k / n)) for k in range(n + 1)]


def f16():
    rings = [f16_section(*s) for s in F16_BODY]
    g = tube(rings, F16_GREY, paints=[RADOME, RADOME] + [F16_GREY] * (len(rings) - 3))
    g.add(tube([f16_arch(*s) for s in F16_CANOPY], CANOPY))
    g.add(tube([f16_arch(*s) for s in F16_FAIRING], F16_GREY))
    g.add(tube([f16_section(y, top, top - 0.3, hw, top - 0.42, hw, 3.0, 3.0, m=2) for y, top, hw in F16_DORSAL], F16_GREY))
    # The chin intake: its mouth raked back from the upper lip, its trunk running into the
    # fuselage's underside.
    def mouth(y, zc, hw, hh, rake=0.0):
        return [(x, y + rake * (z - zc) / hh, z) for x, _, z in ring((0, 0, zc), hw, hh, 12, 2.4)]
    g.add(tube([mouth(3.0, -0.64, 0.60, 0.29, 0.1), mouth(2.5, -0.64, 0.62, 0.30), mouth(1.4, -0.62, 0.64, 0.28)],
               F16_GREY))
    g.face2(mouth(2.96, -0.64, 0.53, 0.24, 0.1), BLACK)
    # The strakes' last stretch to the wing's leading edge, the wing (straight trailing edge,
    # tip rails), the stabilizers on their booms (10 degrees anhedral).
    g.add(wing((1.0, 2.0, 0.0), 1.3, (1.36, 0.42, 0.0), 0.05, F16_GREY, t=0.03))
    g.add(wing((1.0, 0.69, 0.0), 4.10, (4.60, -2.19, 0.0), 1.22, F16_GREY, t=0.04))
    g.add(wing((0.6, -3.3, 0.0), 3.5, (1.05, -3.5, 0.0), 3.35, F16_GREY, t=0.06, t_tip=0.04))
    g.add(wing((1.0, -4.70, 0.02), 2.39, (2.83, -6.19, -0.30), 0.90, F16_GREY, t=0.04))
    for s in (-1, 1):
        g.add(block(s * 4.58, s * 4.72, -1.7, -3.6, -0.05, 0.05, F16_GREY))
        g.add(missile(s * 4.66, -0.9, -0.14, 3.0))
    # The fin on its base, the ventral fins canted out.
    g.add(wing((0, -3.87, 1.1), 2.58, (0, -6.77, 3.27), 0.71, F16_GREY, t=0.05, vertical=True, mirror=False))
    g.add(wing((0.5, -2.96, -0.72), 1.31, (0.67, -3.34, -1.33), 0.3, F16_GREY, t=0.05, vertical=True))
    g.add(nozzle(0, -6.15, 0.04, 0.52, 1.05, n=12))
    fl = flame(0, -7.2, 0.04, 0.45, 3.2)
    return Craft(g, (-4.70, -3.3, 0.05), (4.70, -3.3, 0.05), [(0, -7.0, 3.27)], fl, gear=f16_gear())


# z of the ground under the wheels (`F16_LIFT` and `gear_depth` in src/aircraft.rs).
F16_GROUND = -1.75
F18_GROUND = -1.65
MIG23_GROUND = -1.7


def tricycle(ground, nose, main, paint):
    """The gear down: a nose leg leaning forward to its wheel (or twin wheels), braced from
    behind, its door beside it; two main legs, braced, their doors outboard. `nose` is (y, z where
    the leg meets the body, wheel radius, twin), `main` (y, x and z where a leg meets the body, x
    of its wheel, wheel radius)."""
    g = Geo()
    y, z, r, twin = nose
    axle = (0.0, y + 0.25, ground + r)
    for x in (-0.13, 0.13) if twin else (0.0,):
        g.add(mklib.wheel((x, axle[1], axle[2]), r, 0.12 if twin else 0.16, BLACK, n=8))
    g.add(frustum((0.0, y, z), (0.0, axle[1] - 0.02, axle[2] + 0.1), 0.07, 0.06, GREY_L, n=5))
    g.add(frustum((0.0, y - 0.6, z), (0.0, y + 0.12, (z + axle[2]) / 2), 0.035, 0.035, GREY_L, n=4))
    g.face2([(-0.3, y + 0.7, z), (-0.3, y - 0.5, z), (-0.34, y - 0.5, z - 0.38), (-0.34, y + 0.7, z - 0.38)], paint)
    y, ax, az, wx, r = main
    for s in (-1, 1):
        axle = (s * wx, y, ground + r)
        g.add(mklib.wheel(axle, r, 0.22, BLACK, n=8))
        g.add(frustum((s * ax, y + 0.15, az), (s * (wx - 0.13), y, axle[2] + 0.12), 0.08, 0.07, GREY_L, n=5))
        g.add(frustum((s * ax, y + 0.8, az), (s * (wx - 0.1), y + 0.1, axle[2] + 0.45), 0.04, 0.04, GREY_L, n=4))
        d = s * (ax + 0.2)
        g.face2([(d, y + 0.9, az), (d, y - 0.9, az), (d + s * 0.05, y - 0.9, az - 0.6), (d + s * 0.05, y + 0.9, az - 0.6)],
                paint)
    return g


def f16_gear():
    """The gear down: the nose leg under the intake trunk with its door, the main legs from the
    fuselage's underside below the wing roots with their doors outboard."""
    g = Geo()
    keel = -0.92  # the intake trunk's underside
    r, w = 0.23, 0.16
    axle = (0.0, 2.40, F16_GROUND + r)
    g.add(mklib.wheel(axle, r, w, BLACK, n=8))
    # The leg leans forward down to the wheel, braced from behind.
    g.add(frustum((0.0, 2.15, keel), (0.0, 2.38, axle[2] + 0.1), 0.07, 0.06, GREY_L, n=5))
    g.add(frustum((0.0, 1.55, keel), (0.0, 2.25, (keel + axle[2]) / 2), 0.035, 0.035, GREY_L, n=4))
    g.face2([(-0.3, 2.5, keel + 0.02), (-0.3, 1.3, keel + 0.02), (-0.34, 1.3, keel - 0.36), (-0.34, 2.5, keel - 0.36)], F16_GREY)
    r, w = 0.34, 0.22
    for s in (-1, 1):
        axle = (s * 1.18, -1.76, F16_GROUND + r)
        g.add(mklib.wheel(axle, r, w, BLACK, n=8))
        g.add(frustum((s * 0.62, -1.6, -0.74), (s * 1.05, -1.76, axle[2] + 0.12), 0.08, 0.07, GREY_L, n=5))
        g.add(frustum((s * 0.62, -0.9, -0.76), (s * 1.0, -1.66, axle[2] + 0.4), 0.04, 0.04, GREY_L, n=4))
        g.face2([(s * 0.86, -0.9, -0.1), (s * 0.86, -2.7, -0.1), (s * 0.9, -2.7, -0.72), (s * 0.9, -0.9, -0.72)], F16_GREY)
    return g


# The Tornado IDS from its three-view (16.72 m long with the nose probe, 13.91 m span at 25
# degrees of sweep, 8.60 m at 67): stations along y, heights above the fuselage datum (z 0,
# the intakes' middle a little below it, the wings' plane 0.3 m above). Fuselage rings as the
# F-16's (`f16_section`): the slim nose and cockpit, widening over the two RB199s side by side.
TORNADO_BODY = [
    (7.83, -0.22, -0.24, 0.00, -0.26, 0.00, 2.0, 2.0),
    (7.50, -0.04, -0.24, 0.18, -0.44, 0.18, 2.0, 2.0),
    (7.00, 0.12, -0.22, 0.32, -0.56, 0.32, 2.0, 2.0),
    (6.40, 0.26, -0.20, 0.42, -0.66, 0.42, 2.0, 2.0),
    (5.90, 0.40, -0.18, 0.50, -0.72, 0.50, 1.9, 2.2),
    (5.30, 0.50, -0.12, 0.56, -0.76, 0.56, 1.7, 2.4),
    (4.50, 0.52, -0.08, 0.62, -0.78, 0.60, 1.5, 2.6),
    (3.60, 0.53, -0.04, 0.66, -0.80, 0.64, 1.4, 2.8),
    (2.80, 0.55, 0.0, 0.70, -0.80, 0.68, 1.4, 2.8),
    (1.80, 0.62, 0.0, 0.76, -0.80, 0.74, 1.4, 3.0),
    (1.20, 0.80, 0.0, 0.80, -0.80, 0.78, 1.5, 3.0),
    (0.00, 1.02, 0.0, 0.90, -0.80, 0.88, 1.6, 3.0),
    (-1.50, 1.08, 0.0, 1.10, -0.80, 1.06, 1.8, 3.0),
    (-3.00, 1.10, 0.0, 1.16, -0.78, 1.12, 2.0, 3.0),
    (-4.50, 1.06, 0.0, 1.16, -0.74, 1.12, 2.2, 3.0),
    (-6.00, 0.86, 0.0, 1.08, -0.66, 1.04, 2.4, 2.8),
    (-6.90, 0.62, -0.05, 1.00, -0.60, 0.98, 2.5, 2.6),
]
# The tandem canopy's glass (pilot ahead, navigator behind) and its fairing into the spine:
# (y, sill z, half width, top). The windscreen starts low on the nose and rises in a long
# shallow ramp (about 23 degrees) to the pilot's head.
TORNADO_CANOPY = [(6.15, 0.33, 0.06, 0.36), (5.70, 0.44, 0.30, 0.62), (5.20, 0.49, 0.40, 0.86),
                  (4.70, 0.51, 0.45, 1.08), (4.10, 0.52, 0.47, 1.24), (3.40, 0.54, 0.48, 1.30),
                  (2.60, 0.56, 0.48, 1.30), (1.90, 0.60, 0.44, 1.22), (1.40, 0.66, 0.38, 1.12)]
TORNADO_FAIRING = [(1.40, 0.66, 0.38, 1.12), (0.60, 0.85, 0.40, 1.12), (0.00, 0.95, 0.42, 1.10)]
# The fin's base along the spine: (y, top, half width).
TORNADO_DORSAL = [(-1.40, 1.08, 0.16), (-2.60, 1.22, 0.22), (-3.60, 1.36, 0.25), (-7.20, 1.36, 0.22),
                  (-7.60, 1.20, 0.15)]
# The outer wing's pivot in the glove (x, y; the wings' plane z), its sweep in the model
# (fully forward, 25 degrees at the leading edge) and the game's sweep slots: right, left
# (src/aircraft.rs `TORNADO_*`).
TORNADO_PIVOT = (1.55, -0.65)
TORNADO_WING_Z = 0.30
TORNADO_WING_SLOTS = (21, 22)
TORNADO_GROUND = -1.90


def tornado_wing():
    """The right outer wing, swept fully forward, about its pivot (the left one mirrored): its
    root slides into the glove and the fuselage's side as it sweeps back."""
    px, py = TORNADO_PIVOT
    z = TORNADO_WING_Z
    g = wing((1.75, -0.05, z), 2.20, (6.95, -2.45, z), 0.95, TORNADO_GREY, t=0.045, t_tip=0.035, mirror=False)
    # The swivelling pylon under it, the tip's fairing.
    g.add(pylon(3.9, -0.6, -2.0, z - 0.05, z - 0.40, w=0.08, paint=TORNADO_GREY))
    g.add(block(6.93, 7.00, -2.40, -3.30, z - 0.03, z + 0.03, TORNADO_GREY))
    return g.transformed(lambda p: Vector((p.x - px, p.y - py, p.z - z)))


def tornado():
    rings = [f16_section(*s) for s in TORNADO_BODY]
    g = tube(rings, TORNADO_GREY, paints=[RADOME] * 4 + [TORNADO_GREY] * (len(rings) - 5))
    # The nose probe.
    g.add(frustum((0, 7.83, -0.24), (0, 8.36, -0.24), 0.04, 0.015, GREY_L, n=4))
    g.add(tube([f16_arch(*s) for s in TORNADO_CANOPY], CANOPY))
    g.add(tube([f16_arch(*s) for s in TORNADO_FAIRING], TORNADO_GREY))
    g.add(tube([f16_section(y, top, top - 0.3, hw, top - 0.42, hw, 3.0, 3.0, m=2) for y, top, hw in TORNADO_DORSAL],
               TORNADO_GREY))
    for s in (-1, 1):
        # The intakes under the gloves, their trunks running back into the engine bay.
        g.add(box_intake(s * 1.05, 2.95, 1.5, -0.25, 0.36, 0.52, slant=0.5, paint=TORNADO_GREY))
        g.add(loft([(1.5, -0.25, 0.36, 0.52), (-1.0, -0.22, 0.30, 0.50), (-3.6, -0.20, 0.12, 0.45)], TORNADO_GREY, n=8,
                   power=2.8, x=s * 1.05))
        # The engines' nozzles side by side, glowing.
        g.add(nozzle(s * 0.55, -6.75, -0.15, 0.42, 0.6, n=10))
    # The fixed gloves (thicker than the outer wings sliding into them), the tailerons with
    # a little anhedral, the fin on its dorsal fairing.
    g.add(wing((1.05, 2.90, TORNADO_WING_Z), 4.50, (2.35, -0.25, TORNADO_WING_Z), 1.35, TORNADO_GREY, t=0.08))
    g.add(wing((1.00, -4.10, -0.05), 3.25, (3.43, -6.15, -0.15), 1.25, TORNADO_GREY, t=0.045, t_tip=0.035))
    g.add(wing((0, -3.60, 1.30), 4.00, (0, -6.30, 4.15), 1.85, TORNADO_GREY, t=0.05, vertical=True, mirror=False))
    fl = Geo().add(flame(0.55, -7.35, -0.15, 0.38, 3.0)).add(flame(-0.55, -7.35, -0.15, 0.38, 3.0))
    right = tornado_wing()
    px, py = TORNADO_PIVOT
    parts = [(right, (px, py, TORNADO_WING_Z), TORNADO_WING_SLOTS[0]),
             (right.mirrored(), (-px, py, TORNADO_WING_Z), TORNADO_WING_SLOTS[1])]
    # The navigation lights on the gloves' tips (the outer wings move), the white one on the
    # fin's tip.
    return Craft(g, (-2.40, -0.70, TORNADO_WING_Z), (2.40, -0.70, TORNADO_WING_Z), [(0, -8.10, 4.15)], fl,
                 gear=tornado_gear(), parts=parts)


def tornado_gear():
    """The gear down: twin nose wheels under the cockpit, the main legs from the fuselage's
    underside below the gloves, their wheels outboard."""
    return tricycle(TORNADO_GROUND, (3.85, -0.78, 0.25, True), (-1.18, 0.95, -0.62, 1.55, 0.42), TORNADO_GREY)


def twin_nacelle_fighter(L, span, flanker=False, paint=GREY_B):
    """MiG-29 / Su-27 layout: slim forebody, LERX, widely spaced nacelles under the wing, twin fins."""
    k = L / 17.3
    g = loft([(8.6 * k, 0.15, 0, 0), (7.8 * k, 0.15, 0.32, 0.32), (6.4 * k, 0.25, 0.55, 0.6), (4.2 * k, 0.3, 0.65, 0.7),
              (2.0 * k, 0.25, 0.75, 0.55), (-1.5 * k, 0.15, 0.7, 0.4), (-5.0 * k, 0.1, 0.5, 0.35),
              ((-8.7 if flanker else -6.5) * k, 0.1, 0.3, 0.25)], paint, n=8, power=2.4)
    g.add(loft([(8.6 * k, 0.15, 0, 0), (7.8 * k, 0.15, 0.32, 0.32), (7.0 * k, 0.18, 0.45, 0.47)], RADOME, n=8))
    nx = 1.05 * k
    for s in (-1, 1):
        g.add(box_intake(s * nx, 3.2 * k, 1.2 * k, -0.45, 0.42 * k, 0.48, slant=-0.5))
        g.add(loft([(1.2 * k, -0.45, 0.42 * k, 0.48), (-2.0 * k, -0.3, 0.55 * k, 0.55), (-6.6 * k, -0.25, 0.55 * k, 0.5),
                    (-7.6 * k, -0.25, 0.48 * k, 0.45)], paint, n=8, power=3.0, x=s * nx))
        g.add(nozzle(s * nx, -7.6 * k, -0.25, 0.48 * k, 0.8 * k))
    g.add(canopy(5.8 * k, 3.0 * k, 0.85, 0.4, 0.5))
    # LERX and wing
    g.add(wing((0.5, 6.0 * k, 0.15), 5.6 * k, (1.6 * k, 0.8 * k, 0.1), 2.0 * k, paint, t=0.035))
    g.add(wing((1.5 * k, 0.9 * k, 0.1), 4.4 * k, (span / 2, -3.3 * k, 0.05), 1.2 * k, paint, t=0.05))
    g.add(wing((1.6 * k, -5.2 * k, -0.1), 2.6 * k, (3.9 * k, -7.2 * k, -0.15), 1.0 * k, paint, t=0.05))
    for s in (-1, 1):
        g.add(wing((s * 1.55 * k, -3.0 * k, 0.45), 3.2 * k, (s * 1.95 * k, -5.8 * k, 3.4 * k), 1.3 * k, paint, t=0.06,
                   vertical=True, mirror=False))
    if flanker:
        for s in (-1, 1):
            g.add(missile(s * span / 2, -2.6 * k, 0.05, 3.6 * k, 0.1))
    fl = Geo()
    for s in (-1, 1):
        fl.add(flame(s * nx, -8.4 * k, -0.25, 0.45 * k, 3.0))
    tip = (span / 2, -3.9 * k, 0.05)
    return Craft(g, (-tip[0], tip[1], tip[2]), tip, [(-1.95 * k, -6.4 * k, 3.4 * k), (1.95 * k, -6.4 * k, 3.4 * k)], fl)


def mig29():
    return twin_nacelle_fighter(17.3, 11.4)


def su27():
    return twin_nacelle_fighter(21.9, 14.7, flanker=True)


def f18(two_seat=False):
    g = loft([(8.5, 0.1, 0, 0), (7.6, 0.1, 0.33, 0.33), (6.0, 0.15, 0.55, 0.6), (4.0, 0.2, 0.65, 0.72), (1.5, 0.15, 0.95, 0.65),
              (-2.0, 0.1, 1.05, 0.6), (-5.6, 0.1, 0.95, 0.55), (-6.9, 0.1, 0.85, 0.5)], GREY_B, n=8, power=2.6)
    g.add(loft([(8.5, 0.1, 0, 0), (7.6, 0.1, 0.33, 0.33), (6.9, 0.12, 0.45, 0.47)], RADOME, n=8))
    for s in (-1, 1):
        g.add(box_intake(s * 1.15, 1.6, -1.6, -0.35, 0.38, 0.45, slant=-0.4))
        g.add(nozzle(s * 0.55, -6.8, 0.1, 0.5, 1.0))
    g.add(canopy(6.3, 2.6 if two_seat else 3.6, 0.8, 0.42, 0.5))
    g.add(wing((0.5, 6.0, 0.25), 6.0, (1.3, 0.6, 0.2), 1.0, GREY_B, t=0.03))
    g.add(wing((1.2, 0.6, 0.15), 4.0, (6.1, -2.2, 0.1), 1.5, GREY_B, t=0.05))
    g.add(wing((1.0, -4.9, 0.0), 2.8, (3.4, -6.8, -0.1), 1.3, GREY_B, t=0.05))
    for s in (-1, 1):
        g.add(wing((s * 1.0, -2.2, 0.75), 3.2, (s * 2.1, -4.4, 3.5), 1.4, GREY_B, t=0.05, vertical=True, mirror=False))
        g.add(missile(s * 6.3, -0.6, 0.1, 3.0))
    fl = Geo().add(flame(0.55, -7.8, 0.1, 0.45, 3.0)).add(flame(-0.55, -7.8, 0.1, 0.45, 3.0))
    gear = tricycle(F18_GROUND, (4.0, -0.5, 0.28, True), (-1.4, 1.3, -0.8, 1.55, 0.38), GREY_B)
    return Craft(g, (-6.3, -3.0, 0.1), (6.3, -3.0, 0.1), [(-2.1, -5.0, 3.5), (2.1, -5.0, 3.5)], fl, gear=gear)


def mig25():
    g = loft([(9.9, 0.3, 0, 0), (8.6, 0.3, 0.45, 0.45), (6.5, 0.35, 0.7, 0.7), (4.8, 0.4, 0.75, 0.75), (2.0, 0.4, 1.0, 0.6),
              (-4.0, 0.3, 1.0, 0.55), (-8.2, 0.3, 1.0, 0.55)], GREY_L, n=8, power=2.4)
    g.add(loft([(9.9, 0.3, 0, 0), (8.6, 0.3, 0.45, 0.45), (7.8, 0.32, 0.6, 0.6)], RADOME, n=8))
    for s in (-1, 1):
        # huge wedge intakes merging into the engine trunks
        g.add(hexa([(s * 1.0, 4.6, -0.9), (s * 2.3, 3.8, -0.9), (s * 2.3, -6.0, -0.6), (s * 1.0, -6.0, -0.6),
                    (s * 1.0, 4.6, 1.0), (s * 2.3, 4.6, 1.0), (s * 2.0, -6.0, 0.8), (s * 1.0, -6.0, 0.8)], GREY_L))
        g.face2([(s * 1.1, 4.54, -0.8), (s * 2.2, 3.89, -0.8), (s * 2.2, 4.64, 0.9), (s * 1.1, 4.64, 0.9)], BLACK)
        g.add(loft([(-6.0, 0.1, 0.75, 0.75), (-8.6, 0.1, 0.72, 0.72)], GREY_L, n=8, x=s * 0.95))
        g.add(nozzle(s * 0.95, -8.6, 0.1, 0.7, 1.2))
    g.add(canopy(6.6, 4.4, 1.0, 0.4, 0.35))
    g.add(wing((2.3, 0.6, 0.9), 6.2, (7.0, -4.3, 0.6), 2.0, GREY_L, t=0.04))
    g.add(wing((1.6, -5.9, 0.2), 3.2, (4.4, -8.1, 0.1), 1.6, GREY_L, t=0.05))
    for s in (-1, 1):
        g.add(wing((s * 1.6, -3.6, 0.9), 4.4, (s * 2.4, -6.6, 4.2), 1.8, GREY_L, t=0.05, vertical=True, mirror=False))
        g.add(wing((s * 1.7, -5.2, -0.6), 2.4, (s * 2.1, -6.4, -1.6), 1.2, GREY_L, t=0.05, vertical=True, mirror=False))
    fl = Geo().add(flame(0.95, -9.8, 0.1, 0.62, 3.6)).add(flame(-0.95, -9.8, 0.1, 0.62, 3.6))
    return Craft(g, (-7.0, -5.3, 0.6), (7.0, -5.3, 0.6), [(-2.4, -7.6, 4.2), (2.4, -7.6, 4.2)], fl)


def f14():
    g = loft([(9.5, 0.2, 0, 0), (8.5, 0.2, 0.4, 0.4), (6.5, 0.3, 0.65, 0.7), (4.0, 0.35, 0.8, 0.8), (1.5, 0.3, 1.0, 0.6),
              (-3.0, 0.2, 1.0, 0.4), (-7.0, 0.15, 0.8, 0.3)], GREY_L, n=8, power=2.4)
    g.add(loft([(9.5, 0.2, 0, 0), (8.5, 0.2, 0.4, 0.4), (7.6, 0.22, 0.55, 0.55)], RADOME, n=8))
    for s in (-1, 1):
        g.add(box_intake(s * 1.9, 3.0, 0.0, -0.2, 0.55, 0.7, slant=1.0, paint=GREY_L))
        g.add(loft([(0.0, -0.2, 0.55, 0.7), (-5.0, -0.1, 0.65, 0.65), (-8.1, -0.1, 0.62, 0.62)], GREY_L, n=8, power=2.6, x=s * 1.9))
        g.add(nozzle(s * 1.9, -8.1, -0.1, 0.62, 1.1))
        g.add(wing((s * 2.0, -3.6, 0.5), 3.6, (s * 2.6, -6.6, 3.4), 1.4, GREY_L, t=0.05, vertical=True, mirror=False))
    g.add(canopy(6.8, 2.8, 0.9, 0.45, 0.5))
    # glove + half-swept outer wing
    g.add(wing((0.9, 4.6, 0.3), 6.0, (2.6, 0.4, 0.3), 3.2, GREY_L, t=0.04))
    g.add(wing((2.6, 0.4, 0.3), 3.0, (7.6, -3.6, 0.3), 1.6, GREY_L, t=0.05))
    g.add(wing((2.4, -5.8, 0.0), 3.4, (5.0, -8.4, 0.0), 1.4, GREY_L, t=0.04))
    fl = Geo().add(flame(1.9, -9.2, -0.1, 0.55, 3.2)).add(flame(-1.9, -9.2, -0.1, 0.55, 3.2))
    return Craft(g, (-7.6, -4.6, 0.3), (7.6, -4.6, 0.3), [(-2.6, -7.2, 3.4), (2.6, -7.2, 3.4)], fl)


def f4():
    g = loft([(9.6, 0.1, 0, 0), (8.5, 0.1, 0.45, 0.45), (6.5, 0.2, 0.65, 0.7), (4.0, 0.3, 0.75, 0.8), (1.0, 0.2, 1.0, 0.75),
              (-4.0, 0.1, 0.9, 0.7), (-6.5, 0.2, 0.5, 0.55), (-9.6, 0.4, 0.2, 0.25)], GREY_D, n=8, power=2.3)
    g.add(loft([(9.6, 0.1, 0, 0), (8.5, 0.1, 0.45, 0.45), (7.6, 0.12, 0.58, 0.6)], RADOME, n=8))
    for s in (-1, 1):
        g.add(box_intake(s * 1.15, 4.0, 0.0, -0.05, 0.4, 0.7, slant=-0.4, paint=GREY_D))
        g.add(nozzle(s * 0.55, -5.0, -0.45, 0.45, 0.9))
        # outer wing panel with dihedral
        g.add(wing((s * 4.0, -0.9, 0.0), 3.2, (s * 5.85, -2.6, 0.75), 1.6, GREY_D, t=0.05, mirror=False))
    g.add(canopy(6.7, 2.6, 0.85, 0.45, 0.5))
    g.add(wing((0.9, 1.6, -0.2), 5.6, (4.0, -0.9, 0.0), 3.2, GREY_D, t=0.05))
    g.add(wing((0.3, -6.2, 0.4), 2.6, (2.9, -8.0, -0.4), 1.3, GREY_D, t=0.05))
    g.add(wing((0, -4.6, 0.6), 3.8, (0, -7.5, 3.6), 1.5, GREY_D, t=0.05, vertical=True, mirror=False))
    fl = Geo().add(flame(0.55, -5.9, -0.45, 0.42, 2.8)).add(flame(-0.55, -5.9, -0.45, 0.42, 2.8))
    return Craft(g, (-5.85, -3.2, 0.75), (5.85, -3.2, 0.75), [(0, -8.0, 3.6)], fl)


def mig21():
    g = loft([(7.6, 0.0, 0.0, 0.0), (7.0, 0.0, 0.28, 0.28), (6.6, 0.0, 0.5, 0.5), (4.0, 0.05, 0.62, 0.7), (0.0, 0.05, 0.66, 0.68),
              (-4.6, 0.05, 0.6, 0.6), (-6.4, 0.05, 0.5, 0.5)], GREY_L, n=8)
    g.add(frustum((0, 7.0, 0), (0, 7.9, 0), 0.26, 0.02, RADOME, 8))
    g.add(frustum((0, 6.95, 0), (0, 6.9, 0), 0.48, 0.48, BLACK, 8, caps=True))
    g.add(loft([(4.4, 0.55, 0.0, 0.0), (2.0, 0.6, 0.25, 0.25), (-3.6, 0.55, 0.18, 0.18)], GREY_L, n=6))
    g.add(canopy(4.6, 2.4, 0.55, 0.32, 0.38))
    g.add(wing((0.55, 1.6, -0.15), 5.4, (3.6, -3.4, -0.15), 0.6, GREY_L, t=0.04))
    g.add(wing((0.4, -4.6, 0.0), 1.8, (1.9, -5.7, 0.0), 0.8, GREY_L, t=0.05))
    g.add(wing((0, -2.4, 0.6), 3.4, (0, -5.0, 2.9), 1.2, GREY_L, t=0.05, vertical=True, mirror=False))
    g.add(wing((0, -4.6, -0.55), 1.4, (0, -5.4, -1.0), 0.8, GREY_L, t=0.06, vertical=True, mirror=False))
    g.add(nozzle(0, -6.4, 0.05, 0.5, 0.6))
    fl = flame(0, -7.0, 0.05, 0.45, 2.8)
    return Craft(g, (-3.6, -3.7, -0.15), (3.6, -3.7, -0.15), [(0, -5.6, 2.9)], fl)


def mig23():
    g = loft([(8.4, 0.1, 0, 0), (7.4, 0.1, 0.4, 0.4), (5.5, 0.2, 0.6, 0.65), (3.5, 0.3, 0.7, 0.75), (0.0, 0.2, 0.9, 0.7),
              (-5.0, 0.15, 0.75, 0.65), (-7.3, 0.15, 0.6, 0.6)], GREY_D, n=8, power=2.4)
    g.add(loft([(8.4, 0.1, 0, 0), (7.4, 0.1, 0.4, 0.4), (6.8, 0.12, 0.5, 0.5)], RADOME, n=8))
    for s in (-1, 1):
        g.add(box_intake(s * 1.15, 3.6, -1.5, 0.15, 0.45, 0.65, slant=0.3, paint=GREY_D))
    g.add(canopy(5.0, 2.6, 0.75, 0.38, 0.45))
    # fixed glove, outer panels at mid sweep
    g.add(wing((0.9, 1.4, 0.45), 4.4, (2.2, -0.6, 0.45), 2.6, GREY_D, t=0.04))
    g.add(wing((2.2, -0.4, 0.45), 2.6, (6.2, -3.6, 0.45), 1.3, GREY_D, t=0.05))
    g.add(wing((0.7, -5.1, 0.1), 2.6, (3.3, -6.8, -0.2), 1.1, GREY_D, t=0.05))
    g.add(wing((0, -2.4, 0.75), 4.2, (0, -5.8, 3.8), 1.6, GREY_D, t=0.05, vertical=True, mirror=False))
    g.add(wing((0, -5.2, -0.6), 1.8, (0, -6.0, -1.6), 1.1, GREY_D, t=0.06, vertical=True, mirror=False))
    g.add(nozzle(0, -7.3, 0.15, 0.6, 0.8))
    fl = flame(0, -8.1, 0.15, 0.55, 3.0)
    gear = tricycle(MIG23_GROUND, (4.6, -0.45, 0.3, True), (-1.0, 1.2, -0.5, 1.33, 0.4), GREY_D)
    return Craft(g, (-6.2, -4.2, 0.45), (6.2, -4.2, 0.45), [(0, -6.6, 3.8)], fl, gear=gear)


def f5():
    g = loft([(7.2, 0.0, 0, 0), (6.0, 0.05, 0.3, 0.3), (4.4, 0.15, 0.5, 0.55), (2.5, 0.2, 0.6, 0.65), (0.0, 0.1, 0.75, 0.6),
              (-5.0, 0.05, 0.75, 0.55), (-6.6, 0.05, 0.7, 0.5)], GREY_B, n=8, power=2.4)
    for s in (-1, 1):
        g.add(box_intake(s * 0.85, 1.6, -1.2, 0.05, 0.3, 0.4, slant=-0.2, paint=GREY_B))
        g.add(nozzle(s * 0.38, -6.6, 0.05, 0.34, 0.6))
        g.add(missile(s * 4.1, -0.6, 0.0, 2.8))
    g.add(canopy(4.6, 2.2, 0.62, 0.36, 0.42))
    g.add(wing((0.3, 2.6, 0.0), 2.0, (0.7, 0.8, 0.0), 1.0, GREY_B, t=0.03))
    g.add(wing((0.7, 0.9, 0.0), 3.6, (4.05, -1.6, 0.0), 1.2, GREY_B, t=0.045))
    g.add(wing((0.6, -5.0, 0.15), 1.8, (2.4, -6.4, 0.15), 0.8, GREY_B, t=0.05))
    g.add(wing((0, -3.4, 0.6), 3.0, (0, -6.0, 3.1), 1.1, GREY_B, t=0.05, vertical=True, mirror=False))
    fl = Geo().add(flame(0.38, -7.2, 0.05, 0.3, 2.4)).add(flame(-0.38, -7.2, 0.05, 0.3, 2.4))
    return Craft(g, (-4.1, -2.4, 0.0), (4.1, -2.4, 0.0), [(0, -6.6, 3.1)], fl)


def mirage_f1():
    g = loft([(7.6, 0.1, 0, 0), (6.6, 0.1, 0.35, 0.35), (4.8, 0.2, 0.55, 0.6), (2.6, 0.25, 0.65, 0.7), (0.0, 0.2, 0.75, 0.7),
              (-5.0, 0.15, 0.7, 0.65), (-6.8, 0.15, 0.6, 0.6)], GREY_B, n=8, power=2.3)
    g.add(frustum((0, 7.6, 0.1), (0, 8.4, 0.1), 0.04, 0.02, BLACK, 4))
    for s in (-1, 1):
        g.add(loft([(3.0, 0.25, 0.0, 0.0), (2.2, 0.25, 0.4, 0.5), (-1.0, 0.25, 0.45, 0.55)], GREY_B, n=6, x=s * 0.75))
    g.add(canopy(4.8, 2.4, 0.72, 0.36, 0.42))
    g.add(wing((0.6, 0.0, 0.62), 3.8, (4.2, -2.8, 0.45), 1.6, GREY_B, t=0.04))
    g.add(wing((0.6, -4.8, 0.0), 2.2, (2.6, -6.4, 0.0), 1.0, GREY_B, t=0.05))
    g.add(wing((0, -2.6, 0.7), 3.6, (0, -5.6, 3.4), 1.3, GREY_B, t=0.05, vertical=True, mirror=False))
    g.add(nozzle(0, -6.8, 0.15, 0.6, 0.7))
    fl = flame(0, -7.5, 0.15, 0.55, 3.0)
    return Craft(g, (-4.2, -3.6, 0.45), (4.2, -3.6, 0.45), [(0, -6.2, 3.4)], fl)


def mig17():
    g = loft([(5.6, 0.0, 0.42, 0.45), (4.6, 0.05, 0.6, 0.65), (2.0, 0.1, 0.68, 0.72), (-1.5, 0.1, 0.6, 0.65), (-4.6, 0.15, 0.4, 0.45),
              (-5.6, 0.15, 0.36, 0.4)], GREY_L, n=8)
    g.add(frustum((0, 5.62, 0), (0, 5.55, 0), 0.36, 0.36, BLACK, 8))
    g.add(canopy(3.9, 2.0, 0.55, 0.32, 0.38))
    g.add(wing((0.6, 1.6, -0.1), 3.4, (4.8, -2.6, -0.3), 1.5, GREY_L, t=0.06))
    for s in (-1, 1):
        for fx in (2.0, 3.4):
            f = fx * s
            g.add(hexa([(f - 0.02, 1.6 - (fx - 0.6), -0.05), (f + 0.02, 1.6 - (fx - 0.6), -0.05), (f + 0.02, -0.9 - (fx - 0.6) * 0.75, -0.05),
                        (f - 0.02, -0.9 - (fx - 0.6) * 0.75, -0.05), (f - 0.02, 1.4 - (fx - 0.6), 0.2), (f + 0.02, 1.4 - (fx - 0.6), 0.2),
                        (f + 0.02, -0.8 - (fx - 0.6) * 0.75, 0.2), (f - 0.02, -0.8 - (fx - 0.6) * 0.75, 0.2)], GREY_D))
    g.add(wing((0, -2.4, 0.4), 3.4, (0, -5.0, 3.0), 1.6, GREY_L, t=0.06, vertical=True, mirror=False))
    g.add(wing((0.1, -3.9, 1.8), 1.9, (1.8, -5.0, 1.7), 0.9, GREY_L, t=0.05))
    g.add(nozzle(0, -5.6, 0.15, 0.36, 0.4))
    fl = flame(0, -6.0, 0.15, 0.32, 2.2)
    return Craft(g, (-4.8, -3.4, -0.3), (4.8, -3.4, -0.3), [(0, -5.8, 3.0)], fl)


def yak38():
    g = loft([(8.2, 0.1, 0, 0), (7.2, 0.1, 0.35, 0.35), (5.2, 0.2, 0.6, 0.65), (3.0, 0.3, 0.7, 0.8), (0.0, 0.2, 0.95, 0.85),
              (-5.0, 0.2, 0.75, 0.7), (-8.0, 0.3, 0.35, 0.4)], GREY_D, n=8, power=2.3)
    for s in (-1, 1):
        g.add(box_intake(s * 1.05, 2.4, -1.4, 0.2, 0.42, 0.65, slant=0.3, paint=GREY_D))
        g.add(frustum((s * 1.15, -1.0, -0.2), (s * 1.15, -1.9, -0.3), 0.45, 0.4, DARK, 8))
    g.add(canopy(5.6, 3.4, 0.75, 0.38, 0.45))
    g.add(wing((0.9, 0.0, 0.7), 3.6, (3.65, -2.0, 0.45), 1.6, GREY_D, t=0.05))
    g.add(wing((0.4, -5.2, 0.4), 1.9, (2.0, -6.4, 0.4), 0.9, GREY_D, t=0.05))
    g.add(wing((0, -3.6, 0.9), 3.4, (0, -6.6, 3.2), 1.4, GREY_D, t=0.05, vertical=True, mirror=False))
    return Craft(g, (-3.65, -2.8, 0.45), (3.65, -2.8, 0.45), [(0, -7.2, 3.2)])


def a6():
    g = loft([(8.4, -0.1, 0, 0), (7.6, -0.1, 0.6, 0.55), (6.0, 0.0, 1.0, 0.95), (3.5, 0.0, 1.05, 0.95), (0.0, 0.0, 0.95, 0.75),
              (-4.0, 0.2, 0.55, 0.5), (-8.2, 0.4, 0.25, 0.3)], GREY_L, n=8, power=2.2)
    g.add(loft([(6.2, 0.7, 0.05, 0.05), (5.4, 0.95, 0.8, 0.5), (4.0, 1.05, 0.85, 0.5), (3.0, 0.8, 0.5, 0.2)], CANOPY, n=6))
    for s in (-1, 1):
        g.add(nacelle(s * 1.1, 3.0, -2.2, -0.5, 0.48, GREY_L))
    g.add(frustum((0, 6.0, 0.9), (0, 7.4, 1.1), 0.05, 0.03, BLACK, 4))
    g.add(wing((0.9, 1.4, 0.2), 4.4, (8.1, -2.4, 0.2), 1.8, GREY_L, t=0.06))
    g.add(wing((0.3, -6.0, 0.6), 2.4, (3.4, -7.8, 0.6), 1.2, GREY_L, t=0.05))
    g.add(wing((0, -4.6, 0.6), 3.2, (0, -7.4, 4.0), 1.7, GREY_L, t=0.05, vertical=True, mirror=False))
    return Craft(g, (-8.1, -3.3, 0.2), (8.1, -3.3, 0.2), [(0, -8.2, 4.0)])


def ef111():
    g = loft([(11.6, 0.0, 0, 0), (10.4, 0.0, 0.45, 0.45), (8.6, 0.1, 0.85, 0.75), (6.4, 0.2, 1.1, 0.9), (2.0, 0.2, 1.5, 0.75),
              (-6.0, 0.2, 1.6, 0.7), (-9.6, 0.2, 1.4, 0.6), (-10.6, 0.25, 0.9, 0.5)], GREY_B, n=8, power=2.6)
    g.add(loft([(7.8, 0.8, 0.05, 0.05), (7.2, 1.05, 0.85, 0.4), (5.6, 1.15, 0.85, 0.4), (4.4, 0.9, 0.5, 0.2)], CANOPY, n=6))
    for s in (-1, 1):
        g.add(nozzle(s * 0.75, -10.2, 0.15, 0.62, 1.2))
        g.add(box_intake(s * 1.65, 2.6, -1.2, 0.1, 0.35, 0.6, slant=0.3, paint=GREY_B))
    g.add(wing((1.0, 3.4, 0.6), 5.4, (2.6, 2.4, 0.6), 4.0, GREY_B, t=0.04))
    g.add(wing((2.4, 2.2, 0.55), 3.4, (7.0, -3.3, 0.55), 1.5, GREY_B, t=0.05))
    g.add(wing((1.2, -7.6, 0.0), 3.6, (4.4, -10.6, 0.0), 1.5, GREY_B, t=0.05))
    g.add(wing((0, -5.0, 0.8), 4.8, (0, -9.6, 5.0), 2.4, GREY_B, t=0.05, vertical=True, mirror=False))
    g.add(loft([(-8.0, 5.2, 0.0, 0.0), (-8.6, 5.25, 0.32, 0.38), (-11.8, 5.25, 0.32, 0.38), (-12.3, 5.25, 0.1, 0.1)], GREY_B, n=6))
    fl = Geo().add(flame(0.75, -11.4, 0.15, 0.55, 3.4)).add(flame(-0.75, -11.4, 0.15, 0.55, 3.4))
    return Craft(g, (-7.0, -4.0, 0.55), (7.0, -4.0, 0.55), [(0, -12.3, 5.25)], fl)


# The A-10A from its three-view (16.26 m long, 17.53 m span): stations along y, heights above
# the fuselage datum (z 0, the nose's middle). Fuselage rings as the F-16's (`f16_section`): the
# blunt nose round the cannon, the deep cockpit section with the hump behind the canopy, the
# fuselage tapering in straight lines from the canopy to the tail cone under the fins.
A10_BODY = [
    (8.15, -0.06, -0.06, 0.00, -0.06, 0.00, 2.0, 2.0),
    (8.02, 0.16, -0.06, 0.22, -0.30, 0.22, 2.0, 2.0),
    (7.70, 0.34, -0.06, 0.34, -0.40, 0.34, 2.0, 2.0),
    (7.20, 0.53, -0.05, 0.46, -0.47, 0.46, 2.0, 2.2),
    (6.62, 0.71, -0.03, 0.57, -0.50, 0.56, 2.2, 2.4),
    (5.90, 0.86, 0.0, 0.65, -0.52, 0.64, 2.6, 2.6),
    (5.00, 0.96, 0.0, 0.69, -0.54, 0.68, 2.8, 2.8),
    (4.00, 1.00, 0.0, 0.70, -0.55, 0.69, 2.8, 2.8),
    (3.00, 1.22, 0.0, 0.70, -0.57, 0.69, 2.4, 2.8),
    (0.30, 0.97, 0.0, 0.70, -0.72, 0.69, 2.6, 3.0),
    (-1.40, 0.82, 0.0, 0.66, -0.78, 0.65, 2.6, 3.0),
    (-3.80, 0.61, 0.0, 0.52, -0.42, 0.51, 2.6, 2.8),
    (-5.60, 0.44, 0.0, 0.42, -0.15, 0.41, 2.6, 2.6),
    (-7.50, 0.26, 0.0, 0.32, -0.08, 0.31, 2.4, 2.4),
    (-8.10, 0.12, 0.04, 0.08, -0.02, 0.07, 2.0, 2.0),
]
# The flat-paneled windscreen (a centre panel and two side panels, leaning back from the nose):
# (y, sill z, half width at the sill, half width and z of the top edge).
A10_WINDSCREEN = [(6.66, 0.70, 0.30, 0.18, 0.73), (5.55, 0.90, 0.40, 0.18, 1.46)]
# The frame bow behind it, a band round the glass seen only from outside: front and rear y,
# sill z, half width, top.
A10_BOW = (5.62, 5.48, 0.90, 0.42, 1.54)
# The bubble canopy over the hump: (y, sill z, half width, top).
A10_CANOPY = [(5.50, 0.90, 0.40, 1.50), (4.80, 0.95, 0.40, 1.58), (4.00, 0.98, 0.39, 1.52), (3.50, 1.00, 0.37, 1.40),
              (3.05, 1.02, 0.30, 1.22)]
# The wing (low, through under the fuselage): its plane, the main gear pods' x where the outer
# panels' dihedral starts, the engines' centres on their pylons (x, z), their radius.
A10_WING_Z = -0.42
A10_POD_X = 2.57
A10_ENGINE = (1.44, 0.80)
A10_ENGINE_R = 0.68
A10_FIN_X = 2.87
A10_GROUND = -2.06


def a10_wing_le(x):
    """The wing's leading edge (y, z) at x out from the centreline: straight, the outer panels
    7 degrees up from the pods."""
    t = (x - 0.70) / (8.76 - 0.70)
    z = A10_WING_Z + max(0.0, x - A10_POD_X) * math.tan(math.radians(7.0))
    return 1.37 + t * (0.69 - 1.37), z


def a10_wing_chord(x):
    t = (x - 0.70) / (8.76 - 0.70)
    return 3.17 + t * (1.99 - 3.17)


def a10():
    p = A10_GREY
    rings = [f16_section(*s) for s in A10_BODY]
    g = tube(rings, p)
    g.add(tube([[(hw, y, sill), (tw, y, top), (-tw, y, top), (-hw, y, sill)] for y, sill, hw, tw, top in A10_WINDSCREEN],
               CANOPY))
    y0, y1, sill, hw, top = A10_BOW
    front, rear = f16_arch(y0, sill, hw, top), f16_arch(y1, sill, hw, top)
    for k in range(len(front) - 1):
        g.face([front[k], front[k + 1], rear[k + 1], rear[k]], DARK, (0, (y0 + y1) / 2, sill))
    g.add(tube([f16_arch(*s) for s in A10_CANOPY], CANOPY))
    # The GAU-8's muzzle under the nose, the refuelling receptacle's fairing ahead of the canopy.
    g.add(frustum((0, 7.95, -0.34), (0, 8.32, -0.34), 0.13, 0.12, DARK, 8))
    g.add(frustum((0, 8.32, -0.34), (0, 8.36, -0.34), 0.07, 0.07, BLACK, 6))
    g.add(loft([(7.50, 0.42, 0.0, 0.0), (7.30, 0.47, 0.10, 0.06), (6.90, 0.60, 0.10, 0.06)], DARK, n=6))
    # The straight wing: the centre section flat between the pods, the outer panels with
    # dihedral and drooped tips.
    xs = [0.40, A10_POD_X, 8.60]
    for a, b in zip(xs, xs[1:]):
        (ya, za), (yb, zb) = a10_wing_le(a), a10_wing_le(b)
        g.add(wing((a, ya, za), a10_wing_chord(a), (b, yb, zb), a10_wing_chord(b), p, t=0.16 if a < 1 else 0.13,
                   t_tip=0.13 if a < 1 else 0.10))
    y8, z8 = a10_wing_le(8.60)
    tip = [(8.60, y8, z8), (8.60, y8 - a10_wing_chord(8.60), z8), (8.80, y8 - a10_wing_chord(8.60) + 0.1, z8 - 0.18),
           (8.80, y8 - 0.1, z8 - 0.18)]
    g.face2(tip, p)
    g.face2([(-x, y, z) for x, y, z in tip], p)
    for s in (-1, 1):
        # The main gear pods under the wing, the half-retracted wheels showing under them.
        g.add(loft([(2.12, -0.55, 0.0, 0.0), (1.80, -0.58, 0.26, 0.26), (1.20, -0.62, 0.33, 0.30),
                    (-0.50, -0.60, 0.33, 0.30), (-1.20, -0.52, 0.12, 0.16)], p, n=8, x=s * A10_POD_X))
        g.add(mklib.wheel((s * A10_POD_X, -0.42, -0.86), 0.40, 0.24, BLACK, n=8))
        # The engines high on the rear fuselage: the big fan intakes, the nacelles tapering to
        # their exhausts, the pylons down to the fuselage's shoulders.
        ex, ez = A10_ENGINE
        r = A10_ENGINE_R
        g.add(loft([(-1.42, ez, r * 0.97, r * 0.97), (-1.62, ez, r, r), (-3.80, ez, r * 0.98, r * 0.98),
                    (-4.55, ez, r * 0.80, r * 0.80), (-4.75, ez, r * 0.62, r * 0.62)], p, n=12, x=s * ex))
        g.add(frustum((s * ex, -1.40, ez), (s * ex, -1.60, ez), r * 0.82, r * 0.82, BLACK, 12))
        g.add(frustum((s * ex, -1.58, ez), (s * ex, -1.70, ez), r * 0.24, r * 0.05, GREY_L, 8))
        g.add(frustum((s * ex, -4.74, ez), (s * ex, -5.02, ez), r * 0.58, r * 0.42, DARK, 10))
        g.add(frustum((s * ex, -4.80, ez), (s * ex, -5.03, ez), r * 0.40, r * 0.40, BLACK, 10))
        g.add(wing((s * 0.30, -1.75, 0.62), 1.75, (s * (ex - 0.25), -1.95, ez - r + 0.10), 1.55, p, t=0.12,
                   mirror=False))
        # The twin fins and rudders at the stabilizer's tips, reaching below it.
        g.add(wing((s * A10_FIN_X, -5.72, -0.85), 1.95, (s * A10_FIN_X, -6.10, 2.25), 1.50, p, t=0.07,
                   vertical=True, mirror=False))
        g.add(block(s * A10_FIN_X - 0.06, s * A10_FIN_X + 0.06, -7.30, -6.10, 2.22, 2.28, p))
        # The pylons: two on the centre section, three on each outer panel.
        for px in (1.65, 3.55, 4.75, 5.95):
            y, z = a10_wing_le(px)
            g.add(pylon(s * px, y - 0.50, y - 1.90, z - 0.08, z - 0.38, w=0.07, paint=p))
    # The fuselage's three stations under the centre section.
    for px in (-0.45, 0.0, 0.45):
        g.add(pylon(px, 1.10, -0.60, -0.70, -0.92, w=0.07, paint=p))
    # The stabilizer between the fins.
    g.add(wing((0.30, -5.90, 0.05), 1.40, (A10_FIN_X, -5.95, 0.05), 1.30, p, t=0.06))
    # The navigation lights on the drooped tips, the white one on the tail cone.
    tip_y = y8 - 0.9
    return Craft(g, (-8.80, tip_y, z8 - 0.18), (8.80, tip_y, z8 - 0.18), [(0, -8.15, 0.06)], gear=a10_gear())


def a10_gear():
    """The gear down: the nose leg offset right of the cannon, the main legs from the pods, the
    wheels under them."""
    g = Geo()
    r = 0.30
    axle = (0.30, 5.05, A10_GROUND + r)
    g.add(mklib.wheel(axle, r, 0.18, BLACK, n=8))
    g.add(frustum((0.30, 5.15, -0.54), (0.30, 5.08, axle[2] + 0.1), 0.07, 0.06, GREY_L, n=5))
    g.add(frustum((0.30, 4.40, -0.54), (0.30, 5.00, (axle[2] - 0.54) / 2), 0.035, 0.035, GREY_L, n=4))
    g.face2([(0.05, 5.70, -0.54), (0.05, 4.20, -0.54), (0.05, 4.20, -0.92), (0.05, 5.70, -0.92)], A10_GREY)
    r = 0.47
    for s in (-1, 1):
        axle = (s * A10_POD_X, -0.42, A10_GROUND + r)
        g.add(mklib.wheel(axle, r, 0.26, BLACK, n=8))
        g.add(frustum((s * (A10_POD_X - 0.16), -0.25, -0.80), (s * (A10_POD_X - 0.16), -0.42, axle[2] + 0.12), 0.08,
                      0.07, GREY_L, n=5))
        g.add(frustum((s * (A10_POD_X - 0.16), 0.40, -0.84), (s * (A10_POD_X - 0.16), -0.30, axle[2] + 0.40), 0.04,
                      0.04, GREY_L, n=4))
    return g


# The AV-8B Harrier II from its three-view (GR.5/7 drawing, 153 px per metre: 14.4 m long,
# 9.2 m span): stations along y, heights above the fuselage datum. Fuselage rings as the F-16's
# (`f16_section`): the slim nose and cockpit, the wide middle over the Pegasus with the intakes
# either side, the tail cone tapering to the reaction control boom.
HARRIER_GREY = Ramp(232, 238)
HARRIER_BODY = [
    (7.27, -0.08, -0.10, 0.00, -0.12, 0.00, 2.0, 2.0),
    (7.10, 0.02, -0.10, 0.10, -0.22, 0.10, 2.0, 2.0),
    (6.97, 0.10, -0.10, 0.15, -0.29, 0.15, 2.0, 2.0),
    (6.65, 0.25, -0.10, 0.25, -0.44, 0.25, 2.0, 2.2),
    (6.32, 0.34, -0.08, 0.37, -0.47, 0.36, 2.0, 2.4),
    (5.86, 0.50, -0.05, 0.52, -0.52, 0.50, 2.2, 2.6),
    (5.34, 0.55, 0.00, 0.57, -0.56, 0.55, 2.6, 2.8),
    (4.36, 0.57, 0.00, 0.58, -0.58, 0.56, 2.8, 2.8),
    (3.50, 0.68, 0.05, 0.60, -0.63, 0.58, 2.8, 2.8),
    (2.60, 0.96, 0.10, 0.66, -0.65, 0.62, 2.8, 3.0),
    (1.60, 1.10, 0.10, 0.76, -0.65, 0.72, 3.2, 3.0),
    (0.00, 1.14, 0.10, 0.80, -0.63, 0.74, 3.4, 3.0),
    (-1.52, 1.12, 0.10, 0.72, -0.59, 0.68, 3.2, 3.0),
    (-2.83, 1.04, 0.15, 0.60, -0.42, 0.56, 2.8, 2.8),
    (-4.14, 0.98, 0.30, 0.42, -0.23, 0.40, 2.2, 2.6),
    (-5.12, 0.97, 0.45, 0.24, -0.10, 0.22, 2.0, 2.4),
    (-6.10, 0.93, 0.58, 0.16, 0.20, 0.15, 2.0, 2.2),
    (-6.40, 0.93, 0.78, 0.15, 0.63, 0.15, 2.0, 2.0),
]
# The raised bubble canopy's glass (the inner line of the drawing): (y, sill z, half width,
# top), the windscreen from the nose's top, the frame bow between it and the bubble, and the
# fairing behind it into the spine.
HARRIER_CANOPY = [(5.90, 0.49, 0.06, 0.52), (5.40, 0.53, 0.34, 0.82), (4.95, 0.55, 0.46, 1.08),
                  (4.40, 0.57, 0.50, 1.27), (3.90, 0.60, 0.48, 1.31), (3.45, 0.63, 0.42, 1.25),
                  (3.15, 0.66, 0.34, 1.14)]
HARRIER_BOW = (5.00, 4.90, 0.55, 0.47, 1.11)
HARRIER_FAIRING = [(3.15, 0.66, 0.34, 1.14), (2.60, 0.80, 0.42, 1.10), (2.00, 0.90, 0.46, 1.12),
                   (1.40, 0.95, 0.44, 1.14)]
# The wing on top of the fuselage (14 degrees anhedral), flowing into its top: its root inside
# the fuselage near the centreline, the leading edge's root and tip, the chords.
HARRIER_WING_ROOT = (0.30, 1.86, 1.10)
HARRIER_WING_TIP = (4.49, -1.20, 0.08)
HARRIER_CHORDS = (3.73, 0.83)
# The nozzles: front (cold, fan air) and rear (hot) pairs, their pivots (x, y, z) on the
# fuselage's sides, turned down about x by the nozzle angle in state slot 25.
HARRIER_NOZZLES = [((1.10, 1.70, -0.22), 0.27), ((0.94, -0.40, -0.28), 0.29)]
HARRIER_NOZZLE_SLOT = 25
# The ground under the wheels: 5.5 degrees nose up on its short main gear (the side view's
# ground line), through the main wheels' bottom. `HARRIER_GROUND` is its height under the model
# origin (`HARRIER_LIFT`, `gear_depth` in src/aircraft.rs).
HARRIER_GROUND_PITCH = math.radians(5.5)
HARRIER_MAIN = (-0.95, -1.47)
HARRIER_OUTRIGGER_X = 2.60


def harrier_ground(y):
    """z of the ground under station y, the aircraft at its ground attitude."""
    return HARRIER_MAIN[1] - (y - HARRIER_MAIN[0]) * math.tan(HARRIER_GROUND_PITCH)


HARRIER_GROUND = harrier_ground(0.0)


def harrier_wing_at(x):
    """The wing's leading edge (y, z) and chord at x out from the centreline."""
    (x0, y0, z0), (x1, y1, z1) = HARRIER_WING_ROOT, HARRIER_WING_TIP
    t = (x - x0) / (x1 - x0)
    c0, c1 = HARRIER_CHORDS
    return y0 + t * (y1 - y0), z0 + t * (z1 - z0), c0 + t * (c1 - c0)


def harrier_nozzle(r, hot):
    """One nozzle about its pivot, pointing aft (nozzle angle 0): the bearing's drum on the
    fuselage's side, the elbow out of it with its vanes and dark mouth."""
    g = frustum((0, 0.18, 0), (0, -0.30, 0), r * 1.02, r * 0.92, DARK, 10, caps=False)
    g.add(frustum((0, -0.30, 0), (0, -0.62, -0.04), r * 0.92, r * 0.86, DARK if hot else HARRIER_GREY, 10, caps=False))
    g.add(frustum((0, -0.60, -0.04), (0, -0.55, -0.04), r * 0.80, r * 0.80, BLACK, 10))
    for k in (-1, 0, 1):
        z = -0.04 + k * r * 0.45
        w = r * (0.8 if k == 0 else 0.6)
        g.face2([(-w, -0.60, z), (w, -0.60, z), (w, -0.40, z + 0.03), (-w, -0.40, z + 0.03)], GREY_D)
    return g


def harrier():
    p = HARRIER_GREY
    rings = [f16_section(*s) for s in HARRIER_BODY]
    g = tube(rings, p, paints=[RADOME] * 3 + [p] * (len(rings) - 4))
    # The reaction control boom aft of the tail cone, its rounded end.
    g.add(loft([(-6.40, 0.78, 0.15, 0.15), (-7.00, 0.78, 0.14, 0.14), (-7.18, 0.78, 0.11, 0.11),
                (-7.27, 0.78, 0.04, 0.04)], p, n=8))
    # The angle rate bombing set's window in the nose's tip.
    g.add(frustum((0, 7.12, -0.10), (0, 7.20, -0.10), 0.07, 0.04, BLACK, 6))
    g.add(tube([f16_arch(*s) for s in HARRIER_CANOPY], CANOPY))
    y0, y1, sill, hw, top = HARRIER_BOW
    front, rear = f16_arch(y0, sill, hw, top), f16_arch(y1, sill, hw, top)
    for k in range(len(front) - 1):
        g.face([front[k], front[k + 1], rear[k + 1], rear[k]], DARK, (0, (y0 + y1) / 2, sill))
    g.add(tube([f16_arch(*s) for s in HARRIER_FAIRING], p))
    for s in (-1, 1):
        # The big semicircular intakes either side behind the cockpit, their trunks running
        # into the fuselage over the fan; the dark mouths.
        x = s * 0.86
        g.add(loft([(3.62, 0.14, 0.37, 0.70), (3.30, 0.15, 0.40, 0.71), (2.40, 0.15, 0.40, 0.70),
                    (1.30, 0.20, 0.30, 0.62), (0.20, 0.30, 0.12, 0.45)], p, n=12, x=x))
        g.add(frustum((x, 3.56, 0.14), (x, 3.63, 0.14), 0.30, 0.30, BLACK, 12, r0b=0.64, r1b=0.64))
        # The blow-in doors' row round the intake's lip.
        for z in (-0.25, 0.15, 0.55):
            g.face2([(s * 1.265, 3.20, z - 0.12), (s * 1.265, 2.95, z - 0.12), (s * 1.265, 2.95, z + 0.12),
                     (s * 1.265, 3.20, z + 0.12)], DARK)
        # The gun and ammunition pods under the fuselage, the strakes on them.
        g.add(loft([(3.00, -0.70, 0.05, 0.05), (2.60, -0.80, 0.17, 0.20), (1.60, -0.88, 0.20, 0.25),
                    (-0.40, -0.86, 0.20, 0.25), (-0.90, -0.78, 0.08, 0.14)], p, n=8, x=s * 0.38))
        g.add(block(s * 0.37, s * 0.39, 1.90, -0.80, -1.25, -1.05, p))
        # The leading edge root extensions.
        g.add(wing((s * 0.70, 2.30, 0.96), 0.95, (s * 1.02, 1.36, 0.96), 0.25, p, t=0.03, mirror=False))
    # The wing; the outrigger fairings; the pylons (three a side and the Sidewinder rails
    # outboard) and the centreline station between the pods.
    g.add(wing(HARRIER_WING_ROOT, HARRIER_CHORDS[0], HARRIER_WING_TIP, HARRIER_CHORDS[1], p, t=0.07, t_tip=0.05))
    for s in (-1, 1):
        le, z, c = harrier_wing_at(HARRIER_OUTRIGGER_X)
        x = s * HARRIER_OUTRIGGER_X
        g.add(loft([(-0.50, z - 0.06, 0.02, 0.02), (-0.90, z - 0.10, 0.12, 0.14), (-2.50, z - 0.10, 0.12, 0.13),
                    (-2.90, z - 0.08, 0.03, 0.04)], p, n=8, x=x))
        for px in (1.88, 2.60, 3.28):
            le, z, c = harrier_wing_at(px)
            g.add(pylon(s * px, le - 0.35, le - 1.85, z - 0.03, z - 0.33, w=0.06, paint=p))
        le, z, c = harrier_wing_at(3.90)
        g.add(pylon(s * 3.90, le - 0.25, le - 0.95, z - 0.02, z - 0.18, w=0.04, paint=p))
    g.add(pylon(0.0, 1.20, -0.30, -0.66, -0.90, w=0.07, paint=p))
    # The anhedral tailplane; the fin on its dorsal fillet with the ram air intake at its base;
    # the ventral fin under the tail cone.
    g.add(wing((0.20, -4.59, 0.82), 1.57, (2.10, -6.03, 0.50), 0.52, p, t=0.045, t_tip=0.035))
    g.add(wing((0, -4.46, 0.97), 1.87, (0, -6.10, 2.95), 0.49, p, t=0.05, vertical=True, mirror=False))
    g.add(wing((0, -2.90, 0.98), 1.60, (0, -4.40, 1.30), 0.25, p, t=0.06, vertical=True, mirror=False))
    g.add(frustum((0, -3.05, 1.06), (0, -2.85, 1.06), 0.06, 0.08, BLACK, 6))
    g.add(wing((0, -4.80, -0.12), 1.10, (0, -5.40, -0.42), 0.55, p, t=0.06, vertical=True, mirror=False))
    # The nozzles, turning down about their pivots (`_part1`.. `_part4`, state slot 25).
    parts = []
    for k, (pivot, r) in enumerate(HARRIER_NOZZLES):
        right = harrier_nozzle(r, hot=k == 1)
        parts.append((right, pivot, HARRIER_NOZZLE_SLOT, 1))
        parts.append((right.mirrored(), (-pivot[0], pivot[1], pivot[2]), HARRIER_NOZZLE_SLOT, 1))
    # The navigation lights on the wingtips, the white one on the boom's end.
    le, z, c = harrier_wing_at(HARRIER_WING_TIP[0])
    tip = (HARRIER_WING_TIP[0] + 0.03, le - c / 2, z)
    return Craft(g, (-tip[0], tip[1], tip[2]), tip, [(0, -7.30, 0.78)], gear=harrier_gear(), parts=parts)


def harrier_gear():
    """The bicycle gear down: the nose unit under the intakes, the twin-wheeled main unit behind
    the rear nozzles, the outriggers from their fairings; every wheel on the ground line of the
    nose-up attitude (`harrier_ground`)."""
    g = Geo()
    # The nose unit: the leg raked forward to its wheel, its door beside it.
    y, r = 2.53, 0.30
    axle = (0.0, y, harrier_ground(y) + r)
    g.add(mklib.wheel(axle, r, 0.20, BLACK, n=10))
    g.add(frustum((0.0, y - 0.25, -0.66), (0.0, y - 0.02, axle[2] + 0.08), 0.08, 0.07, GREY_L, n=5))
    g.add(frustum((0.0, y - 0.70, -0.66), (0.0, y - 0.18, (axle[2] - 0.66) / 2), 0.035, 0.035, GREY_L, n=4))
    g.face2([(0.05, y + 0.45, -0.64), (0.05, y - 0.85, -0.64), (0.05, y - 0.85, -1.02), (0.05, y + 0.45, -1.02)],
            HARRIER_GREY)
    # The main unit: two wheels side by side on one axle.
    y, r = HARRIER_MAIN[0], 0.32
    axle_z = harrier_ground(y) + r
    for s in (-1, 1):
        g.add(mklib.wheel((s * 0.17, y, axle_z), r, 0.18, BLACK, n=10))
    g.add(frustum((0.0, y + 0.15, -0.60), (0.0, y, axle_z + 0.05), 0.09, 0.08, GREY_L, n=5))
    g.add(frustum((0.0, y + 0.85, -0.60), (0.0, y + 0.10, axle_z + 0.35), 0.04, 0.04, GREY_L, n=4))
    g.add(block(-0.08, 0.08, y + 0.05, y - 0.05, axle_z - 0.03, axle_z + 0.03, GREY_L))
    # The outriggers: long thin legs down from the fairings.
    y, r = -1.46, 0.17
    _, z, _ = harrier_wing_at(HARRIER_OUTRIGGER_X)
    for s in (-1, 1):
        x = s * HARRIER_OUTRIGGER_X
        axle = (x, y, harrier_ground(y) + r)
        g.add(mklib.wheel(axle, r, 0.10, BLACK, n=8))
        g.add(frustum((x, y + 0.05, z - 0.18), (x, y, axle[2] + 0.06), 0.045, 0.04, GREY_L, n=5))
    return g


# ---------------------------------------------------------------------------------------------
# Transports, bombers, helicopter


def high_wing_transport(L, span, engines, t_tail=True, paint=GREY_L, z_wing=2.0):
    k = L / 46.6
    r = 2.0 * k * 1.15
    g = loft([(L / 2, -0.2 * k, 0, 0), (L / 2 - 1.0 * k, -0.1, r * 0.6, r * 0.6), (L / 2 - 4 * k, 0, r, r * 1.05),
              (-L * 0.2, 0, r, r * 1.05), (-L * 0.38, 0.6 * k, r * 0.6, r * 0.7), (-L / 2 + 2 * k, 1.4 * k, r * 0.25, r * 0.35)],
             paint, n=10, power=2.2)
    g.add(loft([(L / 2 - 2.0 * k, r * 0.55, 0.05, 0.05), (L / 2 - 2.8 * k, r * 0.78, r * 0.7, 0.4 * k),
                (L / 2 - 4.5 * k, r * 0.9, r * 0.75, 0.4 * k)], CANOPY, n=6))
    zw = z_wing * k * 1.1
    g.add(wing((r * 0.9, 2.0 * k, zw), 8.0 * k, (span / 2, -6.0 * k, zw - 1.0 * k), 2.6 * k, paint, t=0.1))
    for ex, ey in engines:
        for s in (-1, 1):
            x = s * ex * k
            g.add(nacelle(x, ey * k + 2.5 * k, ey * k - 3.5 * k, zw - 1.6 * k, 0.85 * k, paint))
            g.add(pylon(x, ey * k + 2.0 * k, ey * k - 1.5 * k, zw - 1.0 * k, zw, 0.15 * k, paint))
    if t_tail:
        g.add(wing((0, -L * 0.36, r * 0.9), 6.0 * k, (0, -L / 2 + 2.5 * k, r + 7.0 * k), 3.6 * k, paint, t=0.08, vertical=True,
                   mirror=False))
        g.add(wing((0.2, -L / 2 + 2.2 * k, r + 7.0 * k), 4.2 * k, (8.0 * k, -L / 2 - 1.0 * k, r + 6.6 * k), 2.0 * k, paint, t=0.06))
        tail = (0, -L / 2 + 1.0 * k, r + 7.2 * k)
    else:
        g.add(wing((0, -L * 0.36, r * 0.9), 6.0 * k, (0, -L / 2 + 2.0 * k, r + 6.0 * k), 3.0 * k, paint, t=0.08, vertical=True,
                   mirror=False))
        g.add(wing((0.5, -L * 0.4, r * 0.4), 5.0 * k, (8.0 * k, -L / 2, r * 0.6), 2.0 * k, paint, t=0.06))
        tail = (0, -L / 2, r + 6.0 * k)
    # main gear fairings
    for s in (-1, 1):
        g.add(loft([(3.0 * k, -r * 0.6, 0.0, 0.0), (1.0 * k, -r * 0.7, 0.6 * k, 0.6 * k), (-6.0 * k, -r * 0.7, 0.6 * k, 0.6 * k),
                    (-8.0 * k, -r * 0.6, 0.0, 0.0)], paint, n=6, x=s * r * 0.95))
    tip = (span / 2, -7.3 * k, zw - 1.0 * k)
    return g, tip, tail


def a50():
    g, tip, tail = high_wing_transport(46.6, 50.5, [(8.8, 0.2), (15.5, -4.3)])
    for y in (2.0, -2.0):
        g.add(pylon(0, y + 0.8, y - 0.8, 2.4, 5.0, 0.2))
    part = Geo().add(frustum((0, 0, 5.0), (0, 0, 5.8), 5.3, 5.3, Ramp(232, 238), 12, caps=True)).add(
        frustum((0, 0, 5.8), (0, 0, 6.6), 5.3, 5.3, GREY_L, 12, caps=True))
    return Craft(g, (-tip[0], tip[1], tip[2]), tip, [tail], part=part, pivot=True)


def an72():
    L, span = 28.1, 31.9
    k = 1.0
    r = 1.6
    g = loft([(L / 2, -0.2, 0, 0), (L / 2 - 0.8, -0.1, r * 0.6, r * 0.6), (L / 2 - 3, 0, r, r * 1.05), (-4.0, 0, r, r * 1.05),
              (-9.0, 0.7, r * 0.55, r * 0.6), (-L / 2 + 0.5, 1.6, r * 0.25, r * 0.3)], GREY_L, n=10, power=2.2)
    g.add(loft([(L / 2 - 1.6, r * 0.55, 0.05, 0.05), (L / 2 - 2.2, r * 0.8, r * 0.7, 0.35), (L / 2 - 3.6, r * 0.9, r * 0.75, 0.35)],
               CANOPY, n=6))
    g.add(wing((r * 0.9, 2.4, r + 0.4), 4.2, (span / 2, -0.6, r + 0.1), 1.8, GREY_L, t=0.1))
    for s in (-1, 1):
        # engines above and ahead of the wing (Coanda layout)
        g.add(nacelle(s * 3.2, 6.0, -1.5, r + 1.3, 0.85, GREY_L))
        g.add(loft([(-6.0, -r * 0.6, 0.0, 0.0), (-4.0, -r * 0.7, 0.5, 0.6), (2.0, -r * 0.7, 0.5, 0.6), (3.0, -r * 0.6, 0, 0)],
                   GREY_L, n=6, x=s * r * 0.95))
    g.add(wing((0, -7.0, r * 0.9), 5.0, (0, -11.5, 8.2), 3.0, GREY_L, t=0.08, vertical=True, mirror=False))
    g.add(wing((0.2, -11.3, 8.0), 3.4, (5.2, -13.6, 7.8), 1.6, GREY_L, t=0.06))
    return Craft(g, (-span / 2, -1.5, r + 0.1), (span / 2, -1.5, r + 0.1), [(-5.2, -14.6, 7.8), (5.2, -14.6, 7.8)])


def c5():
    g, tip, tail = high_wing_transport(75.3, 67.9, [(9.5, 1.5), (16.5, -4.0)], paint=GREY_B)
    return Craft(g, (-tip[0], tip[1], tip[2]), tip, [tail])


def tu95():
    L, span = 46.2, 50.1
    r = 1.45
    p = GREY_L
    g = loft([(L / 2, 0.0, 0, 0), (L / 2 - 1.2, 0.0, 0.8, 0.8), (L / 2 - 4, 0.1, r, r), (-L * 0.3, 0.1, r, r),
              (-L / 2 + 3.0, 0.4, r * 0.55, r * 0.55), (-L / 2, 0.6, 0.3, 0.35)], p, n=8)
    g.add(loft([(L / 2 - 2.4, r * 0.55, 0.05, 0.05), (L / 2 - 3.2, r * 0.85, r * 0.7, 0.35), (L / 2 - 5.0, r * 0.95, r * 0.7, 0.35)],
               CANOPY, n=6))
    g.add(frustum((0, L / 2 - 0.2, 0.3), (0, L / 2 + 1.4, 0.3), 0.06, 0.04, BLACK, 4))
    g.add(wing((r * 0.9, 1.5, -0.2), 7.0, (span / 2, -14.0, 1.4), 2.4, p, t=0.1))

    def le(x):
        t = (x - r * 0.9) / (span / 2 - r * 0.9)
        return 1.5 + t * (-14.0 - 1.5), -0.2 + t * 1.6

    for ex in (6.8, 13.6):
        for s in (-1, 1):
            y, z = le(ex)
            x = s * ex
            g.add(loft([(y + 1.6, z - 0.1, 0.25, 0.25), (y + 1.0, z - 0.1, 0.75, 0.75), (y - 3.0, z, 0.75, 0.8),
                        (y - 6.0, z + 0.1, 0.25, 0.3)], p, n=8, x=x))
            g.add(frustum((x, y + 1.6, z - 0.1), (x, y + 2.6, z - 0.1), 0.25, 0.02, GREY_D, 6))
            # contra-rotating propellers: two four-blade discs
            for py, rot in ((y + 1.45, 0.0), (y + 1.05, 0.785)):
                for b in range(4):
                    a = rot + b * 1.5708
                    dx, dz = math.cos(a), math.sin(a)
                    g.face2([(x + dx * 0.25 - dz * 0.12, py, z - 0.1 + dz * 0.25 + dx * 0.12),
                            (x + dx * 2.8 - dz * 0.15, py, z - 0.1 + dz * 2.8 + dx * 0.15),
                            (x + dx * 2.8 + dz * 0.15, py, z - 0.1 + dz * 2.8 - dx * 0.15),
                            (x + dx * 0.25 + dz * 0.12, py, z - 0.1 + dz * 0.25 - dx * 0.12)], GREY_D)
    g.add(wing((0, -L / 2 + 9.0, r * 0.9), 7.0, (0, -L / 2 + 1.5, r + 8.0), 3.0, p, t=0.08, vertical=True, mirror=False))
    g.add(wing((0.4, -L / 2 + 7.0, 0.6), 5.4, (7.2, -L / 2 + 0.8, 0.6), 2.2, p, t=0.07))
    return Craft(g, (-span / 2, -15.3, 1.4), (span / 2, -15.3, 1.4), [(0, -L / 2 + 0.3, r + 8.0)])


# ---------------------------------------------------------------------------------------------
# Stores and the wreck (each a `Store`: the body and an optional motor flame)


class Store:
    def __init__(self, body, flame=None):
        self.body, self.flame = body, flame


def fins(g, y0, y1, r, span, paint, n=4, rot=45.0, sweep=0.0):
    """`n` fins round a body of radius `r`, root chord from y0 back to y1, tip `span` out."""
    for k in range(n):
        a = math.radians(rot + k * 360.0 / n)
        d = Vector((math.cos(a), 0.0, math.sin(a)))
        root0, root1 = Vector((0, y0, 0)) + d * r, Vector((0, y1, 0)) + d * r
        tip0, tip1 = root0 + d * span + Vector((0, -sweep, 0)), root1 + d * span
        g.face2([root0, root1, tip1, tip0], paint)


def round_body(stations, paint, n=8):
    """A body of revolution from (y, radius) stations, nose first."""
    g = Geo()
    for (ya, ra), (yb, rb) in zip(stations, stations[1:]):
        g.add(frustum((0, ya, 0), (0, yb, 0), max(ra, 0.005), max(rb, 0.005), paint, n, caps=False))
    return g


def air_missile():
    """A slim air-to-air or anti-radar missile (AIM-9 / AIM-7 class): 3.6 m, canards and tail fins."""
    g = round_body([(1.8, 0.0), (1.55, 0.07), (1.3, 0.095), (-1.8, 0.095)], GREY_L)
    fins(g, 1.1, 0.8, 0.095, 0.22, GREY_L)
    fins(g, -1.25, -1.78, 0.095, 0.3, GREY_L)
    flame = frustum((0, -1.8, 0), (0, -2.9, 0), 0.08, 0.02, FLAME, 6)
    return Store(g, flame)


def maverick():
    """AGM-65 class: a stubby 2.5 m missile, its seeker dome, long-chord delta fins."""
    g = round_body([(1.25, 0.0), (1.15, 0.1), (1.0, 0.15), (-1.25, 0.15)], OLIVE)
    g.add(round_body([(1.27, 0.0), (1.2, 0.08)], DARK, n=6))
    fins(g, 0.2, -1.2, 0.15, 0.27, OLIVE, sweep=0.5)
    flame = frustum((0, -1.25, 0), (0, -2.3, 0), 0.12, 0.03, FLAME, 6)
    return Store(g, flame)


def bomb():
    """A 2,000 lb guided bomb: seeker, canards, body and the folded-out tail fins."""
    g = round_body([(1.95, 0.0), (1.8, 0.09), (1.5, 0.14), (1.2, 0.23), (-1.2, 0.23), (-1.7, 0.16), (-1.95, 0.1)], OLIVE)
    fins(g, 1.4, 1.1, 0.14, 0.18, OLIVE)
    fins(g, -1.35, -1.95, 0.16, 0.32, OLIVE, sweep=0.15)
    return Store(g)


def sam_missile():
    """A surface-to-air missile (SA-6 class): 5.8 m, wings mid-body, tail fins, a long flame."""
    g = round_body([(2.9, 0.0), (2.4, 0.12), (2.0, 0.17), (-2.9, 0.17)], GREY_L)
    fins(g, 0.5, -0.6, 0.17, 0.42, GREY_L, sweep=0.3)
    fins(g, -2.1, -2.85, 0.17, 0.38, GREY_L, rot=0.0)
    flame = frustum((0, -2.9, 0), (0, -5.2, 0), 0.14, 0.03, FLAME, 6)
    return Store(g, flame)


def supply_drop():
    """Equipment dropped by parachute: a round canopy in red and white gores, its lines, the crate."""
    g = Geo()
    gores, r, top, rim = 12, 4.2, 9.0, 6.6
    for k in range(gores):
        a0, a1 = 2 * math.pi * k / gores, 2 * math.pi * (k + 1) / gores
        paint = Ramp(208, 215) if k % 2 else Ramp(224, 230)
        p0 = Vector((r * math.cos(a0), r * math.sin(a0), rim))
        p1 = Vector((r * math.cos(a1), r * math.sin(a1), rim))
        m0, m1 = p0 * 0.6 + Vector((0, 0, top - rim)) * 0.75 + Vector((0, 0, rim * 0.4)), p1 * 0.6 + Vector((0, 0, top - rim)) * 0.75 + Vector((0, 0, rim * 0.4))
        g.face2([p0, p1, m1, m0], paint)
        g.face2([m0, m1, Vector((0, 0, top))], paint)
        if k % 3 == 0:
            g.line(p0, Vector((0, 0, 1.2)), 236)
    g.add(block(-0.6, 0.6, -0.6, 0.6, 0.0, 1.2, OLIVE))
    return Store(g)


def wreck():
    """A crashed aircraft: the broken fuselage, a torn wing, the tail and scattered pieces."""
    g = Geo()
    burnt = Ramp(234, 239)
    g.add(frustum((0.0, 3.0, 0.5), (0.3, -2.5, 0.6), 0.7, 0.6, burnt, 6))
    g.add(frustum((0.4, -3.0, 0.4), (1.4, -7.0, 0.9), 0.55, 0.3, burnt, 6))
    g.add(wing((0.6, 0.5, 0.15), 3.2, (5.2, -1.2, 0.1), 1.4, burnt, t=0.06, mirror=False))
    g.add(wing((-0.6, 1.2, 0.2), 2.8, (-2.6, 0.4, 0.05), 2.2, burnt, t=0.06, mirror=False))
    g.add(wing((1.4, -6.0, 1.0), 2.0, (1.6, -7.3, 3.0), 1.0, burnt, t=0.05, vertical=True, mirror=False))
    for x, y, a in ((-3.0, -4.0, 0.3), (3.5, 3.5, 1.1), (-1.5, 5.5, 0.7), (4.0, -5.0, 2.0)):
        c, sn = math.cos(a), math.sin(a)
        g.add(block(x - 0.5 * c, x + 0.5 * c, y - 0.5 * sn - 0.4, y + 0.5 * sn + 0.4, 0.0, 0.3, burnt))
    return Store(g)


# Model index -> builder: aircraft, then stores and the wreck.
BUILDERS = {
    0: mig25,
    3: lambda: f18(two_seat=True),
    5: f14,
    6: an72,
    7: mig23,
    8: f4,
    9: a50,
    13: yak38,
    14: tu95,
    16: mig29,
    18: mig17,
    19: mirage_f1,
    20: f5,
    21: f15,
    22: su27,
    23: f16,
    26: f18,
    27: a6,
    28: mig21,
    29: ef111,
    30: c5,
    32: a10,
    45: tornado,
    47: harrier,
}
STORES = {1: air_missile, 10: maverick, 11: supply_drop, 12: bomb, 15: sam_missile, 44: wreck}
# Models no part of the game draws.
UNUSED = [2, 34, 35, 36, 37, 38, 39, 40, 41, 42]
K = 16.384
MAX_DIST = 32769


def fresh_root(index):
    """The model's root with its children removed (its `attr` kept; a new model gets the stores'
    one), and a new detail level."""
    name = f"aircraft_{index:03}"
    root = bpy.data.objects.get(name)
    if root is None:
        root = bpy.data.objects.new(name, None)
        bpy.context.scene.collection.objects.link(root)
        # The stores' attributes; the aircraft's (45, the Tornado) the F-16's.
        root["attr"] = 0x83 if index in BUILDERS else 0x82
    for c in list(root.children):
        mklib.delete_tree(c)
    root["lod_dists"] = [MAX_DIST]
    return root


def centred(geo, shift):
    return geo.transformed(lambda p: (p + shift) * K)


def build_craft(index, craft):
    root = fresh_root(index)
    lo, hi = craft.body.bounds()
    shift = Vector((0, -(lo.y + hi.y) / 2, 0))
    body = centred(craft.body, shift)
    hot = Geo()
    hot.faces = [f for f in body.faces if f[1] == FLAME]
    body.faces = [f for f in body.faces if f[1] != FLAME]
    lod = heli.mesh_with_lines(root, f"{root.name}_lod0", body)
    lod["max_dist"] = MAX_DIST
    print(f"{root.name}: centred {shift.y:+.4f} m along y")
    gear = None if craft.gear is None else centred(craft.gear, shift)
    lod["radius"] = heli.radius(body, *[g for g in [gear] if g is not None])
    if hot.faces:
        mklib.link_child(lod, f"{lod.name}_exhaust", mklib.faces_mesh(f"{lod.name}_exhaust", hot))
    if craft.flames is not None:
        group = bpy.data.objects.new(f"{lod.name}_s13v1", None)
        group["conds"] = [[13, 1]]
        for col in lod.users_collection:
            col.objects.link(group)
        group.parent = lod
        mklib.link_child(group, f"{group.name}_body", mklib.faces_mesh(f"{group.name}_body", centred(craft.flames, shift)))
    if gear is not None:
        group = bpy.data.objects.new(f"{lod.name}_s5v1", None)
        group["conds"] = [[5, 1]]
        for col in lod.users_collection:
            col.objects.link(group)
        group.parent = lod
        mklib.link_child(group, f"{group.name}_body", mklib.faces_mesh(f"{group.name}_body", gear))
    if craft.part is not None:
        part = heli.mesh_with_lines(lod, f"{lod.name}_part1", craft.part.scaled(K), tuple(shift * K))
        part["state_slot"] = 17
        part["axis"] = 0
    for k, (geo, pivot, slot, *axis) in enumerate(craft.parts):
        part = heli.mesh_with_lines(lod, f"{lod.name}_part{k + 1}", geo.scaled(K), tuple((Vector(pivot) + shift) * K))
        part["state_slot"] = slot
        part["axis"] = axis[0] if axis else 0
    heli.lights(lod, 12, {c: [(p + shift) * K for p in pts] for c, pts in craft.lights.items()})
    return lod


def build_store(index, store):
    root = fresh_root(index)
    body = store.body.scaled(K)
    lod = heli.mesh_with_lines(root, f"{root.name}_lod0", body)
    lod["max_dist"] = MAX_DIST
    lod["radius"] = heli.radius(body)
    if store.flame is not None:
        group = bpy.data.objects.new(f"{lod.name}_s14v1", None)
        group["conds"] = [[14, 1]]
        for col in lod.users_collection:
            col.objects.link(group)
        group.parent = lod
        mklib.link_child(group, f"{group.name}_body", mklib.faces_mesh(f"{group.name}_body", store.flame.scaled(K)))
    return lod


def build():
    for index in UNUSED:
        name = f"aircraft_{index:03}"
        if name in bpy.data.objects:
            mklib.delete_tree(bpy.data.objects[name])
    for index, make in sorted(BUILDERS.items()):
        lod = build_craft(index, make())
        print(f"{lod.name}: radius {lod['radius']}")
    for index, make in sorted(STORES.items()):
        lod = build_store(index, make())
        print(f"{lod.name}: radius {lod['radius']}")


if __name__ == "__main__":
    mklib.load_palette(os.path.join(ROOT, "assets", "palettes", "tod_central_europe.ron"))
    build()
    bpy.ops.wm.save_mainfile()
