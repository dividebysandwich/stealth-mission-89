# Kampaņas operāciju ekrāns (src/campaign/ops.rs).

ops-thunderstorms = PERKONAUDAS
ops-wx-short = WX { $now }, { $next } PLKST. { $at }
ops-wx-line = { $weather } Mainīsies uz: { $next }, plkst. { $at }.
ops-clock = { $day }. DIENA  { $time }
ops-landed-at = NOSĒDIES: { $base }
ops-ground-crew = Zemes apkalpe ir gatava
ops-rearm = PAPILDINĀT BRUŅOJUMU UN DEGVIELU
ops-end-sortie = BEIGT LIDOJUMU
ops-on-server = SERVERĪ: { $server }
ops-serving = DARBOJAS
ops-tab-missions = MISIJAS
ops-tab-map = KARA KARTE
ops-tab-air-defence = PRETGAISA AIZSARDZĪBA
ops-tab-forces = SPĒKI
ops-tab-players = SPĒLĒTĀJI
ops-tab-debrief = DEBRĪFINGS
ops-tab-log = ŽURNĀLS
ops-tab-chat = ČATS
ops-blue = ZILIE { $count }
ops-red = SARKANIE { $count }
ops-war-over = Karš ir beidzies
ops-running = Kara simulācija darbojas
ops-save-exit = SAGLABĀT UN IZIET
ops-disconnect = ATSAVIENOTIES
ops-leave-local = SAGLABĀT UN AIZIET NO KAMPAŅAS?
    .text = Karš apstājas, līdz jūs turpināsiet. Gaisā esošās lidmašīnas tiek zaudētas; notiekošie desanti sākas no jauna no savas bāzes.
ops-leave-host = SAGLABĀT UN PārTRAUKT KAMPAŅAS SERVERA DARBĪBU?
    .text = Pievienojušies spēlētāji tiek atvienoti, un karš apstājas, līdz jūs turpināsiet. Gaisā esošās lidmašīnas tiek zaudētas; notiekošie desanti sākas no jauna no savas bāzes.
ops-leave-client = ATSAVIENOTIES NO SERVERA?
    .text = Kampaņa turpinās serverī; šeit nekas netiek saglabāts. Jūsu lidojums beidzas.
ops-unit-destroyed = { $unit } iznīcināta: atpakaļ pie operācijām

## Misijas

ops-section-missions = Misijas
ops-threat = DRAUDS { $threat }
ops-no-missions = Nav misiju: fronte ir mierīga
ops-section-underway = Desanti un reidi
ops-enemy-liftoff = IENAIDNIEKS: { $place }: pacelšanās pēc { $mins } min
ops-liftoff = { $place }: pacelšanās pēc { $mins } min
ops-enemy-helis = IENAIDNIEKS: { $place }: tuvojas { $count } helikopteri
ops-helis = { $place }: ceļā { $count } helikopteri
ops-raid-sead = AI SEAD: { $place }
ops-raid-drones = DRONI: { $place }
ops-raid = REIDS uz lidlauku: { $place }
ops-raid-launch = { $raid }: starts pēc { $mins } min
ops-raid-on-way = { $raid }: ceļā { $count }
ops-resupply-ai = PAPILDINĀJUMS vienībai { $unit }: UH-60 ir ceļā
ops-awacs = IENAIDNIEKA AWACS gaisā vēl { $mins } min
ops-down = NOTRIEKTS: { $pilot } - glābšana { $mins } min laikā
ops-aboard = UZ KLĀJA: { $pilot } - ceļā mājup
ops-none-underway = Nekā nav
ops-ai-helicopter = AI helikopters to veic pēc { $time }, ja vien to nepaveic UH-60 pilots
ops-defences = Aizsardzība: { $sams } SAM pozīcijas, drauds { $threat }
ops-recon-intel = IZLŪKDATI
ops-order-insertion = PASŪTĪT DESANTU
    .facts = Lidojiet pavadībā, lai tos nosegtu, vai vediet tos ar UH-60
ops-from = No
ops-planned-by-group = Plāno grupa
ops-stores = { $aircraft } - iekārtas: { $stores }
ops-fly-uh60 = LIDOT AR UH-60
ops-accept = PIEŅEMT MISIJU
ops-planning = PLĀNO: { $pilots }
ops-flying = LIDO: { $pilots }
ops-package = PAKETE ar { $kinds }: izpildiet divus veidus šeit { $mins } min laikā, +{ $bonus } punkti katrs
ops-sead-on-way = AI SEAD ir ceļā
ops-request-sead = PIEPRASĪT AI SEAD ({ $cost } materiālie resursi, { $aircraft } lidmašīnas)
    .facts = Draudzīgas trieciena lidmašīnas pirms jums uzbrūk šīs vietas SAM pozīcijai
    .disabled = Nepietiek materiālo resursu vai lidmašīnu uz zemes (skat. SPĒKI)

## Noteikumi, materiālie resursi, spēki

ops-rule-damage = reālistiski bojājumi
ops-rule-guidance = reālistiska vadība
ops-rule-no-crashes = bez avārijām
ops-rule-time = nepārtraukts laiks
ops-rule-weapons = reālistisks F-117A bruņojums
ops-rule-friendly-fire = draudzīgā uguns
ops-rule-classic = klasiskie
ops-materiel = MATERIĀLIE RESURSI
ops-materiel-blue = ZILIE { $materiel } (+{ $income }/h)
ops-materiel-red = SARKANIE { $materiel } (+{ $income }/h)
ops-friendly = DRAUDZĪGIE
ops-enemy = IENAIDNIEKS
ops-section-forces = Spēki
ops-places-held = Turētās vietas
ops-materiel-row = Materiālie resursi
ops-income = Ienākumi stundā
ops-income-from = no
ops-income-sources = { $base } + { $places } vietas x { $per_place } + { $cities } pilsētas x { $per_city }
ops-fighters-ground = Iznīcinātāji uz zemes
ops-helicopters-ground = Helikopteri uz zemes
ops-in-air = Gaisā
ops-in-air-value = { $fighters } iznīcinātāji, { $strike } trieciena, { $helicopters } helikopteri
ops-since-start = Kopš sākuma
ops-earned = nopelnīts
ops-repairs = remonti
ops-operations = lidojumi, reidi, desanti
ops-new-aircraft = jaunas lidmašīnas
ops-lost = zaudēts (pilsētas, piloti)
ops-section-how = Kā tas darbojas
ops-how =
    Katra puse katru stundu pelna materiālos resursus: pamatsummu, vairāk par katru turēto vietu un par katru neskarto pilsētu starp tām. Bombardējiet ienaidnieka pilsētas (PIEGĀDES CEĻU PĀRTRAUKŠANA), lai samazinātu tā ienākumus.
    AI maksā par visu, ko dara: remontiem, katru iznīcinātāju lidojumu, reidiem un saviem desantiem. Ja resursu trūkst, bojājumi paliek nelaboti un lidmašīnas paliek mājās.
    AI lido tikai ar tām lidmašīnām, kas tam ir; notriektas tās ir zudušas, nosēdušās lido atkal. Katru stundu katra puse nopērk jaunas, kamēr pietiek resursu.
    Jums nekad netrūkst: jūsu lidmašīnas, jūsu bāzes (bombardētas vai nē) un jūsu pašu desanta pasūtījumu helikopteri vienmēr ir pieejami.
ops-section-prices = Cenas
ops-price-airfield = Lidlauka remonts
ops-price-sam = SAM pozīcijas remonts
ops-price-city = Pilsētas atjaunošana
ops-price-ship = Kuģa remonts
ops-price-gun = Lielgabals vai MANPADS komanda
ops-price-sortie = Iznīcinātāja lidojums
ops-price-raid = Reids
ops-price-insertion = AI desants
ops-price-fighter = Jauns iznīcinātājs
ops-price-helicopter = Jauns helikopters
ops-price-city-lost = Pilsēta iznīcināta (zaudēta)
ops-price-captured = Pilots sagūstīts (zaudēts)

## Kara karte

ops-click-to-watch = { $unit } - klikšķiniet, lai vērotu
ops-section-legend = Apzīmējumi
ops-legend-friendly-airfield = Draudzīgs lidlauks
ops-legend-enemy-airfield = Ienaidnieka lidlauks
ops-legend-city = Draudzīga / ienaidnieka pilsēta
ops-legend-airfield = Savs / ienaidnieka lidlauks
ops-legend-insertion = Desanta maršruts
ops-watch-hint = Klikšķiniet uz vienības, lai to vērotu tiešraidē
ops-map-hint = Ritenītis: tālummaiņa, vilkšana: pārbīde
ops-patriot = Patriot baterija
ops-drone-center = Dronu vadības punkts
ops-manned-by = Apkalpo: { $pilot }
ops-manned-by-other = Apkalpo cits spēlētājs
ops-control = VADĪT
    .facts = Pārņemt tās konsoli

## Debrīfings, beigas, žurnāls, čats

ops-no-sortie = Vēl nav veikts neviens lidojums
ops-section-sortie = Lidojums
ops-achieved = UZDEVUMS IZPILDĪTS
ops-not-achieved = UZDEVUMS NEIZPILDĪTS
ops-waiting-raid = GAIDA REIDU
ops-waiting-insertion = GAIDA DESANTU
ops-flight-time = Lidojuma ilgums { $mins } min
ops-kills = Uzvaras: { $air } lidmašīnas, { $ground } uz zemes
ops-points = { $points } punkti
ops-points-bonus = { $points } punkti (paketes bonuss +{ $bonus })
ops-campaign-points = { $rank } - { $points } kampaņas punkti
ops-campaign-record = Kampaņa: { $sorties } lidojumi, { $won } uzdevumi izpildīti
ops-section-meanwhile = Pa to laiku
ops-victory = UZVARA
    .text = Ienaidniekam vairs nav neviena lidlauka: operāciju teātris ir mūsu.
ops-defeat = SAKĀVE
    .text = Draudzīgo lidlauku vairs nav: operāciju teātris zaudēts.
ops-end-day = { $day }. diena. Draudzīgās vietas { $friendly }, ienaidnieka vietas { $enemy }.
ops-next-campaign = NĀKAMĀ KAMPAŅA SĀKSIES PĒC { $time }
ops-next-campaign-now = NĀKAMĀ KAMPAŅA SĀKAS
ops-next-from-menu = Sāciet nākamo kampaņu no galvenās izvēlnes.
ops-section-log = Kampaņas žurnāls
ops-section-chat = Čats
ops-no-messages = Vēl nav ziņojumu
ops-message = Ziņojums
    .hint = Ierakstiet ziņojumu, Enter nosūta
ops-send = NOSŪTĪT

## Brīfings

ops-briefing = MISIJAS BRĪFINGS
ops-briefing-line = No { $base } - { $km } km līdz mērķim - { $aircraft } - iekārtas: { $stores }
ops-route-locked = Maršruts slēgts: lido { $pilots }
ops-route-threats = MARŠRUTS UN DRAUDI
ops-take-off = PACELTIES
ops-radars-shown = Rādītie radari: draudzīgo lidmašīnu atklātie, vēl darbojas

## Spēlētāji

ops-section-players = Spēlētāji
ops-col-name = VĀRDS
ops-col-status = STATUSS
ops-col-sorties = LIDOJUMI
ops-col-points = PUNKTI
ops-col-pilot = PILOTS
ops-col-objectives = UZDEVUMI
ops-col-kills = UZVARAS GAISĀ/ZEMĒ
ops-col-medal = LABĀKĀ MEDAĻA
ops-you = (jūs)
ops-host = SAIMNIEKS
ops-status-planning = Plāno: { $title }
ops-status-board = Misiju dēlī
ops-status-flying = Lido: { $title } - { $aircraft }
ops-status-reconnecting = Savienojums zaudēts - lidmašīna turēta { $seconds } s
ops-flown = { $flown } ({ $won } izpildīti)
ops-section-leaderboard = Rezultātu tabula
ops-alone = Jūs lidojat viens. Lai lidotu kopā, rīkojiet šo kampaņu: galvenajā izvēlnē izvēlieties MULTIPLAYER.
ops-rules = Šīs kampaņas noteikumi (saglabāti no sākuma): { $rules }
ops-no-sorties = Vēl nav veikts neviens lidojums
