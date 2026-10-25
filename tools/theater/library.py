"""Writes the model library descriptions, tools/theater/libraries/<id>.json, from a classic
theater's current mission data: per object model its name, target role and target class, and
`kinds`, the model to use for each kind of object a layout places (named after the model, plus
`EXTRA_KINDS` given here for unnamed ones, and the plain land and open sea ground), and `keep`,
the models a rebuilt theater carries over. Other terrain models (tiles, mountains) are not part
of a library: the builder makes its own.

    python3 tools/theater/library.py [--force]

Run once per theater, before its rebuild: afterwards the description is the source (a rebuilt
theater's data no longer lists its models). Existing descriptions are kept without --force.
"""

import json
import os
import re
import sys

ROOT = os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "..")
THEATERS = ["libya", "persian_gulf", "north_cape", "central_europe", "vietnam", "middle_east", "desert_storm", "cuba",
            "korea"]
# Unnamed models a layout uses: (theater, kind) -> model.
EXTRA_KINDS = {"central_europe": {"houses": 26, "houses2": 27, "land": 0, "sea": 20}}


def kind(name):
    return re.sub(r"[^a-z0-9]+", "_", name.lower()).strip("_")


def levels(td):
    """The terrain level each model is placed on most often in a classic theater's tile grids."""
    g, tiles = td["g"], td["tiles"]
    count = {}

    def add(level, objs):
        for _, _, shape in objs:
            c = count.setdefault(shape, {})
            c[level] = c.get(level, 0) + 1

    for t4 in g[4]:
        add(4, tiles[4][t4])
    for t3 in g[3]:
        add(3, tiles[3][t3])
        for k in range(16):
            t2 = g[2][t3 * 16 + k]
            add(2, tiles[2][t2])
            for k1 in range(16):
                add(1, tiles[1][g[1][t2 * 16 + k1]])
    return {shape: max(c, key=c.get) for shape, c in count.items()}


def add_levels(th, path):
    lib = json.load(open(path))
    td = json.load(open(os.path.join(ROOT, "assets", "mission", "theaters", th + ".json")))
    if len(td["g"][1]) != 512:
        return  # already rebuilt: its grids are the builder's
    lv = levels(td)
    wanted = {m["index"] for m in lib["models"]} | set(lib["kinds"].values())
    lib["levels"] = {str(i): lv[i] for i in sorted(wanted) if i in lv and lv[i] != 1}
    with open(path, "w") as f:
        json.dump(lib, f, indent=1)
        f.write("\n")


def main():
    for th in THEATERS:
        path = os.path.join(os.path.dirname(os.path.abspath(__file__)), "libraries", th + ".json")
        if "--levels" in sys.argv:
            if os.path.exists(path):
                add_levels(th, path)
            continue
        if os.path.exists(path) and "--force" not in sys.argv:
            continue
        w = json.load(open(os.path.join(ROOT, "assets", "mission", "theaters", th + ".json")))["world"]
        models, kinds = [], {}
        for i, n in enumerate(w["names"]):
            if not n or not (w["roles"][i] or w["shape_class"][i]):
                continue
            models.append({"index": i, "name": n, "role": w["roles"][i], "class": w["shape_class"][i]})
            kinds.setdefault(kind(n), i)
        kinds.update(EXTRA_KINDS.get(th, {}))
        keep = sorted({m["index"] for m in models} | set(kinds.values()) | {w["land_rubble"], w["water_rubble"]})
        out = {"theater": th, "land_rubble": w["land_rubble"], "water_rubble": w["water_rubble"], "kinds": kinds,
               "keep": keep, "models": models}
        with open(path, "w") as f:
            json.dump(out, f, indent=1)
            f.write("\n")


main()
