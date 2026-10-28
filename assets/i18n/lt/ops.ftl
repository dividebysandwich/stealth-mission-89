# Kampanijos operacijų ekranas (src/campaign/ops.rs).

ops-thunderstorms = GRIAUSTINIAI
ops-wx-short = ORAI { $now }, { $next } { $at }
ops-wx-line = { $weather } Keisis į: { $next }, { $at }.
ops-clock = { $day } DIENA  { $time }
ops-landed-at = NUSILEISTA: { $base }
ops-ground-crew = Antžeminė įgula pasirengusi
ops-rearm = PAPILDYTI GINKLUS IR KURĄ
ops-end-sortie = BAIGTI SKRYDĮ
ops-on-server = SERVERIS: { $server }
ops-serving = PASLAUGA VEIKIA
ops-tab-missions = MISIJOS
ops-tab-map = KARO ŽEMĖLAPIS
ops-tab-air-defence = PRIEŠLĖKTUVINĖ GYNYBA
ops-tab-forces = PAJĖGOS
ops-tab-players = ŽAIDĖJAI
ops-tab-debrief = DEBRIFINGAS
ops-tab-log = ŽURNALAS
ops-tab-chat = POKALBIS
ops-blue = MĖLYNIEJI { $count }
ops-red = RAUDONIEJI { $count }
ops-war-over = Karas baigtas
ops-running = Karo simuliacija vyksta
ops-save-exit = IŠSAUGOTI IR IŠEITI
ops-disconnect = ATSIJUNGTI
ops-leave-local = IŠSAUGOTI IR PALIKTI KAMPANIJĄ?
    .text = Karas sustoja, kol jį tęsite. Ore esantys orlaiviai prarandami; vykstantys desantavimai prasideda iš naujo nuo savo bazės.
ops-leave-host = IŠSAUGOTI IR NUSTOTI TEIKTI KAMPANIJĄ?
    .text = Prisijungę žaidėjai atjungiami, o karas sustoja, kol jį tęsite. Ore esantys orlaiviai prarandami; vykstantys desantavimai prasideda iš naujo nuo savo bazės.
ops-leave-client = ATSIJUNGTI NUO SERVERIO?
    .text = Kampanija tęsiasi serveryje; čia niekas neišsaugoma. Jūsų skrydis baigiasi.
ops-unit-destroyed = Dalinys „{ $unit }“ sunaikintas: grįžtama į operacijas

## Misijos

ops-section-missions = Misijos
ops-threat = GRĖSMĖ { $threat }
ops-no-missions = Misijų nėra: fronte ramu
ops-section-underway = Desantavimai ir reidai
ops-enemy-liftoff = PRIEŠAS, vietovė: { $place }. Pakilimas po { $mins } min
ops-liftoff = { $place }: pakilimas po { $mins } min
ops-enemy-helis = PRIEŠAS, vietovė: { $place }. Sraigtasparnių artėja: { $count }
ops-helis = { $place }: sraigtasparnių pakeliui: { $count }
ops-raid-sead = DI SEAD, vietovė: { $place }
ops-raid-drones = DRONAI, vietovė: { $place }
ops-raid = REIDAS į aerodromą { $place }
ops-raid-launch = { $raid }: paleidimas po { $mins } min
ops-raid-on-way = { $raid }: pakeliui { $count }
ops-resupply-ai = PAPILDYMAS daliniui „{ $unit }“: UH-60 pakeliui
ops-awacs = PRIEŠO AWACS ore dar { $mins } min
ops-down = NUMUŠTAS: { $pilot } - gelbėjimas per { $mins } min
ops-aboard = SRAIGTASPARNYJE: { $pilot } - grįžta namo
ops-none-underway = Nieko nevyksta
ops-ai-helicopter = DI sraigtasparnis ją atliks po { $time }, jei neatliks UH-60 pilotas
ops-defences = Gynyba: SAM pozicijų { $sams }, grėsmė { $threat }
ops-recon-intel = ŽVALGYBOS DUOMENYS
ops-order-insertion = PASKIRTI DESANTAVIMĄ
    .facts = Skraidinkite palydos orlaivį jiems dengti arba vesti juos UH-60
ops-from = Iš
ops-planned-by-group = Planuoja grupė
ops-stores = { $aircraft } - ginkluotė: { $stores }
ops-fly-uh60 = SKRISTI UH-60
ops-accept = PRIIMTI MISIJĄ
ops-planning = PLANUOJA: { $pilots }
ops-flying = SKRENDA: { $pilots }
ops-package = GRUPĖ su užduotimis ({ $kinds }): įvykdykite čia dvi rūšis per { $mins } min, +{ $bonus } taškų už kiekvieną
ops-sead-on-way = DI SEAD jau pakeliui
ops-request-sead = UŽSAKYTI DI SEAD (išteklių: { $cost }, orlaivių: { $aircraft })
    .facts = Draugiški smogiamieji orlaiviai pirmiau jūsų puola šios vietovės SAM poziciją
    .disabled = Nepakanka išteklių arba orlaivių ant žemės (žr. PAJĖGOS)

## Taisyklės, ištekliai, pajėgos

ops-rule-damage = realistiški pažeidimai
ops-rule-guidance = realistiškas valdymas
ops-rule-no-crashes = be avarijų
ops-rule-time = nuolatinis laikas
ops-rule-weapons = realistiški F-117A ginklai
ops-rule-friendly-fire = draugiška ugnis
ops-rule-classic = klasikinės
ops-materiel = IŠTEKLIAI
ops-materiel-blue = MĖLYNIEJI { $materiel } (+{ $income }/val.)
ops-materiel-red = RAUDONIEJI { $materiel } (+{ $income }/val.)
ops-friendly = DRAUGIŠKA
ops-enemy = PRIEŠAS
ops-section-forces = Pajėgos
ops-places-held = Valdomos vietovės
ops-materiel-row = Ištekliai
ops-income = Pajamos per valandą
ops-income-from = iš
ops-income-sources = { $base } + vietovių { $places } x { $per_place } + miestų { $cities } x { $per_city }
ops-fighters-ground = Naikintuvai ant žemės
ops-helicopters-ground = Sraigtasparniai ant žemės
ops-in-air = Ore
ops-in-air-value = naikintuvų { $fighters }, smogiamųjų { $strike }, sraigtasparnių { $helicopters }
ops-since-start = Nuo pradžios
ops-earned = uždirbta
ops-repairs = remontas
ops-operations = skrydžiai, reidai, desantavimai
ops-new-aircraft = nauji orlaiviai
ops-lost = prarasta (miestai, pilotai)
ops-section-how = Kaip tai veikia
ops-how =
    Kiekviena pusė kas valandą gauna išteklių: bazinį kiekį, daugiau už kiekvieną valdomą vietovę ir už kiekvieną nesugriautą miestą tarp jų. Bombarduokite priešo miestus (INTERDIKCIJA), kad sumažintumėte jo pajamas.
    DI už viską moka: remontą, kiekvieną naikintuvų skrydį, reidus ir savo desantavimus. Trūkstant išteklių, jo pažeidimai lieka neatitaisyti, o orlaiviai lieka namie.
    DI skraido tik tais orlaiviais, kuriuos turi; numušti jie prarasti, nusileidę gali skristi vėl. Kas valandą kiekviena pusė perka naujus, kol užtenka išteklių.
    Jums išteklių niekada netrūksta: jūsų orlaiviai, bazės (subombarduotos ar ne) ir jūsų paskirtų desantavimų sraigtasparniai visada yra.
ops-section-prices = Kainos
ops-price-airfield = Aerodromo remontas
ops-price-sam = SAM pozicijos remontas
ops-price-city = Miesto atstatymas
ops-price-ship = Laivo remontas
ops-price-gun = Pabūklas arba MANPADS komanda
ops-price-sortie = Naikintuvo skrydis
ops-price-raid = Reidas
ops-price-insertion = DI desantavimas
ops-price-fighter = Naujas naikintuvas
ops-price-helicopter = Naujas sraigtasparnis
ops-price-city-lost = Miestas sunaikintas (prarasta)
ops-price-captured = Pilotas paimtas į nelaisvę (prarasta)

## Karo žemėlapis

ops-click-to-watch = { $unit } - spustelėkite stebėti
ops-section-legend = Legenda
ops-legend-friendly-airfield = Draugiškas aerodromas
ops-legend-enemy-airfield = Priešo aerodromas
ops-legend-city = Draugiškas / priešo miestas
ops-watch-hint = Spustelėkite dalinį, kad stebėtumėte jį gyvai
ops-map-hint = Ratukas: didinti, vilkimas: slinkti
ops-patriot = Patriot baterija
ops-manned-by = Valdo: { $pilot }
ops-manned-by-other = Valdo kitas žaidėjas
ops-control = VALDYTI
    .facts = Perimti jos pultą

## Debrifingas, pabaiga, žurnalas, pokalbis

ops-no-sortie = Dar neatlikta jokio skrydžio
ops-section-sortie = Skrydis
ops-achieved = UŽDUOTIS ĮVYKDYTA
ops-not-achieved = UŽDUOTIS NEĮVYKDYTA
ops-waiting-raid = LAUKIAMA REIDO
ops-waiting-insertion = LAUKIAMA DESANTAVIMO
ops-flight-time = Skrydžio trukmė: { $mins } min
ops-kills = Pergalės: ore { $air }, ant žemės { $ground }
ops-points = Taškai: { $points }
ops-points-bonus = Taškai: { $points } (grupės priedas +{ $bonus })
ops-campaign-points = { $rank } - kampanijos taškai: { $points }
ops-campaign-record = Kampanija: skrydžių { $sorties }, įvykdytų užduočių { $won }
ops-section-meanwhile = Tuo metu
ops-victory = PERGALĖ
    .text = Priešas nebeturi nė vieno aerodromo: operacijų teatras mūsų.
ops-defeat = PRALAIMĖJIMAS
    .text = Nebeliko nė vieno draugiško aerodromo: operacijų teatras prarastas.
ops-end-day = { $day } diena. Draugiškos vietovės: { $friendly }, priešo vietovės: { $enemy }.
ops-next-campaign = KITA KAMPANIJA PRASIDĖS PO { $time }
ops-next-campaign-now = KITA KAMPANIJA PRASIDEDA
ops-next-from-menu = Kitą kampaniją pradėkite iš pagrindinio meniu.
ops-section-log = Kampanijos žurnalas
ops-section-chat = Pokalbis
ops-no-messages = Žinučių dar nėra
ops-message = Žinutė
    .hint = Įveskite žinutę, Enter išsiunčia
ops-send = SIŲSTI

## Brifingas

ops-briefing = MISIJOS BRIFINGAS
ops-briefing-line = Iš { $base } - iki taikinio { $km } km - { $aircraft } - ginkluotė: { $stores }
ops-route-locked = Maršrutas užrakintas: skrenda { $pilots }
ops-route-threats = MARŠRUTAS IR GRĖSMĖS
ops-take-off = PAKILTI
ops-radars-shown = Rodomi radarai: draugiškų orlaivių aptikti, dar veikiantys

## Žaidėjai

ops-section-players = Žaidėjai
ops-col-name = VARDAS
ops-col-status = BŪSENA
ops-col-sorties = SKRYDŽIAI
ops-col-points = TAŠKAI
ops-col-pilot = PILOTAS
ops-col-objectives = UŽDUOTYS
ops-col-kills = PERGALĖS ORE/ŽEMĖJE
ops-col-medal = GERIAUSIAS APDOVANOJIMAS
ops-you = (jūs)
ops-host = ŠEIMININKAS
ops-status-planning = Planuoja: { $title }
ops-status-board = Misijų lentoje
ops-status-flying = Skrenda: { $title } - { $aircraft }
ops-status-reconnecting = Ryšys prarastas - orlaivis laikomas { $seconds } s
ops-flown = { $flown } (įvykdyta: { $won })
ops-section-leaderboard = Lyderių lentelė
ops-alone = Skrendate vienas. Norėdami skristi kartu, pagrindiniame meniu per DAUGIAŽAIDĖ priimkite šią kampaniją.
ops-rules = Šios kampanijos taisyklės (galioja nuo jos pradžios): { $rules }
ops-no-sorties = Dar neatlikta jokių skrydžių
