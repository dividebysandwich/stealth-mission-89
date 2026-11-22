"""Assembles a theater from its region module (regions/<id>.py) and the models built by models.py.

    python3 tools/theater/assemble.py ukraine <new.glb>

Writes:
- assets/models/ukraine.glb: Central Europe's models (as ukraine_000..ukraine_093, the theater shares their
  meaning: airbases, SAM sites, depots, ...) followed by the new ones.
- assets/theaters/ukraine.ron: tile grid and tile objects, the world (objects, names) and the extras the remake
  reads (front line, towns, model attributes).
- assets/mission/theaters/ukraine.json: the same for the mission generator, with the models'
  target roles (Central Europe's) and the theater's own date and grid letters.
- assets/palettes/tod_ukraine.ron: Central Europe's palettes with steppe ground and dark soil.
"""

import json
import math
import os
import random
import re
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.abspath(os.path.join(HERE, "..", ".."))
sys.path.insert(0, HERE)
sys.path.insert(0, os.path.join(ROOT, "tools", "models"))

import common  # noqa: E402
import jsonfmt  # noqa: E402
import kerch  # noqa: E402
from glbio import append, read, subset, write  # noqa: E402

geo = common.load(sys.argv[1])

UNIT_WU = {1: 1, 2: 4, 3: 16, 4: 64}
MOUNTAIN_BASE = 110  # models.py's mountain ranges
THEATER_IDS = ["libya", "persian_gulf", "north_cape", "central_europe", "vietnam", "middle_east", "desert_storm", "cuba",
               "korea", "ukraine", "gulf_of_finland", "suwalki"]
DIM = {1: 256, 2: 64, 3: 16}

LIBRARY = json.load(open(os.path.join(HERE, "libraries", geo.LIBRARY + ".json")))
# The region's own names for library models (the kinds its layouts use) come on top.
KINDS = dict(LIBRARY["kinds"], **getattr(geo, "KINDS", {}))
# The terrain level each model is drawn for (its units); most are LOD1 objects.
LEVEL = {int(k): v for k, v in LIBRARY.get("levels", {}).items()}


def K(kind):
    """The library model of a kind of object, None if the theater has none."""
    return KINDS.get(kind)


BASE, DEPOT, SAM_RADAR, BRIDGE, BOAT, BUILDINGS, OTH = (K(k) for k in (
    "airbase", "depot", "sam_radar", "bridge", "patrol_boat", "buildings", "oth_radar"))
HOUSES, HOUSES2 = K("houses"), K("houses2")
CARRIER = K("carrier")

# Room on land all round an object (metres): its footprint and the shore's refinement.
DRY_MARGIN = 150
# Half extents of an airfield, the runway and its aprons (metres east, north; runways run
# north-south).
FIELD_HALF = (700, 1300)
ROAD_STEP = 150.0  # metres between a supply road's points at most

# Layouts of a LOD1 cell (2 km square): (metres east, metres north of the cell centre, kind).
# The first entry of a site is its main object (the SAM sites' radar).
TEMPLATES = {
    # Airbase support on the airfield model's aprons (east 170-500 m from the runway's axis,
    # 165 m either side of its middle; west 140-500 m, the parallel taxiway along its inner
    # edge): hangars along the east apron's far edge, the tower on the west apron's south side,
    # its view of the runway clear, short of the parallel taxiway (the main menu's scene is laid out from it), aircraft
    # on both aprons; the fuel farm off the north-west corner of the field.
    "base_support": [(-265, -150, "tower"), (430, -110, "hangar"), (430, 0, "hangar"), (430, 110, "hangar"),
                     (300, 60, "airplane"), (-300, -80, "airplane"), (-300, 80, "airplane"),
                     (-650, 650, "tank_farm")],
    "base_support_blue": [(-265, -150, "tower"), (430, -110, "hangar"), (430, 0, "hangar"), (430, 110, "hangar"),
                          (-650, 650, "tank_farm")],
    # A fixed SAM site: the radar in the middle of six launchers on a hexagon.
    "sam": [(0, 0, "sam_radar")] + [(round(420 * math.cos(math.radians(30 + 60 * k))),
                                     round(420 * math.sin(math.radians(30 + 60 * k))), "sa_missile") for k in range(6)],
    # A mobile SAM battery: its radar and five launchers on a pentagon.
    "msam": [(0, 0, "sam_radar")] + [(round(450 * math.sin(math.radians(72 * k))),
                                      round(450 * math.cos(math.radians(72 * k))), "mobile_sam") for k in range(5)],
    "hq": [(0, 0, "headquarters"), (-350, 0, "depot"), (-500, -450, "houses"), (650, 150, "houses"), (-600, 500, "houses"),
           (0, 420, "mobile_sam"), (0, -420, "mobile_sam")],
    "depot": [(-300, 0, "depot"), (0, 0, "depot"), (300, 0, "depot"), (0, 450, "tank_farm"), (450, -350, "houses"),
              (0, -600, "mobile_sam")],
    "tankfarm": [(-150, -150, "tank_farm"), (150, -150, "tank_farm"), (-150, 150, "tank_farm"), (150, 150, "tank_farm"),
                 (550, 0, "houses"), (-500, -450, "houses")],
    "oth": [(0, 0, "oth_radar"), (300, 250, "depot")],
    # Armour along a road, with its air defence.
    "armor": [(-700 + 180 * k, -500 + 120 * k, "tank") for k in range(6)] + [(400, 300, "mobile_sam"), (800, -200, "houses")],
    "ss": [(0, 0, "ss_missile"), (-250, 200, "depot")],
    "launch": [(0, 0, "depot"), (-300, -200, "tank"), (0, -250, "tank"), (300, -200, "tank"), (0, 400, "hangar")],
    "pow": [(0, 0, "pow_camp"), (350, 120, "houses")],
    "farm_a": [(0, 0, "windmill"), (180, 90, "houses"), (-220, 310, "houses"), (-60, -400, "houses2")],
    "farm_b": [(300, 300, "houses2"), (-400, -200, "houses")],
    "farm_c": [(0, 0, "houses")],
}


def base_support(aprons):
    """The airbase support layouts for an airfield model with other aprons than Central
    Europe's: `aprons` lists its aprons as (west, east, south, north) metres from the runway's
    middle. Hangars along the far edge of the biggest apron east of the runway (or west), the
    tower at the south end of the other one (or of the same, away from the hangars), aircraft
    on both, the fuel farm off the field."""
    east = max((a for a in aprons if a[0] > 0), key=lambda a: (a[1] - a[0]) * (a[3] - a[2]), default=None)
    west = max((a for a in aprons if a[1] < 0), key=lambda a: (a[1] - a[0]) * (a[3] - a[2]), default=None)
    main, other = (east, west) if east else (west, None)
    side = 1 if main[0] > 0 else -1
    far = main[1] - 70 if side > 0 else main[0] + 70
    mid = (main[2] + main[3]) / 2
    step = min(110, (main[3] - main[2] - 80) / 2)
    rows = (mid - step, mid, mid + step)
    hangars = [(round(far), round(n), "hangar") for n in rows]
    near = main[0] + 100 if side > 0 else main[1] - 100
    if other:
        tower = (round((other[0] + other[1]) / 2), round(other[2] + 30), "tower")
        planes = [(round(near), round(mid + 60), "airplane"),
                  (round((other[0] + other[1]) / 2 - 35), round((other[2] + other[3]) / 2 + 40), "airplane")]
    else:
        tower = (round(near), round(main[2] + 30), "tower")
        planes = [(round(near), round(mid + 60), "airplane"), (round(near), round(mid - 40), "airplane")]
    fuel = [(-650, 650, "tank_farm")]
    return {"base_support": [tower] + hangars + planes + fuel, "base_support_blue": [tower] + hangars + fuel}


if hasattr(geo, "APRONS"):
    TEMPLATES.update(base_support(geo.APRONS))
TEMPLATES.update(getattr(geo, "TEMPLATES", {}))
# Patterns for the empty LOD2 cells away from towns and woods: {(col, row) of a LOD1 cell in
# it: template}; the first is the empty one.
FILLER = getattr(geo, "FILLER", [
    {},
    {(1, 2): "farm_a"},
    {(2, 1): "farm_b", (0, 3): "farm_c"},
    {(3, 0): "farm_c"},
    {(1, 1): "farm_b", (3, 2): "farm_a"},
])

def unit_ids(section):
    """The ids of a units.ron list, in index order."""
    text = open(os.path.join(ROOT, "assets", "sim", "units.ron")).read()
    start = text.index(section + ": [")
    end = text.index("\n    ],", start)
    return re.findall(r'\(id: "(\w+)"', text[start:end])


RADARS = unit_ids("radars")
# Radar types (units.ron ids) of the region's fixed and mobile SAM sites and its patrol boats.
SAM_TYPES = [RADARS.index(r) for r in geo.SAM_RADARS]
MOBILE_SAM_TYPES = [RADARS.index(r) for r in geo.MOBILE_SAM_RADARS]
BOAT_TYPES = [RADARS.index(r) for r in geo.BOAT_RADARS]
OTH_TYPE = RADARS.index("oth")
# SAM systems with models of their own (when the library has them): the kinds a site of that
# radar type places instead of the generic radar, fixed site launchers and mobile launchers.
SITE_KINDS = {
    "sa17": {"sam_radar": "sa17_radar", "sa_missile": "sa17_launcher", "mobile_sam": "sa17_launcher"},
    "sa22": {"sam_radar": "sa22_radar", "sa_missile": "sa22_unit", "mobile_sam": "sa22_unit"},
    "sa15": {"sam_radar": "sa15_radar"},
}


def site_kinds(unit_type):
    """The kinds a SAM site of radar type `unit_type` places instead of the generic ones."""
    return {k: v for k, v in SITE_KINDS.get(RADARS[unit_type], {}).items() if K(v) is not None}


def radar_shape(unit_type):
    """The model of a SAM site's radar (its world object)."""
    return K(site_kinds(unit_type).get("sam_radar", "sam_radar"))


def wu(x_m):
    return x_m / geo.WU


class Layout:
    def __init__(self, meta):
        self.meta = meta
        self.water = geo.Water()
        self.sides = geo.Sides()
        self.front = geo.front_m()
        self.objs = {1: {}, 2: {}, 3: {}, 4: {}}
        self.rng = random.Random(2026)

    # --- placement -----------------------------------------------------------------------

    def cell(self, lod, x_m, y_m):
        size = 4096 * UNIT_WU[lod]
        return int(wu(x_m) // size), int(wu(y_m) // size)

    def put(self, lod, x_m, y_m, shape):
        """A tile object at a point; returns its (lod, col, row, ox, oy)."""
        u = UNIT_WU[lod]
        c, r = self.cell(lod, x_m, y_m)
        ox = int(round((wu(x_m) - (c * 4096 + 2048) * u) / u))
        oy = int(round((wu(y_m) - (r * 4096 + 2048) * u) / u))
        self.objs[lod].setdefault((c, r), []).append((ox, oy, shape))
        return (lod, c, r, ox, oy)

    def put_at_cell(self, lod, c, r, ox, oy, shape):
        self.objs[lod].setdefault((c, r), []).append((ox, oy, shape))
        return (lod, c, r, ox, oy)

    @staticmethod
    def map_pos(t):
        lod, c, r, ox, oy = t
        u = UNIT_WU[lod]
        east = (c * 4096 + 2048 + ox) * u
        north = (r * 4096 + 2048 + oy) * u
        return east >> 5, 0x8000 - (north >> 5)

    @staticmethod
    def centre_m(lod, c, r):
        u = UNIT_WU[lod]
        return (c * 4096 + 2048) * u * geo.WU, (r * 4096 + 2048) * u * geo.WU

    def template(self, name, c1, r1, main=True, kinds=None):
        """A LOD1 template in cell (c1, r1); returns the placements by shape. Kinds the
        theater has no model for are left out, and so are the objects that would stand in
        water (but not the main one if `main`); models of a coarser level go on that level.
        `kinds` replaces some of its kinds (a SAM system's own models)."""
        out = []
        cx, cy = self.centre_m(1, c1, r1)
        for k, (dx, dy, kind) in enumerate(TEMPLATES[name]):
            shape = K((kinds or {}).get(kind, kind))
            if shape is None or ((k > 0 or not main) and not self.dry(cx + dx, cy + dy)):
                continue
            if LEVEL.get(shape, 1) == 1:
                out.append((shape, self.put_at_cell(1, c1, r1, round(wu(dx)), round(wu(dy)), shape)))
            else:
                out.append((shape, self.put(LEVEL[shape], cx + dx, cy + dy, shape)))
        return out

    def land(self, x, y):
        return self.water.at(x, y) is None

    def dry(self, x, y, r=DRY_MARGIN):
        """Land at the point and all round it within `r` metres."""
        return self.land(x, y) and all(
            self.land(x + r * math.cos(k * math.tau / 8), y + r * math.sin(k * math.tau / 8)) for k in range(8))

    def template_dry(self, name, c1, r1):
        """Every object of a LOD1 template in cell (c1, r1) on dry land."""
        cx, cy = self.centre_m(1, c1, r1)
        return all(self.dry(cx + dx, cy + dy) for dx, dy, kind in TEMPLATES[name] if K(kind) is not None)

    def land_cell(self, lod, c, r, n=5, margin=0.0):
        size = 4096 * UNIT_WU[lod] * geo.WU
        for i in range(n):
            for j in range(n):
                x = (c + (i + 0.5) / n) * size
                y = (r + (j + 0.5) / n) * size
                if not self.land(x, y):
                    return False
        return True

    def red(self, x, y):
        return not self.sides.blue_at(x, y)

    # --- content -------------------------------------------------------------------------

    def build(self):
        self.ground_units = []  # (name, placement, unit_type, flags)
        self.red_bases = []
        self.blue_bases = []
        self.cities = []  # (name, placement or point, red, shape)
        self.npp_sites = []  # (name, placement, red, shape)
        self.launch = []  # (name, placement)
        self.neutral_bases = []
        self.fields = []  # runway centres (metres)
        self.field_of = {}  # airbase name -> runway centre (metres)
        self.airstrips = []
        self.carriers = []
        self.airbases()
        self.npps()
        self.towns()
        self.sites()
        self.kerch_bridge()
        self.road_net = self.roads()
        self.city_sams()
        self.launch_sites()
        self.objects()
        self.boats()
        self.ships()
        self.mountains()
        self.generic()

    def airbases(self):
        for name, lon, lat, friendly in geo.AIRBASES:
            x, y = geo.ll(lon, lat)
            if not (2048 <= x < geo.THEATER_M - 2048 and 2048 <= y < geo.THEATER_M - 2048):
                print("airbase beyond the theater's edge:", name)
                continue
            c1, r1 = self.field_cell(name, x, y)
            c2, r2 = c1 >> 2, r1 >> 2
            # Runway centred on the LOD1 cell, the model on the LOD2 level.
            cx, cy = (c1 * 4096 + 2048), (r1 * 4096 + 2048)
            ox2 = (cx - (c2 * 4096 + 2048) * 4) // 4
            oy2 = (cy - (r2 * 4096 + 2048) * 4) // 4
            t = self.put_at_cell(2, c2, r2, ox2, oy2, BASE)
            self.fields.append(self.centre_m(1, c1, r1))
            self.field_of[name] = self.centre_m(1, c1, r1)
            self.template("base_support_blue" if friendly else "base_support", c1, r1)
            if friendly is None:
                self.neutral_bases.append((name, t))
            elif friendly:
                self.blue_bases.append((name, t))
            else:
                self.red_bases.append((name, t))
                # SAM site two cells to the side, inside the same LOD2 cell, or the nearest
                # cell with room on land.
                sc = c1 - 2 if c1 & 3 >= 2 else c1 + 2
                cell = (sc, r1) if self.template_dry("sam", sc, r1) else self.site_cell(*self.centre_m(1, sc, r1), "sam")
                if cell is None:
                    print("no room for the SAM site of", name)
                    continue
                ut = self.rng.choice(SAM_TYPES)
                radar = self.template("sam", *cell, kinds=site_kinds(ut))[0][1]
                self.ground_units.append((name, radar, ut, 0))

        for name, lon, lat, friendly in geo.AIRSTRIPS:
            x, y = geo.ll(lon, lat)
            self.airstrips.append((name, self.put(LEVEL.get(K("airstrip"), 2), x, y, K("airstrip")), friendly))
        # The friendly carrier groups: `CARRIER`, one (name, lon, lat) or a list of them.
        groups = geo.CARRIER or []
        for name, lon, lat in [groups] if groups and isinstance(groups[0], str) else groups:
            x, y = geo.ll(lon, lat)
            self.carriers.append((name, self.put(2, x, y, CARRIER)))

    def npps(self):
        for name, lon, lat, kind in geo.NPPS:
            x, y = geo.ll(lon, lat)
            shape = {"znpp": 94, "rostov": 95, "south": 96, "kursk": 97, "novovoronezh": 98}[kind]
            self.npp_sites.append((name, self.put(2, x, y, shape), self.red(x, y), shape))

    def towns(self):
        """Tile objects of the towns: central buildings (or ruins), slag heaps in the Donbas.
        Only the named towns and the ruined ones, within the central LOD1 cell (the scenery
        builds the towns themselves)."""
        self.town_sites = []
        for name, lon, lat, kind, r, ruin, wld in geo.TOWNS:
            x, y = self.ashore(*geo.ll(lon, lat))
            self.town_sites.append((name, x, y, r, kind, ruin))
            red = self.red(x, y)
            if not (wld or ruin >= 0.5) or not self.dry(x, y):
                continue
            if not (0 <= x < geo.THEATER_M and 0 <= y < geo.THEATER_M):
                continue  # beyond the theater's edge: scenery only
            rng = random.Random(name)
            c1, r1 = self.cell(1, x, y)
            cx, cy = self.centre_m(1, c1, r1)
            ruined = lambda: rng.random() < ruin
            central = rng.choice([99, 100, 101]) if ruin >= 0.5 else BUILDINGS
            if self.on_field(x, y):
                # A town whose airfield is in it: its centre beside the field.
                fx, _ = min(self.fields, key=lambda f: math.hypot(x - f[0], y - f[1]))
                x = fx + (900 if x >= fx else -900)
            t = self.put(1, x, y, central)
            self.cities.append((name, t, red, central))
            extra = {2: 3, 1: 2, 0: 1}[kind]
            for k in range(extra):
                a = rng.uniform(0, math.tau)
                d = rng.uniform(250, 700)
                px, py = x + math.cos(a) * d, y + math.sin(a) * d
                if self.cell(1, px, py) != (c1, r1) or not self.dry(px, py) or self.on_field(px, py):
                    continue
                shape = rng.choice([99, 100, 101]) if ruined() else rng.choice([BUILDINGS, HOUSES, HOUSES2])
                self.put(1, px, py, shape)
            if name in geo.SLAG_HEAP_TOWNS:
                # Slag heaps at the edge of the mining towns.
                for k in range(3):
                    a = rng.uniform(0, math.tau)
                    d = r + rng.uniform(250, 1200)
                    px, py = x + math.cos(a) * d, y + math.sin(a) * d
                    if self.land(px, py):
                        self.put(1, px, py, 103)

    def field_dry(self, c1, r1):
        """Whether an airfield centred on LOD1 cell (c1, r1) stays clear of all water."""
        cx, cy = self.centre_m(1, c1, r1)
        hx, hy = FIELD_HALF[0] + DRY_MARGIN, FIELD_HALF[1] + DRY_MARGIN
        nx, ny = math.ceil(hx / 100), math.ceil(hy / 100)
        return all(self.land(cx + hx * i / nx, cy + hy * j / ny)
                   for i in range(-nx, nx + 1) for j in range(-ny, ny + 1))

    def field_cell(self, name, x, y):
        """LOD1 cell for an airfield: the one under it, or the nearest one within 12 km whose
        field is all on dry land (coastal and riverside bases)."""
        c1, r1 = self.cell(1, x, y)
        if self.field_dry(c1, r1):
            return c1, r1
        best = None
        for dc in range(-6, 7):
            for dr in range(-6, 7):
                cx, cy = self.centre_m(1, c1 + dc, r1 + dr)
                d = math.hypot(cx - x, cy - y)
                if d < 12_000 and (best is None or d < best[0]) and self.field_dry(c1 + dc, r1 + dr):
                    best = (d, c1 + dc, r1 + dr)
        if best is None:
            print("airbase on water, no dry ground near:", name)
            return c1, r1
        print("airbase %s moved %.1f km off the water" % (name, best[0] / 1000))
        return best[1], best[2]

    def on_field(self, x, y, margin=0.0):
        """Within an airfield: the runway and its aprons (runways run north-south)."""
        return any(abs(x - fx) < FIELD_HALF[0] + margin and abs(y - fy) < FIELD_HALF[1] + margin
                   for fx, fy in self.fields)

    def ashore(self, x, y):
        """The point, or the nearest dry land within 4 km (a town on its river or coast)."""
        if self.dry(x, y):
            return x, y
        for d in range(250, 4001, 250):
            for k in range(16):
                a = k * math.tau / 16
                px, py = x + math.cos(a) * d, y + math.sin(a) * d
                if self.dry(px, py):
                    return px, py
        return x, y

    def site_cell(self, x, y, name):
        """LOD1 cell for a site: the one under it, or the nearest one with all of the site's
        template on land."""
        c1, r1 = self.cell(1, x, y)
        best = None
        for dc in range(-3, 4):
            for dr in range(-3, 4):
                cx, cy = self.centre_m(1, c1 + dc, r1 + dr)
                if (self.land_cell(1, c1 + dc, r1 + dr, 3) and self.template_dry(name, c1 + dc, r1 + dr)
                        and (c1 + dc, r1 + dr) not in self.objs[1]
                        and not self.on_field(cx, cy, 900)):
                    d = dc * dc + dr * dr
                    if best is None or d < best[0]:
                        best = (d, c1 + dc, r1 + dr)
        return best[1:] if best else None

    def sites(self):
        for kind, lon, lat, name in geo.SITES:
            x, y = geo.ll(lon, lat)
            if kind == "bridge":
                bx, by = self.on_river(x, y)
                self.put(1, bx, by, BRIDGE)
                continue
            cell = self.site_cell(x, y, kind)
            if cell is None:
                print("no room for", kind, name)
                continue
            # A mobile SAM battery (its radar in the middle): its type, its models.
            ut = self.rng.choice(MOBILE_SAM_TYPES) if TEMPLATES[kind][0][2] == "sam_radar" else None
            placed = self.template(kind, *cell, kinds=site_kinds(ut) if ut is not None else None)
            for shape, t in placed:
                if ut is not None and shape == radar_shape(ut):
                    self.ground_units.append((name, t, ut, 0))
                if shape == OTH:
                    self.ground_units.append((name, t, OTH_TYPE, 0))

    def kerch_bridge(self):
        """The Crimean bridge's arches and viaduct segments (models.py), each at its snapped
        LOD2 origin."""
        if "kerch_bridge" not in geo.EXTRA_MODELS:
            return
        for shape, _, _, (x, y) in kerch.Bridge(geo).pieces():
            self.put(kerch.LEVEL, x, y, shape)

    def roads(self):
        """The supply roads (`ROADS` of the region): {"nodes": [(name, x, y)], "edges": [(a, b,
        name, bridge, [(x, y, z)])]}, metres; z is the bridge's deck over the sea (0 on land).
        Spurs join the `ROAD_BASES` airfields to the nearest point of a road (that edge is split
        there); points every `ROAD_STEP` metres at most."""
        if not getattr(geo, "ROADS", None):
            return None
        nodes, edges = [], []
        index = {}

        def node(name, x, y):
            index[name] = len(nodes)
            nodes.append((name, x, y))
            return index[name]

        for name, (lon, lat) in geo.ROAD_NODES.items():
            node(name, *geo.ll(lon, lat))
        bridge = kerch.Bridge(geo) if "kerch_bridge" in geo.EXTRA_MODELS else None
        for road, items in geo.ROADS:
            a, pts = None, []
            k = 0
            while k < len(items):
                item = items[k]
                if item == "Kerch bridge":
                    # The deck from its east end to its west end (the roads come from the east).
                    # From the foot of the east embankment to the foot of the west one.
                    n = kerch.ROAD[0]
                    s_e, s_w = bridge.length + kerch.RAMP, -kerch.RAMP
                    e = node("Kerch bridge E", *bridge.point(s_e, n))
                    w = node("Kerch bridge W", *bridge.point(s_w, n))
                    pts.append((*bridge.point(s_e, n), 0.0))
                    edges.append((a, e, road, False, pts))

                    def deck_z(s):
                        if s < 0.0:
                            return bridge.deck(0.0) * (1 + s / kerch.RAMP)
                        if s > bridge.length:
                            return bridge.deck(bridge.length) * (1 - (s - bridge.length) / kerch.RAMP)
                        return bridge.deck(s)

                    steps = max(1, math.ceil((s_e - s_w) / ROAD_STEP))
                    along = [s_e + (s_w - s_e) * j / steps for j in range(steps + 1)]
                    # (The ends of the decks themselves, where the slope changes.)
                    along = sorted(set(along + [0.0, bridge.length]), reverse=True)
                    deck = [(*bridge.point(s, n), deck_z(s)) for s in along]
                    # The road arch (its own model, a target): a road cut while it is down.
                    edges.append((e, w, road, {"arch": bridge.point(kerch.ARCH_S, n), "shape": kerch.ROAD_ARCH}, deck))
                    a, pts = w, [(*bridge.point(s_w, n), 0.0)]
                elif isinstance(item, str):
                    i = index[item]
                    pts.append((nodes[i][1], nodes[i][2], 0.0))
                    if a is not None:
                        edges.append((a, i, road, False, pts))
                    a, pts = i, [(nodes[i][1], nodes[i][2], 0.0)]
                else:
                    pts.append((*geo.ll(*item), 0.0))
                k += 1
        # Spurs to the airfields: from the road's nearest point to beside the runway.
        for base in getattr(geo, "ROAD_BASES", []):
            if base not in self.field_of:
                print("road base without a field:", base)
                continue
            fx, fy = self.field_of[base]
            best = None
            for ei, (a, b, road, br, pts) in enumerate(edges):
                if br:
                    continue
                for j in range(len(pts) - 1):
                    (ax, ay, _), (bx, by, _) = pts[j], pts[j + 1]
                    dx, dy = bx - ax, by - ay
                    t = max(0.0, min(1.0, ((fx - ax) * dx + (fy - ay) * dy) / max(dx * dx + dy * dy, 1e-9)))
                    px, py = ax + dx * t, ay + dy * t
                    d = math.hypot(fx - px, fy - py)
                    if best is None or d < best[0]:
                        best = (d, ei, j, t, px, py)
            d, ei, j, t, px, py = best
            a, b, road, br, pts = edges[ei]
            if t <= 0.0 and j == 0:
                junction = a
            elif t >= 1.0 and j == len(pts) - 2:
                junction = b
            else:
                junction = node(base + " junction", px, py)
                edges[ei] = (a, junction, road, br, pts[:j + 1] + [(px, py, 0.0)])
                edges.append((junction, b, road, br, [(px, py, 0.0)] + pts[j + 1:]))
            # The spur ends beside the runway, on the road's side of the field.
            side = 1.0 if px > fx else -1.0
            ex, ey = fx + side * (FIELD_HALF[0] + 200.0), fy + max(-FIELD_HALF[1], min(FIELD_HALF[1], py - fy)) * 0.5
            end = node(base + " base", ex, ey)
            edges.append((junction, end, base + " spur", False, [(px, py, 0.0), (ex, ey, 0.0)]))
        out_edges = []
        for a, b, road, br, pts in edges:
            fine = [pts[0]]
            for p, q in zip(pts, pts[1:]):
                n = max(1, math.ceil(math.hypot(q[0] - p[0], q[1] - p[1]) / ROAD_STEP))
                fine += [(p[0] + (q[0] - p[0]) * j / n, p[1] + (q[1] - p[1]) * j / n, p[2] + (q[2] - p[2]) * j / n)
                         for j in range(1, n + 1)]
            wet = sum(1 for x, y, z in fine if z == 0.0 and not self.land(x, y))
            if wet > 3:
                print("road %s: %d points in water" % (road, wet))
            out_edges.append((a, b, road, br, fine))
        return {"nodes": nodes, "edges": out_edges, "entries": [index[n] for n in getattr(geo, "ROAD_ENTRIES", [])]}

    def launch_sites(self):
        for name, lon, lat in geo.LAUNCH_SITES:
            cell = self.site_cell(*geo.ll(lon, lat), "launch")
            if cell is None:
                print("no room for launch site", name)
                continue
            self.launch.append((name, self.template("launch", *cell)[0][1]))

    def city_sams(self):
        for name in geo.CITY_SAMS:
            t = next(t for t in geo.TOWNS if t[0] == name)
            x, y = geo.ll(t[1], t[2])
            for a in range(8):
                ang = a * math.tau / 8 + 0.4
                px, py = x + math.cos(ang) * (t[4] + 4000), y + math.sin(ang) * (t[4] + 4000)
                cell = self.site_cell(px, py, "sam")
                if cell:
                    ut = self.rng.choice(SAM_TYPES)
                    radar = self.template("sam", *cell, kinds=site_kinds(ut))[0][1]
                    self.ground_units.append((name, radar, ut, 0))
                    break

    def on_river(self, x, y):
        best = None
        for r in geo.RIVERS:
            pts = [geo.ll(lon, lat) for lon, lat, _ in r]
            for a, b in zip(pts, pts[1:]):
                dx, dy = b[0] - a[0], b[1] - a[1]
                t = max(0.0, min(1.0, ((x - a[0]) * dx + (y - a[1]) * dy) / (dx * dx + dy * dy)))
                p = (a[0] + dx * t, a[1] + dy * t)
                d = math.hypot(p[0] - x, p[1] - y)
                if best is None or d < best[0]:
                    best = (d, p)
        return best[1]

    def objects(self):
        """Single objects where the region puts them (special targets, oil rigs)."""
        for kind, lon, lat, _ in geo.OBJECTS:
            shape = K(kind)
            x, y = geo.ll(lon, lat)
            self.put(LEVEL.get(shape, 1), x, y, shape)

    def ships(self):
        """Ships at sea; the armed ones (with a radar) are ground units like the boats."""
        for kind, lon, lat, radar in geo.SHIPS:
            shape = K(kind)
            x, y = geo.ll(lon, lat)
            if self.water.at(x, y) != "sea":
                print("ship on land at", lon, lat)
                continue
            t = self.put(LEVEL.get(shape, 1), x, y, shape)
            if radar:
                self.boat_units.append((t, RADARS.index(radar), shape))

    def boats(self):
        self.boat_units = []
        for k, (lon, lat) in enumerate(geo.BOATS):
            x, y = geo.ll(lon, lat)
            if self.water.at(x, y) != "sea":
                print("boat on land at", lon, lat)
                continue
            t = self.put(1, x, y, BOAT)
            self.boat_units.append((t, BOAT_TYPES[k % len(BOAT_TYPES)], BOAT))

    def mountains(self):
        """The mountain ranges built by models.py, on the coarsest level."""
        for k, (lon, lat, *_) in enumerate(geo.MOUNTAINS):
            x, y = geo.ll(lon, lat)
            self.put(4, x, y, MOUNTAIN_BASE + k)

    def forest_density(self, x, y):
        d = 0.0
        for lon, lat, rad, dens in geo.FORESTS:
            fx, fy = geo.ll(lon, lat)
            k = 1.0 - math.hypot(x - fx, y - fy) / (rad * 1000.0)
            d = max(d, dens * k)
        return d

    def generic(self):
        """Forest patches and farmsteads in the LOD2 cells without anything else, from a few
        shared patterns (each distinct cell content costs one of the 256 tile types)."""
        towns = [(x, y, r) for _, x, y, r, _, _ in self.town_sites]
        rng = random.Random(7)
        busy2 = set(self.objs[2])
        busy2 |= {(c >> 2, r >> 2) for c, r in self.objs[1]}
        patterns = FILLER
        sparse = getattr(geo, "SPARSE_FOREST", True)
        self.pattern_cells = {}
        for r2 in range(64):
            for c2 in range(64):
                if (c2, r2) in busy2:
                    continue
                if not self.land_cell(2, c2, r2, 5):
                    continue
                cx, cy = self.centre_m(2, c2, r2)
                if any(math.hypot(cx - x, cy - y) < r + 4500 for x, y, r in towns):
                    continue
                d = self.forest_density(cx, cy) + rng.uniform(-0.12, 0.12)
                forest = None
                if d > 0.55:
                    forest = 107
                elif d > 0.4:
                    forest = 106
                elif d > 0.25:
                    forest = 105
                elif d > 0.12 or (rng.random() < 0.03 and sparse):
                    forest = 104
                if forest is not None:
                    self.objs[2].setdefault((c2, r2), []).append((0, 0, forest))
                p = patterns[rng.randrange(len(patterns))] if forest is None else patterns[0]
                for (sc, sr), name in p.items():
                    self.template(name, c2 * 4 + sc, r2 * 4 + sr, main=False)

    # --- grids ---------------------------------------------------------------------------

    def grids(self):
        """Tile types per level, deduplicated (types are bytes)."""
        lod1_types = {(): 0}
        lod1_list = [[]]

        def t1(c, r):
            objs = tuple(self.objs[1].get((c, r), []))
            if objs not in lod1_types:
                lod1_types[objs] = len(lod1_list)
                lod1_list.append(list(objs))
            return lod1_types[objs]

        lod2_types = {((), (0,) * 16): 0}
        lod2_list = [([], [0] * 16)]

        def t2(c, r):
            subs = tuple(t1(c * 4 + (k & 3), r * 4 + (k >> 2)) for k in range(16))
            key = (tuple(self.objs[2].get((c, r), [])), subs)
            if key not in lod2_types:
                lod2_types[key] = len(lod2_list)
                lod2_list.append((list(key[0]), list(subs)))
            return lod2_types[key]

        lod3_list = []
        lod3_types = {}
        lod3 = []
        for r in range(16):
            for c in range(16):
                subs = tuple(t2(c * 4 + (k & 3), r * 4 + (k >> 2)) for k in range(16))
                key = (tuple(self.objs[3].get((c, r), [])), subs)
                if key not in lod3_types:
                    lod3_types[key] = len(lod3_list)
                    lod3_list.append((list(key[0]), list(subs)))
                lod3.append(lod3_types[key])
        lod4_types = {}
        lod4_list = []
        lod4 = []
        for r in range(-2, 6):
            for c in range(-2, 6):
                shape = self.meta["lod4"]["%d,%d" % (c, r)]
                key = ((0, 0, shape),) + tuple(self.objs[4].get((c, r), []))
                if key not in lod4_types:
                    lod4_types[key] = len(lod4_list)
                    lod4_list.append(list(key))
                lod4.append(lod4_types[key])
        print("tile types: lod1 %d, lod2 %d, lod3 %d, lod4 %d" % (len(lod1_list), len(lod2_list), len(lod3_list), len(lod4_list)))
        assert max(len(lod1_list), len(lod2_list), len(lod3_list), len(lod4_list)) <= 256, "too many tile types"
        g2 = [t for _, subs in lod3_list for t in subs]
        g1 = [t for _, subs in lod2_list for t in subs]
        tiles = [[[]], lod1_list, [o for o, _ in lod2_list], [o for o, _ in lod3_list], lod4_list]
        inner4 = [lod4[(c + 2) + (r + 2) * 8] for r in range(4) for c in range(4)]
        return {"lod4": lod4, "lod3": lod3, "lod2": g2, "lod1": g1, "inner4": inner4, "tiles": tiles}

    def territory(self):
        """16x16 grid, row 0 north: side (3 friendly, 0 enemy), terrain class (8 land, 4 water)
        and 0x10 for enemy forces massed near the front."""
        grid = [0] * 256
        size = 32_000.0
        for row in range(16):
            for col in range(16):
                land = blue = 0
                n = 8
                for i in range(n):
                    for j in range(n):
                        x, y = (col + (i + 0.5) / n) * size, (row + (j + 0.5) / n) * size
                        if self.land(x, y):
                            land += 1
                            blue += self.sides.blue_at(x, y)
                friendly = blue * 2 > land if land else self.sides.blue_at((col + 0.5) * size, (row + 0.5) * size)
                # A cell holding an enemy airfield (and no friendly one) is the enemy's, however
                # much of it is friendly ground (a base on a border river).
                cell_of = lambda t: Layout.map_pos(t)[0] // 2048 == col and 15 - Layout.map_pos(t)[1] // 2048 == row
                if friendly and any(cell_of(t) for _, t in self.red_bases) and not any(cell_of(t) for _, t in self.blue_bases):
                    friendly = False
                v = (3 if friendly else 0) | (8 if land * 3 >= n * n else 4)
                cx, cy = (col + 0.5) * size, (row + 0.5) * size
                if not friendly and land and geo.dist_to_front(cx, cy, self.front) < 40_000:
                    v |= 0x10
                grid[(15 - row) * 16 + col] = v
        return grid


# ---------------------------------------------------------------------------------------------
# World


# Flight unit flags: what each entry of a region's order of battle does.
TASKS = {"special": 0x1, "base": 0x1, "patrol": 0x81, "patrol_wide": 0xC1, "escort": 0x101, "friendly": 0x1}


def flights():
    """The flight units from the region's order of battle: the special aircraft first, the four
    friendly flights last."""
    aircraft, missiles = unit_ids("aircraft"), unit_ids("missiles")
    oob = geo.ORDER_OF_BATTLE
    assert oob[0][2] == "special" and [t for _, _, t in oob[-4:]] == ["friendly"] * 4, "order of battle"
    zero = dict(waypoint=0, x=0, y=0, alt=0, xp=0, yp=0, heading=0, pitch=0, roll=0, speed=0, fuel=0)
    return [dict(zero, plane_type=aircraft.index(a), flags=TASKS[t], reserved=[missiles.index(m), 0, 0]) for a, m, t in oob]


def build_world(lay):
    models = {m["index"]: m for m in LIBRARY["models"]}
    # (The modern SAM models go unnamed: their names would crowd towns out of the name table.)
    fixed = {i: m["name"] for i, m in models.items() if not m.get("unnamed")}
    # The names of the special target models standing in for a target.
    fixed.update({92: "Special Object 1", 93: "Special Object 2"})
    # The Crimean bridge's arches are bridges to strike, like the library's (by role: their
    # names would crowd towns out of the name table).
    arches = [kerch.ROAD_ARCH, kerch.RAIL_ARCH] if "kerch_bridge" in geo.EXTRA_MODELS else []
    names = [""] * 128
    for i, n in fixed.items():
        names[i] = n
    # (A shape's slot names it, so the unnamed models' slots stay empty.)
    free = [i for i in range(1, 128) if i not in models and i not in fixed and i not in arches]
    index = {}

    def name_id(n):
        if n not in index:
            index[n] = free.pop(0)
            names[index[n]] = n
        return index[n]

    objects = [dict(name=0, x=0, y=0, unit_type=0, flags=0, occupant_type=0, patrol_count=0, object_idx=0) for _ in range(3)]

    def obj(name, t, unit_type, flags, shape):
        x, y = Layout.map_pos(t)
        return dict(name=name_id(name) if name else 0, x=x, y=y, unit_type=unit_type, flags=flags, occupant_type=0, patrol_count=0,
                    object_idx=shape)

    for name, t, ut, fl in lay.ground_units:
        objects.append(obj(name, t, ut, fl, OTH if ut == OTH_TYPE else radar_shape(ut)))
    for t, ut, shape in lay.boat_units:
        objects.append(obj("", t, ut, 8, shape))
    guc = len(objects)
    for name, t, friendly in lay.airstrips:
        if not friendly:
            objects.append(obj(name, t, 0, 0x901, K("airstrip")))
    for name, t in lay.red_bases:
        objects.append(obj(name, t, 0, 1, BASE))
    launch = []
    for name, t in lay.launch:
        launch.append(len(objects))
        objects.append(obj(name, t, 0, 0, DEPOT))
    blue_npps = [n for n in lay.npp_sites if not n[2]]
    for n in blue_npps:
        name_id(n[0])
    # Towns by size while the name table has room (750 bytes); red ones before the world
    # object count, friendly ones after it.
    cities = sorted(lay.cities, key=lambda c: -next(t[4] for t in geo.TOWNS if t[0] == c[0]))
    size = lambda: 1 + sum(len(n) + 1 for n in names[1:])
    kept = []
    for c in cities:
        if c[0] in index or size() + len(c[0]) <= 750:
            name_id(c[0])
            kept.append(c)
    for name, t, red, shape in kept:
        if red and name not in geo.NEUTRAL:
            objects.append(obj(name, t, 0, 0, shape))
    woc = len(objects)
    # Neutral places, then the friendly ones.
    for name, t, red, shape in kept:
        if name in geo.NEUTRAL:
            objects.append(obj(name, t, 0, 0x100, shape))
    for name, t in lay.neutral_bases:
        objects.append(obj(name, t, 0, 0x101, BASE))
    for name, t, red, shape in kept:
        if not red and name not in geo.NEUTRAL:
            objects.append(obj(name, t, 0, 0x400, shape))
    for name, t, _, shape in blue_npps:
        objects.append(obj(name, t, 0, 0x400, shape))
    for name, t, friendly in lay.airstrips:
        if friendly:
            objects.append(obj(name, t, 0, 0xC01, K("airstrip")))
    for name, t in lay.blue_bases:
        objects.append(obj(name, t, 0, 0x401, BASE))
    for name, t in lay.carriers:
        objects.append(obj(name, t, 0, 0x601, K("carrier_object") or CARRIER))
    assert len(objects) <= 128, len(objects)
    print("world: %d objects (%d ground units, world object count %d), names %d bytes" % (len(objects), guc, woc, size()))
    roles = [models[i]["role"] if i in models else None for i in range(128)]
    shape_class = [models[i]["class"] if i in models else 0 for i in range(128)]
    for a in arches:
        roles[a], shape_class[a] = "Bridge", kerch.ARCH_CLASS
    world = dict(land_rubble=geo.LAND_RUBBLE, water_rubble=LIBRARY["water_rubble"], ground_unit_count=guc,
                 world_object_count=woc, objects=objects, flights=flights(), shape_class=shape_class, roles=roles,
                 names=names)
    return world, launch


# ---------------------------------------------------------------------------------------------
# Models


def merge_models(new_glb, meta):
    """The library's object models (its `keep` list) and the theater's own ones, which replace
    a library model of the same number."""
    a, abin = read(os.path.join(ROOT, "assets", "models", LIBRARY["theater"] + ".glb"))
    b, bbin = read(new_glb)
    own = {m["index"] for m in meta["models"]}
    keep = (set(LIBRARY["keep"]) | {v for v in KINDS.values() if v is not None}) - own
    a, abin = subset(a, abin, lambda i: i in keep)
    attrs = {}
    prefix = LIBRARY["theater"] + "_"
    for n in a["nodes"]:
        n["name"] = geo.ID + "_" + n["name"][len(prefix):] if n["name"].startswith(prefix) else n["name"]
    for m in a["meshes"]:
        name = m.get("name", "")
        m["name"] = geo.ID + "_" + name[len(prefix):] if name.startswith(prefix) else name
    child = {c for n in a["nodes"] for c in n.get("children", [])}
    for i, n in enumerate(a["nodes"]):
        if i not in child:
            attrs[int(n["name"].rsplit("_", 1)[1])] = n.get("extras", {}).get("attr", 0)
    info = {m["name"]: m for m in meta["models"]}
    lods = {l["name"]: l for m in meta["models"] for l in m["lods"]}
    for n in b["nodes"]:
        if n["name"] in info:
            m = info[n["name"]]
            n["extras"] = {"attr": m["attr"], "lod_dists": [l["max_dist"] for l in m["lods"]]}
            attrs[m["index"]] = m["attr"]
        elif n["name"] in lods:
            l = lods[n["name"]]
            n["extras"] = {"max_dist": l["max_dist"], "planes": l.get("planes", [[0, 16384, 0, 0]]), "radius": l["radius"]}
        else:
            n.pop("extras", None)
    out, blob = append(a, abin, b, bbin)
    write(os.path.join(ROOT, "assets", "models", geo.ID + ".glb"), out, blob)
    count = max(attrs) + 1
    return count, [attrs.get(i, 0) for i in range(count)]


# ---------------------------------------------------------------------------------------------
# Output


def ron_list(v):
    return "[" + ", ".join(str(x) for x in v) + "]"


def ron_str(s):
    return '"' + s.replace("\\", "\\\\").replace('"', '\\"') + '"'


def write_ron(lay, grids, world, model_count, model_attr):
    tiles = grids["tiles"]
    tl = lambda objs: "[" + ", ".join("(x: %d, y: %d, z: 0, shape: %d)" % o for o in objs) + "]"
    level = lambda lv: "[" + ", ".join(tl(t) for t in lv) + "]"
    objs = ",\n            ".join(
        "(name: %d, x: %d, y: %d, unit_type: %d, target_flags: %d, occupant_type: %d, patrol_count: %d, object_idx: %d)"
        % (o["name"], o["x"], o["y"], o["unit_type"], o["flags"], o["occupant_type"], o["patrol_count"], o["object_idx"])
        for o in world["objects"])
    front = ", ".join("(%.1f, %.1f, %.2f)" % p for p in geo.front_m())
    roads = ""
    if lay.road_net:
        net = lay.road_net
        nodes = ", ".join("(name: %s, x: %.1f, y: %.1f)" % (ron_str(n), x, y) for n, x, y in net["nodes"])
        edges = ",\n            ".join(
            "(a: %d, b: %d, name: %s, bridge: %s,%s pts: [%s])" % (
                a, b, ron_str(name), "true" if br else "false",
                " arch: Some((%.1f, %.1f, %d))," % (*br["arch"], br["shape"]) if br else "",
                ", ".join("(%.1f, %.1f, %.1f)" % p for p in pts))
            for a, b, name, br, pts in net["edges"])
        roads = "\n    roads: (\n        nodes: [%s],\n        edges: [\n            %s\n        ],\n        entries: %s,\n    )," % (
            nodes, edges, ron_list(net["entries"]))
    towns = ",\n        ".join(
        "(name: %s, x: %.1f, y: %.1f, r: %.1f, kind: %d, ruined: %.2f)" % (ron_str(n), x, y, r, k, ru)
        for n, x, y, r, k, ru in lay.town_sites)
    text = f"""(
    id: "{geo.ID}",
    index: {THEATER_IDS.index(geo.ID)},
    grid: (
        lod4: {ron_list(grids['lod4'])},
        lod3: {ron_list(grids['lod3'])},
        lod2: {ron_list(grids['lod2'])},
        lod1: {ron_list(grids['lod1'])},
    ),
    tiles: (
        tiles: [{", ".join(level(lv) for lv in tiles)}],
    ),
    world: (
        ground_unit_count: {world['ground_unit_count']},
        world_object_count: {world['world_object_count']},
        objects: [
            {objs}
        ],
        shape_class: {ron_list(world['shape_class'])},
        names: [{", ".join(ron_str(n) for n in world['names'])}],
    ),
    model_count: {model_count},
    model_attr: {ron_list(model_attr)},
    front: [{front}],
    towns: [
        {towns}
    ],{roads}
)
"""
    open(os.path.join(ROOT, "assets", "theaters", geo.ID + ".ron"), "w").write(text)


def training(world):
    """The training range's targets and bases as world object indices, from the region's
    `TRAINING` = {"strike": [target, target, base, base], "air": [...]} of place names: a
    target is the first object of that name (ground units come first; "<name>/base" its
    airfield; "boat <k>" the k-th patrol boat), a base the airfield."""
    spec = getattr(geo, "TRAINING", None)
    if not spec:
        return None
    names, objects = world["names"], world["objects"]

    def find(name, base):
        if name.startswith("boat "):
            return [i for i, o in enumerate(objects) if o["flags"] & 8][int(name[5:])]
        for i, o in enumerate(objects):
            if names[o["name"]] == name and (not base or o["flags"] & 1):
                return i
        raise SystemExit("training range: no %s %r" % ("airfield" if base else "object", name))

    return {k: [find(n.removesuffix("/base"), j >= 2 or n.endswith("/base")) for j, n in enumerate(v)]
            for k, v in spec.items()}


def write_json(grids, world, launch, road_net=None):
    td = {
        "launch_sites": launch,
        "date": list(geo.DATE),
        "grid": list(geo.GRID),
        "squadrons": geo.SQUADRONS,
        **{k: v for k, v in (("training", training(world)), ("closed_bases", [world["names"].index(n) for n in getattr(geo, "CLOSED_BASES", [])]),
                             ("patrol_fighter", getattr(geo, "PATROL_FIGHTER", None))) if v},
        **({"roads": {
            "nodes": [{"name": n, "x": round(x, 1), "y": round(y, 1)} for n, x, y in road_net["nodes"]],
            "edges": [{"a": a, "b": b, "name": name, "bridge": bool(br), "pts": [[round(v, 1) for v in p] for p in pts],
                       **({"arch": [round(br["arch"][0], 1), round(br["arch"][1], 1), br["shape"]]} if br else {})}
                      for a, b, name, br, pts in road_net["edges"]],
            "entries": road_net["entries"]}} if road_net else {}),
        "world": world,
        "g": [[0] * 512, grids["lod1"], grids["lod2"], grids["lod3"], grids["inner4"]],
        "tiles": [[[list(o) for o in t] for t in lv] for lv in grids["tiles"]],
        "grid_size": [1024, 256, 64, 16, 4],
    }
    jsonfmt.write(os.path.join(ROOT, "assets", "mission", "theaters", geo.ID + ".json"), td)


def main():
    new_glb = sys.argv[2]
    meta = json.load(open(new_glb + ".json"))
    lay = Layout(meta)
    lay.build()
    grids = lay.grids()
    world, launch = build_world(lay)
    world["grid"] = lay.territory()
    model_count, model_attr = merge_models(new_glb, meta)
    write_ron(lay, grids, world, model_count, model_attr)
    write_json(grids, world, launch, lay.road_net)
    if hasattr(geo, "palette"):
        geo.palette(ROOT)
    # The simulation reads a rebuilt classic theater's tiles from its theater file.
    stale = os.path.join(ROOT, "assets", "sim", "theaters", geo.ID + ".json")
    if os.path.exists(stale):
        os.remove(stale)
    print("%s: %d models" % (geo.ID, model_count))


main()
