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
afterwards. The F-117A (24, f117.py), the UH-60 (25, uh60.py) and the AH-1Z (43, ah1z.py) have
generators of their own. The flyable F-16 (23) sets the eye, lift and exhaust in
src/aircraft.rs: change them with its builder; the gear's depth below the origin is in
`gear_depth` there too.
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
CANOPY = Ramp(144, 145)
DARK = Ramp(235, 239)
RADOME = Ramp(230, 236)
OLIVE = Ramp(165, 173)
FLAME = 13
BLACK = 239


class Craft:
    def __init__(self, body, red, green, white=(), flames=None, part=None, pivot=False, gear=None):
        self.body, self.flames, self.part, self.gear = body, flames, part, gear
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
    g = tube(rings, GREY_B, paints=[RADOME, RADOME] + [GREY_B] * (len(rings) - 3))
    g.add(tube([f16_arch(*s) for s in F16_CANOPY], CANOPY))
    g.add(tube([f16_arch(*s) for s in F16_FAIRING], GREY_B))
    g.add(tube([f16_section(y, top, top - 0.3, hw, top - 0.42, hw, 3.0, 3.0, m=2) for y, top, hw in F16_DORSAL], GREY_B))
    # The chin intake: its mouth raked back from the upper lip, its trunk running into the
    # fuselage's underside.
    def mouth(y, zc, hw, hh, rake=0.0):
        return [(x, y + rake * (z - zc) / hh, z) for x, _, z in ring((0, 0, zc), hw, hh, 12, 2.4)]
    g.add(tube([mouth(3.0, -0.64, 0.60, 0.29, 0.1), mouth(2.5, -0.64, 0.62, 0.30), mouth(1.4, -0.62, 0.64, 0.28)],
               GREY_B))
    g.face2(mouth(2.96, -0.64, 0.53, 0.24, 0.1), BLACK)
    # The strakes' last stretch to the wing's leading edge, the wing (straight trailing edge,
    # tip rails), the stabilizers on their booms (10 degrees anhedral).
    g.add(wing((1.0, 2.0, 0.0), 1.3, (1.36, 0.42, 0.0), 0.05, GREY_B, t=0.03))
    g.add(wing((1.0, 0.69, 0.0), 4.10, (4.60, -2.19, 0.0), 1.22, GREY_B, t=0.04))
    g.add(wing((0.6, -3.3, 0.0), 3.5, (1.05, -3.5, 0.0), 3.35, GREY_B, t=0.06, t_tip=0.04))
    g.add(wing((1.0, -4.70, 0.02), 2.39, (2.83, -6.19, -0.30), 0.90, GREY_B, t=0.04))
    for s in (-1, 1):
        g.add(block(s * 4.58, s * 4.72, -1.7, -3.6, -0.05, 0.05, GREY_B))
        g.add(missile(s * 4.66, -0.9, -0.14, 3.0))
    # The fin on its base, the ventral fins canted out.
    g.add(wing((0, -3.87, 1.1), 2.58, (0, -6.77, 3.27), 0.71, GREY_B, t=0.05, vertical=True, mirror=False))
    g.add(wing((0.5, -2.96, -0.72), 1.31, (0.67, -3.34, -1.33), 0.3, GREY_B, t=0.05, vertical=True))
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
    g.face2([(-0.3, 2.5, keel + 0.02), (-0.3, 1.3, keel + 0.02), (-0.34, 1.3, keel - 0.36), (-0.34, 2.5, keel - 0.36)], GREY_B)
    r, w = 0.34, 0.22
    for s in (-1, 1):
        axle = (s * 1.18, -1.76, F16_GROUND + r)
        g.add(mklib.wheel(axle, r, w, BLACK, n=8))
        g.add(frustum((s * 0.62, -1.6, -0.74), (s * 1.05, -1.76, axle[2] + 0.12), 0.08, 0.07, GREY_L, n=5))
        g.add(frustum((s * 0.62, -0.9, -0.76), (s * 1.0, -1.66, axle[2] + 0.4), 0.04, 0.04, GREY_L, n=4))
        g.face2([(s * 0.86, -0.9, -0.1), (s * 0.86, -2.7, -0.1), (s * 0.9, -2.7, -0.72), (s * 0.9, -0.9, -0.72)], GREY_B)
    return g


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


def a10():
    p = GREY_D
    g = loft([(8.0, -0.2, 0, 0), (7.6, -0.2, 0.35, 0.35), (6.4, 0.0, 0.7, 0.7), (4.0, 0.15, 0.9, 0.95), (0.0, 0.15, 0.9, 0.9),
              (-4.5, 0.4, 0.55, 0.55), (-7.8, 0.6, 0.3, 0.35)], p, n=8, power=2.3)
    g.add(frustum((0, 8.0, -0.35), (0, 8.6, -0.35), 0.12, 0.12, BLACK, 6))
    g.add(canopy(6.2, 4.0, 0.8, 0.42, 0.55))
    g.add(wing((0.8, 1.4, -0.4), 3.2, (8.8, 0.5, -0.1), 1.8, p, t=0.12, peak=0.3))
    for s in (-1, 1):
        g.add(nacelle(s * 1.55, -2.0, -5.6, 1.35, 0.62, p))
        g.add(pylon(s * 1.0, -2.6, -4.4, 0.8, 1.0, 0.2, p))
        g.add(wing((s * 3.2, -6.4, 0.9), 2.4, (s * 3.25, -7.0, 3.2), 1.6, p, t=0.06, vertical=True, mirror=False))
        g.add(wing((s * 3.2, -6.6, 0.9), 1.6, (s * 3.25, -6.9, -0.4), 1.2, p, t=0.06, vertical=True, mirror=False))
    g.add(wing((0.3, -6.0, 0.8), 2.2, (3.3, -6.4, 0.9), 1.6, p, t=0.06))
    return Craft(g, (-8.8, -0.4, -0.1), (8.8, -0.4, -0.1), [(-3.25, -8.4, 3.2), (3.25, -8.4, 3.2)])


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
        root["attr"] = 0x82
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
