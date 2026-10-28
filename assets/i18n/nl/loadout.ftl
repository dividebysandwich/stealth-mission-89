# De bewapeningseditor (src/loadout.rs). De wapennamen zijn de echte, onvertaald.

loadout-none = Geen
loadout-briefed = BRIEFING
    .facts = De stores waarmee de missie is gebriefd
loadout-clear = LEEGMAKEN
    .facts = Elk ophangpunt leeg
loadout-done = KLAAR
loadout-no-hardpoints = Geen ophangpunten
loadout-no-stores = Dit vliegtuig draagt geen stores
loadout-fantasy = Niet op het echte vliegtuig
loadout-empty = Leeg
loadout-out-of-stock = Geen voorraad voor deze missie
loadout-mirror = Aan beide kanten hetzelfde
    .facts = Laadt ook { $station }
loadout-stores = Stores
loadout-needs-equipment = De missie heeft haar camera of speciale uitrusting nodig

## Waar elk wapen voor dient (wapentypes 0 tot 20)

# AIM-9M Sidewinder
weapon-role-0 = Lucht-lucht, infraroodgeleiding, korte afstand
# AIM-120A AMRAAM
weapon-role-1 = Lucht-lucht, radargeleiding, middellange afstand
# AGM-88A HARM
weapon-role-2 = Anti-radarraket: radars en SAM-stellingen
# Penguin 3 ASM
weapon-role-3 = Antischeepsraket
# AGM-84A Harpoon
weapon-role-4 = Antischeepsraket met groot bereik
# AGM-65 Maverick
weapon-role-5 = Tv-geleide raket: voertuigen, bunkers, gebouwen
# GBU-12 Paveway
weapon-role-6 = Lasergeleide bom: versterkte doelen
# Mk 20 Rockeye
weapon-role-7 = Clusterbom: zachte doelen en gebiedsdoelen
# Durandal
weapon-role-8 = Bom tegen start- en landingsbanen
# Mk 82 LDGP
weapon-role-9 = Vrijvallende bom van 500 lb
# Mk 82 Snakeye
weapon-role-10 = Afgeremde bom van 500 lb voor afworpen op lage hoogte
# Mk 20 Rockeye II
weapon-role-11 = Clusterbom: pantservoertuigen en gebiedsdoelen
# Mk 122 Fireye
weapon-role-12 = Brandbom
# CBU-72 FAE
weapon-role-13 = Brandstof-luchtexplosief: gebiedsdoelen
# Mk 35 incendiary
weapon-role-14 = Brandclusterbom
# ISCB-1 mines
weapon-role-15 = Mijnen om een gebied te blokkeren
# IR/optical camera
weapon-role-16 = Verkenningsfoto's van het doel
# 1900 lb fuel tank
weapon-role-17 = Extra brandstof, overgepompt bij het afwerpen
# Special cargo
weapon-role-18 = Voorraden of lading voor de missie
# 20 mm cannon
weapon-role-19 = Het kanon (geen store)
# AGM-114 Hellfire
weapon-role-20 = Lasergeleide antitankraket: vliegt naar waar de targeting pod kijkt

## Hoe elk wapen wordt ingezet (bereik in km van de HUD, hoogtes in voet)

employment-0 = Bereik: 1-12 km, verder van voren
employment-1 = Bereik: 1-38 km, verder van voren
employment-2 =
    Bereik: 9-50 km
    Lancering: 5.000-20.000 ft
employment-3 =
    Bereik: 6-14 km (klassieke geleiding: 13-14 km)
    Lancering: 100-500 ft
employment-4 =
    Bereik: 6-41 km
    Lancering: 2.000-5.000 ft
employment-5 =
    Bereik: 5-10 km
    Lancering: 2.000-5.000 ft
employment-6 =
    Bereik: van 1,5 km tot 2,5 keer de hoogte, LOCK constant (14 km vanaf 15.000 ft, 18 km vanaf 25.000 ft)
    Afworp: 15.000-25.000 ft
employment-8 =
    Bereik: 0,9-1,3 km voor de baan
    Afworp: 200-500 ft
employment-9 =
    Bereik: 5-11 km voor het doel
    Afworp: 3.000-20.000 ft
employment-12 =
    Bereik: 0,9-2,2 km voor het doel
    Afworp: 100-500 ft
employment-16 = Bereik: foto's vanaf 2-8 km (hoe meer tegenstanders, hoe korter), neus op het doel
employment-18 = Bereik: afwerpen bij "Drop supplies now", binnen 4 km van de dropzone
employment-20 =
    Bereik: 4-8 km
    Lancering: 2.000-3.000 ft
employment-guided-cluster =
    Bereik: van 1,5 km tot 2,5 keer de hoogte, LOCK constant (3,7 km vanaf 3.000 ft, 6 km vanaf 5.000 ft)
    Afworp: 3.000-5.000 ft
employment-medium-retarded =
    Bereik: 1,1-1,6 km voor het doel
    Afworp: 300-1.000 ft
employment-low-retarded =
    Bereik: 0,9-1,3 km voor het doel
    Afworp: 200-500 ft

## Ophangpunten

hardpoint-left-bay = Linker schacht
hardpoint-right-bay = Rechter schacht
hardpoint-left-wing-bay = Linker vleugelschacht
hardpoint-right-wing-bay = Rechter vleugelschacht
hardpoint-station-1-left-wingtip = Station 1, linker vleugeltip
hardpoint-station-2-left-outer-wing = Station 2, linker buitenvleugel
hardpoint-station-3-left-wing = Station 3, linker vleugel
hardpoint-station-4-left-inner-wing = Station 4, linker binnenvleugel
hardpoint-station-5-centreline = Station 5, middenlijn
hardpoint-station-6-right-inner-wing = Station 6, rechter binnenvleugel
hardpoint-station-7-right-wing = Station 7, rechter vleugel
hardpoint-station-8-right-outer-wing = Station 8, rechter buitenvleugel
hardpoint-station-9-right-wingtip = Station 9, rechter vleugeltip
hardpoint-station-1-left-wingtip-rail = Station 1, rail op linker vleugeltip
hardpoint-station-2-left-outer-pylon = Station 2, linker buitenpylon
hardpoint-station-3-left-inner-pylon = Station 3, linker binnenpylon
hardpoint-station-4-right-inner-pylon = Station 4, rechter binnenpylon
hardpoint-station-5-right-outer-pylon = Station 5, rechter buitenpylon
hardpoint-station-6-right-wingtip-rail = Station 6, rail op rechter vleugeltip
hardpoint-tornado-left-wing-rail = Linker vleugel, Sidewinder-rail
hardpoint-tornado-left-wing-pylon = Pylon van de linker vleugel
hardpoint-tornado-left-shoulder-forward = Linker schouder, voor
hardpoint-tornado-left-shoulder-aft = Linker schouder, achter
hardpoint-tornado-right-shoulder-aft = Rechter schouder, achter
hardpoint-tornado-right-shoulder-forward = Rechter schouder, voor
hardpoint-tornado-right-wing-pylon = Pylon van de rechter vleugel
hardpoint-tornado-right-wing-rail = Rechter vleugel, Sidewinder-rail
hardpoint-a10-1-left-outer-wing = Station 1, linker buitenvleugel
hardpoint-a10-2-left-wing = Station 2, linker vleugel
hardpoint-a10-3-left-wing = Station 3, linker vleugel
hardpoint-a10-4-left-centre-wing = Station 4, linker middenvleugel
hardpoint-a10-5-left-fuselage = Station 5, linker romp
hardpoint-a10-6-centreline = Station 6, middenlijn
hardpoint-a10-7-right-fuselage = Station 7, rechter romp
hardpoint-a10-8-right-centre-wing = Station 8, rechter middenvleugel
hardpoint-a10-9-right-wing = Station 9, rechter vleugel
hardpoint-a10-10-right-wing = Station 10, rechter vleugel
hardpoint-a10-11-right-outer-wing = Station 11, rechter buitenvleugel
