"""Redraws the runway of every theater master's airbase model: ICAO-style markings (threshold
stripes, touchdown zone bars, aiming point blocks), the daytime centreline dashes and the
cratered runway it becomes when destroyed. The pad, taxiways, aprons, lights and detail levels
stay as they are.

    blender -b assets-src/blend/<file>.blend --python-exit-code 1 --python tools/models/runway_marks.py
    tools/models/export.sh <file>

Runs on the open master and saves it. In the finest level the base mesh loses its marking
faces (colour 203, or its own white ones from an earlier run) and gets the new ones, `_s11v0` (shown by day) gets the centreline dashes;
in the two finest levels `_s1v1` (destroyed) gets the craters. Model space: x east, y north
(along the runway), z up, in LOD2 units (4 WU, about 1.95 m); the runway is 44 units wide and
840 long.
"""

import json
import math
import os
import random
import sys

import bpy

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.abspath(os.path.join(HERE, "..", ".."))
sys.path.insert(0, HERE)

import mklib  # noqa: E402
from mklib import Geo, Ramp  # noqa: E402

UNIT_M = 4 * 512_000.0 / 1_048_576.0
HALF_W, HALF_L = 22.0, 420.0
OLD_MARK, MARK, DASH = 203, 224, 224
MARK_Z, DASH_Z = 0.45, 0.5
DEBRIS = Ramp(0xE8, 0xEF)
SCORCH = 0xEE


def m(v):
    return v / UNIT_M


def rect(g, x0, x1, y0, y1, z, c):
    g.faces.append(([mklib.v3(p) for p in ((x0, y0, z), (x1, y0, z), (x1, y1, z), (x0, y1, z))], c))


def markings():
    """Both ends: 12 threshold stripes 30 m long, touchdown zone bars at 150, 300 and 450 m,
    the aiming point blocks at 400 m (ICAO for a runway this size, slightly simplified)."""
    g = Geo()
    for end in (-1, 1):
        def along(d0, d1):
            a, b = end * (HALF_L - m(d0)), end * (HALF_L - m(d1))
            return min(a, b), max(a, b)

        # Threshold stripes, 1.8 m wide with 1.8 m gaps, in two groups of six about the axis.
        y0, y1 = along(6, 36)
        for side in (-1, 1):
            for k in range(6):
                x = side * (m(3.6) + k * m(3.6))
                rect(g, min(x, x + side * m(1.8)), max(x, x + side * m(1.8)), y0, y1, MARK_Z, MARK)
        # Touchdown zone: three, two and one bar pairs, 22.5 m long.
        for d, bars in ((150, 3), (300, 2), (450, 1)):
            y0, y1 = along(d, d + 22.5)
            for side in (-1, 1):
                for k in range(bars):
                    x = side * (m(6) + k * m(3))
                    rect(g, min(x, x + side * m(1.8)), max(x, x + side * m(1.8)), y0, y1, MARK_Z, MARK)
        # Aiming point: a 45 m block on each side.
        y0, y1 = along(400, 445)
        for side in (-1, 1):
            x = side * m(6)
            rect(g, min(x, x + side * m(5)), max(x, x + side * m(5)), y0, y1, MARK_Z, MARK)
    return g


def centreline():
    """30 m dashes with 20 m gaps, 0.9 m wide, between the threshold stripes' inner ends."""
    g = Geo()
    y, top = -HALF_L + m(60), HALF_L - m(60)
    while y + m(30) <= top:
        rect(g, -m(0.45), m(0.45), y, y + m(30), DASH_Z, DASH)
        y += m(50)
    return g


def crater(g, cx, cy, r, rng, detail):
    """A crater: a scorched disc, a broken ring of upturned slabs round it, rubble in it."""
    n = 7 if detail else 5
    ring = []
    for k in range(n):
        a = (k + rng.uniform(-0.2, 0.2)) * math.tau / n
        ring.append((cx + math.cos(a) * r * rng.uniform(0.85, 1.15), cy + math.sin(a) * r * rng.uniform(0.85, 1.15)))
    g.faces.append(([mklib.v3((x, y, 0.6)) for x, y in ring], SCORCH))
    for k in range(n):
        if detail and rng.random() < 0.25:
            continue  # a gap in the rim
        (x0, y0), (x1, y1) = ring[k], ring[(k + 1) % n]
        h = rng.uniform(4.0, 8.0) if detail else 5.0
        ox, oy = (x0 + x1) / 2 - cx, (y0 + y1) / 2 - cy
        out = (x0 + x1) / 2 + ox * 0.35, (y0 + y1) / 2 + oy * 0.35
        top = ((x0 + x1) / 2 + ox * 0.1, (y0 + y1) / 2 + oy * 0.1, h)
        g.face([(x0, y0, 0.6), (x1, y1, 0.6), top], DEBRIS, (cx, cy, 0))
        g.face([(x1, y1, 0.6), (out[0], out[1], 0.6), top], DEBRIS, (cx, cy, -5))
        g.face([(out[0], out[1], 0.6), (x0, y0, 0.6), top], DEBRIS, (cx, cy, -5))
    if detail:
        for _ in range(3):
            x, y = cx + rng.uniform(-r, r) * 0.4, cy + rng.uniform(-r, r) * 0.4
            s = rng.uniform(2.5, 5.0)
            g.face([(x - s, y - s, 0.7), (x + s, y - s * 0.5, 0.7), (x, y + s, s)], DEBRIS, (x, y, -5))


def wreck(detail):
    """The destroyed runway: craters along it and across the taxiway crossing."""
    rng = random.Random(841)
    g = Geo()
    for cy, cx, r in ((-300, -5, 18), (-110, 8, 24), (45, -9, 20), (210, 4, 26), (345, -3, 19)):
        crater(g, cx, cy, r, rng, detail)
    return g


def rebuild(root):
    lods = sorted((c for c in root.children if "_lod" in c.name), key=lambda c: int(c.name.rsplit("_lod", 1)[1]))
    lod0 = lods[0]
    # The base mesh: everything but the old markings, and the new ones.
    me = lod0.data
    g = Geo()
    for p in me.polygons:
        colour = int(me.materials[p.material_index].name[4:7])
        flat_at = all(abs(me.vertices[i].co.z - MARK_Z) < 1e-3 for i in p.vertices)
        if colour != OLD_MARK and not (colour == MARK and flat_at):
            g.faces.append(([me.vertices[i].co.copy() for i in p.vertices], colour))
    for c in lod0.children:
        if c.name.startswith(lod0.name + "_lines_pal_"):
            colour = int(c.name.rsplit("_", 1)[1][:3])
            for e in c.data.edges:
                g.line(c.data.vertices[e.vertices[0]].co.copy(), c.data.vertices[e.vertices[1]].co.copy(), colour)
    g.add(markings())
    mklib.set_geometry(lod0, g)
    for lod, detail in zip(lods[:2], (True, False)):
        for c in lod.children:
            if c.name == lod.name + "_s11v0" and c.type == "MESH":
                mklib.set_geometry(c, centreline())
            if c.name == lod.name + "_s1v1" and c.type == "MESH":
                mklib.set_geometry(c, wreck(detail))


def main():
    stem = os.path.splitext(os.path.basename(bpy.data.filepath))[0]
    library = json.load(open(os.path.join(ROOT, "tools", "theater", "libraries", stem + ".json")))
    mklib.load_palette(os.path.join(ROOT, "assets", "palettes", "tod_%s.ron" % stem))
    root = bpy.data.objects["%s_%03d" % (stem, library["kinds"]["airbase"])]
    rebuild(root)
    bpy.ops.wm.save_mainfile()
    print("runway redrawn:", root.name)


main()
