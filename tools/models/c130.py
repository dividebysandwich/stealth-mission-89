"""The AC-130 gunship's geometry (model 76, built by aircraft.py's `c130`), measured on a C-130H
three-view at 39.67 px/m (side view: nose tip at px 763, fuselage centre line at row 372.5; plan
view: nose tip at px 42, centre line at row 821.5).

Stations `d` run aft from the nose tip; the model's y is `Y0 - d` (nose along +y), x right, z up
from the fuselage's centre line. The guns (40 mm Bofors, 105 mm howitzer) fire from the left side
aft of the main gear's sponson; the sensor turret hangs under the left side of the nose.
"""

import math

from mathutils import Vector

import mklib
from mklib import Geo, Ramp, area, cut, frustum, hexa, loft, tube, v3, wing

Y0 = 14.8
# Gunship grey, darker for the modern shading.
GREY = Ramp(232, 238)
GREY_L = Ramp(226, 236)
DARK = Ramp(235, 239)
CANOPY = Ramp(144, 145)
BLACK = 239
SEAM = 238
# The ground below the centre line (wheels on it), the gear's wheels.
GROUND = -2.74
MAIN_R = 0.55
NOSE_R = 0.40


def y_of(d):
    return Y0 - d


# The fuselage: (station, top, belly, half width) from the side view's silhouette and the plan
# view's outline (the tail cone's width under the tailplane interpolated).
BODY = [
    (-0.15, -0.48, -0.52, 0.02), (-0.08, -0.05, -0.98, 0.30), (0.05, 0.08, -1.17, 0.42), (0.25, 0.15, -1.27, 0.52),
    (0.5, 0.24, -1.35, 0.60), (0.75, 0.33, -1.43, 0.68), (1.0, 0.58, -1.53, 0.80), (1.25, 0.85, -1.61, 0.93),
    (1.5, 1.13, -1.68, 1.04), (1.75, 1.36, -1.74, 1.14), (2.0, 1.55, -1.81, 1.24), (2.45, 1.78, -1.90, 1.43),
    (2.95, 1.90, -2.00, 1.62), (3.45, 1.98, -2.05, 1.80), (3.96, 2.03, -2.06, 1.98), (4.5, 2.07, -2.08, 2.08),
    (5.5, 2.08, -2.08, 2.13), (17.07, 2.08, -2.08, 2.13), (17.57, 2.08, -1.80, 2.13), (18.07, 2.08, -1.63, 2.12),
    (18.58, 2.08, -1.42, 2.11), (19.59, 2.08, -1.00, 2.08), (20.6, 2.08, -0.54, 2.05), (21.6, 2.08, -0.09, 2.00),
    (22.6, 2.08, 0.34, 1.90), (23.6, 2.08, 0.77, 1.76), (24.6, 2.07, 1.17, 1.55), (25.6, 2.06, 1.27, 1.30),
    (26.6, 2.04, 1.42, 1.05), (27.65, 2.02, 1.63, 0.75), (28.66, 2.00, 1.83, 0.42), (29.2, 1.97, 1.88, 0.20),
    (29.5, 1.94, 1.91, 0.04),
]
# Points per half ring, from the belly's centre up to the top's.
HALF = 12


def section(top, belly, hw, d):
    """The right half of the section, belly to top: a superellipse, flatter at the cabin's floor."""
    zc, hh = (top + belly) / 2, (top - belly) / 2
    floor = 2.0 + 0.6 * min(max((d - 3.0) / 2.0, 0.0), 1.0)
    pts = []
    for k in range(1, HALF):
        a = -math.pi / 2 + math.pi * k / HALF
        ca, sa = math.cos(a), math.sin(a)
        p = floor if sa < 0 else 2.0
        x = abs(ca) ** (2 / p) * hw
        z = zc + math.copysign(abs(sa) ** (2 / p), sa) * hh
        pts.append((x, z))
    return pts


def ring(d, top, belly, hw):
    y = y_of(d)
    right = [(x, y, z) for x, z in section(top, belly, hw, d)]
    return [(0.0, y, belly)] + right + [(0.0, y, top)] + [(-x, y, z) for x, y, z in reversed(right)]


def station(d):
    """(top, belly, half width) at station d."""
    for (da, ta, ba, wa), (db, tb, bb, wb) in zip(BODY, BODY[1:]):
        if da <= d <= db:
            u = (d - da) / (db - da)
            return ta + (tb - ta) * u, ba + (bb - ba) * u, wa + (wb - wa) * u
    return BODY[-1][1:]


def skin_x(d, z):
    """The skin's half width at station d, height z."""
    top, belly, hw = station(d)
    pts = [(0.0, belly)] + section(top, belly, hw, d) + [(0.0, top)]
    for (xa, za), (xb, zb) in zip(pts, pts[1:]):
        if za <= z <= zb:
            return xa + (xb - xa) * (z - za) / max(zb - za, 1e-6)
    return 0.0


# The flight deck's glazing, cut into the skin where both views hold a pane: side outlines in
# (station, z), front outlines in (x, z), all convex. The windscreen's four panes (two flat ones
# either side of the centre post, two corner panes), the eyebrows over them, two side windows and
# two eyebrows a side, and the lower nose windows by the pilots' feet.
WINDSCREEN_SIDE = [(0.95, 0.55), (1.55, 0.55), (2.0, 1.6), (1.25, 1.6)]
SCREEN_PANES = [(0.07, 0.56), (0.62, 0.98)]
SCREEN_Z = (0.66, 1.24)
BROW_Z = (1.33, 1.56)
BROW_PANES = [(0.10, 0.46), (0.54, 0.92)]
SIDE_WINDOWS = [[(1.55, 0.72), (2.0, 0.72), (2.0, 1.22), (1.75, 1.22)],
                [(2.06, 0.72), (2.5, 0.72), (2.5, 1.22), (2.06, 1.22)]]
SIDE_BROWS = [[(1.85, 1.30), (2.15, 1.30), (2.15, 1.52), (1.95, 1.52)],
              [(2.21, 1.30), (2.5, 1.30), (2.5, 1.50), (2.21, 1.50)]]
LOWER_SIDE = [(0.92, -0.42), (1.3, -0.42), (1.3, 0.08), (0.98, 0.08)]
LOWER_FRONT = (0.5, 0.98)


def windows():
    out = []
    side = [(y_of(d), z) for d, z in WINDSCREEN_SIDE]
    lo, hi = SCREEN_Z
    blo, bhi = BROW_Z
    for s in (1, -1):
        for a, b in SCREEN_PANES:
            out.append([("side", side), ("front", [(s * a, lo), (s * b, lo), (s * b, hi), (s * a, hi)])])
        for a, b in BROW_PANES:
            out.append([("side", side), ("front", [(s * a, blo), (s * b, blo), (s * b, bhi), (s * a, bhi)])])
        far = [(s * 0.8, -2.0), (s * 3.0, -2.0), (s * 3.0, 2.5), (s * 0.8, 2.5)]
        for w in SIDE_WINDOWS + SIDE_BROWS:
            out.append([("side", [(y_of(d), z) for d, z in w]), ("front", far)])
        a, b = LOWER_FRONT
        out.append([("side", [(y_of(d), z) for d, z in LOWER_SIDE]),
                    ("front", [(s * a, -1.0), (s * b, -1.0), (s * b, 0.5), (s * a, 0.5)])])
    return out


def fuselage():
    rings = [[v3(p) for p in ring(*b)] for b in BODY]
    wins = windows()
    g = Geo()
    m = len(rings[0])
    for i in range(len(rings) - 1):
        a, b = rings[i], rings[i + 1]
        (da, ta, ba, _), (db, tb, bb, _) = BODY[i], BODY[i + 1]
        mid = Vector((0, y_of((da + db) / 2), (ta + ba + tb + bb) / 4))
        for k in range(m):
            k1 = (k + 1) % m
            quad = [a[k], a[k1], b[k1], b[k]]
            for t in ([quad[0], quad[1], quad[2]], [quad[0], quad[2], quad[3]]):
                if area(t) < 1e-7:
                    continue
                if da < 3.0:
                    glass, body = cut(t, wins)
                else:
                    glass, body = [], [t]
                for piece in glass:
                    if area(piece) > 1e-6:
                        g.face(piece, CANOPY, mid)
                for piece in body:
                    if area(piece) > 1e-6:
                        g.face(piece, GREY, mid)
    return g


def skin_point(d, z, s, out=0.02):
    return Vector((s * (skin_x(d, z) + out), y_of(d), z))


def seam(g, path, s, step=0.3):
    """A seam along the skin through side view points (d, z), on side s."""
    for (da, za), (db, zb) in zip(path, path[1:]):
        n = max(1, math.ceil(math.hypot(db - da, zb - za) / step))
        for k in range(n):
            u0, u1 = k / n, (k + 1) / n
            g.line(skin_point(da + (db - da) * u0, za + (zb - za) * u0, s),
                   skin_point(da + (db - da) * u1, za + (zb - za) * u1, s), SEAM)


def seams():
    """The crew door, the paratroop doors, the ramp's outline, the radome's joint, the windows'
    frames."""
    g = Geo()
    for s in (-1, 1):
        # The radome's joint.
        seam(g, [(0.7, 0.30), (0.62, -0.4), (0.7, -1.38)], s)
        for d0, d1, z0, z1 in [(3.4, 4.3, 0.75, -1.85), (18.0, 18.9, 1.25, -0.95)]:
            seam(g, [(d0, z0), (d1, z0), (d1, z1), (d0, z1), (d0, z0)], s)
        # The ramp and the aft cargo door.
        seam(g, [(19.1, -1.2), (24.9, 1.15)], s)
        for w in SIDE_WINDOWS + SIDE_BROWS + [LOWER_SIDE]:
            seam(g, w + w[:1], s)
        # Portholes along the cabin.
        for d in (6.5, 9.6, 15.6, 21.4):
            r = 0.17
            pts = [(d + r * math.cos(a), 0.55 + r * math.sin(a)) for a in [k * math.pi / 4 for k in range(9)]]
            seam(g, pts, s, step=1.0)
    return g


# The wing: leading edge at station 11.29 all along; the trailing edge at 15.78 out to x 5.84,
# then straight to 14.07 at the tip (x 20.08); 1.5 degrees of dihedral from the root's chord line.
WING_LE = 11.29
WING_BREAK = 5.84
WING_TIP = 20.08
WING_Z = 1.80
DIHEDRAL = math.tan(math.radians(1.5))


def wing_z(x):
    return WING_Z + abs(x) * DIHEDRAL


def wing_te(x):
    x = abs(x)
    if x <= WING_BREAK:
        return 15.78
    return 15.78 + (14.07 - 15.78) * (x - WING_BREAK) / (WING_TIP - WING_BREAK)


def wing_geo():
    g = Geo()
    p = GREY
    for (xa, ta), (xb, tb) in [((0.0, 0.17), (WING_BREAK, 0.16)), ((WING_BREAK, 0.16), (WING_TIP - 0.15, 0.12))]:
        g.add(wing((xa, y_of(WING_LE), wing_z(xa)), wing_te(xa) - WING_LE, (xb, y_of(WING_LE), wing_z(xb)),
                   wing_te(xb) - WING_LE, p, t=ta, t_tip=tb, peak=0.3))
    # The rounded tips.
    x0 = WING_TIP - 0.15
    c0 = wing_te(x0) - WING_LE
    tips = [(x0, WING_LE, c0), (WING_TIP + 0.1, WING_LE + 0.25, c0 - 0.45), (WING_TIP + 0.22, WING_LE + 0.8, c0 - 1.4)]
    for (xa, la, ca), (xb, lb, cb) in zip(tips, tips[1:]):
        g.add(wing((xa, y_of(la), wing_z(xa)), ca, (xb, y_of(lb), wing_z(xb)), cb, p, t=0.12, peak=0.3))
    return g


# The engines: x of the inner and outer nacelles; the spinner's tip, the propeller's plane and the
# nacelle's end (stations); the nacelle's axis below the wing's chord line.
ENGINES = [5.25, 10.0]
PROP = 8.75
NACELLE_END = 15.6
NACELLE_DROP = 0.62
PROP_R = 2.05
PROP_SLOT = 17


def nacelle_z(x):
    return wing_z(x) - NACELLE_DROP


def nacelle(x):
    z = nacelle_z(x)
    g = Geo()
    # The cowling: round behind the spinner, deepening below for the intake and the oil cooler,
    # its top running into the wing's leading edge, the tail under the wing.
    st = [(PROP + 0.05, z - 0.02, 0.44, 0.46), (PROP + 0.6, z - 0.12, 0.52, 0.62), (PROP + 1.4, z - 0.2, 0.55, 0.78),
          (WING_LE - 0.2, z - 0.15, 0.55, 0.82), (WING_LE + 1.0, z - 0.05, 0.52, 0.80), (13.4, z + 0.05, 0.48, 0.66),
          (14.8, z + 0.25, 0.36, 0.42), (NACELLE_END, z + 0.45, 0.12, 0.10)]
    g.add(loft([(y_of(d), zc, hw, hh) for d, zc, hw, hh in st], GREY, n=12, x=x))
    # The intake's dark mouth under the spinner.
    g.add(frustum((x, y_of(PROP + 0.55), z - 0.55), (x, y_of(PROP + 0.75), z - 0.55), 0.22, 0.22, BLACK, 8, r0b=0.12,
                  r1b=0.12))
    # The exhaust on the nacelle's outboard flank.
    s = 1 if x > 0 else -1
    g.add(frustum((x + s * 0.45, y_of(12.6), z + 0.25), (x + s * 0.5, y_of(13.3), z + 0.25), 0.14, 0.14, BLACK, 6))
    return g


def propeller():
    """A propeller about its hub: the spinner and four blades (dark, a separate colour so the rotor
    blur finds the blades), turning about the fore-aft axis (part axis 2)."""
    g = Geo()
    g.add(loft([(0.75, 0.0, 0.02, 0.02), (0.55, 0.0, 0.22, 0.22), (0.25, 0.0, 0.38, 0.38), (-0.1, 0.0, 0.44, 0.44)],
               GREY_L, n=10))
    for k in range(4):
        a = math.pi / 4 + k * math.pi / 2
        u = Vector((math.cos(a), 0, math.sin(a)))
        side = Vector((-math.sin(a), 0, math.cos(a)))
        pts = []
        for r, c, twist in ((0.35, 0.30, 0.9), (1.2, 0.34, 0.6), (PROP_R - 0.15, 0.24, 0.3), (PROP_R, 0.12, 0.25)):
            ch = side * math.cos(twist) * c / 2 + Vector((0, 1, 0)) * math.sin(twist) * c / 2
            pts.append((u * r + ch, u * r - ch))
        for (a0, b0), (a1, b1) in zip(pts, pts[1:]):
            g.face2([a0, a1, b1, b0], BLACK)
    return g


def sponsons():
    """The main gear's sponsons along the lower fuselage, and the fairing over the wing's centre
    section on top."""
    g = Geo()
    for s in (-1, 1):
        st = [(8.1, -1.25, 0.05, 0.05), (8.35, -1.25, 0.35, 0.45), (8.9, -1.3, 0.55, 0.68), (9.8, -1.32, 0.66, 0.76),
              (15.2, -1.32, 0.66, 0.76), (16.2, -1.35, 0.55, 0.66), (16.9, -1.45, 0.3, 0.42), (17.3, -1.55, 0.05, 0.05)]
        g.add(loft([(y_of(d), z, hw, hh) for d, z, hw, hh in st], GREY, n=10, power=2.6, x=s * 2.02))
    st = [(9.4, 1.98, 0.05, 0.05), (10.0, 2.04, 0.9, 0.16), (11.0, 2.1, 1.25, 0.2), (14.5, 2.12, 1.3, 0.2),
          (16.4, 2.08, 1.1, 0.16), (17.4, 2.02, 0.3, 0.05)]
    g.add(loft([(y_of(d), z, hw, hh) for d, z, hw, hh in st], GREY, n=10))
    return g


# The fin: its leading edge from the dorsal fillet's top (22.1, 3.06) to the tip (25.95, 9.0),
# its trailing edge near upright (28.55 at the root, 28.3 at the tip); the dorsal fillet along the
# top from 17.3.
FIN_ROOT = (21.95, 2.0, 28.3, 2.0)
FIN_TIP = (25.55, 8.9, 27.95, 8.9)
FILLET = [(17.3, 2.04, 0.05, 0.02), (18.5, 2.12, 0.25, 0.1), (20.0, 2.3, 0.3, 0.25), (21.2, 2.45, 0.28, 0.42),
          (22.1, 2.55, 0.24, 0.52), (22.7, 2.6, 0.18, 0.55)]
# The tailplane: root and tip leading edges (station, x), chords; at the tail cone's sides.
TAIL_Z = 1.85
TAIL_ROOT = (23.85, 0.6, 4.15)
TAIL_TIP = (25.3, 7.75, 2.7)


def fin_section(d_le, z_le, d_te, z_te, t):
    le, te = Vector((0, y_of(d_le), z_le)), Vector((0, y_of(d_te), z_te))
    mid = le + (te - le) * 0.3
    w = Vector(((te - le).length * t / 2, 0, 0))
    return [le, mid + w, te, mid - w]


def tail():
    g = Geo()
    g.add(loft([(y_of(d), z, hw, hh) for d, z, hw, hh in FILLET], GREY, n=8))
    g.add(tube([fin_section(*FIN_ROOT, 0.13), fin_section(*FIN_TIP, 0.10)], GREY))
    # The fin's rounded top.
    d0, z0, d1, _ = FIN_TIP
    g.add(tube([fin_section(d0, z0, d1, z0, 0.10), fin_section(d0 + 0.5, z0 + 0.12, d1 - 0.15, z0 + 0.12, 0.08),
                fin_section(d0 + 1.1, z0 + 0.18, d1 - 0.5, z0 + 0.18, 0.05)], GREY))
    (dr, xr, cr), (dt, xt, ct) = TAIL_ROOT, TAIL_TIP
    g.add(wing((xr, y_of(dr), TAIL_Z), cr, (xt, y_of(dt), TAIL_Z + 0.05), ct, GREY, t=0.11, peak=0.3))
    tips = [(xt, dt, ct), (xt + 0.2, dt + 0.25, ct - 0.4), (xt + 0.32, dt + 0.7, ct - 1.3)]
    for (xa, la, ca), (xb, lb, cb) in zip(tips, tips[1:]):
        g.add(wing((xa, y_of(la), TAIL_Z + 0.05), ca, (xb, y_of(lb), TAIL_Z + 0.05), cb, GREY, t=0.09, peak=0.3))
    return g


# The guns on the left side: (name, muzzle's station, height, barrel's radius, length out of the
# skin, depression). The 40 mm Bofors just aft of the sponson, the 105 mm howitzer further aft.
GUNS = [("40mm", 17.7, -0.55, 0.055, 1.05, 20.0), ("105mm", 19.75, -0.15, 0.10, 1.25, 20.0)]
# The sensor turret under the left side of the nose, behind the nose gear.
SENSOR = (5.2, -1.95, -1.25)
SENSOR_R = 0.33


def gun_port(d, z):
    """The barrel's root on the left skin."""
    return Vector((-skin_x(d, z), y_of(d), z))


def gun_muzzle(name):
    """Where the barrel ends (model metres): the muzzle flash's and the shells' origin."""
    for n, d, z, r, length, dep in GUNS:
        if n == name:
            dirn = Vector((-math.cos(math.radians(dep)), 0, -math.sin(math.radians(dep))))
            return gun_port(d, z) + dirn * length
    raise KeyError(name)


def guns():
    g = Geo()
    for name, d, z, r, length, dep in GUNS:
        root = gun_port(d, z) + Vector((0.25, 0, 0))
        dirn = Vector((-math.cos(math.radians(dep)), 0, -math.sin(math.radians(dep))))
        tip = root + dirn * (length + 0.25)
        # The port's frame round the barrel, a dark opening, the barrel, its muzzle device.
        port = gun_port(d, z) + Vector((-0.01, 0, 0))
        w = r * 4.0
        g.face([port + Vector((0, w, w)), port + Vector((0, -w, w)), port + Vector((0, -w, -w)),
                port + Vector((0, w, -w))], BLACK, port + Vector((1.0, 0, 0)))
        g.add(frustum(root, tip, r, r * 0.9, DARK, 8))
        brake = 0.18 if name == "105mm" else 0.12
        g.add(frustum(tip - dirn * brake, tip, r * 1.6, r * 1.6, BLACK, 8))
        # The 40 mm's flash hider is a cone.
        if name == "40mm":
            g.add(frustum(tip - dirn * 0.25, tip - dirn * brake, r * 1.0, r * 1.5, DARK, 8, caps=False))
    # The sensor turret: a ball on a short neck.
    d, z, x = SENSOR
    c = Vector((x, y_of(d), z))
    g.add(frustum(c + Vector((0, 0, 0.45)), c + Vector((0, 0, 0.1)), 0.2, 0.2, GREY, 8))
    g.add(loft([(c.y + SENSOR_R, c.z, 0.02, 0.02), (c.y + SENSOR_R * 0.7, c.z, SENSOR_R * 0.7, SENSOR_R * 0.7),
                (c.y, c.z, SENSOR_R, SENSOR_R), (c.y - SENSOR_R * 0.7, c.z, SENSOR_R * 0.7, SENSOR_R * 0.7),
                (c.y - SENSOR_R, c.z, 0.02, 0.02)], GREY_L, n=10, x=c.x))
    # Its window, facing left and down.
    win = c + Vector((-0.22, 0, -0.22))
    n = Vector((-1, 0, -1)).normalized()
    side = Vector((0, 1, 0))
    up = n.cross(side).normalized()
    q = 0.14
    g.face([win + side * q + up * q, win - side * q + up * q, win - side * q - up * q, win + side * q - up * q],
           CANOPY, c)
    # A second, smaller ball (the low-light TV) aft of the crew door.
    c2 = Vector((-1.75, y_of(6.4), -1.55))
    g.add(loft([(c2.y + 0.22, c2.z, 0.02, 0.02), (c2.y, c2.z, 0.22, 0.22), (c2.y - 0.22, c2.z, 0.02, 0.02)], GREY_L,
               n=8, x=c2.x))
    return g


# The gear: the nose gear's twin wheels, the mains' tandem wheels in the sponsons.
NOSE_GEAR = 3.45
MAIN_AXLES = (12.5, 14.0)
MAIN_X = 2.1


def gear():
    g = Geo()
    az = GROUND + NOSE_R
    y = y_of(NOSE_GEAR)
    for x in (-0.28, 0.28):
        g.add(mklib.wheel((x, y, az), NOSE_R, 0.26, BLACK, n=10))
    g.add(frustum((0, y + 0.05, -1.9), (0, y, az), 0.09, 0.08, GREY_L, 6))
    g.add(frustum((-0.4, y, az), (0.4, y, az), 0.05, 0.05, GREY_L, 6))
    for s in (-1, 1):
        g.face2([(s * 0.45, y + 0.8, -2.0), (s * 0.45, y - 0.8, -2.0), (s * 0.47, y - 0.8, -2.45),
                 (s * 0.47, y + 0.8, -2.45)], GREY)
    az = GROUND + MAIN_R
    for s in (-1, 1):
        for d in MAIN_AXLES:
            g.add(mklib.wheel((s * MAIN_X, y_of(d), az), MAIN_R, 0.42, BLACK, n=12))
            g.add(frustum((s * MAIN_X, y_of(d), -1.2), (s * MAIN_X, y_of(d), az), 0.1, 0.09, GREY_L, 6))
    return g


def nav_tips():
    tip = (WING_TIP + 0.15, y_of(WING_LE + 1.2), wing_z(WING_TIP))
    return (-tip[0], tip[1], tip[2]), tip, [(0, y_of(29.55), 1.93)]


def build():
    """(body, gear, parts): parts are (geometry about its pivot, pivot, slot, axis)."""
    g = fuselage()
    g.add(seams())
    g.add(sponsons())
    g.add(wing_geo())
    for x in ENGINES:
        g.add(nacelle(x)).add(nacelle(-x))
    g.add(tail())
    g.add(guns())
    parts = []
    prop = propeller()
    for x in ENGINES:
        for s in (1, -1):
            parts.append((prop, (s * x, y_of(PROP), nacelle_z(x)), PROP_SLOT, 2))
    return g, gear(), parts
