"""Builds the red side's SAM sites into the theater masters, at real size, after the systems the
theater's sites stand for, and saves the masters:

    blender -b --python-exit-code 1 --python tools/models/sam_sites.py [-- <theater> ...]

then export each master to a temporary file and splice the changed models into the shipped
theater files (tools/models/splice_model.py), never exporting a theater whole.

Each theater's library has three kinds of SAM model (tools/theater/libraries/<id>.json `kinds`):
`sa_missile` (the six launchers round a fixed site), `sam_radar` (the radar in the middle of a
fixed or mobile battery) and `mobile_sam` (a mobile battery's launchers, also guarding depots,
headquarters and armour). Per family of theaters:

- The S-75 (SA-2) theaters: the SM-90 launcher with its V-750 missile in a horseshoe revetment;
  the SNR-75 "Fan Song" antenna cabin on its trailer (turning, slot 16) with the P-12 "Spoon
  Rest" search radar (turning, slot 15), the UV control cabin and the diesel generator. Libya's
  mobile launcher is the 9K33 Osa (SA-8) on its BAZ-5937 6x6, search antenna turning.
- The S-300 (SA-10) theaters (Central Europe, whose models Ukraine, Suwalki and the Gulf of
  Finland reuse, and the North Cape): the 5P85 TEL on its MAZ-543 with the four canisters
  erected (a contributed model, assets-src/blend/s300_tel.blend); the 64N6 "Big Bird" on its semitrailer (turning) and the 5N63 "Flap Lid" fire control
  radar on its MAZ-543. Central Europe's mobile launcher is the 9K330 Tor (SA-15), search
  antenna turning.
- The Persian Gulf (Iranian Hawks): the M192 triple launcher with three MIM-23s; the AN/MPQ-50
  pulse acquisition radar (turning), the AN/MPQ-46 illuminator and the battery control van.

Each root keeps its name, `attr` and `lod_dists`. The finest level holds the ground (a
revetment, always drawn), `_s1v0` the intact unit with its turning antennas (`_part1` slot 16,
`_part2` slot 15, yaw), `_s1v1` the wreck, `_s10v1` the radars' lights at night; the next level
is a copy of it (tools/models/lower_lods.py), the last one the theater's old dot. The collision
hull is the convex hull of the main vehicle.

Model space: x east, y north (the vehicles' noses), z up, in world units (the models are LOD1
tile objects); built in metres.
"""

import json
import math
import os
import random
import sys

import bpy

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.abspath(os.path.join(HERE, "..", ".."))
sys.path.insert(0, HERE)

import air_defence as ad  # noqa: E402
import heli  # noqa: E402
import lower_lods  # noqa: E402
import mklib  # noqa: E402
import sites  # noqa: E402
from mklib import Geo, Ramp, block, frustum, hexa, v3, wheel  # noqa: E402

K = sites.K  # world units per metre

TYRE = Ramp(0xEC, 0xEF)
STEEL = Ramp(0xE9, 0xEE)
DARK = Ramp(0xEB, 0xEF)
BURNT = Ramp(0xEC, 0xEF)
WHITE = Ramp(0xE0, 0xE4)
MISSILE = Ramp(0xE1, 0xE6)
APERTURE = Ramp(0xE7, 0xEB)
GRAVEL = 0xE6
GLASS = 0x9A
SCORCH = 0xEF
FIRE = 0x0D
CABLE = 0xEE
RED_LIGHT = 0x92
WHITE_LIGHT = 0x94


class Paints:
    def __init__(self, camo, camo_dark, earth):
        self.camo, self.camo_dark, self.earth = camo, camo_dark, earth


FOREST = Paints(Ramp(0xA6, 0xAF), Ramp(0xAA, 0xAF), Ramp(0xA4, 0xA9))
DESERT = Paints(Ramp(0xC1, 0xC5), Ramp(0xC3, 0xC5), Ramp(0xC1, 0xC4))


# ------------------------------------------------------------------------------------- helpers

def moved(g, d):
    return ad.moved(g, d)


def turned(g, heading, at=(0, 0, 0)):
    """`g` turned by `heading` (degrees clockwise from north) and moved to `at`."""
    return moved(g.rotated_z(-math.radians(heading)), at)


def turned_point(p, heading, at=(0, 0, 0)):
    """A point of a model placed with `turned`."""
    a = -math.radians(heading)
    c, s = math.cos(a), math.sin(a)
    return v3(at) + v3((p[0] * c - p[1] * s, p[0] * s + p[1] * c, p[2]))


def beam(a, b, w, paint, n=4):
    """A square-ish strut of width `w` between two points."""
    return frustum(a, b, w / 2, w / 2, paint, n=n, rot=math.pi / n)


def panel(w, h, t, paint, face, lines=None, nx=0, nz=0):
    """A flat antenna array (w across x, h up z, t thick in y) facing +y, centred on the origin;
    `lines` draws its element grid on the face."""
    g = block(-w / 2, w / 2, -t / 2, t / 2, -h / 2, h / 2, paint)
    e = min(w, h) * 0.04
    y = t / 2 + 0.01
    g.face([(-w / 2 + e, y, -h / 2 + e), (w / 2 - e, y, -h / 2 + e), (w / 2 - e, y, h / 2 - e), (-w / 2 + e, y, h / 2 - e)],
           face, (0, -1, 0))
    if lines is not None:
        g.add(mklib.lattice(-w / 2 + e, w / 2 - e, -h / 2 + e, h / 2 - e, y + 0.01, nx, nz, lines))
    return g


def trough(w, h, depth, paint, face, n=6):
    """A horizontal trough antenna (Fan Song): `w` across x, the aperture `h` tall at y = 0
    facing +y, the reflector curving back to y = -depth."""
    prof = []
    for k in range(n + 1):
        a = math.pi / 2 - math.pi * k / n
        prof.append((-depth * math.cos(a), h / 2 * math.sin(a)))
    g = mklib.side_prism(prof, -w / 2, w / 2, paint)
    g.face([(-w / 2 + 0.05, 0.02, -h / 2 + 0.05), (w / 2 - 0.05, 0.02, -h / 2 + 0.05),
            (w / 2 - 0.05, 0.02, h / 2 - 0.05), (-w / 2 + 0.05, 0.02, h / 2 - 0.05)], face, (0, -1, 0))
    return g


def dish(r, depth, paint, face, n=10, feed=None):
    """A dish antenna facing +y, its rim at y = 0."""
    g = frustum((0, -depth, 0), (0, 0, 0), r * 0.3, r, paint, n=n, caps=False)
    g.face(mklib.ring((0, -depth - 0.01, 0), r * 0.3, r * 0.3, n), paint, (0, 1, 0))
    g.face(mklib.ring((0, -0.02, 0), r * 0.97, r * 0.97, n), face, (0, -1, 0))
    if feed is not None:
        tip = (0, r * 0.7, 0)
        for k in range(3):
            a = 2 * math.pi * k / 3 + math.pi / 2
            g.line((r * 0.95 * math.cos(a), 0, r * 0.95 * math.sin(a)), tip, feed)
        g.add(frustum((0, r * 0.62, 0), (0, r * 0.75, 0), 0.06, 0.06, DARK, n=4))
    return g


def fin(r0, r1, le0, c0, le1, c1, roll, paint):
    """A thin fin from radius r0 (root: leading edge at y = le0, chord c0) to r1 (tip), turned
    `roll` degrees round the y axis from +x."""
    a = math.radians(roll)
    ca, sa = math.cos(a), math.sin(a)

    def p(r, y):
        return (r * ca, y, r * sa)

    g = Geo()
    g.face2([p(r0, le0), p(r1, le1), p(r1, le1 - c1), p(r0, le0 - c0)], paint)
    return g


def cross_fins(r0, r1, le0, c0, le1, c1, paint, roll=45):
    g = Geo()
    for k in range(4):
        g.add(fin(r0, r1, le0, c0, le1, c1, roll + 90 * k, paint))
    return g


def axles(ys, half_track, r, w, paint=TYRE):
    g = Geo()
    for y in ys:
        for sx in (-1, 1):
            g.add(wheel((sx * half_track, y, r), r, w, paint, n=8))
    return g


def jack(x, y, z1, paint):
    """An outrigger jack from height z1 down to its pad on the ground."""
    g = frustum((x, y, 0.12), (x, y, z1), 0.08, 0.08, paint, n=4)
    g.add(block(x - 0.25, x + 0.25, y - 0.25, y + 0.25, 0.0, 0.12, STEEL))
    return g


def cab_glass(x0, x1, y, z0, z1, inset=0.1):
    """A windscreen on a cab front face at y."""
    g = Geo()
    g.face([(x0 + inset, y + 0.02, z0), (x1 - inset, y + 0.02, z0), (x1 - inset, y + 0.02, z1), (x0 + inset, y + 0.02, z1)],
           GLASS, (0, y - 1, (z0 + z1) / 2))
    return g


def van(length, paint, wheels=True, body_h=2.4, w=2.5):
    """A box shelter on a two-axle trailer (control cabins, generators), nose to +y."""
    g = Geo()
    if wheels:
        g.add(axles((length / 2 - 1.2, -length / 2 + 1.2), w / 2 - 0.3, 0.5, 0.35))
    g.add(block(-w / 2 + 0.2, w / 2 - 0.2, -length / 2, length / 2, 0.55, 0.85, STEEL))
    g.add(block(-w / 2, w / 2, -length / 2, length / 2, 0.85, 0.85 + body_h, paint, taper=0.03, taper_y=0.03))
    g.add(beam((0, length / 2, 0.7), (0, length / 2 + 1.6, 0.55), 0.12, STEEL))
    # Door and its steps on the right side, a vent on the roof.
    x = w / 2 + 0.01
    g.face([(x, -0.6, 0.95), (x, 0.4, 0.95), (x, 0.4, 2.85), (x, -0.6, 2.85)], DARK, (0, 0, 1.5))
    g.add(block(w / 2, w / 2 + 0.5, -0.6, 0.4, 0.3, 0.45, STEEL))
    g.add(block(-0.4, 0.4, -length / 2 + 0.8, -length / 2 + 1.6, 0.85 + body_h, 1.15 + body_h, DARK))
    return g


def truck_cab(paint, x0=-1.25, x1=1.25, y0=0.0, y1=2.2, z0=0.9, z1=2.9, glass=True):
    """A cab-over truck cab, nose at y1."""
    g = block(x0, x1, y0, y1, z0, z1, paint, taper=0.06, taper_y=0.12)
    if glass:
        g.add(cab_glass(x0 + 0.1, x1 - 0.1, y1 - 0.12, z0 + (z1 - z0) * 0.5, z1 - 0.15))
    return g


def scorch(r, rng, z=0.03, n=11):
    g = sites.blob(0, 0, r, z, SCORCH, rng, n=n)
    for k in range(3):
        a = rng.uniform(0, 2 * math.pi)
        d = rng.uniform(0, r * 0.4)
        g.add(sites.blob(d * math.cos(a), d * math.sin(a), r * 0.2, z + 0.02, FIRE, rng, n=6))
    return g


def wreck_of(g, tilt=4.0, roll=3.0, sink=0.35):
    """A burnt-out hulk of `g`: blackened, sagging and settled into the ground."""
    g = g.recolour(BURNT)
    g = g.rotated_x(math.radians(tilt))
    g = ad.rotated_y(g, math.radians(roll))
    return moved(g, (0, 0, -sink))


def lying(g, heading, at, tip=-85.0):
    """A broken-off part `g` (built standing, facing +y) toppled onto its face, burnt."""
    g = g.recolour(BURNT).rotated_x(math.radians(tip))
    lo, _ = g.bounds()
    return turned(moved(g, (0, 0, -lo.z)), heading, at)


# ----------------------------------------------------------------------- S-75 (SA-2 Guideline)

def v750(paint=MISSILE):
    """The V-750 missile, 10.6 m long, tail at y = 0: the booster (0.65 m) with its four big
    fins, the sustainer (0.5 m) with mid-body wings, tail controls and nose canards."""
    g = frustum((0, 0, 0), (0, 3.2, 0), 0.327, 0.327, paint, n=8)
    g.add(frustum((0, 3.2, 0), (0, 3.55, 0), 0.327, 0.25, paint, n=8, caps=False))
    g.add(frustum((0, 3.55, 0), (0, 9.0, 0), 0.25, 0.25, paint, n=8, caps=False))
    g.add(frustum((0, 9.0, 0), (0, 10.6, 0), 0.25, 0.01, paint, n=8, caps=False))
    g.add(frustum((0, -0.15, 0), (0, 0.0, 0), 0.22, 0.3, DARK, n=8))
    g.add(cross_fins(0.32, 1.25, 1.75, 1.75, 0.95, 0.95, paint))
    g.add(cross_fins(0.24, 0.85, 6.45, 1.9, 5.75, 0.6, paint))
    g.add(cross_fins(0.24, 0.6, 4.25, 0.7, 3.95, 0.35, paint))
    g.add(cross_fins(0.24, 0.43, 8.95, 0.4, 8.8, 0.18, paint))
    return g


SM90_PIVOT = (0.0, -0.2, 2.35)
SM90_ELEVATION = 30.0


def sm90_rail(p, burnt=False):
    """The launch beam with its missile, built level about the trunnions (the origin)."""
    paint = BURNT if burnt else p.camo
    g = block(-0.2, 0.2, -2.2, 7.4, -0.25, 0.12, paint)
    g.add(block(-0.32, 0.32, -2.4, -1.4, -0.45, 0.2, paint))
    for y in (-0.6, 2.5, 5.6):
        g.add(block(-0.3, 0.3, y - 0.15, y + 0.15, -0.3, 0.2, paint))
    if not burnt:
        g.add(moved(v750(), (0, -2.0, 0.95)))
        g.add(block(-0.08, 0.08, -1.8, 6.6, 0.12, 0.62, DARK))
    return g


def sm90(p, burnt=False):
    """The SM-90 launcher: a turntable on a cruciform base with jacks, the cheeks holding the
    beam's trunnions, the beam raised 30 degrees towards +y."""
    paint = BURNT if burnt else p.camo
    dark = BURNT if burnt else p.camo_dark
    g = Geo()
    for a in (45, 135, 225, 315):
        leg = block(-0.16, 0.16, 0.6, 3.3, 0.25, 0.6, dark)
        leg.add(block(-0.35, 0.35, 3.0, 3.7, 0.0, 0.25, STEEL if not burnt else BURNT))
        g.add(leg.rotated_z(math.radians(a)))
    g.add(frustum((0, 0, 0.2), (0, 0, 1.2), 0.95, 0.8, paint, n=10))
    g.add(frustum((0, 0, 1.2), (0, 0, 1.5), 1.45, 1.45, paint, n=12))
    for sx in (-1, 1):
        x0, x1 = sorted((sx * 0.45, sx * 0.75))
        g.add(block(x0, x1, -1.2, 0.9, 1.5, 2.6, paint, taper_y=0.35))
    # The operator's seat box and the elevating ram from the turntable's front to the beam.
    g.add(block(0.9, 1.5, -1.0, 0.0, 1.5, 2.3, dark))
    rail = sm90_rail(p, burnt)
    el = math.radians(SM90_ELEVATION if not burnt else 6.0)
    pivot = v3(SM90_PIVOT)
    g.add(moved(rail.rotated_x(el), pivot))
    ram_top = pivot + v3((0, 2.4 * math.cos(el) + 0.25 * math.sin(el), 2.4 * math.sin(el) - 0.25 * math.cos(el)))
    g.add(beam((0, 1.1, 1.5), ram_top, 0.22, STEEL if not burnt else BURNT))
    return g


def berm(r0, r1, h, top, gap, paint, n=22, z0=0.0):
    """A horseshoe earth wall from radius r0 to r1, `h` high with a flat crest `top` wide, open
    over `gap` degrees round -y."""
    half = math.radians(gap) / 2
    start = -math.pi / 2 + half
    span = 2 * math.pi - 2 * half
    mid = (r0 + r1) / 2
    prof = [(r0, z0), (mid - top / 2, h), (mid + top / 2, h), (r1, z0)]
    rings = []
    for k in range(n + 1):
        a = start + span * k / n
        ca, sa = math.cos(a), math.sin(a)
        rings.append([(r * ca, r * sa, z) for r, z in prof])
    return mklib.tube(rings, paint)


def sa2_ground(p):
    rng = random.Random(75)
    g = berm(7.2, 12.0, 1.8, 1.2, 64, p.earth)
    g.add(sites.blob(0, 0, 7.0, 0.02, GRAVEL, rng, n=12))
    # The loading road in through the gap.
    g.add(sites.flat(-2.0, 2.0, -18.0, -6.0, 0.02, GRAVEL))
    return g


def sa2_cables():
    g = Geo()
    g.line((0.6, -0.8, 0.05), (1.5, -6.0, 0.05), CABLE)
    g.line((1.5, -6.0, 0.05), (2.4, -18.0, 0.05), CABLE)
    return g


def sa2_launcher(p):
    rng = random.Random(33)
    intact = sm90(p)
    intact.add(block(-2.6, -1.6, -3.4, -2.6, 0.0, 0.9, p.camo_dark))
    wreck = sm90(p, burnt=True)
    wreck = moved(wreck.rotated_z(math.radians(8)), (0, 0, -0.1))
    wreck.add(scorch(5.5, rng))
    # The missile broke in two: the booster nose-down by the wall, the sustainer across the pit.
    booster = frustum((0, 0, 0), (0, 3.2, 0), 0.327, 0.327, BURNT, n=8)
    booster.add(cross_fins(0.32, 1.25, 1.75, 1.75, 0.95, 0.95, BURNT))
    wreck.add(turned(booster.rotated_x(math.radians(-12)), 120, (-3.8, 2.5, 1.1)))
    sust = frustum((0, 3.55, 0), (0, 9.0, 0), 0.25, 0.25, BURNT, n=8)
    sust.add(frustum((0, 9.0, 0), (0, 10.6, 0), 0.25, 0.01, BURNT, n=8, caps=False))
    sust.add(cross_fins(0.24, 0.85, 6.45, 1.9, 5.75, 0.6, BURNT))
    wreck.add(turned(sust, 70, (-6.0, 2.0, 0.6)))
    return {"ground": sa2_ground(p), "lines": sa2_cables(), "intact": intact, "destroyed": wreck,
            "hull": sm90(p), "lights": {}}


def fan_song_antennas(p):
    """The SNR-75 antenna cabin and its arrays, turning round its middle (the origin at the
    trailer's deck): the horizontal trough on the roof, the vertical trough beside the cabin's
    front, the two LORO dishes on top and the command uplink antenna below."""
    g = block(-1.45, 1.45, -2.2, 2.0, 0.0, 2.4, p.camo, taper=0.04, taper_y=0.04)
    g.add(block(-1.6, 1.6, -2.4, 2.2, -0.15, 0.0, STEEL))
    # Supports for the arrays.
    for x in (-3.2, -0.6):
        g.add(beam((x * 0.5, 1.4, 2.4), (x, 2.5, 2.9), 0.16, STEEL))
    a = trough(6.2, 1.5, 0.9, p.camo, APERTURE)
    g.add(moved(a.rotated_x(math.radians(-6)), (-1.7, 2.85, 3.55)))
    b = trough(5.6, 1.4, 0.85, p.camo, APERTURE)
    b = ad.rotated_y(b, math.radians(90))
    g.add(moved(b, (2.25, 2.7, 2.6)))
    g.add(beam((1.5, 1.7, 0.4), (2.25, 2.0, 0.4), 0.18, STEEL))
    g.add(beam((1.5, 1.7, 2.2), (2.25, 2.0, 2.2), 0.18, STEEL))
    for x in (-3.9, -0.4):
        d = dish(0.75, 0.35, p.camo, APERTURE)
        g.add(moved(d.rotated_x(math.radians(5)), (x, 2.6, 4.95)))
        g.add(beam((x, 2.4, 4.3), (x, 2.4, 4.7), 0.14, STEEL))
    g.add(frustum((-1.3, 2.45, 1.5), (1.2, 2.45, 1.5), 0.32, 0.32, p.camo_dark, n=8))
    g.add(beam((-0.9, 2.0, 1.5), (-0.9, 2.45, 1.5), 0.14, STEEL))
    g.add(beam((0.9, 2.0, 1.5), (0.9, 2.45, 1.5), 0.14, STEEL))
    return g


FAN_SONG_PIVOT = (0.0, 0.0, 1.05)


def fan_song_trailer(p):
    g = axles((2.2, -2.2), 1.15, 0.5, 0.4)
    g.add(block(-1.25, 1.25, -3.3, 3.1, 0.55, 0.9, STEEL))
    g.add(frustum((0, 0, 0.9), (0, 0, 1.05), 0.9, 0.9, p.camo_dark, n=10))
    g.add(beam((0, 3.1, 0.75), (0, 5.0, 0.55), 0.14, STEEL))
    for x, y in ((-1.4, 2.9), (1.4, 2.9), (-1.4, -3.1), (1.4, -3.1)):
        g.add(jack(x, y, 0.8, STEEL))
    return g


def spoon_rest_array(p):
    """The P-12's antenna, turning on its mast (origin at the mast's top): a frame of two rows
    of six yagis pointing +y, drawn as lines."""
    g = Geo()
    w, z0, z1 = 9.6, 0.4, 3.6
    g.add(block(-w / 2, w / 2, -0.1, 0.1, z0, z0 + 0.15, STEEL))
    g.add(block(-w / 2, w / 2, -0.1, 0.1, z1 - 0.15, z1, STEEL))
    g.add(block(-0.12, 0.12, -0.12, 0.12, -0.3, z1, STEEL))
    for x in (-w / 2, w / 2):
        g.add(block(x - 0.06, x + 0.06, -0.06, 0.06, z0, z1, STEEL))
    g.add(beam((0, 0, 0.1), (-w / 2, 0, z0 + 0.1), 0.1, STEEL))
    g.add(beam((0, 0, 0.1), (w / 2, 0, z0 + 0.1), 0.1, STEEL))
    for row in (z0 + 0.8, z1 - 0.8):
        for k in range(6):
            x = -w / 2 + w * (k + 0.5) / 6
            g.line((x, -0.3, row), (x, 2.2, row), DARK.hi)
            for j in range(7):
                y = -0.3 + 2.5 * j / 6
                half = 0.75 - 0.05 * j
                g.line((x - half, y, row), (x + half, y, row), DARK.hi)
    return g


def spoon_rest_truck(p):
    """The P-12's van on a ZIL-157-like truck with the antenna mast at its rear."""
    g = axles((2.0, -1.0, -2.3), 1.0, 0.55, 0.35)
    g.add(block(-0.9, 0.9, -3.1, 3.2, 0.6, 0.95, STEEL))
    g.add(block(-0.75, 0.75, 2.4, 3.8, 0.9, 1.6, p.camo, taper=0.12, taper_y=0.2))
    g.add(truck_cab(p.camo, -1.15, 1.15, 0.9, 2.4, 0.95, 2.55))
    g.add(block(-1.2, 1.2, -3.2, 0.7, 0.95, 3.1, p.camo, taper=0.03, taper_y=0.03))
    g.add(frustum((0, SPOON_REST_MAST, 0.0), (0, SPOON_REST_MAST, 6.0), 0.2, 0.14, STEEL, n=6))
    for x, y in ((-1.8, -5.6), (1.8, -5.6), (0.0, -2.0)):
        g.line((0, SPOON_REST_MAST, 5.0), (x, y, 0.05), CABLE)
    return g


SPOON_REST_AT = (17.0, 11.0)
# The antenna mast behind the van (metres along the truck).
SPOON_REST_MAST = -3.42
SPOON_REST_HEADING = -30.0


def fan_song_site(p):
    rng = random.Random(38)
    intact = fan_song_trailer(p)
    intact.add(turned(van(8.0, p.camo, body_h=2.5), 75, (-13.0, -7.5, 0.0)))
    intact.add(turned(van(5.5, p.camo_dark, body_h=2.1), -100, (9.0, -12.0, 0.0)))
    intact.add(turned(spoon_rest_truck(p), SPOON_REST_HEADING, (*SPOON_REST_AT, 0.0)))
    lines = Geo()
    for a, b in (((-1.2, -3.3), (-10.5, -6.5)), ((1.2, -3.3), (7.0, -11.0)), ((-11.0, -5.0), (14.0, 8.0))):
        lines.line((*a, 0.05), (*b, 0.05), CABLE)
    sr = turned_point((0.0, SPOON_REST_MAST, 6.0), SPOON_REST_HEADING, (*SPOON_REST_AT, 0.0))
    antennas = fan_song_antennas(p)
    wreck = wreck_of(fan_song_trailer(p))
    wreck.add(lying(antennas, 40, (-1.5, 4.5, 0.0)))
    wreck.add(turned(wreck_of(van(8.0, p.camo, body_h=2.5)), 75, (-13.0, -7.5, 0.0)))
    wreck.add(turned(van(5.5, p.camo_dark, body_h=2.1), -100, (9.0, -12.0, 0.0)))
    wreck.add(turned(wreck_of(spoon_rest_truck(p)), SPOON_REST_HEADING, (*SPOON_REST_AT, 0.0)))
    wreck.add(lying(spoon_rest_array(p), 100, (SPOON_REST_AT[0] - 6.0, SPOON_REST_AT[1] + 2.0, 0.0)))
    wreck.add(scorch(7.0, rng))
    return {
        "ground": Geo(), "lines": lines, "intact": intact, "destroyed": wreck,
        "parts": [(antennas, FAN_SONG_PIVOT, 16), (spoon_rest_array(p), tuple(sr), 15)],
        "hull": fan_song_trailer(p).add(moved(antennas, FAN_SONG_PIVOT)),
        "lights": {RED_LIGHT: [(0.0, 0.0, 6.2), tuple(sr + v3((0, 0, 3.8)))], WHITE_LIGHT: [(-11.2, -7.0, 3.3)]},
    }


# -------------------------------------------------------------------------- 9K33 Osa (SA-8)

def osa_hull(paint, dark):
    """The BAZ-5937's boat hull, 9.1 m long, 2.8 m wide, nose to +y."""
    prof = [(-4.55, 0.65), (-4.55, 1.95), (3.2, 2.05), (4.55, 1.35), (4.2, 0.75), (3.6, 0.5), (-3.9, 0.5)]
    g = mklib.side_prism(prof, -1.4, 1.4, paint)
    for y in (2.9, 0.0, -2.9):
        for sx in (-1, 1):
            g.add(wheel((sx * 1.2, y, 0.62), 0.62, 0.5, TYRE, n=8))
    # Windows of the crew compartment on the sloping nose, the water jet covers at the back.
    g.face([(-1.0, 3.62, 1.95), (1.0, 3.62, 1.95), (1.0, 4.2, 1.55), (-1.0, 4.2, 1.55)], GLASS, (0, 3.0, 1.0))
    for sx in (-1, 1):
        x0, x1 = sorted((sx * 0.3, sx * 1.0))
        g.add(block(x0, x1, -4.65, -4.55, 0.8, 1.4, dark))
    return g


def osa_turret(paint, dark, burnt=False):
    """The turret (origin at its ring on the deck): the tracking radar's dish and the two
    guidance dishes in front, three missile containers either side, raised a little."""
    g = frustum((0, 0, 0), (0, 0, 0.55), 1.15, 1.05, paint, n=10)
    g.add(block(-0.85, 0.85, -1.5, 1.0, 0.55, 1.55, paint, taper=0.08, taper_y=0.1))
    g.add(block(-0.45, 0.45, -1.3, -0.4, 1.55, 1.95, dark))
    if not burnt:
        g.add(moved(dish(0.62, 0.35, paint, APERTURE, feed=DARK.hi), (0, 1.5, 1.15)))
        for sx in (-1, 1):
            g.add(moved(dish(0.22, 0.15, paint, APERTURE), (sx * 0.55, 1.15, 1.75)))
    for sx in (-1, 1):
        pod = Geo()
        for k in range(3):
            x = sx * (1.15 + 0.42 * k)
            pod.add(block(x - 0.19, x + 0.19, -1.7, 1.6, -0.19, 0.19, paint if not burnt else BURNT))
            pod.add(block(x - 0.16, x + 0.16, 1.6, 1.64, -0.16, 0.16, DARK))
        pod = pod.rotated_x(math.radians(12))
        g.add(moved(pod, (0, -0.2, 1.05)))
        g.add(beam((sx * 0.85, -0.2, 1.05), (sx * 1.15, -0.2, 1.05), 0.25, dark))
    return g


def osa_search(paint):
    """The surveillance radar on its mast (origin at the mast's foot on the turret roof), a wide
    curved reflector with its feed, facing +y."""
    g = frustum((0, 0, 0), (0, 0, 0.5), 0.18, 0.14, STEEL, n=6)
    refl = Geo()
    n = 6
    w, h = 2.6, 0.9
    for k in range(n):
        x0, x1 = -w / 2 + w * k / n, -w / 2 + w * (k + 1) / n
        y0, y1 = 0.28 * (x0 / (w / 2)) ** 2, 0.28 * (x1 / (w / 2)) ** 2
        refl.face2([(x0, -y0, 0), (x1, -y1, 0), (x1, -y1, h), (x0, -y0, h)], paint)
    refl = refl.rotated_x(math.radians(-15))
    g.add(moved(refl, (0, 0.1, 0.45)))
    g.add(beam((0, 0.0, 0.55), (0, 0.75, 0.9), 0.06, STEEL))
    g.add(block(-0.12, 0.12, 0.7, 0.85, 0.82, 0.98, DARK))
    return g


OSA_TURRET = (0.0, -0.4, 2.05)
OSA_SEARCH = (0.0, -1.3, 1.95)


def osa_unit(p):
    rng = random.Random(76)
    hull = osa_hull(p.camo, p.camo_dark)
    turret = osa_turret(p.camo, p.camo_dark)
    intact = hull.add(moved(turret, OSA_TURRET))
    search_at = v3(OSA_TURRET) + v3(OSA_SEARCH)
    wreck = wreck_of(osa_hull(p.camo, p.camo_dark), tilt=-3.0, roll=4.0, sink=0.45)
    wreck.add(moved(osa_turret(p.camo, p.camo_dark, burnt=True).recolour(BURNT).rotated_z(math.radians(35)),
                    v3(OSA_TURRET) + v3((0.3, 0.2, -0.55))))
    wreck.add(lying(osa_search(p.camo), 60, (2.6, -3.2, 0.0)))
    wreck.add(scorch(4.5, rng))
    return {"ground": Geo(), "lines": Geo(), "intact": intact, "destroyed": wreck,
            "parts": [(osa_search(p.camo), tuple(search_at), 16)],
            "hull": osa_hull(p.camo, p.camo_dark).add(moved(osa_turret(p.camo, p.camo_dark), OSA_TURRET)),
            "lights": {}}


# ------------------------------------------------------------------- S-300PS (SA-10 Grumble)

def maz543(paint):
    return sites.maz543(paint)


# The 5P85 TEL is a contributed model (assets-src/blend/s300_tel.blend: metres, nose +y, the
# ground at z = 0): the MAZ-543 chassis with its cab, equipment cabin and erector, and the four
# canisters standing upright behind it as a separate object, so the wreck can topple them.
S300_TEL = os.path.join(ROOT, "assets-src", "blend", "s300_tel.blend")


def contributed(path, name, paints):
    """Geometry of a hand-made mesh (object `name` in the .blend at `path`, metres, normals out):
    each face takes the paint its material `pal_NNN` maps to in `paints` (a Ramp shades it by
    its normal, like the generated models)."""
    with bpy.data.libraries.load(path) as (src, dst):
        dst.objects = [name]
    ob = dst.objects[0]
    me = ob.data
    mats = list(me.materials)
    g = Geo()
    for f in me.polygons:
        g.face([ob.matrix_world @ me.vertices[i].co for i in f.vertices], paints[int(mats[f.material_index].name[4:7])])
    bpy.data.objects.remove(ob)
    bpy.data.meshes.remove(me)
    for m in mats:
        if m.users == 0:
            bpy.data.materials.remove(m)
    return g


def tel_5p85(p):
    """The 5P85 TEL's truck and its erected canister pack."""
    truck = contributed(S300_TEL, "s300_tel", {168: p.camo, 234: STEEL, 150: GLASS, 239: TYRE})
    pack = contributed(S300_TEL, "s300_tel_canisters", {224: MISSILE, 238: DARK})
    return truck, pack


def s300_launcher(p):
    rng = random.Random(330)
    truck, pack = tel_5p85(p)
    wreck = wreck_of(truck, sink=0.3)
    # The burnt canister pack fallen backwards off its erector.
    lo, hi = pack.bounds()
    fallen = pack.recolour(BURNT).rotated_x(math.radians(82), (0, lo.y, lo.z))
    flo, _ = fallen.bounds()
    wreck.add(turned(moved(fallen, (0, 0, -flo.z)), 12, (0.6, -0.8, 0.0)))
    wreck.add(scorch(7.0, rng))
    return {"ground": Geo(), "lines": Geo(), "intact": Geo().add(truck).add(pack), "destroyed": wreck,
            "hull": Geo().add(truck).add(pack), "lights": {}}


def big_bird_array(p):
    """The 64N6's back-to-back array on its yoke (origin at the turntable): the main face 8.4 m
    across facing +y, tilted back, the IFF array on its back."""
    g = frustum((0, 0, 0), (0, 0, 0.6), 0.9, 0.8, STEEL, n=8)
    for sx in (-1, 1):
        g.add(beam((sx * 0.6, 0, 0.5), (sx * 2.4, 0.0, 2.2), 0.3, p.camo_dark))
    a = panel(8.4, 4.4, 0.75, p.camo, APERTURE, lines=DARK.lo, nx=8, nz=4)
    a.add(moved(panel(4.4, 1.0, 0.3, p.camo_dark, APERTURE).rotated_z(math.pi), (0, -0.55, 1.2)))
    g.add(moved(a.rotated_x(math.radians(20)), (0, 0.0, 2.9)))
    return g


def big_bird_vehicle(p):
    """The MAZ-7410 tractor and the semitrailer carrying the 64N6's equipment shelter and its
    turntable at the rear (the turntable over the origin)."""
    g = Geo()
    # Semitrailer.
    g.add(axles((-2.2, -3.6), 1.15, 0.55, 0.45))
    g.add(block(-1.4, 1.4, -4.6, 7.8, 1.0, 1.45, STEEL))
    g.add(block(-1.4, 1.4, 1.6, 7.6, 1.45, 4.2, p.camo, taper=0.04, taper_y=0.04))
    g.add(frustum((0, 0, 1.45), (0, 0, 3.2), 1.1, 1.0, p.camo_dark, n=10))
    for x, y in ((-1.7, -4.3), (1.7, -4.3), (-1.7, 1.2), (1.7, 1.2)):
        g.add(jack(x, y, 1.2, STEEL))
    # Tractor: three axles, cab-over with an equipment body behind.
    g.add(axles((14.5, 11.2, 9.8), 1.15, 0.65, 0.5))
    g.add(block(-1.2, 1.2, 8.6, 15.6, 0.8, 1.25, STEEL))
    g.add(truck_cab(p.camo, -1.35, 1.35, 13.2, 16.2, 1.2, 3.3))
    g.add(block(-1.35, 1.35, 10.6, 13.1, 1.25, 2.8, p.camo_dark))
    g.add(block(-0.9, 0.9, 7.6, 10.2, 1.25, 1.6, STEEL))
    return g


BIG_BIRD_PIVOT = (0.0, 0.0, 3.2)


def flap_lid(p):
    """The 5N63S on its MAZ-543: the F1 cabin on the deck and the 30N6 array raised on its post
    over the rear, facing back (-y)."""
    g = maz543(p.camo)
    g.add(block(-1.3, 1.3, -4.5, 1.8, 1.95, 4.2, p.camo, taper=0.04, taper_y=0.04))
    g.add(block(-0.5, 0.5, -6.6, -5.6, 1.95, 6.4, p.camo_dark))
    arr = panel(3.3, 3.3, 0.55, p.camo, APERTURE, lines=DARK.lo, nx=6, nz=6)
    arr = arr.rotated_x(math.radians(12)).rotated_z(math.pi)
    g.add(moved(arr, (0, -6.5, 7.3)))
    for x, y in ((-1.7, 3.2), (1.7, 3.2), (-1.7, -4.6), (1.7, -4.6)):
        g.add(jack(x, y, 1.4, STEEL))
    return g


FLAP_LID_AT = (16.0, -6.0)
FLAP_LID_HEADING = 160.0


def s300_radar(p):
    rng = random.Random(64)
    vehicle = big_bird_vehicle(p)
    intact = Geo().add(vehicle)
    intact.add(turned(flap_lid(p), FLAP_LID_HEADING, (*FLAP_LID_AT, 0.0)))
    intact.add(turned(van(7.0, p.camo, body_h=2.4), 100, (-13.0, 4.0, 0.0)))
    lines = Geo()
    for a, b in (((1.4, 3.0), (13.5, -3.0)), ((-1.4, 4.0), (-9.5, 4.3))):
        lines.line((*a, 0.05), (*b, 0.05), CABLE)
    array = big_bird_array(p)
    wreck = wreck_of(big_bird_vehicle(p))
    wreck.add(lying(array, -70, (-6.5, -3.0, 0.0)))
    wreck.add(turned(wreck_of(flap_lid(p), tilt=-3.0), FLAP_LID_HEADING, (*FLAP_LID_AT, 0.0)))
    wreck.add(turned(van(7.0, p.camo, body_h=2.4), 100, (-13.0, 4.0, 0.0)))
    wreck.add(scorch(8.0, rng))
    fl_top = turned_point((0.0, -6.5, 9.1), FLAP_LID_HEADING, (*FLAP_LID_AT, 0.0))
    return {"ground": Geo(), "lines": lines, "intact": intact, "destroyed": wreck,
            "parts": [(array, BIG_BIRD_PIVOT, 16)],
            "hull": big_bird_vehicle(p).add(moved(array, BIG_BIRD_PIVOT)),
            "lights": {RED_LIGHT: [(0.0, 0.0, 8.9), tuple(fl_top)], WHITE_LIGHT: [(-11.8, 4.0, 3.2)]}}


# ----------------------------------------------------------------------- 9K330 Tor (SA-15)

def tor_hull(paint, dark):
    """The GM-355 chassis, 7.5 m long and 3.3 m wide over the tracks, nose to +y."""
    g = Geo()
    prof = [(-3.7, 0.55), (-3.75, 1.85), (3.0, 1.9), (3.85, 1.25), (3.6, 0.55)]
    g.add(mklib.side_prism(prof, -1.1, 1.1, paint))
    for sx in (-1, 1):
        x0, x1 = sorted((sx * 1.1, sx * 1.65))
        g.add(block(x0, x1, -3.7, 3.4, 1.25, 1.85, paint))
        g.add(ad.running_gear(sx * 1.37, 0.5, [2.45, 1.45, 0.45, -0.55, -1.55, -2.55], 0.31,
                              (3.3, 0.95, 0.3), (-3.3, 0.75, 0.3), dark, rollers=(1.9, -0.1, -2.0),
                              track=TYRE, hub=STEEL))
    g.add(cab_glass(-1.0, -0.1, 3.42, 1.4, 1.75))
    return g


def tor_turret(paint, dark):
    """The turret (origin at its ring): the eight missiles' cells under the roof hatches, the
    tracking radar's flat array on the sloping front, the TV tracker beside it."""
    g = block(-1.45, 1.45, -2.0, 1.4, 0.0, 1.7, paint, taper=0.08, taper_y=0.05)
    g.add(block(-1.1, 1.1, -2.2, -0.6, 1.7, 1.95, dark))
    hatch = Geo()
    for x in (-1.1, 0.0, 1.1):
        hatch.line((x, -2.2, 1.96), (x, -0.6, 1.96), DARK.hi)
    for y in (-2.2, -1.4, -0.6):
        hatch.line((-1.1, y, 1.96), (1.1, y, 1.96), DARK.hi)
    g.add(hatch)
    arr = panel(1.7, 1.5, 0.3, paint, APERTURE, lines=DARK.lo, nx=5, nz=4)
    g.add(moved(arr.rotated_x(math.radians(25)), (0.15, 1.6, 0.95)))
    g.add(block(-1.4, -0.9, 1.3, 1.75, 0.6, 1.1, dark))
    g.face([(-1.3, 1.76, 0.7), (-1.0, 1.76, 0.7), (-1.0, 1.76, 1.0), (-1.3, 1.76, 1.0)], GLASS, (-1.15, 1.0, 0.85))
    return g


def tor_search(paint):
    """The search radar on its short mast (origin at the mast's foot), a flat array tilted back,
    facing +y."""
    g = frustum((0, 0, 0), (0, 0, 0.45), 0.25, 0.2, STEEL, n=6)
    a = panel(2.6, 1.05, 0.25, paint, APERTURE, lines=DARK.lo, nx=6, nz=2)
    g.add(moved(a.rotated_x(math.radians(22)), (0, 0.0, 0.95)))
    return g


TOR_TURRET = (0.0, -0.4, 1.9)
TOR_SEARCH = (0.0, -1.35, 1.7)


def tor_unit(p):
    rng = random.Random(15)
    intact = tor_hull(p.camo, p.camo_dark).add(moved(tor_turret(p.camo, p.camo_dark), TOR_TURRET))
    search_at = v3(TOR_TURRET) + v3(TOR_SEARCH)
    wreck = wreck_of(tor_hull(p.camo, p.camo_dark), sink=0.25)
    wreck.add(moved(tor_turret(p.camo, p.camo_dark).recolour(BURNT).rotated_z(math.radians(-25)),
                    v3(TOR_TURRET) + v3((0.2, -0.3, -0.35))))
    wreck.add(lying(tor_search(p.camo), 110, (-2.8, -3.0, 0.0)))
    wreck.add(scorch(4.5, rng))
    return {"ground": Geo(), "lines": Geo(), "intact": intact, "destroyed": wreck,
            "parts": [(tor_search(p.camo), tuple(search_at), 16)],
            "hull": tor_hull(p.camo, p.camo_dark).add(moved(tor_turret(p.camo, p.camo_dark), TOR_TURRET))
            .add(moved(tor_search(p.camo), search_at)),
            "lights": {}}


# ----------------------------------------------------------------------------- MIM-23 Hawk

def hawk(paint=WHITE):
    """The MIM-23 missile, 5.1 m long, 0.37 m across, tail at y = 0, its long clipped-delta
    wings at the rear."""
    g = frustum((0, 0, 0), (0, 3.85, 0), 0.185, 0.185, paint, n=8)
    g.add(frustum((0, 3.85, 0), (0, 5.08, 0), 0.185, 0.01, paint, n=8, caps=False))
    g.add(frustum((0, -0.1, 0), (0, 0.0, 0), 0.13, 0.17, DARK, n=8))
    g.add(cross_fins(0.18, 0.6, 2.2, 2.15, 0.5, 0.45, paint))
    return g


HAWK_PIVOT = (0.0, -0.3, 2.0)
HAWK_ELEVATION = 25.0


def m192(p, burnt=False):
    """The M192 towed launcher: its two-wheel carriage with four outriggers, the pedestal and
    the three launch beams with a missile riding on each, raised towards +y."""
    paint = BURNT if burnt else p.camo
    dark = BURNT if burnt else p.camo_dark
    g = Geo()
    g.add(axles((-0.4,), 1.25, 0.45, 0.35, BURNT if burnt else TYRE))
    g.add(block(-1.0, 1.0, -1.6, 1.4, 0.55, 0.95, dark))
    g.add(beam((0, 1.4, 0.75), (0, 4.0, 0.55), 0.16, dark))
    for a in (40, 140, 220, 320):
        leg = beam((0, 0.6, 0.75), (0, 2.5, 0.3), 0.14, dark)
        leg.add(block(-0.3, 0.3, 2.3, 2.9, 0.0, 0.15, STEEL if not burnt else BURNT))
        g.add(leg.rotated_z(math.radians(a)))
    g.add(frustum((0, 0, 0.95), (0, 0, 1.6), 0.6, 0.55, paint, n=10))
    for sx in (-1, 1):
        x0, x1 = sorted((sx * 0.35, sx * 0.6))
        g.add(block(x0, x1, -0.8, 0.4, 1.6, 2.2, paint, taper_y=0.2))
    rails = Geo()
    rails.add(block(-1.35, 1.35, -0.25, 0.25, -0.2, 0.2, paint))
    for x in (-1.15, 0.0, 1.15):
        rails.add(block(x - 0.09, x + 0.09, -1.2, 2.8, 0.15, 0.35, paint))
        if not burnt:
            rails.add(moved(hawk(), (x, -1.5, 0.58)))
    el = math.radians(HAWK_ELEVATION if not burnt else 3.0)
    g.add(moved(rails.rotated_x(el), HAWK_PIVOT))
    g.add(beam((0, 0.6, 1.6), v3(HAWK_PIVOT) + v3((0, 1.2 * math.cos(el), 1.2 * math.sin(el) - 0.2)), 0.16, STEEL))
    return g


def hawk_launcher(p):
    rng = random.Random(23)
    intact = m192(p)
    wreck = moved(m192(p, burnt=True).rotated_z(math.radians(-6)), (0, 0, -0.15))
    wreck.add(turned(hawk(BURNT), 75, (-2.6, 1.0, 0.2)))
    wreck.add(turned(hawk(BURNT).rotated_z(0.0), 200, (2.4, -0.5, 0.2)))
    wreck.add(scorch(4.0, rng))
    return {"ground": Geo(), "lines": Geo(), "intact": intact, "destroyed": wreck, "hull": m192(p),
            "lights": {}}


def par_antenna(p):
    """The AN/MPQ-50's reflector (origin at the pedestal's top): a wide curved reflector facing
    +y, the feed horn on its arm, the IFF bar along the top."""
    g = frustum((0, 0, 0), (0, 0, 0.35), 0.35, 0.3, STEEL, n=6)
    refl = Geo()
    n, w, h = 8, 5.2, 1.7
    for k in range(n):
        x0, x1 = -w / 2 + w * k / n, -w / 2 + w * (k + 1) / n
        y0, y1 = 0.55 * (x0 / (w / 2)) ** 2, 0.55 * (x1 / (w / 2)) ** 2
        refl.face2([(x0, -y0, 0), (x1, -y1, 0), (x1, -y1, h), (x0, -y0, h)], p.camo)
    refl.add(block(-w / 2 + 0.3, w / 2 - 0.3, -0.75, -0.55, 0.6, 0.85, STEEL))
    refl = refl.rotated_x(math.radians(-10))
    g.add(moved(refl, (0, 0.1, 0.3)))
    g.add(block(-1.8, 1.8, -0.15, 0.1, 2.05, 2.3, p.camo_dark))
    g.add(beam((0, 0.0, 0.4), (0, 1.7, 0.95), 0.1, STEEL))
    g.add(block(-0.25, 0.25, 1.55, 1.95, 0.8, 1.15, DARK))
    return g


def two_axle_trailer(length, w=2.3):
    g = axles((length / 2 - 1.0, -length / 2 + 1.0), w / 2 - 0.2, 0.45, 0.35)
    g.add(block(-w / 2, w / 2, -length / 2, length / 2, 0.55, 0.85, STEEL))
    g.add(beam((0, length / 2, 0.7), (0, length / 2 + 1.5, 0.5), 0.12, STEEL))
    for x, y in ((-w / 2 - 0.2, length / 2 - 0.3), (w / 2 + 0.2, length / 2 - 0.3),
                 (-w / 2 - 0.2, -length / 2 + 0.3), (w / 2 + 0.2, -length / 2 + 0.3)):
        g.add(jack(x, y, 0.7, STEEL))
    return g


def par_trailer(p):
    g = two_axle_trailer(5.4)
    g.add(block(-1.1, 1.1, -2.6, -0.6, 0.85, 2.1, p.camo))
    g.add(frustum((0, 0.9, 0.85), (0, 0.9, 2.3), 0.45, 0.4, p.camo_dark, n=8))
    return g


PAR_PIVOT = (0.0, 0.9, 2.3)


def hpi(p):
    """The AN/MPQ-46 illuminator: its twin dishes side by side on a pedestal on a two-axle
    trailer, aimed up towards +y."""
    g = two_axle_trailer(5.0)
    g.add(block(-1.0, 1.0, -2.4, -0.8, 0.85, 1.9, p.camo))
    g.add(frustum((0, 0.6, 0.85), (0, 0.6, 2.2), 0.4, 0.35, p.camo_dark, n=8))
    head = block(-0.4, 0.4, -0.5, 0.3, -0.35, 0.35, p.camo_dark)
    for sx in (-1, 1):
        head.add(moved(dish(0.8, 0.4, p.camo, APERTURE, feed=DARK.hi), (sx * 0.9, 0.4, 0.0)))
    g.add(moved(head.rotated_x(math.radians(12)), (0, 0.6, 2.75)))
    return g


HPI_AT = (11.0, 6.5)


def hawk_radar(p):
    rng = random.Random(46)
    intact = par_trailer(p)
    intact.add(turned(hpi(p), 20, (*HPI_AT, 0.0)))
    intact.add(turned(van(6.5, p.camo, body_h=2.3), -80, (-11.0, -5.5, 0.0)))
    intact.add(turned(van(4.0, p.camo_dark, body_h=1.8), 10, (-4.0, 11.0, 0.0)))
    lines = Geo()
    for a, b in (((1.2, -2.0), (9.5, 4.5)), ((-1.2, -2.0), (-8.5, -5.0)), ((-0.5, 2.7), (-3.5, 8.5))):
        lines.line((*a, 0.05), (*b, 0.05), CABLE)
    ant = par_antenna(p)
    wreck = wreck_of(par_trailer(p))
    wreck.add(lying(ant, 30, (2.8, 2.8, 0.0)))
    wreck.add(turned(wreck_of(hpi(p)), 20, (*HPI_AT, 0.0)))
    wreck.add(turned(van(6.5, p.camo, body_h=2.3), -80, (-11.0, -5.5, 0.0)))
    wreck.add(turned(van(4.0, p.camo_dark, body_h=1.8), 10, (-4.0, 11.0, 0.0)))
    wreck.add(scorch(6.0, rng))
    return {"ground": Geo(), "lines": lines, "intact": intact, "destroyed": wreck,
            "parts": [(ant, PAR_PIVOT, 16)],
            "hull": par_trailer(p).add(moved(ant, PAR_PIVOT)),
            "lights": {RED_LIGHT: [(0.0, 0.9, 4.8)], WHITE_LIGHT: [(-9.8, -5.3, 3.0)]}}


# ----------------------------------------------------------------------------------- assembly

def last_dot(root):
    """The colour of the dot the model's coarsest level draws."""
    dists = list(root["lod_dists"])
    last = bpy.data.objects.get(f"{root.name}_lod{len(dists) - 1}")
    for c in (last.children if last else ()):
        if "_dots_pal_" in c.name:
            return int(c.name.rsplit("_", 1)[1][:3])
    return 0xBF


def build(name, unit, dists):
    root = bpy.data.objects[name]
    dot = last_dot(root)
    for c in list(root.children):
        mklib.delete_tree(c)
    ground = unit["ground"].scaled(K)
    lod0 = heli.mesh_with_lines(root, f"{name}_lod0", ground)
    lod0["max_dist"] = dists[0]
    lod0["planes"] = sites.hull_planes(unit["hull"])
    if unit["lines"].lines:
        detail = bpy.data.objects.new(f"{lod0.name}_detail", None)
        for col in lod0.users_collection:
            col.objects.link(detail)
        detail.parent = lod0
        for c, segs in sorted(unit["lines"].scaled(K).lines.items()):
            ln = f"{detail.name}_lines_pal_{c:03}"
            mklib.link_child(detail, ln, mklib.lines_mesh(ln, segs, c))
    intact = sites.group(lod0, "s1v0", [[1, 0]], unit["intact"])
    everything = [unit["intact"].scaled(K), unit["destroyed"].scaled(K)]
    for k, (geo, pivot, slot) in enumerate(unit.get("parts", []), 1):
        part = heli.mesh_with_lines(intact, f"{intact.name}_part{k}", geo.scaled(K), tuple(v3(pivot) * K))
        part["state_slot"] = slot
        part["axis"] = 0
        everything.append(moved(geo, pivot).scaled(K))
    sites.group(lod0, "s1v1", [[1, 1]], unit["destroyed"])
    if unit["lights"]:
        heli.lights(lod0, 10, {c: [v3(p) * K for p in pts] for c, pts in unit["lights"].items()})
    if ground.faces:
        everything.append(ground)
    lod0["radius"] = heli.radius(*everything)
    near = bpy.data.objects.new(f"{name}_lod1", None)
    for col in lod0.users_collection:
        col.objects.link(near)
    near.parent = root
    near["max_dist"] = dists[1]
    lower_lods.rebuild(root, 1)
    last = bpy.data.objects.new(f"{name}_lod2", None)
    for col in lod0.users_collection:
        col.objects.link(last)
    last.parent = root
    last["max_dist"] = dists[-1]
    last["planes"] = []
    last["radius"] = 0
    dn = f"{last.name}_dots_pal_{dot:03}"
    mklib.link_child(last, dn, mklib.points_mesh(dn, [(0, 0, 0)], dot))
    root["lod_dists"] = list(dists)
    lo, hi = unit["intact"].bounds()
    print(f"{name}: {len(unit['intact'].faces)} faces intact, {len(unit['destroyed'].faces)} destroyed, "
          f"{len(lod0['planes'])} planes, radius {lod0['radius']}, size {tuple(round(c, 1) for c in hi - lo)} m")


# Switch distances (world units) of the detailed level, its copy and the dot; the radar sites
# keep their copy longer (the classic libraries had some launchers with odd or single levels).
DISTS = {"sa_missile": (1024, 2048, 32769), "sam_radar": (1024, 4096, 32769), "mobile_sam": (1024, 2048, 32769)}

SA2 = {"sa_missile": sa2_launcher, "sam_radar": fan_song_site}
S300 = {"sa_missile": s300_launcher, "sam_radar": s300_radar}
HAWK = {"sa_missile": hawk_launcher, "sam_radar": hawk_radar}
THEATERS = {
    "central_europe": (FOREST, dict(S300, mobile_sam=tor_unit)),
    "north_cape": (FOREST, S300),
    "cuba": (FOREST, SA2),
    "korea": (FOREST, SA2),
    "vietnam": (FOREST, SA2),
    "desert_storm": (DESERT, SA2),
    "libya": (DESERT, dict(SA2, mobile_sam=osa_unit)),
    "middle_east": (DESERT, SA2),
    "persian_gulf": (DESERT, HAWK),
}


def models(theater):
    """{root name: (builder, switch distances)} of a theater's SAM models."""
    paints, kinds = THEATERS[theater]
    lib = json.load(open(os.path.join(ROOT, "tools", "theater", "libraries", f"{theater}.json")))["kinds"]
    return {f"{theater}_{lib[kind]:03d}": ((lambda f=f: f(paints)), DISTS[kind]) for kind, f in kinds.items()}


def main():
    only = sys.argv[sys.argv.index("--") + 1:] if "--" in sys.argv else []
    for theater in THEATERS:
        if only and theater not in only:
            continue
        mklib.load_palette(os.path.join(ROOT, "assets", "palettes", f"tod_{theater}.ron"))
        bpy.ops.wm.open_mainfile(filepath=os.path.join(ROOT, "assets-src", "blend", f"{theater}.blend"))
        for name, (unit, dists) in models(theater).items():
            build(name, unit(), dists)
        bpy.ops.wm.save_mainfile()


if __name__ == "__main__":
    main()
