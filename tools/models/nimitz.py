"""Converts the Sketchfab "USS Nimitz class aircraft carrier" by Larcenie Grant (CC BY 4.0,
https://skfb.ly/pvWrZ, 98k polygons, textured) into the game's source mesh
assets-src/blend/nimitz.blend, which tools/models/carrier.py builds the theaters' carriers from:

    blender -b --factory-startup --python-exit-code 1 --python tools/models/nimitz.py -- "<dir>/carrier(baked).gltf"

(The download's glTF names three textures `<name>_png.png`; link them to `<name>.png` first.)

The ship keeps its full detail: every part of it above the waterline, down to the rails and
gratings. Only the hull below `KEEL_M` under the waterline goes, and the source's three Phalanx
mounts on the deck-edge sponsons (`CIWS`): the game draws its own there, their guns following
what they fight (crates/stealth-sim/src/naval.rs `CARRIER_MOUNTS`). Each face takes the colour
of its texture (or material) at its middle, snapped to a paint role: its material is `pal_NNN`
of the role's colour byte (`role`; carrier.py maps them to paints). Flat areas of one role are
merged (`FLAT_DEG`), which changes no shape. The light fixtures also become points
(`nimitz_lights_NNN`, NNN the light's colour). The hull is cut at the waterline, red below and
grey above. The flight deck's painted lines are kept whole in `nimitz_marks` (their roles: the
day colours). Two things the source draws in ways the game cannot: the island's window panes
are decals (`WINDOW_DECAL_U`) painted barely darker than the wall, so they become their own
faces in the classic window colour (`WINDOW`, the airfield towers' glass: dark by day, amber at
night); and the deck-edge walkways are gratings of tiny bars facing down (the game culls back
faces), so each also gets a solid floor traced from the bars' footprint (`walkways`), and the
plates the others stand on face down only, so they get an upward twin.

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
WINDOW = 150  # the classic window colour (tools/models/control_tower.py)
WINDOW_DECAL_U = 0.54  # the decal sheet's window panes lie right of this (its left: the hull number)
WINDOW_LIFT_M = 0.03  # panes stand this far off the wall
# The walkway gratings: the source's "metallic" faces this high (m over the waterline), facing
# down; their footprint is traced on a grid of GRID_M cells.
GRATING_Z = (16.3, 16.9)
# The other walkways' floors are plates facing down only, this high: they get an upward twin.
WALKWAY_Z = (15.5, 16.9)
GRID_M = 0.5
LIGHT_REACH_M = 1.5
KEEL_M = 2.5  # hull kept this far under the waterline
# Parts at least this big (m) with faces at walkway height facing down get upward twins (the
# gratings' bars are smaller: the floors under them stand in).
PLATE_PART_M = 2.0
FLAT_DEG = 1.0  # neighbouring faces of one role this close to coplanar are merged
# The source's Phalanx mounts (m: middle, the sponson floor they stand on): the bow's two, and
# the one on the low sponson at the port quarter. Its parts within `CIWS_REACH_M` of the
# middle, up to `CIWS_HIGH_M` over the floor, go. naval.rs `CARRIER_MOUNTS` stands the game's
# there.
CIWS = ((-22.5, 89.4, 14.64), (22.9, 89.3, 14.64), (-15.5, -147.0, 9.44))
CIWS_REACH_M, CIWS_HIGH_M = 1.7, 3.3
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


def merge_flat(ob):
    """Neighbouring faces of one role within FLAT_DEG of coplanar merged into one."""
    before = tris(ob)
    bpy.context.view_layer.objects.active = ob
    m = ob.modifiers.new("planar", "DECIMATE")
    m.decimate_type = "DISSOLVE"
    m.angle_limit = math.radians(FLAT_DEG)
    m.delimit = {"MATERIAL", "NORMAL"}
    bpy.ops.object.modifier_apply(modifier=m.name)
    print(f"body: {before} -> {tris(ob)} triangles (flat areas merged)")


def is_ciws(lo, hi):
    """Whether a part (its bounds) belongs to one of the source's Phalanx mounts."""
    for x, y, z in CIWS:
        if (lo.z > z - 0.2 and hi.z < z + CIWS_HIGH_M
                and max(abs(lo.x - x), abs(hi.x - x), abs(lo.y - y), abs(hi.y - y)) < CIWS_REACH_M):
            return True
    return False


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


def walkways(grating):
    """Solid floors under the walkway gratings' footprint: (x0, x1, y0, y1, z) strips, merged
    from the grid cells the bars cover (one-cell gaps between bars closed)."""
    cells = set()
    z = {}
    for f in grating:
        xs = [v.co.x for v in f.verts]
        ys = [v.co.y for v in f.verts]
        for i in range(math.floor(min(xs) / GRID_M), math.floor(max(xs) / GRID_M) + 1):
            for j in range(math.floor(min(ys) / GRID_M), math.floor(max(ys) / GRID_M) + 1):
                cells.add((i, j))
                z[(i, j)] = max(z.get((i, j), -1e9), max(v.co.z for v in f.verts))
    # Close one-cell gaps between bars, across and along.
    for i, j in list(cells):
        for di, dj in ((2, 0), (0, 2)):
            if (i + di, j + dj) in cells:
                cells.add((i + di // 2, j + dj // 2))
    top = max(z.values()) if z else 0.0
    # Runs along y per column, then columns with the same run merged across.
    runs = collections.defaultdict(list)
    for i in sorted({c[0] for c in cells}):
        js = sorted(j for ii, j in cells if ii == i)
        start = prev = js[0]
        for j in js[1:] + [None]:
            if j is not None and j == prev + 1:
                prev = j
                continue
            runs[(start, prev)].append(i)
            if j is not None:
                start = prev = j
    strips = []
    for (j0, j1), cols in runs.items():
        cols.sort()
        a = b = cols[0]
        for i in cols[1:] + [None]:
            if i is not None and i == b + 1:
                b = i
                continue
            strips.append((a * GRID_M, (b + 1) * GRID_M, j0 * GRID_M, (j1 + 1) * GRID_M, top))
            if i is not None:
                a = b = i
    return strips


def two_sided(bm, faces):
    """Upward twins of `faces` (a millimetre up, on their own vertices: bmesh takes no second
    face over the same ones)."""
    out = []
    for f in faces:
        vs = [bm.verts.new(v.co + Vector((0, 0, 0.001))) for v in reversed(f.verts)]
        g = bm.faces.new(vs)
        g.material_index = f.material_index
        out.append(g)
    return out


def floors_object(strips):
    """The walkway floors, faces up and down (mid grey)."""
    me = bpy.data.meshes.new("nimitz_floors")
    verts, polys = [], []
    for x0, x1, y0, y1, z in strips:
        k = len(verts)
        verts += [(x0, y0, z), (x1, y0, z), (x1, y1, z), (x0, y1, z)]
        polys += [(k, k + 1, k + 2, k + 3), (k + 3, k + 2, k + 1, k)]
    me.from_pydata(verts, [], polys)
    me.materials.append(bpy.data.materials.get("pal_%03d" % MID) or bpy.data.materials.new("pal_%03d" % MID))
    me.update()
    ob = bpy.data.objects.new("nimitz_floors", me)
    bpy.context.scene.collection.objects.link(ob)
    return ob


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
    # Window panes: the decals on the island's walls.
    windows = []
    for p, f in zip(me.polygons, bm.faces):
        if mat_of[f] == "decal texture" and min(uv[i].uv[0] for i in p.loop_indices) > WINDOW_DECAL_U:
            windows.append(f)
            roles[f] = WINDOW
    # Walkway gratings.
    grating = [f for f in bm.faces if mat_of[f] == "metallic" and f.normal.z < -0.9
               and GRATING_Z[0] < f.calc_center_median().z < GRATING_Z[1]]
    floors = walkways(grating)
    skip = set(windows)
    rest = [f for f in bm.faces if mat_of[f] != "paint bump" and f not in skip]
    # The hull deep under water and the source's Phalanx mounts go; the plates at walkway
    # height facing down only get upward twins.
    body, plates = [], []
    under = ciws = 0
    for part in islands(bm, rest):
        lo, hi = extent(part)
        if hi.z < -KEEL_M:
            under += len(part)
            continue
        if is_ciws(lo, hi):
            ciws += len(part)
            continue
        body += part
        if max(hi - lo) >= PLATE_PART_M:
            plates += [f for f in part if f.normal.z < -0.9 and WALKWAY_Z[0] < f.calc_center_median().z < WALKWAY_Z[1]]
    for f, g in zip(plates, two_sided(bm, plates)):
        roles[g] = roles[f]
        body.append(g)
    print(f"deck z {deck_z:.2f}; dropped {under} faces under water, {ciws} of the Phalanx mounts; "
          f"body {len(body)} ({len(plates)} plates made two-sided), marks {len(marks)}")

    for o in list(bpy.data.objects):
        bpy.data.objects.remove(o)
    body_ob = mesh_object("nimitz", bm, body, roles)
    marks_ob = mesh_object("nimitz_marks", bm, marks, roles)
    for f in windows:
        for v in f.verts:
            v.co += f.normal * WINDOW_LIFT_M
    windows_ob = mesh_object("nimitz_windows", bm, windows, roles)
    bm.free()
    merge_flat(body_ob)
    floors_ob = floors_object(floors)
    bpy.ops.object.select_all(action="DESELECT")
    for o in (body_ob, windows_ob, floors_ob):
        o.select_set(True)
    bpy.context.view_layer.objects.active = body_ob
    bpy.ops.object.join()
    print(f"windows {len(windows)} panes, walkway floors {len(floors)} strips")
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

    # Lights off the ship (none expected now that every fitting stays) would float.
    from mathutils.bvhtree import BVHTree
    tree = BVHTree.FromObject(body_ob, bpy.context.evaluated_depsgraph_get())
    for c in list(lights):
        near = [p for p in lights[c] if tree.find_nearest(p, LIGHT_REACH_M)[0] is not None]
        print(f"lights {c}: {len(near)} of {len(lights[c])} on the ship")
        lights[c] = near
    for c, pts in sorted(lights.items()):
        lm = bpy.data.meshes.new("nimitz_lights_%03d" % c)
        lm.from_pydata([tuple(p) for p in pts], [], [])
        lo = bpy.data.objects.new(lm.name, lm)
        bpy.context.scene.collection.objects.link(lo)
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
