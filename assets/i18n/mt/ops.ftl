# L-iskrin tal-Ops tal-kampanja (src/campaign/ops.rs).

ops-thunderstorms = MALTEMPATI
ops-wx-short = WX { $now }, { $next } FI { $at }
ops-wx-line = { $weather } Tibdil għal { $next } fi { $at }.
ops-clock = JUM { $day }  { $time }
ops-landed-at = INŻILT F'{ $base }
ops-ground-crew = L-ekwipaġġ tal-art lest
ops-rearm = ERĠA' ARMI U MIMLI BIL-FUEL
ops-end-sortie = AGĦLAQ IT-TITJIRA
ops-on-server = FUQ { $server }
ops-serving = QED JOSPITA
ops-tab-missions = MISSJONIJIET
ops-tab-map = MAPPA TAL-GWERRA
ops-tab-flights = TITJIRIET
ops-tab-air-defence = DIFIŻA TAL-AJRU
ops-tab-forces = FORZI
ops-tab-players = PLEJERS
ops-tab-debrief = DEBRIEFING
ops-tab-log = ĠURNAL
ops-tab-chat = CHAT
ops-top-campaign = KAMPANJA
ops-view-war = GWERRA
ops-view-ground = ART
ops-view-weather = TEMP
ops-bar-missions = Missjonijiet fuq il-bord: { $count }
ops-bar-hint = Ikklikkja fuq marka ta' mira jew MISSJONIJIET biex tagħżel, fuq unità biex tarahom live
ops-filter-all = KOLLHA
ops-filter-attack = ATTAKK
ops-filter-support = APPOĠĠ
ops-watch = ARA
ops-blue = BLU { $count }
ops-red = AĦMAR { $count }
ops-war-over = Il-gwerra spiċċat
ops-running = Is-simulazzjoni tal-gwerra għaddejja
ops-save-exit = IĦŻEN U ARĠA
ops-disconnect = SKONNETTJA
ops-leave-local = TIĦŻEN U TIĦLI L-KAMPANJA?
    .text = Il-gwerra tieqaf sakemm tkompliha. L-inġenji tal-ajru fl-ajru jintilfu; l-inserzjonijiet għaddejjin jerġgħu jibdew mill-bażi tagħhom.
ops-leave-host = TIĦŻEN U TWAQQAF L-OSPITAR TAL-KAMPANJA?
    .text = Il-plejers li ngħaqdu jiġu skonnettjati u l-gwerra tieqaf sakemm tkompliha. L-inġenji tal-ajru fl-ajru jintilfu; l-inserzjonijiet għaddejjin jerġgħu jibdew mill-bażi tagħhom.
ops-leave-client = TISKONNETTJA MIS-SERVER?
    .text = Il-kampanja tkompli fuq is-server; xejn ma jinħażen hawn. It-titjira tiegħek tintemm.
ops-unit-destroyed = { $unit } meqrud: lura għall-operazzjonijiet

## Missjonijiet

ops-section-missions = Missjonijiet
ops-threat = THEDDIDA { $threat }
ops-no-missions = L-ebda missjoni: il-front kwiet
ops-ai-helicopter = Ħelikopter tal-AI jieħu ħsiebu fi { $time } sakemm pilota tal-UH-60 ma jagħmilx dan
ops-defences = Difiżi: { $sams } siti SAM, theddida { $threat }
ops-recon-intel = INTEL TAR-RECON
ops-order-insertion = ORDNA INSERZJONI
    .facts = Itir l-escort biex tkoprihom, jew mexxihom bl-UH-60
ops-from = Minn
ops-planned-by-group = Ippjanat mill-grupp
ops-stores = { $aircraft } - armamenti: { $stores }
ops-fly-uh60 = ITIR L-UH-60
ops-accept = AĊĊETTA L-MISSJONI
ops-fly-with-ai = TLUQ
ops-take-control = IEĦU L-KONTROLL
    .facts = Kompli ttir wieħed mill-ajruplani tal-IA minn fejn qiegħed
ops-join-gunner = INGĦAQAD BĦALA GUNNER
    .facts = Oqgħod fil-console tal-gunner ta' dan l-ajruplan: dawwar il-pod tal-immirar u spara l-kanuni tal-ġenb waqt li l-pilota tiegħu jtir
    .ai = Oqgħod fil-console tal-gunner ta' dan l-ajruplan: l-IA ddur f'orbita madwar il-punt li żżomm bil-pod tal-immirar
ops-planning = QED JIPPJANA: { $pilots }
ops-flying = QED JITIR: { $pilots }
ops-package = PACKAGE b'{ $kinds }: wettaq żewġ tipi hawn fi { $mins } min, +{ $bonus } punti kull wieħed
ops-sead-on-way = SEAD tal-AI fi triqu
ops-request-sead = ITLOB SEAD TAL-AI ({ $cost } materiel, { $aircraft } inġenji tal-ajru)
    .facts = Inġenji tal-ajru strike ħbieb jattakkaw sit SAM ta' dan il-post qabilek
    .disabled = Materiel jew inġenji tal-ajru fl-art mhux biżżejjed (ara FORZI)

## Regoli, materiel, forzi

ops-rule-damage = ħsara realistika
ops-rule-guidance = gwida realistika
ops-rule-no-crashes = l-ebda waqgħat
ops-rule-time = ħin kontinwu
ops-rule-weapons = armi realistiċi tal-F-117A
ops-rule-friendly-fire = nar alleat
ops-rule-airframe-losses = it-telf jiswa riżorsi
ops-rule-no-ai-missions = l-ebda missjoni tal-IA alleata
ops-rule-classic = klassiku
ops-materiel = MATERJAL
ops-materiel-blue = BLU { $materiel } (+{ $income }/h)
ops-materiel-red = AĦMAR { $materiel } (+{ $income }/h)
ops-crew-tempo = RITMU TAL-GĦADU ×{ $pace }
ops-due = T-{ $mins } min
ops-due-now = ISSA
ops-front-blue = IL-FRONT: IL-BLU JAVVANZA
ops-front-red = IL-FRONT: L-AĦMAR JAVVANZA
ops-front-even = IL-FRONT: STABBLI
ops-operation = Qbid ta' { $place }:
ops-friendly = ĦBIEB
ops-enemy = GĦADU
ops-section-forces = Forzi
ops-places-held = Postijiet miżmuma
ops-materiel-row = Materjal
ops-income = Dħul fis-siegħa
ops-income-from = minn
ops-income-sources = { $base } + { $places } postijiet x { $per_place } + { $cities } bliet x { $per_city }
ops-fighters-ground = Fighters fl-art
ops-helicopters-ground = Ħelikopters fl-art
ops-in-air = Fl-ajru
ops-in-air-value = { $fighters } fighters, { $strike } strike, { $helicopters } ħelikopters
ops-since-start = Mill-bidu
ops-earned = miksub
ops-repairs = tiswijiet
ops-operations = titjiriet, rejds, inserzjonijiet
ops-new-aircraft = inġenji tal-ajru ġodda
ops-lost = mitlufa (bliet, piloti)
ops-section-how = Kif taħdem
ops-how =
    Kull naħa tikseb materiel kull siegħa: ammont bażiku, aktar għal kull post li żżomm u għal kull belt sħiħa fosthom. Bombardja l-bliet tal-għadu (INTERDICTION) biex tnaqqas id-dħul tiegħu.
    L-AI thallas għal kull ma tagħmel: tiswijiet, kull titjira ta' fighter, rejds u l-inserzjonijiet tagħha. Jekk jonqosha l-materiel, il-ħsara tagħha tibqa' mhix imsewwija u l-inġenji tal-ajru tagħha jibqgħu d-dar.
    L-AI ittir biss l-inġenji tal-ajru li għandha; jekk jitwaqqgħu jintilfu, jekk jinżlu jerġgħu jtiru. Kull siegħa kull naħa tixtri oħrajn ġodda sakemm idum il-materiel tagħha.
    Int qatt ma tispiċċa mill-materiel: l-inġenji tal-ajru tiegħek, il-bażijiet tiegħek (bombardati jew le) u l-ħelikopters tal-ordnijiet ta' inserzjoni tiegħek huma dejjem hemm.
ops-how-repairs = Kull naħa ssewwi biss ftit pożizzjonijiet f'daqqa (aktar b'aktar bliet sħaħ), l-eqreb tal-front l-ewwel, u pożizzjoni meqruda mill-ġdid ftit wara t-tiswija tieħu aktar żmien kull darba.
ops-how-airframes = It-telf jiswa riżorsi: kull ajruplan li jitlef pilota huwa ġellied jew ħelikopter wieħed inqas għall-IA alleata.
ops-section-prices = Prezzijiet
ops-price-airfield = Tiswija ta' ajrudrom
ops-price-sam = Tiswija ta' sit SAM
ops-price-city = Belt mibnija mill-ġdid
ops-price-ship = Tiswija ta' vapur
ops-price-gun = Kanun jew tim MANPADS
ops-price-sortie = Titjira ta' fighter
ops-price-raid = Attakk
ops-price-insertion = Inserzjoni tal-AI
ops-price-fighter = Fighter ġdid
ops-price-helicopter = Ħelikopter ġdid
ops-price-city-lost = Belt meqruda (mitlufa)
ops-price-captured = Pilota maqbud (mitlufa)

## Mappa tal-gwerra

ops-click-to-watch = { $unit } - ikklikkja biex tarah
ops-section-legend = Leġenda
ops-legend-friendly-airfield = Ajrudrom ħabib
ops-legend-enemy-airfield = Ajrudrom tal-għadu
ops-legend-city = Belt ħabiba / tal-għadu
ops-legend-airfield = Ajruport ħabib / tal-għadu
ops-legend-insertion = Rotta tal-inserzjoni
ops-legend-missions = Missjonijiet fuq il-bord
ops-legend-defence-reach = Siti tad-difiża mill-ajru u l-firxa
ops-watch-hint = Ikklikkja unità biex tarha live
ops-map-hint = Rota: zoom, kaxkar: għaddi
ops-patriot = Batterija Patriot
ops-nasams = Batterija NASAMS
ops-manned-by = Mmexxija minn { $pilot }
ops-manned-by-other = Mmexxija minn plejer ieħor
ops-control = IKKONTROLLA
    .facts = Ħu f'idejk il-console tagħha

## Debriefing, tmiem, ġurnal, chat

ops-no-sortie = Għadha ma saret l-ebda titjira
ops-section-sortie = Titjira
ops-achieved = GĦAN MILĦUQ
ops-not-achieved = GĦAN MHUX MILĦUQ
ops-waiting-raid = QED JISTENNA R-RAID
ops-waiting-insertion = QED JISTENNA L-INSERZJONI
ops-flight-time = Ħin tat-titjira { $mins } min
ops-kills = Qtil: { $air } inġenji tal-ajru, { $ground } fl-art
ops-crew = Ekwipaġġ: { $names }
ops-points = { $points } punti
ops-points-bonus = { $points } punti (bonus tal-package +{ $bonus })
ops-campaign-points = { $rank } - { $points } punti tal-kampanja
ops-campaign-record = Kampanja: { $sorties } titjiriet, { $won } għanijiet milħuqa
ops-section-meanwhile = Sadanittant
ops-section-effects = L-effett tiegħek fuq il-gwerra
ops-victory = REBĦA
    .text = L-għadu m'għadu jżomm l-ebda ajrudrom: it-teatru huwa tagħna.
ops-defeat = TELFA
    .text = M'għadu fadal l-ebda ajrudrom ħabib: it-teatru intilef.
ops-end-day = Jum { $day }. Postijiet ħbieb { $friendly }, postijiet tal-għadu { $enemy }.
ops-next-campaign = IL-KAMPANJA LI JMISS TIBDA FI { $time }
ops-next-campaign-now = IL-KAMPANJA LI JMISS QED TIBDA
ops-next-from-menu = Ibda l-kampanja li jmiss mill-menu prinċipali.
ops-section-log = Ġurnal tal-kampanja
ops-section-chat = Chat
ops-no-messages = Għad m'hemm l-ebda messaġġ
ops-message = Messaġġ
    .hint = Ikteb messaġġ, Enter jibgħat
ops-send = IBGĦAT

## Briefing

ops-briefing = BRIEFING TAL-MISSJONI
ops-briefing-line = Minn { $base } - { $km } km lejn il-mira - { $aircraft } - armamenti: { $stores }
brief-choose-aircraft = AGĦŻEL L-AJRUPLAN
brief-aircraft-for = Ajruplani f'{ $base } għal { $kind }
brief-next = LI JMISS: { $step }
brief-from = { $aircraft } minn { $base }, { $km } km sal-mira
ops-route-locked = Rotta msakkra: { $pilots } qed jtiruha
ops-route-threats = ROTTA U THEDDIDIET
ops-take-off = QUM
ops-radars-shown = Radars murija: skoperti minn inġenji tal-ajru ħbieb, għadhom wieqfa

## Plejers

ops-section-players = Plejers
ops-col-name = ISEM
ops-col-status = STATUS
ops-col-sorties = TITJIRIET
ops-col-points = PUNTI
ops-col-pilot = PILOTA
ops-col-objectives = GĦANIJIET
ops-col-kills = QTIL AIR/GND
ops-col-medal = L-AQWA MEDALJA
ops-you = (int)
ops-host = OSPITANT
ops-status-planning = Qed jippjana { $title }
ops-status-board = Fuq il-bord tal-missjonijiet
ops-status-flying = Qed jitir { $title } - { $aircraft }
ops-status-reconnecting = Konnessjoni mitlufa - inġenju miżmum { $seconds } s
ops-status-gunner = Gunner ma' { $pilot }: { $title } - { $aircraft }
ops-flown = { $flown } ({ $won } milħuqa)
ops-section-leaderboard = Klassifika
ops-alone = Qed itir waħdek. Biex ittiru flimkien, ospita din il-kampanja minn MULTIPLAYER fil-menu prinċipali.
ops-rules = Regoli ta' din il-kampanja (miżmuma mill-bidu): { $rules }
ops-no-sorties = Għad m'hemm l-ebda titjira
ops-ai-load = Ħelikopter tal-AI jieħu tagħbija fi żmien { $time } jekk pilota tal-UH-60 ma jagħmilx hekk
ops-ai-takes-off = Tluq f'{ $time }
ops-ai-aircraft = IA { $aircraft }
ops-cargo-in = Tagħbija { $delivered } minn { $loads } wasslet - il-heliport jistenna { $time } siegħa oħra
ops-rule-classic-terrain = art klassika
ops-rule-classic-ai-flight = titjira klassika tal-IA
ops-layers-hint = QALIEB (fuq il-lemin): dak li turi l-mappa
ops-map-tab = MAPPA
ops-on-radio = FIR-RADJU
ops-mute = SKOTTA
    .facts = Isma' (jew le) dan il-plejer fir-radju
ops-unmute = REĠA' ISMA'

## Flights

ops-section-flights = Titjiriet ħbiebi
ops-no-flights = L-ebda ajruplan ħbiberi fl-ajru jew qed jistenna biex jitlaq
ops-flight-pilot = Pilota:
ops-flight-gunner = Gunner:
ops-seat-ai = IA
ops-seat-free = liberu
ops-flight-airborne = Fl-ajru
ops-flight-on-ground = Fl-art
ops-flight-starting = Qed jixgħel
