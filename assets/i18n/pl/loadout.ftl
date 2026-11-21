# Edytor uzbrojenia (src/loadout.rs). Nazwy uzbrojenia są prawdziwe, nietłumaczone.

loadout-none = Brak
loadout-briefed = Z ODPRAWY
    .facts = Podwieszenia przewidziane w odprawie misji
loadout-own = Twoje własne uzbrojenie: wróci w kolejnych misjach tego rodzaju na tym samolocie. { loadout-briefed } przywraca uzbrojenie misji.
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
loadout-realistic-f117 = Realistyczne uzbrojenie F-117A
loadout-stations = Węzły podwieszeń
loadout-click-drawing = Lub kliknij węzeł na rysunku
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
hardpoint-av8b-1-left-rail = Stacja 1, lewa prowadnica Sidewindera
hardpoint-av8b-3-left-middle-wing = Stacja 3, lewe skrzydło, środek
hardpoint-av8b-4-left-inboard-wing = Stacja 4, lewe skrzydło, wewnątrz
hardpoint-av8b-6-right-inboard-wing = Stacja 6, prawe skrzydło, wewnątrz
hardpoint-av8b-7-right-middle-wing = Stacja 7, prawe skrzydło, środek
hardpoint-av8b-9-right-rail = Stacja 9, prawa prowadnica Sidewindera
hardpoint-f22-left-side-bay = Lewa komora boczna (Sidewinder)
hardpoint-f22-left-main-bay = Komora główna, lewa połowa
hardpoint-f22-right-main-bay = Komora główna, prawa połowa
hardpoint-f22-right-side-bay = Prawa komora boczna (Sidewinder)
hardpoint-fa18e-8-right-inner-wing = Stacja 8, prawe skrzydło wewnętrzna
hardpoint-fa18e-10-right-outer-wing = Stacja 10, prawe skrzydło zewnętrzna
hardpoint-fa18e-11-right-wingtip = Stacja 11, koniec prawego skrzydła
hardpoint-c5-hold = Ładownia (palety zrzutowe)
hardpoint-ch47-cabin = Kabina (spadochroniarze)
weapon-role-21 = Powietrze-powietrze, naprowadzanie na podczerwień, bardzo krótki zasięg
employment-21 = Zasięg: 1-6 km, większy czołowo
loadout-none-in-stock = (brak w magazynie)
loadout-empties = Opróżnia stację { $stations } po załadowaniu
loadout-empties-two = Opróżnia stacje { $first } i { $second } po załadowaniu
hardpoint-oh58-left-pylon = Lewy pylon uzbrojenia
hardpoint-oh58-right-pylon = Prawy pylon uzbrojenia
hardpoint-mh60r-left-outer = Lewy zewnętrzny węzeł (wyrzutnia Hellfire)
hardpoint-mh60r-left-inner = Lewy wewnętrzny węzeł
hardpoint-mh60r-right-inner = Prawy wewnętrzny węzeł
weapon-role-22 = Gondola karabinu maszynowego: strzela jak działko, wzdłuż dziobu
employment-22 = Zasięg: do 1,5 km, 500 nabojów w gondoli
weapon-role-23 = Broń z zapalnikiem sensorowym: czterdzieści skeetów strzela w dół w pojazdy
employment-23 =
    Naprowadzana na punkt oznaczony zasobnikiem celowniczym przy zrzucie
    Zrzut: powyżej wysokości otwarcia ustawionej na stronie SMS (500-3000 ft)
weapon-role-24 = Lekka torpeda: opada na spadochronie, szuka pod wodą i naprowadza się na okręt podwodny
employment-24 =
    Programowana na stronie WPN operatora sensorów: kurs, opóźnienie aktywacji, wzór poszukiwania
    Zrzut: nad wodą, poniżej 300 ft i 100 kt
weapon-role-25 = Pocisk do atakowania celów naziemnych z dystansu: leci do celu, a potem pilot naprowadza go według obrazu z głowicy
employment-25 =
    Programowany na stronie SLAM: wysokość przelotowa, odległość włączenia głowicy, cel główny lub zapasowy
    Zasięg: do 250 km, cztery w locie jednocześnie
weapon-role-26 = Bomba szybująca: szybuje do wskazanego punktu; zasięg rośnie z wysokością i prędkością zrzutu
employment-26 =
    Celuje w punkt zasobnika lub namiar naziemny, inaczej w cel główny
    Zasięg: ok. 20 km nisko, do 130 km z dużej wysokości; obrona przeciwlotnicza może ją zestrzelić
weapon-role-27 = Wabik zrzucany z powietrza: wlatuje w obronę przeciwlotniczą z sygnaturą samolotu i ściąga jej pociski
employment-27 =
    Leci do wskazanego punktu, potem prosto do 10 minut
    Nieszkodliwy dla celów
weapon-role-28 = Spadochroniarze: grupa żołnierzy zrzucana nad miejscem, które zajmą
employment-28 =
    Przez drzwi C-5 (dwie grupy) lub rampę CH-47 (jedna grupa) nad strefą zrzutu
    Skok od 500 do 1500 ft nad ziemią, poniżej 150 kt; większość grupy musi wylądować w strefie
weapon-role-29 = Taktyczny zasobnik zakłóceń: ogranicza zakłócane radary do ich zasięgu przebicia
employment-29 =
    Ustawiany na stronie JAM: radar lub sektor dla każdego zasobnika, wł. lub wył.
    Zakłócanie sprawia, że widać cię z większej odległości
weapon-role-30 = Zasobnik do ataku na lotniska: rozrzuca na spadochronikach bombki kraterujące i miny przeciwpancerne w pasie 500 m
employment-30 =
    Zostaje na pylonie i wyrzuca przez dwie sekundy; leć poziomo i nisko nad celem
    Z wysokości nie większej niż 1000 ft nad ziemią; trzy kratery zamykają pas, miny czekają na pojazdy
weapon-role-31 = Niekierowane rakiety 70 mm w wyrzutni: parami lub salwami przeciw pojazdom i piechocie w terenie odkrytym
employment-31 =
    Celuj celownikiem rakietowym (CCIP): pokazuje, gdzie spadnie rakieta odpalona teraz
    Naciśnij, by odpalić parę, przytrzymaj dla salwy; najlepiej z płytkiego nurkowania z mniej niż 4 km
weapon-role-32 = Rakiety 70 mm naprowadzane laserowo: każda leci na plamkę lasera zasobnika celowniczego
employment-32 =
    Nakieruj zasobnik na cel i strzelaj wzdłuż nosa; skręcą na plamkę
    Do 6 km; trzymaj zasobnik na celu aż do trafienia
loadout-wingman = SKRZYDŁOWY
    .facts = Skrzydłowy SI w tym samym samolocie i z tym samym uzbrojeniem dołącza w powietrzu; menu radiowe (\) wydaje mu rozkazy
