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
ops-fly-with-ai = VZLETI
ops-take-control = PREVZEMI UPRAVLJANJE
    .facts = Nadaljuj z letalom UI od tam, kjer je
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
ops-rule-airframe-losses = izgube stanejo vire
ops-rule-no-ai-missions = brez prijateljskih misij UI
ops-rule-classic = klasično
ops-materiel = MATERIAL
ops-materiel-blue = MODRI { $materiel } (+{ $income }/h)
ops-materiel-red = RDEČI { $materiel } (+{ $income }/h)
ops-crew-tempo = TEMPO SOVRAŽNIKA ×{ $pace }
ops-due = T-{ $mins } min
ops-due-now = ZDAJ
ops-front-blue = FRONTA: MODRI NAPREDUJEJO
ops-front-red = FRONTA: RDEČI NAPREDUJEJO
ops-front-even = FRONTA: STOJI
ops-operation = Zavzetje { $place }:
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
ops-how-repairs = Vsaka stran hkrati popravlja le nekaj položajev (več z več nepoškodovanimi mesti), najprej tiste najbližje fronti, in položaj, ki je kmalu po popravilu znova uničen, vsakič traja dlje.
ops-how-airframes = Izgube stanejo vire: vsako letalo, ki ga pilot izgubi, je en lovec ali helikopter manj za lastno UI.
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
ops-legend-airfield = Lastno / sovražno letališče
ops-legend-insertion = Pot vstavitve
ops-watch-hint = Kliknite enoto za ogled v živo
ops-map-hint = Kolesce: povečava, vlečenje: pomik
ops-patriot = Baterija Patriot
ops-drone-center = Poveljniško mesto dronov
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
ops-section-effects = Tvoj vpliv na vojno
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
ops-ai-load = Helikopter UI prevzame tovor čez { $time }, če tega ne stori pilot UH-60
ops-ai-takes-off = Vzlet čez { $time }
ops-ai-aircraft = UI { $aircraft }
ops-cargo-in = Tovor { $delivered } od { $loads } dostavljen - heliport čaka še { $time } h
ops-rule-classic-terrain = klasični teren
ops-rule-classic-ai-flight = klasični let UI
ops-layers-hint = SLOJI (zgoraj desno): kaj prikazuje zemljevid
ops-map-tab = KARTA
ops-on-radio = V RADIU
ops-mute = UTIŠAJ
    .facts = Slišati (ali ne) tega igralca v radiu
ops-unmute = SLIŠI
