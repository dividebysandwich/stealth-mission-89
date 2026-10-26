"""Reading and writing binary glTF (.glb) files: (JSON dict, binary chunk)."""

import json
import struct


def read(path):
    d = open(path, "rb").read()
    assert d[:4] == b"glTF"
    off, js, binary = 12, None, b""
    while off < len(d):
        n, kind = struct.unpack_from("<II", d, off)
        chunk = d[off + 8:off + 8 + n]
        if kind == 0x4E4F534A:
            js = json.loads(chunk)
        elif kind == 0x004E4942:
            binary = chunk
        off += 8 + n
    return js, binary


def write(path, js, binary):
    j = json.dumps(js, separators=(",", ":")).encode()
    j += b" " * (-len(j) % 4)
    binary += b"\0" * (-len(binary) % 4)
    total = 12 + 8 + len(j) + 8 + len(binary)
    with open(path, "wb") as f:
        f.write(struct.pack("<III", 0x46546C67, 2, total))
        f.write(struct.pack("<II", len(j), 0x4E4F534A) + j)
        f.write(struct.pack("<II", len(binary), 0x004E4942) + binary)


def subset(g, gbin, wanted):
    """The glTF with only the root models whose number `wanted` accepts (and their children)."""
    roots = g["scenes"][g.get("scene", 0)]["nodes"]
    kept = []

    def walk(i):
        kept.append(i)
        for c in g["nodes"][i].get("children", []):
            walk(c)

    for r in roots:
        if wanted(int(g["nodes"][r]["name"].rsplit("_", 1)[1])):
            walk(r)
    index = {old: new for new, old in enumerate(kept)}
    nodes = []
    for old in kept:
        n = dict(g["nodes"][old])
        if "children" in n:
            n["children"] = [index[c] for c in n["children"]]
        nodes.append(n)
    out = dict(g)
    out["nodes"] = nodes
    return out, gbin


def append(a, abin, b, bbin):
    """All of A's nodes, then all of B's (root order kept), with one buffer holding only what
    the nodes use."""
    out = {k: v for k, v in a.items() if k not in ("nodes", "meshes", "accessors", "bufferViews", "buffers", "materials", "scenes")}
    nodes, meshes, accessors, views, materials = [], [], [], [], []
    blob = bytearray()
    mat_index = {}
    for src, sbin in ((a, abin), (b, bbin)):
        used_meshes = {n["mesh"] for n in src["nodes"] if "mesh" in n}
        used_acc = {i for m in used_meshes for p in src["meshes"][m]["primitives"]
                    for i in list(p["attributes"].values()) + ([p["indices"]] if "indices" in p else [])}
        used_views = {src["accessors"][i]["bufferView"] for i in used_acc if "bufferView" in src["accessors"][i]}
        vmap, amap, mmap = {}, {}, {}
        for i, v in enumerate(src["bufferViews"]):
            if i not in used_views:
                continue
            v = dict(v)
            o = v.get("byteOffset", 0)
            data = sbin[o:o + v["byteLength"]]
            blob.extend(b"\0" * (-len(blob) % 4))
            v["byteOffset"] = len(blob)
            v["buffer"] = 0
            blob.extend(data)
            vmap[i] = len(views)
            views.append(v)
        for i, acc in enumerate(src["accessors"]):
            if i not in used_acc:
                continue
            acc = dict(acc)
            if "bufferView" in acc:
                acc["bufferView"] = vmap[acc["bufferView"]]
            amap[i] = len(accessors)
            accessors.append(acc)
        for i, m in enumerate(src.get("materials", [])):
            if m["name"] not in mat_index:
                mat_index[m["name"]] = len(materials)
                materials.append(m)
        for i, m in enumerate(src["meshes"]):
            if i not in used_meshes:
                continue
            m = dict(m)
            prims = []
            for p in m["primitives"]:
                p = dict(p)
                p["attributes"] = {k: amap[v] for k, v in p["attributes"].items()}
                if "indices" in p:
                    p["indices"] = amap[p["indices"]]
                if "material" in p:
                    p["material"] = mat_index[src["materials"][p["material"]]["name"]]
                prims.append(p)
            m["primitives"] = prims
            mmap[i] = len(meshes)
            meshes.append(m)
        base = len(nodes)
        for n in src["nodes"]:
            n = dict(n)
            if "children" in n:
                n["children"] = [c + base for c in n["children"]]
            if "mesh" in n:
                n["mesh"] = mmap[n["mesh"]]
            nodes.append(n)
    child = {c for n in nodes for c in n.get("children", [])}
    out["scene"] = 0
    out["scenes"] = [{"name": "Scene", "nodes": [i for i in range(len(nodes)) if i not in child]}]
    out["nodes"], out["meshes"], out["accessors"], out["bufferViews"], out["materials"] = nodes, meshes, accessors, views, materials
    out["buffers"] = [{"byteLength": len(blob) + (-len(blob) % 4)}]
    if "KHR_materials_unlit" not in out.get("extensionsUsed", []):
        out.setdefault("extensionsUsed", []).append("KHR_materials_unlit")
    return out, bytes(blob)
