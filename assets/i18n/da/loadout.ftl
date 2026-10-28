# Bevæbningseditoren (src/loadout.rs). Våbennavnene er de rigtige, uoversatte.

loadout-none = Ingen
loadout-briefed = BRIEFING
    .facts = Den last, missionen blev briefet med
loadout-clear = RYD
    .facts = Alle våbenpunkter tomme
loadout-done = OK
loadout-no-hardpoints = Ingen våbenpunkter
loadout-no-stores = Dette fly bærer ingen last
loadout-fantasy = Findes ikke på det rigtige fly
loadout-empty = Tom
loadout-out-of-stock = Ingen på lager til denne mission
loadout-mirror = Ens på begge sider
    .facts = Lader også { $station }
loadout-stores = Last
loadout-needs-equipment = Missionen kræver sit kamera eller sit specialudstyr

## Hvad hvert våben bruges til

# AIM-9M Sidewinder
weapon-role-0 = Luft-til-luft, infrarød målsøgning, kort rækkevidde
# AIM-120A AMRAAM
weapon-role-1 = Luft-til-luft, radarmålsøgning, mellemlang rækkevidde
# AGM-88A HARM
weapon-role-2 = Antiradarmissil: radarer og SAM-stillinger
# Penguin 3 ASM
weapon-role-3 = Antiskibsmissil
# AGM-84A Harpoon
weapon-role-4 = Antiskibsmissil med lang rækkevidde
# AGM-65 Maverick
weapon-role-5 = TV-styret missil: køretøjer, bunkere, bygninger
# GBU-12 Paveway
weapon-role-6 = Lasersyret bombe: hårde mål
# Mk 20 Rockeye
weapon-role-7 = Klyngebombe: bløde mål og områdemål
# Durandal
weapon-role-8 = Bombe til at ødelægge landingsbaner
# Mk 82 LDGP
weapon-role-9 = Frit faldende bombe på 500 lb
# Mk 82 Snakeye
weapon-role-10 = Bremset bombe på 500 lb til udløsning i lav højde
# Mk 20 Rockeye II
weapon-role-11 = Klyngebombe: panserkøretøjer og områdemål
# Mk 122 Fireye
weapon-role-12 = Brandbombe
# CBU-72 FAE
weapon-role-13 = Brændstof-luft-sprængstof: områdemål
# Mk 35 incendiary
weapon-role-14 = Brandklyngebombe
# ISCB-1 mines
weapon-role-15 = Miner til områdespærring
# IR/optical camera
weapon-role-16 = Rekognosceringsfotos af målet
# 1900 lb fuel tank
weapon-role-17 = Ekstra brændstof, overføres ved udløsning
# Special cargo
weapon-role-18 = Forsyninger eller fragt til missionen
# 20 mm cannon
weapon-role-19 = Kanonen (ikke last)
# AGM-114 Hellfire
weapon-role-20 = Lasersyret panserværnsmissil: flyver mod det sted, podden kigger på

## Hvordan hvert våben bruges (rækkevidder i HUD'ens km, højder i fod)

employment-0 = Rækkevidde: 1-12 km, længere forfra
employment-1 = Rækkevidde: 1-38 km, længere forfra
employment-2 =
    Rækkevidde: 9-50 km
    Affyring: 5.000-20.000 ft
employment-3 =
    Rækkevidde: 6-14 km (klassisk målsøgning: 13-14 km)
    Affyring: 100-500 ft
employment-4 =
    Rækkevidde: 6-41 km
    Affyring: 2.000-5.000 ft
employment-5 =
    Rækkevidde: 5-10 km
    Affyring: 2.000-5.000 ft
employment-6 =
    Rækkevidde: fra 1,5 km til 2,5 gange højden, LOCK konstant (14 km fra 15.000 ft, 18 km fra 25.000 ft)
    Udløsning: 15.000-25.000 ft
employment-8 =
    Rækkevidde: 0,9-1,3 km før landingsbanen
    Udløsning: 200-500 ft
employment-9 =
    Rækkevidde: 5-11 km før målet
    Udløsning: 3.000-20.000 ft
employment-12 =
    Rækkevidde: 0,9-2,2 km før målet
    Udløsning: 100-500 ft
employment-16 = Rækkevidde: fotos fra 2-8 km (jo flere modstandere, jo tættere), næsen mod målet
employment-18 = Rækkevidde: udløs ved "Drop supplies now", inden for 4 km af nedkastningszonen
employment-20 =
    Rækkevidde: 4-8 km
    Affyring: 2.000-3.000 ft
employment-guided-cluster =
    Rækkevidde: fra 1,5 km til 2,5 gange højden, LOCK konstant (3,7 km fra 3.000 ft, 6 km fra 5.000 ft)
    Udløsning: 3.000-5.000 ft
employment-medium-retarded =
    Rækkevidde: 1,1-1,6 km før målet
    Udløsning: 300-1.000 ft
employment-low-retarded =
    Rækkevidde: 0,9-1,3 km før målet
    Udløsning: 200-500 ft

## Våbenpunkter

hardpoint-left-bay = Venstre skakt
hardpoint-right-bay = Højre skakt
hardpoint-left-wing-bay = Venstre vingeskakt
hardpoint-right-wing-bay = Højre vingeskakt
hardpoint-station-1-left-wingtip = Station 1, venstre vingespids
hardpoint-station-2-left-outer-wing = Station 2, venstre ydre vinge
hardpoint-station-3-left-wing = Station 3, venstre vinge
hardpoint-station-4-left-inner-wing = Station 4, venstre indre vinge
hardpoint-station-5-centreline = Station 5, midterlinje
hardpoint-station-6-right-inner-wing = Station 6, højre indre vinge
hardpoint-station-7-right-wing = Station 7, højre vinge
hardpoint-station-8-right-outer-wing = Station 8, højre ydre vinge
hardpoint-station-9-right-wingtip = Station 9, højre vingespids
hardpoint-station-1-left-wingtip-rail = Station 1, venstre vingespidsskinne
hardpoint-station-2-left-outer-pylon = Station 2, venstre ydre pylon
hardpoint-station-3-left-inner-pylon = Station 3, venstre indre pylon
hardpoint-station-4-right-inner-pylon = Station 4, højre indre pylon
hardpoint-station-5-right-outer-pylon = Station 5, højre ydre pylon
hardpoint-station-6-right-wingtip-rail = Station 6, højre vingespidsskinne
hardpoint-tornado-left-wing-rail = Venstre vinge, Sidewinder-skinne
hardpoint-tornado-left-wing-pylon = Venstre vingepylon
hardpoint-tornado-left-shoulder-forward = Venstre skulder, forrest
hardpoint-tornado-left-shoulder-aft = Venstre skulder, bagest
hardpoint-tornado-right-shoulder-aft = Højre skulder, bagest
hardpoint-tornado-right-shoulder-forward = Højre skulder, forrest
hardpoint-tornado-right-wing-pylon = Højre vingepylon
hardpoint-tornado-right-wing-rail = Højre vinge, Sidewinder-skinne
hardpoint-a10-1-left-outer-wing = Station 1, venstre ydre vinge
hardpoint-a10-2-left-wing = Station 2, venstre vinge
hardpoint-a10-3-left-wing = Station 3, venstre vinge
hardpoint-a10-4-left-centre-wing = Station 4, venstre midtervinge
hardpoint-a10-5-left-fuselage = Station 5, venstre skrog
hardpoint-a10-6-centreline = Station 6, midterlinje
hardpoint-a10-7-right-fuselage = Station 7, højre skrog
hardpoint-a10-8-right-centre-wing = Station 8, højre midtervinge
hardpoint-a10-9-right-wing = Station 9, højre vinge
hardpoint-a10-10-right-wing = Station 10, højre vinge
hardpoint-a10-11-right-outer-wing = Station 11, højre ydre vinge
weapon-role-21 = Air-to-air, infrared homing, very short range
employment-21 = Range: 1-6 km, farther head-on
