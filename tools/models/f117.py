"""Builds the F-117A Nighthawk (`aircraft_024`) in aircraft.blend and saves it:

    blender -b assets-src/blend/aircraft.blend --python-exit-code 1 --python tools/models/f117.py
    tools/models/export.sh aircraft

Faceted like the real aircraft, from public three-view drawings: length 20.1 m, span 13 m,
3.8 m high with the gear down, the leading edge swept 67 degrees from the nose to the wingtips,
the trailing edge swept forward to the body, two canted tails and the exhaust deck between them.
A few large flat facets: the flat underside, the nose pyramid up to the windshield, the canopy's
panes, the upper deck and the flanks down to the wedge-section wings.

Children of the one detail level:
- `_s8v0` / `_s8v1` / `_s8v2`: the two weapon bays' doors shut, half open, and open with the bay
  inside and a GBU-27 in each (state slot 8, the bay door animation).
- `_s5v1`: the landing gear down (slot 5).
- `_s12v1` navigation lights on the wingtips, `_s20v1` the red beacons above and below.

Built in metres (x right, y forward, z up; the origin on the underside, 10.6 m behind the nose)
and scaled to aircraft units (K per metre). The wheels touch the ground 1.65 m below the underside
(`F117_LIFT` in src/aircraft.rs).
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
from mklib import Geo, Ramp, frustum, v3  # noqa: E402

NAME = "aircraft_024"
# Unused variants of the F-117 left over in the file.
OBSOLETE = ["aircraft_031", "aircraft_033"]
K = 16.384
MAX_DIST = 32769

BODY = Ramp(152, 159)
DARK = Ramp(155, 159)
WING = Ramp(154, 159)
GLASS = Ramp(144, 145)
BAY = Ramp(157, 159)
METAL = Ramp(0xE8, 0xEC)
BOMB = Ramp(0xA8, 0xAE)
EXHAUST = 13
TYRE = 0xEF
SEAM = 159
UNDER = 158  # the underside, and its panel lines
UNDER_SEAM = 154
PROBE = 154
RED, GREEN = 147, 151

NOSE = 10.6  # y of the nose tip; points are given aft of it (s)
GROUND = -1.65  # z of the ground under the wheels

# Points (x, s aft of the nose, z above the flat underside), right half. The underside of the
# body and the wings is one plane (z 0); the chine runs from the nose tip along the leading
# edge to the wingtip.
N = (0.0, 0.0, 0.0)  # nose tip
C1 = (1.43, 3.3, 0.0)  # chine beside the windshield
LE_TIP, TIP, TIP_TRAIL = (6.5, 15.0, 0.0), (6.5, 16.5, 0.0), (5.9, 17.2, 0.0)
TIP_RIDGE = (6.3, 16.0, 0.05)
# The body's base, where its flanks meet the wings' upper surface (the wing root, 0.3 m thick).
B1 = (2.17, 5.0, 0.0)  # where the leading edge meets the body
B2, B3, B4 = (2.7, 8.5, 0.3), (2.5, 12.0, 0.3), (2.75, 13.5, 0.3)  # B4: trailing edge root
B5, B6 = (0.95, 16.0, 0.3), (0.15, 17.1, 0.25)  # the exhaust deck's edge, the tail boom's end
END = (0.0, 17.1, 0.3)
# Cockpit: the canopy's panes fan out forward and down from its apex. Along its base: the
# windshield's (centre and corner), the front side pane's, the frame post's and the side pane's
# rear corner; then its rear edge.
APEX = (0.0, 5.1, 2.26)
W0, WS = (0.0, 3.1, 1.22), (0.14, 3.1, 1.22)
FS, FP = (0.52, 3.55, 1.14), (0.62, 3.75, 1.17)
CS, CR, PR = (0.95, 4.85, 1.34), (0.26, 6.0, 1.98), (0.0, 6.05, 2.02)
# The upper deck: spine (centre) and shoulders.
S2, S3, S4 = (0.0, 9.0, 1.75), (0.0, 13.8, 1.15), (0.0, 15.3, 0.80)
SH2, SH3, SH4 = (1.55, 9.0, 1.48), (1.55, 12.5, 1.15), (0.90, 15.0, 0.75)
SP2 = (0.45, 9.0, 1.71)  # the centre panel's edge, converging on S3

# Weapon bays: either side of the keel, doors hinged at the bay's sides.
BAY_X = (0.10, 0.95)
BAY_S = (6.5, 11.2)
BAY_DEPTH = 0.9
# Landing gear: nose wheel and main wheels (x, s), wheel radius and width.
NOSE_GEAR = (0.0, 5.0, 0.30, 0.18)
MAIN_GEAR = (1.60, 11.15, 0.36, 0.24)


def P(x, s, z):
    return Vector((x, NOSE - s, z))


def V(p):
    return P(*p)


def on(a, b, s):
    """The point of edge a-b (planform points) at station s."""
    a, b = V(a), V(b)
    t = (NOSE - s - a.y) / (b.y - a.y)
    return a.lerp(b, t)


def flat(p):
    return Vector((p.x, p.y, 0.0))


def window(tri, frame):
    """A window pane set into triangle `tri`, inside a frame `frame[i]` m wide along the edge
    from corner i to the next, lifted off the facet."""
    a, b, c = tri
    n = (b - a).cross(c - a).normalized()
    if n.dot(a - P(0, 9.0, 0.6)) < 0:
        n = -n
    # Each edge moved inward by its frame width; the pane's corners where they meet.
    def inward(p, q, r, w):
        d = (q - p).normalized()
        out = d.cross(n).normalized()
        if out.dot(r - p) < 0:
            out = -out
        return p + out * w, d
    edges = [inward(a, b, c, frame[0]), inward(b, c, a, frame[1]), inward(c, a, b, frame[2])]
    pts = []
    for (p0, d0), (p1, d1) in zip(edges[-1:] + edges[:-1], edges):
        # intersection of line p0 + t d0 and p1 + u d1 (in the facet's plane)
        w = p1 - p0
        cr = d0.cross(d1)
        t = w.cross(d1).dot(cr) / cr.length_squared
        pts.append(p0 + d0 * t + n * 0.012)
    return pts


def body():
    """Half the body (x >= 0), mirrored by the caller."""
    g = Geo()
    inside = P(0, 9.0, 0.6)
    face = lambda pts, paint: g.face([V(p) if isinstance(p, tuple) else p for p in pts], paint, inside)
    # Nose pyramid, windshield and canopy.
    face([N, W0, WS], BODY)
    face([N, WS, FS], BODY)
    face([N, FS, C1], BODY)
    face([C1, FS, FP, CS, B1], BODY)
    # The canopy: its frame, the narrow windshield (one pane across the centre), a front side
    # pane and a side pane either side split by a wider post; solid behind the apex.
    face([W0, WS, APEX], DARK)
    face([WS, FS, APEX], DARK)
    face([FS, FP, APEX], DARK)
    face([FP, CS, APEX], DARK)
    face([APEX, CS, CR], DARK)
    face([APEX, CR, PR], DARK)
    for pane, frame in (([W0, WS, APEX], (0.06, 0.04, 0.0)),
                        ([WS, FS, APEX], (0.06, 0.05, 0.04)),
                        ([FP, CS, APEX], (0.06, 0.07, 0.05))):
        g.face(window([V(p) for p in pane], frame), GLASS, inside)
    # The upper deck and the flanks down to the wing roots.
    face([PR, CR, SP2, S2], DARK)
    face([CR, CS, SH2, SP2], BODY)
    face([S2, SP2, S3], DARK)
    face([SP2, SH2, SH3, S3], BODY)
    face([S3, SH3, SH4, S4], BODY)
    face([CS, B1, B2, SH2], BODY)
    face([SH2, B2, B3, SH3], BODY)
    face([SH3, B3, B4, SH4], BODY)
    # The exhaust deck sloping down to its edge, the slot along the edge.
    face([SH4, B4, B5], DARK)
    face([S4, SH4, B5, B6, END], DARK)
    b4, b5, sh4 = V(B4), V(B5), V(SH4)
    up = (b5 - b4).cross(sh4 - b4).normalized() * 0.012
    if up.z < 0:
        up = -up
    q = [b4 + up, b5 + up, b5.lerp(sh4, 0.4) + up, b4.lerp(sh4, 0.4) + up]
    g.face(q, EXHAUST, inside)
    for k in range(1, 9):
        g.line(q[0].lerp(q[1], k / 9) + up, q[3].lerp(q[2], k / 9) + up, DARK.hi)
    g.line(q[0].lerp(q[3], 0.5) + up, q[1].lerp(q[2], 0.5) + up, DARK.hi)
    # The underside, open over the weapon bays (their doors are in `bay_doors`).
    (x0, x1), (s0, s1) = BAY_X, BAY_S
    down = P(0, 9.0, 1.0)
    under = lambda pts: g.face([flat(V(p)) if isinstance(p, tuple) else flat(p) for p in pts], UNDER, down)
    under([N, C1, B1, on(B1, B2, s0), (0, s0, 0)])
    under([(0, s0, 0), (x0, s0, 0), (x0, s1, 0), (0, s1, 0)])
    under([(x1, s0, 0), on(B1, B2, s0), B2, on(B2, B3, s1), (x1, s1, 0)])
    under([(0, s1, 0), on(B2, B3, s1), B3, B4, B5, B6, (0.0, 17.1, 0.0)])
    # Panel lines underneath: the keel, the gear doors, the access panels.
    seam = lambda a, b: g.line(flat(V(a)) - Vector((0, 0, 0.006)), flat(V(b)) - Vector((0, 0, 0.006)), UNDER_SEAM)
    seam((0, 1.5, 0), (0, s0, 0))
    seam((0, s1, 0), (0, 16.8, 0))
    for a, b, c, d in (((0, 4.1, 0), (0.22, 4.1, 0), (0.22, 5.6, 0), (0, 5.6, 0)),
                       ((1.25, 10.3, 0), (1.95, 10.3, 0), (1.95, 12.0, 0), (1.25, 12.0, 0)),
                       ((0.5, 2.6, 0), (0.9, 2.6, 0), (0.9, 3.4, 0), (0.5, 3.4, 0)),
                       ((1.1, 12.6, 0), (1.9, 12.6, 0), (1.6, 14.4, 0), (0.6, 14.4, 0))):
        for e, f in ((a, b), (b, c), (c, d), (d, a)):
            seam(e, f)
    # The trailing edges' thickness and the boom's end.
    for a, b in ((B4, B5), (B5, B6), (B6, END)):
        face([a, b, flat(V(b)), flat(V(a))], DARK)
    # The canopy's rear edge, the intake grid on the forward flank.
    lift = Vector((0, 0, 0.012))
    for a, b in ((CS, CR), (CR, PR)):
        g.line(V(a) + lift, V(b) + lift, SEAM)
    q = [V(CS).lerp(V(B1), 0.25), V(CS).lerp(V(B1), 0.75)]
    q = [q[0], q[1], q[1].lerp(V(B2), 0.3) + (V(SH2) - V(CS)) * 0.15, q[0].lerp(V(SH2), 0.3)]
    out = (q[1] - q[0]).cross(q[3] - q[0]).normalized() * 0.015
    if out.dot(q[0] - inside) < 0:
        out = -out
    g.face([p + out for p in q], DARK.hi, inside)
    for k in range(1, 6):
        g.line(q[0].lerp(q[1], k / 6) + out * 2, q[3].lerp(q[2], k / 6) + out * 2, BODY.lo)
    for k in range(1, 4):
        g.line(q[0].lerp(q[3], k / 4) + out * 2, q[1].lerp(q[2], k / 4) + out * 2, BODY.lo)
    # Air data probes at the nose.
    tip = V(N)
    for dx, dz in ((0.0, 0.03), (0.16, 0.0), (-0.16, 0.0), (0.0, 0.12)):
        g.line(tip + Vector((dx * 0.3, 0, dz * 0.3)), tip + Vector((dx, 0.9, dz)), PROBE)
    return g


def wing():
    """The right wing: a wedge from the 0.3 m root down to the sharp leading and trailing edges."""
    g = Geo()
    below = P(4.0, 12.0, -1.0)
    above = P(4.0, 12.0, 1.0)
    pts = lambda l: [V(p) for p in l]
    g.face(pts([B1, LE_TIP, TIP_RIDGE, B2]), WING, below)
    g.face(pts([B2, TIP_RIDGE, TIP_TRAIL, B4, B3]), WING, below)
    g.face(pts([LE_TIP, TIP, TIP_TRAIL, TIP_RIDGE]), WING, below)
    g.face([flat(p) for p in pts([B1, LE_TIP, TIP, TIP_TRAIL, B4, B3, B2])], UNDER, above)
    for a, b in ((LE_TIP, (5.2, 15.6, 0)), ((3.0, 9.3, 0), (5.4, 15.7, 0))):
        g.line(flat(V(a)) - Vector((0, 0, 0.006)), flat(V(b)) - Vector((0, 0, 0.006)), UNDER_SEAM)
    g.face([V(B4), V(TIP_TRAIL), flat(V(B4))], DARK, P(4.0, 12.0, 0.1))
    # Elevon hinge lines.
    for t in (0.18, 0.55):
        a = V(B4).lerp(V(TIP_TRAIL), t) + Vector((0, 0.9, 0.0))
        b = V(B4).lerp(V(TIP_TRAIL), t + 0.33) + Vector((0, 0.9, 0.0))
        g.line(a + Vector((0, 0, 0.27 * (1 - t) + 0.01)), b + Vector((0, 0, 0.27 * (0.67 - t) + 0.02)), SEAM)
    return g


def tail():
    """The right tail, canted out about 45 degrees, thin with a ridge along 40% of its chord."""
    g = Geo()
    root_le, root_te = P(0.45, 15.0, 0.80), P(0.35, 17.6, 0.30)
    tip_le, tip_te = P(1.75, 19.3, 2.15), P(1.75, 20.3, 2.20)
    span = ((tip_le + tip_te) / 2 - (root_le + root_te) / 2).normalized()
    chord = (root_te - root_le).normalized()
    n = span.cross(chord).normalized()
    if n.x < 0:
        n = -n
    ridge_root, ridge_tip = root_le.lerp(root_te, 0.4), tip_le.lerp(tip_te, 0.4)
    for side in (n, -n):
        rr, rt = ridge_root + side * 0.08, ridge_tip + side * 0.04
        centre = (root_le + tip_te) / 2 - side
        g.face([root_le, rr, rt, tip_le], BODY, centre)
        g.face([rr, root_te, tip_te, rt], BODY, centre)
    g.face([tip_le, ridge_tip + n * 0.04, tip_te, ridge_tip - n * 0.04], DARK, (root_le + root_te) / 2)
    g.line(root_le.lerp(root_te, 0.62) + n * 0.09, tip_le.lerp(tip_te, 0.62) + n * 0.05, SEAM)
    return g


def door(hinge0, hinge1, width, angle, out):
    """A bay door hinged along `hinge0`-`hinge1` (on the belly), `width` m wide, folded down
    `angle` degrees from the belly towards `out` (+1: away from the keel)."""
    a = math.radians(angle)
    d = Vector((out * math.cos(a) * width, 0, -math.sin(a) * width))
    g = Geo()
    g.face2([hinge0, hinge1, hinge1 + d, hinge0 + d], BODY)
    return g


def bay_doors(state):
    """Both bays' doors at `state` 0 shut, 1 half open, 2 open, the bays' inside with 2."""
    g = Geo()
    (x0, x1), (s0, s1) = BAY_X, BAY_S
    half = (x1 - x0) / 2
    angle = (0, 45, 90)[state]
    for sx in (-1, 1):
        inner = (P(sx * x0, s0, 0), P(sx * x0, s1, 0))
        outer = (P(sx * x1, s0, 0), P(sx * x1, s1, 0))
        if state == 0:
            g.face([inner[0], inner[1], outer[1], outer[0]], UNDER, P(sx * 0.5, 8.0, 1.0))
            mid0, mid1 = P(sx * (x0 + half), s0, -0.005), P(sx * (x0 + half), s1, -0.005)
            for p, q in ((inner[0], inner[1]), (outer[0], outer[1]), (inner[0], outer[0]), (inner[1], outer[1]), (mid0, mid1)):
                g.line(p + Vector((0, 0, -0.006)), q + Vector((0, 0, -0.006)), UNDER_SEAM)
            continue
        g.add(door(inner[0], inner[1], half, angle, -sx))
        g.add(door(outer[0], outer[1], half, angle, sx))
        if state == 2:
            g.add(bay_inside(sx))
    return g


def bay_inside(sx):
    """A bay's walls and roof, seen from below, and its bomb."""
    g = Geo()
    (x0, x1), (s0, s1) = BAY_X, BAY_S
    lo = [P(sx * x0, s0, 0), P(sx * x1, s0, 0), P(sx * x1, s1, 0), P(sx * x0, s1, 0)]
    hi = [p + Vector((0, 0, BAY_DEPTH)) for p in lo]
    inside = P(sx * (x0 + x1) / 2, (s0 + s1) / 2, BAY_DEPTH / 2)
    outside = lambda f: (sum(f, Vector()) / len(f)) * 2 - inside
    walls = [[lo[i], lo[(i + 1) % 4], hi[(i + 1) % 4], hi[i]] for i in range(4)] + [hi]
    for f in walls:
        g.face(f, BAY, outside(f))
    # GBU-27: a 2,000 lb bomb with a laser seeker and fins, hung from the bay's roof.
    x, z = sx * (x0 + x1) / 2, BAY_DEPTH - 0.32
    y0, y1 = NOSE - s0 - 0.25, NOSE - s1 + 0.3
    body = [(y0, 0.04), (y0 - 0.35, 0.16), (y0 - 0.9, 0.19), (y1 + 0.6, 0.19), (y1 + 0.15, 0.12), (y1, 0.06)]
    for (ya, ra), (yb, rb) in zip(body, body[1:]):
        g.add(frustum((x, ya, z), (x, yb, z), ra, rb, BOMB, n=6, caps=False))
    for a in (45, 135, 225, 315):
        d = Vector((math.cos(math.radians(a)), 0, math.sin(math.radians(a))))
        g.face2([Vector((x, y1 + 0.55, z)) + d * 0.16, Vector((x, y1 + 0.05, z)) + d * 0.16,
                 Vector((x, y1 + 0.15, z)) + d * 0.36], METAL)
    return g


def gear():
    """The gear down: legs, wheels and the open gear doors."""
    g = Geo()
    x, s, r, w = NOSE_GEAR
    axle = P(x, s, GROUND + r)
    g.add(mklib.wheel(tuple(axle), r, w, TYRE, n=8))
    g.add(frustum(tuple(P(x, s - 0.25, 0.0)), tuple(axle + Vector((0, 0.05, 0.1))), 0.07, 0.06, METAL, n=5))
    for sx in (-1, 1):
        g.face2([P(sx * 0.20, s - 0.9, 0), P(sx * 0.20, s + 0.3, 0), P(sx * 0.22, s + 0.3, -0.45), P(sx * 0.22, s - 0.9, -0.45)], BODY)
    x, s, r, w = MAIN_GEAR
    for sx in (-1, 1):
        axle = P(sx * x, s, GROUND + r)
        g.add(mklib.wheel(tuple(axle), r, w, TYRE, n=8))
        g.add(frustum(tuple(P(sx * (x - 0.1), s - 0.3, 0.0)), tuple(axle + Vector((-sx * 0.12, 0, 0.12))), 0.09, 0.08, METAL, n=5))
        g.add(frustum(tuple(P(sx * (x - 0.1), s + 0.6, 0.0)), tuple(axle + Vector((-sx * 0.12, 0, 0.2))), 0.05, 0.05, METAL, n=4))
        g.face2([P(sx * (x + 0.32), s - 0.7, 0), P(sx * (x + 0.32), s + 0.8, 0), P(sx * (x + 0.36), s + 0.8, -0.6), P(sx * (x + 0.36), s - 0.7, -0.6)], BODY)
    return g


def whole(geo_half):
    return Geo().add(geo_half).add(geo_half.mirrored())


def group(parent, slot, value, geo):
    name = f"{parent.name}_s{slot}v{value}"
    ob = bpy.data.objects.new(name, None)
    ob["conds"] = [[slot, value]]
    for col in parent.users_collection:
        col.objects.link(ob)
    ob.parent = parent
    if geo.faces:
        heli.mesh_with_lines(ob, f"{name}_body", geo.scaled(K))
    else:
        for c, segs in sorted(geo.scaled(K).lines.items()):
            ln = f"{name}_lines_pal_{c:03}"
            mklib.link_child(ob, ln, mklib.lines_mesh(ln, segs, c))
    return ob


def build():
    for name in OBSOLETE:
        if name in bpy.data.objects:
            mklib.delete_tree(bpy.data.objects[name])
    root = bpy.data.objects[NAME]
    for c in list(root.children):
        mklib.delete_tree(c)
    root["attr"] = 0x84
    root["lod_dists"] = [MAX_DIST]
    shell = whole(body()).add(whole(wing())).add(whole(tail()))
    geo = shell.scaled(K)
    lod = heli.mesh_with_lines(root, f"{NAME}_lod0", geo)
    lod["max_dist"] = MAX_DIST
    lod["radius"] = heli.radius(geo, gear().scaled(K))
    # Aircraft are never tested against their hull: one plane, like the other generated models.
    lod["planes"] = [[0, 16384, 0, 0]]
    for state in range(3):
        group(lod, 8, state, bay_doors(state))
    group(lod, 5, 1, gear())
    wingtip = lambda sx: [v3(P(sx * (TIP[0] - 0.1), 15.9, 0.06)) * K]
    heli.lights(lod, 12, {RED: wingtip(-1), GREEN: wingtip(1)})
    heli.lights(lod, 20, {RED: [v3(P(0, 10.0, 1.67)) * K, v3(P(0, 12.0, -0.02)) * K]})
    print(f"{lod.name}: {len(geo.faces)} faces, radius {lod['radius']}")


mklib.load_palette(os.path.join(ROOT, "assets", "palettes", "tod_central_europe.ron"))
build()
bpy.ops.wm.save_mainfile()
