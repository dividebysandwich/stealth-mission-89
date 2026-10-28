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
sim-bay-closed = Komora zamknięta
sim-insufficient-altitude = Zbyt mała wysokość
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
sim-visual-id = Identyfikacja wzrokowa - { $aircraft }
sim-fired-by = { $missile } - odpalenie: { $launcher }
sim-frame = Klatka { $frame }
sim-beacon = Radiolatarnia, namiar { $bearing }
sim-fired = { $weapon } - odpalenie
sim-released = { $weapon } - zrzut
sim-destroyed-gunfire = Zniszczono z działka: { $target }
