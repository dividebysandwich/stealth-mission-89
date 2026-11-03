# The campaign's Ops screen (src/campaign/ops.rs).

ops-thunderstorms = THUNDERSTORMS
ops-wx-short = WX { $now }, { $next } AT { $at }
ops-wx-line = { $weather } Changing to { $next } at { $at }.
ops-clock = DAY { $day }  { $time }
ops-landed-at = LANDED AT { $base }
ops-ground-crew = The ground crew is ready
ops-rearm = REARM & REFUEL
ops-end-sortie = END SORTIE
ops-on-server = ON { $server }
ops-serving = SERVING
ops-tab-missions = MISSIONS
ops-tab-map = WAR MAP
ops-tab-air-defence = AIR DEFENSE
ops-tab-forces = FORCES
ops-tab-players = PLAYERS
ops-tab-debrief = DEBRIEF
ops-tab-log = LOG
ops-tab-chat = CHAT
ops-blue = BLUE { $count }
ops-red = RED { $count }
ops-war-over = The war is over
ops-running = War simulation running
ops-save-exit = SAVE & EXIT
ops-disconnect = DISCONNECT
ops-leave-local = SAVE AND LEAVE THE CAMPAIGN?
    .text = The war stops until you continue it. Aircraft in the air are lost; insertions under way start again from their base.
ops-leave-host = SAVE AND STOP SERVING THE CAMPAIGN?
    .text = The players who joined are disconnected and the war stops until you continue it. Aircraft in the air are lost; insertions under way start again from their base.
ops-leave-client = DISCONNECT FROM THE SERVER?
    .text = The campaign goes on on the server; nothing is saved here. Your sortie ends.
ops-unit-destroyed = { $unit } destroyed: back to operations

## Missions

ops-section-missions = Missions
ops-threat = THREAT { $threat }
ops-no-missions = No missions: the front is quiet
ops-ai-helicopter = An AI helicopter takes it in { $time } unless a UH-60 pilot does
ops-ai-load = An AI helicopter takes a load in { $time } unless a UH-60 pilot does
ops-ai-takes-off = Takeoff in { $time }
ops-ai-aircraft = AI { $aircraft }
ops-cargo-in = Cargo { $delivered } of { $loads } in - the helipad waits { $time } h more
ops-defences = Defences: { $sams } SAM sites, threat { $threat }
ops-recon-intel = RECON INTEL
ops-order-insertion = ORDER INSERTION
    .facts = Fly the escort to cover them, or lead them in the UH-60
ops-from = From
ops-planned-by-group = Planned by the group
ops-stores = { $aircraft } - stores: { $stores }
ops-fly-uh60 = FLY THE UH-60
ops-accept = ACCEPT MISSION
ops-fly-with-ai = TAKE OFF
ops-take-control = TAKE CONTROL
    .facts = Fly one of the AI flight's aircraft on from where it is
ops-planning = PLANNING: { $pilots }
ops-flying = FLYING: { $pilots }
ops-package = PACKAGE with { $kinds }: achieve two kinds here within { $mins } min, +{ $bonus } points each
ops-sead-on-way = AI SEAD is on its way
ops-request-sead = REQUEST AI SEAD ({ $cost } materiel, { $aircraft } aircraft)
    .facts = Friendly strike aircraft go for a SAM site of this place ahead of you
    .disabled = Not enough materiel or aircraft on the ground (see FORCES)

## Rules, materiel, forces

ops-rule-damage = realistic damage
ops-rule-guidance = realistic guidance
ops-rule-no-crashes = no crashes
ops-rule-time = continuous time
ops-rule-weapons = realistic F-117A weapons
ops-rule-friendly-fire = friendly fire
ops-rule-airframe-losses = losses cost resources
ops-rule-no-ai-missions = no friendly AI missions
ops-rule-classic-terrain = classic terrain
ops-rule-classic-ai-flight = classic AI flight
ops-rule-classic = classic
ops-materiel = MATERIEL
ops-materiel-blue = BLUE { $materiel } (+{ $income }/h)
ops-materiel-red = RED { $materiel } (+{ $income }/h)
ops-crew-tempo = ENEMY TEMPO ×{ $pace }
ops-due = T-{ $mins } min
ops-due-now = NOW
ops-front-blue = FRONT: BLUE GAINING
ops-front-red = FRONT: RED GAINING
ops-front-even = FRONT: HOLDING
ops-operation = Capture of { $place }:
ops-friendly = FRIENDLY
ops-enemy = ENEMY
ops-section-forces = Forces
ops-places-held = Places held
ops-materiel-row = Materiel
ops-income = Income per hour
ops-income-from = from
ops-income-sources = { $base } + { $places } places x { $per_place } + { $cities } cities x { $per_city }
ops-fighters-ground = Fighters on the ground
ops-helicopters-ground = Helicopters on the ground
ops-in-air = In the air
ops-in-air-value = { $fighters } fighters, { $strike } strike, { $helicopters } helicopters
ops-since-start = Since the start
ops-earned = earned
ops-repairs = repairs
ops-operations = sorties, raids, insertions
ops-new-aircraft = new aircraft
ops-lost = lost (cities, pilots)
ops-section-how = How it works
ops-how =
    Each side earns materiel every hour: a base amount, more for every place it holds and for every intact city among them. Bomb the enemy's cities (INTERDICTION) to cut its income.
    The AI pays for everything it does: repairs, every fighter sortie, raids and its own insertions. Short of materiel, its damage stays unrepaired and its aircraft stay home.
    The AI flies only the aircraft it has; shot down they are gone, landed they fly again. Every hour each side buys new ones while its materiel lasts.
    You never run short: your aircraft, your bases (bombed or not) and the helicopters of your own insertion orders are always there.
ops-how-repairs = Each side repairs only a few sites at once (more with more intact cities), nearest the front first, and a site destroyed again soon after its repair takes longer each time.
ops-how-airframes = Losses cost resources: every aircraft a pilot loses is one fighter or helicopter fewer for the friendly AI.
ops-section-prices = Prices
ops-price-airfield = Airfield repair
ops-price-sam = SAM site repair
ops-price-city = City rebuilt
ops-price-ship = Ship repair
ops-price-gun = Gun or MANPADS team
ops-price-sortie = Fighter sortie
ops-price-raid = Raid
ops-price-insertion = AI insertion
ops-price-fighter = New fighter
ops-price-helicopter = New helicopter
ops-price-city-lost = City destroyed (lost)
ops-price-captured = Pilot captured (lost)

## War map

ops-click-to-watch = { $unit } - click to watch
ops-section-legend = Legend
ops-legend-friendly-airfield = Friendly airfield
ops-legend-enemy-airfield = Enemy airfield
ops-legend-city = Friendly / enemy city
ops-legend-airfield = Friendly / enemy airfield
ops-legend-insertion = Insertion route
ops-watch-hint = Click a unit to watch it live
ops-layers-hint = LAYERS (top right): what the map shows
ops-map-tab = MAP
ops-on-radio = ON THE RADIO
ops-mute = MUTE
    .facts = Hear (or not) this player on the radio
ops-unmute = UNMUTE
ops-map-hint = Wheel: zoom, drag: pan
ops-patriot = Patriot battery
ops-drone-center = Drone command center
ops-manned-by = Manned by { $pilot }
ops-manned-by-other = Manned by another player
ops-control = CONTROL
    .facts = Take over its console

## Debrief, end, log, chat

ops-no-sortie = No sortie flown yet
ops-section-sortie = Sortie
ops-achieved = OBJECTIVE ACHIEVED
ops-not-achieved = OBJECTIVE NOT ACHIEVED
ops-waiting-raid = WAITING FOR THE RAID
ops-waiting-insertion = WAITING FOR THE INSERTION
ops-flight-time = Flight time { $mins } min
ops-kills = Kills: { $air } aircraft, { $ground } on the ground
ops-points = { $points } points
ops-points-bonus = { $points } points (package bonus +{ $bonus })
ops-campaign-points = { $rank } - { $points } campaign points
ops-campaign-record = Campaign: { $sorties } sorties, { $won } objectives achieved
ops-section-meanwhile = Meanwhile
ops-section-effects = Your effect on the war
ops-victory = VICTORY
    .text = The enemy holds no airfield any more: the theater is ours.
ops-defeat = DEFEAT
    .text = No friendly airfield is left: the theater is lost.
ops-end-day = Day { $day }. Friendly places { $friendly }, enemy places { $enemy }.
ops-next-campaign = THE NEXT CAMPAIGN STARTS IN { $time }
ops-next-campaign-now = THE NEXT CAMPAIGN IS STARTING
ops-next-from-menu = Start the next campaign from the main menu.
ops-section-log = Campaign log
ops-section-chat = Chat
ops-no-messages = No messages yet
ops-message = Message
    .hint = Type a message, Enter sends
ops-send = SEND

## Briefing

ops-briefing = MISSION BRIEFING
ops-briefing-line = From { $base } - { $km } km to the target - { $aircraft } - stores: { $stores }
ops-route-locked = Route locked: { $pilots } flying it
ops-route-threats = ROUTE & THREATS
ops-take-off = TAKE OFF
ops-radars-shown = Radars shown: detected by friendly aircraft, still standing

## Players

ops-section-players = Players
ops-col-name = NAME
ops-col-status = STATUS
ops-col-sorties = SORTIES
ops-col-points = POINTS
ops-col-pilot = PILOT
ops-col-objectives = OBJECTIVES
ops-col-kills = KILLS AIR/GND
ops-col-medal = BEST MEDAL
ops-you = (you)
ops-host = HOST
ops-status-planning = Planning { $title }
ops-status-board = On the mission board
ops-status-flying = Flying { $title } - { $aircraft }
ops-status-reconnecting = Connection lost - aircraft held { $seconds } s
ops-flown = { $flown } ({ $won } achieved)
ops-section-leaderboard = Leaderboard
ops-alone = Flying alone. To fly together, host this campaign from MULTIPLAYER in the main menu.
ops-rules = Rules of this campaign (kept from its start): { $rules }
ops-no-sorties = No sorties flown yet
