"""Replaces one model of a shipped theater file with the same model of a fresh export of its
master, leaving every other model as it is. The theater files are also rebuilt by the theater
builder (tools/theater/build.sh), so exporting the whole master would lose its changes.

    blender -b assets-src/blend/<file>.blend --python-exit-code 1 --python tools/models/export.py -- /tmp/<file>.glb
    python3 tools/models/splice_model.py <file> <NNN> /tmp/<file>.glb

The model keeps its place in the file (the model viewer's `--model` index).
"""

import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from glbio import append, read, subset, write  # noqa: E402

ROOT = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", ".."))


def main():
    stem, number, fresh = sys.argv[1], int(sys.argv[2]), sys.argv[3]
    path = os.path.join(ROOT, "assets", "models", stem + ".glb")
    a, abin = read(path)
    b, bbin = read(fresh)
    name = "%s_%03d" % (stem, number)
    roots = [a["nodes"][r]["name"] for r in a["scenes"][a.get("scene", 0)]["nodes"]]
    if name not in roots:
        raise SystemExit("%s: no model %s" % (path, name))
    if not any(n["name"] == name for n in b["nodes"]):
        raise SystemExit("%s: no model %s" % (fresh, name))
    a, abin = subset(a, abin, lambda i: i != number)
    b, bbin = subset(b, bbin, lambda i: i == number)
    out, blob = append(a, abin, b, bbin)
    # Back to the old root order.
    scene = out["scenes"][0]["nodes"]
    scene.sort(key=lambda r: roots.index(out["nodes"][r]["name"]))
    write(path, out, blob)
    print("%s: %s replaced" % (path, name))


main()
