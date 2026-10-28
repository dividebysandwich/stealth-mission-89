# Editor výzbroje (src/loadout.rs). Názvy zbraní jsou skutečné, nepřekládají se.

loadout-none = Žádná
loadout-briefed = BRIEFING
    .facts = Výzbroj, se kterou byla mise zadána v briefingu
loadout-clear = VYPRÁZDNIT
    .facts = Všechny závěsníky prázdné
loadout-done = HOTOVO
loadout-no-hardpoints = Žádné závěsníky
loadout-no-stores = Tento letoun nenese žádnou výzbroj
loadout-fantasy = Na skutečném letounu neexistuje
loadout-empty = Prázdné
loadout-out-of-stock = Pro tuto misi nejsou na skladě
loadout-mirror = Stejně na obou stranách
    .facts = Naloží také: { $station }
loadout-stores = Výzbroj
loadout-needs-equipment = Mise vyžaduje kameru nebo zvláštní vybavení

## K čemu která zbraň slouží (typy zbraní 0 až 20)

# AIM-9M Sidewinder
weapon-role-0 = Vzduch-vzduch, infračervené navádění, krátký dosah
# AIM-120A AMRAAM
weapon-role-1 = Vzduch-vzduch, radarové navádění, střední dosah
# AGM-88A HARM
weapon-role-2 = Protiradiolokační střela: radary a stanoviště SAM
# Penguin 3 ASM
weapon-role-3 = Protilodní střela
# AGM-84A Harpoon
weapon-role-4 = Protilodní střela dlouhého dosahu
# AGM-65 Maverick
weapon-role-5 = Střela naváděná televizí: vozidla, bunkry, budovy
# GBU-12 Paveway
weapon-role-6 = Laserem naváděná puma: odolné cíle
# Mk 20 Rockeye
weapon-role-7 = Kazetová puma: měkké a plošné cíle
# Durandal
weapon-role-8 = Puma k ničení přistávacích drah
# Mk 82 LDGP
weapon-role-9 = Volně padající puma 500 lb
# Mk 82 Snakeye
weapon-role-10 = Brzděná puma 500 lb pro nízké odhozy
# Mk 20 Rockeye II
weapon-role-11 = Kazetová puma: obrněné a plošné cíle
# Mk 122 Fireye
weapon-role-12 = Zápalná puma
# CBU-72 FAE
weapon-role-13 = Termobarická puma: plošné cíle
# Mk 35 incendiary
weapon-role-14 = Zápalná kazetová puma
# ISCB-1 mines
weapon-role-15 = Miny k blokování prostoru
# IR/optical camera
weapon-role-16 = Průzkumné snímky cíle
# 1900 lb fuel tank
weapon-role-17 = Další palivo, po odhození se přečerpá
# Special cargo
weapon-role-18 = Zásoby nebo náklad pro misi
# 20 mm cannon
weapon-role-19 = Kanón (není součástí výzbroje)
# AGM-114 Hellfire
weapon-role-20 = Laserem naváděná protitanková střela: letí tam, kam se dívá zaměřovací kontejner

## Jak se která zbraň používá (dosahy v km podle HUD, výšky ve stopách)

employment-0 = Dosah: 1-12 km, při čelním útoku dále
employment-1 = Dosah: 1-38 km, při čelním útoku dále
employment-2 =
    Dosah: 9-50 km
    Odpal: 5 000-20 000 ft
employment-3 =
    Dosah: 6-14 km (klasické navádění: 13-14 km)
    Odpal: 100-500 ft
employment-4 =
    Dosah: 6-41 km
    Odpal: 2 000-5 000 ft
employment-5 =
    Dosah: 5-10 km
    Odpal: 2 000-5 000 ft
employment-6 =
    Dosah: od 1,5 km do 2,5násobku výšky, LOCK stálý (14 km z 15 000 ft, 18 km z 25 000 ft)
    Odhoz: 15 000-25 000 ft
employment-8 =
    Dosah: 0,9-1,3 km před dráhou
    Odhoz: 200-500 ft
employment-9 =
    Dosah: 5-11 km před cílem
    Odhoz: 3 000-20 000 ft
employment-12 =
    Dosah: 0,9-2,2 km před cílem
    Odhoz: 100-500 ft
employment-16 = Dosah: snímky z 2-8 km (čím více protivníků, tím blíže), příď na cíl
employment-18 = Dosah: odhoďte na pokyn "Drop supplies now", do 4 km od zóny odhozu
employment-20 =
    Dosah: 4-8 km
    Odpal: 2 000-3 000 ft
employment-guided-cluster =
    Dosah: od 1,5 km do 2,5násobku výšky, LOCK stálý (3,7 km z 3 000 ft, 6 km z 5 000 ft)
    Odhoz: 3 000-5 000 ft
employment-medium-retarded =
    Dosah: 1,1-1,6 km před cílem
    Odhoz: 300-1 000 ft
employment-low-retarded =
    Dosah: 0,9-1,3 km před cílem
    Odhoz: 200-500 ft

## Závěsníky

hardpoint-left-bay = Levá šachta
hardpoint-right-bay = Pravá šachta
hardpoint-left-wing-bay = Levá křídlová šachta
hardpoint-right-wing-bay = Pravá křídlová šachta
hardpoint-station-1-left-wingtip = Stanice 1, levý konec křídla
hardpoint-station-2-left-outer-wing = Stanice 2, levé vnější křídlo
hardpoint-station-3-left-wing = Stanice 3, levé křídlo
hardpoint-station-4-left-inner-wing = Stanice 4, levé vnitřní křídlo
hardpoint-station-5-centreline = Stanice 5, osa trupu
hardpoint-station-6-right-inner-wing = Stanice 6, pravé vnitřní křídlo
hardpoint-station-7-right-wing = Stanice 7, pravé křídlo
hardpoint-station-8-right-outer-wing = Stanice 8, pravé vnější křídlo
hardpoint-station-9-right-wingtip = Stanice 9, pravý konec křídla
hardpoint-station-1-left-wingtip-rail = Stanice 1, kolejnice na levém konci křídla
hardpoint-station-2-left-outer-pylon = Stanice 2, levý vnější pylon
hardpoint-station-3-left-inner-pylon = Stanice 3, levý vnitřní pylon
hardpoint-station-4-right-inner-pylon = Stanice 4, pravý vnitřní pylon
hardpoint-station-5-right-outer-pylon = Stanice 5, pravý vnější pylon
hardpoint-station-6-right-wingtip-rail = Stanice 6, kolejnice na pravém konci křídla
hardpoint-tornado-left-wing-rail = Levé křídlo, kolejnice Sidewinder
hardpoint-tornado-left-wing-pylon = Pylon levého křídla
hardpoint-tornado-left-shoulder-forward = Levé rameno, vpředu
hardpoint-tornado-left-shoulder-aft = Levé rameno, vzadu
hardpoint-tornado-right-shoulder-aft = Pravé rameno, vzadu
hardpoint-tornado-right-shoulder-forward = Pravé rameno, vpředu
hardpoint-tornado-right-wing-pylon = Pylon pravého křídla
hardpoint-tornado-right-wing-rail = Pravé křídlo, kolejnice Sidewinder
hardpoint-a10-1-left-outer-wing = Stanice 1, levé vnější křídlo
hardpoint-a10-2-left-wing = Stanice 2, levé křídlo
hardpoint-a10-3-left-wing = Stanice 3, levé křídlo
hardpoint-a10-4-left-centre-wing = Stanice 4, levé střední křídlo
hardpoint-a10-5-left-fuselage = Stanice 5, levá strana trupu
hardpoint-a10-6-centreline = Stanice 6, osa trupu
hardpoint-a10-7-right-fuselage = Stanice 7, pravá strana trupu
hardpoint-a10-8-right-centre-wing = Stanice 8, pravé střední křídlo
hardpoint-a10-9-right-wing = Stanice 9, pravé křídlo
hardpoint-a10-10-right-wing = Stanice 10, pravé křídlo
hardpoint-a10-11-right-outer-wing = Stanice 11, pravé vnější křídlo
