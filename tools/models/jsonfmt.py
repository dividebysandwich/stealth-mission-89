"""Readable JSON for the data files kept in git (assets/mission/theaters/*.json): one record per
line, so a change diffs as the records it touches.

A value that fits in WIDTH columns stays on one line, and so does a record (an object of plain
values) whatever its width; a list of numbers or short values (a grid, a road's points) is packed
into lines of WIDTH; anything else gets one member per line. It costs some 10 % over compact
JSON (the files are compiled into the browser and Android builds), where indent=1 doubles them.
"""

import json

WIDTH = 100


def _one(v):
    return json.dumps(v, separators=(",", ":"), ensure_ascii=False)


def _fmt(v, ind):
    s = _one(v)
    if len(ind) + len(s) <= WIDTH or not isinstance(v, (dict, list)) or not v:
        return s
    if isinstance(v, dict) and not any(isinstance(x, (dict, list)) for x in v.values()):
        return s
    inner = ind + " "
    if isinstance(v, dict):
        body = ",\n".join(inner + _one(k) + ":" + _fmt(x, inner) for k, x in v.items())
        return "{\n" + body + "\n" + ind + "}"
    items = [_one(x) for x in v]
    if all(not isinstance(x, (dict, list)) for x in v) or all(len(t) <= 24 for t in items):
        lines, cur = [], ""
        for t in items:
            if cur and len(inner) + len(cur) + 1 + len(t) > WIDTH:
                lines.append(cur)
                cur = t
            else:
                cur = cur + "," + t if cur else t
        lines.append(cur)
        return "[\n" + ",\n".join(inner + line for line in lines) + "\n" + ind + "]"
    return "[\n" + ",\n".join(inner + _fmt(x, inner) for x in v) + "\n" + ind + "]"


def dumps(v):
    return _fmt(v, "") + "\n"


def write(path, v):
    with open(path, "w", encoding="utf-8") as f:
        f.write(dumps(v))


if __name__ == "__main__":
    # Reformat the files given (python3 tools/models/jsonfmt.py assets/mission/theaters/*.json).
    import sys

    for path in sys.argv[1:]:
        with open(path, encoding="utf-8") as f:
            data = json.load(f)
        write(path, data)
        with open(path, encoding="utf-8") as f:
            assert json.load(f) == data, path
