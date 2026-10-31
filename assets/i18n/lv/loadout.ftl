# Bruņojuma redaktors (src/loadout.rs). Ieroču nosaukumi ir īstie, netulkoti.

loadout-none = Nav
loadout-briefed = BRĪFINGS
    .facts = Iekārtas, ar kurām misija tika izstrādāta brīfingā
loadout-clear = NOTĪRĪT
    .facts = Visi piekares punkti tukši
loadout-done = GATAVS
loadout-no-hardpoints = Nav piekares punktu
loadout-no-stores = Šī lidmašīna nenes iekārtas
loadout-fantasy = Nav uz īstās lidmašīnas
loadout-empty = Tukšs
loadout-out-of-stock = Šai misijai krājumā nav
loadout-mirror = Vienādi abās pusēs
    .facts = Piekar arī { $station }
loadout-stores = Iekārtas
loadout-needs-equipment = Misijai vajadzīga sava kamera vai īpašais aprīkojums

## Kam kalpo katrs ierocis (ieroču veidi no 0 līdz 20)

# AIM-9M Sidewinder
weapon-role-0 = Gaiss-gaiss, infrasarkanā vadība, mazs rādiuss
# AIM-120A AMRAAM
weapon-role-1 = Gaiss-gaiss, radara vadība, vidējs rādiuss
# AGM-88A HARM
weapon-role-2 = Pretradara raķete: radari un SAM pozīcijas
# Penguin 3 ASM
weapon-role-3 = Prettkuģu raķete
# AGM-84A Harpoon
weapon-role-4 = Tāla darbības rādiusa prettkuģu raķete
# AGM-65 Maverick
weapon-role-5 = TV vadāma raķete: transportlīdzekļi, bunkuri, ēkas
# GBU-12 Paveway
weapon-role-6 = Ar lāzeru vadāma bumba: nocietināti mērķi
# Mk 20 Rockeye
weapon-role-7 = Kasešu bumba: vāji aizsargāti un platības mērķi
# Durandal
weapon-role-8 = Skrejceļa bojāšanas bumba
# Mk 82 LDGP
weapon-role-9 = 500 lb brīvās krišanas bumba
# Mk 82 Snakeye
weapon-role-10 = 500 lb bremzējama bumba zemiem nomešanas augstumiem
# Mk 20 Rockeye II
weapon-role-11 = Kasešu bumba: bruņumašīnas un platības mērķi
# Mk 122 Fireye
weapon-role-12 = Aizdedzinošā bumba
# CBU-72 FAE
weapon-role-13 = Gaisa-degvielas sprāgstviela: platības mērķi
# Mk 35 incendiary
weapon-role-14 = Aizdedzinošā kasešu bumba
# ISCB-1 mines
weapon-role-15 = Teritorijas aizlieguma mīnas
# IR/optical camera
weapon-role-16 = Mērķa izlūkfotogrāfijas
# 1900 lb fuel tank
weapon-role-17 = Papildu degviela, pārlej nometot
# Special cargo
weapon-role-18 = Krājumi vai krava misijai
# 20 mm cannon
weapon-role-19 = Lielgabals (nav iekārta)
# AGM-114 Hellfire
weapon-role-20 = Ar lāzeru vadāma prettanku raķete: lido uz vietu, kurai pod skatās

## Kā lieto katru ieroci (rādiusi HUD km, augstumi pēdās)

employment-0 = Rādiuss: 1-12 km, tālāk pret pretinieku
employment-1 = Rādiuss: 1-38 km, tālāk pret pretinieku
employment-2 =
    Rādiuss: 9-50 km
    Palaišana: 5 000-20 000 ft
employment-3 =
    Rādiuss: 6-14 km (klasiskā vadība: 13-14 km)
    Palaišana: 100-500 ft
employment-4 =
    Rādiuss: 6-41 km
    Palaišana: 2 000-5 000 ft
employment-5 =
    Rādiuss: 5-10 km
    Palaišana: 2 000-5 000 ft
employment-6 =
    Rādiuss: no 1,5 km līdz 2,5 reizēm augstumam, LOCK stabils (14 km no 15 000 ft, 18 km no 25 000 ft)
    Nomešana: 15 000-25 000 ft
employment-8 =
    Rādiuss: 0,9-1,3 km pirms skrejceļa
    Nomešana: 200-500 ft
employment-9 =
    Rādiuss: 5-11 km pirms mērķa
    Nomešana: 3 000-20 000 ft
employment-12 =
    Rādiuss: 0,9-2,2 km pirms mērķa
    Nomešana: 100-500 ft
employment-16 = Rādiuss: foto no 2-8 km (jo vairāk pretinieku, jo tuvāk), deguns uz mērķi
employment-18 = Rādiuss: nomest pie "Drop supplies now", tuvāk par 4 km no nomešanas zonas
employment-20 =
    Rādiuss: 4-8 km
    Palaišana: 2 000-3 000 ft
employment-guided-cluster =
    Rādiuss: no 1,5 km līdz 2,5 reizēm augstumam, LOCK stabils (3,7 km no 3 000 ft, 6 km no 5 000 ft)
    Nomešana: 3 000-5 000 ft
employment-medium-retarded =
    Rādiuss: 1,1-1,6 km pirms mērķa
    Nomešana: 300-1 000 ft
employment-low-retarded =
    Rādiuss: 0,9-1,3 km pirms mērķa
    Nomešana: 200-500 ft

## Piekares punkti

hardpoint-left-bay = Kreisais nodalījums
hardpoint-right-bay = Labais nodalījums
hardpoint-left-wing-bay = Kreisā spārna nodalījums
hardpoint-right-wing-bay = Labā spārna nodalījums
hardpoint-station-1-left-wingtip = Stacija 1, kreisais spārna gals
hardpoint-station-2-left-outer-wing = Stacija 2, kreisais ārējais spārns
hardpoint-station-3-left-wing = Stacija 3, kreisais spārns
hardpoint-station-4-left-inner-wing = Stacija 4, kreisais iekšējais spārns
hardpoint-station-5-centreline = Stacija 5, centrālā ass
hardpoint-station-6-right-inner-wing = Stacija 6, labais iekšējais spārns
hardpoint-station-7-right-wing = Stacija 7, labais spārns
hardpoint-station-8-right-outer-wing = Stacija 8, labais ārējais spārns
hardpoint-station-9-right-wingtip = Stacija 9, labais spārna gals
hardpoint-station-1-left-wingtip-rail = Stacija 1, kreisā spārna gala šķērsis
hardpoint-station-2-left-outer-pylon = Stacija 2, kreisais ārējais pilons
hardpoint-station-3-left-inner-pylon = Stacija 3, kreisais iekšējais pilons
hardpoint-station-4-right-inner-pylon = Stacija 4, labais iekšējais pilons
hardpoint-station-5-right-outer-pylon = Stacija 5, labais ārējais pilons
hardpoint-station-6-right-wingtip-rail = Stacija 6, labā spārna gala šķērsis
hardpoint-tornado-left-wing-rail = Kreisais spārns, Sidewinder šķērsis
hardpoint-tornado-left-wing-pylon = Kreisā spārna pilons
hardpoint-tornado-left-shoulder-forward = Kreisais plecs, priekšā
hardpoint-tornado-left-shoulder-aft = Kreisais plecs, aizmugurē
hardpoint-tornado-right-shoulder-aft = Labais plecs, aizmugurē
hardpoint-tornado-right-shoulder-forward = Labais plecs, priekšā
hardpoint-tornado-right-wing-pylon = Labā spārna pilons
hardpoint-tornado-right-wing-rail = Labais spārns, Sidewinder šķērsis
hardpoint-a10-1-left-outer-wing = Stacija 1, kreisais ārējais spārns
hardpoint-a10-2-left-wing = Stacija 2, kreisais spārns
hardpoint-a10-3-left-wing = Stacija 3, kreisais spārns
hardpoint-a10-4-left-centre-wing = Stacija 4, kreisais vidējais spārns
hardpoint-a10-5-left-fuselage = Stacija 5, kreisais fizelāžas sāns
hardpoint-a10-6-centreline = Stacija 6, centrālā ass
hardpoint-a10-7-right-fuselage = Stacija 7, labais fizelāžas sāns
hardpoint-a10-8-right-centre-wing = Stacija 8, labais vidējais spārns
hardpoint-a10-9-right-wing = Stacija 9, labais spārns
hardpoint-a10-10-right-wing = Stacija 10, labais spārns
hardpoint-a10-11-right-outer-wing = Stacija 11, labais ārējais spārns
hardpoint-av8b-1-left-rail = Stacija 1, kreisais Sidewinder šķērsis
hardpoint-av8b-3-left-middle-wing = Stacija 3, kreisais spārns, vidus
hardpoint-av8b-4-left-inboard-wing = Stacija 4, kreisais spārns, pie fizelāžas
hardpoint-av8b-6-right-inboard-wing = Stacija 6, labais spārns, pie fizelāžas
hardpoint-av8b-7-right-middle-wing = Stacija 7, labais spārns, vidus
hardpoint-av8b-9-right-rail = Stacija 9, labais Sidewinder šķērsis
weapon-role-21 = Air-to-air, infrared homing, very short range
employment-21 = Range: 1-6 km, farther head-on
