# Tehtävän purku (src/debrief.rs) ja simulaation raportti (stealth-sim score.rs).

## Lennon päättyminen

outcome-landed = Turvallinen lasku
outcome-bad-landing = Huono lasku
outcome-ejected = Katapultoitu - laskeutui laskuvarjolla
outcome-rescued = Katapultoitu - CSAR pelasti
outcome-captured = Katapultoitu - vangittu
outcome-crashed = Syöksyi maahan
outcome-ejection-failed = Katapultointi epäonnistui
outcome-aborted = Tehtävä keskeytetty
outcome-collision = Törmäys

## Arvosanat

rating-unsatisfactory = Riittämätön
rating-marginal = Välttävä
rating-satisfactory = Tyydyttävä
rating-excellent = Erinomainen
rating-outstanding = Poikkeuksellinen

## Pisteet

points-primary = Päätavoite
points-secondary = Sivutavoite
points-aircraft = Tuhotut koneet
points-unseen = Alasammuttu huomaamatta
points-ground = Tuhotut maakohteet
points-drones = Alasammutut droonit
points-drones-through = Läpi päässeet droonit
points-nuclear = Osuma ydinvoimalaan
points-detected = Havaittu
points-missile-hits = Ohjusosumat

## Aikajana (paikkojen ja koneiden nimet kuten tehtävässä)

timeline-primary = {" "}(päätavoite)
timeline-secondary = {" "}(sivutavoite)
timeline-destroyed = Tuhottu: { $place }{ $tag }
timeline-destroyed-ground = Tuhottu maakohde{ $tag }
timeline-shot-down = Alasammuttu: { $aircraft }{ $tag }
timeline-hit = Ohjusosuma
timeline-seen = Havaitsija: { $aircraft }
timeline-radar-id = Tutkatunnistus: { $place }
timeline-objective = Tavoite saavutettu{ $tag }
timeline-got-through = Läpi päässyt: { $aircraft }
timeline-photo = Kuva otettu{ $tag }

## Purkuikkuna

debrief-title = TEHTÄVÄN PURKU
debrief-training = HARJOITUSTEHTÄVÄ
debrief-results = TULOKSET
debrief-flight-map = LENTOKARTTA
debrief-section-results = Tulokset
debrief-accomplished = SUORITETTU
debrief-failed = EPÄONNISTUI
debrief-primary = PÄÄTAVOITE
debrief-secondary = SIVUTAVOITE
debrief-aircraft = KONEET
debrief-drones = DROONIT
debrief-drones-value = alas { $down }, läpi { $through }
debrief-ground = MAA
debrief-weapons = ASEET
debrief-radar-ids = TUTKATUNNISTUKSET
debrief-visual-ids = NÄKÖTUNNISTUKSET
debrief-hits-taken = SAADUT OSUMAT
debrief-time = AIKA
debrief-section-points = Pisteet
debrief-difficulty = VAIKEUS
debrief-total = YHTEENSÄ
debrief-section-timeline = Aikajana
debrief-no-events = Ei tapahtumia
debrief-section-exposure = Tutka-altistus
debrief-tracked = SEURATTU
debrief-closest = LÄHIN
debrief-worst = HUONOIN
debrief-at = Aika { $time }
debrief-altitude = KORKEUS
debrief-speed = NOPEUS
debrief-margin = MARGINAALI
debrief-strongest = VAHVIN
debrief-tracked-by = SEURAAJA: { $sites }
debrief-section-map = Kartta
debrief-legend-safe = Ei havaittavissa
debrief-legend-close = Lähellä havaitsemista
debrief-legend-detectable = Havaittavissa
debrief-legend-tracked = Seurattu / ohjuksen laukaisu
debrief-legend-hit = Osuma saatu / tuho
debrief-pause = TAUKO
debrief-play = TOISTA
