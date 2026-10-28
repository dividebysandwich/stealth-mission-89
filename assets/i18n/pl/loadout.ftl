# Edytor uzbrojenia (src/loadout.rs). Nazwy uzbrojenia są prawdziwe, nietłumaczone.

loadout-none = Brak
loadout-briefed = Z ODPRAWY
    .facts = Podwieszenia przewidziane w odprawie misji
loadout-clear = OPRÓŻNIĆ
    .facts = Wszystkie punkty podwieszenia puste
loadout-done = GOTOWE
loadout-no-hardpoints = Brak punktów podwieszenia
loadout-no-stores = Ta maszyna nie przenosi podwieszeń
loadout-fantasy = Nie występuje w prawdziwym samolocie
loadout-empty = Pusty
loadout-out-of-stock = Brak w zapasie na tę misję
loadout-mirror = Tak samo po obu stronach
    .facts = Ładuje również: { $station }
loadout-stores = Podwieszenia
loadout-needs-equipment = Misja wymaga swojej kamery lub wyposażenia specjalnego

## Do czego służy każda broń

# AIM-9M Sidewinder
weapon-role-0 = Powietrze-powietrze, naprowadzanie na podczerwień, krótki zasięg
# AIM-120A AMRAAM
weapon-role-1 = Powietrze-powietrze, naprowadzanie radarowe, średni zasięg
# AGM-88A HARM
weapon-role-2 = Pocisk przeciwradarowy: radary i stanowiska SAM
# Penguin 3 ASM
weapon-role-3 = Pocisk przeciwokrętowy
# AGM-84A Harpoon
weapon-role-4 = Pocisk przeciwokrętowy dalekiego zasięgu
# AGM-65 Maverick
weapon-role-5 = Pocisk naprowadzany telewizyjnie: pojazdy, bunkry, budynki
# GBU-12 Paveway
weapon-role-6 = Bomba naprowadzana laserowo: cele umocnione
# Mk 20 Rockeye
weapon-role-7 = Bomba kasetowa: cele nieopancerzone i powierzchniowe
# Durandal
weapon-role-8 = Bomba do niszczenia pasów startowych
# Mk 82 LDGP
weapon-role-9 = Bomba swobodnie spadająca 500 lb
# Mk 82 Snakeye
weapon-role-10 = Bomba hamowana 500 lb do zrzutów z małej wysokości
# Mk 20 Rockeye II
weapon-role-11 = Bomba kasetowa: pojazdy opancerzone i cele powierzchniowe
# Mk 122 Fireye
weapon-role-12 = Bomba zapalająca
# CBU-72 FAE
weapon-role-13 = Ładunek paliwowo-powietrzny: cele powierzchniowe
# Mk 35 incendiary
weapon-role-14 = Zapalająca bomba kasetowa
# ISCB-1 mines
weapon-role-15 = Miny do blokowania terenu
# IR/optical camera
weapon-role-16 = Zdjęcia rozpoznawcze celu
# 1900 lb fuel tank
weapon-role-17 = Dodatkowe paliwo, przepompowywane przy zrzucie
# Special cargo
weapon-role-18 = Zaopatrzenie lub ładunek dla misji
# 20 mm cannon
weapon-role-19 = Działko (nie jest podwieszeniem)
# AGM-114 Hellfire
weapon-role-20 = Przeciwpancerny pocisk naprowadzany laserowo: leci tam, gdzie patrzy zasobnik celowniczy

## Użycie każdej broni (odległości w km jak na HUD, wysokości w stopach)

employment-0 = Zasięg: 1-12 km, większy przy ataku z przedniej półsfery
employment-1 = Zasięg: 1-38 km, większy przy ataku z przedniej półsfery
employment-2 =
    Zasięg: 9-50 km
    Odpalenie: 5 000-20 000 ft
employment-3 =
    Zasięg: 6-14 km (klasyczne naprowadzanie: 13-14 km)
    Odpalenie: 100-500 ft
employment-4 =
    Zasięg: 6-41 km
    Odpalenie: 2 000-5 000 ft
employment-5 =
    Zasięg: 5-10 km
    Odpalenie: 2 000-5 000 ft
employment-6 =
    Zasięg: od 1,5 km do 2,5-krotności wysokości, LOCK stale (14 km z 15 000 ft, 18 km z 25 000 ft)
    Zrzut: 15 000-25 000 ft
employment-8 =
    Zasięg: 0,9-1,3 km przed pasem
    Zrzut: 200-500 ft
employment-9 =
    Zasięg: 5-11 km przed celem
    Zrzut: 3 000-20 000 ft
employment-12 =
    Zasięg: 0,9-2,2 km przed celem
    Zrzut: 100-500 ft
employment-16 = Zasięg: zdjęcia z 2-8 km (im więcej przeciwników, tym bliżej), nos na cel
employment-18 = Zasięg: zrzut na „Drop supplies now”, w promieniu 4 km od strefy zrzutu
employment-20 =
    Zasięg: 4-8 km
    Odpalenie: 2 000-3 000 ft
employment-guided-cluster =
    Zasięg: od 1,5 km do 2,5-krotności wysokości, LOCK stale (3,7 km z 3 000 ft, 6 km z 5 000 ft)
    Zrzut: 3 000-5 000 ft
employment-medium-retarded =
    Zasięg: 1,1-1,6 km przed celem
    Zrzut: 300-1 000 ft
employment-low-retarded =
    Zasięg: 0,9-1,3 km przed celem
    Zrzut: 200-500 ft

## Punkty podwieszenia

hardpoint-left-bay = Lewa komora
hardpoint-right-bay = Prawa komora
hardpoint-left-wing-bay = Komora lewego skrzydła
hardpoint-right-wing-bay = Komora prawego skrzydła
hardpoint-station-1-left-wingtip = Stacja 1, koniec lewego skrzydła
hardpoint-station-2-left-outer-wing = Stacja 2, lewe skrzydło zewnętrzna
hardpoint-station-3-left-wing = Stacja 3, lewe skrzydło
hardpoint-station-4-left-inner-wing = Stacja 4, lewe skrzydło wewnętrzna
hardpoint-station-5-centreline = Stacja 5, podkadłubowa
hardpoint-station-6-right-inner-wing = Stacja 6, prawe skrzydło wewnętrzna
hardpoint-station-7-right-wing = Stacja 7, prawe skrzydło
hardpoint-station-8-right-outer-wing = Stacja 8, prawe skrzydło zewnętrzna
hardpoint-station-9-right-wingtip = Stacja 9, koniec prawego skrzydła
hardpoint-station-1-left-wingtip-rail = Stacja 1, prowadnica na końcu lewego skrzydła
hardpoint-station-2-left-outer-pylon = Stacja 2, lewy zewnętrzny pylon
hardpoint-station-3-left-inner-pylon = Stacja 3, lewy wewnętrzny pylon
hardpoint-station-4-right-inner-pylon = Stacja 4, prawy wewnętrzny pylon
hardpoint-station-5-right-outer-pylon = Stacja 5, prawy zewnętrzny pylon
hardpoint-station-6-right-wingtip-rail = Stacja 6, prowadnica na końcu prawego skrzydła
hardpoint-tornado-left-wing-rail = Lewe skrzydło, prowadnica Sidewinder
hardpoint-tornado-left-wing-pylon = Pylon lewego skrzydła
hardpoint-tornado-left-shoulder-forward = Lewe ramię, przód
hardpoint-tornado-left-shoulder-aft = Lewe ramię, tył
hardpoint-tornado-right-shoulder-aft = Prawe ramię, tył
hardpoint-tornado-right-shoulder-forward = Prawe ramię, przód
hardpoint-tornado-right-wing-pylon = Pylon prawego skrzydła
hardpoint-tornado-right-wing-rail = Prawe skrzydło, prowadnica Sidewinder
hardpoint-a10-1-left-outer-wing = Stacja 1, lewe skrzydło zewnętrzna
hardpoint-a10-2-left-wing = Stacja 2, lewe skrzydło
hardpoint-a10-3-left-wing = Stacja 3, lewe skrzydło
hardpoint-a10-4-left-centre-wing = Stacja 4, lewe skrzydło środkowa
hardpoint-a10-5-left-fuselage = Stacja 5, lewa strona kadłuba
hardpoint-a10-6-centreline = Stacja 6, podkadłubowa
hardpoint-a10-7-right-fuselage = Stacja 7, prawa strona kadłuba
hardpoint-a10-8-right-centre-wing = Stacja 8, prawe skrzydło środkowa
hardpoint-a10-9-right-wing = Stacja 9, prawe skrzydło
hardpoint-a10-10-right-wing = Stacja 10, prawe skrzydło
hardpoint-a10-11-right-outer-wing = Stacja 11, prawe skrzydło zewnętrzna
weapon-role-21 = Air-to-air, infrared homing, very short range
employment-21 = Range: 1-6 km, farther head-on
