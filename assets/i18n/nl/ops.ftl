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
ops-tab-flights = VLUCHTEN
ops-tab-air-defence = LUCHTVERDEDIGING
ops-tab-forces = STRIJDKRACHTEN
ops-tab-players = SPELERS
ops-tab-debrief = DEBRIEF
ops-tab-log = LOGBOEK
ops-tab-chat = CHAT
ops-top-campaign = CAMPAIGN
ops-view-war = WAR
ops-view-ground = GROUND
ops-view-weather = WEATHER
ops-bar-missions = Missions on the board: { $count }
ops-bar-hint = Click a target marker or MISSIONS to choose one, a unit to watch it live
ops-filter-all = ALL
ops-filter-attack = ATTACK
ops-filter-support = SUPPORT
ops-watch = WATCH
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
ops-fly-with-ai = OPSTIJGEN
ops-take-control = BESTURING OVERNEMEN
    .facts = Vlieg een toestel van de AI-vlucht verder vanaf waar het is
ops-join-gunner = MEEDOEN ALS BOORDSCHUTTER
    .facts = Neem plaats aan de schuttersconsole van dit toestel: richt de doelaanwijzingspod en vuur de zijkanonnen af terwijl de piloot vliegt
    .ai = Neem plaats aan de schuttersconsole van dit toestel: de AI vliegt een baan rond het punt dat je met de doelaanwijzingspod vasthoudt
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
ops-rule-airframe-losses = verliezen kosten middelen
ops-rule-no-ai-missions = geen eigen AI-missies
ops-rule-classic = klassiek
ops-materiel = MATERIEEL
ops-materiel-blue = BLAUW { $materiel } (+{ $income }/u)
ops-materiel-red = ROOD { $materiel } (+{ $income }/u)
ops-crew-tempo = VIJANDELIJK TEMPO ×{ $pace }
ops-due = T-{ $mins } min
ops-due-now = NU
ops-front-blue = FRONT: BLAUW WINT TERREIN
ops-front-red = FRONT: ROOD WINT TERREIN
ops-front-even = FRONT: HOUDT STAND
ops-operation = Verovering van { $place }:
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
ops-how-repairs = Elke kant herstelt maar een paar stellingen tegelijk (meer met meer intacte steden), die bij het front eerst, en een stelling die kort na herstel opnieuw wordt vernietigd duurt elke keer langer.
ops-how-airframes = Verliezen kosten middelen: elk toestel dat een piloot verliest is één jager of helikopter minder voor de eigen AI.
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
ops-legend-missions = Missions on the board
ops-legend-defence-reach = Air defense sites and reach
ops-watch-hint = Klik op een eenheid om live mee te kijken
ops-map-hint = Wiel: zoomen, slepen: pannen
ops-patriot = Patriot-batterij
ops-nasams = NASAMS-batterij
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
ops-crew = Bemanning: { $names }
ops-points = { $points } punten
ops-points-bonus = { $points } punten (packagebonus +{ $bonus })
ops-campaign-points = { $rank } - { $points } campagnepunten
ops-campaign-record = Campagne: { $sorties } vluchten, { $won } doelen bereikt
ops-section-meanwhile = Intussen
ops-section-effects = Jouw invloed op de oorlog
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
ops-status-gunner = Boordschutter bij { $pilot }: { $title } - { $aircraft }
ops-flown = { $flown } ({ $won } bereikt)
ops-section-leaderboard = Ranglijst
ops-alone = Je vliegt alleen. Om samen te vliegen host je deze campagne via MULTIPLAYER in het hoofdmenu.
ops-rules = Regels van deze campagne (behouden vanaf de start): { $rules }
ops-no-sorties = Nog geen vluchten gevlogen
ops-ai-load = Een AI-helikopter neemt een lading over in { $time } tenzij een UH-60-piloot het doet
ops-ai-takes-off = Opstijgen over { $time }
ops-ai-aircraft = AI { $aircraft }
ops-cargo-in = Lading { $delivered } van { $loads } binnen - het helikopterplatform wacht nog { $time } u
ops-rule-classic-terrain = klassiek terrein
ops-rule-classic-ai-flight = klassiek AI-vliegen
ops-layers-hint = LAGEN (rechtsboven): wat de kaart toont
ops-map-tab = KAART
ops-on-radio = OP DE RADIO
ops-mute = DEMPEN
    .facts = Deze speler wel of niet horen op de radio
ops-unmute = HOREN

## Flights

ops-section-flights = Eigen vluchten
ops-no-flights = Geen eigen toestellen in de lucht of wachtend om op te stijgen
ops-flight-pilot = Piloot:
ops-flight-gunner = Schutter:
ops-seat-ai = AI
ops-seat-free = vrij
ops-flight-airborne = In de lucht
ops-flight-on-ground = Op de grond
ops-flight-starting = Opstarten
