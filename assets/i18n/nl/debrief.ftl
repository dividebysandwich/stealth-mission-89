# De debriefing van de missie (src/debrief.rs) en het rapport van de simulatie (stealth-sim score.rs).

## Hoe een vlucht eindigde

outcome-landed = Veilige landing
outcome-bad-landing = Slechte landing
outcome-ejected = Uitgeschoten - geland met parachute
outcome-rescued = Uitgeschoten - gered door CSAR
outcome-captured = Uitgeschoten - gevangengenomen
outcome-crashed = Neergestort
outcome-ejection-failed = Uitschieten mislukt
outcome-aborted = Missie afgebroken
outcome-collision = Botsing

## Beoordelingen

rating-unsatisfactory = Onvoldoende
rating-marginal = Matig
rating-satisfactory = Voldoende
rating-excellent = Uitstekend
rating-outstanding = Uitmuntend

## Punten

points-primary = Primair doel
points-secondary = Secundair doel
points-aircraft = Toestellen vernietigd
points-unseen = Ongezien neergehaald
points-ground = Gronddoelen vernietigd
points-drones = Drones neergehaald
points-drones-through = Drones doorgekomen
points-nuclear = Kerncentrale geraakt
points-detected = Gedetecteerd
points-missile-hits = Raketinslagen

## Tijdlijn (namen van plaatsen en toestellen zoals in de missie)

timeline-primary = {" "}(primair)
timeline-secondary = {" "}(secundair)
timeline-destroyed = { $place } vernietigd{ $tag }
timeline-destroyed-ground = Gronddoel vernietigd{ $tag }
timeline-shot-down = { $aircraft } neergehaald{ $tag }
timeline-hit = Geraakt door een raket
timeline-seen = Gezien door een { $aircraft }
timeline-radar-id = Radar-ID door { $place }
timeline-objective = Doel bereikt{ $tag }
timeline-got-through = { $aircraft } is doorgekomen
timeline-photo = Foto gemaakt{ $tag }

## Het debriefingvenster

debrief-title = MISSIE-DEBRIEFING
debrief-training = TRAININGSMISSIE
debrief-results = RESULTATEN
debrief-flight-map = VLUCHTKAART
debrief-section-results = Resultaten
debrief-accomplished = VOLBRACHT
debrief-failed = MISLUKT
debrief-primary = PRIMAIR
debrief-secondary = SECUNDAIR
debrief-aircraft = TOESTELLEN
debrief-drones = DRONES
debrief-drones-value = { $down } neergehaald, { $through } doorgekomen
debrief-ground = GROND
debrief-weapons = WAPENS
debrief-radar-ids = RADAR-ID'S
debrief-visual-ids = VISUELE ID'S
debrief-hits-taken = INCASSEERDE TREFFERS
debrief-time = TIJD
debrief-section-points = Punten
debrief-difficulty = MOEILIJKHEID
debrief-total = TOTAAL
debrief-section-timeline = Tijdlijn
debrief-no-events = Geen gebeurtenissen
debrief-section-exposure = Radarblootstelling
debrief-tracked = GEVOLGD
debrief-closest = DICHTSTBIJ
debrief-worst = SLECHTST
debrief-at = Om { $time }
debrief-altitude = HOOGTE
debrief-speed = SNELHEID
debrief-margin = MARGE
debrief-strongest = STERKST
debrief-tracked-by = GEVOLGD DOOR { $sites }
debrief-section-map = Kaart
debrief-legend-safe = Niet detecteerbaar
debrief-legend-close = Bijna gedetecteerd
debrief-legend-detectable = Detecteerbaar
debrief-legend-tracked = Gevolgd / raketlancering
debrief-legend-hit = Treffer ontvangen / kill
map-symbols = Symbolen
map-legend-clouds = Wolken / onweerstorens
map-legend-snow = Sneeuw, licht tot zwaar
map-legend-rain = Regen, licht tot zwaar
map-legend-thunderstorm = Onweerscel
debrief-pause = PAUZE
debrief-play = AFSPELEN
