"""Converts the Sketchfab "USS Nimitz class aircraft carrier" by Larcenie Grant (CC BY 4.0,
https://skfb.ly/pvWrZ, 98k polygons, textured) into the game's low-poly source mesh
assets-src/blend/nimitz.blend, which tools/models/carrier.py builds the theaters' carriers from:

    blender -b --factory-startup --python-exit-code 1 --python tools/models/nimitz.py -- "<dir>/carrier(baked).gltf"

(The download's glTF names three textures `<name>_png.png`; link them to `<name>.png` first.)

Each face takes the colour of its texture (or material) at its middle, snapped to a paint role:
its material is `pal_NNN` of the role's colour byte (`role`; carrier.py maps them to paints).
The light fixtures become points (`nimitz_lights_NNN`, NNN the light's colour), the small
clutter (rails, stanchions, small gear: loose parts under `CLUTTER_M`) goes, the hull below
`KEEL_M` under the waterline goes. Flat areas are merged; the hull, deck, island and masts
keep the rest of their shape, and every smaller fitting is collapsed on its own to a budget
proportional to its size squared (about its area on screen), the factor solved so they come to
`SMALL_TRIS` together: simplified, none of them lost. The hull is cut at the waterline after that, red below and grey above. The flight deck's
painted lines are kept whole in `nimitz_marks` (their roles: the day colours).

Output frame: metres, bow along +y, starboard +x, the hull's centreline at x = 0, the ship's
middle at y = 0, the waterline (the top of the red antifouling) at z = 0.
"""

import collections
import math
import os
import sys

import bmesh
import bpy
import numpy as np
from mathutils import Matrix, Vector

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.abspath(os.path.join(HERE, "..", ".."))
OUT = os.path.join(ROOT, "assets-src", "blend", "nimitz.blend")

# Paint roles by colour byte: hull/island haze grey, mid grey, flight deck, black, red
# antifouling, white and yellow paint.
HULL, MID, DECK, BLACK, RED, WHITE, YELLOW = 226, 230, 236, 239, 219, 224, 199
# Loose parts smaller than this (largest extent, m) are dropped; up on the island and its masts
# (`HIGH_M` over the flight deck) smaller ones count: the yards and antennas.
CLUTTER_M, CLUTTER_HIGH_M, HIGH_M = 2.0, 1.0, 6.0
KEEL_M = 2.5  # hull kept this far under the waterline
SMALL_TRIS = 3000  # triangles of all the parts that are not kept whole
MIN_TRIS = 12, 40, 8.0  # the least a part keeps: 8 per metre of size, within 12..40 (domes stay round)
# The island and its masts (parts at least KEEP_M big, standing on the flight deck) and the
# hull, deck and sponsons (at least BIG_M) keep every triangle: collapsing boxy, detailed
# structure wrecks it, and the big parts are cheap.
KEEP_M, BIG_M = 8.0, 40.0
LIGHTS = {"red light": 146, "white light": 148, "yellow light": 19, "blue light": 9}
LIGHT_MERGE_M = 2.0


def role(c):
    """Paint role of a linear RGB colour."""
    r, g, b = c
    lum = 0.2126 * r + 0.7152 * g + 0.0722 * b
    if r > 0.2 and r > 2.5 * g and r > 2.5 * b:
        return RED
    if r > 0.5 and g > 0.5 and b < 0.3:
        return YELLOW
    if lum > 0.68:
        return WHITE
    if lum > 0.45:
        return HULL
    if lum > 0.3:
        return MID
    if lum > 0.1:
        return DECK
    return BLACK


def base_colour(m, cache):
    """A material's base colour: an image (H x W x 4 floats) or an RGBA constant."""
    bsdf = next(n for n in m.node_tree.nodes if n.type == "BSDF_PRINCIPLED")
    inp = bsdf.inputs["Base Color"]
    if not inp.is_linked:
        return np.array(inp.default_value)
    n = inp.links[0].from_node
    while n.type != "TEX_IMAGE":
        n = [link.from_node for i in n.inputs for link in i.links][0]
    im = n.image
    if im.name not in cache:
        a = np.empty(im.size[0] * im.size[1] * 4, np.float32)
        im.pixels.foreach_get(a)
        cache[im.name] = a.reshape(im.size[1], im.size[0], 4)
    return cache[im.name]


def sample(img, uvs):
    """Mean colour of an image at the corners and middle of a face's UVs."""
    h, w, _ = img.shape
    pts = list(uvs) + [np.mean(uvs, axis=0)]
    cs = [img[int((u[1] % 1.0) * (h - 1)), int((u[0] % 1.0) * (w - 1)), :3] for u in pts]
    return np.mean(cs, axis=0)


def islands(bm, faces):
    """Loose parts (lists of faces) of `faces`."""
    inside, seen, out = set(faces), set(), []
    for f in faces:
        if f in seen:
            continue
        part, stack = [], [f]
        seen.add(f)
        while stack:
            g = stack.pop()
            part.append(g)
            for e in g.edges:
                for h in e.link_faces:
                    if h in inside and h not in seen:
                        seen.add(h)
                        stack.append(h)
        out.append(part)
    return out


def extent(part):
    vs = {v for f in part for v in f.verts}
    lo = Vector([min(v.co[i] for v in vs) for i in range(3)])
    hi = Vector([max(v.co[i] for v in vs) for i in range(3)])
    return lo, hi


def mesh_object(name, bm, faces, roles):
    """A new object of `faces` with materials pal_NNN by role."""
    me = bpy.data.meshes.new(name)
    out = bmesh.new()
    vmap = {}
    slots = {}
    for f in faces:
        vs = []
        for v in f.verts:
            if v not in vmap:
                vmap[v] = out.verts.new(v.co)
            vs.append(vmap[v])
        try:
            nf = out.faces.new(vs)
        except ValueError:
            continue
        r = roles[f]
        if r not in slots:
            slots[r] = len(slots)
            mat = bpy.data.materials.get("pal_%03d" % r) or bpy.data.materials.new("pal_%03d" % r)
            me.materials.append(mat)
        nf.material_index = slots[r]
    bmesh.ops.remove_doubles(out, verts=out.verts, dist=1e-4)
    out.to_mesh(me)
    out.free()
    ob = bpy.data.objects.new(name, me)
    bpy.context.scene.collection.objects.link(ob)
    return ob


def tris(ob):
    return sum(len(p.vertices) - 2 for p in ob.data.polygons)


def least(size):
    lo, hi, per_m = MIN_TRIS
    return min(hi, max(lo, per_m * size))


def decimate(ob, deck_z):
    """Flat areas merged (colours kept apart); the parts not kept whole are collapsed to their
    share of SMALL_TRIS (proportional to their size squared, at least `least`, at most what
    they have)."""
    bpy.context.view_layer.objects.active = ob
    m = ob.modifiers.new("planar", "DECIMATE")
    m.decimate_type = "DISSOLVE"
    m.angle_limit = math.radians(2.0)
    m.delimit = {"MATERIAL", "NORMAL"}
    bpy.ops.object.modifier_apply(modifier=m.name)
    before = tris(ob)
    bpy.ops.object.select_all(action="DESELECT")
    ob.select_set(True)
    bpy.ops.object.mode_set(mode="EDIT")
    bpy.ops.mesh.select_all(action="SELECT")
    bpy.ops.mesh.separate(type="LOOSE")
    bpy.ops.object.mode_set(mode="OBJECT")
    parts = list(bpy.context.selected_objects)
    sizes = []
    for o in parts:
        vs = [v.co for v in o.data.vertices]
        sizes.append(max(max(v[i] for v in vs) - min(v[i] for v in vs) for i in range(3)))
    have = [tris(o) for o in parts]
    keep_all = []
    for o, s in zip(parts, sizes):
        lo = min(v.co.z for v in o.data.vertices)
        keep_all.append(s >= BIG_M or (s >= KEEP_M and lo > deck_z - 1.0))
    kept_tris = sum(n for n, kept in zip(have, keep_all) if kept)
    target = kept_tris + SMALL_TRIS

    def share(k, n, s, kept):
        return n if kept else min(n, max(least(s), k * s * s))

    def total(k):
        return sum(share(k, n, s, kept) for n, s, kept in zip(have, sizes, keep_all))

    lo, hi = 0.0, 1e4
    for _ in range(60):
        mid = (lo + hi) / 2
        lo, hi = (mid, hi) if total(mid) < target else (lo, mid)
    k = lo
    for o, n, s, kept in zip(parts, have, sizes, keep_all):
        budget = share(k, n, s, kept)
        if budget < n:
            bpy.context.view_layer.objects.active = o
            m = o.modifiers.new("collapse", "DECIMATE")
            m.decimate_type = "COLLAPSE"
            m.ratio = budget / n
            m.use_collapse_triangulate = True
            bpy.ops.object.modifier_apply(modifier=m.name)
    bpy.ops.object.select_all(action="DESELECT")
    for o in parts:
        o.select_set(True)
    keep = max(parts, key=lambda o: len(o.data.polygons))
    bpy.context.view_layer.objects.active = keep
    bpy.ops.object.join()
    keep.name = keep.data.name = "nimitz"
    print(f"body: {len(parts)} parts, {before} -> {tris(keep)} triangles ({kept_tris} in parts kept whole, "
          f"k {k:.3f} per m^2)")


def waterline(ob):
    """Cuts the hull at the waterline and under KEEL_M; the hull's paint is red below, grey above."""
    me = ob.data
    slot = {int(m.name[4:7]): i for i, m in enumerate(me.materials)}
    for r in (RED, HULL):
        if r not in slot:
            slot[r] = len(me.materials)
            me.materials.append(bpy.data.materials.get("pal_%03d" % r) or bpy.data.materials.new("pal_%03d" % r))
    bm = bmesh.new()
    bm.from_mesh(me)
    geom = lambda: bm.verts[:] + bm.edges[:] + bm.faces[:]  # noqa: E731
    bmesh.ops.bisect_plane(bm, geom=geom(), plane_co=(0, 0, -KEEL_M), plane_no=(0, 0, 1), clear_inner=True)
    bmesh.ops.bisect_plane(bm, geom=geom(), plane_co=(0, 0, 0), plane_no=(0, 0, 1))
    hullish = {slot.get(r) for r in (HULL, MID, RED)}
    for f in bm.faces:
        z = f.calc_center_median().z
        if z < -0.01 and f.material_index in hullish:
            f.material_index = slot[RED]
        elif z > 0.01 and f.material_index == slot[RED] and z < 6.0:
            f.material_index = slot[HULL]
    bm.to_mesh(me)
    bm.free()


def main():
    src = sys.argv[sys.argv.index("--") + 1]
    bpy.ops.wm.read_factory_settings(use_empty=True)
    bpy.ops.import_scene.gltf(filepath=src)
    ob = next(o for o in bpy.data.objects if o.type == "MESH")
    me = ob.data
    me.transform(ob.matrix_world)
    ob.matrix_world = Matrix()
    mats = [m.name for m in me.materials]
    cache = {}
    bases = [base_colour(m, cache) for m in me.materials]
    uv = me.uv_layers.active.data

    bm = bmesh.new()
    bm.from_mesh(me)
    bm.faces.ensure_lookup_table()
    roles = {}
    for p, f in zip(me.polygons, bm.faces):
        b = bases[p.material_index]
        roles[f] = role(b[:3] if b.ndim == 1 else sample(b, [uv[i].uv for i in p.loop_indices]))
    mat_of = {f: mats[f.material_index] for f in bm.faces}

    # Frame: the bow is the source's -y and its starboard -x; turn it round. The waterline is
    # the top of the red antifouling on the hull's sides.
    hull = [f for f in bm.faces if mat_of[f] == "hull texture"]
    red_top = sorted(f.calc_center_median().z for f in hull if roles[f] == RED and f.calc_area() > 0.5)
    wl = red_top[int(len(red_top) * 0.995)]
    under = [v.co for f in hull if roles[f] == RED for v in f.verts]
    xc = (min(v.x for v in under) + max(v.x for v in under)) / 2
    ys = [v.co.y for v in bm.verts]
    yc = (min(ys) + max(ys)) / 2
    turn = Matrix.Rotation(math.pi, 4, "Z") @ Matrix.Translation((-xc, -yc, -wl))
    bmesh.ops.transform(bm, matrix=turn, verts=bm.verts)
    print(f"waterline z {wl:.2f}, centreline x {xc:.2f}, length {max(ys) - min(ys):.1f} m")

    # Lights: one point per fixture (merged within LIGHT_MERGE_M).
    lights = collections.defaultdict(list)
    light_faces = [f for f in bm.faces if mat_of[f] in LIGHTS]
    for part in islands(bm, light_faces):
        c = sum((f.calc_center_median() for f in part), Vector()) / len(part)
        pts = lights[LIGHTS[mat_of[part[0]]]]
        if all((c - q).length > LIGHT_MERGE_M for q in pts):
            pts.append(c)
    # Deck markings: the painted lines (flat on the deck).
    marks = [f for f in bm.faces if mat_of[f] == "paint bump"]
    deck_z = max(v.co.z for f in marks for v in f.verts)
    rest = [f for f in bm.faces if mat_of[f] not in LIGHTS and mat_of[f] != "paint bump"]
    # Clutter: small loose parts, and the hull deep under water.
    body = []
    dropped = 0
    for part in islands(bm, rest):
        lo, hi = extent(part)
        size = max(hi - lo)
        least = CLUTTER_HIGH_M if lo.z > deck_z + HIGH_M else CLUTTER_M
        if size < least or hi.z < -KEEL_M:
            dropped += len(part)
            continue
        body += part
    print(f"deck z {deck_z:.2f}; dropped {dropped} faces of clutter; body {len(body)}, marks {len(marks)}")

    for o in list(bpy.data.objects):
        bpy.data.objects.remove(o)
    body_ob = mesh_object("nimitz", bm, body, roles)
    marks_ob = mesh_object("nimitz_marks", bm, marks, roles)
    bm.free()
    decimate(body_ob, deck_z)
    body_ob = bpy.data.objects["nimitz"]
    waterline(body_ob)
    tris2 = tris(body_ob)
    # Painted lines: face up (most of the source's face down, and the game culls back faces);
    # merge each line's pieces.
    for poly in marks_ob.data.polygons:
        if poly.normal.z < 0:
            poly.flip()
    marks_ob.data.update()
    bpy.context.view_layer.objects.active = marks_ob
    m = marks_ob.modifiers.new("planar", "DECIMATE")
    m.decimate_type = "DISSOLVE"
    m.angle_limit = math.radians(1.0)
    m.delimit = {"MATERIAL"}
    bpy.ops.object.modifier_apply(modifier=m.name)
    mtris = tris(marks_ob)
    print(f"body {tris2} triangles; marks {mtris} triangles")

    for c, pts in sorted(lights.items()):
        lm = bpy.data.meshes.new("nimitz_lights_%03d" % c)
        lm.from_pydata([tuple(p) for p in pts], [], [])
        lo = bpy.data.objects.new(lm.name, lm)
        bpy.context.scene.collection.objects.link(lo)
        print(f"lights {c}: {len(pts)}")
    for o in bpy.data.objects:
        o["source"] = "USS Nimitz class aircraft carrier by Larcenie Grant, CC BY 4.0, https://skfb.ly/pvWrZ"
    for img in list(bpy.data.images):
        bpy.data.images.remove(img)
    for mat in list(bpy.data.materials):
        if mat.users == 0:
            bpy.data.materials.remove(mat)
    bpy.context.preferences.filepaths.save_version = 0
    bpy.ops.wm.save_as_mainfile(filepath=OUT)


main()
