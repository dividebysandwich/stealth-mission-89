# Beväpningsredigeraren (src/loadout.rs). Vapennamnen är de verkliga och översätts inte.

loadout-none = Ingen
loadout-briefed = BRIEFAD
    .facts = Lasten som uppdraget briefades med
loadout-clear = RENSA
    .facts = Alla lastpunkter tomma
loadout-done = KLAR
loadout-no-hardpoints = Inga lastpunkter
loadout-no-stores = Det här flygplanet bär ingen last
loadout-fantasy = Finns inte på det verkliga flygplanet
loadout-empty = Tom
loadout-out-of-stock = Inget i lager för det här uppdraget
loadout-mirror = Samma på båda sidor
    .facts = Lastar även { $station }
loadout-stores = Last
loadout-needs-equipment = Uppdraget kräver sin kamera eller sin specialutrustning

## Vad varje vapen är till för (vapentyp 0 till 20)

# AIM-9M Sidewinder
weapon-role-0 = Luft-luft, infraröd målsökning, kort räckvidd
# AIM-120A AMRAAM
weapon-role-1 = Luft-luft, radarmålsökning, medellång räckvidd
# AGM-88A HARM
weapon-role-2 = Antiradarrobot: radarer och SAM-ställningar
# Penguin 3 ASM
weapon-role-3 = Sjömålsrobot
# AGM-84A Harpoon
weapon-role-4 = Sjömålsrobot med lång räckvidd
# AGM-65 Maverick
weapon-role-5 = TV-styrd robot: fordon, bunkrar, byggnader
# GBU-12 Paveway
weapon-role-6 = Laserstyrd bomb: hårda mål
# Mk 20 Rockeye
weapon-role-7 = Klusterbomb: mjuka mål och ytmål
# Durandal
weapon-role-8 = Bomb för att kratera startbanor
# Mk 82 LDGP
weapon-role-9 = Fritt fallande bomb på 500 lb
# Mk 82 Snakeye
weapon-role-10 = Bromsad bomb på 500 lb för fällning på låg höjd
# Mk 20 Rockeye II
weapon-role-11 = Klusterbomb: pansar och ytmål
# Mk 122 Fireye
weapon-role-12 = Brandbomb
# CBU-72 FAE
weapon-role-13 = Bränsle-luft-sprängämne: ytmål
# Mk 35 incendiary
weapon-role-14 = Brandklusterbomb
# ISCB-1 mines
weapon-role-15 = Minor för områdesspärr
# IR/optical camera
weapon-role-16 = Spaningsbilder av målet
# 1900 lb fuel tank
weapon-role-17 = Extra bränsle, överförs när den fälls
# Special cargo
weapon-role-18 = Förnödenheter eller last för uppdraget
# 20 mm cannon
weapon-role-19 = Kanonen (ingen last)
# AGM-114 Hellfire
weapon-role-20 = Laserstyrd pansarvärnsrobot: flyger dit målpoden tittar

## Hur varje vapen används (räckvidder i HUD:ens km, höjder i fot)

employment-0 = Räckvidd: 1-12 km, längre rakt framifrån
employment-1 = Räckvidd: 1-38 km, längre rakt framifrån
employment-2 =
    Räckvidd: 9-50 km
    Avfyring: 5 000-20 000 ft
employment-3 =
    Räckvidd: 6-14 km (klassisk målsökning: 13-14 km)
    Avfyring: 100-500 ft
employment-4 =
    Räckvidd: 6-41 km
    Avfyring: 2 000-5 000 ft
employment-5 =
    Räckvidd: 5-10 km
    Avfyring: 2 000-5 000 ft
employment-6 =
    Räckvidd: 1,5 km till 2,5 gånger höjden, LOCK stadig (14 km från 15 000 ft, 18 km från 25 000 ft)
    Fällning: 15 000-25 000 ft
employment-8 =
    Räckvidd: 0,9-1,3 km före startbanan
    Fällning: 200-500 ft
employment-9 =
    Räckvidd: 5-11 km före målet
    Fällning: 3 000-20 000 ft
employment-12 =
    Räckvidd: 0,9-2,2 km före målet
    Fällning: 100-500 ft
employment-16 = Räckvidd: bilder från 2-8 km (ju kortare, desto fler motståndare), nosen mot målet
employment-18 = Räckvidd: fäll vid "Drop supplies now", inom 4 km från fällningsplatsen
employment-20 =
    Räckvidd: 4-8 km
    Avfyring: 2 000-3 000 ft
employment-guided-cluster =
    Räckvidd: 1,5 km till 2,5 gånger höjden, LOCK stadig (3,7 km från 3 000 ft, 6 km från 5 000 ft)
    Fällning: 3 000-5 000 ft
employment-medium-retarded =
    Räckvidd: 1,1-1,6 km före målet
    Fällning: 300-1 000 ft
employment-low-retarded =
    Räckvidd: 0,9-1,3 km före målet
    Fällning: 200-500 ft

## Lastpunkter

hardpoint-left-bay = Vänster utrymme
hardpoint-right-bay = Höger utrymme
hardpoint-left-wing-bay = Vänster vingutrymme
hardpoint-right-wing-bay = Höger vingutrymme
hardpoint-station-1-left-wingtip = Station 1, vänster vingspets
hardpoint-station-2-left-outer-wing = Station 2, vänster yttervinge
hardpoint-station-3-left-wing = Station 3, vänster vinge
hardpoint-station-4-left-inner-wing = Station 4, vänster innervinge
hardpoint-station-5-centreline = Station 5, mittlinje
hardpoint-station-6-right-inner-wing = Station 6, höger innervinge
hardpoint-station-7-right-wing = Station 7, höger vinge
hardpoint-station-8-right-outer-wing = Station 8, höger yttervinge
hardpoint-station-9-right-wingtip = Station 9, höger vingspets
hardpoint-station-1-left-wingtip-rail = Station 1, vänster vingspetsskena
hardpoint-station-2-left-outer-pylon = Station 2, vänster ytterpylon
hardpoint-station-3-left-inner-pylon = Station 3, vänster innerpylon
hardpoint-station-4-right-inner-pylon = Station 4, höger innerpylon
hardpoint-station-5-right-outer-pylon = Station 5, höger ytterpylon
hardpoint-station-6-right-wingtip-rail = Station 6, höger vingspetsskena
hardpoint-tornado-left-wing-rail = Vänster vinge, Sidewinder-skena
hardpoint-tornado-left-wing-pylon = Vänster vingpylon
hardpoint-tornado-left-shoulder-forward = Vänster skuldra, fram
hardpoint-tornado-left-shoulder-aft = Vänster skuldra, bak
hardpoint-tornado-right-shoulder-aft = Höger skuldra, bak
hardpoint-tornado-right-shoulder-forward = Höger skuldra, fram
hardpoint-tornado-right-wing-pylon = Höger vingpylon
hardpoint-tornado-right-wing-rail = Höger vinge, Sidewinder-skena
hardpoint-a10-1-left-outer-wing = Station 1, vänster yttervinge
hardpoint-a10-2-left-wing = Station 2, vänster vinge
hardpoint-a10-3-left-wing = Station 3, vänster vinge
hardpoint-a10-4-left-centre-wing = Station 4, vänster mittvinge
hardpoint-a10-5-left-fuselage = Station 5, vänster flygkropp
hardpoint-a10-6-centreline = Station 6, mittlinje
hardpoint-a10-7-right-fuselage = Station 7, höger flygkropp
hardpoint-a10-8-right-centre-wing = Station 8, höger mittvinge
hardpoint-a10-9-right-wing = Station 9, höger vinge
hardpoint-a10-10-right-wing = Station 10, höger vinge
hardpoint-a10-11-right-outer-wing = Station 11, höger yttervinge
