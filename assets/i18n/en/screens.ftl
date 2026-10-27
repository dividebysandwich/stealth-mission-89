# The kneeboard (src/kneeboard.rs, stroke font), the mission planner (src/planner.rs) and the
# saved missions (src/savegame.rs). Navigation abbreviations (PT, BRG, ETA, PRI, SEC, KM, KT)
# stay in English.

kb-brief = BRIEF
kb-route = ROUTE
kb-map = MAP
kb-target = TARGET
kb-keys = KEYS
kb-tip = Maps, mission notes and the key guide
kb-geran-file = GERAN-2 (FILE)
kb-objectives = OBJECTIVES
kb-primary = PRIMARY: { $objective }
kb-secondary = SECONDARY: { $objective }
kb-times = TIMES
kb-deadline = DEADLINE { $time }
kb-now = NOW { $time }
kb-conditions = CONDITIONS
kb-moon = MOON: { $phase } ({ $lit }% LIT)
kb-armament = ARMAMENT
kb-roe = RULES OF ENGAGEMENT
kb-total = TOTAL { $km } KM AT { $kt } KT
kb-fuel = FUEL { $lbs } LBS
kb-preparing = PREPARING...
kb-airborne = AIRBORNE
kb-developing = DEVELOPING...
kb-no-photo = NO PHOTO
kb-airborne-target = AIRBORNE TARGET
kb-scroll-vr = SCROLL: THUMBSTICK, FINGER ON THE BOARD
kb-scroll-desktop = SCROLL: MOUSE WHEEL OVER THE BOARD

## Mission planner

planner-low = LOW 500 FT
planner-medium = MED 5000 FT
planner-high = HIGH 20000 FT
planner-title = MISSION PLANNER
planner-route = Route
planner-base = BASE
planner-total = TOTAL { $km } KM  { $time }
planner-briefed = Briefed route { $km } km, fuel { $fuel } lbs
planner-coverage = Radar coverage for
    .note = Radar coverage assumes 480kt level flight, bay doors closed.
planner-detected = { $km } km of the route detected
planner-pulse = Pulse radar (P) detection range
planner-doppler = Doppler radar (D) detection range
planner-missile = Missile range
planner-uav = Enemy UAV
planner-edit-hint = Drag a point to move it, the map to pan. Click a leg to add a point, right click to remove it. Wheel: zoom.
planner-reset = RESET
planner-view-hint = Drag the map to pan. Wheel: zoom.

## Saved missions

saves-title = SAVED MISSIONS
saves-subtitle = Saved in flight from the menu: SAVE MISSION
saves-none = No saved missions
saves-line = { $theater }  -  { $aircraft }  -  mission time { $time }
saves-saved-at = Saved { $date }
saves-unreadable = Unreadable save (another version)
saves-delete =
    .facts = Delete this save
saves-back =
    .facts = Back to the single mission screen
