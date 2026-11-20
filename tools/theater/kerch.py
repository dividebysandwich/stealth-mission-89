"""Layout of the Crimean (Kerch) bridge for the Ukraine theater, shared by models.py (geometry)
and assemble.py (placement).

The bridge crosses the squeezed Kerch Strait in a straight line from the Kerch peninsula to
the Taman shore: the road bridge on the north side, the railway bridge on the south, side by
side on their own piers. Near the Crimean shore both pass over the Kerch-Yenikale shipping
channel on two parallel steel tied arches (227 m span, 35 m clearance, 45 m rise above the
deck). The eastern half runs over Tuzla island and then beside the dam on the Tuzla spit.

Real sizes for everything across (decks, piers, arches, heights); only the length along the
strait follows the theater's squeeze. Distances along the bridge are `s` metres from its
western end, offsets across it `n` metres to the left (north).

Each piece is its own model on the LOD2 grid (1.95 m units, streamed out to ~16 km): the
viaduct in segments of about a kilometre, so their detail levels switch near the camera, and
the two arches as separate targets (role Bridge) with a wrecked state.
"""

import math

# Ends of the bridge (lon, lat): just ashore on the theater's coast at Kerch and on Taman.
WEST = (36.4605, 45.3125)
EAST = (36.7160, 45.2690)

# The arches' middle (metres along), half span, deck top over them.
ARCH_S = 1000.0
ARCH_HALF = 113.5
ARCH_DECK = 38.0
ARCH_RISE = 45.0
# Pier spacing of the viaduct.
SPAN = 55.0

# Cross-section: (centre n, deck width, girder depth) of the road and the railway bridge.
ROAD = (20.0, 22.0, 3.2)
RAIL = (-20.0, 12.0, 3.8)

# Tuzla island and the spit's dam (metres along, as a share of the length: the island lies a
# third of the way over, the dam reaches the Taman shore).
ISLAND = (0.31, 0.53)
DAM_END = 1.0

# Model numbers: the arches, then the viaduct segments (LOD1/LOD2 models stay below 128).
ROAD_ARCH, RAIL_ARCH = 114, 115
SEGMENT_BASE = 116
SEGMENTS = 10
MODELS = [ROAD_ARCH, RAIL_ARCH] + list(range(SEGMENT_BASE, SEGMENT_BASE + SEGMENTS))
LEVEL = 2
# The arches as targets (the library bridge's class and attributes: size 4, own wreck, map flag).
ARCH_CLASS = 132
ARCH_ATTR = 0xA4


class Bridge:
    def __init__(self, geo):
        self.geo = geo
        self.w = geo.ll(*WEST)
        e = geo.ll(*EAST)
        dx, dy = e[0] - self.w[0], e[1] - self.w[1]
        self.length = math.hypot(dx, dy)
        self.dir = (dx / self.length, dy / self.length)
        # Left of the direction of travel (north side going east).
        self.left = (-self.dir[1], self.dir[0])
        self.angle = math.atan2(dy, dx)
        self.island = (ISLAND[0] * self.length, ISLAND[1] * self.length)
        water = geo.Water()
        # Where the water starts and ends along the centre line.
        steps = [k * 5.0 for k in range(int(self.length / 5.0) + 1)]
        wet = [s for s in steps if water.at(*self.point(s)) is not None]
        self.shore = (wet[0], wet[-1])

    def point(self, s, n=0.0):
        return (self.w[0] + self.dir[0] * s + self.left[0] * n, self.w[1] + self.dir[1] * s + self.left[1] * n)

    def deck(self, s):
        """Deck top above the sea at `s` (metres)."""
        a0, a1 = ARCH_S - ARCH_HALF, ARCH_S + ARCH_HALF
        if a0 <= s <= a1:
            return ARCH_DECK
        ease = lambda t: 0.5 - 0.5 * math.cos(math.pi * min(1.0, max(0.0, t)))
        if s < a0:
            return 7.0 + (ARCH_DECK - 7.0) * ease(s / a0)
        east_ramp = 1400.0
        if s < a1 + east_ramp:
            return 14.0 + (ARCH_DECK - 14.0) * (1.0 - ease((s - a1) / east_ramp))
        i0 = self.island[0]
        if s < i0 - 500.0:
            return 14.0
        if s < i0:
            return 14.0 - 4.0 * ease((s - i0 + 500.0) / 500.0)
        end = self.length
        if s > end - 300.0:
            return 10.0 - 3.0 * ease((s - end + 300.0) / 300.0)
        return 10.0

    def piers(self):
        """Pier positions along the bridge: the arches' two main piers, the viaduct's every
        `SPAN` metres outwards from them, the abutments at both ends."""
        a0, a1 = ARCH_S - ARCH_HALF, ARCH_S + ARCH_HALF
        out = [0.0]
        s = a0
        while s > SPAN * 0.6:
            out.append(s)
            s -= SPAN
        s = a1
        while s < self.length - SPAN * 0.6:
            out.append(s)
            s += SPAN
        out.append(self.length)
        return sorted(out)

    def segments(self):
        """The viaduct's pieces: [(model number, s0, s1)], the arch span left out, each
        ending at a pier."""
        piers = self.piers()
        a0, a1 = ARCH_S - ARCH_HALF, ARCH_S + ARCH_HALF
        west = [p for p in piers if p <= a0 + 1e-6]
        east = [p for p in piers if p >= a1 - 1e-6]
        out = [(west[0], west[-1])]
        n = SEGMENTS - 1
        per = (len(east) - 1) / n
        cuts = [east[round(k * per)] for k in range(n)] + [east[-1]]
        out += list(zip(cuts, cuts[1:]))
        return [(SEGMENT_BASE + k, s0, s1) for k, (s0, s1) in enumerate(out)]

    def pieces(self):
        """Every model with its range along the bridge and its snapped origin (theater metres):
        [(model number, s0, s1, (x, y))]. Arches first."""
        out = [(ROAD_ARCH, ARCH_S - ARCH_HALF, ARCH_S + ARCH_HALF, self.snap(self.point(ARCH_S, ROAD[0]))),
               (RAIL_ARCH, ARCH_S - ARCH_HALF, ARCH_S + ARCH_HALF, self.snap(self.point(ARCH_S, RAIL[0])))]
        for shape, s0, s1 in self.segments():
            out.append((shape, s0, s1, self.snap(self.point((s0 + s1) / 2))))
        return out

    def snap(self, p):
        """The point on the LOD2 grid where assemble.py's `put` places an object."""
        u = 4 * self.geo.WU  # metres per LOD2 unit
        out = []
        for v in p:
            cell = math.floor(v / (4096 * u))
            centre = (cell * 4096 + 2048) * u
            out.append(centre + round((v - centre) / u) * u)
        return tuple(out)

    def local(self, origin):
        """Function (s, n, z) metres -> model-space metres around `origin`."""
        ox, oy = origin[0] - self.w[0], origin[1] - self.w[1]
        d, l = self.dir, self.left

        def f(s, n, z):
            return (d[0] * s + l[0] * n - ox, d[1] * s + l[1] * n - oy, z)

        return f
