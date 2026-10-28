# Kampanjens Ops-skärm (src/campaign/ops.rs).

ops-thunderstorms = ÅSKVÄDER
ops-wx-short = VÄDER { $now }, { $next } KL { $at }
ops-wx-line = { $weather } Övergår till { $next } kl { $at }.
ops-clock = DAG { $day }  { $time }
ops-landed-at = LANDAD VID { $base }
ops-ground-crew = Marktjänsten är redo
ops-rearm = BEVÄPNA OM & TANKA
ops-end-sortie = AVSLUTA FLYGNING
ops-on-server = PÅ { $server }
ops-serving = VÄRD
ops-tab-missions = UPPDRAG
ops-tab-map = KRIGSKARTA
ops-tab-air-defence = LUFTVÄRN
ops-tab-forces = STYRKOR
ops-tab-players = SPELARE
ops-tab-debrief = DEBRIEF
ops-tab-log = LOGG
ops-tab-chat = CHATT
ops-blue = BLÅ { $count }
ops-red = RÖD { $count }
ops-war-over = Kriget är över
ops-running = Krigssimuleringen pågår
ops-save-exit = SPARA & AVSLUTA
ops-disconnect = KOPPLA FRÅN
ops-leave-local = SPARA OCH LÄMNA KAMPANJEN?
    .text = Kriget står stilla tills du fortsätter det. Flygplan i luften går förlorade; insättningar som pågår börjar om från sin bas.
ops-leave-host = SPARA OCH SLUTA VARA VÄRD FÖR KAMPANJEN?
    .text = Spelarna som gått med kopplas från och kriget står stilla tills du fortsätter det. Flygplan i luften går förlorade; insättningar som pågår börjar om från sin bas.
ops-leave-client = KOPPLA FRÅN SERVERN?
    .text = Kampanjen fortsätter på servern; ingenting sparas här. Din flygning avslutas.
ops-unit-destroyed = { $unit } förstörd: tillbaka till operationerna

## Uppdrag

ops-section-missions = Uppdrag
ops-threat = HOT { $threat }
ops-no-missions = Inga uppdrag: fronten är lugn
ops-section-underway = Insättningar och anfall
ops-enemy-liftoff = FIENDE vid { $place }: lyfter om { $mins } min
ops-liftoff = { $place }: lyfter om { $mins } min
ops-enemy-helis = FIENDE vid { $place }: { $count } helikoptrar på anflygning
ops-helis = { $place }: { $count } helikoptrar på väg
ops-raid-sead = AI-SEAD vid { $place }
ops-raid-drones = DRÖNARE vid { $place }
ops-raid = ANFALL mot flygfältet { $place }
ops-raid-launch = { $raid }: start om { $mins } min
ops-raid-on-way = { $raid }: { $count } på väg
ops-resupply-ai = FÖRNÖDENHETER till { $unit }: UH-60 på väg
ops-awacs = FIENDENS AWACS uppe i ytterligare { $mins } min
ops-down = NEDSKJUTEN: { $pilot } - räddning inom { $mins } min
ops-aboard = OMBORD: { $pilot } - på väg hem
ops-none-underway = Inga pågår
ops-ai-helicopter = En AI-helikopter tar uppdraget om { $time } om inte en UH-60-pilot gör det
ops-defences = Försvar: { $sams } SAM-ställningar, hot { $threat }
ops-recon-intel = SPANINGSINFO
ops-order-insertion = BEORDRA INSÄTTNING
    .facts = Flyg eskorten för att skydda dem, eller led in dem med UH-60
ops-from = Från
ops-planned-by-group = Planerad av gruppen
ops-stores = { $aircraft } - last: { $stores }
ops-fly-uh60 = FLYG UH-60
ops-accept = ANTA UPPDRAG
ops-planning = PLANERAR: { $pilots }
ops-flying = FLYGER: { $pilots }
ops-package = PACKAGE med { $kinds }: uppnå två typer här inom { $mins } min, +{ $bonus } poäng var
ops-sead-on-way = AI-SEAD är på väg
ops-request-sead = BEGÄR AI-SEAD ({ $cost } materiel, { $aircraft } flygplan)
    .facts = Egna attackflygplan går mot en SAM-ställning på platsen före dig
    .disabled = Inte tillräckligt med materiel eller flygplan på marken (se STYRKOR)

## Regler, materiel, styrkor

ops-rule-damage = realistiska skador
ops-rule-guidance = realistisk målsökning
ops-rule-no-crashes = inga haverier
ops-rule-time = kontinuerlig tid
ops-rule-weapons = realistiska F-117A-vapen
ops-rule-friendly-fire = vådabeskjutning
ops-rule-classic = klassiskt
ops-materiel = MATERIEL
ops-materiel-blue = BLÅ { $materiel } (+{ $income }/h)
ops-materiel-red = RÖD { $materiel } (+{ $income }/h)
ops-friendly = EGNA
ops-enemy = FIENDE
ops-section-forces = Styrkor
ops-places-held = Hållna platser
ops-materiel-row = Materiel
ops-income = Inkomst per timme
ops-income-from = från
ops-income-sources = { $base } + { $places } platser x { $per_place } + { $cities } städer x { $per_city }
ops-fighters-ground = Jaktplan på marken
ops-helicopters-ground = Helikoptrar på marken
ops-in-air = I luften
ops-in-air-value = { $fighters } jaktplan, { $strike } attackplan, { $helicopters } helikoptrar
ops-since-start = Sedan starten
ops-earned = intjänat
ops-repairs = reparationer
ops-operations = flygningar, anfall, insättningar
ops-new-aircraft = nya flygplan
ops-lost = förlorat (städer, piloter)
ops-section-how = Så fungerar det
ops-how =
    Varje sida tjänar materiel varje timme: ett grundbelopp, mer för varje plats den håller och för varje obruten stad bland dem. Bomba fiendens städer (INTERDIKTION) för att minska dess inkomst.
    AI:n betalar för allt den gör: reparationer, varje jaktflygning, anfall och egna insättningar. Har den ont om materiel förblir skadorna oreparerade och flygplanen stannar hemma.
    AI:n flyger bara de flygplan den har; nedskjutna är de borta, landade flyger de igen. Varje timme köper varje sida nya så länge materielen räcker.
    Du får aldrig ont om något: dina flygplan, dina baser (bombade eller inte) och helikoptrarna i dina egna insättningsordrar finns alltid.
ops-section-prices = Priser
ops-price-airfield = Reparation av flygfält
ops-price-sam = Reparation av SAM-ställning
ops-price-city = Stad återuppbyggd
ops-price-ship = Reparation av fartyg
ops-price-gun = Kanon eller MANPADS-grupp
ops-price-sortie = Jaktflygning
ops-price-raid = Anfall
ops-price-insertion = AI-insättning
ops-price-fighter = Nytt jaktplan
ops-price-helicopter = Ny helikopter
ops-price-city-lost = Stad förstörd (förlust)
ops-price-captured = Pilot tillfångatagen (förlust)

## Krigskartan

ops-click-to-watch = { $unit } - klicka för att titta
ops-section-legend = Teckenförklaring
ops-legend-friendly-airfield = Eget flygfält
ops-legend-enemy-airfield = Fientligt flygfält
ops-legend-city = Egen / fientlig stad
ops-watch-hint = Klicka på en enhet för att följa den live
ops-map-hint = Hjul: zoom, dra: panorera
ops-patriot = Patriot-batteri
ops-manned-by = Bemannad av { $pilot }
ops-manned-by-other = Bemannad av en annan spelare
ops-control = STYR
    .facts = Ta över dess konsol

## Debrief, slut, logg, chatt

ops-no-sortie = Ingen flygning ännu
ops-section-sortie = Flygning
ops-achieved = MÅLET UPPNÅTT
ops-not-achieved = MÅLET EJ UPPNÅTT
ops-waiting-raid = VÄNTAR PÅ ANFALLET
ops-waiting-insertion = VÄNTAR PÅ INSÄTTNINGEN
ops-flight-time = Flygtid { $mins } min
ops-kills = Nedskjutningar: { $air } flygplan, { $ground } på marken
ops-points = { $points } poäng
ops-points-bonus = { $points } poäng (package-bonus +{ $bonus })
ops-campaign-points = { $rank } - { $points } kampanjpoäng
ops-campaign-record = Kampanj: { $sorties } flygningar, { $won } mål uppnådda
ops-section-meanwhile = Under tiden
ops-victory = SEGER
    .text = Fienden håller inte längre något flygfält: operationsområdet är vårt.
ops-defeat = NEDERLAG
    .text = Inget eget flygfält finns kvar: operationsområdet är förlorat.
ops-end-day = Dag { $day }. Egna platser { $friendly }, fientliga platser { $enemy }.
ops-next-campaign = NÄSTA KAMPANJ STARTAR OM { $time }
ops-next-campaign-now = NÄSTA KAMPANJ STARTAR
ops-next-from-menu = Starta nästa kampanj från huvudmenyn.
ops-section-log = Kampanjlogg
ops-section-chat = Chatt
ops-no-messages = Inga meddelanden ännu
ops-message = Meddelande
    .hint = Skriv ett meddelande, Enter skickar
ops-send = SKICKA

## Briefing

ops-briefing = UPPDRAGSBRIEFING
ops-briefing-line = Från { $base } - { $km } km till målet - { $aircraft } - last: { $stores }
ops-route-locked = Rutten låst: { $pilots } flyger den
ops-route-threats = RUTT & HOT
ops-take-off = STARTA
ops-radars-shown = Visade radarstationer: upptäckta av egna flygplan och fortfarande intakta

## Spelare

ops-section-players = Spelare
ops-col-name = NAMN
ops-col-status = STATUS
ops-col-sorties = FLYGNINGAR
ops-col-points = POÄNG
ops-col-pilot = PILOT
ops-col-objectives = MÅL
ops-col-kills = NEDSKJ. LUFT/MARK
ops-col-medal = BÄSTA MEDALJ
ops-you = (du)
ops-host = VÄRD
ops-status-planning = Planerar { $title }
ops-status-board = Vid uppdragstavlan
ops-status-flying = Flyger { $title } - { $aircraft }
ops-status-reconnecting = Anslutningen bruten - flygplanet hålls { $seconds } s
ops-flown = { $flown } ({ $won } uppnådda)
ops-section-leaderboard = Resultattavla
ops-alone = Du flyger ensam. För att flyga tillsammans, bli värd för kampanjen via FLERSPELARE i huvudmenyn.
ops-rules = Den här kampanjens regler (behålls från starten): { $rules }
ops-no-sorties = Inga flygningar ännu
