# Der Bewaffnungseditor (src/loadout.rs). Die Waffennamen sind die echten, nicht übersetzt.

loadout-none = Keine
loadout-briefed = BRIEFING
    .facts = Die Außenlasten aus dem Missionsbriefing
loadout-clear = LEEREN
    .facts = Alle Aufhängepunkte leer
loadout-done = FERTIG
loadout-no-hardpoints = Keine Aufhängepunkte
loadout-no-stores = Dieses Fluggerät trägt keine Außenlasten
loadout-fantasy = Nicht am echten Flugzeug
loadout-empty = Leer
loadout-out-of-stock = Für diese Mission nicht vorrätig
loadout-mirror = Gleich auf beiden Seiten
    .facts = Belädt auch { $station }
loadout-stores = Außenlasten
loadout-needs-equipment = Die Mission braucht ihre Kamera oder Sonderausrüstung

## Wofür jede Waffe dient (Waffentypen 0 bis 20)

# AIM-9M Sidewinder
weapon-role-0 = Luft-Luft, Infrarotsuchkopf, kurze Reichweite
# AIM-120A AMRAAM
weapon-role-1 = Luft-Luft, Radarsuchkopf, mittlere Reichweite
# AGM-88A HARM
weapon-role-2 = Radarbekämpfungsrakete: Radare und SAM-Stellungen
# Penguin 3 ASM
weapon-role-3 = Seezielflugkörper
# AGM-84A Harpoon
weapon-role-4 = Seezielflugkörper großer Reichweite
# AGM-65 Maverick
weapon-role-5 = TV-gelenkte Rakete: Fahrzeuge, Bunker, Gebäude
# GBU-12 Paveway
weapon-role-6 = Lasergelenkte Bombe: harte Ziele
# Mk 20 Rockeye
weapon-role-7 = Streubombe: weiche Ziele und Flächenziele
# Durandal
weapon-role-8 = Bombe gegen Start- und Landebahnen
# Mk 82 LDGP
weapon-role-9 = 500-lb-Freifallbombe
# Mk 82 Snakeye
weapon-role-10 = 500-lb-Verzögerungsbombe für Abwürfe aus geringer Höhe
# Mk 20 Rockeye II
weapon-role-11 = Streubombe: Panzer und Flächenziele
# Mk 122 Fireye
weapon-role-12 = Brandbombe
# CBU-72 FAE
weapon-role-13 = Aerosolbombe: Flächenziele
# Mk 35 incendiary
weapon-role-14 = Brand-Streubombe
# ISCB-1 mines
weapon-role-15 = Minen zur Gebietssperrung
# IR/optical camera
weapon-role-16 = Aufklärungsfotos des Ziels
# 1900 lb fuel tank
weapon-role-17 = Zusatztreibstoff, beim Abwurf übernommen
# Special cargo
weapon-role-18 = Nachschub oder Fracht für die Mission
# 20 mm cannon
weapon-role-19 = Die Bordkanone (keine Außenlast)
# AGM-114 Hellfire
weapon-role-20 = Lasergelenkte Panzerabwehrrakete: fliegt dorthin, wohin der Zielbehälter blickt

## Wie jede Waffe eingesetzt wird (Entfernungen in den km des HUD, Höhen in Fuß)

employment-0 = Reichweite: 1-12 km, frontal weiter
employment-1 = Reichweite: 1-38 km, frontal weiter
employment-2 =
    Reichweite: 9-50 km
    Abschuss: 5.000-20.000 ft
employment-3 =
    Reichweite: 6-14 km (klassische Lenkung: 13-14 km)
    Abschuss: 100-500 ft
employment-4 =
    Reichweite: 6-41 km
    Abschuss: 2.000-5.000 ft
employment-5 =
    Reichweite: 5-10 km
    Abschuss: 2.000-5.000 ft
employment-6 =
    Reichweite: 1,5 km bis 2,5-mal die Höhe, LOCK stetig (14 km aus 15.000 ft, 18 km aus 25.000 ft)
    Abwurf: 15.000-25.000 ft
employment-8 =
    Reichweite: 0,9-1,3 km vor der Startbahn
    Abwurf: 200-500 ft
employment-9 =
    Reichweite: 5-11 km vor dem Ziel
    Abwurf: 3.000-20.000 ft
employment-12 =
    Reichweite: 0,9-2,2 km vor dem Ziel
    Abwurf: 100-500 ft
employment-16 = Reichweite: Fotos aus 2-8 km (je mehr Gegner, desto näher), Nase auf das Ziel
employment-18 = Reichweite: bei „Drop supplies now“ abwerfen, höchstens 4 km von der Absetzzone
employment-20 =
    Reichweite: 4-8 km
    Abschuss: 2.000-3.000 ft
employment-guided-cluster =
    Reichweite: 1,5 km bis 2,5-mal die Höhe, LOCK stetig (3,7 km aus 3.000 ft, 6 km aus 5.000 ft)
    Abwurf: 3.000-5.000 ft
employment-medium-retarded =
    Reichweite: 1,1-1,6 km vor dem Ziel
    Abwurf: 300-1.000 ft
employment-low-retarded =
    Reichweite: 0,9-1,3 km vor dem Ziel
    Abwurf: 200-500 ft

## Aufhängepunkte

hardpoint-left-bay = Linker Schacht
hardpoint-right-bay = Rechter Schacht
hardpoint-left-wing-bay = Linker Flügelschacht
hardpoint-right-wing-bay = Rechter Flügelschacht
hardpoint-station-1-left-wingtip = Station 1, linke Flügelspitze
hardpoint-station-2-left-outer-wing = Station 2, linker Außenflügel
hardpoint-station-3-left-wing = Station 3, linker Flügel
hardpoint-station-4-left-inner-wing = Station 4, linker Innenflügel
hardpoint-station-5-centreline = Station 5, Rumpfmitte
hardpoint-station-6-right-inner-wing = Station 6, rechter Innenflügel
hardpoint-station-7-right-wing = Station 7, rechter Flügel
hardpoint-station-8-right-outer-wing = Station 8, rechter Außenflügel
hardpoint-station-9-right-wingtip = Station 9, rechte Flügelspitze
hardpoint-station-1-left-wingtip-rail = Station 1, Startschiene linke Flügelspitze
hardpoint-station-2-left-outer-pylon = Station 2, linker Außenpylon
hardpoint-station-3-left-inner-pylon = Station 3, linker Innenpylon
hardpoint-station-4-right-inner-pylon = Station 4, rechter Innenpylon
hardpoint-station-5-right-outer-pylon = Station 5, rechter Außenpylon
hardpoint-station-6-right-wingtip-rail = Station 6, Startschiene rechte Flügelspitze
hardpoint-tornado-left-wing-rail = Linker Flügel, Sidewinder-Schiene
hardpoint-tornado-left-wing-pylon = Pylon am linken Flügel
hardpoint-tornado-left-shoulder-forward = Linke Rumpfschulter, vorn
hardpoint-tornado-left-shoulder-aft = Linke Rumpfschulter, hinten
hardpoint-tornado-right-shoulder-aft = Rechte Rumpfschulter, hinten
hardpoint-tornado-right-shoulder-forward = Rechte Rumpfschulter, vorn
hardpoint-tornado-right-wing-pylon = Pylon am rechten Flügel
hardpoint-tornado-right-wing-rail = Rechter Flügel, Sidewinder-Schiene
hardpoint-a10-1-left-outer-wing = Station 1, linker Außenflügel
hardpoint-a10-2-left-wing = Station 2, linker Flügel
hardpoint-a10-3-left-wing = Station 3, linker Flügel
hardpoint-a10-4-left-centre-wing = Station 4, linker mittlerer Flügel
hardpoint-a10-5-left-fuselage = Station 5, linker Rumpf
hardpoint-a10-6-centreline = Station 6, Rumpfmitte
hardpoint-a10-7-right-fuselage = Station 7, rechter Rumpf
hardpoint-a10-8-right-centre-wing = Station 8, rechter mittlerer Flügel
hardpoint-a10-9-right-wing = Station 9, rechter Flügel
hardpoint-a10-10-right-wing = Station 10, rechter Flügel
hardpoint-a10-11-right-outer-wing = Station 11, rechter Außenflügel
