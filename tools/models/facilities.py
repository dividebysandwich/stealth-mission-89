"""The campaign's strategic facilities (crates/stealth-campaign/src/facilities.rs), built into
aaa.glb by tools/models/air_defence.py after the ground units:

- 137 the ammo storage beside an airfield: three earth-covered igloo bunkers with concrete
  headwalls and steel doors, earth traverses between them, a guard house and an apron.
- 138 a fuel depot: four storage tanks in earth bunds, a pump house, a pipe rack and a truck
  loading gantry.
- 139 a power plant: a turbine hall and a boiler house, two stacks, a cooling tower, and a
  switchyard of transformers under gantries with a line of pylons leaving it.
- 140 the drone factory: a saw-tooth roofed assembly hall, a vaulted hangar, a test launch rail
  and crates on the apron.
- 141 the tank factory: a long saw-tooth hall with a rail spur into it, a test track and new
  tanks parked in rows.
- 142 the headquarters: a four-storey office block with a wing, an antenna mast with dishes,
  a bunker entrance and a vehicle park.

Each is a unit model as the others in aaa.glb: a root `aaa_NNN` with `attr` and `lod_dists`, the
detailed level holding the intact buildings in `_s1v0` and the ruins in `_s1v1` (state slot 1),
the far level a few boxes. Its ground (a concrete or gravel pad, drawn always) is `_cold`, so a
FLIR shows the buildings hot and the yard as the ground about it. Model space: x east, y north,
z up, built in metres and scaled to model units.
"""

import math
import random

from mklib import Geo, Ramp, block, frustum, tube, v3

CONCRETE = Ramp(0xE3, 0xEA)
WALL = Ramp(0xE3, 0xEA)
ROOF = Ramp(0xE5, 0xED)
DARK_ROOF = Ramp(0xE8, 0xEE)
STEEL = Ramp(0xEB, 0xEF)
PAINT_WHITE = Ramp(0xE1, 0xE7)
EARTH = Ramp(0xA4, 0xA9)
GRASS = Ramp(0xA6, 0xAD)
OLIVE = Ramp(0xA6, 0xAF)
SAND = Ramp(0xC1, 0xC5)
BURNT = Ramp(0xEC, 0xEF)
RUBBLE = Ramp(0xE6, 0xEE)
GLASS = 0x9A
DOOR = Ramp(0xEA, 0xEE)
PAD = 0xE6
ASPHALT = 0xEB
SCORCH = 0xEF
FIRE = 0x0D

AMMO_DEPOT = 137
FUEL_DEPOT = 138
POWER_PLANT = 139
DRONE_FACTORY = 140
TANK_FACTORY = 141
HEADQUARTERS = 142


# ------------------------------------------------------------------------------------ shapes


def flat(x0, x1, y0, y1, z, c):
    """A flat patch on the ground, facing up."""
    g = Geo()
    g.face([(x0, y0, z), (x1, y0, z), (x1, y1, z), (x0, y1, z)], c)
    return g


def disc(cx, cy, r, z, c, n=12):
    g = Geo()
    g.face([(cx + r * math.cos(2 * math.pi * k / n), cy + r * math.sin(2 * math.pi * k / n), z) for k in range(n)], c)
    return g


def blob(cx, cy, r, z, c, rng, n=9):
    """An irregular patch (scorch marks)."""
    g = Geo()
    pts = []
    for k in range(n):
        a = 2 * math.pi * k / n
        rr = r * (0.65 + 0.35 * rng.random())
        pts.append((cx + rr * math.cos(a), cy + rr * math.sin(a), z))
    g.face(pts, c)
    return g


def moved(g, d):
    return g.transformed(lambda p: p + v3(d))


def flipped(g):
    """Every face turned round (an inside wall)."""
    out = Geo()
    out.faces = [(list(reversed(pts)), c) for pts, c in g.faces]
    out.lines = dict(g.lines)
    return out


def vault_y(x0, x1, y0, y1, z0, rise, paint, ends=None, n=6):
    """A half barrel along y from z0 up to z0 + rise (an igloo's earth cover, a hangar)."""
    cx, hw = (x0 + x1) / 2, (x1 - x0) / 2
    arc = [(cx - hw * math.cos(math.pi * k / n), z0 + rise * math.sin(math.pi * k / n)) for k in range(n + 1)]
    g = Geo()
    c = (cx, (y0 + y1) / 2, z0 + rise / 3)
    for (xa, za), (xb, zb) in zip(arc, arc[1:]):
        g.face([(xa, y0, za), (xb, y0, zb), (xb, y1, zb), (xa, y1, za)], paint, c)
    for y in (y0, y1):
        g.face([(x, y, z) for x, z in arc], ends or paint, c)
    return g


def berm(x0, x1, y0, y1, h, crest=1.0, paint=EARTH):
    """A wall of earth filling the box, sloping up to a crest along its longer side."""
    if x1 - x0 >= y1 - y0:
        cy = (y0 + y1) / 2
        return tube(
            [[(x, y0, 0), (x, y1, 0), (x, cy + crest / 2, h), (x, cy - crest / 2, h)] for x in (x0, x1)], paint
        )
    cx = (x0 + x1) / 2
    return tube([[(x0, y, 0), (x1, y, 0), (cx + crest / 2, y, h), (cx - crest / 2, y, h)] for y in (y0, y1)], paint)


def ring_bund(cx, cy, r, h, paint=EARTH, n=16):
    """A low circular earth wall round a tank (a bund), open nowhere."""
    g = Geo()
    for k in range(n):
        a0, a1 = 2 * math.pi * k / n, 2 * math.pi * (k + 1) / n
        p = lambda a, rr, z: (cx + rr * math.cos(a), cy + rr * math.sin(a), z)  # noqa: E731
        g.face([p(a0, r + 1.5, 0), p(a1, r + 1.5, 0), p(a1, r + 0.5, h), p(a0, r + 0.5, h)], paint)
        g.face([p(a0, r - 0.5, h), p(a1, r - 0.5, h), p(a1, r - 1.0, 0), p(a0, r - 1.0, 0)], paint)
        g.face([p(a0, r + 0.5, h), p(a1, r + 0.5, h), p(a1, r - 0.5, h), p(a0, r - 0.5, h)], paint)
    return g


def windows_x(x0, x1, y, z0, z1, n, face):
    """A row of windows on a wall facing +y (face 1) or -y (-1)."""
    g = Geo()
    step = (x1 - x0) / n
    yy = y + 0.06 * face
    for k in range(n):
        a, b = x0 + step * (k + 0.2), x0 + step * (k + 0.8)
        pts = [(a, yy, z0), (b, yy, z0), (b, yy, z1), (a, yy, z1)]
        g.face(pts if face < 0 else pts[::-1], GLASS)
    return g


def windows_y(y0, y1, x, z0, z1, n, face):
    """A row of windows on a wall facing +x (face 1) or -x (-1)."""
    g = Geo()
    step = (y1 - y0) / n
    xx = x + 0.06 * face
    for k in range(n):
        a, b = y0 + step * (k + 0.2), y0 + step * (k + 0.8)
        pts = [(xx, a, z0), (xx, b, z0), (xx, b, z1), (xx, a, z1)]
        g.face(pts if face > 0 else pts[::-1], GLASS)
    return g


def sawtooth(x0, x1, y0, y1, h, teeth, rise, wall=WALL, roof=DARK_ROOF):
    """A factory hall with a saw-tooth roof: `teeth` ridges along x, each a glazed face to the
    north and a roof slope to the south."""
    g = block(x0, x1, y0, y1, 0.0, h, wall)
    step = (y1 - y0) / teeth
    for k in range(teeth):
        a = y0 + k * step
        b = a + step
        prof = [(a, h), (b, h), (b, h + rise)]
        g.add(tube([[(x, y, z) for y, z in prof] for x in (x0, x1)], roof))
        # The north lights: glass on the tooth's steep face.
        g.face([(x0 + 0.5, b + 0.05, h + 0.3), (x1 - 0.5, b + 0.05, h + 0.3), (x1 - 0.5, b + 0.05, h + rise - 0.3), (x0 + 0.5, b + 0.05, h + rise - 0.3)][::-1], GLASS)
    return g


def rubble(x0, x1, y0, y1, hmax, rng, n, paint=RUBBLE):
    """A field of fallen blocks."""
    g = Geo()
    for _ in range(n):
        w, d = rng.uniform(1.5, 5.0), rng.uniform(1.5, 5.0)
        x, y = rng.uniform(x0, x1 - w), rng.uniform(y0, y1 - d)
        h = rng.uniform(0.4, hmax)
        piece = block(x, x + w, y, y + d, -0.2, h, paint, taper=w * 0.2, taper_y=d * 0.2)
        g.add(piece.rotated_z(rng.uniform(-0.6, 0.6), (x + w / 2, y + d / 2, 0)))
    return g


def ruin(x0, x1, y0, y1, h, rng, wall=BURNT):
    """A building fallen in: stumps of its walls and a heap of rubble inside."""
    g = Geo()
    t = 0.6
    for (a0, a1, b0, b1) in ((x0, x1, y0, y0 + t), (x0, x1, y1 - t, y1), (x0, x0 + t, y0, y1), (x1 - t, x1, y0, y1)):
        horiz = a1 - a0 > b1 - b0
        n = max(2, int((a1 - a0 if horiz else b1 - b0) / 6))
        for k in range(n):
            if rng.random() < 0.3:
                continue
            if horiz:
                s0, s1 = a0 + (a1 - a0) * k / n, a0 + (a1 - a0) * (k + 1) / n
                g.add(block(s0, s1, b0, b1, 0.0, h * rng.uniform(0.15, 0.6), wall))
            else:
                s0, s1 = b0 + (b1 - b0) * k / n, b0 + (b1 - b0) * (k + 1) / n
                g.add(block(a0, a1, s0, s1, 0.0, h * rng.uniform(0.15, 0.6), wall))
    area = (x1 - x0) * (y1 - y0)
    g.add(rubble(x0 + 1, x1 - 1, y0 + 1, y1 - 1, max(1.0, h * 0.3), rng, int(min(40, 6 + area / 60))))
    return g


def scorched(x0, x1, y0, y1, rng, n=6, fire=2):
    """Scorch marks over the yard, and a fire or two still burning."""
    g = Geo()
    for _ in range(n):
        g.add(blob(rng.uniform(x0, x1), rng.uniform(y0, y1), rng.uniform(6, 16), 0.08, SCORCH, rng))
    for _ in range(fire):
        g.add(blob(rng.uniform(x0, x1), rng.uniform(y0, y1), rng.uniform(2, 4), 0.12, FIRE, rng, n=7))
    return g


def tank_vehicle(paint=OLIVE):
    """A new main battle tank parked, nose to +y (a few boxes: hull, turret, gun)."""
    g = block(-1.8, 1.8, -3.6, 3.6, 0.4, 1.6, paint, taper_y=0.4)
    g.add(block(-1.9, -1.4, -3.4, 3.4, 0.0, 0.9, STEEL))
    g.add(block(1.4, 1.9, -3.4, 3.4, 0.0, 0.9, STEEL))
    g.add(block(-1.2, 1.2, -1.6, 1.0, 1.6, 2.3, paint, taper=0.25, taper_y=0.3))
    g.add(frustum((0, 1.0, 1.95), (0, 6.0, 1.95), 0.12, 0.1, paint, n=6))
    return g


def truck(paint=OLIVE):
    """A lorry, nose to +y."""
    g = block(-1.2, 1.2, 1.6, 3.6, 0.5, 2.7, paint, taper_y=0.2)
    g.add(block(-1.25, 1.25, -3.6, 1.5, 0.9, 3.0, paint))
    g.add(block(-1.1, 1.1, -3.4, 3.4, 0.0, 0.9, STEEL))
    return g


def pylon(x, y, h, paint=STEEL):
    """A lattice pylon (as a tapering post with cross arms)."""
    g = frustum((x, y, 0), (x, y, h), 1.2, 0.35, paint, n=4)
    for z, w in ((h * 0.75, 5.0), (h * 0.92, 3.5)):
        g.add(block(x - w, x + w, y - 0.25, y + 0.25, z, z + 0.4, paint))
    return g


def hyperboloid(cx, cy, r0, rw, r1, h, paint, n=16):
    """A cooling tower: wide at its foot, narrow at its waist (two thirds up), flaring a little at
    its rim; open at the top (its inside shows)."""
    stations = [(0.0, r0), (h * 0.35, (r0 + rw) / 2 + rw * 0.05), (h * 0.68, rw), (h, r1)]
    rings = [[(cx + r * math.cos(2 * math.pi * k / n), cy + r * math.sin(2 * math.pi * k / n), z) for k in range(n)] for z, r in stations]
    g = tube(rings, paint, cap0=False, cap1=False)
    inner = [[(cx + (r - 0.6) * math.cos(2 * math.pi * k / n), cy + (r - 0.6) * math.sin(2 * math.pi * k / n), z) for k in range(n)] for z, r in stations]
    g.add(flipped(tube(inner, Ramp(0xE8, 0xEE), cap0=False, cap1=False)))
    g.add(disc(cx, cy, r0 - 0.7, 1.5, 0x9C, n))
    return g


def stack(cx, cy, r, h, paint=CONCRETE, band=0xEE):
    g = frustum((cx, cy, 0), (cx, cy, h), r * 1.15, r, paint, n=10)
    g.add(frustum((cx, cy, h - 4.0), (cx, cy, h), r + 0.05, r + 0.05, band, n=10, caps=False))
    return g


def fallen_stack(cx, cy, r, length, heading, paint=BURNT):
    """A stack fallen over, lying in pieces along `heading` (rad from north)."""
    g = Geo()
    dx, dy = math.sin(heading), math.cos(heading)
    s = 0.0
    k = 0
    while s < length:
        piece = min(12.0, length - s)
        a = (cx + dx * s, cy + dy * s, r)
        b = (cx + dx * (s + piece), cy + dy * (s + piece), r)
        g.add(frustum(a, b, r, r * 0.97, paint, n=8))
        s += piece + 1.5 + (k % 2)
        k += 1
    return g


# ------------------------------------------------------------------------------------ models


def ammo_depot(rng):
    """Three igloo bunkers side by side, doors to the south, on an apron."""
    pad = flat(-45, 45, -40, 30, 0.05, PAD)
    pad.add(flat(-40, 40, -40, -28, 0.08, ASPHALT))
    pad.add(flat(-4, 4, -60, -40, 0.08, ASPHALT))
    intact, wreck = Geo(), Geo()
    for k, x in enumerate((-26, 0, 26)):
        intact.add(vault_y(x - 8, x + 8, -22, 18, 0.0, 6.5, GRASS, ends=EARTH))
        # The concrete headwall with its wing walls, and the steel door.
        intact.add(block(x - 9.5, x + 9.5, -23.2, -22.0, 0.0, 7.2, CONCRETE))
        intact.add(block(x - 9.5, x - 8.5, -27, -22, 0.0, 4.0, CONCRETE, taper_y=0.0))
        intact.add(block(x + 8.5, x + 9.5, -27, -22, 0.0, 4.0, CONCRETE))
        door = Geo()
        door.face([(x - 2.5, -23.3, 0.0), (x + 2.5, -23.3, 0.0), (x + 2.5, -23.3, 4.2), (x - 2.5, -23.3, 4.2)], DOOR)
        intact.add(door)
        # Ventilators on the mound.
        intact.add(frustum((x, 8, 5.0), (x, 8, 7.6), 0.35, 0.35, STEEL, n=6))
        # Destroyed: the cover blown off, the headwall down, a crater where the magazine was.
        wreck.add(disc(x, -2, 11, 0.06, SCORCH, 10))
        wreck.add(vault_y(x - 8, x + 8, -10, 18, 0.0, 2.2, BURNT, ends=EARTH))
        wreck.add(rubble(x - 10, x + 10, -28, -14, 2.0, rng, 12))
        if k != 1:
            wreck.add(block(x - 9.5, x - 4, -23.2, -22.0, 0.0, 4.5, BURNT).rotated_x(-0.4, (x, -23, 0)))
    for x in (-13, 13):
        intact.add(berm(x - 3, x + 3, -20, 16, 5.0))
        wreck.add(berm(x - 3, x + 3, -20, 16, 3.5, paint=EARTH))
    # The guard house by the gate, and a forklift shed.
    intact.add(block(6, 11, -40, -35, 0.0, 3.0, WALL))
    intact.add(block(5.7, 11.3, -40.3, -34.7, 3.0, 3.3, ROOF))
    intact.add(block(-38, -30, -38, -30, 0.0, 4.0, WALL))
    intact.add(block(-38.3, -29.7, -38.3, -29.7, 4.0, 4.3, ROOF))
    wreck.add(block(6, 11, -40, -35, 0.0, 3.0, WALL))
    wreck.add(block(5.7, 11.3, -40.3, -34.7, 3.0, 3.3, ROOF))
    wreck.add(ruin(-38, -30, -38, -30, 4.0, rng))
    wreck.add(scorched(-40, 40, -35, 20, rng, 8, 3))
    boxes = [(-34, 34, -23, 18, 0.0, 6.5), (6, 11, -40, -35, 0.0, 3.3)]
    return pad, intact, wreck, boxes, 60.0


def fuel_depot(rng):
    """Four storage tanks in their bunds, the pump house and the loading gantry."""
    pad = flat(-42, 42, -42, 42, 0.05, 0xE7)
    pad.add(flat(-42, 42, -42, -32, 0.08, ASPHALT))
    intact, wreck = Geo(), Geo()
    tanks = [(-18, 14, 9.5, 11.0), (18, 14, 9.5, 11.0), (-18, -12, 8.0, 9.0), (18, -12, 8.0, 9.0)]
    for cx, cy, r, h in tanks:
        intact.add(frustum((cx, cy, 0), (cx, cy, h), r, r, PAINT_WHITE, n=16, caps=False))
        # A shallow cone roof.
        intact.add(frustum((cx, cy, h), (cx, cy, h + r * 0.18), r, 0.4, PAINT_WHITE, n=16))
        intact.add(ring_bund(cx, cy, r + 4.0, 1.6))
        # A spiral stair as a slanted strip up its side.
        intact.add(block(cx + r, cx + r + 0.8, cy - 1, cy + 1, 0.0, h, STEEL))
        # Destroyed: burst, burnt out and slumped, the roof fallen in; the bund still there.
        wreck.add(frustum((cx, cy, 0), (cx, cy, h * rng.uniform(0.35, 0.55)), r, r * 1.05, BURNT, n=16, caps=False))
        wreck.add(disc(cx, cy, r - 0.3, 0.9, SCORCH, 16))
        wreck.add(ring_bund(cx, cy, r + 4.0, 1.6, BURNT))
        wreck.add(blob(cx, cy, r * 0.5, 1.0, FIRE, rng, n=8))
    # The pipe rack along the middle and the pump house.
    for y in (-1.0, 1.0):
        intact.add(block(-30, 30, y - 0.25, y + 0.25, 2.4, 2.9, STEEL))
    for x in range(-28, 31, 8):
        intact.add(block(x - 0.2, x + 0.2, -1.5, 1.5, 0.0, 2.4, STEEL))
    intact.add(block(-6, 6, -36, -28, 0.0, 4.5, WALL))
    intact.add(block(-6.3, 6.3, -36.3, -27.7, 4.5, 4.8, ROOF))
    # The loading gantry over the road, a tanker under it.
    intact.add(block(22, 36, -40, -34, 6.0, 6.6, STEEL))
    for x in (22.5, 35.5):
        intact.add(block(x - 0.3, x + 0.3, -40, -34, 0.0, 6.0, STEEL))
    intact.add(moved(truck(PAINT_WHITE).rotated_z(math.pi / 2), (29, -37, 0)))
    wreck.add(rubble(-30, 30, -2, 2, 1.0, rng, 10, BURNT))
    wreck.add(ruin(-6, 6, -36, -28, 4.5, rng))
    wreck.add(block(22, 36, -40, -34, 0.0, 1.5, BURNT).rotated_x(0.1, (29, -37, 0)))
    wreck.add(scorched(-40, 40, -40, 40, rng, 10, 2))
    boxes = [(-28, -8, 4, 24, 0.0, 12.5), (8, 28, 4, 24, 0.0, 12.5), (-26, -10, -20, -4, 0.0, 10.5), (10, 26, -20, -4, 0.0, 10.5)]
    return pad, intact, wreck, boxes, 70.0


def power_plant(rng):
    """The turbine hall and boiler house, two stacks, the cooling tower and the switchyard."""
    pad = flat(-70, 70, -55, 55, 0.05, PAD)
    pad.add(flat(20, 66, -50, -10, 0.08, 0xE8))
    intact, wreck = Geo(), Geo()
    # Turbine hall (long, gabled) and the tall boiler house at its end.
    intact.add(block(-50, 10, 10, 40, 0.0, 20.0, WALL))
    intact.add(tube([[(x, 10, 20), (x, 40, 20), (x, 25, 25)] for x in (-50, 10)], ROOF))
    intact.add(windows_x(-48, 8, 10, 12, 18, 10, -1))
    intact.add(block(10, 34, 8, 42, 0.0, 38.0, CONCRETE))
    intact.add(windows_y(10, 40, 34, 30, 36, 6, 1))
    # Coal bunker gallery from the yard up to the boiler house.
    intact.add(block(34, 60, 22, 26, 22.0, 26.0, STEEL).rotated_x(0.0))
    intact.add(block(56, 64, 18, 30, 0.0, 26.0, WALL))
    # The stacks behind the boiler house.
    for cx in (16, 28):
        intact.add(stack(cx, 50, 3.2, 95.0))
    # The cooling tower.
    intact.add(hyperboloid(-42, -28, 24.0, 15.0, 16.5, 58.0, Ramp(0xE2, 0xE9)))
    # The switchyard: transformers and gantries, pylons going off east.
    for k, x in enumerate((26, 38, 50, 62)):
        intact.add(block(x - 3, x + 3, -26, -20, 0.0, 4.0, OLIVE))
        intact.add(block(x - 1.2, x + 1.2, -25, -21, 4.0, 6.5, STEEL))
        for y in (-44, -14):
            intact.add(block(x - 0.3, x + 0.3, y - 0.3, y + 0.3, 0.0, 14.0, STEEL))
        intact.add(block(x - 0.25, x + 0.25, -44, -14, 13.6, 14.0, STEEL))
    intact.add(pylon(68, -30, 32.0))
    # Destroyed: the hall's roof in, the boiler house gutted, one stack down across the yard,
    # the transformers burnt; the cooling tower stands, blackened, its fill burning.
    wreck.add(ruin(-50, 10, 10, 40, 20.0, rng))
    wreck.add(block(10, 34, 8, 42, 0.0, 14.0, BURNT))
    wreck.add(ruin(12, 32, 10, 40, 38.0, rng))
    wreck.add(block(56, 64, 18, 30, 0.0, 12.0, BURNT))
    wreck.add(stack(16, 50, 3.2, 95.0, paint=Ramp(0xE8, 0xEE)))
    wreck.add(block(25, 31, 47, 53, 0.0, 9.0, BURNT))
    wreck.add(fallen_stack(28, 52, 3.2, 80.0, math.radians(-95)))
    wreck.add(hyperboloid(-42, -28, 24.0, 15.0, 16.5, 58.0, Ramp(0xE9, 0xEF)))
    wreck.add(disc(-42, -28, 22, 1.7, FIRE, 16))
    for x in (26, 38, 50, 62):
        wreck.add(block(x - 3, x + 3, -26, -20, 0.0, 2.5, BURNT).rotated_z(rng.uniform(-0.4, 0.4), (x, -23, 0)))
    wreck.add(pylon(68, -30, 32.0, BURNT).rotated_x(1.2, (68, -30, 0)))
    wreck.add(scorched(-60, 66, -50, 50, rng, 12, 3))
    boxes = [(-50, 10, 10, 40, 0.0, 22.0), (10, 34, 8, 42, 0.0, 38.0), (13, 31, 47, 53, 0.0, 95.0), (-60, -24, -46, -10, 0.0, 58.0)]
    return pad, intact, wreck, boxes, 150.0


def drone_factory(rng):
    """The assembly hall, a vaulted hangar, the test rail and the apron with crates."""
    pad = flat(-55, 55, -42, 40, 0.05, PAD)
    pad.add(flat(-50, 0, -40, -18, 0.08, ASPHALT))
    intact, wreck = Geo(), Geo()
    intact.add(sawtooth(-40, 25, -10, 32, 9.0, 6, 3.5))
    intact.add(windows_x(-38, 23, -10, 3, 6, 9, -1))
    # The offices at its end.
    intact.add(block(25, 40, 6, 26, 0.0, 10.5, WALL))
    intact.add(windows_y(8, 24, 40, 3, 5, 5, 1))
    intact.add(windows_y(8, 24, 40, 7, 9, 5, 1))
    # The hangar: a barrel vault, its doors to the south.
    intact.add(vault_y(-48, -28, -38, -14, 0.0, 9.0, ROOF, ends=WALL))
    intact.add(block(-44, -32, -38.3, -38.0, 0.0, 6.0, DOOR))
    # The launch rail on its trestle, pointing east, and a catapult van.
    rail = block(-0.3, 0.3, 0.0, 14.0, 0.0, 0.4, STEEL).rotated_x(0.2)
    intact.add(moved(rail.rotated_z(-math.pi / 2), (10, -30, 1.0)))
    for x in (11, 21):
        intact.add(block(x - 0.2, x + 0.2, -30.5, -29.5, 0.0, 1.4 + (x - 11) * 0.2, STEEL))
    intact.add(moved(truck().rotated_z(math.pi / 2), (5, -36, 0)))
    # Crates of finished drones on the apron.
    for i in range(3):
        for j in range(5):
            x, y = -24 + j * 4.5, -34 + i * 4.0
            intact.add(block(x, x + 3.4, y, y + 2.6, 0.0, 1.4 * (1 + (i + j) % 2), SAND))
    wreck.add(ruin(-40, 25, -10, 32, 9.0, rng))
    wreck.add(ruin(25, 40, 6, 26, 10.5, rng))
    wreck.add(vault_y(-48, -28, -38, -14, 0.0, 3.5, BURNT, ends=BURNT))
    wreck.add(rubble(-48, -28, -38, -14, 2.0, rng, 14, BURNT))
    wreck.add(moved(rail.rotated_z(-math.pi / 2 + 0.4).recolour(BURNT), (10, -30, 0.2)))
    wreck.add(rubble(-26, -2, -36, -20, 1.2, rng, 12, BURNT))
    wreck.add(scorched(-50, 40, -40, 35, rng, 10, 3))
    boxes = [(-40, 25, -10, 32, 0.0, 12.5), (25, 40, 6, 26, 0.0, 10.5), (-48, -28, -38, -14, 0.0, 9.0)]
    return pad, intact, wreck, boxes, 90.0


def tank_factory(rng):
    """The long assembly hall, the rail spur into it, the test track and the tanks in rows."""
    pad = flat(-80, 80, -55, 50, 0.05, PAD)
    intact, wreck = Geo(), Geo()
    intact.add(sawtooth(-60, 50, 0, 44, 13.0, 7, 4.5))
    intact.add(windows_x(-58, 48, 0, 5, 9, 14, -1))
    intact.add(block(50, 66, 10, 34, 0.0, 16.0, CONCRETE))
    intact.add(windows_y(12, 32, 66, 9, 13, 5, 1))
    # The rail spur from the west into the hall.
    spur = Geo()
    for y in (19.3, 20.7):
        spur.add(block(-80, -60, y - 0.1, y + 0.1, 0.08, 0.25, STEEL))
    for x in range(-79, -60, 2):
        spur.add(block(x - 0.25, x + 0.25, 18.5, 21.5, 0.05, 0.15, 0xE9))
    pad.add(spur)
    intact.add(block(-78, -64, 18.6, 21.4, 0.3, 3.8, OLIVE))
    # The test track: a loop of beaten earth south of the hall.
    track = Geo()
    for k in range(20):
        a0, a1 = 2 * math.pi * k / 20, 2 * math.pi * (k + 1) / 20
        p = lambda a, r: (10 + r * 1.9 * math.cos(a), -32 + r * math.sin(a), 0.1)  # noqa: E731
        track.face([p(a0, 13), p(a1, 13), p(a1, 18), p(a0, 18)][::-1], SAND)
    pad.add(track)
    # New tanks in rows before the hall.
    for i in range(2):
        for j in range(6):
            intact.add(moved(tank_vehicle(), (-56 + j * 7.0, -14 - i * 11.0, 0)))
    wreck.add(ruin(-60, 50, 0, 44, 13.0, rng))
    wreck.add(ruin(50, 66, 10, 34, 16.0, rng))
    wreck.add(block(-78, -64, 18.6, 21.4, 0.0, 2.0, BURNT).rotated_z(0.15, (-71, 20, 0)))
    for i in range(2):
        for j in range(6):
            if rng.random() < 0.6:
                wreck.add(moved(tank_vehicle(BURNT).rotated_z(rng.uniform(-0.5, 0.5)), (-56 + j * 7.0, -14 - i * 11.0, 0)))
    wreck.add(scorched(-70, 60, -45, 40, rng, 12, 3))
    boxes = [(-60, 50, 0, 44, 0.0, 17.5), (50, 66, 10, 34, 0.0, 16.0), (-58, -18, -30, -10, 0.0, 2.5)]
    return pad, intact, wreck, boxes, 110.0


def headquarters(rng):
    """The office block with its wing, the antenna mast, the bunker entrance, the vehicle park."""
    pad = flat(-42, 42, -38, 38, 0.05, PAD)
    pad.add(flat(-40, -12, -36, -14, 0.08, ASPHALT))
    intact, wreck = Geo(), Geo()
    floors = 4
    h = floors * 3.6
    intact.add(block(-25, 20, 6, 22, 0.0, h, WALL))
    intact.add(block(-25.3, 20.3, 5.7, 22.3, h, h + 0.6, ROOF))
    intact.add(block(8, 22, -10, 6, 0.0, h - 3.6, WALL))
    intact.add(block(7.7, 22.3, -10.3, 6, h - 3.6, h - 3.0, ROOF))
    for f in range(floors):
        z = f * 3.6 + 1.0
        intact.add(windows_x(-24, 19, 6, z, z + 1.6, 12, -1))
        intact.add(windows_x(-24, 19, 22, z, z + 1.6, 12, 1))
        if f < floors - 1:
            intact.add(windows_y(-9, 5, 22, z, z + 1.6, 4, 1))
    # Roof plant and the antenna mast with its dishes and whips.
    intact.add(block(-8, 0, 10, 16, h + 0.6, h + 3.0, STEEL))
    mast = frustum((-30, 28, 0), (-30, 28, 42.0), 1.1, 0.3, STEEL, n=4)
    for z, d in ((20, 1.6), (28, 1.3), (35, 1.0)):
        mast.add(frustum((-30, 28 - 0.8, z), (-30, 28 - 1.6, z), 0.2, d, PAINT_WHITE, n=10))
    for dx in (-0.8, 0.8):
        mast.add(frustum((-30 + dx, 28, 42.0), (-30 + dx, 28, 48.0), 0.06, 0.03, STEEL, n=4))
    intact.add(mast)
    # The bunker: an earth mound with a concrete entrance down a ramp.
    intact.add(vault_y(26, 40, 10, 32, 0.0, 4.0, GRASS, ends=EARTH))
    intact.add(block(30, 36, 6, 10, 0.0, 3.4, CONCRETE))
    # Vehicles in the park, and the guard post at the gate.
    for k in range(5):
        intact.add(moved(truck(), (-36 + k * 5.5, -25, 0)))
    intact.add(block(-4, 0, -38, -34, 0.0, 2.8, WALL))
    wreck.add(ruin(-25, 20, 6, 22, h, rng))
    wreck.add(block(-25, -6, 6, 22, 0.0, h * 0.55, BURNT))
    wreck.add(ruin(8, 22, -10, 6, h - 3.6, rng))
    wreck.add(frustum((-30, 28, 0.5), (-30 + 38 * math.sin(0.8), 28 - 38 * math.cos(0.8), 0.6), 1.1, 0.3, BURNT, n=4))
    wreck.add(vault_y(26, 40, 10, 32, 0.0, 4.0, BURNT, ends=EARTH))
    wreck.add(block(30, 36, 6, 10, 0.0, 3.4, BURNT))
    for k in range(5):
        if rng.random() < 0.7:
            wreck.add(moved(truck(BURNT).rotated_z(rng.uniform(-0.6, 0.6)), (-36 + k * 5.5, -25, 0)))
    wreck.add(block(-4, 0, -38, -34, 0.0, 2.8, WALL))
    wreck.add(scorched(-40, 40, -30, 35, rng, 9, 3))
    boxes = [(-25, 20, 6, 22, 0.0, h + 0.6), (8, 22, -10, 6, 0.0, h - 3.0), (-31, -29, 27, 29, 0.0, 42.0)]
    return pad, intact, wreck, boxes, 60.0


MODELS = [
    (AMMO_DEPOT, ammo_depot),
    (FUEL_DEPOT, fuel_depot),
    (POWER_PLANT, power_plant),
    (DRONE_FACTORY, drone_factory),
    (TANK_FACTORY, tank_factory),
    (HEADQUARTERS, headquarters),
]


def build_all(levels, mesh_obj, far, K):
    """Every facility into the scene (air_defence.py's helpers): a root, its detailed level
    (the pad always, the buildings intact or ruined) and the far level of boxes."""
    for index, make in MODELS:
        rng = random.Random(index)
        pad, intact, wreck, boxes, radius = make(rng)
        # Seen from far off: the detailed level out to about 12 km (the game's distances are
        # four times these).
        root, lod0, lod1 = levels(index, 48000)
        lod0["radius"] = int(radius * K)
        lod1["radius"] = int(radius * K)
        name = "aaa_%03d" % index
        mesh_obj(name + "_lod0_cold", pad, lod0)
        mesh_obj(name + "_lod0_s1v0", intact, lod0)
        mesh_obj(name + "_lod0_s1v1", wreck, lod0)
        far(lod1, name, boxes, CONCRETE)
