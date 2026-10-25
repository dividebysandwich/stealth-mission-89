"""Geography of the Cuba theater (1994): Cuba with the Isla de la Juventud, the Florida Keys and
the south of Florida, the Bahamas, Jamaica, the Cayman Islands and the west of Haiti.

The theater squeezes 85.15-73.70 W, 16.0-26.68 N (about 1190 by 1190 km) into the 512 km of a
theater. Coordinates here are (lon, lat) degrees. The original game laid Cuba out on the
theater's diagonal; an unrotated map can only lay it across the middle, so the island runs from
the west edge (Cabo San Antonio) to the east edge (Punta Maisi) with the Florida Straits and the
Keys above it and the Caribbean below.

Sides: the United States with the Keys and Florida, the Bahamas, Jamaica, the Caymans and Haiti
are the friendly side; Cuba is the enemy. The US naval station at Guantanamo Bay is US ground,
its airfield (Leeward Point) a neutral field. There is no war on the ground: the front line is
the fence round the Guantanamo enclave, where the two sides stand face to face.
"""

PROJECTION = dict(lon0=-85.15, lat0=16.0, lat_ref=21.3, squeeze=2.32)

LIBRARY = "cuba"
LAND_RUBBLE = 73

DATE = (11, 3, 1994)
GRID = ("CU", 0, 0)
SQUADRONS = 6
CLOSED_BASES = ["Guantanamo"]

ORDER_OF_BATTLE = [
    ("an72", "none", "special"),
    ("mig17", "none", "base"),
    ("mig21", "aa2", "base"),
    ("mig21", "aa2", "escort"),
    ("mig23", "aa2", "patrol"),
    ("mig23", "aa2", "base"),
    ("mig29", "aa10t", "base"),
    ("mig29", "aa10t", "escort"),
    ("f4e_blue", "none", "friendly"),
    ("f4e_blue", "none", "friendly"),
    ("f4e_blue", "none", "friendly"),
    ("f4e_blue", "none", "friendly"),
]

# Radar types (units.ron ids): the fixed SAM sites (SA-2, SA-5), the mobile ones and the patrol
# boats (Osa and Komar types with their point defence).
SAM_RADARS = ["sa2", "sa5"]
MOBILE_SAM_RADARS = ["sa13", "sa8"]
BOAT_RADARS = ["san4", "san5"]
SLAG_HEAP_TOWNS = set()

# Models this theater builds besides the water, forests and mountains (see models.py).
EXTRA_MODELS = []

# ---------------------------------------------------------------------------------------------
# Land and water: everything outside `LANDS` is sea.

LANDS = [
    # Cuba, clockwise from Cabo San Antonio: the north coast of Pinar del Rio, Havana, Matanzas
    # with the Hicacos peninsula (Varadero), the coast behind the Sabana-Camaguey cays, the bay of
    # Nuevitas behind the Sabinal peninsula, Gibara, the bay of Nipe, Moa, Baracoa and Punta
    # Maisi; then the south coast: Guantanamo Bay, Santiago Bay, the Sierra Maestra shore, Cabo
    # Cruz, the Gulf of Guacanayabo, the Gulf of Ana Maria, Trinidad, Cienfuegos Bay, the Bay of
    # Pigs, the Zapata peninsula, the Ensenada de la Broa, the Gulf of Batabano, Cabo Frances and
    # the bay of Corrientes.
    [
        (-84.95, 21.86), (-84.88, 21.92), (-84.75, 21.96), (-84.60, 21.99), (-84.45, 22.03), (-84.33, 22.05),
        (-84.27, 22.05), (-84.29, 22.13), (-84.35, 22.22), (-84.42, 22.30), (-84.38, 22.37), (-84.25, 22.47),
        (-84.10, 22.56), (-83.95, 22.66), (-83.80, 22.74), (-83.62, 22.80), (-83.45, 22.86), (-83.25, 22.92),
        (-83.10, 22.97), (-82.93, 23.01), (-82.75, 23.05), (-82.58, 23.07), (-82.45, 23.12), (-82.37, 23.15),
        (-82.25, 23.17), (-82.10, 23.18), (-81.95, 23.17), (-81.80, 23.13), (-81.65, 23.10),
        # Matanzas Bay
        (-81.62, 23.08), (-81.585, 23.05), (-81.555, 23.05), (-81.52, 23.07), (-81.47, 23.10),
        # the Hicacos peninsula and the bay of Cardenas
        (-81.35, 23.12), (-81.25, 23.155), (-81.15, 23.20), (-81.18, 23.17), (-81.27, 23.11), (-81.24, 23.07),
        (-81.17, 23.055), (-81.05, 23.03), (-80.90, 23.03), (-80.75, 23.01), (-80.58, 22.98), (-80.40, 22.94),
        (-80.22, 22.92), (-80.05, 22.95), (-79.90, 22.88), (-79.75, 22.75), (-79.60, 22.63), (-79.47, 22.53),
        (-79.30, 22.43), (-79.15, 22.37), (-79.00, 22.37), (-78.82, 22.40), (-78.68, 22.35), (-78.55, 22.30),
        (-78.40, 22.24), (-78.25, 22.17), (-78.10, 22.07), (-77.95, 21.97), (-77.80, 21.88), (-77.65, 21.78),
        (-77.50, 21.72), (-77.35, 21.73), (-77.15, 21.70), (-77.10, 21.64),
        # the bay of Nuevitas
        (-77.15, 21.60), (-77.25, 21.60), (-77.32, 21.585), (-77.30, 21.565), (-77.22, 21.555), (-77.12, 21.55),
        (-77.06, 21.58), (-77.00, 21.60),
        (-76.88, 21.50), (-76.75, 21.40), (-76.62, 21.30), (-76.50, 21.24), (-76.35, 21.18), (-76.20, 21.14),
        (-76.12, 21.12), (-75.95, 21.12), (-75.80, 21.12), (-75.68, 21.09), (-75.60, 21.03), (-75.64, 20.95),
        (-75.66, 20.87), (-75.70, 20.85),
        # the bay of Nipe
        (-75.75, 20.82), (-75.86, 20.77), (-75.80, 20.70), (-75.68, 20.69), (-75.58, 20.73), (-75.52, 20.80),
        (-75.40, 20.75), (-75.25, 20.72), (-75.10, 20.70), (-74.95, 20.68), (-74.82, 20.60), (-74.68, 20.48),
        (-74.55, 20.40), (-74.49, 20.375), (-74.44, 20.34), (-74.35, 20.30), (-74.22, 20.27), (-74.13, 20.24),
        # Punta Maisi round to the south coast
        (-74.18, 20.15), (-74.30, 20.08), (-74.50, 20.06), (-74.65, 20.06), (-74.85, 20.04), (-75.00, 19.98),
        (-75.08, 19.91), (-75.12, 19.89),
        # Guantanamo Bay up to Caimanera
        (-75.12, 19.95), (-75.10, 20.00), (-75.13, 20.07), (-75.17, 20.10), (-75.20, 20.05), (-75.19, 19.95),
        (-75.195, 19.90), (-75.22, 19.895),
        (-75.35, 19.88), (-75.50, 19.89), (-75.65, 19.92), (-75.78, 19.955), (-75.85, 19.965),
        # Santiago Bay
        (-75.86, 19.975), (-75.85, 20.00), (-75.84, 20.02), (-75.87, 20.04), (-75.89, 20.01), (-75.88, 19.975),
        (-75.90, 19.96), (-75.95, 19.95), (-76.10, 19.95), (-76.25, 19.96), (-76.40, 19.97), (-76.55, 19.94),
        (-76.70, 19.90), (-76.85, 19.88), (-77.00, 19.88), (-77.15, 19.90), (-77.30, 19.89), (-77.45, 19.85),
        (-77.60, 19.83), (-77.73, 19.83),
        # Cabo Cruz, the Gulf of Guacanayabo and the Cauto's mouth
        (-77.70, 19.92), (-77.65, 20.00), (-77.58, 20.05), (-77.48, 20.12), (-77.38, 20.20), (-77.25, 20.27),
        (-77.13, 20.33), (-77.10, 20.42), (-77.18, 20.52), (-77.25, 20.57), (-77.40, 20.62), (-77.55, 20.65),
        (-77.70, 20.66), (-77.85, 20.68), (-78.00, 20.71), (-78.15, 20.78), (-78.30, 20.87), (-78.45, 20.98),
        (-78.58, 21.12), (-78.70, 21.30), (-78.80, 21.48), (-78.85, 21.60), (-79.00, 21.62), (-79.20, 21.60),
        (-79.40, 21.58), (-79.55, 21.62), (-79.70, 21.66), (-79.85, 21.70), (-79.98, 21.72), (-80.10, 21.80),
        (-80.25, 21.90), (-80.38, 21.98), (-80.43, 22.03),
        # Cienfuegos Bay
        (-80.44, 22.06), (-80.45, 22.12), (-80.46, 22.17), (-80.50, 22.19), (-80.54, 22.15), (-80.52, 22.08),
        (-80.47, 22.04), (-80.52, 22.02), (-80.65, 22.03), (-80.80, 22.04), (-80.95, 22.05), (-81.03, 22.06),
        # the Bay of Pigs
        (-81.10, 22.08), (-81.15, 22.15), (-81.17, 22.24), (-81.20, 22.29), (-81.24, 22.22), (-81.25, 22.12),
        (-81.30, 22.08),
        # the Zapata peninsula and the Ensenada de la Broa
        (-81.45, 22.08), (-81.60, 22.10), (-81.75, 22.13), (-81.90, 22.17), (-82.03, 22.26), (-81.95, 22.35),
        (-81.90, 22.42), (-81.93, 22.51),
        # the Gulf of Batabano shore
        (-82.05, 22.58), (-82.15, 22.65), (-82.30, 22.69), (-82.45, 22.70), (-82.60, 22.68), (-82.75, 22.64),
        (-82.90, 22.57), (-83.05, 22.48), (-83.20, 22.40), (-83.35, 22.33), (-83.50, 22.27), (-83.62, 22.22),
        (-83.75, 22.14), (-83.87, 22.07), (-83.95, 22.00), (-83.98, 21.95), (-84.04, 21.90),
        # the bay of Corrientes
        (-84.15, 21.93), (-84.25, 21.93), (-84.40, 21.88), (-84.48, 21.82), (-84.51, 21.75), (-84.62, 21.78),
        (-84.75, 21.80), (-84.88, 21.83),
    ],
    # Isla de la Juventud (the Siguanea bay on its west side) and Cayo Largo.
    [(-82.98, 21.93), (-82.80, 21.91), (-82.65, 21.85), (-82.55, 21.75), (-82.55, 21.57), (-82.75, 21.45),
     (-82.95, 21.43), (-83.10, 21.47), (-83.18, 21.60), (-83.05, 21.63), (-83.05, 21.75), (-83.10, 21.85)],
    [(-81.68, 21.625), (-81.50, 21.615), (-81.38, 21.58), (-81.40, 21.565), (-81.52, 21.59), (-81.68, 21.605)],
    # The Sabana-Camaguey cays: Santa Maria, Coco, Romano.
    [(-79.25, 22.68), (-78.95, 22.67), (-78.95, 22.63), (-79.20, 22.64)],
    [(-78.65, 22.55), (-78.40, 22.55), (-78.20, 22.45), (-78.35, 22.42), (-78.60, 22.48)],
    [(-78.15, 22.27), (-77.95, 22.22), (-77.70, 22.08), (-77.50, 21.95), (-77.55, 21.90), (-77.75, 21.98),
     (-78.00, 22.12), (-78.18, 22.22)],
    # Florida south of Charlotte Harbor: the Gulf coast to Cape Sable, Florida Bay, Biscayne Bay and
    # the Atlantic coast past Miami.
    [(-82.80, 28.50), (-82.65, 27.50), (-82.25, 26.95), (-82.10, 26.65), (-81.95, 26.48), (-81.82, 26.30),
     (-81.80, 26.10), (-81.72, 25.92), (-81.50, 25.82), (-81.30, 25.62), (-81.15, 25.35), (-81.12, 25.12),
     (-80.90, 25.14), (-80.65, 25.20), (-80.45, 25.25), (-80.35, 25.40), (-80.30, 25.55), (-80.20, 25.72),
     (-80.13, 25.85), (-80.10, 26.10), (-80.06, 26.40), (-80.03, 26.75), (-80.05, 27.50), (-80.30, 28.50)],
    # The Florida Keys: the Lower Keys from Key West and Boca Chica to Big Pine Key, the Middle
    # and Upper Keys from Marathon to Key Largo.
    [(-81.82, 24.53), (-81.60, 24.555), (-81.38, 24.62), (-81.30, 24.67), (-81.35, 24.71), (-81.55, 24.63),
     (-81.80, 24.59)],
    [(-81.25, 24.66), (-81.00, 24.70), (-80.75, 24.83), (-80.50, 25.00), (-80.30, 25.25), (-80.35, 25.28),
     (-80.55, 25.05), (-80.80, 24.88), (-81.05, 24.75), (-81.25, 24.70)],
    # The Bahamas: Grand Bahama, Abaco, Andros, New Providence, Eleuthera, Great Exuma, Cat
    # Island, Long Island, Crooked and Acklins, Great Inagua.
    [(-79.00, 26.70), (-78.60, 26.75), (-78.20, 26.70), (-77.90, 26.62), (-78.00, 26.55), (-78.50, 26.55),
     (-78.75, 26.50), (-79.00, 26.60)],
    [(-77.55, 26.90), (-77.10, 26.60), (-77.00, 26.25), (-77.20, 25.88), (-77.35, 25.95), (-77.25, 26.30),
     (-77.30, 26.60), (-77.65, 26.85)],
    [(-78.25, 25.15), (-77.90, 25.10), (-77.70, 24.75), (-77.60, 24.25), (-77.75, 23.85), (-77.95, 23.70),
     (-78.15, 24.00), (-78.40, 24.40), (-78.40, 24.80)],
    [(-77.55, 25.08), (-77.25, 25.08), (-77.25, 24.98), (-77.55, 24.98)],
    [(-76.80, 25.55), (-76.60, 25.45), (-76.40, 25.20), (-76.15, 24.90), (-76.15, 24.65), (-76.25, 24.75),
     (-76.50, 25.15), (-76.75, 25.40)],
    [(-76.05, 23.70), (-75.75, 23.50), (-75.70, 23.40), (-75.95, 23.55), (-76.15, 23.70)],
    [(-75.75, 24.65), (-75.55, 24.45), (-75.30, 24.15), (-75.40, 24.15), (-75.65, 24.45), (-75.80, 24.60)],
    [(-75.35, 23.65), (-75.05, 23.25), (-74.85, 22.85), (-74.90, 22.85), (-75.10, 23.15), (-75.30, 23.45)],
    [(-74.35, 22.85), (-73.85, 22.75), (-73.85, 22.35), (-74.05, 22.20), (-74.30, 22.40), (-74.15, 22.65)],
    [(-73.75, 21.20), (-73.20, 21.15), (-73.10, 20.95), (-73.70, 20.92)],
    # Jamaica: Negril, Montego Bay, Ocho Rios, Port Antonio, Morant Point, Kingston, Portland
    # Point, Black River.
    [(-78.37, 18.28), (-78.20, 18.45), (-77.92, 18.50), (-77.60, 18.49), (-77.25, 18.45), (-77.00, 18.40),
     (-76.75, 18.30), (-76.45, 18.20), (-76.25, 18.03), (-76.20, 17.90), (-76.45, 17.87), (-76.80, 17.93),
     (-77.05, 17.88), (-77.18, 17.73), (-77.45, 17.85), (-77.75, 17.88), (-77.95, 18.10), (-78.20, 18.20)],
    # The Cayman Islands: Grand Cayman, Little Cayman and Cayman Brac.
    [(-81.42, 19.30), (-81.38, 19.38), (-81.10, 19.35), (-81.10, 19.28), (-81.40, 19.26)],
    [(-80.12, 19.66), (-79.95, 19.70), (-79.95, 19.68), (-80.12, 19.645)],
    [(-79.90, 19.69), (-79.72, 19.755), (-79.72, 19.735), (-79.90, 19.67)],
    # Hispaniola's west: Haiti's northern and southern (Tiburon) peninsulas round the Gulf of
    # Gonave, coarse beyond the theater's east edge.
    [(-72.00, 19.95), (-73.45, 19.85), (-73.40, 19.60), (-72.75, 19.45), (-72.60, 19.00), (-72.40, 18.80),
     (-72.70, 18.60), (-73.40, 18.60), (-73.90, 18.62), (-74.20, 18.66), (-74.47, 18.45), (-74.42, 18.30),
     (-74.10, 18.25), (-73.80, 18.18), (-73.50, 18.20), (-72.00, 18.20)],
    # The Mosquito Coast of Honduras just below the south-west corner.
    [(-86.50, 15.80), (-85.90, 15.95), (-85.40, 15.92), (-85.05, 15.98), (-84.60, 15.85), (-84.00, 15.60),
     (-83.40, 15.25), (-83.15, 15.00), (-86.50, 14.00)],
]

# Lagoons, lakes and reservoirs.
LAKES = [
    # Laguna de la Leche by Moron, the Zaza reservoir, the Hanabanilla reservoir in the
    # Escambray, the Laguna del Tesoro in the Zapata swamp
    [(-78.70, 22.25), (-78.58, 22.27), (-78.55, 22.20), (-78.65, 22.17)],
    [(-79.42, 21.86), (-79.32, 21.90), (-79.25, 21.84), (-79.35, 21.78)],
    [(-80.14, 22.07), (-80.10, 22.07), (-80.10, 22.03), (-80.14, 22.03)],
    [(-81.16, 22.38), (-81.10, 22.38), (-81.10, 22.34), (-81.16, 22.34)],
    # Lake Okeechobee (its south shore in the theater)
    [(-80.95, 26.72), (-80.65, 26.78), (-80.62, 26.98), (-80.80, 27.18), (-81.00, 27.00)],
]

# Rivers: polylines of (lon, lat, width in theater km).
RIVERS = [
    # Cauto from the Sierra Maestra past Palma Soriano and Cauto Cristo to the Gulf of Guacanayabo
    [(-75.95, 20.15, 0.05), (-76.05, 20.25, 0.06), (-76.25, 20.40, 0.07), (-76.45, 20.52, 0.08), (-76.70, 20.55, 0.1),
     (-76.95, 20.58, 0.1), (-77.10, 20.57, 0.12), (-77.24, 20.57, 0.2)],
    # Zaza through its reservoir to the sea at Tunas de Zaza
    [(-79.70, 22.25, 0.04), (-79.55, 22.05, 0.05), (-79.40, 21.88, 0.05), (-79.40, 21.75, 0.06), (-79.48, 21.65, 0.08),
     (-79.54, 21.63, 0.12)],
    # Sagua la Grande
    [(-80.05, 22.35, 0.04), (-80.08, 22.60, 0.04), (-80.095, 22.80, 0.05), (-80.04, 22.94, 0.07)],
    # Cuyaguateje in Pinar del Rio
    [(-83.85, 22.55, 0.03), (-84.00, 22.30, 0.04), (-84.05, 22.05, 0.04), (-84.05, 21.94, 0.05)],
    # Mayari into the bay of Nipe, Toa to the sea by Baracoa
    [(-75.66, 20.45, 0.03), (-75.70, 20.60, 0.04), (-75.71, 20.70, 0.05)],
    [(-74.85, 20.42, 0.04), (-74.70, 20.42, 0.05), (-74.57, 20.40, 0.06)],
]

# ---------------------------------------------------------------------------------------------
# Sides

# The fence round the Guantanamo Bay naval station, west shore to east shore: (lon, lat,
# intensity).
FRONT = [(lon, lat, 0.2) for lon, lat in [
    (-75.25, 19.89), (-75.25, 19.95), (-75.22, 19.98), (-75.18, 19.99), (-75.13, 20.00), (-75.08, 19.97),
    (-75.06, 19.92),
]]

# The friendly side: the United States with the Keys and the Bahamas north of a line through the
# Florida Straits and the Old Bahama Channel; Jamaica, the Caymans and Haiti south of a line
# through the Caribbean and the Windward Passage; the Guantanamo enclave.
BLUE = [
    [(-90.0, 23.50), (-84.0, 23.50), (-82.0, 23.75), (-80.5, 23.45), (-79.3, 23.10), (-78.2, 22.95), (-77.0, 22.25),
     (-75.8, 21.75), (-74.9, 21.30), (-73.9, 20.75), (-72.5, 20.70), (-70.0, 21.0), (-70.0, 30.0), (-90.0, 30.0)],
    [(-90.0, 10.0), (-70.0, 10.0), (-70.0, 20.60), (-73.8, 19.90), (-74.6, 19.40), (-75.5, 19.40), (-77.0, 19.40),
     (-78.0, 19.40), (-79.0, 19.90), (-79.5, 20.0), (-81.0, 20.0), (-83.0, 20.50), (-85.0, 20.80), (-90.0, 20.80)],
    [(lon, lat) for lon, lat, _ in FRONT] + [(-75.06, 19.85), (-75.25, 19.85)],
]

# Neutral places: the player never flies from or to them, they are no one's targets.
NEUTRAL = {"Nassau", "Freeport", "Kingston", "Montego Bay", "George Town", "Jeremie"}

# ---------------------------------------------------------------------------------------------
# Places

# Towns: (name, lon, lat, kind 0 village / 1 town / 2 city, radius in theater metres, ruined
# share, in the world name table).
TOWNS = [
    # Pinar del Rio and Havana
    ("Havana", -82.36, 23.12, 2, 3400, 0.0, True),
    ("Pinar del Rio", -83.70, 22.42, 2, 1700, 0.0, True),
    ("Guane", -84.09, 22.20, 0, 700, 0.0, False),
    ("Vinales", -83.71, 22.62, 0, 600, 0.0, False),
    ("San Cristobal", -83.05, 22.72, 1, 900, 0.0, False),
    ("Mariel", -82.75, 23.02, 1, 900, 0.0, True),
    ("Artemisa", -82.76, 22.81, 1, 1000, 0.0, False),
    ("Guira", -82.51, 22.795, 1, 900, 0.0, True),
    ("San Antonio de los Banos", -82.50, 22.89, 1, 900, 0.0, False),
    ("Bejucal", -82.39, 22.93, 0, 700, 0.0, False),
    ("Managua", -82.29, 22.95, 0, 700, 0.0, True),
    ("Batabano", -82.30, 22.72, 0, 700, 0.0, False),
    ("Guines", -82.03, 22.84, 1, 1200, 0.0, True),
    ("Madruga", -81.86, 22.91, 0, 600, 0.0, False),
    ("Jaruco", -82.01, 23.04, 0, 600, 0.0, False),
    ("Santa Cruz del Norte", -81.92, 23.15, 0, 600, 0.0, False),
    ("Nueva Gerona", -82.80, 21.88, 1, 1000, 0.0, True),
    ("Santa Fe", -82.75, 21.75, 0, 600, 0.0, False),
    # Matanzas, Las Villas and Cienfuegos
    ("Matanzas", -81.58, 23.035, 2, 1700, 0.0, True),
    ("Cardenas", -81.21, 23.03, 1, 1200, 0.0, True),
    ("Varadero", -81.24, 23.145, 0, 700, 0.0, False),
    ("Colon", -80.90, 22.72, 1, 900, 0.0, False),
    ("Jaguey Grande", -81.13, 22.53, 0, 700, 0.0, False),
    ("Playa Giron", -81.03, 22.08, 0, 600, 0.0, True),
    ("Cienfuegos", -80.43, 22.15, 2, 1800, 0.0, True),
    ("Santa Clara", -79.96, 22.41, 2, 2000, 0.0, True),
    ("Sagua la Grande", -80.07, 22.81, 1, 1000, 0.0, False),
    ("Caibarien", -79.48, 22.51, 1, 900, 0.0, False),
    ("Placetas", -79.65, 22.31, 1, 900, 0.0, False),
    ("Manicaragua", -79.98, 22.15, 0, 700, 0.0, False),
    ("Trinidad", -79.98, 21.80, 1, 1100, 0.0, True),
    ("Sancti Spiritus", -79.44, 21.93, 2, 1500, 0.0, True),
    ("Cabaiguan", -79.50, 22.08, 0, 700, 0.0, False),
    # Ciego de Avila and Camaguey
    ("Chambas", -78.91, 22.19, 0, 700, 0.0, True),
    ("Moron", -78.63, 22.11, 1, 1000, 0.0, True),
    ("Ciego de Avila", -78.76, 21.84, 2, 1500, 0.0, True),
    ("Cunagua", -78.35, 22.08, 0, 600, 0.0, True),
    ("Camaguey", -77.92, 21.38, 2, 2400, 0.0, True),
    ("Nuevitas", -77.26, 21.545, 1, 1000, 0.0, True),
    ("Esmeralda", -78.12, 21.85, 0, 600, 0.0, False),
    ("Vertientes", -78.15, 21.26, 0, 600, 0.0, False),
    ("Santa Cruz del Sur", -77.99, 20.73, 0, 700, 0.0, False),
    # Oriente
    ("Las Tunas", -76.95, 20.96, 2, 1500, 0.0, True),
    ("Puerto Padre", -76.60, 21.19, 1, 900, 0.0, False),
    ("Holguin", -76.26, 20.89, 2, 2000, 0.0, True),
    ("Gibara", -76.13, 21.10, 0, 700, 0.0, False),
    ("Banes", -75.72, 20.96, 1, 900, 0.0, False),
    ("Mayari", -75.68, 20.66, 1, 900, 0.0, False),
    ("Moa", -74.94, 20.66, 1, 1000, 0.0, False),
    ("Baracoa", -74.50, 20.35, 1, 1000, 0.0, True),
    ("Bayamo", -76.64, 20.38, 2, 1600, 0.0, True),
    ("Manzanillo", -77.10, 20.34, 2, 1500, 0.0, True),
    ("Niquero", -77.56, 20.04, 0, 700, 0.0, False),
    ("Palma Soriano", -76.00, 20.21, 1, 1100, 0.0, False),
    ("Santiago de Cuba", -75.82, 20.02, 2, 2600, 0.0, True),
    ("Guantanamo", -75.21, 20.14, 2, 1600, 0.0, True),
    # Florida and the Keys
    ("Key West", -81.78, 24.555, 1, 1000, 0.0, True),
    ("Marathon", -81.09, 24.71, 0, 600, 0.0, False),
    ("Miami", -80.23, 25.77, 2, 3200, 0.0, True),
    ("Homestead", -80.48, 25.47, 1, 900, 0.0, False),
    ("Fort Lauderdale", -80.15, 26.12, 2, 2000, 0.0, False),
    ("Naples", -81.77, 26.15, 1, 1000, 0.0, False),
    # The Bahamas, Jamaica, the Caymans and Haiti
    ("Nassau", -77.35, 25.05, 1, 1300, 0.0, True),
    ("Freeport", -78.70, 26.53, 1, 900, 0.0, False),
    ("Kingston", -76.79, 18.00, 2, 2400, 0.0, True),
    ("Montego Bay", -77.92, 18.46, 1, 1000, 0.0, False),
    ("George Town", -81.38, 19.30, 0, 700, 0.0, False),
    ("Jeremie", -74.12, 18.62, 1, 800, 0.0, False),
]

# Airbases: (name, lon, lat, side: True friendly, False enemy, None neutral). The runway runs
# north-south.
AIRBASES = [
    ("Havana", -82.41, 22.99, False),  # Jose Marti
    ("San Julian", -84.15, 22.10, False),
    ("Guira", -82.51, 22.87, False),  # San Antonio de los Banos
    ("Cienfuegos", -80.41, 22.15, False),
    ("Santa Clara", -79.94, 22.49, False),  # Abel Santamaria
    ("Chambas", -78.92, 22.16, False),
    ("Ciego de Avila", -78.79, 22.03, False),  # Maximo Gomez
    ("Camaguey", -77.85, 21.42, False),  # Ignacio Agramonte
    ("Nuevitas", -77.33, 21.50, False),
    ("Santiago de Cuba", -76.32, 20.79, False),  # the Frank Pais field at Holguin (see the report)
    ("Antonio Maceo", -75.83, 19.975, False),  # Santiago de Cuba's airport
    ("Baracoa", -74.51, 20.36, False),  # Gustavo Rizo
    ("Managua", -82.31, 22.90, False),
    ("Guantanamo", -75.215, 19.905, None),  # Leeward Point
    ("Key West", -81.69, 24.575, True),  # Boca Chica
]

# The friendly carrier group in the deep water of the Cayman Trough.
CARRIER = ("USS America", -82.55, 18.95)

# The theater's own layouts (metres east, metres north of a 2 km cell's centre, library kind):
# the library has no houses, tanks or mobile SAMs, so its sites use its own kinds.
TEMPLATES = {
    # An army or naval headquarters with its depot and barracks.
    "cuba_hq": [(0, 0, "headquarters"), (-350, 0, "depot"), (-500, -450, "buildings"), (650, 150, "buildings"),
                (-600, 500, "buildings"), (350, -450, "supply_dump")],
    # An arms and supply depot.
    "cuba_depot": [(-300, 0, "depot"), (0, 0, "depot"), (300, 0, "depot"), (0, 450, "tank_farm"),
                   (450, -350, "buildings"), (-350, -450, "supply_dump")],
    # A fuel depot, refinery or tanker terminal.
    "fuel": [(-150, -150, "tank_farm"), (150, -150, "tank_farm"), (-150, 150, "tank_farm"), (150, 150, "tank_farm"),
             (550, 0, "buildings"), (-500, -450, "buildings")],
    # A guerrilla training camp: the camp, its huts and its stores.
    "camp": [(0, 0, "terrorist_camp"), (-300, 250, "buildings"), (250, 350, "buildings"), (350, -250, "supply_dump")],
    # A supply dump in the hills.
    "supply": [(-250, 0, "supply_dump"), (250, 0, "supply_dump"), (0, 400, "supply_dump"), (0, -400, "depot"),
               (500, 400, "buildings")],
}

# Enemy target sites: (template, lon, lat, name).
SITES = [
    # Army, navy and air force headquarters: MINFAR in Havana, the Soviet listening station at
    # Lourdes, the Western Army's tank division at Managua, the province commands, the Central
    # and Eastern Army commands at Matanzas and Santiago, the Cienfuegos naval base.
    ("cuba_hq", -82.385, 23.11, "Havana"), ("cuba_hq", -82.48, 22.96, "Lourdes"), ("cuba_hq", -82.26, 22.97, "Managua"),
    ("cuba_hq", -83.67, 22.39, "Pinar del Rio"), ("cuba_hq", -81.62, 23.01, "Matanzas"),
    ("cuba_hq", -79.99, 22.43, "Santa Clara"), ("cuba_hq", -80.42, 22.08, "Cienfuegos"),
    ("cuba_hq", -79.42, 21.95, "Sancti Spiritus"), ("cuba_hq", -78.74, 21.86, "Ciego de Avila"),
    ("cuba_hq", -77.90, 21.35, "Camaguey"), ("cuba_hq", -76.97, 20.93, "Las Tunas"), ("cuba_hq", -76.23, 20.87, "Holguin"),
    ("cuba_hq", -76.62, 20.36, "Bayamo"), ("cuba_hq", -75.79, 20.05, "Santiago de Cuba"),
    ("cuba_hq", -75.23, 20.17, "Guantanamo"), ("cuba_hq", -82.82, 21.85, "Nueva Gerona"),
    # Missile sites: the 1962 MRBM and IRBM sites of San Cristobal, Guanajay, Sagua la Grande and
    # Remedios, and later ones down the island.
    ("ss", -83.10, 22.76, "San Cristobal"), ("ss", -82.95, 22.75, "Candelaria"), ("ss", -82.67, 22.92, "Guanajay"),
    ("ss", -80.12, 22.72, "Sagua la Grande"), ("ss", -79.55, 22.46, "Remedios"), ("ss", -78.97, 22.10, "Florencia"),
    ("ss", -77.53, 21.24, "Sibanicu"), ("ss", -76.25, 21.05, "Velasco"), ("ss", -76.42, 20.37, "Jiguani"),
    ("ss", -76.25, 20.30, "Contramaestre"),
    # Arms depots: the Bejucal warhead store, the ports, the army depots inland.
    ("cuba_depot", -82.38, 22.92, "Bejucal"), ("cuba_depot", -82.77, 23.00, "Mariel"),
    ("cuba_depot", -82.76, 22.84, "Artemisa"), ("cuba_depot", -81.10, 22.52, "Jaguey Grande"),
    ("cuba_depot", -80.91, 22.74, "Colon"), ("cuba_depot", -79.97, 22.17, "Manicaragua"),
    ("cuba_depot", -79.67, 22.33, "Placetas"), ("cuba_depot", -78.60, 22.09, "Moron"),
    ("cuba_depot", -78.15, 21.24, "Vertientes"), ("cuba_depot", -76.62, 21.16, "Puerto Padre"),
    ("cuba_depot", -75.66, 20.63, "Mayari"), ("cuba_depot", -75.98, 20.23, "Palma Soriano"),
    ("cuba_depot", -77.08, 20.32, "Manzanillo"), ("cuba_depot", -74.96, 20.63, "Moa"),
    ("cuba_depot", -74.63, 20.09, "Imias"), ("cuba_depot", -81.23, 23.01, "Cardenas"),
    ("cuba_depot", -79.95, 21.83, "Trinidad"),
    # Fuel: the Havana (Nico Lopez), Cienfuegos and Santiago refineries, the Matanzas supertanker
    # terminal, the ports of Nuevitas and Antilla.
    ("fuel", -82.32, 23.13, "Regla"), ("fuel", -81.52, 23.045, "Matanzas"), ("fuel", -80.48, 22.21, "Cienfuegos"),
    ("fuel", -75.86, 20.05, "Santiago de Cuba"), ("fuel", -77.24, 21.53, "Nuevitas"), ("fuel", -75.74, 20.86, "Antilla"),
    # Guerrilla training camps: Punto Cero at Guanabo, Bauta, the Vinales hills, the Isla de la
    # Juventud, Minas, Cueto, La Plata in the Sierra Maestra, El Cobre.
    ("camp", -82.11, 23.13, "Guanabo"), ("camp", -82.55, 22.97, "Bauta"), ("camp", -83.75, 22.60, "Vinales"),
    ("camp", -82.75, 21.72, "La Fe"), ("camp", -77.60, 21.49, "Minas"), ("camp", -75.93, 20.65, "Cueto"),
    ("camp", -76.88, 20.06, "La Plata"), ("camp", -75.95, 20.06, "El Cobre"),
    # Supply dumps in the hills.
    ("supply", -83.40, 22.62, "La Palma"), ("supply", -80.10, 21.95, "Topes de Collantes"),
    ("supply", -77.70, 21.65, "Sierra de Cubitas"), ("supply", -76.55, 20.12, "Bartolome Maso"),
    ("supply", -75.30, 20.40, "Sagua de Tanamo"),
]

# Cities of the enemy side with a SAM site at their edge.
CITY_SAMS = ["Guines", "Matanzas", "Cunagua", "Holguin", "Nueva Gerona", "Pinar del Rio", "Sancti Spiritus", "Las Tunas",
             "Bayamo"]

# Patrol boats off Cuba's coasts: (lon, lat).
BOATS = [(-82.75, 23.15), (-82.20, 23.30), (-81.05, 23.30), (-78.30, 22.75), (-76.10, 21.25), (-74.05, 20.05),
         (-75.85, 19.85), (-77.50, 20.45), (-80.50, 21.90), (-82.50, 22.20)]

# Other ships: (kind, lon, lat, radar or None). The carrier's destroyers, tankers and cargo ships
# in the shipping lanes (the Florida Straits, the Yucatan Channel, the Windward Passage, the Old
# Bahama Channel) and off the Cuban ports, fishing boats on the banks.
SHIPS = [
    ("destroyer", -82.45, 19.02, None), ("destroyer", -82.66, 19.05, None), ("destroyer", -82.70, 18.88, None),
    ("destroyer", -82.42, 18.86, None), ("destroyer", -82.55, 19.15, None),
    ("tanker", -83.50, 24.10, None), ("tanker", -81.00, 24.10, None), ("tanker", -79.80, 25.30, None),
    ("tanker", -85.05, 21.50, None), ("tanker", -74.00, 19.60, None), ("tanker", -80.00, 18.50, None),
    ("tanker", -77.20, 22.60, None), ("tanker", -77.00, 17.30, None),
    ("merchant_ship", -82.35, 23.40, None), ("merchant_ship", -82.80, 23.30, None), ("merchant_ship", -81.50, 23.25, None),
    ("merchant_ship", -75.85, 19.75, None), ("merchant_ship", -80.40, 21.85, None), ("merchant_ship", -77.05, 21.80, None),
    ("merchant_ship", -79.00, 19.00, None), ("merchant_ship", -84.00, 24.00, None),
    ("trawler", -82.00, 22.10, None), ("trawler", -83.00, 22.20, None), ("trawler", -80.20, 23.80, None),
    ("trawler", -78.70, 21.20, None), ("trawler", -77.70, 20.55, None), ("trawler", -77.90, 19.70, None),
    ("trawler", -79.20, 20.90, None), ("trawler", -81.00, 24.90, None),
]

# Mountain ranges: (lon, lat, length theater km, width theater km, height m, bearing of the
# long axis in degrees from north).
MOUNTAINS = [
    # The Cordillera de Guaniguanico: Sierra de los Organos and Sierra del Rosario
    (-83.85, 22.55, 30, 10, 600, 60), (-83.25, 22.75, 30, 10, 700, 70),
    # The Escambray, the hills of Jatibonico, the Sierra de Cubitas
    (-80.05, 21.98, 28, 14, 1100, 80), (-79.20, 22.25, 18, 8, 450, 95), (-77.75, 21.65, 18, 6, 330, 100),
    # The Sierra Maestra from Cabo Cruz past Pico Turquino to the Gran Piedra
    (-77.45, 20.00, 30, 12, 1300, 80), (-76.95, 20.03, 30, 13, 1970, 90), (-76.40, 20.06, 30, 12, 1400, 95),
    (-75.65, 20.03, 20, 9, 1200, 85),
    # The Nipe-Sagua-Baracoa mountains: Sierra de Nipe, Sierra del Cristal and Moa, the
    # Cuchillas de Baracoa
    (-75.70, 20.50, 22, 10, 900, 70), (-75.20, 20.45, 30, 12, 1200, 95), (-74.75, 20.25, 24, 10, 1100, 110),
    # Jamaica: the central hills and the Blue Mountains; Haiti's Massif de la Hotte
    (-77.45, 18.20, 30, 14, 900, 95), (-76.60, 18.07, 22, 10, 2250, 100), (-74.05, 18.38, 26, 12, 2300, 95),
]

# Forest regions: (lon, lat, radius theater km, density 0..1).
FORESTS = [
    # Cuba: Guanahacabibes, the Organos and Rosario hills, the Zapata swamp, the Escambray, the
    # south of the Isla de la Juventud, the Sierra Maestra, the pine and rain forests of Nipe,
    # Moa and Baracoa, the Cubitas and Jatibonico hills
    (-84.55, 21.92, 12, 0.75), (-83.70, 22.60, 12, 0.6), (-83.05, 22.78, 10, 0.6), (-81.40, 22.30, 18, 0.6),
    (-80.05, 21.98, 12, 0.65), (-82.85, 21.55, 8, 0.6), (-77.35, 19.97, 12, 0.6), (-76.85, 20.03, 13, 0.75),
    (-76.30, 20.07, 11, 0.65), (-75.65, 20.04, 6, 0.5), (-75.65, 20.50, 9, 0.55), (-75.10, 20.45, 13, 0.75),
    (-74.70, 20.30, 10, 0.8), (-77.75, 21.65, 6, 0.5), (-79.30, 22.20, 8, 0.4),
    # Florida: the Everglades and Big Cypress; Andros's pine; Jamaica's Cockpit Country and Blue
    # Mountains; the Massif de la Hotte
    (-80.90, 25.55, 22, 0.3), (-81.35, 26.15, 16, 0.45), (-78.00, 24.40, 14, 0.4), (-77.65, 18.30, 12, 0.6),
    (-76.55, 18.08, 9, 0.7), (-74.05, 18.40, 10, 0.35),
]

# Green farmland, cane fields and pasture between the woods.
GREEN = True


# ---------------------------------------------------------------------------------------------
# Models of this theater's library the layouts use, and its airfield's aprons.

KINDS = {"houses": 26, "houses2": 26, "buildings": 26, "patrol_boat": 53, "sea": 20, "carrier": 75,
         "carrier_object": 75}
APRONS = [(-500, -78, -180, 180), (78, 500, -180, 180)]
LAND_COLOUR = 50
