# Debriefing misije (src/debrief.rs) i izvještaj simulacije (stealth-sim score.rs).

## Kako je let završio

outcome-landed = Sigurno slijetanje
outcome-bad-landing = Loše slijetanje
outcome-ejected = Katapultiranje - slijetanje padobranom
outcome-rescued = Katapultiranje - spašen CSAR-om
outcome-captured = Katapultiranje - zarobljen
outcome-crashed = Pad zrakoplova
outcome-ejection-failed = Katapultiranje nije uspjelo
outcome-aborted = Misija prekinuta
outcome-collision = Sudar

## Ocjene

rating-unsatisfactory = Nezadovoljavajuće
rating-marginal = Granično
rating-satisfactory = Zadovoljavajuće
rating-excellent = Izvrsno
rating-outstanding = Iznimno

## Bodovi

points-primary = Glavni cilj
points-secondary = Sporedni cilj
points-aircraft = Uništeni zrakoplovi
points-unseen = Oboreni neviđeno
points-ground = Uništeni ciljevi na tlu
points-drones = Oboreni dronovi
points-drones-through = Dronovi koji su prošli
points-nuclear = Pogođena nuklearna elektrana
points-detected = Otkriven
points-missile-hits = Pogoci projektilima

## Vremenski slijed (nazivi mjesta i zrakoplova kao u misiji)

timeline-primary = {" "}(glavni)
timeline-secondary = {" "}(sporedni)
timeline-destroyed = Uništeno: { $place }{ $tag }
timeline-destroyed-ground = Uništen cilj na tlu{ $tag }
timeline-shot-down = Oboren: { $aircraft }{ $tag }
timeline-hit = Pogođen projektilom
timeline-seen = Uočen od: { $aircraft }
timeline-radar-id = Radarska identifikacija: { $place }
timeline-objective = Cilj dosegnut{ $tag }
timeline-got-through = Probio se: { $aircraft }
timeline-photo = Fotografija snimljena{ $tag }

## Prozor debriefinga

debrief-title = DEBRIEFING MISIJE
debrief-training = VJEŽBOVNA MISIJA
debrief-results = REZULTATI
debrief-flight-map = KARTA LETA
debrief-section-results = Rezultati
debrief-accomplished = IZVRŠENO
debrief-failed = NEUSPJELO
debrief-primary = GLAVNI
debrief-secondary = SPOREDNI
debrief-aircraft = ZRAKOPLOVI
debrief-drones = DRONOVI
debrief-drones-value = oboreno: { $down }, prošlo: { $through }
debrief-ground = TLO
debrief-weapons = ORUŽJE
debrief-radar-ids = RADARSKE ID
debrief-visual-ids = VIZUALNE ID
debrief-hits-taken = PRIMLJENI POGOCI
debrief-time = VRIJEME
debrief-section-points = Bodovi
debrief-difficulty = TEŽINA
debrief-total = UKUPNO
debrief-section-timeline = Vremenski slijed
debrief-no-events = Nema događaja
debrief-section-exposure = Izloženost radaru
debrief-tracked = PRAĆEN
debrief-closest = NAJBLIŽE
debrief-worst = NAJGORE
debrief-at = U { $time }
debrief-altitude = VISINA
debrief-speed = BRZINA
debrief-margin = MARGINA
debrief-strongest = NAJJAČI
debrief-tracked-by = PRAĆEN: { $sites }
debrief-section-map = Karta
debrief-legend-safe = Nije moguće otkrivanje
debrief-legend-close = Blizu otkrivanja
debrief-legend-detectable = Moguće otkrivanje
debrief-legend-tracked = Praćen / lansiran projektil
debrief-legend-hit = Primljen pogodak / uništenje
debrief-pause = PAUZA
debrief-play = POKRENI
