"""Rebuilds a model's coarser detail levels from its own finest one: each named level is replaced
by a copy of `_lod0` without its detail lines (the fine trim drawn as edges, which would only
flicker at that distance), so the model keeps its shape and colours when it switches level. The level keeps
its switch distance, takes the finest level's collision planes (the game tests the shown level)
and gets its radius from the new geometry.

    blender -b assets-src/blend/<file>.blend --python-exit-code 1 --python tools/models/lower_lods.py -- <file>_NNN:1,2 ...
    tools/models/export.sh <file>

Runs on the open master and saves it. `<file>_NNN:1,2` names the model root and the levels to
rebuild.
"""

import math
import sys

import bpy


def subtree(o):
    yield o
    for c in o.children:
        yield from subtree(c)


def remove(o):
    for x in reversed(list(subtree(o))):
        data = x.data if x.type == "MESH" else None
        bpy.data.objects.remove(x, do_unlink=True)
        if data is not None and data.users == 0:
            bpy.data.meshes.remove(data)


def copy(src, parent, old, new, coll):
    """Copies `src` and its children below `parent`, renaming `old` to `new`; leaves out the
    detail lines."""
    tail = src.name[len(old):]
    if "_detail" in tail and "_lines_" in tail:
        return None
    o = src.copy()
    if src.type == "MESH":
        o.data = src.data.copy()
        o.data.name = src.data.name.replace(old, new)
    o.name = src.name.replace(old, new)
    coll.objects.link(o)
    o.parent = parent
    o.matrix_parent_inverse = src.matrix_parent_inverse.copy()
    for c in src.children:
        copy(c, o, old, new, coll)
    return o


def destroyed(o, top):
    """True when `o` (or a node between it and the level node) shows only once destroyed."""
    while o is not None and o != top:
        conds = o.get("conds")
        if conds and any(int(c[0]) == 1 and int(c[1]) != 0 for c in conds):
            return True
        o = o.parent
    return False


def points(level, root):
    """The level's intact face vertices in the root's frame."""
    inv = root.matrix_world.inverted()
    out = []
    for o in subtree(level):
        if o.type != "MESH" or not o.data.polygons or destroyed(o, level):
            continue
        m = inv @ o.matrix_world
        used = {i for p in o.data.polygons for i in p.vertices}
        out.extend(m @ o.data.vertices[i].co for i in used)
    return out


def rebuild(root, k):
    name = root.name
    lod0 = bpy.data.objects[f"{name}_lod0"]
    old = bpy.data.objects.get(f"{name}_lod{k}")
    props = {"max_dist": old["max_dist"]} if old is not None and "max_dist" in old else {}
    coll = (old or lod0).users_collection[0]
    if old is not None:
        remove(old)
    level = copy(lod0, root, f"{name}_lod0", f"{name}_lod{k}", coll)
    for key in list(level.keys()):
        del level[key]
    for key, value in props.items():
        level[key] = value
    pts = points(level, root)
    level["planes"] = [list(p) for p in lod0.get("planes", [])]
    level["radius"] = int(math.ceil(max((p.length for p in pts), default=0.0)))
    return len(pts)


def main():
    args = sys.argv[sys.argv.index("--") + 1:]
    for arg in args:
        name, levels = arg.split(":")
        root = bpy.data.objects[name]
        for k in map(int, levels.split(",")):
            n = rebuild(root, k)
            print(f"{name}_lod{k}: rebuilt from lod0, {n} vertices")
    bpy.ops.wm.save_mainfile()


if __name__ == "__main__":
    main()
