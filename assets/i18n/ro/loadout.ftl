# Editorul de armament (src/loadout.rs). Numele armelor sunt cele reale, netraduse.

loadout-none = Niciuna
loadout-briefed = BRIEFING
    .facts = Încărcăturile cu care a fost planificată misiunea
loadout-clear = GOLEȘTE
    .facts = Toate punctele de armare goale
loadout-done = GATA
loadout-no-hardpoints = Niciun punct de armare
loadout-no-stores = Această aeronavă nu poartă încărcături
loadout-fantasy = Nu există pe aeronava reală
loadout-empty = Gol
loadout-out-of-stock = Niciuna în stoc pentru această misiune
loadout-mirror = La fel pe ambele părți
    .facts = Încarcă și { $station }
loadout-stores = Încărcături
loadout-needs-equipment = Misiunea are nevoie de camera sau de echipamentul ei special

## La ce folosește fiecare armă (tipurile de armă 0 până la 20)

# AIM-9M Sidewinder
weapon-role-0 = Aer-aer, ghidare în infraroșu, rază scurtă
# AIM-120A AMRAAM
weapon-role-1 = Aer-aer, ghidare radar, rază medie
# AGM-88A HARM
weapon-role-2 = Rachetă antiradiație: radare și situri SAM
# Penguin 3 ASM
weapon-role-3 = Rachetă antinavă
# AGM-84A Harpoon
weapon-role-4 = Rachetă antinavă cu rază lungă
# AGM-65 Maverick
weapon-role-5 = Rachetă ghidată TV: vehicule, buncăre, clădiri
# GBU-12 Paveway
weapon-role-6 = Bombă ghidată cu laser: ținte întărite
# Mk 20 Rockeye
weapon-role-7 = Bombă cu dispersie: ținte ușoare și de suprafață
# Durandal
weapon-role-8 = Bombă pentru craterizarea pistei
# Mk 82 LDGP
weapon-role-9 = Bombă cu cădere liberă de 500 lb
# Mk 82 Snakeye
weapon-role-10 = Bombă frânată de 500 lb pentru lansări la joasă înălțime
# Mk 20 Rockeye II
weapon-role-11 = Bombă cu dispersie: blindate și ținte de suprafață
# Mk 122 Fireye
weapon-role-12 = Bombă incendiară
# CBU-72 FAE
weapon-role-13 = Explozibil combustibil-aer: ținte de suprafață
# Mk 35 incendiary
weapon-role-14 = Bombă incendiară cu dispersie
# ISCB-1 mines
weapon-role-15 = Mine pentru interzicerea zonei
# IR/optical camera
weapon-role-16 = Fotografii de recunoaștere ale țintei
# 1900 lb fuel tank
weapon-role-17 = Combustibil suplimentar, transferat la lansare
# Special cargo
weapon-role-18 = Provizii sau încărcătură pentru misiune
# 20 mm cannon
weapon-role-19 = Tunul (nu este o încărcătură)
# AGM-114 Hellfire
weapon-role-20 = Rachetă antitanc ghidată cu laser: zboară acolo unde privește nacela de desemnare

## Cum se folosește fiecare armă (distanțe în km pe HUD, înălțimi în picioare)

employment-0 = Distanță: 1-12 km, mai mare din față
employment-1 = Distanță: 1-38 km, mai mare din față
employment-2 =
    Distanță: 9-50 km
    Lansare: 5 000-20 000 ft
employment-3 =
    Distanță: 6-14 km (ghidare clasică: 13-14 km)
    Lansare: 100-500 ft
employment-4 =
    Distanță: 6-41 km
    Lansare: 2 000-5 000 ft
employment-5 =
    Distanță: 5-10 km
    Lansare: 2 000-5 000 ft
employment-6 =
    Distanță: de la 1,5 km la de 2,5 ori înălțimea, LOCK stabil (14 km de la 15 000 ft, 18 km de la 25 000 ft)
    Lansare: 15 000-25 000 ft
employment-8 =
    Distanță: 0,9-1,3 km înaintea pistei
    Lansare: 200-500 ft
employment-9 =
    Distanță: 5-11 km înaintea țintei
    Lansare: 3 000-20 000 ft
employment-12 =
    Distanță: 0,9-2,2 km înaintea țintei
    Lansare: 100-500 ft
employment-16 = Distanță: fotografii de la 2-8 km (cu cât mai aproape, cu atât mai mulți adversari), botul spre țintă
employment-18 = Distanță: lansați la "Drop supplies now", la mai puțin de 4 km de zona de lansare
employment-20 =
    Distanță: 4-8 km
    Lansare: 2 000-3 000 ft
employment-guided-cluster =
    Distanță: de la 1,5 km la de 2,5 ori înălțimea, LOCK stabil (3,7 km de la 3 000 ft, 6 km de la 5 000 ft)
    Lansare: 3 000-5 000 ft
employment-medium-retarded =
    Distanță: 1,1-1,6 km înaintea țintei
    Lansare: 300-1 000 ft
employment-low-retarded =
    Distanță: 0,9-1,3 km înaintea țintei
    Lansare: 200-500 ft

## Puncte de armare

hardpoint-left-bay = Compartiment stânga
hardpoint-right-bay = Compartiment dreapta
hardpoint-left-wing-bay = Compartiment aripă stânga
hardpoint-right-wing-bay = Compartiment aripă dreapta
hardpoint-station-1-left-wingtip = Stația 1, vârf de aripă stânga
hardpoint-station-2-left-outer-wing = Stația 2, aripa stângă exterioară
hardpoint-station-3-left-wing = Stația 3, aripa stângă
hardpoint-station-4-left-inner-wing = Stația 4, aripa stângă interioară
hardpoint-station-5-centreline = Stația 5, axul median
hardpoint-station-6-right-inner-wing = Stația 6, aripa dreaptă interioară
hardpoint-station-7-right-wing = Stația 7, aripa dreaptă
hardpoint-station-8-right-outer-wing = Stația 8, aripa dreaptă exterioară
hardpoint-station-9-right-wingtip = Stația 9, vârf de aripă dreapta
hardpoint-station-1-left-wingtip-rail = Stația 1, șină de vârf de aripă stânga
hardpoint-station-2-left-outer-pylon = Stația 2, pilon exterior stânga
hardpoint-station-3-left-inner-pylon = Stația 3, pilon interior stânga
hardpoint-station-4-right-inner-pylon = Stația 4, pilon interior dreapta
hardpoint-station-5-right-outer-pylon = Stația 5, pilon exterior dreapta
hardpoint-station-6-right-wingtip-rail = Stația 6, șină de vârf de aripă dreapta
hardpoint-tornado-left-wing-rail = Aripa stângă, șină Sidewinder
hardpoint-tornado-left-wing-pylon = Pilon aripă stânga
hardpoint-tornado-left-shoulder-forward = Umăr stânga, față
hardpoint-tornado-left-shoulder-aft = Umăr stânga, spate
hardpoint-tornado-right-shoulder-aft = Umăr dreapta, spate
hardpoint-tornado-right-shoulder-forward = Umăr dreapta, față
hardpoint-tornado-right-wing-pylon = Pilon aripă dreapta
hardpoint-tornado-right-wing-rail = Aripa dreaptă, șină Sidewinder
hardpoint-a10-1-left-outer-wing = Stația 1, aripa stângă exterioară
hardpoint-a10-2-left-wing = Stația 2, aripa stângă
hardpoint-a10-3-left-wing = Stația 3, aripa stângă
hardpoint-a10-4-left-centre-wing = Stația 4, aripa stângă centrală
hardpoint-a10-5-left-fuselage = Stația 5, fuzelaj stânga
hardpoint-a10-6-centreline = Stația 6, axul median
hardpoint-a10-7-right-fuselage = Stația 7, fuzelaj dreapta
hardpoint-a10-8-right-centre-wing = Stația 8, aripa dreaptă centrală
hardpoint-a10-9-right-wing = Stația 9, aripa dreaptă
hardpoint-a10-10-right-wing = Stația 10, aripa dreaptă
hardpoint-a10-11-right-outer-wing = Stația 11, aripa dreaptă exterioară
hardpoint-av8b-1-left-rail = Stația 1, șină Sidewinder stânga
hardpoint-av8b-3-left-middle-wing = Stația 3, aripa stângă, mijloc
hardpoint-av8b-4-left-inboard-wing = Stația 4, aripa stângă, interior
hardpoint-av8b-6-right-inboard-wing = Stația 6, aripa dreaptă, interior
hardpoint-av8b-7-right-middle-wing = Stația 7, aripa dreaptă, mijloc
hardpoint-av8b-9-right-rail = Stația 9, șină Sidewinder dreapta
weapon-role-21 = Air-to-air, infrared homing, very short range
employment-21 = Range: 1-6 km, farther head-on
