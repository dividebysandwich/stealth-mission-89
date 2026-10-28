# Urejevalnik oborožitve (src/loadout.rs). Imena orožij so prava, neprevedena.

loadout-none = Nič
loadout-briefed = BRIEFING
    .facts = Oborožitev, s katero je bila misija začrtana v briefingu
loadout-clear = POČISTI
    .facts = Vsi nosilci prazni
loadout-done = KONEC
loadout-no-hardpoints = Ni nosilcev
loadout-no-stores = To letalo ne nosi oborožitve
loadout-fantasy = Ni na pravem letalu
loadout-empty = Prazno
loadout-out-of-stock = Za to misijo ni nič na zalogi
loadout-mirror = Enako na obeh straneh
    .facts = Naloži tudi: { $station }
loadout-stores = Oborožitev
loadout-needs-equipment = Misija potrebuje kamero ali posebno opremo

## Čemu služi posamezno orožje (vrste orožja od 0 do 20)

# AIM-9M Sidewinder
weapon-role-0 = Zrak-zrak, infrardeče samonavajanje, kratek doseg
# AIM-120A AMRAAM
weapon-role-1 = Zrak-zrak, radarsko samonavajanje, srednji doseg
# AGM-88A HARM
weapon-role-2 = Protiradarski izstrelek: radarji in položaji SAM
# Penguin 3 ASM
weapon-role-3 = Protiladijski izstrelek
# AGM-84A Harpoon
weapon-role-4 = Protiladijski izstrelek dolgega dosega
# AGM-65 Maverick
weapon-role-5 = TV-vodeni izstrelek: vozila, bunkerji, zgradbe
# GBU-12 Paveway
weapon-role-6 = Lasersko vodena bomba: utrjeni cilji
# Mk 20 Rockeye
weapon-role-7 = Kasetna bomba: mehki in površinski cilji
# Durandal
weapon-role-8 = Bomba za uničenje vzletne steze
# Mk 82 LDGP
weapon-role-9 = 500-funtna bomba prostega pada
# Mk 82 Snakeye
weapon-role-10 = 500-funtna zavirana bomba za nizke spuste
# Mk 20 Rockeye II
weapon-role-11 = Kasetna bomba: oklepniki in površinski cilji
# Mk 122 Fireye
weapon-role-12 = Zažigalna bomba
# CBU-72 FAE
weapon-role-13 = Termobarična bomba: površinski cilji
# Mk 35 incendiary
weapon-role-14 = Zažigalna kasetna bomba
# ISCB-1 mines
weapon-role-15 = Mine za zapiranje območja
# IR/optical camera
weapon-role-16 = Izvidniške fotografije cilja
# 1900 lb fuel tank
weapon-role-17 = Dodatno gorivo, ob spustu se prečrpa
# Special cargo
weapon-role-18 = Zaloge ali tovor za misijo
# 20 mm cannon
weapon-role-19 = Top (ni oborožitev)
# AGM-114 Hellfire
weapon-role-20 = Lasersko voden protioklepni izstrelek: leti tja, kamor gleda ciljniški pod

## Kako se posamezno orožje uporablja (dosegi v km na HUD, višine v čevljih)

employment-0 = Doseg: 1-12 km, dlje čelno
employment-1 = Doseg: 1-38 km, dlje čelno
employment-2 =
    Doseg: 9-50 km
    Izstrelitev: 5 000-20 000 ft
employment-3 =
    Doseg: 6-14 km (klasično vodenje: 13-14 km)
    Izstrelitev: 100-500 ft
employment-4 =
    Doseg: 6-41 km
    Izstrelitev: 2 000-5 000 ft
employment-5 =
    Doseg: 5-10 km
    Izstrelitev: 2 000-5 000 ft
employment-6 =
    Doseg: od 1,5 km do 2,5-kratne višine, LOCK neprekinjen (14 km z 15 000 ft, 18 km z 25 000 ft)
    Spust: 15 000-25 000 ft
employment-8 =
    Doseg: 0,9-1,3 km pred stezo
    Spust: 200-500 ft
employment-9 =
    Doseg: 5-11 km pred ciljem
    Spust: 3 000-20 000 ft
employment-12 =
    Doseg: 0,9-2,2 km pred ciljem
    Spust: 100-500 ft
employment-16 = Doseg: fotografije z 2-8 km (krajše, več nasprotnikov), nos proti cilju
employment-18 = Doseg: spust ob "Drop supplies now", znotraj 4 km od cone spusta
employment-20 =
    Doseg: 4-8 km
    Izstrelitev: 2 000-3 000 ft
employment-guided-cluster =
    Doseg: od 1,5 km do 2,5-kratne višine, LOCK neprekinjen (3,7 km s 3 000 ft, 6 km s 5 000 ft)
    Spust: 3 000-5 000 ft
employment-medium-retarded =
    Doseg: 1,1-1,6 km pred ciljem
    Spust: 300-1 000 ft
employment-low-retarded =
    Doseg: 0,9-1,3 km pred ciljem
    Spust: 200-500 ft

## Nosilci

hardpoint-left-bay = Levi prostor
hardpoint-right-bay = Desni prostor
hardpoint-left-wing-bay = Levi prostor v krilu
hardpoint-right-wing-bay = Desni prostor v krilu
hardpoint-station-1-left-wingtip = Postaja 1, leva konica krila
hardpoint-station-2-left-outer-wing = Postaja 2, levo zunanje krilo
hardpoint-station-3-left-wing = Postaja 3, levo krilo
hardpoint-station-4-left-inner-wing = Postaja 4, levo notranje krilo
hardpoint-station-5-centreline = Postaja 5, središčnica
hardpoint-station-6-right-inner-wing = Postaja 6, desno notranje krilo
hardpoint-station-7-right-wing = Postaja 7, desno krilo
hardpoint-station-8-right-outer-wing = Postaja 8, desno zunanje krilo
hardpoint-station-9-right-wingtip = Postaja 9, desna konica krila
hardpoint-station-1-left-wingtip-rail = Postaja 1, tirnica na levi konici krila
hardpoint-station-2-left-outer-pylon = Postaja 2, levi zunanji pilon
hardpoint-station-3-left-inner-pylon = Postaja 3, levi notranji pilon
hardpoint-station-4-right-inner-pylon = Postaja 4, desni notranji pilon
hardpoint-station-5-right-outer-pylon = Postaja 5, desni zunanji pilon
hardpoint-station-6-right-wingtip-rail = Postaja 6, tirnica na desni konici krila
hardpoint-tornado-left-wing-rail = Levo krilo, tirnica za Sidewinder
hardpoint-tornado-left-wing-pylon = Pilon levega krila
hardpoint-tornado-left-shoulder-forward = Leva rama, spredaj
hardpoint-tornado-left-shoulder-aft = Leva rama, zadaj
hardpoint-tornado-right-shoulder-aft = Desna rama, zadaj
hardpoint-tornado-right-shoulder-forward = Desna rama, spredaj
hardpoint-tornado-right-wing-pylon = Pilon desnega krila
hardpoint-tornado-right-wing-rail = Desno krilo, tirnica za Sidewinder
hardpoint-a10-1-left-outer-wing = Postaja 1, levo zunanje krilo
hardpoint-a10-2-left-wing = Postaja 2, levo krilo
hardpoint-a10-3-left-wing = Postaja 3, levo krilo
hardpoint-a10-4-left-centre-wing = Postaja 4, levo srednje krilo
hardpoint-a10-5-left-fuselage = Postaja 5, levi trup
hardpoint-a10-6-centreline = Postaja 6, središčnica
hardpoint-a10-7-right-fuselage = Postaja 7, desni trup
hardpoint-a10-8-right-centre-wing = Postaja 8, desno srednje krilo
hardpoint-a10-9-right-wing = Postaja 9, desno krilo
hardpoint-a10-10-right-wing = Postaja 10, desno krilo
hardpoint-a10-11-right-outer-wing = Postaja 11, desno zunanje krilo
