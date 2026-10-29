"""Builds the remake's own aircraft models (Geran-2 / Geran-3 attack drones, the Mi-24 Hind
from mi24.py, the drone command center's FP-2 attack drone and Stork-100 surveillance drone) in
Blender:

    blender -b --factory-startup --python-exit-code 1 --python tools/models/drones.py -- assets/models/drones.glb

Nodes follow aircraft.glb's conventions: a root `drones_NNN` with `attr`, a `_lod0` mesh and a
spinning `_part1` (state slot 17; axis 2 = fore-aft, the Geran-2's pusher propeller). Model
space: x right, y forward, z up, aircraft model units (WU / 8 m).
"""

import math
import os
import sys

import bpy

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.abspath(os.path.join(HERE, "..", ".."))
sys.path.insert(0, HERE)

import mi24  # noqa: E402
import mklib  # noqa: E402
from mklib import Geo, Ramp, frustum, loft  # noqa: E402

# Model numbers (kept clear of aircraft.glb's 0..44).
GERAN2 = 100
GERAN3 = 101
# The Mi-24 Hind (tools/models/mi24.py).
HIND = 102
# The drone command center's FP-2 and Stork-100 (crates/stealth-sim/src/uav.rs).
FP2 = 103
STORK = 104

# Model units per metre (a unit is WU / 8).
K = 8.0 * 1_048_576 / 512_000

# Dark grey/black paint of the recent Gerans, a lighter grey belly.
BODY = Ramp(0xE9, 0xEF)
WING = Ramp(0xE8, 0xEE)
BELLY = Ramp(0xE4, 0xEA)
PROP = 0xEF
NOZZLE = Ramp(0xEB, 0xEF)
# Nav-light colours used by aircraft (red / green / white).
RED, GREEN, WHITE = 147, 149, 151


def empty(name, parent=None, loc=(0, 0, 0)):
    ob = bpy.data.objects.new(name, None)
    bpy.context.scene.collection.objects.link(ob)
    ob.parent = parent
    ob.location = loc
    return ob


def mesh_obj(name, g, parent, loc=(0, 0, 0)):
    ob = bpy.data.objects.new(name, mklib.faces_mesh(name, g))
    bpy.context.scene.collection.objects.link(ob)
    ob.parent = parent
    ob.location = loc
    return ob


def delta_wing(span, root_le, root_chord, tip_le_y, tip_chord, z, t):
    """Mirrored cropped-delta wing: root leading edge at (0, root_le), tip at +-span/2."""
    return mklib.wing((0, root_le, z), root_chord, (span / 2, tip_le_y, z), tip_chord, WING, t=t, t_tip=0.05, peak=0.4)


def tip_fins(span, y, chord, up, down, z):
    """Fins at the wing tips, reaching above and below the wing."""
    g = Geo()
    for sx in (-1, 1):
        x = sx * span / 2
        g.face2([(x, y - chord * 0.1, z), (x, y - chord, z), (x, y - chord, z + up), (x, y - chord * 0.45, z + up)], WING)
        g.face2([(x, y - chord * 0.3, z), (x, y - chord, z), (x, y - chord, z - down), (x, y - chord * 0.6, z - down)], WING)
    return g


# Proportions from photos of the Shahed-136 (3.5 m long, 2.5 m span, about 0.45 m across the
# nose): a round nose ahead of the wing apex, then a blended wing-body as thick as the wing root.
LENGTH_NOSE, TAIL = 36, -30
SPAN = 48
APEX = 19
TIP_LE, TIP_CHORD = -19, 10


def airframe(nose=LENGTH_NOSE):
    g = Geo()
    # Nose: a tube of constant diameter with a rounded tip, running straight into the hull;
    # behind the wing apex it flattens into the low spine (as thick as the wing root).
    r = 4.2
    cap = [(nose - r * (1 - math.cos(a)), 0, max(r * math.sin(a), 0.5), max(r * math.sin(a), 0.5)) for a in (0.0, 0.5, 0.95, 1.3, math.pi / 2)]
    g.add(loft(cap + [(APEX - 2, 0, r, r), (APEX - 12, 0, 3.6, 2.9), (-12, 0, 2.8, 2.5), (TAIL + 2, 0, 2.2, 2.0), (TAIL, 0, 1.6, 1.4)], BODY, n=10))
    g.add(delta_wing(SPAN, APEX, APEX - TAIL, TIP_LE, TIP_CHORD, 0, t=0.1))
    g.add(tip_fins(SPAN, TIP_LE, TIP_CHORD, 8, 4.5, 0))
    g.line((-SPAN / 2, TIP_LE - TIP_CHORD, 0), (-SPAN / 2, TIP_LE, 0), RED)
    g.line((SPAN / 2, TIP_LE - TIP_CHORD, 0), (SPAN / 2, TIP_LE, 0), GREEN)
    return g


def geran2():
    """Shahed-136 / Geran-2: piston engine in the tail with a pusher propeller."""
    g = airframe()
    g.add(frustum((0, TAIL + 1, 0), (0, TAIL - 3, 0), 2.2, 1.6, BELLY, n=8))
    prop = Geo()
    # Two-bladed pusher propeller around its hub (the part origin), about 0.8 m across.
    for a in (0, math.pi):
        c, s = math.cos(a), math.sin(a)
        prop.face2([(c * 1.0, 0, s * 1.0), (c * 7, 0.4, s * 7 + 1.4 * c), (c * 7, -0.4, s * 7 - 1.4 * c), (c * 1.0, 0.3, -s * 1.0)], PROP)
    return g, prop, (0, TAIL - 4, 0)


def geran3():
    """Shahed-238 / Geran-3: the same airframe with a small turbojet on the back of the tail."""
    g = airframe(LENGTH_NOSE + 2)
    g.add(loft([(-8, 2.2, 0.4, 0.4), (-10, 2.2, 1.9, 1.9), (-26, 2.2, 2.1, 2.1), (TAIL - 3, 2.2, 1.6, 1.6)], NOZZLE, n=8))
    return g, None, None


# ------------------------------------------------------------------------------------------ FP-2

FP2_GREY = Ramp(0xE7, 0xED)
CARBON = Ramp(0xEC, 0xEF)
WOOD = Ramp(0xC2, 0xC6)


def fp2():
    """FP-2: a 4.2 m straight high wing on a boxy pod with a rounded nose, the two-stroke
    engine and its exhaust pipes on top of the pod's tail, a pusher propeller, and two carbon
    booms to the tailplane (metres; scaled to model units)."""
    g = Geo()
    pod = [(1.05, 0.0, 0.04, 0.04), (1.0, 0.0, 0.14, 0.12), (0.9, 0.0, 0.2, 0.18), (0.6, 0.0, 0.21, 0.2),
           (-0.4, 0.0, 0.21, 0.2), (-0.65, 0.03, 0.17, 0.16), (-0.8, 0.05, 0.1, 0.1)]
    g.add(loft(pod, FP2_GREY, n=12, power=3.0))
    # The hatch hump on the pod ahead of the wing.
    g.add(loft([(0.75, 0.17, 0.02, 0.01), (0.65, 0.19, 0.12, 0.05), (0.3, 0.2, 0.14, 0.06), (0.15, 0.18, 0.1, 0.02)], FP2_GREY, n=8, power=3.0))
    g.add(mklib.wing((0, 0.25, 0.21), 0.46, (2.1, 0.23, 0.22), 0.42, FP2_GREY, t=0.12, t_tip=0.1, peak=0.3))
    # The engine behind the wing, the two exhaust pipes along its top.
    g.add(mklib.block(-0.12, 0.12, -0.62, -0.32, 0.12, 0.32, CARBON))
    for x in (-0.07, 0.07):
        g.add(frustum((x, 0.05, 0.3), (x, -0.75, 0.3), 0.035, 0.035, Ramp(0xE4, 0xEA), n=6))
    g.add(frustum((0, -0.62, 0.2), (0, -0.82, 0.2), 0.08, 0.04, CARBON, n=8))
    # The booms from the wing to the tailplane, its small fins under the boom ends.
    for x in (-0.55, 0.55):
        g.add(frustum((x, 0.15, 0.17), (x, -2.0, 0.17), 0.025, 0.025, CARBON, n=6))
    g.add(mklib.wing((0, -1.7, 0.17), 0.3, (0.66, -1.7, 0.17), 0.3, FP2_GREY, t=0.08, peak=0.3))
    for x in (-0.55, 0.55):
        g.face2([(x, -1.72, 0.15), (x, -2.0, 0.15), (x, -2.0, -0.05), (x, -1.85, -0.05)], FP2_GREY)
    g.line((-2.1, 0.21, 0.22), (-2.1, -0.19, 0.22), RED)
    g.line((2.1, 0.21, 0.22), (2.1, -0.19, 0.22), GREEN)
    prop = Geo()
    # A wooden two-bladed pusher propeller, about 0.8 m across.
    for a in (0, math.pi):
        c, s = math.cos(a), math.sin(a)
        prop.face2([(c * 0.04, 0, s * 0.04), (c * 0.4, 0.02, s * 0.4 + 0.05 * c), (c * 0.4, -0.02, s * 0.4 - 0.05 * c), (c * 0.04, 0.02, -s * 0.04)], WOOD)
    return g.scaled(K), prop.scaled(K), (0, -0.84 * K, 0.2 * K)


# ------------------------------------------------------------------------------------- Stork-100

STORK_GREY = Ramp(0xE1, 0xE7)


def stork():
    """Stork-100: a 2 m span, 1.3 m long pale grey airframe, the camera ball under the nose, a
    high wing with rounded tips, a conventional tail and a pusher propeller."""
    g = Geo()
    body = [(0.58, 0.0, 0.015, 0.015), (0.54, 0.0, 0.05, 0.05), (0.45, 0.0, 0.07, 0.075), (0.1, 0.01, 0.075, 0.08),
            (-0.2, 0.02, 0.06, 0.06), (-0.55, 0.035, 0.025, 0.03), (-0.62, 0.04, 0.015, 0.015)]
    g.add(loft(body, STORK_GREY, n=10))
    g.add(frustum((0, 0.4, -0.06), (0, 0.4, -0.1), 0.035, 0.04, Ramp(0xEB, 0xEF), n=8))
    g.add(mklib.wing((0, 0.14, 0.07), 0.24, (0.85, 0.12, 0.075), 0.2, STORK_GREY, t=0.12, peak=0.3))
    # The rounded tips.
    g.add(mklib.wing((0.85, 0.12, 0.075), 0.2, (1.0, 0.07, 0.075), 0.1, STORK_GREY, t=0.1, peak=0.3))
    g.add(mklib.wing((0, -0.5, 0.04), 0.13, (0.26, -0.53, 0.04), 0.09, STORK_GREY, t=0.08, peak=0.3))
    g.face2([(0, -0.47, 0.05), (0, -0.63, 0.05), (0, -0.66, 0.2), (0, -0.58, 0.2)], STORK_GREY)
    g.line((-1.0, 0.07, 0.075), (-1.0, -0.03, 0.075), RED)
    g.line((1.0, 0.07, 0.075), (1.0, -0.03, 0.075), GREEN)
    prop = Geo()
    for a in (0, math.pi):
        c, s = math.cos(a), math.sin(a)
        prop.face2([(c * 0.02, 0, s * 0.02), (c * 0.16, 0.01, s * 0.16 + 0.02 * c), (c * 0.16, -0.01, s * 0.16 - 0.02 * c), (c * 0.02, 0.01, -s * 0.02)], PROP)
    return g.scaled(K), prop.scaled(K), (0, -0.66 * K, 0.04 * K)


def model(index, geo, prop=None, prop_at=None):
    name = "drones_%03d" % index
    root = empty(name)
    root["attr"] = 0x81
    root["lod_dists"] = [32769]
    lod = mesh_obj(name + "_lod0", geo, root)
    pts = [p for f, _ in geo.faces for p in f]
    lod["max_dist"] = 32769
    lod["radius"] = int(math.ceil(max(p.length for p in pts)))
    for c, segs in sorted(geo.lines.items()):
        ln = "%s_lod0_lines_pal_%03d" % (name, c)
        lo = bpy.data.objects.new(ln, mklib.lines_mesh(ln, segs, c))
        bpy.context.scene.collection.objects.link(lo)
        lo.parent = lod
    if prop is not None:
        part = mesh_obj(name + "_lod0_part1", prop, lod, prop_at)
        part["state_slot"] = 17
        part["axis"] = 2


def main():
    out = sys.argv[sys.argv.index("--") + 1]
    mklib.load_palette(os.path.join(ROOT, "assets", "palettes", "tod_central_europe.ron"))
    for o in list(bpy.data.objects):
        bpy.data.objects.remove(o, do_unlink=True)
    model(GERAN2, *geran2())
    g3, _, _ = geran3()
    model(GERAN3, g3)
    mi24.build("drones_%03d" % HIND)
    model(FP2, *fp2())
    model(STORK, *stork())
    bpy.ops.export_scene.gltf(
        filepath=out, export_format="GLB", export_extras=True, export_yup=True, export_texcoords=False,
        export_normals=False, export_materials="EXPORT", use_visible=False, export_animations=False,
        use_mesh_edges=True, use_mesh_vertices=True,
    )


main()
