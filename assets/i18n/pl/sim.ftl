# Komunikaty HUD symulacji (crates/stealth-sim). Wyświetlane czcionkami lotu: wielkimi literami,
# bez znaków diakrytycznych (języki łacińskie; pozostałe widzą wersję angielską). Nazwy
# jednostek, uzbrojenia i miejsc są przekazywane bez zmian.

sim-missile-launch = Odpalenie rakiety
sim-no-flares = Brak rac sygnałowych
sim-hind-spotted = Wykryty przez Hind
sim-rescue-lost = Śmigłowiec ratowniczy stracony
sim-rescued = Uratowany!
sim-captured = Pojmany
sim-aircraft-destroyed-eject = Samolot zniszczony - katapultuj się
sim-fire-out = Pożar ugaszony
sim-aircraft-exploded = Samolot eksplodował
sim-equipment-delivered = Sprzęt dostarczony
sim-bad-drop = Nieudany zrzut
sim-secondary-target = Cel drugorz.
sim-primary-target = Cel główny
sim-troops-inserted = Oddział wysadzony
sim-time-expired = Czas minął
sim-cargo-delivered = Ładunek dostarczony
sim-safe-landing = Bezpieczne lądowanie
sim-no-target-ahead = Brak celu z przodu
sim-target-obscured = Cel zasłonięty
sim-target-photographed = Cel sfotografowany.
sim-drop-supplies = Zrzuć zaopatrzenie
sim-no-target = Brak celu
sim-no-target-designated = Nie wskazano celu
sim-slam-channels-busy = Wszystkie kanały SLAM-ER zajęte
sim-jsow-out-of-range = JSOW: cel poza zasięgiem
sim-jsow-too-close = JSOW: cel za blisko
sim-bay-closed = Komora zamknięta
sim-ramp-closed = Rampa zamknięta
sim-insufficient-altitude = Zbyt mała wysokość
sim-below-burst-altitude = Poniżej wysokości otwarcia ({ $feet } ft)
sim-fuel-transfer = Przetaczanie paliwa
sim-stores-exhausted = Uzbrojenie wyczerpane
sim-hit-gunfire = Trafiony ogniem działek
sim-hit-aaa = Trafiony przez AAA
sim-hit-flak = Trafiony przez artylerię plot.
sim-destroyed = Zniszczono: { $target }
sim-landing = { $base } - { $aircraft } ląduje
sim-on-final = { $base } - { $aircraft } na podejściu końcowym
sim-taking-off = { $base } - { $aircraft } startuje
sim-shot-down = Zestrzelono: { $target }
sim-lost = Utracono: { $target }
sim-uav-hit = { $weapon } trafił: { $target }
sim-locked-on = { $weapon } namierzył: { $target }
sim-lost-lock = { $weapon } stracił namiar
sim-csar-down = Na ziemi na terytorium wroga - CSAR z bazy { $base }
sim-flare-left = Raca sygnałowa - pozostało: { $count }
sim-drone-hit-npp = Dron trafił w elektrownię jądrową { $place }
sim-drone-hit = Trafienie drona: { $place }
sim-defense-failed = Obrona przełamana - { $count ->
    [one] przedarł się { $count } dron
    [few] przedarły się { $count } drony
    [many] przedarło się { $count } dronów
   *[other] przedarło się { $count } drona
}
sim-hit-by = Trafienie: { $weapon }
sim-target-hit-by = { $target } - trafienie: { $weapon }
sim-ineffective = Bez efektu
sim-impact = { $weapon } - uderzenie
# Spacja na końcu pochodzi z klasycznej gry.
sim-misses = { $weapon } - chybienie{" "}
sim-destroyed-by = Zniszczono: { $target } ({ $weapon })
sim-ground-impact = { $weapon } - uderzenie w ziemię
sim-detected = Wykryto: { $target }
sim-radar-id = { $site } - identyfikacja radarowa
sim-firing = { $site } odpala { $missile }
sim-firing-at-standoff = { $site } odpala { $missile } w broń dystansową
sim-standoff-shot-down = { $weapon } zestrzelony
sim-visual-id = Identyfikacja wzrokowa - { $aircraft }
sim-fired-by = { $missile } - odpalenie: { $launcher }
sim-frame = Klatka { $frame }
sim-beacon = Radiolatarnia, namiar { $bearing }
sim-fired = { $weapon } - odpalenie
sim-released = { $weapon } - zrzut
sim-destroyed-gunfire = Zniszczono z działka: { $target }
sim-gun-selected = { $gun } wybrane
sim-gun-gimbal = Działa nie sięgają: leć lewym kręgiem
sim-gun-no-pod = Działa wymagają zasobnika celowniczego na ziemi
sim-gun-empty = { $gun } brak amunicji
sim-select-aa = Wybierz tryb A-A
sim-select-ag = Wybierz tryb A-G
sim-select-aa-ag = Wybierz tryb A-A lub A-G
asw-not-seahawk = Nie jesteś w załodze Seahawka
asw-no-buoys = Brak pław { $kind }
asw-buoys-live = Już { $n } pławy w wodzie
asw-route-full = Trasa jest pełna
asw-no-such-buoy = Nie ma takiej pławy
asw-dip-broken = Sonar opuszczany utracony
asw-dip-limits = Opuszczanie: zawis poniżej 70 ft, poniżej 5 kt
asw-hover-request = Operator: zawis do opuszczenia sonaru, 40 ft nad morzem
asw-hover-cancelled = Operator: zawis do opuszczenia sonaru już niepotrzebny
asw-not-over-water = Nie nad wodą
asw-mad-limits = MAD: zwolnij poniżej 120 kt
asw-not-armed = Zrzut torpedy nieuzbrojony
asw-no-torpedo = Brak torpedy na tym węźle
asw-torpedo-away = Torpeda zrzucona
asw-torpedo-limits = Torpeda: poniżej 300 ft i 100 kt
asw-sub-sunk = Okręt podwodny zniszczony
asw-sub-escaped = Okręt podwodny przedarł się przez blokadę
asw-ship-torpedoed = Okręt został storpedowany
asw-layer = Głębokość warstwy { $m } m
asw-cable-parted = Kabel sonaru zerwany
asw-mad-contact = Kontakt MAD: oznaczony
