# Kampagnens Ops-skærm (src/campaign/ops.rs).

ops-thunderstorms = TORDENVEJR
ops-wx-short = VEJR { $now }, { $next } KL. { $at }
ops-wx-line = { $weather } Skifter til { $next } kl. { $at }.
ops-clock = DAG { $day }  { $time }
ops-landed-at = LANDET I { $base }
ops-ground-crew = Jordbesætningen er klar
ops-rearm = BEVÆBN OG TANK OP
ops-end-sortie = AFSLUT FLYVNINGEN
ops-on-server = PÅ { $server }
ops-serving = HOSTER
ops-tab-missions = MISSIONER
ops-tab-map = KRIGSKORT
ops-tab-air-defence = LUFTFORSVAR
ops-tab-forces = STYRKER
ops-tab-players = SPILLERE
ops-tab-debrief = DEBRIEFING
ops-tab-log = LOG
ops-tab-chat = CHAT
ops-blue = BLÅ { $count }
ops-red = RØD { $count }
ops-war-over = Krigen er forbi
ops-running = Krigssimulationen kører
ops-save-exit = GEM OG AFSLUT
ops-disconnect = FRAKOBL
ops-leave-local = GEM OG FORLAD KAMPAGNEN?
    .text = Krigen står stille, til du fortsætter den. Fly i luften går tabt; indsættelser i gang starter forfra fra deres base.
ops-leave-host = GEM OG STOP HOSTING AF KAMPAGNEN?
    .text = De spillere, der er kommet med, frakobles, og krigen står stille, til du fortsætter den. Fly i luften går tabt; indsættelser i gang starter forfra fra deres base.
ops-leave-client = FRAKOBL FRA SERVEREN?
    .text = Kampagnen fortsætter på serveren; intet gemmes her. Din flyvning slutter.
ops-unit-destroyed = { $unit } ødelagt: tilbage til operationer

## Missioner

ops-section-missions = Missioner
ops-threat = TRUSSEL { $threat }
ops-no-missions = Ingen missioner: fronten er rolig
ops-section-underway = Indsættelser og angreb
ops-enemy-liftoff = FJENDE ved { $place }: afgang om { $mins } min
ops-liftoff = { $place }: afgang om { $mins } min
ops-enemy-helis = FJENDE ved { $place }: { $count } helikoptere indflyvende
ops-helis = { $place }: { $count } helikoptere på vej
ops-raid-sead = AI SEAD ved { $place }
ops-raid-drones = DRONER ved { $place }
ops-raid = ANGREB på flyvepladsen { $place }
ops-raid-launch = { $raid }: start om { $mins } min
ops-raid-on-way = { $raid }: { $count } på vej
ops-resupply-ai = FORSYNING til { $unit }: UH-60 på vej
ops-awacs = FJENDTLIG AWACS i luften i yderligere { $mins } min
ops-down = NEDE: { $pilot } - redning inden for { $mins } min
ops-aboard = OM BORD: { $pilot } - på vej hjem
ops-none-underway = Ingen i gang
ops-ai-helicopter = En AI-helikopter tager den om { $time }, medmindre en UH-60-pilot gør det
ops-defences = Forsvar: { $sams } SAM-stillinger, trussel { $threat }
ops-recon-intel = RECON-EFTERRETNING
ops-order-insertion = BESTIL INDSÆTTELSE
    .facts = Flyv eskorten for at dække dem, eller før dem ind i UH-60
ops-from = Fra
ops-planned-by-group = Planlagt af gruppen
ops-stores = { $aircraft } - bevæbning: { $stores }
ops-fly-uh60 = FLYV UH-60
ops-accept = ACCEPTÉR MISSION
ops-planning = PLANLÆGGER: { $pilots }
ops-flying = FLYVER: { $pilots }
ops-package = PACKAGE med { $kinds }: opnå to typer her inden for { $mins } min, +{ $bonus } point hver
ops-sead-on-way = AI SEAD er på vej
ops-request-sead = ANMOD OM AI SEAD ({ $cost } materiel, { $aircraft } fly)
    .facts = Egne angrebsfly angriber en SAM-stilling på dette sted foran dig
    .disabled = Ikke nok materiel eller fly på jorden (se STYRKER)

## Regler, materiel, styrker

ops-rule-damage = realistisk skade
ops-rule-guidance = realistisk styring
ops-rule-no-crashes = ingen styrt
ops-rule-time = kontinuerlig tid
ops-rule-weapons = realistiske F-117A-våben
ops-rule-friendly-fire = egenbeskydning
ops-rule-classic = klassisk
ops-materiel = MATERIEL
ops-materiel-blue = BLÅ { $materiel } (+{ $income }/t)
ops-materiel-red = RØD { $materiel } (+{ $income }/t)
ops-friendly = EGNE
ops-enemy = FJENDE
ops-section-forces = Styrker
ops-places-held = Holdte steder
ops-materiel-row = Materiel
ops-income = Indtægt pr. time
ops-income-from = fra
ops-income-sources = { $base } + { $places } steder x { $per_place } + { $cities } byer x { $per_city }
ops-fighters-ground = Jagerfly på jorden
ops-helicopters-ground = Helikoptere på jorden
ops-in-air = I luften
ops-in-air-value = { $fighters } jagerfly, { $strike } angrebsfly, { $helicopters } helikoptere
ops-since-start = Siden start
ops-earned = tjent
ops-repairs = reparationer
ops-operations = flyvninger, angreb, indsættelser
ops-new-aircraft = nye fly
ops-lost = tabt (byer, piloter)
ops-section-how = Sådan virker det
ops-how =
    Hver side tjener materiel hver time: et grundbeløb, mere for hvert sted, den holder, og for hver intakt by blandt dem. Bomb fjendens byer (AFSKÆRING) for at skære i dens indtægt.
    AI betaler for alt, hvad den gør: reparationer, hver jagerflyvning, angreb og sine egne indsættelser. Mangler den materiel, forbliver dens skader urepareret, og dens fly bliver hjemme.
    AI flyver kun med de fly, den har; nedskudte er væk, landede kan flyve igen. Hver time køber hver side nye, så længe materiellet rækker.
    Du løber aldrig tør: dine fly, dine baser (bombede eller ej) og helikopterne fra dine egne indsættelsesordrer er der altid.
ops-section-prices = Priser
ops-price-airfield = Reparation af flyveplads
ops-price-sam = Reparation af SAM-stilling
ops-price-city = By genopbygget
ops-price-ship = Reparation af skib
ops-price-gun = Kanon eller MANPADS-hold
ops-price-sortie = Jagerflyvning
ops-price-raid = Angreb
ops-price-insertion = AI-indsættelse
ops-price-fighter = Nyt jagerfly
ops-price-helicopter = Ny helikopter
ops-price-city-lost = By ødelagt (tabt)
ops-price-captured = Pilot fanget (tabt)

## Krigskort

ops-click-to-watch = { $unit } - klik for at følge
ops-section-legend = Signaturforklaring
ops-legend-friendly-airfield = Egen flyveplads
ops-legend-enemy-airfield = Fjendtlig flyveplads
ops-legend-city = Egen / fjendtlig by
ops-legend-airfield = Egen / fjendtlig flyveplads
ops-legend-insertion = Indsættelsesrute
ops-watch-hint = Klik på en enhed for at følge den live
ops-map-hint = Hjul: zoom, træk: panorér
ops-patriot = Patriot-batteri
ops-drone-center = Droneføringscentral
ops-manned-by = Bemandet af { $pilot }
ops-manned-by-other = Bemandet af en anden spiller
ops-control = OVERTAG
    .facts = Overtag dens konsol

## Debriefing, slutning, log, chat

ops-no-sortie = Ingen flyvning endnu
ops-section-sortie = Flyvning
ops-achieved = MÅL NÅET
ops-not-achieved = MÅL IKKE NÅET
ops-waiting-raid = AFVENTER ANGREBET
ops-waiting-insertion = AFVENTER INDSÆTTELSEN
ops-flight-time = Flyvetid { $mins } min
ops-kills = Nedskydninger: { $air } fly, { $ground } på jorden
ops-points = { $points } point
ops-points-bonus = { $points } point (package-bonus +{ $bonus })
ops-campaign-points = { $rank } - { $points } kampagnepoint
ops-campaign-record = Kampagne: { $sorties } flyvninger, { $won } mål nået
ops-section-meanwhile = Imens
ops-victory = SEJR
    .text = Fjenden har ikke flere flyvepladser: operationsområdet er vores.
ops-defeat = NEDERLAG
    .text = Ingen af vores flyvepladser er tilbage: operationsområdet er tabt.
ops-end-day = Dag { $day }. Egne steder { $friendly }, fjendtlige steder { $enemy }.
ops-next-campaign = NÆSTE KAMPAGNE STARTER OM { $time }
ops-next-campaign-now = NÆSTE KAMPAGNE STARTER
ops-next-from-menu = Start den næste kampagne fra hovedmenuen.
ops-section-log = Kampagnelog
ops-section-chat = Chat
ops-no-messages = Ingen beskeder endnu
ops-message = Besked
    .hint = Skriv en besked, Enter sender
ops-send = SEND

## Briefing

ops-briefing = MISSIONSBRIEFING
ops-briefing-line = Fra { $base } - { $km } km til målet - { $aircraft } - bevæbning: { $stores }
ops-route-locked = Ruten er låst: { $pilots } flyver den
ops-route-threats = RUTE & TRUSLER
ops-take-off = START
ops-radars-shown = Viste radarer: opdaget af egne fly, stadig intakte

## Spillere

ops-section-players = Spillere
ops-col-name = NAVN
ops-col-status = STATUS
ops-col-sorties = FLYVNINGER
ops-col-points = POINT
ops-col-pilot = PILOT
ops-col-objectives = MÅL
ops-col-kills = NEDSKYDNINGER LUFT/JORD
ops-col-medal = BEDSTE MEDALJE
ops-you = (dig)
ops-host = VÆRT
ops-status-planning = Planlægger { $title }
ops-status-board = På missionstavlen
ops-status-flying = Flyver { $title } - { $aircraft }
ops-status-reconnecting = Forbindelsen mistet - fly holdt i { $seconds } s
ops-flown = { $flown } ({ $won } nået)
ops-section-leaderboard = Rangliste
ops-alone = Flyver alene. For at flyve sammen skal du være vært for kampagnen under MULTIPLAYER i hovedmenuen.
ops-rules = Kampagnens regler (bevaret fra start): { $rules }
ops-no-sorties = Ingen flyvninger endnu
