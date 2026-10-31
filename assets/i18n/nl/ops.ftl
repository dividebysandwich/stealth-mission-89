# Het Ops-scherm van de campagne (src/campaign/ops.rs).

ops-thunderstorms = ONWEER
ops-wx-short = WX { $now }, { $next } OM { $at }
ops-wx-line = { $weather } Wisselt naar { $next } om { $at }.
ops-clock = DAG { $day }  { $time }
ops-landed-at = GELAND OP { $base }
ops-ground-crew = De grondploeg staat klaar
ops-rearm = HERBEWAPENEN & TANKEN
ops-end-sortie = VLUCHT BEËINDIGEN
ops-on-server = OP { $server }
ops-serving = HOSTEN
ops-tab-missions = MISSIES
ops-tab-map = OORLOGSKAART
ops-tab-air-defence = LUCHTVERDEDIGING
ops-tab-forces = STRIJDKRACHTEN
ops-tab-players = SPELERS
ops-tab-debrief = DEBRIEF
ops-tab-log = LOGBOEK
ops-tab-chat = CHAT
ops-blue = BLAUW { $count }
ops-red = ROOD { $count }
ops-war-over = De oorlog is voorbij
ops-running = Oorlogssimulatie loopt
ops-save-exit = OPSLAAN & STOPPEN
ops-disconnect = VERBINDING VERBREKEN
ops-leave-local = OPSLAAN EN DE CAMPAGNE VERLATEN?
    .text = De oorlog stopt tot je hem hervat. Toestellen in de lucht gaan verloren; lopende inzetten beginnen opnieuw vanaf hun basis.
ops-leave-host = OPSLAAN EN STOPPEN MET HOSTEN VAN DE CAMPAGNE?
    .text = De spelers die zijn aangesloten, worden losgekoppeld en de oorlog stopt tot je hem hervat. Toestellen in de lucht gaan verloren; lopende inzetten beginnen opnieuw vanaf hun basis.
ops-leave-client = VERBINDING MET DE SERVER VERBREKEN?
    .text = De campagne gaat door op de server; hier wordt niets opgeslagen. Je vlucht eindigt.
ops-unit-destroyed = { $unit } vernietigd: terug naar de operaties

## Missies

ops-section-missions = Missies
ops-threat = DREIGING { $threat }
ops-no-missions = Geen missies: het front is rustig
ops-section-underway = Inzetten en aanvallen
ops-enemy-liftoff = VIJAND bij { $place }: opstijgen over { $mins } min
ops-liftoff = { $place }: opstijgen over { $mins } min
ops-enemy-helis = VIJAND bij { $place }: { $count } helikopters naderen
ops-helis = { $place }: { $count } helikopters onderweg
ops-raid-sead = AI-SEAD bij { $place }
ops-raid-drones = DRONES bij { $place }
ops-raid = AANVAL op vliegveld { $place }
ops-raid-launch = { $raid }: lancering over { $mins } min
ops-raid-on-way = { $raid }: { $count } onderweg
ops-resupply-ai = BEVOORRADING voor de { $unit }: UH-60 onderweg
ops-awacs = VIJANDELIJKE AWACS nog { $mins } min in de lucht
ops-down = NEERGEHAALD: { $pilot } - redding binnen { $mins } min
ops-aboard = AAN BOORD: { $pilot } - op weg naar huis
ops-none-underway = Geen lopend
ops-ai-helicopter = Een AI-helikopter neemt het over in { $time } tenzij een UH-60-piloot het doet
ops-defences = Verdediging: { $sams } SAM-sites, dreiging { $threat }
ops-recon-intel = RECON-INFO
ops-order-insertion = INZET BEVELEN
    .facts = Vlieg de escorte om hen te dekken, of leid hen aan in de UH-60
ops-from = Van
ops-planned-by-group = Gepland door de groep
ops-stores = { $aircraft } - bewapening: { $stores }
ops-fly-uh60 = VLIEG DE UH-60
ops-accept = MISSIE AANVAARDEN
ops-planning = PLANNEN: { $pilots }
ops-flying = VLIEGEN: { $pilots }
ops-package = PACKAGE met { $kinds }: haal hier twee soorten binnen { $mins } min, +{ $bonus } punten elk
ops-sead-on-way = AI-SEAD is onderweg
ops-request-sead = AI-SEAD AANVRAGEN ({ $cost } materieel, { $aircraft } toestellen)
    .facts = Bevriende aanvalstoestellen vallen vóór jou een SAM-site van deze plaats aan
    .disabled = Te weinig materieel of toestellen op de grond (zie STRIJDKRACHTEN)

## Regels, materieel, strijdkrachten

ops-rule-damage = realistische schade
ops-rule-guidance = realistische geleiding
ops-rule-no-crashes = geen crashes
ops-rule-time = doorlopende tijd
ops-rule-weapons = realistische F-117A-wapens
ops-rule-friendly-fire = friendly fire
ops-rule-classic = klassiek
ops-materiel = MATERIEEL
ops-materiel-blue = BLAUW { $materiel } (+{ $income }/u)
ops-materiel-red = ROOD { $materiel } (+{ $income }/u)
ops-friendly = BEVRIEND
ops-enemy = VIJAND
ops-section-forces = Strijdkrachten
ops-places-held = Gehouden plaatsen
ops-materiel-row = Materieel
ops-income = Inkomsten per uur
ops-income-from = uit
ops-income-sources = { $base } + { $places } plaatsen x { $per_place } + { $cities } steden x { $per_city }
ops-fighters-ground = Jagers op de grond
ops-helicopters-ground = Helikopters op de grond
ops-in-air = In de lucht
ops-in-air-value = { $fighters } jagers, { $strike } aanvalstoestellen, { $helicopters } helikopters
ops-since-start = Sinds de start
ops-earned = verdiend
ops-repairs = reparaties
ops-operations = vluchten, aanvallen, inzetten
ops-new-aircraft = nieuwe toestellen
ops-lost = verloren (steden, piloten)
ops-section-how = Hoe het werkt
ops-how =
    Elke partij verdient elk uur materieel: een basisbedrag, meer voor elke plaats die ze houdt en voor elke intacte stad daarbij. Bombardeer de steden van de vijand (INTERDICTION) om zijn inkomsten te verlagen.
    De AI betaalt voor alles wat ze doet: reparaties, elke jagervlucht, aanvallen en haar eigen inzetten. Bij gebrek aan materieel blijft haar schade onhersteld en blijven haar toestellen thuis.
    De AI vliegt alleen met de toestellen die ze heeft; neergehaald zijn ze weg, geland vliegen ze weer. Elk uur koopt elke partij nieuwe zolang haar materieel strekt.
    Jij komt nooit tekort: je toestellen, je bases (gebombardeerd of niet) en de helikopters van je eigen inzetbevelen zijn er altijd.
ops-section-prices = Prijzen
ops-price-airfield = Vliegveld repareren
ops-price-sam = SAM-site repareren
ops-price-city = Stad herbouwd
ops-price-ship = Schip repareren
ops-price-gun = Kanon of MANPADS-team
ops-price-sortie = Jagervlucht
ops-price-raid = Aanval
ops-price-insertion = AI-inzet
ops-price-fighter = Nieuwe jager
ops-price-helicopter = Nieuwe helikopter
ops-price-city-lost = Stad vernietigd (verloren)
ops-price-captured = Piloot gevangengenomen (verloren)

## Oorlogskaart

ops-click-to-watch = { $unit } - klik om te kijken
ops-section-legend = Legenda
ops-legend-friendly-airfield = Bevriend vliegveld
ops-legend-enemy-airfield = Vijandelijk vliegveld
ops-legend-city = Bevriende / vijandelijke stad
ops-legend-airfield = Eigen / vijandelijk vliegveld
ops-legend-insertion = Inzetroute
ops-watch-hint = Klik op een eenheid om live mee te kijken
ops-map-hint = Wiel: zoomen, slepen: pannen
ops-patriot = Patriot-batterij
ops-drone-center = Dronecommandopost
ops-manned-by = Bemand door { $pilot }
ops-manned-by-other = Bemand door een andere speler
ops-control = BESTUREN
    .facts = Neem zijn console over

## Debrief, einde, logboek, chat

ops-no-sortie = Nog geen vlucht gevlogen
ops-section-sortie = Vlucht
ops-achieved = DOEL BEREIKT
ops-not-achieved = DOEL NIET BEREIKT
ops-waiting-raid = WACHTEN OP DE AANVAL
ops-waiting-insertion = WACHTEN OP DE INZET
ops-flight-time = Vliegtijd { $mins } min
ops-kills = Kills: { $air } toestellen, { $ground } op de grond
ops-points = { $points } punten
ops-points-bonus = { $points } punten (packagebonus +{ $bonus })
ops-campaign-points = { $rank } - { $points } campagnepunten
ops-campaign-record = Campagne: { $sorties } vluchten, { $won } doelen bereikt
ops-section-meanwhile = Intussen
ops-victory = OVERWINNING
    .text = De vijand houdt geen vliegveld meer: het operatiegebied is van ons.
ops-defeat = NEDERLAAG
    .text = Er is geen bevriend vliegveld meer: het operatiegebied is verloren.
ops-end-day = Dag { $day }. Bevriende plaatsen { $friendly }, vijandelijke plaatsen { $enemy }.
ops-next-campaign = DE VOLGENDE CAMPAGNE BEGINT OVER { $time }
ops-next-campaign-now = DE VOLGENDE CAMPAGNE BEGINT
ops-next-from-menu = Start de volgende campagne vanuit het hoofdmenu.
ops-section-log = Campagnelogboek
ops-section-chat = Chat
ops-no-messages = Nog geen berichten
ops-message = Bericht
    .hint = Typ een bericht, Enter verstuurt
ops-send = VERSTUUR

## Briefing

ops-briefing = MISSIEBRIEFING
ops-briefing-line = Vanaf { $base } - { $km } km naar het doel - { $aircraft } - bewapening: { $stores }
ops-route-locked = Route vergrendeld: { $pilots } vliegt hem
ops-route-threats = ROUTE & DREIGINGEN
ops-take-off = OPSTIJGEN
ops-radars-shown = Getoonde radars: gedetecteerd door bevriende toestellen, nog intact

## Spelers

ops-section-players = Spelers
ops-col-name = NAAM
ops-col-status = STATUS
ops-col-sorties = VLUCHTEN
ops-col-points = PUNTEN
ops-col-pilot = PILOOT
ops-col-objectives = DOELEN
ops-col-kills = KILLS LUCHT/GROND
ops-col-medal = BESTE MEDAILLE
ops-you = (jij)
ops-host = HOST
ops-status-planning = Plant { $title }
ops-status-board = Op het missiebord
ops-status-flying = Vliegt { $title } - { $aircraft }
ops-status-reconnecting = Verbinding verloren - toestel { $seconds } s vastgehouden
ops-flown = { $flown } ({ $won } bereikt)
ops-section-leaderboard = Ranglijst
ops-alone = Je vliegt alleen. Om samen te vliegen host je deze campagne via MULTIPLAYER in het hoofdmenu.
ops-rules = Regels van deze campagne (behouden vanaf de start): { $rules }
ops-no-sorties = Nog geen vluchten gevlogen
