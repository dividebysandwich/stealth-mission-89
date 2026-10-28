# Debrifing misije (src/debrief.rs) in poročilo simulacije (stealth-sim score.rs).

## Kako se je let končal

outcome-landed = Varen pristanek
outcome-bad-landing = Slab pristanek
outcome-ejected = Katapultiranje - pristanek s padalom
outcome-rescued = Katapultiranje - rešitev s CSAR
outcome-captured = Katapultiranje - ujetje
outcome-crashed = Strmoglavljenje
outcome-ejection-failed = Katapultiranje ni uspelo
outcome-aborted = Misija prekinjena
outcome-collision = Trčenje

## Ocene

rating-unsatisfactory = Nezadostno
rating-marginal = Mejno
rating-satisfactory = Zadovoljivo
rating-excellent = Odlično
rating-outstanding = Izjemno

## Točke

points-primary = Glavni cilj
points-secondary = Drugotni cilj
points-aircraft = Uničena letala
points-unseen = Sestreljeni neopaženo
points-ground = Uničeni kopenski cilji
points-drones = Sestreljeni droni
points-drones-through = Prebiti droni
points-nuclear = Zadeta jedrska elektrarna
points-detected = Odkriti
points-missile-hits = Zadetki izstrelkov

## Časovnica (imena krajev in letal kot v misiji)

timeline-primary = {" "}(glavni)
timeline-secondary = {" "}(drugotni)
timeline-destroyed = Uničeno: { $place }{ $tag }
timeline-destroyed-ground = Uničen kopenski cilj{ $tag }
timeline-shot-down = Sestreljen: { $aircraft }{ $tag }
timeline-hit = Zadetek izstrelka
timeline-seen = Opazilo vas je letalo: { $aircraft }
timeline-radar-id = Radarska identifikacija: { $place }
timeline-objective = Cilj dosežen{ $tag }
timeline-got-through = Prebilo se je: { $aircraft }
timeline-photo = Fotografija posneta{ $tag }

## Okno debrifinga

debrief-title = DEBRIFING MISIJE
debrief-training = VADBENA MISIJA
debrief-results = REZULTATI
debrief-flight-map = KARTA LETA
debrief-section-results = Rezultati
debrief-accomplished = IZPOLNJEN
debrief-failed = NEUSPEŠEN
debrief-primary = GLAVNI
debrief-secondary = DRUGOTNI
debrief-aircraft = LETALA
debrief-drones = DRONI
debrief-drones-value = { $down } sestreljenih, { $through } prebitih
debrief-ground = KOPENSKI
debrief-weapons = ORUŽJE
debrief-radar-ids = RADARSKE ID
debrief-visual-ids = VIZUALNE ID
debrief-hits-taken = PREJETI ZADETKI
debrief-time = ČAS
debrief-section-points = Točke
debrief-difficulty = TEŽAVNOST
debrief-total = SKUPAJ
debrief-section-timeline = Časovnica
debrief-no-events = Ni dogodkov
debrief-section-exposure = Radarska izpostavljenost
debrief-tracked = SLEDENJE
debrief-closest = NAJBLIŽJE
debrief-worst = NAJHUJŠE
debrief-at = Ob { $time }
debrief-altitude = VIŠINA
debrief-speed = HITROST
debrief-margin = REZERVA
debrief-strongest = NAJMOČNEJŠI
debrief-tracked-by = SLEDI: { $sites }
debrief-section-map = Karta
debrief-legend-safe = Neodkrivljiv
debrief-legend-close = Blizu odkritja
debrief-legend-detectable = Odkrivljiv
debrief-legend-tracked = Sledenje / izstrelitev izstrelka
debrief-legend-hit = Prejet zadetek / uničenje
debrief-pause = PREMOR
debrief-play = PREDVAJAJ
