"""Converts the Sketchfab "F-117 Nighthawk (War Thunder)" by Köjf (CC BY 4.0,
https://sketchfab.com/3d-models/f-117-nighthawk-war-thunder-8c0fa3d7bb2b4188a75e3d73d76df0f2,
267k triangles, textured) into the game's source mesh assets-src/blend/f117a.blend, which
tools/models/f117.py builds the F-117A (`aircraft_024`) and tools/models/build_cockpit_f117.py
its modern cockpit from:

    blender -b --factory-startup --python-exit-code 1 --python tools/models/f117_source.py -- "<dir>/f-117_nighthawk_war_thunder.glb"

The exterior (objects `airframe`, `glass`, `intake`, `sensor`): the source models everything,
the engines, the gear in its wells and the bombs in their bays. Only the skin is kept: the
faces of the airframe's shell, the tails and the elevons that enough of the sky can see
(`visible`; the source's faces are two-sided, each is turned to the side that sees out). The
shell's openings (the weapon bays, the gear wells, the sensor apertures, the intakes and the
canopy's panes) are closed flat, so the builder can cut the bays and lay the panes, grids
and sensor windows over the skin (their outlines are kept: `glass`, `intake`, `sensor`). The
air data probes go (the builder draws lines). The right half is then reduced to `HALF_TRIS`
triangles and mirrored. The aircraft is faceted, so its large facets survive this untouched:
what goes are the saw-toothed panel edges, the exhaust slots' vanes, the rounded leading edges
and the like.

The cockpit (objects `cockpit`, `cockpit_stick`, `hud`): the source's cockpit with its canopy
frame and its HUD's glass,
without the pilot, the seat and what is behind it, in the game's cockpit materials and without
the source's pictures (see `cockpit`).

Output frame: metres, x right, y forward, z up; the nose tip (without the probes) at
y = `NOSE`, the lowest point of the underside at z = 0, as tools/models/f117.py builds it. The
pilot's eye is at `EYE`.
"""

import math
import os
import sys

import bmesh
import bpy
from mathutils import Matrix, Vector
from mathutils.bvhtree import BVHTree

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.abspath(os.path.join(HERE, "..", ".."))
OUT = os.path.join(ROOT, "assets-src", "blend", "f117a.blend")

# The source's materials (their names' first characters): the airframe, the cockpit, the
# transparent panes (the pilot is another, not used).
AIRFRAME, COCKPIT, PANES = "51911afb", "80aa9aa0", "9275bc17"
# The source frame (Blender's, after import): the nose towards -y; its nose tip and the
# underside's lowest facets.
SRC_NOSE_Y, SRC_BELLY_Z = -9.93, -0.50
NOSE = 10.6
# Loose parts of the airframe with at least this much visible skin (m2) are kept: the shell,
# the tails and the elevons.
PART_M2 = 3.0
# Parts with this much skin are doors and panels of the skin (`skin`), and how far inside it
# (m, at least and at most) a point of the shell is moved out onto it (`settle`).
DOOR_M2 = 0.3
SETTLE_M = (0.004, 0.7)
BESIDE = [(0.0, 0.0)] + [(0.04 * math.cos(a * math.pi / 4), 0.04 * math.sin(a * math.pi / 4)) for a in range(8)]
# A face is skin when this many of the RAYS directions over one of its sides see the sky.
RAYS, SEEN = 64, 4
WELD_M = 0.0005
FLAT_DEG = 1.5
HALF_TRIS = 560
# The probes stand out of the nose's tip and edges: points this far (m) ahead of the tip, or
# outside a line of slope PROBE_SLOPE (half width per metre aft of the tip) over the nose's
# first PROBE_S metres.
PROBE_SLOPE, PROBE_M, PROBE_S = 0.44, 0.03, 1.3
# The pilot's eye (the source's pilot figure's), a point inside the fuselage under it, and
# the HUD's glass's half width (not a pane of the canopy).
EYE = Vector((0.0, NOSE - 4.90, 1.86))
INSIDE = Vector((0.0, NOSE - 4.90, 0.9))
HUD_X = 0.14
OUTLINE_M = 0.03
# The cockpit's stick: the loose parts wholly inside this box (m, about the eye).
STICK = ((-0.09, 0.54, -1.15), (0.09, 0.77, -0.45))
SHARP_DEG = 35.0
# The seat and what is behind it: the loose parts wholly aft of SEAT_Y (m ahead of the eye),
# but for the walls (parts of at least WALL_M2).
SEAT_Y, WALL_M2 = 0.15, 1.5
PANE_M2 = 0.03


def world(o):
    """An object's world matrix (the imported ones' are not evaluated yet)."""
    m = o.matrix_parent_inverse @ o.matrix_basis
    return world(o.parent) @ m if o.parent else m


def frame():
    """The source frame to the output frame: turned half round about z."""
    return Matrix.Translation((0.0, NOSE + SRC_NOSE_Y, -SRC_BELLY_Z)) @ Matrix.Rotation(math.pi, 4, "Z")


def gather(objects, prefix):
    """All faces of the objects with material `prefix` in the output frame, as a bmesh."""
    bm = bmesh.new()
    f = frame()
    for o in objects:
        if o.type == "MESH" and o.data.materials and o.data.materials[0].name.startswith(prefix):
            me = o.data.copy()
            me.transform(f @ world(o))
            bm.from_mesh(me)
            bpy.data.meshes.remove(me)
    return bm


def directions(n):
    out = []
    for i in range(n):
        z = 1 - 2 * (i + 0.5) / n
        r = math.sqrt(1 - z * z)
        a = i * 2.399963
        out.append(Vector((r * math.cos(a), r * math.sin(a), z)))
    return out


def visible(bm, tree=None, seen_at_least=None):
    """Deletes the faces that do not see the sky (past `tree`: the mesh itself by default) and
    turns the others to face it."""
    seen_at_least = SEEN if seen_at_least is None else seen_at_least
    if tree is None:
        # In triangles: the middle of a bent polygon need not lie on it.
        bmesh.ops.triangulate(bm, faces=[f for f in bm.faces if len(f.verts) > 3])
        bm.normal_update()
        tree = BVHTree.FromBMesh(bm)
    dirs = directions(RAYS)
    gone = []
    for f in bm.faces:
        n, c = f.normal.copy(), f.calc_center_median()
        seen = []
        for side in (1.0, -1.0):
            p = c + n * side * 0.002
            k = 0
            for d in dirs:
                if d.dot(n) * side > 0.05 and tree.ray_cast(p, d, 80.0)[0] is None:
                    k += 1
            seen.append(k)
        if max(seen) < seen_at_least:
            gone.append(f)
        elif seen[1] > seen[0]:
            f.normal_flip()
    bmesh.ops.delete(bm, geom=gone, context="FACES")


def islands(bm):
    seen, out = set(), []
    for f in bm.faces:
        if f in seen:
            continue
        part, stack = [], [f]
        seen.add(f)
        while stack:
            g = stack.pop()
            part.append(g)
            for e in g.edges:
                for h in e.link_faces:
                    if h not in seen:
                        seen.add(h)
                        stack.append(h)
        out.append(part)
    return out


def under(part):
    """A part that lies flat on the underside between the wing roots."""
    vs = {v for f in part for v in f.verts}
    return max(v.co.z for v in vs) < 0.2 and max(abs(v.co.x) for v in vs) < 1.0


def probe(p):
    s = NOSE - p.y
    return s < -PROBE_M or (s < PROBE_S and abs(p.x) > max(s, 0.0) * PROBE_SLOPE + PROBE_M)


def tail(part):
    vs = [v for f in part for v in f.verts]
    return NOSE - max(v.co.y for v in vs) > 15.0 and max(v.co.z for v in vs) > 1.5


def skin(bm, parts, objects):
    """The outer skin with its doors shut and its panes in: what the shell's closed openings
    are settled on."""
    nb = bmesh.new()
    vm = {}
    for p in parts:
        for f in p:
            nb.faces.new([vm.setdefault(v, nb.verts.new(v.co)) for v in f.verts])
    g = gather(objects, PANES)
    me = bpy.data.meshes.new("panes")
    g.to_mesh(me)
    g.free()
    nb.from_mesh(me)
    bpy.data.meshes.remove(me)
    nb.normal_update()
    tree = BVHTree.FromBMesh(nb)
    nb.free()
    return tree


def cycles(edges):
    """Closed walks along the edges (lists of points): loops that touch at a point come apart
    there."""
    left = set(edges)
    out = []
    while left:
        e = left.pop()
        walk, v = [e.verts[0]], e.verts[1]
        while v is not None and v not in walk:
            walk.append(v)
            step = next((g for g in v.link_edges if g in left), None)
            if step is None:
                v = None
            else:
                left.remove(step)
                v = step.other_vert(v)
        if v is not None:
            cycle = walk[walk.index(v):]
            if len(cycle) >= 3:
                out.append(cycle)
    return out


def settle(bm, part, envelope, rims):
    """Moves the shell's points that lie inside the skin (the rims of its closed openings, what
    is left of the wells and ducts behind them) straight up or down onto it, so that the
    closed openings lie flat in their facets. A rim is where the skin has a gap, between a
    door and its frame: there the skin is taken from just beside the point, its facet carried
    on to the point."""
    moved = 0
    bm.normal_update()
    for v in part:
        if not v.is_valid or not v.link_faces:
            continue
        up = sum((f.normal * f.calc_area() for f in v.link_faces), Vector()).z > 0
        d = Vector((0, 0, -1.0 if up else 1.0))
        best = None
        for dx, dy in BESIDE if v in rims else BESIDE[:1]:
            hit, n, _, _ = envelope.ray_cast(Vector((v.co.x + dx, v.co.y + dy, 10.0 if up else -10.0)), d, 20.0)
            if hit is None or abs(n.z) < 0.3:
                continue
            z = hit.z + (n.x * dx + n.y * dy) / n.z
            if best is None or (z > best) == up:
                best = z
        if best is not None and SETTLE_M[0] < abs(best - v.co.z) < SETTLE_M[1] and (best > v.co.z) == up:
            v.co.z = best
            moved += 1
    bmesh.ops.remove_doubles(bm, verts=bm.verts, dist=WELD_M)
    bmesh.ops.dissolve_degenerate(bm, edges=bm.edges, dist=WELD_M)
    print(f"settled {moved} points")


def loops(bm):
    """The boundary loops (lists of edges)."""
    edges = {e for e in bm.edges if e.is_boundary}
    out = []
    while edges:
        e = edges.pop()
        loop, stack = [e], [e]
        while stack:
            for v in stack.pop().verts:
                for g in v.link_edges:
                    if g in edges:
                        edges.remove(g)
                        loop.append(g)
                        stack.append(g)
        out.append(loop)
    return out


def opening(lo, hi):
    """What an opening of the shell with these bounds is: an intake, or None (closed and
    forgotten)."""
    c = (lo + hi) / 2
    s = NOSE - c.y
    if 4.3 < s < 6.4 and 0.9 < abs(c.x) < 2.0 and c.z > 0.5:
        return "intake"
    return None


def outline(faces):
    """The convex outline of flat faces (points in order, facing as the faces do), its corners
    at least OUTLINE_M off the line between their neighbours."""
    n = sum((f.normal * f.calc_area() for f in faces), Vector()).normalized()
    u = n.orthogonal().normalized()
    w = n.cross(u)
    pts = sorted({(round(v.co.dot(u), 5), round(v.co.dot(w), 5)): v.co.copy() for f in faces for v in f.verts}.items())
    cross = lambda o, a, b: (a[0] - o[0]) * (b[1] - o[1]) - (a[1] - o[1]) * (b[0] - o[0])
    hull = []
    for seq in (pts, pts[::-1]):
        chain = []
        for k, p in seq:
            while len(chain) >= 2 and cross(chain[-2][0], chain[-1][0], k) <= 0:
                chain.pop()
            chain.append((k, p))
        hull += chain[:-1]
    changed = True
    while changed and len(hull) > 3:
        changed = False
        for i in range(len(hull)):
            a, b, c = hull[i - 1][0], hull[i][0], hull[(i + 1) % len(hull)][0]
            base = math.hypot(c[0] - a[0], c[1] - a[1])
            if base < 1e-9 or abs(cross(a, b, c)) / base < OUTLINE_M:
                del hull[i]
                changed = True
                break
    return [p for _, p in hull]


def panes(objects):
    """The transparent panes' outer faces: the canopy's (`glass`) and the sensor windows'
    (`sensor`)."""
    bm = gather(objects, PANES)
    bmesh.ops.remove_doubles(bm, verts=bm.verts, dist=WELD_M)
    out = {"glass": [], "sensor": []}
    combiners = []
    for part in islands(bm):
        if sum(f.calc_area() for f in part) < PANE_M2:
            continue
        vs = {v for f in part for v in f.verts}
        c = sum((v.co for v in vs), Vector()) / len(vs)
        s = NOSE - c.y
        # Not the lights' lenses further aft.
        if s > 5.6:
            continue
        # The HUD's glass (narrow, on the centreline): its side facing the pilot.
        if max(abs(v.co.x) for v in vs) < HUD_X:
            side = [f for f in part if f.normal.dot(EYE - f.calc_center_median()) > 0]
            combiners.append((sum(f.calc_area() for f in side), side))
            continue
        canopy = c.z > 1.3 and s > 3.3
        # One side of each pane, the one facing out; its outline without the saw teeth.
        side = [f for f in part if f.normal.dot(f.calc_center_median() - INSIDE) > 0]
        if side:
            out["glass" if canopy else "sensor"].append(outline(side))
    for what, polys in out.items():
        nb = bmesh.new()
        for poly in polys:
            nb.faces.new([nb.verts.new(p) for p in poly])
        emit(what, nb)
    # The larger of the HUD's two glasses, about the eye as the cockpit is: `hud`.
    nb = bmesh.new()
    nb.faces.new([nb.verts.new(p - EYE) for p in outline(max(combiners, key=lambda c: c[0])[1])])
    emit("hud", nb)
    bm.free()


def copy_faces(name, bm, faces):
    """A new object of copies of `faces`."""
    nb = bmesh.new()
    vm = {}
    for f in faces:
        nb.faces.new([vm.setdefault(v, nb.verts.new(v.co)) for v in f.verts])
    bmesh.ops.remove_doubles(nb, verts=nb.verts, dist=WELD_M)
    bmesh.ops.dissolve_limit(nb, angle_limit=math.radians(FLAT_DEG), verts=nb.verts, edges=nb.edges, delimit=set())
    return emit(name, nb)


def emit(name, bm):
    me = bpy.data.meshes.new(name)
    bm.normal_update()
    bm.to_mesh(me)
    bm.free()
    o = bpy.data.objects.new(name, me)
    bpy.context.scene.collection.objects.link(o)
    return o


def exterior(objects, tree):
    bm = gather(objects, AIRFRAME)
    bmesh.ops.remove_doubles(bm, verts=bm.verts, dist=WELD_M)
    visible(bm, tree)
    # Not the bay doors: the bays are closed with the shell's other openings.
    every = islands(bm)
    parts = [p for p in every if sum(f.calc_area() for f in p) >= PART_M2 and not under(p)]
    envelope = skin(bm, [p for p in every if sum(f.calc_area() for f in p) >= DOOR_M2 and not tail(p)], objects)
    keep = {f for p in parts for f in p}
    bmesh.ops.delete(bm, geom=[f for f in bm.faces if f not in keep], context="FACES")
    # The probes: thin tubes standing forward out of the nose's edges.
    bmesh.ops.delete(bm, geom=[f for f in bm.faces if any(probe(v.co) for v in f.verts)], context="FACES")
    # The shell is the largest part: its openings closed, the named ones' outlines kept.
    shell = set(max(islands(bm), key=len))
    named = {}
    for loop in loops(bm):
        if not any(f in shell for e in loop for f in e.link_faces):
            continue
        vs = {v for e in loop for v in e.verts}
        lo = Vector([min(v.co[i] for v in vs) for i in range(3)])
        hi = Vector([max(v.co[i] for v in vs) for i in range(3)])
        what = opening(lo, hi)
        if what and (hi - lo).z > 0.2:
            named.setdefault(what, []).extend(bmesh.ops.holes_fill(bm, edges=loop, sides=0)["faces"])
    for what, faces in named.items():
        copy_faces(what, bm, [f for f in faces if f.is_valid])
    # The others: whole loops first, then whatever nets of edges are left; the closed
    # openings settled on the skin, and what then sees no sky at all taken out (which can
    # open the shell again: so thrice).
    part = {v for f in shell if f.is_valid for v in f.verts}
    for _ in range(3):
        rim = lambda: [e for e in bm.edges if e.is_boundary and e.verts[0] in part]
        rims = {v for e in rim() for v in e.verts}
        bmesh.ops.holes_fill(bm, edges=rim(), sides=0)
        for cycle in cycles(rim()):
            try:
                bm.faces.new(cycle)
            except ValueError:
                pass
        settle(bm, part, envelope, rims)
        visible(bm, seen_at_least=1)
        part = {v for v in part if v.is_valid}
        print(f"shell: {len(rim())} open edges")
    # The right half, reduced, and its mirror image.
    bmesh.ops.bisect_plane(bm, geom=bm.verts[:] + bm.edges[:] + bm.faces[:], plane_co=(0, 0, 0), plane_no=(-1, 0, 0), clear_outer=True)
    bmesh.ops.dissolve_limit(bm, angle_limit=math.radians(FLAT_DEG), verts=bm.verts, edges=bm.edges, delimit=set())
    bmesh.ops.triangulate(bm, faces=bm.faces)
    o = emit("airframe", bm)
    bpy.context.view_layer.objects.active = o
    m = o.modifiers.new("reduce", "DECIMATE")
    m.ratio = HALF_TRIS / len(o.data.polygons)
    bpy.ops.object.modifier_apply(modifier="reduce")
    bm = bmesh.new()
    bm.from_mesh(o.data)
    for v in bm.verts:
        if abs(v.co.x) < 0.004:
            v.co.x = 0.0
    bmesh.ops.mirror(bm, geom=bm.verts[:] + bm.edges[:] + bm.faces[:], axis="X", merge_dist=0.001)
    bmesh.ops.remove_doubles(bm, verts=bm.verts, dist=0.001)
    bmesh.ops.dissolve_degenerate(bm, edges=bm.edges, dist=0.001)
    # What the reduction opened is closed again; what is left inside goes.
    rim = [e for e in bm.edges if e.is_boundary]
    bmesh.ops.holes_fill(bm, edges=rim, sides=0)
    visible(bm, seen_at_least=1)
    bm.to_mesh(o.data)
    bm.free()
    print(f"airframe: {len(o.data.polygons)} triangles")


def texture(objects, prefix):
    """The base colour picture (H x W x 4 floats) of the objects' material `prefix`."""
    import numpy as np
    m = next(o.data.materials[0] for o in objects if o.type == "MESH" and o.data.materials and o.data.materials[0].name.startswith(prefix))
    bsdf = next(n for n in m.node_tree.nodes if n.type == "BSDF_PRINCIPLED")
    n = bsdf.inputs["Base Color"].links[0].from_node
    while n.type != "TEX_IMAGE":
        n = [link.from_node for i in n.inputs for link in i.links][0]
    im = n.image
    a = np.empty(im.size[0] * im.size[1] * 4, np.float32)
    im.pixels.foreach_get(a)
    return a.reshape(im.size[1], im.size[0], 4)


# Where a face's colour is taken: its corners' weights.
SAMPLES = [(1 / 3, 1 / 3, 1 / 3), (0.6, 0.2, 0.2), (0.2, 0.6, 0.2), (0.2, 0.2, 0.6), (0.45, 0.45, 0.1), (0.45, 0.1, 0.45), (0.1, 0.45, 0.45)]


def role(c):
    """The game's cockpit material for a colour (sRGB) of the source's picture."""
    r, g, b = c
    lum = 0.2126 * r + 0.7152 * g + 0.0722 * b
    if r > 0.35 and g > 0.28 and b < 0.6 * min(r, g):
        return "yellow"
    if r > 0.25 and r > 1.8 * g and r > 1.8 * b:
        return "red"
    if lum < 0.07 or max(r, g, b) > 1.6 * min(r, g, b) + 0.05:
        # Black, and the pictures on the displays' glass.
        return "black"
    if lum < 0.20:
        return "dark"
    if lum < 0.32:
        return "frame"
    if lum < 0.50:
        return "panel_grey"
    return "metal"


def cockpit(objects):
    """The cockpit about the pilot's eye: `cockpit`, all of the source's but its stick, its
    seat and what is behind the seat, and `cockpit_stick`, the stick from its boot up (its
    origin: the foot of the boot, where it tilts). Smooth but for its edges sharper than
    SHARP_DEG. None of the source's pictures is kept: each face takes the game's cockpit
    material nearest the colour the source paints it (`role`), so the instruments' faces, the
    displays and the consoles' lettering are blank, for the game's own."""
    import numpy as np
    bm = gather(objects, COCKPIT)
    bmesh.ops.remove_doubles(bm, verts=bm.verts, dist=WELD_M)
    bmesh.ops.translate(bm, verts=bm.verts, vec=-EYE)
    lo, hi = Vector(STICK[0]), Vector(STICK[1])
    stick, behind = set(), []
    for part in islands(bm):
        vs = {v for f in part for v in f.verts}
        if all(lo[i] <= v.co[i] <= hi[i] for v in vs for i in range(3)):
            stick.update(part)
        elif max(v.co.y for v in vs) < SEAT_Y and sum(f.calc_area() for f in part) < WALL_M2:
            behind += part
    # The seat's frame and rails and the machinery behind it go: the game stands its own seat
    # there (tools/models/build_cockpit_f117.py).
    bmesh.ops.delete(bm, geom=behind, context="FACES")
    bm.normal_update()
    for f in bm.faces:
        f.smooth = True
    for e in bm.edges:
        e.smooth = len(e.link_faces) == 2 and e.calc_face_angle(0.0) < math.radians(SHARP_DEG)
    # The faces' materials.
    img = texture(objects, COCKPIT)
    h, w, _ = img.shape
    uv = bm.loops.layers.uv.active
    names = ["black", "dark", "frame", "panel_grey", "metal", "yellow", "red"]
    for f in bm.faces:
        uvs = [l[uv].uv for l in f.loops][:3]
        cs = []
        for ws in SAMPLES:
            u = sum(k * p.x for k, p in zip(ws, uvs))
            v = sum(k * p.y for k, p in zip(ws, uvs))
            cs.append(img[int((v % 1.0) * (h - 1)), int((u % 1.0) * (w - 1)), :3])
        f.material_index = names.index(role(np.median(cs, axis=0)))
    # No picture: the UVs are not used.
    for f in bm.faces:
        for l in f.loops:
            l[uv].uv = (0.0, 0.0)
    mats = []
    for n in names:
        m = bpy.data.materials.new(n)
        mats.append(m)
    bm.faces.index_update()
    stick_idx = {f.index for f in stick}
    other = bm.copy()
    bmesh.ops.delete(bm, geom=list(stick), context="FACES")
    other.faces.ensure_lookup_table()
    bmesh.ops.delete(other, geom=[f for f in other.faces if f.index not in stick_idx], context="FACES")
    low = min(v.co.z for v in other.verts)
    base = [v.co for v in other.verts if v.co.z < low + 0.05]
    foot = Vector((0.0, sum(c.y for c in base) / len(base), low))
    bmesh.ops.translate(other, verts=other.verts, vec=-foot)
    print(f"cockpit: {len(bm.faces)} faces, stick {len(other.faces)} faces at {tuple(round(c, 3) for c in foot)}")
    for name, mesh, at in (("cockpit", bm, Vector()), ("cockpit_stick", other, foot)):
        o = emit(name, mesh)
        o.location = at
        for m in mats:
            o.data.materials.append(m)


def main():
    src = sys.argv[sys.argv.index("--") + 1]
    bpy.ops.wm.read_factory_settings(use_empty=True)
    bpy.ops.import_scene.gltf(filepath=src)
    objects = list(bpy.data.objects)
    occluder = bmesh.new()
    f = frame()
    for o in objects:
        if o.type == "MESH":
            me = o.data.copy()
            me.transform(f @ world(o))
            occluder.from_mesh(me)
            bpy.data.meshes.remove(me)
    tree = BVHTree.FromBMesh(occluder)
    exterior(objects, tree)
    panes(objects)
    cockpit(objects)
    occluder.free()
    for o in objects:
        bpy.data.objects.remove(o)
    bpy.ops.outliner.orphans_purge(do_recursive=True)
    bpy.ops.wm.save_as_mainfile(filepath=OUT)


main()
