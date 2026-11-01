"""Checks that the stores the flyable jets hang (assets/models/stores.json, written by
aircraft.py) clear the airframe, each other and the ground:

    blender -b assets-src/blend/aircraft.blend --python-exit-code 1 --python tools/models/store_clearance.py

For every jet, every station and every load it takes (its rack and all its rounds), the posed
meshes must not cut into the airframe (its body, parts and pylons; the Tornado's at full forward
and full aft sweep), nor into the gear when down, nor reach below the ground under the wheels,
and no two stations' loads may cut into each other (but those never loaded together). Prints each failure and exits
non-zero if there are any.
"""

import itertools
import json
import math
import os
import sys

import bpy
from mathutils import Matrix, Vector
from mathutils.bvhtree import BVHTree

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.abspath(os.path.join(HERE, "..", ".."))
sys.path.insert(0, HERE)

import aircraft  # noqa: E402

K = aircraft.K
# The ground under the wheels (m below the origin), the Harrier's along its nose-up attitude.
GROUND = {23: lambda y: aircraft.F16_GROUND, 32: lambda y: aircraft.A10_GROUND, 45: lambda y: aircraft.TORNADO_GROUND,
          47: lambda y: aircraft.harrier_ground(y)}
# Full aft sweep of the Tornado's outer wings (degrees from fully forward).
SWEEP = 42.0
# The least gap (m) between a store and the ground under the wheels.
GROUND_GAP = 0.10


def mesh_tris(ob, m):
    """Triangles of mesh object `ob` (not its children) under matrix `m`, in model units."""
    me = ob.data
    out = []
    for p in me.polygons:
        vs = [m @ me.vertices[i].co for i in p.vertices]
        for k in range(1, len(vs) - 1):
            out.append((vs[0], vs[k], vs[k + 1]))
    return out


def tree(tris):
    verts, polys = [], []
    for t in tris:
        polys.append([len(verts), len(verts) + 1, len(verts) + 2])
        verts.extend(t)
    return BVHTree.FromPolygons(verts, polys) if polys else None


def local(ob):
    return Matrix.LocRotScale(ob.location, ob.rotation_euler.to_quaternion(), ob.scale)


def walk(ob, m, sweep, skip_conds=True):
    """(node name, matrix) of `ob` and its descendants under `m`; parts turned for `sweep`
    (degrees aft); conditional groups left out (`skip_conds`)."""
    mm = m @ local(ob)
    if "state_slot" in ob and ob["state_slot"] in (21, 22):
        sign = -1.0 if ob["state_slot"] == 21 else 1.0
        mm = mm @ Matrix.Rotation(math.radians(sign * sweep), 4, "Z")
    yield ob, mm
    for c in ob.children:
        if skip_conds and "conds" in c:
            continue
        yield from walk(c, mm, sweep, skip_conds)


def store_tris(index, m):
    lod = bpy.data.objects[f"aircraft_{index:03}_lod0"]
    return [t for ob, mm in walk(lod, m, 0.0) if ob.type == "MESH" and "_lines_" not in ob.name and "conds" not in ob
            for t in mesh_tris(ob, mm)]


def airframe(index, sweep):
    """The airframe's triangles (body, parts, pylons), its gear's, and the frames of its nodes."""
    lod = bpy.data.objects[f"aircraft_{index:03}_lod0"]
    frames = {}
    body = []
    for ob, mm in walk(lod, Matrix.Identity(4), sweep):
        frames[ob.name[len(lod.name):]] = mm
        if ob.type == "MESH" and "_lines_" not in ob.name and "_dots_" not in ob.name and "_exhaust" not in ob.name:
            body.extend(mesh_tris(ob, mm))
    gear = []
    g = bpy.data.objects.get(f"{lod.name}_s5v1")
    if g is not None:
        for ob, mm in walk(g, local(g), sweep, skip_conds=False):
            if ob.type == "MESH":
                gear.extend(mesh_tris(ob, mm))
    return body, gear, frames


def load_tris(table, jet, frames, st, w):
    """The triangles of station `st`'s load `w` (rack and rounds) on jet model `jet`."""
    s = next(x for x in table["jets"][str(jet)] if x["station"] == st)
    f = frames[s["parent"]]
    load = s["loads"][str(w)]
    tris = []
    if load["rack"] is not None:
        m = f @ Matrix.Translation(Vector(s["at"])) @ Matrix.Rotation(math.radians(s["roll"]), 4, "Y")
        tris += store_tris(load["rack"], m)
    for x, y, z, roll in load["rounds"]:
        m = f @ Matrix.Translation(Vector((x, y, z))) @ Matrix.Rotation(math.radians(roll), 4, "Y")
        tris += store_tris(table["models"][str(w)], m)
    return tris


def rounds_only(table, jet, frames, st, w, inset=0.0):
    """The rounds of station `st`'s load `w`, `inset` m away from their lugs (so a round's lugs
    touching its pylon or rack do not count)."""
    s = next(x for x in table["jets"][str(jet)] if x["station"] == st)
    f = frames[s["parent"]]
    out = []
    for x, y, z, roll in s["loads"][str(w)]["rounds"]:
        m = (f @ Matrix.Translation(Vector((x, y, z))) @ Matrix.Rotation(math.radians(roll), 4, "Y")
             @ Matrix.Translation(Vector((0, 0, -inset * K))))
        out += store_tris(table["models"][str(w)], m)
    return out


def check(table):
    failures = []
    for jet in sorted(int(j) for j in table["jets"]):
        sweeps = [0.0, SWEEP] if jet == 45 else [0.0]
        stations = table["jets"][str(jet)]
        for sweep in sweeps:
            body, gear, frames = airframe(jet, sweep)
            body_t, gear_t = tree(body), tree(gear)
            for s in stations:
                for w in s["loads"]:
                    w = int(w)
                    rtree = tree(rounds_only(table, jet, frames, s["station"], w, inset=0.012))
                    lt = load_tris(table, jet, frames, s["station"], w)
                    name = f"aircraft_{jet:03} sweep {sweep:.0f} station {s['station'] + 1} weapon {w}"
                    if body_t.overlap(rtree):
                        failures.append(f"{name}: into the airframe")
                    if gear_t is not None and gear_t.overlap(tree(lt)):
                        failures.append(f"{name}: into the gear")
                    gap = min(v.z / K - GROUND[jet](v.y / K) for t in lt for v in t)
                    if gap < GROUND_GAP:
                        failures.append(f"{name}: {gap:.2f} m off the ground")
            for a, b in itertools.combinations(stations, 2):
                wa, wb = (frames[x["parent"]] @ Vector(x["at"]) for x in (a, b))
                if b["station"] in a["excludes"] or abs(wa.x - wb.x) > 1.6 * K:
                    continue
                for wa, wb in itertools.product(a["loads"], b["loads"]):
                    ta = tree(load_tris(table, jet, frames, a["station"], int(wa)))
                    tb = tree(load_tris(table, jet, frames, b["station"], int(wb)))
                    if ta.overlap(tb):
                        failures.append(f"aircraft_{jet:03} sweep {sweep:.0f} stations {a['station'] + 1} weapon {wa} "
                                        f"and {b['station'] + 1} weapon {wb}: into each other")
    return failures


if __name__ == "__main__":
    with open(os.path.join(ROOT, "assets", "models", "stores.json")) as f:
        table = json.load(f)
    failures = check(table)
    for x in failures:
        print(x)
    print(f"{len(failures)} failures")
    if failures:
        sys.exit(1)
