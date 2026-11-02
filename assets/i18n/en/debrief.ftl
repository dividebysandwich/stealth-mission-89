# The mission debrief (src/debrief.rs) and the simulation's report (stealth-sim score.rs).

## How a flight ended

outcome-landed = Safe landing
outcome-bad-landing = Bad landing
outcome-ejected = Ejected - landed by parachute
outcome-rescued = Ejected - rescued by CSAR
outcome-captured = Ejected - captured
outcome-crashed = Crashed
outcome-ejection-failed = Ejection failed
outcome-aborted = Mission aborted
outcome-collision = Collision

## Ratings

rating-unsatisfactory = Unsatisfactory
rating-marginal = Marginal
rating-satisfactory = Satisfactory
rating-excellent = Excellent
rating-outstanding = Outstanding

## Points

points-primary = Primary objective
points-secondary = Secondary objective
points-aircraft = Aircraft destroyed
points-unseen = Shot down unseen
points-ground = Ground targets destroyed
points-drones = Drones shot down
points-drones-through = Drones through
points-nuclear = Nuclear plant hit
points-detected = Detected
points-missile-hits = Missile hits

## Timeline (place and aircraft names as in the mission)

timeline-primary = {" "}(primary)
timeline-secondary = {" "}(secondary)
timeline-destroyed = Destroyed { $place }{ $tag }
timeline-destroyed-ground = Destroyed ground target{ $tag }
timeline-shot-down = Shot down { $aircraft }{ $tag }
timeline-hit = Hit by a missile
timeline-seen = Seen by a { $aircraft }
timeline-radar-id = Radar ID by { $place }
timeline-objective = Objective reached{ $tag }
timeline-got-through = { $aircraft } got through
timeline-photo = Photo taken{ $tag }

## The debrief window

debrief-title = MISSION DEBRIEF
debrief-training = TRAINING MISSION
debrief-results = RESULTS
debrief-flight-map = FLIGHT MAP
debrief-section-results = Results
debrief-accomplished = ACCOMPLISHED
debrief-failed = FAILED
debrief-primary = PRIMARY
debrief-secondary = SECONDARY
debrief-aircraft = AIRCRAFT
debrief-drones = DRONES
debrief-drones-value = { $down } down, { $through } through
debrief-ground = GROUND
debrief-weapons = WEAPONS
debrief-radar-ids = RADAR IDS
debrief-visual-ids = VISUAL IDS
debrief-hits-taken = HITS TAKEN
debrief-time = TIME
debrief-section-points = Points
debrief-difficulty = DIFFICULTY
debrief-total = TOTAL
debrief-section-timeline = Timeline
debrief-no-events = No events
debrief-section-exposure = Radar exposure
debrief-tracked = TRACKED
debrief-closest = CLOSEST
debrief-worst = WORST
debrief-at = At { $time }
debrief-altitude = ALTITUDE
debrief-speed = SPEED
debrief-margin = MARGIN
debrief-strongest = STRONGEST
debrief-tracked-by = TRACKED BY { $sites }
debrief-section-map = Map
debrief-legend-safe = Not detectable
debrief-legend-close = Close to detection
debrief-legend-detectable = Detectable
debrief-legend-tracked = Tracked / missile launch
debrief-legend-hit = Hit taken / kill
map-symbols = Symbols
map-legend-clouds = Clouds / storm towers
map-legend-snow = Snow, light to heavy
map-legend-rain = Rain, light to heavy
map-legend-thunderstorm = Thunderstorm cell
debrief-pause = PAUSE
debrief-play = PLAY
