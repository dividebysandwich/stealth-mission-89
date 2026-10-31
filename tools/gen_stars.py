#!/usr/bin/env python3
"""Packs the Yale Bright Star Catalogue (5th ed., Hoffleit & Warren 1991; bsc5.dat from
http://tdc-www.harvard.edu/catalogs/bsc5.html or CDS V/50) into assets/sky/stars.bin, the
night sky's stars (src/stars.rs).

Each star, brightest first, is 6 bytes little endian: right ascension (J2000) u16 of a full
turn, declination i16 of a quarter turn (+-32767 = +-90 degrees), V magnitude i8 in tenths,
B-V colour index i8 in fortieths.

    tools/gen_stars.py bsc5.dat assets/sky/stars.bin
"""
import struct
import sys


def main(src, dst):
    stars = []
    for line in open(src, encoding="latin-1"):
        ra, de, v = line[75:83], line[83:90], line[102:107]
        if not ra.strip() or not v.strip():
            # Entries without a position (novae, non-stellar objects removed from the catalogue).
            continue
        h, m, s = int(ra[0:2]), int(ra[2:4]), float(ra[4:8])
        sign = -1 if de[0] == "-" else 1
        d, dm, ds = int(de[1:3]), int(de[3:5]), int(de[5:7])
        bv = line[109:114].strip()
        stars.append((
            float(v),
            (h + m / 60 + s / 3600) / 24,
            sign * (d + dm / 60 + ds / 3600) / 90,
            float(bv) if bv else 0.6,
        ))
    stars.sort()
    out = bytearray()
    for v, ra, de, bv in stars:
        out += struct.pack(
            "<Hhbb",
            round(ra * 65536) % 65536,
            round(de * 32767),
            max(-128, min(127, round(v * 10))),
            max(-128, min(127, round(bv * 40))),
        )
    open(dst, "wb").write(out)
    print(f"{len(stars)} stars, {len(out)} bytes")


if __name__ == "__main__":
    main(sys.argv[1], sys.argv[2])
