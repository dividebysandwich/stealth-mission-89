"""The external stores hung on the flyable jets' pylons and rails, and their racks (built into
aircraft.blend by aircraft.py, model indices in `WEAPON_MODELS` and `RACKS`).

Every store is drawn at its real size from its three-view (nose along +y, z up), its origin at
its suspension lugs: the top of the body over its centre of gravity, where it touches the rack or
the pylon. Missiles carry their motor flame (`flame`, the `_s14v1` group); bombs, pods and tanks
none. A rack's `mounts` are where its rounds' lugs sit (x, y, z below its own top at the
pylon, m) and their roll (degrees, + turning the round's top to the right), in firing order
backwards (the last round listed leaves first).

Dimensions (m: length, body diameter, span):
    AIM-9M 2.87 x 0.127, canards 0.64, fins 0.63     AIM-120A 3.65 x 0.178, wings 0.53, fins 0.45
    AGM-88A 4.17 x 0.254, wings 1.13, fins 0.64      Penguin Mk 3 3.18 x 0.28, wings 1.0 (folded)
    AGM-84A 3.84 x 0.343, wings and fins 0.91        AGM-65 2.49 x 0.305, wings 0.72
    GBU-12 3.27 x 0.273 (Mk 82), folded wings 0.49   Mk 20 Rockeye 2.34 x 0.335, fins 0.88
    Durandal 2.70 x 0.223, fins 0.43                 Mk 82 LDGP 2.21 x 0.273, conical fin 0.38
    Mk 82 Snakeye 2.48 x 0.273, folded Mk 15 0.51    Mk 122 Fireye (Mk 77 class) 3.0 x 0.48
    CBU-72, Mk 35, ISCB-1 (SUU-30 / SUU-49 class) 2.35 x 0.40, fins 0.60
    recon pod 4.0 x 0.40, 300 US gal tank 4.6 x 0.66, MXU-648 cargo pod 3.4 x 0.48
"""

import math
import os
import re

from mathutils import Matrix, Vector

from mklib import Geo, Ramp, block, frustum, hexa

WHITE = Ramp(224, 229)
MISSILE = Ramp(224, 231)
GREY = Ramp(227, 233)
NAVY = Ramp(228, 235)
DARK = Ramp(235, 239)
OLIVE = Ramp(165, 173)
ALUMINIUM = Ramp(224, 228)
JET_GREY = Ramp(231, 237)
YELLOW = Ramp(197, 201)
BROWN = Ramp(204, 207)
RED = Ramp(214, 219)
GLASS = Ramp(152, 156)
BLACK = 239
FLAME = 13


class Store:
    def __init__(self, body, flame=None):
        self.body, self.flame = body, flame


def body(stations, paint, n=10, bands=()):
    """A body of revolution about the y axis from (y, radius) stations, nose first; `bands` are
    (y0, y1, paint) rings painted over it."""
    g = Geo()
    for (ya, ra), (yb, rb) in zip(stations, stations[1:]):
        if abs(ya - yb) < 1e-6:
            if abs(ra - rb) > 1e-6:
                g.add(ring_face(ya, ra, rb, paint, n))
            continue
        p = paint
        for y0, y1, c in bands:
            if min(y0, y1) <= (ya + yb) / 2 <= max(y0, y1):
                p = c
        g.add(frustum((0, ya, 0), (0, yb, 0), max(ra, 0.004), max(rb, 0.004), p, n, caps=False))
    # The tail's end closed.
    y, r = stations[-1]
    if r > 0.004:
        g.add(frustum((0, y + 0.001, 0), (0, y, 0), r, r, paint, n, caps=True))
    return g


def ring_face(y, r0, r1, paint, n):
    """A flat ring at station y between radii r0 and r1 (a step in a body)."""
    g = Geo()
    for k in range(n):
        a0, a1 = 2 * math.pi * k / n, 2 * math.pi * (k + 1) / n
        pts = [(r0 * math.cos(a0), y, r0 * math.sin(a0)), (r1 * math.cos(a0), y, r1 * math.sin(a0)),
               (r1 * math.cos(a1), y, r1 * math.sin(a1)), (r0 * math.cos(a1), y, r0 * math.sin(a1))]
        g.face2(pts, paint)
    return g


def fins(g, y0, y1, r, height, paint, n=4, rot=45.0, tip=None, thick=0.012):
    """`n` thin fins round a body of radius `r`: root chord from y0 back to y1, `height` out
    from the body; the tip chord (`tip`: (y0, y1)) defaults to a clipped delta's."""
    t0, t1 = tip if tip else (y1 + (y0 - y1) * 0.25, y1)
    for k in range(n):
        a = math.radians(rot + k * 360.0 / n)
        d = Vector((math.cos(a), 0.0, math.sin(a)))
        side = Vector((-math.sin(a), 0.0, math.cos(a))) * (thick / 2)
        root0, root1 = Vector((0, y0, 0)) + d * r, Vector((0, y1, 0)) + d * r
        tip0 = Vector((0, t0, 0)) + d * (r + height)
        tip1 = Vector((0, t1, 0)) + d * (r + height)
        g.add(hexa([root0 - side, root1 - side, tip1 - side, tip0 - side,
                    root0 + side, root1 + side, tip1 + side, tip0 + side], paint))


def hang(g, lug_y, r):
    """The store moved so its lugs (on top of the body at lug_y) are its origin."""
    return g.transformed(lambda p: Vector((p.x, p.y - lug_y, p.z - r)))


def lugs(g, lug_y, r, spacing=0.356, paint=DARK):
    """The two suspension lugs on top of the body (14 in apart)."""
    for y in (lug_y + spacing / 2, lug_y - spacing / 2):
        g.add(block(-0.012, 0.012, y - 0.02, y + 0.02, r - 0.01, r + 0.004, paint))


def motor_flame(y, r, length):
    return frustum((0, y, 0), (0, y - length, 0), r * 0.75, r * 0.15, FLAME, 6)


# ---------------------------------------------------------------------------------------------
# Missiles


def aim9():
    """AIM-9M: the seeker dome, the double-delta canards, the long motor with the yellow (live
    warhead) and brown (live motor) bands, the clipped tail fins with their rollerons."""
    L, r = 2.87, 0.0635
    n, t = L / 2, -L / 2
    g = body([(n, 0.0), (n - 0.04, 0.035), (n - 0.10, 0.055), (n - 0.16, r), (t, r)], MISSILE,
             bands=[(n - 0.62, n - 0.70, YELLOW), (n - 1.10, n - 1.18, BROWN)])
    g.add(body([(n + 0.005, 0.0), (n - 0.05, 0.04)], GLASS, n=8))
    fins(g, n - 0.30, n - 0.58, r, 0.256, MISSILE, tip=(n - 0.50, n - 0.58))
    fins(g, t + 0.38, t + 0.02, r, 0.25, MISSILE, tip=(t + 0.16, t + 0.02))
    lugs(g, 0.0, r, spacing=0.9)
    return Store(hang(g, 0.0, r), hang(motor_flame(t, r, 1.1), 0.0, r))


def aim120():
    """AIM-120A: the ogive radome, the small mid-body wings and the tail control fins."""
    L, r = 3.65, 0.089
    n, t = L / 2, -L / 2
    g = body([(n, 0.0), (n - 0.10, 0.045), (n - 0.30, 0.078), (n - 0.50, r), (t, r)], WHITE,
             bands=[(n - 0.95, n - 1.02, YELLOW), (n - 1.75, n - 1.82, BROWN)])
    fins(g, n - 1.50, n - 1.95, r, 0.176, WHITE, tip=(n - 1.85, n - 1.95))
    fins(g, t + 0.32, t + 0.02, r, 0.135, WHITE, tip=(t + 0.14, t + 0.02))
    lugs(g, 0.0, r, spacing=1.0)
    return Store(hang(g, 0.0, r), hang(motor_flame(t, r, 1.3), 0.0, r))


def harm():
    """AGM-88A HARM: the long radome, the big double-delta wings mid-body, the tail fins."""
    L, r = 4.17, 0.127
    n, t = L / 2, -L / 2
    g = body([(n, 0.0), (n - 0.15, 0.05), (n - 0.45, 0.10), (n - 0.75, r), (t, r)], MISSILE,
             bands=[(n - 1.15, n - 1.25, YELLOW), (n - 2.55, n - 2.65, BROWN)])
    fins(g, n - 1.20, n - 2.25, r, 0.44, MISSILE, tip=(n - 1.95, n - 2.25))
    fins(g, t + 0.36, t + 0.02, r, 0.19, MISSILE, tip=(t + 0.14, t + 0.02))
    lugs(g, 0.0, r, spacing=1.2)
    return Store(hang(g, 0.0, r), hang(motor_flame(t, r, 1.3), 0.0, r))


def penguin():
    """Penguin Mk 3: the pointed IR seeker, the small canards, the four wings folded to fit
    under the wing, the rear body tapering to the motor."""
    L, r = 3.18, 0.14
    n, t = L / 2, -L / 2
    g = body([(n, 0.0), (n - 0.10, 0.06), (n - 0.40, 0.12), (n - 0.60, r), (t + 0.25, r), (t, 0.10)], NAVY,
             bands=[(n - 0.95, n - 1.05, YELLOW)])
    g.add(body([(n + 0.004, 0.0), (n - 0.08, 0.05)], GLASS, n=8))
    fins(g, n - 0.55, n - 0.75, r, 0.10, NAVY, tip=(n - 0.70, n - 0.75))
    fins(g, t + 1.10, t + 0.30, r, 0.24, NAVY, tip=(t + 0.55, t + 0.30))
    lugs(g, 0.0, r)
    return Store(hang(g, 0.0, r), hang(motor_flame(t, 0.10, 1.2), 0.0, r))


def harpoon():
    """AGM-84A Harpoon: the radome, the body with the cruise wings aft and the tail fins
    behind them, the turbojet's flush intake under the body."""
    L, r = 3.84, 0.1715
    n, t = L / 2, -L / 2
    g = body([(n, 0.0), (n - 0.15, 0.08), (n - 0.40, 0.15), (n - 0.60, r), (t + 0.10, r), (t, 0.12)], GREY,
             bands=[(n - 0.70, n - 0.78, YELLOW)])
    g.add(body([(n + 0.003, 0.0), (n - 0.12, 0.07)], DARK, n=8))
    fins(g, -0.20, -0.75, r, 0.29, GREY, tip=(-0.55, -0.75))
    fins(g, t + 0.40, t + 0.08, r, 0.29, GREY, tip=(t + 0.25, t + 0.08))
    # The intake's scoop under the middle.
    g.add(block(-0.10, 0.10, -0.55, -1.15, -r - 0.07, -r + 0.02, DARK, taper_y=0.0))
    lugs(g, 0.0, r, spacing=0.76)
    return Store(hang(g, 0.0, r), hang(motor_flame(t, 0.12, 1.1), 0.0, r))


def maverick():
    """AGM-65: the glass seeker dome, the stubby body, the long-chord delta wings and the
    control fins behind them."""
    L, r = 2.49, 0.1525
    n, t = L / 2, -L / 2
    g = body([(n - 0.05, 0.10), (n - 0.12, 0.14), (n - 0.18, r), (t, r)], GREY,
             bands=[(n - 0.55, n - 0.62, YELLOW), (n - 1.40, n - 1.47, BROWN)])
    g.add(body([(n, 0.0), (n - 0.02, 0.06), (n - 0.05, 0.10)], GLASS, n=10))
    fins(g, n - 0.85, t + 0.30, r, 0.21, GREY, tip=(t + 0.50, t + 0.30))
    fins(g, t + 0.25, t + 0.02, r, 0.21, GREY, tip=(t + 0.10, t + 0.02))
    lugs(g, 0.0, r)
    return Store(hang(g, 0.0, r), hang(motor_flame(t, r, 1.0), 0.0, r))


# ---------------------------------------------------------------------------------------------
# Bombs, dispensers, pods and tanks (no motor)


def mk82_body(n, r=0.1365, paint=OLIVE, nose=YELLOW):
    """The Mk 82's ogive case from its nose fuze at y n back 1.72 m to the tail's start,
    yellow-banded (high explosive)."""
    return body([(n, 0.0), (n - 0.04, 0.03), (n - 0.25, 0.09), (n - 0.55, 0.125), (n - 0.80, r),
                 (n - 1.40, r), (n - 1.72, 0.11)], paint, bands=[(n - 0.30, n - 0.36, nose), (n - 0.42, n - 0.48, nose)])


def gbu12():
    """GBU-12 Paveway II: the laser seeker and its canards on the Mk 82, the folded wing
    group at the tail."""
    L, r = 3.27, 0.1365
    n = L / 2 - 0.10
    g = Geo()
    # The seeker's gimbal ball and ring, its body with the canards.
    g.add(body([(L / 2, 0.0), (L / 2 - 0.04, 0.06), (L / 2 - 0.10, 0.065), (L / 2 - 0.16, 0.06),
                (L / 2 - 0.50, 0.07), (L / 2 - 0.60, 0.09)], OLIVE))
    fins(g, L / 2 - 0.20, L / 2 - 0.42, 0.065, 0.12, OLIVE, tip=(L / 2 - 0.36, L / 2 - 0.42))
    g.add(mk82_body(L / 2 - 0.58))
    t = L / 2 - 0.58 - 1.72
    g.add(body([(t, 0.11), (t - 0.75, 0.10), (-L / 2, 0.08)], OLIVE))
    fins(g, t - 0.10, -L / 2 + 0.02, 0.10, 0.145, OLIVE, tip=(-L / 2 + 0.25, -L / 2 + 0.02))
    lugs(g, -0.05, r)
    return Store(hang(g, -0.05, r))


def mk82():
    """Mk 82 low drag: the ogive case and its conical fin assembly with four fins."""
    r = 0.1365
    n = 1.105
    g = mk82_body(n)
    t = n - 1.72
    g.add(body([(t, 0.11), (-1.105, 0.16)], OLIVE))
    fins(g, t - 0.05, -1.10, 0.10, 0.09, OLIVE, rot=45.0, tip=(-0.95, -1.10))
    lugs(g, 0.0, r)
    return Store(hang(g, 0.0, r))


def snakeye():
    """Mk 82 Snakeye: the case with the Mk 15 retarding tail, its four blades folded into a
    box behind it."""
    r = 0.1365
    n = 1.12
    g = mk82_body(n)
    t = n - 1.72
    g.add(body([(t, 0.11), (t - 0.15, 0.10), (-1.36, 0.06)], OLIVE))
    for k in range(4):
        a = math.radians(45 + 90 * k)
        d, s = Vector((math.cos(a), 0, math.sin(a))), Vector((-math.sin(a), 0, math.cos(a)))
        root0, root1 = Vector((0, t - 0.05, 0)) + d * 0.11, Vector((0, -1.36, 0)) + d * 0.06
        g.face2([root0, root1, root1 + d * 0.19 + s * 0.12, root0 + d * 0.14 + s * 0.12], OLIVE)
        g.face2([root0, root1, root1 + d * 0.19 - s * 0.12, root0 + d * 0.14 - s * 0.12], OLIVE)
    lugs(g, 0.0, r)
    return Store(hang(g, 0.0, r))


def rockeye(band=YELLOW):
    """Mk 20 Rockeye (and the Rockeye II, its band green... here brown): the dispenser's fuzed
    nose, the cylindrical case, the four big tail fins."""
    L, r = 2.34, 0.1675
    n, t = L / 2, -L / 2
    g = body([(n, 0.0), (n - 0.06, 0.03), (n - 0.30, 0.12), (n - 0.55, r), (t + 0.45, r), (t + 0.05, 0.10),
              (t, 0.07)], OLIVE, bands=[(n - 0.62, n - 0.70, band)])
    fins(g, t + 0.50, t + 0.02, 0.12, 0.30, OLIVE, rot=45.0, tip=(t + 0.20, t + 0.02))
    lugs(g, 0.0, r)
    return Store(hang(g, 0.0, r))


def durandal():
    """Durandal: the slim case, its booster aft, the cruciform tail fins round the parachute
    pack."""
    L, r = 2.70, 0.1115
    n, t = L / 2, -L / 2
    g = body([(n, 0.0), (n - 0.10, 0.05), (n - 0.40, 0.10), (n - 0.55, r), (t, r)], Ramp(162, 168),
             bands=[(n - 0.70, n - 0.78, YELLOW)])
    fins(g, t + 0.32, t + 0.02, r, 0.105, Ramp(162, 168), rot=0.0, tip=(t + 0.12, t + 0.02))
    lugs(g, 0.0, r)
    return Store(hang(g, 0.0, r))


def fireye():
    """Mk 122 Fireye: an unpainted fire bomb canister, rounded both ends, finless."""
    L, r = 3.0, 0.24
    n, t = L / 2, -L / 2
    g = body([(n, 0.0), (n - 0.08, 0.10), (n - 0.30, 0.20), (n - 0.60, r), (t + 0.80, r), (t + 0.25, 0.17),
              (t, 0.05)], ALUMINIUM, bands=[(n - 0.65, n - 0.72, RED)])
    lugs(g, 0.0, r)
    return Store(hang(g, 0.0, r))


def dispenser(band, band2=None, paint=OLIVE):
    """A SUU-30 class cluster dispenser (CBU-72, Mk 35, ISCB-1): the ogive nose, the case, the
    tail cone and its fins."""
    L, r = 2.35, 0.20
    n, t = L / 2, -L / 2
    bands = [(n - 0.45, n - 0.53, band)]
    if band2 is not None:
        bands.append((n - 0.60, n - 0.68, band2))
    g = body([(n, 0.0), (n - 0.05, 0.04), (n - 0.30, 0.15), (n - 0.50, r), (t + 0.55, r), (t + 0.05, 0.11),
              (t, 0.08)], paint, bands=bands)
    fins(g, t + 0.55, t + 0.02, 0.15, 0.15, paint, rot=45.0, tip=(t + 0.20, t + 0.02))
    lugs(g, 0.0, r)
    return Store(hang(g, 0.0, r))


def camera_pod():
    """A reconnaissance pod: its long body with the camera windows forward and under it."""
    L, r = 4.0, 0.20
    n, t = L / 2, -L / 2
    g = body([(n, 0.0), (n - 0.10, 0.08), (n - 0.45, 0.18), (n - 0.70, r), (t + 0.60, r), (t + 0.10, 0.10),
              (t, 0.04)], JET_GREY)
    for y in (n - 0.95, n - 1.55):
        for a in (-50.0, 0.0, 50.0):
            ang = math.radians(a - 90.0)
            d = Vector((math.cos(ang), 0, math.sin(ang)))
            s = Vector((-math.sin(ang), 0, math.cos(ang)))
            c = Vector((0, y, 0)) + d * (r + 0.003)
            g.face2([c - s * 0.06 + Vector((0, 0.12, 0)), c + s * 0.06 + Vector((0, 0.12, 0)),
                     c + s * 0.06 - Vector((0, 0.12, 0)), c - s * 0.06 - Vector((0, 0.12, 0))], BLACK)
    lugs(g, 0.0, r)
    return Store(hang(g, 0.0, r))


def fuel_tank():
    """A 300 US gallon tank (1,900 lb of fuel): the long pointed nose, the cylinder, the
    tail cone and its two small fins."""
    L, r = 4.6, 0.33
    n, t = L / 2 - 0.2, -L / 2 - 0.2
    g = body([(n, 0.0), (n - 0.30, 0.13), (n - 0.80, 0.26), (n - 1.40, r), (t + 1.10, r), (t + 0.45, 0.20),
              (t, 0.03)], JET_GREY, n=12)
    fins(g, t + 0.75, t + 0.15, 0.22, 0.20, JET_GREY, n=2, rot=-60.0, tip=(t + 0.35, t + 0.15))
    lugs(g, 0.0, r, spacing=0.76)
    return Store(hang(g, 0.0, r))


def cargo_pod():
    """An MXU-648 cargo pod (a BLU-1 case): rounded nose, the access door's seam, the finned
    tail cone."""
    L, r = 3.4, 0.24
    n, t = L / 2, -L / 2
    g = body([(n, 0.0), (n - 0.08, 0.12), (n - 0.30, 0.21), (n - 0.55, r), (t + 0.75, r), (t + 0.15, 0.12),
              (t, 0.06)], JET_GREY, n=10, bands=[(n - 0.60, n - 0.64, DARK), (t + 0.80, t + 0.84, DARK)])
    fins(g, t + 0.65, t + 0.02, 0.13, 0.13, JET_GREY, rot=45.0, tip=(t + 0.25, t + 0.02))
    lugs(g, 0.0, r)
    return Store(hang(g, 0.0, r))


# Weapon type (src/loadout.rs `WEAPONS`) -> (model index, builder): the stores the jets hang.
# Hellfire, Stinger and the gun pod hang on the helicopters only (their own models).
WEAPON_MODELS = {
    0: (50, aim9), 1: (51, aim120), 2: (52, harm), 3: (53, penguin), 4: (54, harpoon), 5: (55, maverick),
    6: (56, gbu12), 7: (57, rockeye), 8: (58, durandal), 9: (59, mk82), 10: (60, snakeye),
    11: (61, lambda: rockeye(BROWN)), 12: (62, fireye), 13: (63, lambda: dispenser(YELLOW, BROWN)),
    14: (64, lambda: dispenser(RED)), 15: (65, lambda: dispenser(YELLOW, Ramp(176, 180))),
    16: (66, camera_pod), 17: (67, fuel_tank), 18: (68, cargo_pod),
}
# Body radius (m) of each store, and its length: for the racks' spacing and the checks.
RADIUS = {0: 0.0635, 1: 0.089, 2: 0.127, 3: 0.14, 4: 0.1715, 5: 0.1525, 6: 0.1365, 7: 0.1675, 8: 0.1115, 9: 0.1365,
          10: 0.1365, 11: 0.1675, 12: 0.24, 13: 0.20, 14: 0.20, 15: 0.20, 16: 0.20, 17: 0.33, 18: 0.24}


# ---------------------------------------------------------------------------------------------
# Racks and launchers: built about their top at the pylon (origin), hanging below it.


class Rack:
    def __init__(self, body, mounts):
        self.body, self.mounts = body, mounts


def rail_launcher():
    """LAU-7 / LAU-129 class: the missile launcher rail's slim housing, its rail underneath."""
    g = Geo()
    g.add(hexa([(-0.05, 1.25, 0.0), (0.05, 1.25, 0.0), (0.05, -1.30, 0.0), (-0.05, -1.30, 0.0),
                (-0.045, 1.05, -0.10), (0.045, 1.05, -0.10), (0.045, -1.30, -0.10), (-0.045, -1.30, -0.10)], GREY))
    g.add(block(-0.022, 0.022, 0.95, -1.30, -0.12, -0.10, DARK))
    return Rack(g, [((0.0, 0.0, -0.12), 0.0)])


def lau117():
    """LAU-117: the Maverick's launcher, deeper than a Sidewinder rail."""
    g = Geo()
    g.add(hexa([(-0.07, 1.00, 0.0), (0.07, 1.00, 0.0), (0.07, -1.05, 0.0), (-0.07, -1.05, 0.0),
                (-0.065, 0.80, -0.17), (0.065, 0.80, -0.17), (0.065, -1.05, -0.17), (-0.065, -1.05, -0.17)], GREY))
    g.add(block(-0.03, 0.03, 0.75, -1.05, -0.19, -0.17, DARK))
    return Rack(g, [((0.0, 0.0, -0.19), 0.0)])


def lau118():
    """LAU-118: the HARM's long launcher."""
    g = Geo()
    g.add(hexa([(-0.07, 1.40, 0.0), (0.07, 1.40, 0.0), (0.07, -1.55, 0.0), (-0.07, -1.55, 0.0),
                (-0.065, 1.15, -0.16), (0.065, 1.15, -0.16), (0.065, -1.55, -0.16), (-0.065, -1.55, -0.16)], GREY))
    g.add(block(-0.03, 0.03, 1.10, -1.55, -0.18, -0.16, DARK))
    return Rack(g, [((0.0, 0.0, -0.18), 0.0)])


# The triple ejector rack: its beam, the two shoulder stations on its canted faces (45 degrees
# out) and the bottom one, staggered aft so the rounds' tails clear each other.
TER_DEPTH = 0.30
TER_SHOULDER = (0.105, -0.13)
TER_STAGGER = 0.0


def ter():
    """TER-9/A: the beam under the pylon, its sway braces, the three ejector stations."""
    g = Geo()
    w, d = 0.10, TER_DEPTH
    sx, sz = TER_SHOULDER
    g.add(hexa([(-w, 1.15, 0.0), (w, 1.15, 0.0), (w, -1.15, 0.0), (-w, -1.15, 0.0),
                (-0.05, 1.05, -d), (0.05, 1.05, -d), (0.05, -1.05, -d), (-0.05, -1.05, -d)], GREY))
    for y in (0.30, -0.30):
        for s in (-1, 1):
            g.add(block(s * sx - 0.02, s * sx + 0.02, y - 0.03, y + 0.03, sz - 0.03, sz + 0.06, DARK))
    return Rack(g, [((-sx, 0.0, sz), -45.0), ((sx, 0.0, sz), 45.0), ((0.0, TER_STAGGER, -d), 0.0)])


RACKS = {"rail": (70, rail_launcher), "lau117": (71, lau117), "lau118": (72, lau118), "ter": (73, ter)}


def rack_for(kind, weapon, rounds):
    """The rack a station of `kind` ("rail": a missile rail of its own, "pylon", "side": a rail
    on a pylon's side) hangs weapon `weapon` on, `rounds` of it; None: straight on the pylon."""
    if kind in ("rail", "side"):
        return None
    if weapon in (0, 1):
        return "rail"
    if weapon == 5:
        return "lau117"
    if weapon == 2:
        return "lau118"
    if rounds >= 2:
        return "ter"
    return None


def posed(geo, at, roll):
    """`geo` rolled `roll` degrees about its lugs (its top to the right for +) and moved to `at`."""
    m = Matrix.Rotation(math.radians(roll), 4, "Y")
    at = Vector(at)
    return geo.transformed(lambda p: m @ p + at)


# ---------------------------------------------------------------------------------------------
# The stations, from src/loadout.rs: positions (m right of the centreline, aft of the nose),
# allowed loads, rounds per pylon.

LOADOUT_RS = os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "..", "src", "loadout.rs")


def _loadout_text():
    return open(LOADOUT_RS).read()


def _u8_lists(text):
    named = {"HELLFIRE": 20, "STINGER": 21, "GUN_POD": 22}
    out = {}
    for name, body in re.findall(r"^const ([A-Z0-9_]+): \[u8; \d+\] = \[([^\]]*)\];", text, re.M):
        out[name] = [named[v] if v in named else int(v) for v in (x.strip() for x in body.split(",")) if v]
    return out


def pylon_rounds():
    """Rounds of each weapon type on a pylon (`PYLON_QTY`)."""
    return _u8_lists(_loadout_text())["PYLON_QTY"]


def loadout_stations(table):
    """The hardpoints of `table` (e.g. "F16_STATIONS"): [(x, aft, loads)] in station order."""
    return [(x, aft, loads) for x, aft, loads, _ in _hardpoints(table)]


def exclusive_stations(table):
    """Pairs of stations of `table` that are never loaded together (`Hardpoint::excludes`)."""
    return {(min(i, j), max(i, j)) for i, (*_, ex) in enumerate(_hardpoints(table)) for j in ex}


def _hardpoints(table):
    text = _loadout_text()
    lists = _u8_lists(text)
    start = text.index(f"static {table}:")
    body = text[start:text.index("];", start)]
    out = []
    for x, aft, loads, ex in re.findall(
            r"pylon\(\s*\"[^\"]*\",\s*\"[^\"]*\",\s*(-?[\d.]+),\s*(-?[\d.]+),\s*(?:Some\(\d+\)|None),\s*&([A-Z0-9_]+),?\s*\)"
            r"(?:\s*\.excluding\(&\[([\d, ]*)\]\))?",
            body):
        out.append((float(x), float(aft), lists[loads], [int(v) for v in ex.split(",") if v.strip()]))
    return out
