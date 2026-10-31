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
ops-tab-air-defence = DIFIŻA TAL-AJRU
ops-tab-forces = FORZI
ops-tab-players = PLEJERS
ops-tab-debrief = DEBRIEFING
ops-tab-log = ĠURNAL
ops-tab-chat = CHAT
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
ops-section-underway = Inserzjonijiet u rejds
ops-enemy-liftoff = GĦADU f'{ $place }: tluq fi { $mins } min
ops-liftoff = { $place }: tluq fi { $mins } min
ops-enemy-helis = GĦADU f'{ $place }: { $count } ħelikopters ġejjin
ops-helis = { $place }: { $count } ħelikopters fi triqthom
ops-raid-sead = SEAD tal-AI f'{ $place }
ops-raid-drones = DRONES f'{ $place }
ops-raid = RAID fuq l-ajrudrom ta' { $place }
ops-raid-launch = { $raid }: tnedija fi { $mins } min
ops-raid-on-way = { $raid }: { $count } fi triqhom
ops-resupply-ai = RIFORNIMENT għal { $unit }: UH-60 fi triqu
ops-awacs = AWACS TAL-GĦADU fl-ajru għal { $mins } min oħra
ops-down = MWAQQA': { $pilot } - salvataġġ fi { $mins } min
ops-aboard = A BORD: { $pilot } - fi triqu lejn id-dar
ops-none-underway = L-ebda waħda għaddejja
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
ops-rule-friendly-fire = friendly fire
ops-rule-classic = klassiku
ops-materiel = MATERIEL
ops-materiel-blue = BLU { $materiel } (+{ $income }/h)
ops-materiel-red = AĦMAR { $materiel } (+{ $income }/h)
ops-friendly = ĦBIEB
ops-enemy = GĦADU
ops-section-forces = Forzi
ops-places-held = Postijiet miżmuma
ops-materiel-row = Materiel
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
ops-section-prices = Prezzijiet
ops-price-airfield = Tiswija ta' ajrudrom
ops-price-sam = Tiswija ta' sit SAM
ops-price-city = Belt mibnija mill-ġdid
ops-price-ship = Tiswija ta' vapur
ops-price-gun = Kanun jew tim MANPADS
ops-price-sortie = Titjira ta' fighter
ops-price-raid = Raid
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
ops-watch-hint = Ikklikkja unità biex tarha live
ops-map-hint = Rota: zoom, kaxkar: għaddi
ops-patriot = Batterija Patriot
ops-drone-center = Ċentru tal-kmand tad-drones
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
ops-points = { $points } punti
ops-points-bonus = { $points } punti (bonus tal-package +{ $bonus })
ops-campaign-points = { $rank } - { $points } punti tal-kampanja
ops-campaign-record = Kampanja: { $sorties } titjiriet, { $won } għanijiet milħuqa
ops-section-meanwhile = Sadanittant
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
ops-host = HOST
ops-status-planning = Qed jippjana { $title }
ops-status-board = Fuq il-bord tal-missjonijiet
ops-status-flying = Qed jitir { $title } - { $aircraft }
ops-status-reconnecting = Konnessjoni mitlufa - inġenju miżmum { $seconds } s
ops-flown = { $flown } ({ $won } milħuqa)
ops-section-leaderboard = Klassifika
ops-alone = Qed itir waħdek. Biex ittiru flimkien, ospita din il-kampanja minn MULTIPLAYER fil-menu prinċipali.
ops-rules = Regoli ta' din il-kampanja (miżmuma mill-bidu): { $rules }
ops-no-sorties = Għad m'hemm l-ebda titjira
