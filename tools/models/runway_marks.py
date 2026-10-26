"""Redraws the runway of every theater master's airbase model: ICAO markings for its size
(threshold stripes, runway designation numbers from the landing headings, aiming point and
touchdown zone markings, side stripes, centreline) scaled to fit the runway, and the cratered
runway it becomes when destroyed. The pad, taxiways, aprons, lights and the coarser detail levels stay as they are.

    blender -b assets-src/blend/<file>.blend --python-exit-code 1 --python tools/models/runway_marks.py
    tools/models/export.sh <file>

Runs on the open master and saves it. The markings are painted white, which goes black at
dusk, so they are shown by day only: they go into `_s11v0` of the two finest levels (the
second one is drawn from 500 m to 4 km of the runway's middle, the whole approach), and the
base mesh of the finest level loses the marking faces of earlier runs (colours 203 and 224).
In the two finest levels `_s1v1` (destroyed) gets the craters. Model space: x east, y north
(along the runway), z up, in LOD2 units (4 WU, about 1.95 m). The runway is the long asphalt
face of the base mesh (80 x 1024 units: 156 m wide, 2000 m long). The markings are laid out as
ICAO's for a 60 m wide runway of its length, widened to its width (the side stripes on its
edges, the threshold stripes to 3 m from them) and lengthened by half again, so they keep
their proportions.
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
OLD_MARK, MARK, ASPHALT = 203, 224, 238
MARK_Z = 0.45
DEBRIS = Ramp(0xE8, 0xEF)
SCORCH = 0xEE


def m(v):
    return v / UNIT_M


def rect(g, x0, x1, y0, y1, z, c):
    g.faces.append(([mklib.v3(p) for p in ((x0, y0, z), (x1, y0, z), (x1, y1, z), (x0, y1, z))], c))


# Runway direction: the heading from the south end towards the north end (degrees true; the
# model runs north, and the theaters have no magnetic variation).
AXIS_DEG = 0.0
# The ICAO layout's half width (a 60 m runway) and how much longer the marks are drawn.
ICAO_HALF = 30.0
LENGTHEN = 1.5

# Blocky digits on a 3 x 5 cell grid (the middle column twice as wide): (column, row, columns wide, rows high) per bar, row 0 at
# the bottom.
DIGITS = {
    "0": ((0, 0, 3, 1), (0, 4, 3, 1), (0, 1, 1, 3), (2, 1, 1, 3)),
    "1": ((1, 0, 1, 5),),
    "2": ((0, 4, 3, 1), (2, 2, 1, 2), (0, 2, 3, 1), (0, 0, 1, 2), (1, 0, 2, 1)),
    "3": ((0, 0, 3, 1), (0, 2, 2, 1), (0, 4, 3, 1), (2, 1, 1, 3)),
    "4": ((0, 2, 1, 3), (1, 2, 1, 1), (2, 0, 1, 5)),
    "5": ((0, 4, 3, 1), (0, 2, 1, 2), (0, 2, 3, 1), (2, 0, 1, 2), (0, 0, 2, 1)),
    "6": ((0, 0, 1, 5), (1, 4, 2, 1), (1, 2, 2, 1), (1, 0, 2, 1), (2, 1, 1, 1)),
    "7": ((0, 4, 3, 1), (2, 0, 1, 4)),
    "8": ((0, 0, 1, 5), (2, 0, 1, 5), (1, 0, 1, 1), (1, 2, 1, 1), (1, 4, 1, 1)),
    "9": ((2, 0, 1, 5), (0, 4, 2, 1), (0, 2, 2, 1), (0, 3, 1, 1), (0, 0, 2, 1)),
}
# Digit size on a 60 m runway (metres): 18 m tall, 3 m bars round a 6 m wide middle column, 3 m
# between the two digits.
DIGIT_H, DIGIT_BAR, DIGIT_GAP = 18.0, 3.0, 3.0
DIGIT_COLS = (0.0, 1.0, 3.0, 4.0)


def designation(heading):
    """The runway number for a landing heading: tens of degrees, 36 for north."""
    n = round(heading / 10.0) % 36
    return "%02d" % (n or 36)


def markings(half_w, half_l):
    """Markings for a runway `half_w` by `half_l` units about the origin. Both ends, from the
    threshold 6 m in from the end: 16 threshold stripes, the designation past them, touchdown
    zone bars at 150 m (three), 450 and 600 m (two) and the aiming point at 250 m (a runway under
    2400 m); side stripes along the edges and the centreline between the designations. Across
    the runway every size is ICAO's times its width over 60 m, along it times `LENGTHEN`."""
    g = Geo()
    w = UNIT_M * half_w / ICAO_HALF  # metres across per ICAO metre
    edge = UNIT_M * half_w

    def box(x0, x1, y0, y1):
        rect(g, min(x0, x1), max(x0, x1), min(y0, y1), max(y0, y1), MARK_Z, MARK)

    for end in (-1, 1):
        # Distance d from this end along the runway (metres), and the pilot's right (looking
        # along the landing direction, away from this end).
        def at(d):
            return end * (half_l - m(d))

        right = -end
        # Threshold stripes: eight a side of a central gap twice their width, as wide as the
        # gaps between them, out to 3 m from the edges.
        stripe = (edge - 3.0) / 16.0
        t0, t1 = 6.0, 6.0 + 30 * LENGTHEN
        for side in (-1, 1):
            for k in range(8):
                x = side * stripe * (1 + 2 * k)
                box(m(x), m(x + side * stripe), at(t0), at(t1))
        # Designation: the landing heading's number, upright for the pilot, 12 m past them.
        heading = AXIS_DEG if end == -1 else AXIS_DEG + 180.0
        text = designation(heading)
        bar, height, gap = DIGIT_BAR * w, DIGIT_H * w, DIGIT_GAP * w
        cols = [c * bar for c in DIGIT_COLS]
        cell_h, digit_w = height / 5.0, cols[-1]
        width = 2 * digit_w + gap
        d_num = t1 + 12 * LENGTHEN
        for k, ch in enumerate(text):
            left = -width / 2 + k * (digit_w + gap)
            for col, row, cw, ch_ in DIGITS[ch]:
                x0, x1 = left + cols[col], left + cols[col + cw]
                d0, d1 = d_num + row * cell_h, d_num + (row + ch_) * cell_h
                box(m(right * x0), m(right * x1), at(d0), at(d1))
        # Aiming point: 45 m by 6 m, inner edges 11.25 m off the centreline.
        for side in (-1, 1):
            box(m(side * 11.25 * w), m(side * 17.25 * w), at(250), at(250 + 45 * LENGTHEN))
        # Touchdown zone: bars 22.5 m by 1.8 m, 1.5 m apart, from the aiming point's inner edge.
        for d, bars in ((150, 3), (450, 2), (600, 2)):
            for side in (-1, 1):
                for k in range(bars):
                    x = side * (11.25 + k * 3.3) * w
                    box(m(x), m(x + side * 1.8 * w), at(d), at(d + 22.5 * LENGTHEN))
    # Side stripes, 0.9 m wide, on the edges.
    for side in (-1, 1):
        box(m(side * edge), m(side * (edge - 0.9 * w)), -half_l + m(6), half_l - m(6))
    # Centreline: 30 m stripes with 20 m gaps, 0.9 m wide, between the designations.
    for y0, y1 in centreline_dashes(half_l, w):
        box(-m(0.45 * w), m(0.45 * w), y0, y1)
    return g


def centreline_dashes(half_l, w):
    """The centreline stripes (along-runway intervals), clear of the designations."""
    start = 6.0 + (30 + 12) * LENGTHEN + DIGIT_H * w + 12 * LENGTHEN
    y, top, out = -half_l + m(start), half_l - m(start), []
    while y + m(30 * LENGTHEN) <= top:
        out.append((y, y + m(30 * LENGTHEN)))
        y += m(50 * LENGTHEN)
    return out


def centreline_lines(half_l, w):
    """The centreline stripes as lines, which stay a pixel wide far out."""
    g = Geo()
    for y0, y1 in centreline_dashes(half_l, w):
        g.line(mklib.v3((0, y0, MARK_Z)), mklib.v3((0, y1, MARK_Z)), MARK)
    return g


def runway_size(me):
    """Half width and half length (units) of the runway: the longest asphalt face about the
    origin in the base mesh."""
    best = (0.0, 0.0)
    for p in me.polygons:
        if int(me.materials[p.material_index].name[4:7]) != ASPHALT:
            continue
        xs = [abs(me.vertices[i].co.x) for i in p.vertices]
        ys = [abs(me.vertices[i].co.y) for i in p.vertices]
        if max(ys) > best[1]:
            best = (max(xs), max(ys))
    return best


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
    half_w, half_l = runway_size(me)
    if half_w == 0:
        raise SystemExit("no runway face (colour %d) in %s" % (ASPHALT, lod0.name))
    print("runway %.0f x %.0f m" % (2 * half_w * UNIT_M, 2 * half_l * UNIT_M))
    w = UNIT_M * half_w / ICAO_HALF
    mklib.set_geometry(lod0, g)
    for lod, detail in zip(lods[:2], (True, False)):
        for c in lod.children:
            if c.name == lod.name + "_s11v0":
                marks = markings(half_w, half_l)
                if not detail:
                    marks.add(centreline_lines(half_l, w))
                mklib.set_geometry(c, marks)
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
