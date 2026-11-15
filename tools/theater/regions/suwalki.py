"""Geography of the Suwalki Gap theater (2027): Kaliningrad, north-east Poland, Lithuania, the
south of Latvia and western Belarus.

The theater squeezes 19.2-27.8 E, 51.9-56.9 N (about 560 km square) into the 512 km of a
theater, close to true scale, so the gap between Kaliningrad and Belarus stays narrow.
Coordinates here are (lon, lat) degrees.

Sides: Poland, Lithuania and Latvia (NATO) are the friendly side; Kaliningrad, Belarus and Russia
the enemy. A few days into the incursion, Russian pincers have pushed out of Kaliningrad past
Goldap and Vistytis and out of Belarus past Druskininkai, Lazdijai and Sejny, leaving a corridor
of about 30 km round Suwalki and Kalvarija. Elsewhere the front still runs along the borders.
"""

import os
import re

PROJECTION = dict(lon0=19.2, lat0=51.9, lat_ref=54.4, squeeze=1.09)

LIBRARY = "central_europe"
LAND_RUBBLE = 102

DATE = (6, 1, 2027)
GRID = ("FE", 0, 0)
SQUADRONS = 4

EXTRA_MODELS = ["npp", "ruins", "rubble"]

SAM_RADARS = ["sa10", "sa21", "sa17", "sa22"]
MOBILE_SAM_RADARS = ["sa17", "sa15"]
BOAT_RADARS = ["san7", "san5"]

CITY_SAMS = ["Kaliningrad", "Baltiysk", "Grodno", "Lida", "Chernyakhovsk"]

ORDER_OF_BATTLE = [
    ("an72", "none", "special"),
    ("su34", "aa8", "base"),
    ("su35", "aa10", "patrol"),
    ("su27", "aa10t", "patrol"),
    ("a50", "aa10", "patrol_wide"),
    ("mig31", "aa9", "patrol"),
    ("mig31", "aa9", "escort"),
    ("su35", "aa10t", "escort"),
    ("su34", "aa10t", "base"),
    ("f16", "none", "friendly"),
    ("f16", "none", "friendly"),
    ("f16", "none", "friendly"),
    ("f16", "none", "friendly"),
]


# ---------------------------------------------------------------------------------------------
# Land and water: everything outside `LANDS` is sea; the lagoons are seas of their own.

LANDS = [
    # The mainland, coarse far outside the theater: the Polish coast, the Vistula Spit, Sambia,
    # the Curonian Spit, the Lithuanian and Latvian coasts, the Gulf of Riga, Estonia, the Gulf
    # of Finland, Finland and Sweden.
    [
        (14.2, 53.9), (15.5, 54.15), (16.5, 54.55), (17.5, 54.78), (18.35, 54.83), (18.55, 54.6), (18.65, 54.4),
        (19.0, 54.37), (19.3, 54.4), (19.6, 54.5), (19.88, 54.62), (19.92, 54.8), (19.92, 54.95), (20.3, 54.96),
        (20.5, 54.96), (20.8, 55.1), (21.0, 55.3), (21.08, 55.55), (21.1, 55.72), (21.07, 56.0), (21.05, 56.3),
        (21.0, 56.52), (21.1, 56.8), (21.4, 57.1), (21.6, 57.4), (22.0, 57.6), (22.6, 57.76), (23.0, 57.35),
        (23.5, 57.05), (24.1, 57.03), (24.35, 57.25), (24.4, 57.7), (24.45, 58.05), (24.5, 58.38), (23.7, 58.35),
        (23.5, 58.6), (23.45, 59.0), (23.5, 59.25), (24.75, 59.47), (26.5, 59.55), (28.05, 59.46), (29.1, 59.9),
        (30.2, 59.9), (29.7, 60.2), (28.6, 60.4), (27.2, 60.55), (25.0, 60.15), (23.0, 59.85), (22.2, 60.4),
        (21.4, 60.8), (21.3, 62.0), (25.4, 65.0), (24.15, 65.85), (22.2, 65.6), (20.3, 63.8), (17.6, 62.5),
        (17.2, 60.65), (18.9, 59.9), (18.3, 59.35), (16.8, 58.6), (16.5, 57.2), (16.3, 56.65), (15.9, 56.2),
        (14.7, 56.1), (14.2, 55.4), (13.0, 55.4), (12.9, 55.9), (12.5, 56.3), (11.0, 58.0), (10.0, 59.0),
        (5.0, 59.0), (5.0, 72.0), (60.0, 72.0), (60.0, 45.0), (10.0, 45.0), (10.0, 53.9),
    ],
    # Saaremaa, Hiiumaa, Gotland, Öland, Bornholm
    [(21.8, 58.45), (22.6, 58.62), (23.05, 58.4), (22.5, 58.22), (22.1, 57.92), (21.9, 58.3)],
    [(22.05, 58.95), (22.9, 58.98), (22.75, 58.75), (22.2, 58.8)],
    [(18.15, 57.1), (18.3, 57.6), (18.75, 57.95), (19.1, 57.8), (18.8, 57.4), (18.5, 56.92)],
    [(16.4, 56.2), (16.95, 57.35), (17.1, 57.3), (16.6, 56.2)],
    [(14.7, 55.0), (14.75, 55.3), (15.15, 55.15), (15.1, 55.0)],
]

# The Vistula Lagoon behind its spit, from Elblag to the Pregolya's mouth, and the Curonian
# Lagoon behind the Curonian Spit, from Zelenogradsk to Klaipeda.
VISTULA_LAGOON = [
    (19.3, 54.33), (19.6, 54.44), (19.85, 54.57), (20.05, 54.65), (20.3, 54.69), (20.35, 54.66), (20.15, 54.57),
    (19.95, 54.46), (19.68, 54.36), (19.4, 54.24), (19.3, 54.27),
]
CURONIAN_LAGOON = [
    (20.62, 54.98), (20.85, 55.14), (21.04, 55.32), (21.11, 55.52), (21.13, 55.69), (21.2, 55.69), (21.25, 55.5),
    (21.3, 55.3), (21.2, 55.1), (21.0, 54.97), (20.75, 54.93),
]
SEAS = [VISTULA_LAGOON, CURONIAN_LAGOON]

LAKES = [
    # Masuria: Sniardwy, Mamry
    [(21.6, 53.7), (21.65, 53.8), (21.85, 53.8), (21.85, 53.7), (21.7, 53.67)],
    [(21.72, 54.05), (21.75, 54.15), (21.85, 54.17), (21.83, 54.05)],
    # Wigry by Suwalki, Vistytis on the Kaliningrad border
    [(23.04, 54.0), (23.05, 54.08), (23.15, 54.08), (23.12, 54.0)],
    [(22.67, 54.42), (22.71, 54.47), (22.8, 54.44), (22.75, 54.4)],
    # Narach, Druksiai
    [(26.65, 54.82), (26.7, 54.9), (26.85, 54.9), (26.82, 54.82)],
    [(26.5, 55.58), (26.55, 55.67), (26.7, 55.65), (26.65, 55.57)],
]

RIVERS = [
    # Neman: Stowbtsy, Grodno, Druskininkai, Alytus, the Kaunas reservoir, Kaunas, Jurbarkas,
    # Sovetsk, its delta into the Curonian Lagoon
    [(26.74, 53.48, 0.12), (26.06, 53.75, 0.12), (25.32, 53.65, 0.15), (24.53, 53.41, 0.15), (24.2, 53.5, 0.15),
     (23.83, 53.68, 0.18), (23.95, 53.9, 0.18), (23.97, 54.02, 0.18), (24.08, 54.25, 0.18), (24.05, 54.4, 0.18),
     (24.08, 54.6, 0.2), (24.25, 54.8, 0.7), (24.1, 54.88, 0.8), (23.95, 54.9, 0.2), (23.6, 54.95, 0.2),
     (23.2, 55.05, 0.22), (22.77, 55.08, 0.25), (22.2, 55.07, 0.28), (21.87, 55.08, 0.3), (21.5, 55.2, 0.4),
     (21.28, 55.3, 0.6)],
    # Neris through Vilnius to Kaunas
    [(26.6, 54.95, 0.08), (25.8, 54.9, 0.1), (25.28, 54.69, 0.12), (24.9, 54.75, 0.12), (24.27, 55.08, 0.12),
     (23.93, 54.9, 0.15)],
    # Pregolya through Chernyakhovsk and Kaliningrad to the Vistula Lagoon
    [(22.2, 54.6, 0.07), (21.8, 54.63, 0.08), (21.08, 54.65, 0.1), (20.5, 54.71, 0.12), (20.33, 54.67, 0.15)],
    # Narew past Lomza and Ostroleka, Biebrza
    [(23.52, 52.91, 0.07), (22.95, 52.95, 0.08), (22.66, 53.21, 0.08), (22.38, 53.2, 0.09), (22.07, 53.18, 0.1),
     (21.57, 53.08, 0.1), (21.4, 52.88, 0.1), (21.08, 52.7, 0.1), (21.07, 52.52, 0.12)],
    [(23.45, 53.75, 0.06), (23.05, 53.6, 0.06), (22.7, 53.45, 0.06), (22.4, 53.22, 0.07)],
    # Bug: the Polish-Belarusian border river past Brest
    [(23.8, 51.95, 0.1), (23.65, 52.08, 0.1), (23.35, 52.25, 0.1), (23.0, 52.37, 0.1), (22.66, 52.4, 0.1),
     (22.2, 52.55, 0.1), (21.86, 52.7, 0.1), (21.47, 52.6, 0.1)],
    # Lyna through Olsztyn to the Pregolya
    [(20.45, 53.6, 0.05), (20.48, 53.78, 0.05), (20.55, 54.05, 0.05), (20.8, 54.25, 0.06), (21.0, 54.45, 0.06),
     (21.2, 54.6, 0.07)],
    # Daugava past Daugavpils and Jekabpils to Riga
    [(27.6, 55.8, 0.2), (27.0, 55.85, 0.2), (26.53, 55.87, 0.2), (26.2, 56.2, 0.2), (25.86, 56.5, 0.25),
     (25.2, 56.65, 0.3), (24.6, 56.8, 0.3), (24.1, 56.95, 0.4), (24.03, 57.05, 0.5)],
    # Vistula below the western edge to the Gulf of Gdansk
    [(18.6, 53.0, 0.4), (18.75, 53.48, 0.4), (18.87, 53.73, 0.4), (18.9, 54.0, 0.4), (18.95, 54.33, 0.4)],
]


# ---------------------------------------------------------------------------------------------
# Sides

# The Kaliningrad front, from the Vistula Spit east along the Polish border, the Goldap bulge,
# Vistytis, then north-west along the Lithuanian border and the Neman to the Curonian Spit.
KALININGRAD_FRONT = [
    (19.62, 54.46, 0.2), (19.85, 54.4, 0.2), (20.3, 54.39, 0.2), (20.8, 54.36, 0.25), (21.3, 54.33, 0.3),
    (21.7, 54.32, 0.4), (21.92, 54.3, 0.7), (22.02, 54.2, 0.9), (22.2, 54.12, 1.0), (22.45, 54.12, 1.0),
    (22.7, 54.2, 1.0), (22.85, 54.32, 1.0), (22.95, 54.42, 1.0), (22.9, 54.55, 0.7), (22.82, 54.65, 0.5),
    (22.87, 54.77, 0.4), (22.85, 54.92, 0.3), (22.6, 55.04, 0.3), (22.2, 55.07, 0.3), (21.87, 55.1, 0.3),
    (21.5, 55.2, 0.25), (21.27, 55.27, 0.2), (20.95, 55.28, 0.0),
]
# The Belarusian front, from the Latvian border at the theater's eastern edge down the
# Lithuanian border, the Druskininkai-Lazdijai-Sejny bulge, then the Polish border to Brest.
BELARUS_FRONT = [
    (27.8, 56.02, 0.15), (27.2, 55.85, 0.15), (26.63, 55.68, 0.2), (26.8, 55.4, 0.2), (26.62, 55.15, 0.25),
    (26.2, 54.98, 0.3), (25.82, 54.8, 0.35), (25.72, 54.55, 0.35), (25.5, 54.25, 0.4), (25.0, 54.15, 0.5),
    (24.6, 54.1, 0.7), (24.3, 54.25, 0.9), (23.95, 54.35, 1.0), (23.65, 54.35, 1.0), (23.35, 54.25, 1.0), (23.2, 54.15, 1.0),
    (23.25, 54.0, 0.9), (23.4, 53.85, 0.6), (23.55, 53.7, 0.35), (23.65, 53.5, 0.3), (23.9, 53.2, 0.25),
    (23.88, 52.95, 0.2), (23.9, 52.7, 0.2), (23.6, 52.5, 0.2), (23.18, 52.33, 0.2), (23.45, 52.2, 0.25),
    (23.65, 52.08, 0.3), (23.65, 51.85, 0.2),
]
# One line: the hop from the Curonian Spit across Lithuania to the Latvian border is quiet.
FRONT = KALININGRAD_FRONT + BELARUS_FRONT

# Poland, Lithuania, Latvia and the open sea: one outline round Kaliningrad (its coastal waters
# stay Russian) and Belarus.
BLUE = (
    [(lon, lat) for lon, lat, _ in reversed(KALININGRAD_FRONT)]
    + [(19.5, 54.68), (15.0, 54.68), (15.0, 49.0), (23.65, 49.0)]
    + [(lon, lat) for lon, lat, _ in reversed(BELARUS_FRONT)]
    + [(28.2, 56.15), (27.9, 56.9), (27.4, 57.6), (27.9, 59.0), (28.0, 61.0), (15.0, 61.0), (15.0, 55.0),
       (19.7, 55.1)]
)


# ---------------------------------------------------------------------------------------------
# Places

TOWNS = [
    # Poland
    ("Warsaw", 21.01, 52.23, 2, 4000, 0.0, True),
    ("Elblag", 19.40, 54.16, 2, 1500, 0.0, False),
    ("Olsztyn", 20.48, 53.78, 2, 1800, 0.0, True),
    ("Bialystok", 23.16, 53.13, 2, 2600, 0.0, True),
    ("Elk", 22.36, 53.83, 1, 1100, 0.0, True),
    ("Gizycko", 21.76, 54.04, 1, 900, 0.0, True),
    ("Suwalki", 22.93, 54.10, 1, 1300, 0.1, True),
    ("Augustow", 22.98, 53.84, 1, 1000, 0.05, True),
    ("Lomza", 22.07, 53.18, 1, 1300, 0.0, True),
    ("Siedlce", 22.29, 52.17, 1, 1300, 0.0, False),
    ("Ostroleka", 21.57, 53.09, 1, 1100, 0.0, False),
    ("Ciechanow", 20.62, 52.88, 1, 1000, 0.0, False),
    ("Grajewo", 22.45, 53.65, 0, 800, 0.0, False),
    ("Bartoszyce", 20.81, 54.25, 0, 800, 0.1, False),
    ("Ketrzyn", 21.38, 54.08, 1, 900, 0.0, False),
    ("Mragowo", 21.31, 53.87, 0, 800, 0.0, False),
    ("Olecko", 22.50, 54.03, 0, 800, 0.2, False),
    ("Szczytno", 20.99, 53.56, 0, 800, 0.0, False),
    ("Pisz", 21.81, 53.63, 0, 700, 0.0, False),
    ("Sokolka", 23.50, 53.41, 0, 800, 0.05, False),
    ("Hajnowka", 23.58, 52.74, 0, 700, 0.0, False),
    ("Bielsk Podlaski", 23.19, 52.77, 0, 800, 0.0, False),
    ("Siemiatycze", 22.88, 52.43, 0, 600, 0.0, False),
    ("Mlawa", 20.38, 53.11, 0, 800, 0.0, False),
    ("Ostroda", 19.97, 53.70, 0, 800, 0.0, False),
    ("Ilawa", 19.57, 53.60, 0, 800, 0.0, False),
    ("Braniewo", 19.83, 54.38, 0, 700, 0.1, False),
    ("Lidzbark Warminski", 20.58, 54.13, 0, 700, 0.0, False),
    ("Zambrow", 22.25, 52.98, 0, 700, 0.0, False),
    ("Minsk Mazowiecki", 21.56, 52.18, 1, 900, 0.0, False),
    ("Biala Podlaska", 23.12, 52.03, 1, 1000, 0.0, False),
    ("Pultusk", 21.08, 52.70, 0, 700, 0.0, False),
    ("Wyszkow", 21.47, 52.59, 0, 700, 0.0, False),
    # Lithuania
    ("Vilnius", 25.28, 54.69, 2, 3000, 0.02, True),
    ("Kaunas", 23.90, 54.90, 2, 2600, 0.0, True),
    ("Klaipeda", 21.15, 55.71, 2, 1800, 0.0, True),
    ("Siauliai", 23.32, 55.93, 2, 1700, 0.0, True),
    ("Panevezys", 24.36, 55.73, 2, 1600, 0.0, True),
    ("Alytus", 24.05, 54.40, 1, 1300, 0.0, True),
    ("Marijampole", 23.35, 54.56, 1, 1200, 0.0, True),
    ("Kalvarija", 23.22, 54.41, 0, 500, 0.2, False),
    ("Vilkaviskis", 23.03, 54.65, 0, 700, 0.1, False),
    ("Kybartai", 22.75, 54.64, 0, 600, 0.3, False),
    ("Jurbarkas", 22.77, 55.08, 0, 700, 0.2, False),
    ("Taurage", 22.29, 55.25, 1, 900, 0.1, False),
    ("Silute", 21.48, 55.35, 0, 700, 0.1, False),
    ("Jonava", 24.28, 55.08, 1, 900, 0.0, False),
    ("Kedainiai", 23.97, 55.29, 1, 900, 0.0, False),
    ("Ukmerge", 24.75, 55.25, 1, 900, 0.0, False),
    ("Utena", 25.60, 55.50, 1, 900, 0.0, False),
    ("Visaginas", 26.43, 55.60, 1, 800, 0.0, False),
    ("Ignalina", 26.17, 55.34, 0, 500, 0.0, False),
    ("Svencionys", 26.16, 55.13, 0, 600, 0.05, False),
    ("Salcininkai", 25.38, 54.31, 0, 600, 0.2, False),
    ("Varena", 24.57, 54.22, 0, 700, 0.1, False),
    ("Elektrenai", 24.67, 54.79, 0, 700, 0.0, False),
    ("Trakai", 24.93, 54.64, 0, 600, 0.0, False),
    ("Raseiniai", 23.12, 55.37, 0, 700, 0.0, False),
    ("Telsiai", 22.25, 55.98, 0, 800, 0.0, False),
    ("Palanga", 21.07, 55.92, 0, 700, 0.0, False),
    ("Rokiskis", 25.58, 55.96, 0, 600, 0.0, False),
    # Taken in the incursion
    ("Druskininkai", 23.97, 54.02, 0, 800, 0.4, False),
    ("Lazdijai", 23.52, 54.23, 0, 600, 0.6, True),
    ("Sejny", 23.35, 54.11, 0, 700, 0.6, True),
    ("Vistytis", 22.72, 54.45, 0, 400, 0.7, False),
    ("Goldap", 22.31, 54.31, 0, 800, 0.6, True),
    # Latvia
    ("Daugavpils", 26.53, 55.87, 2, 1800, 0.0, True),
    ("Jekabpils", 25.86, 56.49, 1, 900, 0.0, False),
    ("Bauska", 24.19, 56.41, 0, 600, 0.0, False),
    ("Jelgava", 23.72, 56.65, 1, 1300, 0.0, False),
    ("Riga", 24.11, 56.95, 2, 3000, 0.0, False),
    ("Kraslava", 27.17, 55.90, 0, 600, 0.0, False),
    ("Liepaja", 21.01, 56.51, 1, 1300, 0.0, False),
    # Kaliningrad
    ("Kaliningrad", 20.51, 54.71, 2, 2800, 0.0, True),
    ("Baltiysk", 19.92, 54.65, 1, 900, 0.0, True),
    ("Sovetsk", 21.87, 55.07, 1, 1100, 0.1, True),
    ("Chernyakhovsk", 21.80, 54.63, 1, 1100, 0.0, True),
    ("Gusev", 22.20, 54.59, 1, 900, 0.05, True),
    ("Nesterov", 22.57, 54.63, 0, 600, 0.3, False),
    ("Zelenogradsk", 20.48, 54.95, 0, 700, 0.0, False),
    ("Svetly", 20.15, 54.67, 0, 700, 0.0, False),
    ("Gvardeysk", 21.08, 54.65, 0, 700, 0.0, False),
    ("Bagrationovsk", 20.64, 54.39, 0, 600, 0.15, False),
    ("Pravdinsk", 21.01, 54.44, 0, 500, 0.0, False),
    ("Polessk", 21.10, 54.86, 0, 600, 0.0, False),
    ("Neman", 22.03, 55.04, 0, 600, 0.1, False),
    ("Ozyorsk", 22.04, 54.41, 0, 500, 0.2, False),
    ("Krasnoznamensk", 22.49, 54.95, 0, 500, 0.1, False),
    ("Pionersky", 20.23, 54.95, 0, 600, 0.0, False),
    ("Mamonovo", 19.94, 54.46, 0, 600, 0.15, False),
    # Belarus
    ("Minsk", 27.56, 53.90, 2, 4500, 0.0, True),
    ("Grodno", 23.83, 53.68, 2, 2400, 0.0, True),
    ("Lida", 25.30, 53.89, 1, 1400, 0.0, True),
    ("Shchuchyn", 24.74, 53.60, 0, 800, 0.0, True),
    ("Vawkavysk", 24.47, 53.15, 1, 1000, 0.0, True),
    ("Slonim", 25.32, 53.09, 1, 1000, 0.0, False),
    ("Baranovichi", 26.01, 53.13, 2, 1800, 0.0, True),
    ("Brest", 23.70, 52.10, 2, 2400, 0.0, True),
    ("Maladzyechna", 26.85, 54.31, 1, 1300, 0.0, True),
    ("Astravets", 25.95, 54.61, 0, 700, 0.0, False),
    ("Smarhon", 26.40, 54.48, 1, 900, 0.0, False),
    ("Ashmyany", 25.94, 54.42, 0, 600, 0.0, False),
    ("Navahrudak", 25.83, 53.60, 1, 900, 0.0, False),
    ("Masty", 24.54, 53.41, 0, 700, 0.0, False),
    ("Pastavy", 26.83, 55.12, 0, 700, 0.0, False),
    ("Hlybokaye", 27.69, 55.14, 0, 600, 0.0, False),
    ("Braslaw", 27.04, 55.64, 0, 500, 0.0, False),
    ("Pruzhany", 24.46, 52.56, 0, 700, 0.0, False),
    ("Kobryn", 24.36, 52.21, 1, 1000, 0.0, False),
    ("Ivatsevichy", 25.34, 52.71, 0, 700, 0.0, False),
    ("Stowbtsy", 26.74, 53.48, 0, 600, 0.0, False),
    ("Dzyarzhynsk", 27.19, 53.68, 0, 700, 0.0, False),
    ("Valozhyn", 26.53, 54.09, 0, 600, 0.0, False),
]

AIRBASES = [
    ("Szymany", 20.94, 53.48, True),
    ("Bialystok", 23.17, 53.10, True),
    ("Minsk Mazowiecki", 21.66, 52.20, True),
    ("Modlin", 20.65, 52.45, True),
    ("Biala Podlaska", 23.13, 52.00, True),
    ("Siauliai", 23.40, 55.89, True),
    ("Kaunas", 24.08, 54.96, True),
    ("Vilnius", 25.29, 54.63, True),
    ("Palanga", 21.10, 55.97, True),
    ("Daugavpils", 26.66, 55.94, True),
    ("Chkalovsk", 20.40, 54.77, False),
    ("Donskoye", 19.96, 54.94, False),
    ("Chernyakhovsk", 21.79, 54.60, False),
    ("Lida", 25.32, 53.88, False),
    ("Grodno", 24.05, 53.60, False),
    ("Shchuchyn", 24.75, 53.62, False),
    ("Baranovichi", 26.05, 53.10, False),
    ("Machulishchy", 27.58, 53.77, False),
    ("Brest", 23.90, 52.11, False),
]

# Nuclear power plants: the Belarusian plant at Astravets (VVER-1200), Ignalina (RBMK).
NPPS = [
    ("Astravets NPP", 26.09, 54.76, "novovoronezh"),
    ("Ignalina NPP", 26.55, 55.54, "kursk"),
]

LAUNCH_SITES = [
    ("Chernyakhovsk", 21.95, 54.7),
    ("Lida", 25.1, 53.75),
    ("Baranovichi", 25.8, 53.25),
]

SITES = [
    ("hq", 20.62, 54.64, "Kaliningrad"), ("hq", 23.98, 53.58, "Grodno"), ("hq", 22.25, 54.53, "Gusev"),
    ("hq", 25.45, 53.95, "Lida"),
    ("depot", 21.9, 54.62, "Chernyakhovsk"), ("depot", 24.4, 53.2, "Vawkavysk"), ("depot", 25.2, 53.15, "Slonim"),
    ("depot", 21.0, 54.6, "Gvardeysk"), ("depot", 23.9, 52.2, "Brest"), ("depot", 24.6, 53.45, "Masty"),
    ("tankfarm", 20.4, 54.68, "Kaliningrad"), ("tankfarm", 20.05, 54.73, "Baltiysk"),
    ("tankfarm", 26.1, 53.15, "Baranovichi"), ("tankfarm", 23.72, 53.75, "Grodno"),
    ("armor", 22.35, 54.24, "Goldap"), ("armor", 22.86, 54.47, "Vistytis"), ("armor", 23.3, 54.13, "Sejny"),
    ("armor", 23.55, 54.2, "Lazdijai"), ("armor", 24.0, 54.1, "Druskininkai"), ("armor", 22.6, 54.58, "Nesterov"),
    ("armor", 24.85, 54.0, "Varena"),
    ("ss", 21.7, 54.55, "Chernyakhovsk"), ("ss", 20.05, 54.78, "Baltiysk"), ("ss", 26.0, 53.25, "Baranovichi"),
    ("oth", 20.23, 54.93, "Pionersky"),
    ("bridge", 21.87, 55.09, "Sovetsk"), ("bridge", 23.83, 53.68, "Grodno"), ("bridge", 20.5, 54.71, "Kaliningrad"),
    ("bridge", 25.32, 53.65, "Lida"),
    ("msam", 20.3, 54.85, "Kaliningrad"), ("msam", 21.5, 54.5, "Chernyakhovsk"), ("msam", 22.55, 54.25, "Goldap"),
    ("msam", 23.4, 54.03, "Sejny"), ("msam", 23.75, 53.55, "Grodno"), ("msam", 25.2, 53.75, "Lida"),
    ("msam", 23.75, 52.25, "Brest"),
    ("pow", 21.3, 54.7, "Gvardeysk"), ("pow", 25.6, 53.5, "Navahrudak"),
]

# Patrol boats: (lon, lat), off Baltiysk and in the lagoons.
BOATS = [(19.75, 54.75), (19.85, 55.05), (20.9, 55.12), (19.9, 54.55), (19.4, 55.0)]

FORESTS = [
    # Poland
    (23.2, 53.85, 15, 0.75), (23.85, 52.72, 15, 0.8), (21.7, 53.55, 18, 0.7), (20.6, 53.5, 15, 0.6),
    (23.4, 53.3, 12, 0.65), (21.4, 53.3, 20, 0.6), (20.8, 53.85, 15, 0.5),
    # The Rominta heath on the Kaliningrad border, Kaliningrad's north
    (22.4, 54.35, 10, 0.7), (21.6, 54.95, 12, 0.5), (21.0, 55.4, 5, 0.6),
    # Lithuania
    (24.3, 54.1, 20, 0.75), (25.0, 54.4, 12, 0.6), (25.6, 55.2, 15, 0.6), (24.5, 55.6, 15, 0.45),
    (22.6, 55.5, 15, 0.5),
    # Belarus
    (26.3, 53.85, 20, 0.7), (25.5, 52.5, 25, 0.6), (26.8, 53.2, 20, 0.5), (24.4, 53.9, 15, 0.55),
    # Latvia
    (25.0, 56.2, 20, 0.5), (26.5, 56.3, 20, 0.55),
]


# ---------------------------------------------------------------------------------------------
# Palette


def palette(root):
    """Central Europe's palettes, the ground ramp a touch lusher."""
    src = open(os.path.join(root, "assets", "palettes", "tod_central_europe.ron")).read()
    colour = re.compile(r"\((\d+), (\d+), (\d+)\)")
    starts = [src.index(k) for k in ("day:", "dusk:", "night:")] + [len(src)]
    blocks = [src[starts[k]:starts[k + 1]] for k in range(3)]

    def fix(block):
        cols = [tuple(int(v) for v in c) for c in colour.findall(block)]
        out = list(cols)
        for i in range(0x70, 0x80):
            r, g, b = cols[i]
            out[i] = (round(r * 0.95), g, b)
        it = iter(out)
        return colour.sub(lambda m: "(%d, %d, %d)" % next(it), block)

    text = src[:starts[0]] + "".join(fix(b) for b in blocks)
    open(os.path.join(root, "assets", "palettes", "tod_suwalki.ron"), "w").write(text)
