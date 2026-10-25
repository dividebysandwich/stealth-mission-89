"""Marks the tile shapes with water on them: every theater's assets/mission/theaters/<id>.json
gets `water_shapes`, the shapes whose model has a flat blue polygon (sea, lakes, rivers) at its
finest detail level. The campaign keeps its forward batteries off tiles with any of them.

Water is judged as in src/scenery/ground.rs: a flat triangle (|y| < 1 in the shape's frame) of
the detail level `_lod0` (or a model without levels), wreck state and light dots left out,
whose colour word shows blue over the middle of the ground gradient in the day palette.

    python3 tools/models/water_shapes.py
"""

import json
import os
import re
import struct
import sys

sys.path.insert(0, os.path.dirname(__file__))
import glbio  # noqa: E402

ROOT = os.path.join(os.path.dirname(__file__), "..", "..")


def ron_lists(text, key):
    """Every `key: [..]` list of numbers in a RON file."""
    return [
        [int(v) for v in re.findall(r"\d+", m)]
        for m in re.findall(key + r":\s*\[([^\]]*)\]", text)
    ]


def day_palette(theater):
    text = open(os.path.join(ROOT, "assets/palettes", f"tod_{theater}.ron")).read()
    day = text[text.index("day:"):text.index("dusk:")]
    body = day[day.index("colors:"):]
    return [tuple(int(c) for c in t) for t in re.findall(r"\((\d+), (\d+), (\d+)\)", body)]


def surface(word, day):
    """1 for water, 0 for land, None for an overlay the 3D view does not show."""
    lo = word & 0xFF
    hi = word >> 8
    if hi == 0xFB:
        return None
    shown = {0xFD: 0x78 | lo, 0xFE: 0x78 & lo}.get(hi, lo)
    r, g, b = day[shown]
    return int(b > r + 8 and b > g + 8)


def matrix(node):
    if "matrix" in node:
        m = node["matrix"]
        return [[m[c * 4 + r] for c in range(4)] for r in range(4)]
    tx, ty, tz = node.get("translation", [0, 0, 0])
    x, y, z, w = node.get("rotation", [0, 0, 0, 1])
    sx, sy, sz = node.get("scale", [1, 1, 1])
    rot = [
        [1 - 2 * (y * y + z * z), 2 * (x * y - z * w), 2 * (x * z + y * w)],
        [2 * (x * y + z * w), 1 - 2 * (x * x + z * z), 2 * (y * z - x * w)],
        [2 * (x * z - y * w), 2 * (y * z + x * w), 1 - 2 * (x * x + y * y)],
    ]
    s = (sx, sy, sz)
    m = [[rot[r][c] * s[c] for c in range(3)] + [t] for r, t in zip(range(3), (tx, ty, tz))]
    return m + [[0, 0, 0, 1]]


def mul(a, b):
    return [[sum(a[r][k] * b[k][c] for k in range(4)) for c in range(4)] for r in range(4)]


def accessor(js, binary, i):
    a = js["accessors"][i]
    v = js["bufferViews"][a["bufferView"]]
    fmt, size = {5126: ("f", 4), 5125: ("I", 4), 5123: ("H", 2), 5121: ("B", 1)}[
        a["componentType"]
    ]
    n = {"SCALAR": 1, "VEC2": 2, "VEC3": 3, "VEC4": 4}[a["type"]]
    stride = v.get("byteStride", size * n)
    base = v.get("byteOffset", 0) + a.get("byteOffset", 0)
    out = [struct.unpack_from("<" + fmt * n, binary, base + k * stride) for k in range(a["count"])]
    return [o[0] for o in out] if n == 1 else out


def index_from_name(name):
    i = name.rfind("pal_")
    return int(name[i + 4:i + 7]) if i >= 0 and name[i + 4:i + 7].isdigit() else None


def flats(js, binary, node_index, remap, day):
    """The shape's visible flat triangles in drawing order: [x0, y0, x1, y1, x2, y2, water] in
    model units (x east, y north)."""
    nodes, meshes, mats = js["nodes"], js.get("meshes", []), js.get("materials", [])
    out = []

    def walk(i, parent, in_lod0):
        node = nodes[i]
        name = node.get("name", "")
        lod0 = in_lod0 or "_lod0" in name
        if ("_lod" in name and not lod0) or "_s1v1" in name or "_dots_" in name:
            return
        m = mul(parent, matrix(node))
        if lod0 and "mesh" in node:
            for prim in meshes[node["mesh"]]["primitives"]:
                if prim.get("mode", 4) != 4:
                    continue
                colour = index_from_name(name)
                if colour is None and "material" in prim:
                    colour = index_from_name(mats[prim["material"]].get("name", ""))
                colour = 15 if colour is None else colour
                word = remap[colour] if colour < len(remap) else 0xFF00 | colour
                wet = surface(word, day)
                if wet is None:
                    continue
                pos = [
                    [sum(m[r][k] * v for k, v in enumerate((*p, 1))) for r in range(3)]
                    for p in accessor(js, binary, prim["attributes"]["POSITION"])
                ]
                idx = accessor(js, binary, prim["indices"]) if "indices" in prim else range(len(pos))
                idx = list(idx)
                for t in range(0, len(idx) - 2, 3):
                    tri = [pos[idx[t + k]] for k in range(3)]
                    if all(abs(v[1]) < 1.0 for v in tri):
                        out.append([c for v in tri for c in (round(v[0], 2), round(-v[2], 2))] + [wet])
        for c in node.get("children", []):
            walk(c, m, lod0)

    walk(node_index, [[1, 0, 0, 0], [0, 1, 0, 0], [0, 0, 1, 0], [0, 0, 0, 1]], False)
    return out


def detail_high_words():
    """Colour words at the highest detail level (src/palette.rs `remap_tables`)."""
    fills = {"Solid": 0xFF00, "Hidden": 0xFB00, "Checker": 0xFC00, "AndGround": 0xFE00, "OrGround": 0xFD00}
    words = [0xFF00 | c for c in range(256)]
    text = open(os.path.join(ROOT, "assets/palettes/special_colours.ron")).read()
    for m in re.finditer(r"\(colour: (\d+)[^\n]*?high: (\w+)\((\d+)\)", text):
        words[int(m.group(1))] = fills[m.group(2)] | int(m.group(3))
    return words


def main():
    remap = detail_high_words()
    for f in sorted(os.listdir(os.path.join(ROOT, "assets/mission/theaters"))):
        theater = f[:-5]
        path = os.path.join(ROOT, "assets/mission/theaters", f)
        text = open(path).read()
        data = json.loads(text)
        js, binary = glbio.read(os.path.join(ROOT, "assets/models", f"{theater}.glb"))
        names = {n.get("name"): i for i, n in enumerate(js["nodes"])}
        day = day_palette(theater)
        shapes = sorted(
            {o[2] for level in data["tiles"] for tile in level for o in tile}
        )
        ground = [[] for _ in range(max(shapes) + 1)]
        for s in shapes:
            if f"{theater}_{s:03}" in names:
                ground[s] = flats(js, binary, names[f"{theater}_{s:03}"], remap, day)
        data.pop("water_shapes", None)
        # Only a theater with water needs them.
        data["ground"] = ground if any(t[6] for g in ground for t in g) else []
        open(path, "w").write(json.dumps(data, separators=(",", ":")))
        wet = sum(any(t[6] for t in g) for g in ground)
        tris = sum(len(g) for g in ground)
        print(f"{theater}: {wet} of {len(shapes)} shapes have water, {tris} flat triangles")


if __name__ == "__main__":
    main()
