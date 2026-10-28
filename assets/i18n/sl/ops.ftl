# Zaslon operacij kampanje (src/campaign/ops.rs).

ops-thunderstorms = NEVIHTE
ops-wx-short = VREME { $now }, { $next } OB { $at }
ops-wx-line = { $weather } Sprememba: { $next } ob { $at }.
ops-clock = DAN { $day }  { $time }
ops-landed-at = PRISTALI NA { $base }
ops-ground-crew = Zemeljska posadka je pripravljena
ops-rearm = OBOROŽI IN NATOČI GORIVO
ops-end-sortie = KONČAJ BOJNI LET
ops-on-server = NA { $server }
ops-serving = GOSTITELJ
ops-tab-missions = MISIJE
ops-tab-map = VOJNA KARTA
ops-tab-air-defence = PROTIZRAČNA OBRAMBA
ops-tab-forces = SILE
ops-tab-players = IGRALCI
ops-tab-debrief = DEBRIFING
ops-tab-log = DNEVNIK
ops-tab-chat = KLEPET
ops-blue = MODRI { $count }
ops-red = RDEČI { $count }
ops-war-over = Vojne je konec
ops-running = Simulacija vojne teče
ops-save-exit = SHRANI IN IZHOD
ops-disconnect = PREKINI POVEZAVO
ops-leave-local = SHRANIM IN ZAPUSTIM KAMPANJO?
    .text = Vojna se ustavi, dokler je ne nadaljujete. Letala v zraku so izgubljena; izkrcanja v teku se začnejo znova z njihove baze.
ops-leave-host = SHRANIM IN PRENEHAM GOSTITI KAMPANJO?
    .text = Pridruženi igralci so odklopljeni in vojna se ustavi, dokler je ne nadaljujete. Letala v zraku so izgubljena; izkrcanja v teku se začnejo znova z njihove baze.
ops-leave-client = PREKINEM POVEZAVO S STREŽNIKOM?
    .text = Kampanja se na strežniku nadaljuje; tukaj se nič ne shrani. Vaš bojni let se konča.
ops-unit-destroyed = { $unit } uničen: nazaj na operacije

## Misije

ops-section-missions = Misije
ops-threat = NEVARNOST { $threat }
ops-no-missions = Ni misij: fronta je mirna
ops-section-underway = Izkrcanja in napadi
ops-enemy-liftoff = SOVRAŽNIK pri { $place }: vzlet čez { $mins } min
ops-liftoff = { $place }: vzlet čez { $mins } min
ops-enemy-helis = SOVRAŽNIK pri { $place }: { $count } helikopterjev se približuje
ops-helis = { $place }: { $count } helikopterjev na poti
ops-raid-sead = SEAD umetne inteligence pri { $place }
ops-raid-drones = DRONI pri { $place }
ops-raid = NAPAD na letališče { $place }
ops-raid-launch = { $raid }: vzlet čez { $mins } min
ops-raid-on-way = { $raid }: { $count } na poti
ops-resupply-ai = OSKRBA enote { $unit }: UH-60 na poti
ops-awacs = SOVRAŽNI AWACS v zraku še { $mins } min
ops-down = SESTRELJEN: { $pilot } - reševanje v { $mins } min
ops-aboard = NA KROVU: { $pilot } - na poti domov
ops-none-underway = Nič v teku
ops-ai-helicopter = Helikopter umetne inteligence ga opravi čez { $time }, razen če ga prej opravi pilot UH-60
ops-defences = Obramba: { $sams } položajev SAM, nevarnost { $threat }
ops-recon-intel = IZVIDNIŠKI PODATKI
ops-order-insertion = ODREDI IZKRCANJE
    .facts = Letite spremstvo, ki jih krije, ali jih vodite z UH-60
ops-from = Z
ops-planned-by-group = Načrtuje skupina
ops-stores = { $aircraft } - oborožitev: { $stores }
ops-fly-uh60 = LETITE Z UH-60
ops-accept = SPREJMI MISIJO
ops-planning = NAČRTUJE: { $pilots }
ops-flying = LETI: { $pilots }
ops-package = PAKET z vrstami { $kinds }: dosežite dve vrsti tukaj v { $mins } min, +{ $bonus } točk za vsako
ops-sead-on-way = SEAD umetne inteligence je na poti
ops-request-sead = ZAHTEVAJ SEAD UMETNE INTELIGENCE ({ $cost } materiala, { $aircraft } letal)
    .facts = Zavezniška udarna letala napadejo položaj SAM tega kraja pred vami
    .disabled = Premalo materiala ali letal na tleh (glejte SILE)

## Pravila, material, sile

ops-rule-damage = realistična škoda
ops-rule-guidance = realistično vodenje
ops-rule-no-crashes = brez strmoglavljenj
ops-rule-time = neprekinjen čas
ops-rule-weapons = realistično orožje F-117A
ops-rule-friendly-fire = ogenj po svojih
ops-rule-classic = klasično
ops-materiel = MATERIAL
ops-materiel-blue = MODRI { $materiel } (+{ $income }/h)
ops-materiel-red = RDEČI { $materiel } (+{ $income }/h)
ops-friendly = ZAVEZNIŠKO
ops-enemy = SOVRAŽNO
ops-section-forces = Sile
ops-places-held = Držani kraji
ops-materiel-row = Material
ops-income = Dohodek na uro
ops-income-from = iz
ops-income-sources = { $base } + { $places } krajev x { $per_place } + { $cities } mest x { $per_city }
ops-fighters-ground = Lovci na tleh
ops-helicopters-ground = Helikopterji na tleh
ops-in-air = V zraku
ops-in-air-value = { $fighters } lovcev, { $strike } udarnih, { $helicopters } helikopterjev
ops-since-start = Od začetka
ops-earned = zaslužen
ops-repairs = popravila
ops-operations = bojni leti, napadi, izkrcanja
ops-new-aircraft = nova letala
ops-lost = izgubljeno (mesta, piloti)
ops-section-how = Kako deluje
ops-how =
    Vsaka stran vsako uro pridobi material: osnovni znesek, več za vsak kraj, ki ga drži, in za vsako nepoškodovano mesto med njimi. Bombardirajte sovražnikova mesta (INTERDIKCIJA), da mu zmanjšate dohodek.
    Umetna inteligenca plača vse, kar naredi: popravila, vsak bojni let lovcev, napade in lastna izkrcanja. Če ji zmanjka materiala, njena škoda ostane nepopravljena in njena letala ostanejo doma.
    Umetna inteligenca leti samo z letali, ki jih ima; sestreljena so izgubljena, pristala spet letijo. Vsako uro vsaka stran kupi nova, dokler material zadošča.
    Vam nikoli ne zmanjka: vaša letala, vaše baze (bombardirane ali ne) in helikopterji vaših izkrcanj so vedno na voljo.
ops-section-prices = Cene
ops-price-airfield = Popravilo letališča
ops-price-sam = Popravilo položaja SAM
ops-price-city = Obnova mesta
ops-price-ship = Popravilo ladje
ops-price-gun = Top ali ekipa MANPADS
ops-price-sortie = Bojni let lovcev
ops-price-raid = Napad
ops-price-insertion = Izkrcanje umetne inteligence
ops-price-fighter = Nov lovec
ops-price-helicopter = Nov helikopter
ops-price-city-lost = Mesto uničeno (izguba)
ops-price-captured = Pilot ujet (izguba)

## Vojna karta

ops-click-to-watch = { $unit } - kliknite za ogled
ops-section-legend = Legenda
ops-legend-friendly-airfield = Zavezniško letališče
ops-legend-enemy-airfield = Sovražno letališče
ops-legend-city = Zavezniško / sovražno mesto
ops-watch-hint = Kliknite enoto za ogled v živo
ops-map-hint = Kolesce: povečava, vlečenje: pomik
ops-patriot = Baterija Patriot
ops-manned-by = Upravlja: { $pilot }
ops-manned-by-other = Upravlja drug igralec
ops-control = NADZOR
    .facts = Prevzemite njeno konzolo

## Debrifing, konec, dnevnik, klepet

ops-no-sortie = Še noben bojni let
ops-section-sortie = Bojni let
ops-achieved = CILJ DOSEŽEN
ops-not-achieved = CILJ NI DOSEŽEN
ops-waiting-raid = ČAKANJE NA NAPAD
ops-waiting-insertion = ČAKANJE NA IZKRCANJE
ops-flight-time = Čas leta { $mins } min
ops-kills = Zadetki: { $air } letal, { $ground } na tleh
ops-points = { $points } točk
ops-points-bonus = { $points } točk (bonus paketa +{ $bonus })
ops-campaign-points = { $rank } - { $points } točk kampanje
ops-campaign-record = Kampanja: { $sorties } bojnih letov, { $won } doseženih ciljev
ops-section-meanwhile = Medtem
ops-victory = ZMAGA
    .text = Sovražnik nima več nobenega letališča: bojišče je naše.
ops-defeat = PORAZ
    .text = Nobeno zavezniško letališče ni več ostalo: bojišče je izgubljeno.
ops-end-day = Dan { $day }. Zavezniški kraji { $friendly }, sovražni kraji { $enemy }.
ops-next-campaign = NASLEDNJA KAMPANJA SE ZAČNE ČEZ { $time }
ops-next-campaign-now = NASLEDNJA KAMPANJA SE ZAČENJA
ops-next-from-menu = Naslednjo kampanjo začnite iz glavnega menija.
ops-section-log = Dnevnik kampanje
ops-section-chat = Klepet
ops-no-messages = Še ni sporočil
ops-message = Sporočilo
    .hint = Vtipkajte sporočilo, Enter pošlje
ops-send = POŠLJI

## Brifing

ops-briefing = BRIFING MISIJE
ops-briefing-line = Z { $base } - { $km } km do cilja - { $aircraft } - oborožitev: { $stores }
ops-route-locked = Pot zaklenjena: leti { $pilots }
ops-route-threats = POT IN NEVARNOSTI
ops-take-off = VZLETITE
ops-radars-shown = Prikazani radarji: odkriti z zavezniškimi letali, še delujoči

## Igralci

ops-section-players = Igralci
ops-col-name = IME
ops-col-status = STANJE
ops-col-sorties = BOJNI LETI
ops-col-points = TOČKE
ops-col-pilot = PILOT
ops-col-objectives = CILJI
ops-col-kills = ZADETKI ZRAK/TLA
ops-col-medal = NAJBOLJŠA MEDALJA
ops-you = (vi)
ops-host = GOSTITELJ
ops-status-planning = Načrtuje: { $title }
ops-status-board = Na tabli misij
ops-status-flying = Leti: { $title } - { $aircraft }
ops-status-reconnecting = Povezava izgubljena - letalo zadržano { $seconds } s
ops-flown = { $flown } ({ $won } doseženih)
ops-section-leaderboard = Lestvica
ops-alone = Letite sami. Za skupni let gostite to kampanjo iz VEČ IGRALCEV v glavnem meniju.
ops-rules = Pravila te kampanje (ohranjena od začetka): { $rules }
ops-no-sorties = Še noben bojni let
