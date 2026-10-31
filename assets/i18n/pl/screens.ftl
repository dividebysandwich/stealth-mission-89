# Kneeboard (src/kneeboard.rs, czcionka wektorowa), planowanie misji (src/planner.rs) i zapisane
# misje (src/savegame.rs). Skróty nawigacyjne (PT, BRG, ETA, PRI, SEC, KM, KT) pozostają po
# angielsku.

kb-brief = BRIEF
kb-route = TRASA
kb-map = MAPA
kb-target = CEL
kb-keys = KLAWISZE
kb-tip = Mapy, notatki z misji i przewodnik po klawiszach
kb-geran-file = GERAN-2 (ARCHIWUM)
kb-objectives = CELE
kb-primary = GŁÓWNY: { $objective }
kb-secondary = DRUGORZĘDNY: { $objective }
kb-times = CZASY
kb-deadline = TERMIN { $time }
kb-now = TERAZ { $time }
kb-conditions = WARUNKI
kb-moon = KSIĘŻYC: { $phase } ({ $lit }% OŚWIETLENIA)
kb-armament = UZBROJENIE
kb-roe = ZASADY UŻYCIA SIŁY
kb-total = RAZEM { $km } KM PRZY { $kt } KT
kb-fuel = PALIWO { $lbs } LBS
kb-preparing = PRZYGOTOWANIE...
kb-airborne = W POWIETRZU
kb-developing = WYWOŁYWANIE...
kb-no-photo = BRAK ZDJĘCIA
kb-airborne-target = CEL W POWIETRZU
kb-scroll-vr = PRZEWIJANIE: GAŁKA, PALEC NA KNEEBOARDZIE
kb-scroll-desktop = PRZEWIJANIE: KÓŁKO MYSZY NAD KNEEBOARDEM

## Planowanie misji

planner-low = NISKO 500 FT
planner-medium = ŚREDNIO 5000 FT
planner-high = WYSOKO 20000 FT
planner-title = PLANOWANIE MISJI
planner-route = Trasa
planner-base = BAZA
planner-total = RAZEM { $km } KM  { $time }
planner-briefed = Trasa z odprawy { $km } km, paliwo { $fuel } lbs
planner-coverage = Pokrycie radarowe dla
    .note = Pokrycie radarowe zakłada lot poziomy z prędkością 480 kt i zamkniętymi komorami uzbrojenia.
planner-detected = Wykryty odcinek trasy: { $km } km
planner-pulse = Zasięg wykrywania radaru impulsowego (P)
planner-doppler = Zasięg wykrywania radaru dopplerowskiego (D)
planner-missile = Zasięg rakiet
planner-uav = Wrogi dron
planner-edit-hint = Przeciągnij punkt, aby go przesunąć, lub mapę, aby ją przewinąć. Kliknij odcinek, aby dodać punkt, prawym przyciskiem, aby go usunąć. Kółko: zoom.
planner-reset = ZRESETOWAĆ
planner-view-hint = Przeciągnij mapę, aby ją przewinąć. Kółko: zoom.
planner-layers = WARSTWY
planner-layer-sams = Radar / stanowisko SAM
planner-radar-types = Typy radarów
planner-layer-pulse = Zasięg wykrywania radaru impulsowego
planner-layer-doppler = Zasięg wykrywania radaru dopplerowskiego
planner-layer-airfield = Lotnisko

## Zapisane misje

saves-title = ZAPISANE MISJE
saves-subtitle = Zapisywane w locie z menu: ZAPISAĆ MISJĘ
saves-none = Brak zapisanych misji
saves-line = { $theater }  -  { $aircraft }  -  czas misji { $time }
saves-saved-at = Zapisano { $date }
saves-unreadable = Nieczytelny zapis (inna wersja)
saves-delete =
    .facts = Usunąć ten zapis
saves-back =
    .facts = Powrót do ekranu pojedynczej misji
