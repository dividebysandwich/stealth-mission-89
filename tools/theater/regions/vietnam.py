"""Geography of the Vietnam theater (1993): Indochina from Bangkok to the Chinese border, with
the Gulf of Tonkin, the Gulf of Thailand and the edge of the South China Sea.

The theater squeezes 97.3-111.6 E, 8.6-22.4 N (about 1540 by 1540 km) into the 512 km of a
theater. Coordinates here are (lon, lat) degrees. The original theater's map was turned by
roughly 45 degrees to fit the long coast into the square; a north-up projection cannot follow
that, so the window is the one that holds all of the theater's places: Bangkok and Takhli in
the west, Hanoi, Kep and the Chinese border in the north, Saigon and the Mekong delta in the
south.

Sides: Thailand (with Myanmar and Malaysia beyond it) and the open sea, where the US carriers
sail, are the friendly side; Vietnam, Laos and Cambodia are the enemy, from the old Vietnam
War targets round Hanoi and Haiphong (Phuc Yen, Kep, Gia Lam, the Thanh Hoa bridge) down the
Ho Chi Minh Trail to Saigon and Phnom Penh. China and Hainan are not friendly ground (their
towns are neutral). The front is Thailand's border with Laos (mostly the Mekong) and Cambodia:
quiet along the river, border fighting along the Cambodian border and at Ban Rom Klao.
"""

PROJECTION = dict(lon0=97.3, lat0=8.6, lat_ref=15.5, squeeze=3.0)

LIBRARY = "vietnam"
LAND_RUBBLE = 73

DATE = (5, 2, 1993)
GRID = ("XV", 0, 0)
SQUADRONS = 4

ORDER_OF_BATTLE = [
    ("an72", "none", "special"),
    ("yak38", "aa2", "base"),
    ("yak38", "aa2", "patrol"),
    ("mig23", "aa6t", "escort"),
    ("yak38", "aa2", "patrol"),
    ("mig23", "aa6t", "base"),
    ("mig23", "aa6", "base"),
    ("yak38", "aa2", "escort"),
    ("f4e_blue", "none", "friendly"),
    ("f4e_blue", "none", "friendly"),
    ("f4e_blue", "none", "friendly"),
    ("f4e_blue", "none", "friendly"),
]

# Radar types (units.ron ids): the fixed SAM sites (SA-2 and SA-5), the mobile ones and the
# patrol boats (the theater's missile boats carry SA-N-5 launchers).
SAM_RADARS = ["sa2", "sa5"]
MOBILE_SAM_RADARS = ["sa8", "sa13"]
BOAT_RADARS = ["san5"]
SLAG_HEAP_TOWNS = set()

# Models this theater builds besides the water, forests and mountains (see models.py).
EXTRA_MODELS = []

# ---------------------------------------------------------------------------------------------
# Land and water: everything outside `LANDS` is sea.

LANDS = [
    # The mainland: the China coast from far in the north-east past the Leizhou peninsula and
    # the Gulf of Tonkin, the whole coast of Vietnam round Cape Ca Mau, Cambodia and the Gulf
    # of Thailand, round the Malay peninsula and up the Andaman coast of Myanmar.
    [
        # China, coarse far away; Zhanjiang and the Leizhou peninsula
        (125.0, 35.0), (121.9, 30.9), (119.5, 26.0), (117.0, 23.6), (114.2, 22.3), (113.5, 22.2),
        (112.5, 21.8), (111.7, 21.6), (111.0, 21.45), (110.65, 21.2), (110.5, 20.9), (110.55, 20.55),
        (110.5, 20.3), (110.3, 20.22), (110.1, 20.25), (109.92, 20.4), (109.7, 20.6), (109.8, 20.95),
        (109.75, 21.25), (109.75, 21.45),
        # Tieshan bay, Beihai, Qinzhou bay, Fangchenggang
        (109.55, 21.55), (109.1, 21.45), (108.85, 21.6), (108.6, 21.85), (108.35, 21.6), (108.1, 21.55),
        # North Vietnam: Mong Cai, Ha Long bay, Haiphong, the Red River delta
        (107.97, 21.53), (107.75, 21.4), (107.6, 21.3), (107.4, 21.12), (107.3, 21.02), (107.08, 20.95),
        (106.85, 20.85), (106.8, 20.75), (106.79, 20.70), (106.6, 20.5), (106.58, 20.25), (106.35, 20.05),
        (106.1, 19.95), (105.97, 19.88),
        # Thanh Hoa, Vinh, Ha Tinh, Dong Hoi, the DMZ, Hue
        (105.9, 19.75), (105.8, 19.45), (105.82, 19.32), (105.72, 19.05), (105.73, 18.82), (105.77, 18.77),
        (105.9, 18.5), (106.1, 18.3), (106.35, 18.08), (106.5, 17.92), (106.47, 17.72), (106.66, 17.47),
        (106.85, 17.2), (107.1, 17.0), (107.2, 16.9), (107.4, 16.75), (107.6, 16.58), (107.9, 16.35),
        (108.05, 16.25),
        # Hai Van pass, Da Nang bay and the Son Tra peninsula, Hoi An, Chu Lai, Quang Ngai
        (108.2, 16.2), (108.13, 16.12), (108.22, 16.08), (108.27, 16.10), (108.33, 16.13), (108.31, 16.07),
        (108.25, 16.04), (108.35, 15.85), (108.55, 15.6), (108.72, 15.4), (108.8, 15.38), (108.9, 15.2),
        (108.9, 15.0), (108.95, 14.7), (109.05, 14.4), (109.1, 14.1),
        # Quy Nhon, Tuy Hoa, Cape Varella, Nha Trang, Cam Ranh, Phan Rang, Phan Thiet
        (109.22, 13.8), (109.3, 13.55), (109.3, 13.2), (109.32, 13.07), (109.45, 12.88), (109.35, 12.6),
        (109.35, 12.5), (109.22, 12.25), (109.22, 12.0), (109.2, 11.85), (109.12, 11.65), (109.0, 11.57),
        (109.02, 11.35), (108.8, 11.25), (108.55, 11.17), (108.3, 10.95), (108.1, 10.92), (107.98, 10.7),
        (107.7, 10.6), (107.4, 10.5),
        # Vung Tau, the Saigon river mouth, the Mekong delta, Cape Ca Mau, Rach Gia, Ha Tien
        (107.08, 10.35), (106.95, 10.4), (106.8, 10.3), (106.75, 10.25), (106.7, 10.05), (106.6, 9.85),
        (106.45, 9.65), (106.25, 9.5), (106.0, 9.35), (105.75, 9.2), (105.4, 9.0), (105.2, 8.75),
        (104.75, 8.6), (104.8, 8.95), (104.85, 9.35), (104.95, 9.7), (105.08, 10.0), (104.85, 10.2),
        (104.48, 10.38),
        # Cambodia: Kep, Kampot, Ream, Kampong Som, the Koh Kong coast
        (104.3, 10.48), (104.15, 10.55), (103.85, 10.55), (103.65, 10.5), (103.48, 10.63), (103.55, 10.78),
        (103.62, 10.95), (103.35, 11.0), (103.1, 11.2), (102.95, 11.45), (102.98, 11.62), (102.9, 11.72),
        # Thailand: Trat, Chanthaburi, Rayong, Sattahip, Chonburi, the head of the gulf, Hua Hin
        (102.6, 12.0), (102.5, 12.2), (102.3, 12.25), (102.15, 12.45), (102.0, 12.55), (101.7, 12.65),
        (101.4, 12.6), (101.25, 12.65), (101.0, 12.62), (100.92, 12.65), (100.88, 12.9), (100.92, 13.15),
        (100.97, 13.38), (100.85, 13.5), (100.6, 13.52), (100.3, 13.48), (100.05, 13.38), (100.08, 13.05),
        (99.98, 12.8), (99.97, 12.55), (99.95, 12.3), (99.85, 12.0), (99.8, 11.8), (99.6, 11.4),
        (99.52, 11.2), (99.3, 10.8), (99.25, 10.45), (99.2, 10.0), (99.35, 9.4), (99.4, 9.2), (99.8, 9.3),
        (99.88, 9.2), (100.0, 8.9), (100.3, 8.5), (100.25, 8.3), (100.4, 7.8), (100.6, 7.2),
        # The Malay peninsula, coarse
        (101.3, 6.9), (102.2, 6.2), (103.2, 5.0), (103.4, 3.8), (104.2, 1.45), (103.8, 1.25), (101.3, 2.8),
        (100.4, 4.2), (100.3, 5.4), (100.1, 6.4), (99.8, 6.7), (99.4, 7.4), (98.9, 8.05),
        # Phang Nga, Khao Lak, Ranong and the Myanmar coast to the Gulf of Martaban
        (98.5, 8.3), (98.3, 8.2), (98.25, 8.65), (98.3, 9.0), (98.4, 9.4), (98.6, 9.95), (98.7, 10.5),
        (98.55, 11.0), (98.6, 11.7), (98.6, 12.45), (98.45, 13.0), (98.2, 13.6), (98.15, 14.1),
        (97.95, 14.7), (97.8, 15.3), (97.65, 15.9), (97.6, 16.45), (97.3, 16.9), (96.8, 16.8),
        (96.3, 16.6), (95.3, 15.8), (94.3, 16.0), (94.4, 19.5), (92.3, 20.8), (90.0, 22.0), (90.0, 35.0),
    ],
    # Hainan.
    [(110.33, 20.07), (110.7, 20.15), (111.0, 19.95), (111.05, 19.6), (110.65, 19.1), (110.5, 18.75),
     (110.0, 18.4), (109.5, 18.2), (108.9, 18.4), (108.65, 18.75), (108.65, 19.2), (109.2, 19.8),
     (109.6, 19.95), (110.1, 20.05)],
    # Cat Ba in Ha Long bay.
    [(106.93, 20.80), (107.05, 20.85), (107.13, 20.77), (107.05, 20.70), (106.95, 20.72)],
    # Phu Quoc, Koh Rong, Koh Chang, Koh Samui, Con Son, Phuket.
    [(103.85, 10.45), (104.05, 10.42), (104.08, 10.1), (104.02, 9.95), (103.95, 10.05), (103.83, 10.3)],
    [(103.18, 10.80), (103.28, 10.78), (103.25, 10.65), (103.17, 10.70)],
    [(102.30, 12.18), (102.38, 12.15), (102.38, 11.98), (102.30, 12.02)],
    [(99.93, 9.45), (100.08, 9.42), (100.10, 9.55), (99.98, 9.58)],
    [(106.55, 8.70), (106.65, 8.75), (106.68, 8.65), (106.58, 8.62)],
    [(98.28, 8.20), (98.42, 8.10), (98.40, 7.80), (98.30, 7.75)],
]

# Lakes, lagoons and reservoirs.
LAKES = [
    # Tonle Sap (dry-season shore), the Tam Giang lagoon by Hue
    [(103.75, 13.25), (104.05, 13.15), (104.35, 12.9), (104.45, 12.68), (104.3, 12.62), (104.0, 12.85),
     (103.7, 13.0), (103.6, 13.15)],
    [(107.35, 16.75), (107.55, 16.63), (107.75, 16.50), (107.80, 16.45), (107.72, 16.47), (107.50, 16.58),
     (107.33, 16.72)],
    # Nam Ngum (Laos), Ubolratana and Sirindhorn (Thailand)
    [(102.52, 18.62), (102.62, 18.68), (102.72, 18.62), (102.65, 18.52), (102.56, 18.55)],
    [(102.50, 16.82), (102.62, 16.85), (102.72, 16.72), (102.62, 16.68), (102.52, 16.75)],
    [(105.42, 15.22), (105.50, 15.20), (105.48, 15.08), (105.40, 15.12)],
]

# Rivers: polylines of (lon, lat, width in theater km).
RIVERS = [
    # Mekong from the Golden Triangle past Luang Prabang, Vientiane, Thakhek, Savannakhet and
    # Pakse, through Cambodia by Stung Treng, Kratie and Phnom Penh to its delta
    [(100.08, 20.36, 0.15), (100.42, 20.25, 0.15), (100.9, 20.1, 0.15), (101.5, 19.95, 0.15),
     (102.13, 19.89, 0.16), (101.9, 19.4, 0.16), (101.6, 18.9, 0.16), (101.4, 18.4, 0.17), (101.5, 17.85, 0.17),
     (101.67, 17.9, 0.18), (101.9, 18.05, 0.18), (102.35, 18.05, 0.2), (102.5, 17.93, 0.2), (102.62, 17.94, 0.2),
     (102.75, 17.89, 0.2), (103.25, 18.05, 0.2), (103.65, 18.38, 0.2), (104.0, 18.35, 0.2), (104.4, 17.9, 0.2),
     (104.8, 17.4, 0.22), (104.8, 16.9, 0.22), (104.73, 16.55, 0.22), (105.05, 16.1, 0.22), (105.2, 16.05, 0.22),
     (105.5, 15.32, 0.25), (105.8, 15.12, 0.25), (105.88, 14.6, 0.25), (105.85, 14.1, 0.3), (105.97, 13.53, 0.3),
     (106.02, 12.49, 0.3), (105.75, 12.2, 0.3), (105.46, 12.0, 0.3), (104.93, 11.57, 0.3), (105.1, 11.2, 0.3),
     (105.15, 10.8, 0.3), (105.6, 10.5, 0.3), (106.0, 10.38, 0.35), (106.36, 10.33, 0.4), (106.75, 10.08, 0.6)],
    # The Bassac from Phnom Penh past Chau Doc, Long Xuyen and Can Tho
    [(104.93, 11.57, 0.2), (105.0, 11.2, 0.2), (105.12, 10.7, 0.22), (105.43, 10.38, 0.25), (105.78, 10.03, 0.3),
     (106.1, 9.7, 0.4), (106.28, 9.52, 0.6)],
    # The Tonle Sap river from the lake to Phnom Penh
    [(104.45, 12.68, 0.1), (104.67, 12.25, 0.1), (104.75, 11.95, 0.12), (104.93, 11.6, 0.12)],
    # Red River from Lao Cai past Yen Bai, Viet Tri and Hanoi to the sea at Ba Lat
    [(103.97, 22.5, 0.08), (104.4, 22.1, 0.09), (104.87, 21.72, 0.1), (105.2, 21.4, 0.12), (105.43, 21.29, 0.14),
     (105.5, 21.15, 0.15), (105.7, 21.12, 0.15), (105.86, 21.04, 0.15), (105.95, 20.85, 0.15), (106.05, 20.65, 0.15),
     (106.2, 20.4, 0.15), (106.4, 20.3, 0.18), (106.6, 20.24, 0.3)],
    # Black River (Song Da) from Lai Chau past Hoa Binh to the Red River
    [(103.15, 22.07, 0.06), (103.6, 21.6, 0.06), (104.2, 21.2, 0.07), (104.8, 20.95, 0.08), (105.33, 20.82, 0.08),
     (105.37, 21.0, 0.08), (105.37, 21.25, 0.09), (105.43, 21.29, 0.09)],
    # Lo (Clear) River from Ha Giang past Tuyen Quang to Viet Tri
    [(104.98, 22.42, 0.05), (105.1, 22.1, 0.05), (105.22, 21.82, 0.06), (105.35, 21.5, 0.06), (105.43, 21.31, 0.07)],
    # The Duong from Hanoi to Pha Lai, the Thai Binh past Hai Duong to Haiphong and the sea
    [(105.88, 21.07, 0.08), (106.15, 21.1, 0.08), (106.3, 21.1, 0.1)],
    [(106.3, 21.1, 0.1), (106.33, 20.95, 0.1), (106.45, 20.88, 0.1), (106.6, 20.88, 0.12), (106.72, 20.88, 0.15),
     (106.85, 20.84, 0.3)],
    # Ma River past the Thanh Hoa (Ham Rong) bridge
    [(104.5, 20.6, 0.05), (105.0, 20.35, 0.06), (105.5, 20.2, 0.07), (105.7, 19.95, 0.08), (105.8, 19.86, 0.08),
     (105.93, 19.79, 0.12)],
    # Ca River past Do Luong to the sea at Cua Hoi by Vinh
    [(104.2, 19.3, 0.05), (104.88, 19.05, 0.06), (105.3, 18.9, 0.07), (105.6, 18.72, 0.08), (105.77, 18.77, 0.12)],
    # Ben Hai (the DMZ) and the Gianh
    [(106.75, 17.0, 0.04), (106.95, 17.0, 0.04), (107.1, 17.0, 0.06)],
    [(106.1, 17.8, 0.04), (106.3, 17.75, 0.05), (106.47, 17.72, 0.06)],
    # Xe Kong from Attapeu to the Mekong at Stung Treng
    [(107.0, 15.3, 0.05), (106.83, 14.81, 0.06), (106.5, 14.3, 0.06), (106.2, 13.8, 0.07), (105.97, 13.53, 0.07)],
    # Dong Nai past Bien Hoa and the Saigon river to the sea at Can Gio
    [(107.5, 11.6, 0.08), (107.0, 11.12, 0.09), (106.82, 10.95, 0.1), (106.78, 10.8, 0.12), (106.75, 10.6, 0.15),
     (106.85, 10.45, 0.3)],
    [(106.35, 11.3, 0.06), (106.6, 11.0, 0.07), (106.72, 10.79, 0.08)],
    # Chao Phraya from Nakhon Sawan past Ayutthaya and Bangkok, with the Ping and the Nan
    [(100.12, 15.7, 0.1), (100.12, 15.18, 0.1), (100.4, 14.8, 0.1), (100.57, 14.35, 0.1), (100.5, 14.0, 0.12),
     (100.5, 13.75, 0.12), (100.58, 13.53, 0.15)],
    [(99.0, 17.3, 0.06), (99.52, 16.48, 0.07), (99.9, 15.9, 0.08), (100.12, 15.7, 0.08)],
    [(100.6, 18.0, 0.05), (100.26, 16.82, 0.06), (100.2, 16.0, 0.07), (100.12, 15.7, 0.07)],
    # Mun river from Korat past Ubon to the Mekong
    [(102.3, 14.95, 0.04), (103.0, 15.1, 0.05), (103.5, 15.15, 0.05), (104.3, 15.15, 0.06), (104.85, 15.21, 0.06),
     (105.5, 15.32, 0.07)],
]

# ---------------------------------------------------------------------------------------------
# Sides

# Thailand's border with Laos and Cambodia, from the Golden Triangle down the Mekong, overland
# past Ban Rom Klao, down the Mekong again from Chiang Khan to Khong Chiam, along the Dangrek
# escarpment and the Cambodian border to the coast at Hat Lek: (lon, lat, intensity).
FRONT = [
    (100.08, 20.36, 0.0), (100.42, 20.25, 0.0), (100.55, 19.75, 0.05), (100.9, 19.6, 0.05), (101.25, 19.55, 0.1),
    (101.2, 19.0, 0.15), (101.3, 18.4, 0.3), (101.1, 17.95, 0.4), (101.5, 17.85, 0.2), (101.67, 17.9, 0.05),
    (101.9, 18.05, 0.0), (102.35, 18.05, 0.0), (102.5, 17.93, 0.0), (102.75, 17.89, 0.0), (103.25, 18.05, 0.0),
    (103.65, 18.38, 0.0), (104.0, 18.35, 0.0), (104.4, 17.9, 0.0), (104.8, 17.4, 0.0), (104.8, 16.9, 0.0),
    (104.73, 16.55, 0.0), (105.05, 16.1, 0.0), (105.2, 16.05, 0.0), (105.5, 15.32, 0.05), (105.55, 14.8, 0.1),
    (105.2, 14.35, 0.2), (104.5, 14.35, 0.3), (103.6, 14.4, 0.35), (103.0, 14.35, 0.4), (102.55, 14.0, 0.5),
    (102.5, 13.68, 0.5), (102.35, 13.2, 0.45), (102.6, 12.75, 0.4), (102.55, 12.3, 0.3), (102.9, 11.72, 0.2),
]

# The friendly side: Thailand, Myanmar and Malaysia west of the border, and the open sea
# beyond the coastal waters of Cambodia and Vietnam (round the south of Hainan).
BLUE = (
    [(lon, lat) for lon, lat, _ in FRONT]
    + [(102.75, 11.3), (102.6, 11.0), (103.2, 9.4), (104.5, 7.8), (106.0, 7.8), (107.3, 9.0), (108.3, 10.0),
       (109.4, 10.9), (109.75, 11.8), (109.85, 12.5), (109.8, 13.4), (109.6, 14.5), (109.3, 15.3), (109.0, 15.8),
       (108.95, 16.9), (108.7, 17.6), (108.4, 17.9), (109.5, 17.6), (112.5, 17.5), (125.0, 17.5), (125.0, 0.0),
       (90.0, 0.0), (90.0, 24.5), (98.0, 24.5), (99.2, 22.1), (99.9, 22.05), (100.2, 21.45)]
)

# Neutral places: the player never flies from or to them, they are no one's targets.
NEUTRAL = {"Beihai", "Zhanjiang", "Haikou", "Sanya"}

# ---------------------------------------------------------------------------------------------
# Places

# Towns: (name, lon, lat, kind 0 village / 1 town / 2 city, radius in theater metres, ruined
# share, in the world name table).
TOWNS = [
    # North Vietnam: Hanoi, the Red River delta, Haiphong and the coast to the DMZ
    ("Hanoi", 105.84, 21.02, 2, 3000, 0.0, True),
    ("Haiphong", 106.66, 20.85, 2, 2200, 0.0, True),
    ("Nam Dinh", 106.17, 20.43, 2, 1500, 0.0, True),
    ("Hai Duong", 106.31, 20.93, 1, 1000, 0.0, False),
    ("Ninh Binh", 105.97, 20.25, 1, 800, 0.0, False),
    ("Phuc Yen", 105.70, 21.24, 1, 800, 0.0, True),
    ("Bac Giang", 106.20, 21.27, 1, 900, 0.0, False),
    ("Kep", 106.27, 21.42, 0, 600, 0.0, True),
    ("Thai Nguyen", 105.84, 21.59, 2, 1500, 0.0, True),
    ("Viet Tri", 105.40, 21.33, 1, 1100, 0.0, False),
    ("Yen Bai", 104.87, 21.70, 1, 900, 0.0, False),
    ("Lang Son", 106.76, 21.85, 1, 1000, 0.0, True),
    ("Hoa Binh", 105.34, 20.81, 1, 800, 0.0, False),
    ("Son La", 103.91, 21.33, 0, 700, 0.0, False),
    ("Dien Bien Phu", 103.02, 21.39, 0, 700, 0.0, False),
    ("Hon Gai", 107.08, 20.96, 1, 1000, 0.0, False),
    ("Thanh Hoa", 105.78, 19.81, 1, 1400, 0.0, True),
    ("Vinh", 105.68, 18.68, 2, 1500, 0.0, True),
    ("Ha Tinh", 105.90, 18.34, 1, 800, 0.0, False),
    ("Dong Hoi", 106.60, 17.47, 1, 1000, 0.0, True),
    # Central and South Vietnam
    ("Quang Tri", 107.19, 16.75, 0, 700, 0.0, False),
    ("Khe Sanh", 106.73, 16.63, 0, 600, 0.0, False),
    ("Hue", 107.58, 16.46, 2, 1800, 0.0, True),
    ("Da Nang", 108.20, 16.07, 2, 2200, 0.0, True),
    ("Quang Ngai", 108.80, 15.12, 1, 900, 0.0, False),
    ("Kontum", 108.00, 14.35, 1, 800, 0.0, False),
    ("Pleiku", 108.00, 13.98, 1, 1100, 0.0, True),
    ("Qui Nhon", 109.20, 13.77, 1, 1200, 0.0, False),
    ("Ban Me Thuot", 108.05, 12.67, 1, 1100, 0.0, False),
    ("Nha Trang", 109.19, 12.24, 2, 1600, 0.0, True),
    ("Cam Ranh", 109.15, 11.92, 0, 700, 0.0, False),
    ("Da Lat", 108.44, 11.94, 1, 1100, 0.0, False),
    ("Phan Thiet", 108.10, 10.93, 1, 900, 0.0, False),
    ("Bien Hoa", 106.83, 10.95, 1, 1200, 0.0, False),
    ("Ho Chi Minh City", 106.68, 10.78, 2, 3400, 0.0, True),
    ("Vung Tau", 107.08, 10.37, 1, 900, 0.0, False),
    ("My Tho", 106.36, 10.36, 1, 900, 0.0, False),
    ("Can Tho", 105.78, 10.04, 2, 1500, 0.0, False),
    ("Long Xuyen", 105.43, 10.39, 1, 900, 0.0, False),
    ("Rach Gia", 105.09, 10.01, 1, 900, 0.0, False),
    ("Ca Mau", 105.15, 9.18, 1, 900, 0.0, False),
    # Laos
    ("Vientiane", 102.62, 17.97, 2, 2000, 0.0, True),
    ("Luang Prabang", 102.14, 19.89, 1, 900, 0.0, True),
    ("Phonsavan", 103.22, 19.45, 0, 700, 0.0, False),
    ("Sam Neua", 104.04, 20.42, 0, 600, 0.0, False),
    ("Thakhek", 104.83, 17.40, 1, 800, 0.0, False),
    ("Savannakhet", 104.75, 16.55, 1, 1000, 0.0, True),
    ("Tchepone", 106.23, 16.68, 0, 600, 0.0, False),
    ("Saravane", 106.42, 15.72, 0, 600, 0.0, False),
    ("Pakse", 105.84, 15.14, 1, 900, 0.0, False),
    ("Attapeu", 106.87, 14.80, 0, 600, 0.0, False),
    # Cambodia
    ("Phnom Penh", 104.91, 11.56, 2, 2800, 0.0, True),
    ("Takeo", 104.79, 10.99, 1, 900, 0.0, True),
    ("Kampong Cham", 105.48, 11.95, 1, 900, 0.0, False),
    ("Kratie", 106.05, 12.48, 0, 700, 0.0, False),
    ("Stung Treng", 106.01, 13.52, 0, 700, 0.0, False),
    ("Ban Khea", 106.40, 14.10, 0, 600, 0.0, True),
    ("Ban Phlak", 106.27, 12.90, 0, 600, 0.0, True),
    ("Battambang", 103.20, 13.10, 1, 1200, 0.0, False),
    ("Siem Reap", 103.86, 13.36, 1, 900, 0.0, False),
    ("Kampot", 104.18, 10.61, 0, 700, 0.0, False),
    ("Kampong Som", 103.52, 10.63, 1, 800, 0.0, False),
    # Thailand
    ("Bangkok", 100.52, 13.75, 2, 3500, 0.0, True),
    ("Ayutthaya", 100.60, 14.36, 1, 900, 0.0, False),
    ("Takhli", 100.35, 15.26, 0, 700, 0.0, True),
    ("Nakhon Sawan", 100.16, 15.68, 1, 1100, 0.0, False),
    ("Phitsanulok", 100.29, 16.82, 1, 1000, 0.0, False),
    ("Chiang Mai", 98.99, 18.79, 2, 1600, 0.0, False),
    ("Korat", 102.10, 14.97, 2, 1500, 0.0, True),
    ("Khon Kaen", 102.83, 16.43, 1, 1200, 0.0, False),
    ("Udorn", 102.79, 17.41, 1, 1200, 0.0, True),
    ("Nong Khai", 102.74, 17.87, 0, 700, 0.0, False),
    ("Nakhon Phanom", 104.77, 17.40, 0, 700, 0.0, False),
    ("Ubon", 104.85, 15.24, 1, 1100, 0.0, False),
    ("Chanthaburi", 102.10, 12.61, 1, 800, 0.0, False),
    ("Pattaya", 100.92, 12.93, 1, 900, 0.0, False),
    # China (neutral)
    ("Beihai", 109.12, 21.48, 1, 1000, 0.0, False),
    ("Zhanjiang", 110.36, 21.27, 2, 1600, 0.0, False),
    ("Haikou", 110.33, 20.03, 2, 1500, 0.0, False),
    ("Sanya", 109.51, 18.25, 1, 900, 0.0, False),
]

# Airbases: (name, lon, lat, friendly). The runway runs north-south.
AIRBASES = [
    # North Vietnam
    ("Gia Lam", 105.886, 21.041, False),
    ("Phuc Yen", 105.807, 21.221, False),
    ("Kep", 106.253, 21.390, False),
    ("Haiphong", 106.725, 20.819, False),   # Cat Bi
    ("Vinh", 105.671, 18.737, False),
    ("Dong Hoi", 106.590, 17.515, False),
    # The south, Laos and Cambodia
    ("Da Nang", 108.199, 16.044, False),
    ("Nha Trang", 109.192, 12.227, False),
    ("Ho Chi Minh City", 106.652, 10.819, False),   # Tan Son Nhut
    ("Vientiane", 102.563, 17.988, False),   # Wattay
    ("Phnom Penh", 104.844, 11.547, False),   # Pochentong
    # Thailand
    ("Udorn", 102.788, 17.386, True),
    ("Korat", 102.079, 14.934, True),
    ("Takhli", 100.296, 15.277, True),
    ("Don Muang", 100.607, 13.913, True),
    ("Bangkok", 101.005, 12.680, True),   # U-Tapao (Don Muang is Bangkok's own field)
]

# The friendly carrier group in the South China Sea, east of the DMZ.
CARRIER = [("USS Constellation", 110.20, 17.00), ("USS Kitty Hawk", 110.02, 14.12)]

# The theater's own layouts: the Ho Chi Minh Trail's supply dumps, a port with its fuel and
# warehouses.
TEMPLATES = {
    "trail_dump": [(0, 0, "supply_dump"), (260, 160, "supply_dump"), (-220, 260, "supply_dump"),
                   (320, -240, "supply_dump"), (-300, -220, "depot"), (-600, 450, "supply_dump")],
    "port": [(-300, -220, "tank_farm"), (-300, 220, "tank_farm"), (150, 0, "depot"), (450, 0, "depot"),
             (300, 420, "hangar"), (300, -420, "hangar")],
}

# Enemy target sites: (template, lon, lat, name).
SITES = [
    # Headquarters: Hanoi, the panhandle, Laos and the army in Cambodia
    ("hq", 105.80, 21.06, "Hanoi"), ("hq", 105.62, 18.70, "Vinh"), ("hq", 106.52, 17.40, "Dong Hoi"),
    ("hq", 107.52, 16.42, "Hue"), ("hq", 106.60, 10.88, "Ho Chi Minh City"), ("hq", 104.23, 20.42, "Vieng Xai"),
    ("hq", 102.68, 18.05, "Vientiane"), ("hq", 102.45, 18.30, "Viangchan"), ("hq", 104.84, 11.62, "Phnom Penh"),
    ("hq", 104.74, 11.04, "Takeo"), ("hq", 106.30, 12.94, "Ban Phlak"), ("hq", 106.36, 14.14, "Ban Khea"),
    ("hq", 103.15, 13.15, "Battambang"),
    # SS missiles with the headquarters of the west
    ("ss", 104.68, 10.94, "Takeo"), ("ss", 104.78, 11.70, "Phnom Penh"), ("ss", 106.20, 12.85, "Ban Phlak"),
    ("ss", 106.45, 14.20, "Ban Khea"), ("ss", 103.25, 13.22, "Battambang"), ("ss", 102.75, 18.12, "Vientiane"),
    ("ss", 102.38, 18.38, "Viangchan"),
    # Depots: the rail lines from China, the delta, the panhandle, the south
    ("depot", 104.90, 21.66, "Yen Bai"), ("depot", 105.80, 21.63, "Thai Nguyen"), ("depot", 105.45, 21.36, "Viet Tri"),
    ("depot", 106.72, 21.82, "Lang Son"), ("depot", 106.12, 20.40, "Nam Dinh"), ("depot", 105.95, 20.30, "Ninh Binh"),
    ("depot", 105.85, 18.38, "Ha Tinh"), ("depot", 107.12, 16.72, "Quang Tri"), ("depot", 108.12, 16.00, "Da Nang"),
    ("depot", 106.88, 10.98, "Bien Hoa"), ("depot", 105.42, 12.05, "Kampong Cham"),
    # Fuel: Haiphong, Hanoi (Duc Giang), Bac Giang, Nam Dinh, Vinh, Saigon (Nha Be)
    ("tankfarm", 106.62, 20.90, "Haiphong"), ("tankfarm", 105.93, 21.08, "Hanoi"), ("tankfarm", 106.22, 21.30, "Bac Giang"),
    ("tankfarm", 106.20, 20.46, "Nam Dinh"), ("tankfarm", 105.72, 18.64, "Vinh"), ("tankfarm", 106.72, 10.68, "Ho Chi Minh City"),
    ("tankfarm", 104.95, 11.50, "Phnom Penh"),
    # The ports of Haiphong and Cam Ranh Bay
    ("port", 106.70, 20.83, "Haiphong"), ("port", 109.15, 11.97, "Cam Ranh"),
    # The Ho Chi Minh Trail from the passes into Laos down to the Cambodian border
    ("trail_dump", 105.78, 17.68, "Mu Gia"), ("trail_dump", 106.13, 17.20, "Ban Karai"),
    ("trail_dump", 106.18, 16.62, "Tchepone"), ("trail_dump", 106.45, 15.78, "Saravane"),
    ("trail_dump", 106.88, 14.86, "Ho Chi Minh Trail"), ("trail_dump", 106.75, 12.75, "Ho Chi Minh Trail"),
    ("trail_dump", 105.92, 13.48, "Stung Treng"), ("trail_dump", 106.40, 12.08, "Snuol"),
    # Bridges: Long Bien (Paul Doumer) at Hanoi, Ham Rong at Thanh Hoa, Hai Duong, Viet Tri,
    # Bien Hoa, Phnom Penh
    ("bridge", 105.86, 21.04, "Hanoi"), ("bridge", 105.80, 19.86, "Thanh Hoa"), ("bridge", 106.33, 20.95, "Hai Duong"),
    ("bridge", 105.43, 21.29, "Viet Tri"), ("bridge", 106.82, 10.95, "Bien Hoa"), ("bridge", 104.92, 11.62, "Phnom Penh"),
]

# Single objects: the trail's lesser supply dumps along the border.
OBJECTS = [
    ("supply_dump", 105.95, 17.45, "Ban Laboy"),
    ("supply_dump", 106.35, 16.95, "Ban Dong"),
    ("supply_dump", 107.30, 16.15, "A Shau"),
    ("supply_dump", 107.05, 15.95, "Chavane"),
    ("supply_dump", 107.40, 15.70, "Ben Giang"),
    ("supply_dump", 106.70, 15.30, "Bolovens"),
    ("supply_dump", 107.60, 14.70, "Bo Y"),
    ("supply_dump", 107.20, 14.30, "Ban Bak"),
    ("supply_dump", 107.40, 13.70, "Duc Co"),
    ("supply_dump", 107.10, 13.40, "Ratanakiri"),
    ("supply_dump", 107.30, 12.30, "Duc Lap"),
    ("supply_dump", 106.20, 11.80, "Memot"),
]

# Cities of the enemy side with a SAM site at their edge.
CITY_SAMS = ["Hanoi", "Thanh Hoa", "Nam Dinh", "Thai Nguyen", "Hue", "Takeo", "Ban Khea", "Ban Phlak"]

# Missile boats in the Gulf of Tonkin and off the coast: (lon, lat).
BOATS = [(107.05, 20.55), (107.55, 20.35), (106.50, 19.45), (106.30, 18.85), (107.10, 17.65), (108.60, 16.35),
         (109.55, 12.60), (106.85, 9.50), (104.30, 9.70)]

# Other ships: the destroyers round the Constellation and the Kitty Hawk's escort group off
# Qui Nhon (kind, lon, lat, radar or None).
SHIPS = [
    ("destroyer", 110.32, 17.04, None), ("destroyer", 110.10, 17.10, None), ("destroyer", 110.25, 16.88, None),
    ("destroyer", 110.08, 16.92, None), ("destroyer", 110.30, 17.18, None),
    ("destroyer", 109.95, 14.20, None), ("destroyer", 110.10, 14.05, None), ("destroyer", 109.92, 13.98, None),
    ("destroyer", 110.15, 14.28, None), ("destroyer", 109.80, 14.10, None),
]

# Mountain ranges: (lon, lat, length theater km, width theater km, height m, bearing of the
# long axis in degrees from north).
MOUNTAINS = [
    # The north-west: Hoang Lien Son (Fansipan), the Son La plateau, the Viet Bac, Lang Son
    (103.90, 22.20, 30, 10, 2800, 130), (103.70, 21.30, 30, 12, 1800, 130), (105.90, 22.10, 28, 14, 1400, 100),
    
    # Laos: Phou Bia and the Plain of Jars, the Luang Prabang range on the Thai border
    (103.00, 19.20, 32, 16, 2500, 150), (101.00, 18.80, 30, 10, 1800, 170),
    # The Annamite chain from Nghe An down to the Central Highlands and Da Lat
    (104.80, 19.00, 30, 12, 2000, 130), (105.60, 18.10, 30, 12, 1700, 130), (106.40, 17.10, 30, 12, 1500, 145),
    (107.30, 16.00, 30, 12, 2000, 150), (107.90, 14.90, 30, 14, 2500, 165), (108.40, 13.60, 30, 16, 1400, 0),
    (108.40, 12.00, 30, 16, 2100, 60),
    # The Bolovens plateau, the Cardamoms, the Dangrek escarpment
    (106.30, 15.10, 20, 16, 1300, 0), (103.30, 12.10, 32, 14, 1700, 135), 
    # Thailand: Phetchabun, the Tenasserim hills, Doi Inthanon; Hainan
    (101.10, 16.60, 30, 10, 1300, 5), (98.90, 14.80, 35, 10, 1600, 160), (98.70, 18.80, 32, 14, 2400, 0),
    (109.60, 18.90, 22, 14, 1800, 40),
]

# Forest regions: jungle over the mountains of Laos, the Annamites, the Central Highlands, the
# Cardamoms and northern Thailand (radius theater km, density 0..1); the deltas stay paddies.
FORESTS = [
    (103.80, 21.80, 35, 0.75), (105.70, 22.00, 25, 0.7), (106.70, 21.50, 15, 0.55), (102.90, 19.50, 40, 0.75),
    (101.20, 19.20, 25, 0.6), (104.80, 18.90, 25, 0.8), (105.70, 17.90, 25, 0.85), (106.40, 16.90, 25, 0.85),
    (107.20, 15.90, 25, 0.85), (107.80, 14.70, 30, 0.8), (108.20, 13.20, 25, 0.6), (108.20, 11.90, 25, 0.65),
    (107.00, 12.60, 30, 0.65), (106.50, 14.80, 20, 0.6), (103.30, 12.00, 25, 0.8), (104.50, 13.80, 20, 0.4),
    (101.30, 16.80, 20, 0.5), (98.90, 15.00, 30, 0.65), (98.80, 18.50, 30, 0.6), (109.60, 18.90, 15, 0.6),
    (107.40, 11.40, 15, 0.45),
]

# The open land is green: rice paddies in the deltas, farmland in Thailand.
GREEN = True


# ---------------------------------------------------------------------------------------------
# Models of this theater's library the layouts use, and its airfield's aprons.

KINDS = {"houses": 26, "houses2": 26, "buildings": 26, "patrol_boat": 53, "land": 0, "sea": 20, "carrier": 75,
         "carrier_object": 90}
APRONS = [(-500, -78, -180, 180), (78, 500, -164, 164)]
LAND_COLOUR = 50
