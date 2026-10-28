# Omówienie misji (src/debrief.rs) i raport symulacji (stealth-sim score.rs).

## Jak zakończył się lot

outcome-landed = Bezpieczne lądowanie
outcome-bad-landing = Nieudane lądowanie
outcome-ejected = Katapultowanie - lądowanie na spadochronie
outcome-rescued = Katapultowanie - uratowany przez CSAR
outcome-captured = Katapultowanie - pojmany
outcome-crashed = Rozbity
outcome-ejection-failed = Nieudane katapultowanie
outcome-aborted = Misja przerwana
outcome-collision = Zderzenie

## Oceny

rating-unsatisfactory = Niedostateczna
rating-marginal = Mierna
rating-satisfactory = Dostateczna
rating-excellent = Bardzo dobra
rating-outstanding = Celująca

## Punkty

points-primary = Cel główny
points-secondary = Cel drugorzędny
points-aircraft = Zniszczone samoloty
points-unseen = Zestrzelenia bez wykrycia
points-ground = Zniszczone cele naziemne
points-drones = Zestrzelone drony
points-drones-through = Drony, które się przedarły
points-nuclear = Trafienie w elektrownię jądrową
points-detected = Wykrycie
points-missile-hits = Trafienia rakietami

## Przebieg (nazwy miejsc i samolotów jak w misji)

timeline-primary = {" "}(główny)
timeline-secondary = {" "}(drugorzędny)
timeline-destroyed = Zniszczono: { $place }{ $tag }
timeline-destroyed-ground = Zniszczono cel naziemny{ $tag }
timeline-shot-down = Zestrzelono: { $aircraft }{ $tag }
timeline-hit = Trafienie rakietą
timeline-seen = Wykryty przez { $aircraft }
timeline-radar-id = Identyfikacja radarowa: { $place }
timeline-objective = Cel osiągnięty{ $tag }
timeline-got-through = { $aircraft } przedarł się
timeline-photo = Zdjęcie wykonane{ $tag }

## Okno omówienia misji

debrief-title = OMÓWIENIE MISJI
debrief-training = MISJA TRENINGOWA
debrief-results = WYNIKI
debrief-flight-map = MAPA LOTU
debrief-section-results = Wyniki
debrief-accomplished = WYKONANY
debrief-failed = NIEWYKONANY
debrief-primary = GŁÓWNY
debrief-secondary = DRUGORZĘDNY
debrief-aircraft = SAMOLOTY
debrief-drones = DRONY
debrief-drones-value = zestrzelone: { $down }, przedarły się: { $through }
debrief-ground = CELE NAZIEMNE
debrief-weapons = UŻYTE UZBROJENIE
debrief-radar-ids = IDENTYFIKACJE RADAROWE
debrief-visual-ids = IDENTYFIKACJE WZROKOWE
debrief-hits-taken = OTRZYMANE TRAFIENIA
debrief-time = CZAS
debrief-section-points = Punkty
debrief-difficulty = POZIOM TRUDNOŚCI
debrief-total = RAZEM
debrief-section-timeline = Przebieg
debrief-no-events = Brak zdarzeń
debrief-section-exposure = Ekspozycja na radary
debrief-tracked = ŚLEDZONY
debrief-closest = NAJBLIŻEJ
debrief-worst = NAJGORZEJ
debrief-at = O { $time }
debrief-altitude = WYSOKOŚĆ
debrief-speed = PRĘDKOŚĆ
debrief-margin = MARGINES
debrief-strongest = NAJSILNIEJ
debrief-tracked-by = ŚLEDZONY PRZEZ { $sites }
debrief-section-map = Mapa
debrief-legend-safe = Niewykrywalny
debrief-legend-close = Blisko wykrycia
debrief-legend-detectable = Wykrywalny
debrief-legend-tracked = Śledzony / odpalenie rakiety
debrief-legend-hit = Otrzymane trafienie / zestrzelenie
debrief-pause = PAUZA
debrief-play = ODTWARZAĆ
