"""Shared geography for the theater builder: the projection, water, sides and the front.

A region module (regions/<id>.py) holds a theater's geography as (lon, lat) data; `load(id)`
imports it and gives it `ll`/`lls` (lon, lat -> theater metres, x east, y north, 0..512000)
and `Water()`, `Sides()`, `front_m()`, `dist_to_front()` bound to it.
"""

import functools
import importlib
import math
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)

THEATER_M = 512_000.0
WU = 512_000.0 / 1_048_576.0  # metres per world unit


class Projection:
    """Equirectangular, scaled at `lat_ref` and squeezed: `squeeze` real km per theater km."""

    def __init__(self, lon0, lat0, lat_ref, squeeze):
        self.lon0, self.lat0 = lon0, lat0
        self.kx = 111.32 * math.cos(math.radians(lat_ref)) / squeeze  # theater km per degree
        self.ky = 111.2 / squeeze

    def ll(self, lon, lat):
        return ((lon - self.lon0) * self.kx * 1000.0, (lat - self.lat0) * self.ky * 1000.0)

    def lls(self, pts):
        return [self.ll(p[0], p[1]) for p in pts]


def buffer_polyline(pts):
    """Outline of a river polyline (theater metres, width per point in km)."""
    left, right = [], []
    n = len(pts)
    for i in range(n):
        x, y, w = pts[i]
        if i == 0:
            dx, dy = pts[1][0] - x, pts[1][1] - y
        elif i == n - 1:
            dx, dy = x - pts[i - 1][0], y - pts[i - 1][1]
        else:
            ax, ay = x - pts[i - 1][0], y - pts[i - 1][1]
            bx, by = pts[i + 1][0] - x, pts[i + 1][1] - y
            la, lb = math.hypot(ax, ay), math.hypot(bx, by)
            dx, dy = ax / la + bx / lb, ay / la + by / lb
        l = math.hypot(dx, dy)
        nx, ny = -dy / l, dx / l
        h = w * 500.0
        left.append((x + nx * h, y + ny * h))
        right.append((x - nx * h, y - ny * h))
    return left + right[::-1]


def river_polygons(region):
    out = []
    for r in region.RIVERS:
        pts = [region.ll(lon, lat) + (w,) for lon, lat, w in r]
        out.append(buffer_polyline(pts))
    return out


def point_in(poly, x, y):
    inside = False
    n = len(poly)
    j = n - 1
    for i in range(n):
        xi, yi = poly[i]
        xj, yj = poly[j]
        if (yi > y) != (yj > y) and x < (xj - xi) * (y - yi) / (yj - yi) + xi:
            inside = not inside
        j = i
    return inside


class Water:
    """Point classification: 'sea', 'lake' (rivers, reservoirs, lakes) or None (land).

    The sea is `SEAS`, or, for a region with `LANDS`, everything outside them."""

    def __init__(self, region):
        self.seas = [region.lls(p) for p in region.SEAS]
        self.lands = [region.lls(p) for p in region.LANDS]
        self.lakes = [region.lls(p) for p in region.LAKES] + river_polygons(region)
        self.boxes = [(p, bbox(p)) for p in self.seas + self.lands + self.lakes]

    def _in(self, polys, x, y):
        return any(inside_box(bbox_cached(p, self.boxes), x, y) and point_in(p, x, y) for p in polys)

    def at(self, x, y):
        if self._in(self.seas, x, y):
            return "sea"
        if self.lands and not self._in(self.lands, x, y):
            return "sea"
        if self._in(self.lakes, x, y):
            return "lake"
        return None

    def polygons(self):
        """The outlines (their edges are the coasts and shores)."""
        return [("sea", p) for p in self.seas] + [("land", p) for p in self.lands] + [("lake", p) for p in self.lakes]


def bbox(p):
    xs = [q[0] for q in p]
    ys = [q[1] for q in p]
    return (min(xs), min(ys), max(xs), max(ys))


def bbox_cached(p, boxes):
    for q, b in boxes:
        if q is p:
            return b
    return bbox(p)


def inside_box(b, x, y):
    return b[0] <= x <= b[2] and b[1] <= y <= b[3]


class Sides:
    """The friendly side's ground: `BLUE`, a polygon or a list of them."""

    def __init__(self, region):
        blue = region.BLUE
        polys = blue if blue and isinstance(blue[0][0], (list, tuple)) else [blue]
        self.blue = [region.lls(p) for p in polys]

    def blue_at(self, x, y):
        return any(point_in(p, x, y) for p in self.blue)


def front_m(region):
    """The front line in theater metres with its intensity: [(x, y, w)]."""
    return [region.ll(lon, lat) + (w,) for lon, lat, w in region.FRONT]


def dist_to_front(front, x, y):
    best = 1e18
    for a, b in zip(front, front[1:]):
        ax, ay, bx, by = a[0], a[1], b[0], b[1]
        dx, dy = bx - ax, by - ay
        t = max(0.0, min(1.0, ((x - ax) * dx + (y - ay) * dy) / (dx * dx + dy * dy)))
        best = min(best, math.hypot(x - ax - dx * t, y - ay - dy * t))
    return best


def load(region_id):
    """The region module with its projection and geography helpers attached."""
    r = importlib.import_module("regions." + region_id)
    proj = Projection(**r.PROJECTION)
    r.ID = region_id
    r.THEATER_M, r.WU = THEATER_M, WU
    r.ll, r.lls = proj.ll, proj.lls
    r.bbox, r.point_in = bbox, point_in
    r.Water = functools.partial(Water, r)
    r.Sides = functools.partial(Sides, r)
    r.front_m = functools.partial(front_m, r)
    r.dist_to_front = lambda x, y, front=None: dist_to_front(front or front_m(r), x, y)
    for name, default in (("SEAS", []), ("LANDS", []), ("NEUTRAL", set()), ("CITY_SAMS", []), ("SLAG_HEAP_TOWNS", set()),
                          ("LAKES", []), ("RIVERS", []), ("FRONT", []), ("MOUNTAINS", []),
                          ("FORESTS", []), ("NPPS", []), ("LAUNCH_SITES", []), ("SITES", []), ("BOATS", []),
                          ("AIRSTRIPS", []), ("CARRIER", None), ("OBJECTS", []), ("SHIPS", [])):
        if not hasattr(r, name):
            setattr(r, name, default)
    return r
