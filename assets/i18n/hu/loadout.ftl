# A fegyverzet-szerkesztő (src/loadout.rs). A fegyverek nevei az eredetiek, fordítatlanok.

loadout-none = Nincs
loadout-briefed = BRIEFING
    .facts = A küldetés briefingje szerinti felszerelés
loadout-clear = ÜRÍTÉS
    .facts = Minden fegyverfüggesztő üres
loadout-done = KÉSZ
loadout-no-hardpoints = Nincs fegyverfüggesztő
loadout-no-stores = Ez a gép nem visz felszerelést
loadout-fantasy = A valódi repülőgépen nincs ilyen
loadout-empty = Üres
loadout-out-of-stock = Nincs készleten ehhez a küldetéshez
loadout-mirror = Mindkét oldalon ugyanaz
    .facts = Ide is tölt: { $station }
loadout-stores = Felszerelés
loadout-needs-equipment = A küldetéshez kamera vagy különleges felszerelés kell

## Mire való az egyes fegyver (0-20. fegyvertípus)

# AIM-9M Sidewinder
weapon-role-0 = Levegő-levegő, infravörös önirányítás, kis hatótávolság
# AIM-120A AMRAAM
weapon-role-1 = Levegő-levegő, radaros önirányítás, közepes hatótávolság
# AGM-88A HARM
weapon-role-2 = Radarelhárító rakéta: radarok és SAM-állások
# Penguin 3 ASM
weapon-role-3 = Hajók elleni rakéta
# AGM-84A Harpoon
weapon-role-4 = Nagy hatótávolságú hajók elleni rakéta
# AGM-65 Maverick
weapon-role-5 = TV-vezérlésű rakéta: járművek, bunkerek, épületek
# GBU-12 Paveway
weapon-role-6 = Lézervezérlésű bomba: megerősített célpontok
# Mk 20 Rockeye
weapon-role-7 = Kazettás bomba: puha és kiterjedt célpontok
# Durandal
weapon-role-8 = Kifutópálya-romboló bomba
# Mk 82 LDGP
weapon-role-9 = 500 lb-s szabadon eső bomba
# Mk 82 Snakeye
weapon-role-10 = 500 lb-s fékezett bomba alacsony kioldáshoz
# Mk 20 Rockeye II
weapon-role-11 = Kazettás bomba: páncélosok és kiterjedt célpontok
# Mk 122 Fireye
weapon-role-12 = Gyújtóbomba
# CBU-72 FAE
weapon-role-13 = Üzemanyag-levegő robbanószer: kiterjedt célpontok
# Mk 35 incendiary
weapon-role-14 = Gyújtó kazettás bomba
# ISCB-1 mines
weapon-role-15 = Területlezáró aknák
# IR/optical camera
weapon-role-16 = Felderítő fotók a célpontról
# 1900 lb fuel tank
weapon-role-17 = Pótüzemanyag, ledobáskor átfejtve
# Special cargo
weapon-role-18 = Utánpótlás vagy rakomány a küldetéshez
# 20 mm cannon
weapon-role-19 = A gépágyú (nem felszerelés)
# AGM-114 Hellfire
weapon-role-20 = Lézervezérlésű páncélelhárító rakéta: oda repül, ahová a célzópod néz

## Az egyes fegyverek alkalmazása (hatótávolság a HUD km-eiben, magasság lábban)

employment-0 = Hatótávolság: 1-12 km, szemből nagyobb
employment-1 = Hatótávolság: 1-38 km, szemből nagyobb
employment-2 =
    Hatótávolság: 9-50 km
    Kilövés: 5 000-20 000 ft
employment-3 =
    Hatótávolság: 6-14 km (klasszikus rávezetéssel: 13-14 km)
    Kilövés: 100-500 ft
employment-4 =
    Hatótávolság: 6-41 km
    Kilövés: 2 000-5 000 ft
employment-5 =
    Hatótávolság: 5-10 km
    Kilövés: 2 000-5 000 ft
employment-6 =
    Hatótávolság: 1,5 km-től a magasság 2,5-szereséig, LOCK stabil (14 km 15 000 ft-ról, 18 km 25 000 ft-ról)
    Ledobás: 15 000-25 000 ft
employment-8 =
    Hatótávolság: 0,9-1,3 km a kifutópálya előtt
    Ledobás: 200-500 ft
employment-9 =
    Hatótávolság: 5-11 km a célpont előtt
    Ledobás: 3 000-20 000 ft
employment-12 =
    Hatótávolság: 0,9-2,2 km a célpont előtt
    Ledobás: 100-500 ft
employment-16 = Hatótávolság: fotók 2-8 km-ről (minél több az ellenfél, annál közelebbről), orral a célpont felé
employment-18 = Hatótávolság: ledobás a "Drop supplies now" üzenetnél, a ledobási zónától 4 km-en belül
employment-20 =
    Hatótávolság: 4-8 km
    Kilövés: 2 000-3 000 ft
employment-guided-cluster =
    Hatótávolság: 1,5 km-től a magasság 2,5-szereséig, LOCK stabil (3,7 km 3 000 ft-ról, 6 km 5 000 ft-ról)
    Ledobás: 3 000-5 000 ft
employment-medium-retarded =
    Hatótávolság: 1,1-1,6 km a célpont előtt
    Ledobás: 300-1 000 ft
employment-low-retarded =
    Hatótávolság: 0,9-1,3 km a célpont előtt
    Ledobás: 200-500 ft

## Fegyverfüggesztők

hardpoint-left-bay = Bal fegyverterem
hardpoint-right-bay = Jobb fegyverterem
hardpoint-left-wing-bay = Bal szárnyi fegyverterem
hardpoint-right-wing-bay = Jobb szárnyi fegyverterem
hardpoint-station-1-left-wingtip = 1. állomás, bal szárnyvég
hardpoint-station-2-left-outer-wing = 2. állomás, bal külső szárny
hardpoint-station-3-left-wing = 3. állomás, bal szárny
hardpoint-station-4-left-inner-wing = 4. állomás, bal belső szárny
hardpoint-station-5-centreline = 5. állomás, középvonal
hardpoint-station-6-right-inner-wing = 6. állomás, jobb belső szárny
hardpoint-station-7-right-wing = 7. állomás, jobb szárny
hardpoint-station-8-right-outer-wing = 8. állomás, jobb külső szárny
hardpoint-station-9-right-wingtip = 9. állomás, jobb szárnyvég
hardpoint-station-1-left-wingtip-rail = 1. állomás, bal szárnyvég-sín
hardpoint-station-2-left-outer-pylon = 2. állomás, bal külső pilon
hardpoint-station-3-left-inner-pylon = 3. állomás, bal belső pilon
hardpoint-station-4-right-inner-pylon = 4. állomás, jobb belső pilon
hardpoint-station-5-right-outer-pylon = 5. állomás, jobb külső pilon
hardpoint-station-6-right-wingtip-rail = 6. állomás, jobb szárnyvég-sín
hardpoint-tornado-left-wing-rail = Bal szárny, Sidewinder-sín
hardpoint-tornado-left-wing-pylon = Bal szárnypilon
hardpoint-tornado-left-shoulder-forward = Bal váll, elöl
hardpoint-tornado-left-shoulder-aft = Bal váll, hátul
hardpoint-tornado-right-shoulder-aft = Jobb váll, hátul
hardpoint-tornado-right-shoulder-forward = Jobb váll, elöl
hardpoint-tornado-right-wing-pylon = Jobb szárnypilon
hardpoint-tornado-right-wing-rail = Jobb szárny, Sidewinder-sín
hardpoint-a10-1-left-outer-wing = 1. állomás, bal külső szárny
hardpoint-a10-2-left-wing = 2. állomás, bal szárny
hardpoint-a10-3-left-wing = 3. állomás, bal szárny
hardpoint-a10-4-left-centre-wing = 4. állomás, bal középső szárny
hardpoint-a10-5-left-fuselage = 5. állomás, bal törzs
hardpoint-a10-6-centreline = 6. állomás, középvonal
hardpoint-a10-7-right-fuselage = 7. állomás, jobb törzs
hardpoint-a10-8-right-centre-wing = 8. állomás, jobb középső szárny
hardpoint-a10-9-right-wing = 9. állomás, jobb szárny
hardpoint-a10-10-right-wing = 10. állomás, jobb szárny
hardpoint-a10-11-right-outer-wing = 11. állomás, jobb külső szárny
