# Ecranul Ops al campaniei (src/campaign/ops.rs).

ops-thunderstorms = FURTUNI
ops-wx-short = VREME { $now }, { $next } LA { $at }
ops-wx-line = { $weather } Se schimbă în { $next } la { $at }.
ops-clock = ZIUA { $day }  { $time }
ops-landed-at = ATERIZAT LA { $base }
ops-ground-crew = Echipa de sol este pregătită
ops-rearm = REÎNARMARE ȘI REALIMENTARE
ops-end-sortie = ÎNCHEIE ZBORUL
ops-on-server = PE { $server }
ops-serving = GĂZDUIEȘTE
ops-tab-missions = MISIUNI
ops-tab-map = HARTA DE RĂZBOI
ops-tab-air-defence = APĂRARE ANTIAERIANĂ
ops-tab-forces = FORȚE
ops-tab-players = JUCĂTORI
ops-tab-debrief = DEBRIEFING
ops-tab-log = JURNAL
ops-tab-chat = CHAT
ops-blue = ALBASTRU { $count }
ops-red = ROȘU { $count }
ops-war-over = Războiul s-a încheiat
ops-running = Simularea războiului rulează
ops-save-exit = SALVARE ȘI IEȘIRE
ops-disconnect = DECONECTARE
ops-leave-local = SALVAȚI ȘI PĂRĂSIȚI CAMPANIA?
    .text = Războiul se oprește până o continuați. Aeronavele aflate în aer sunt pierdute; debarcările în curs reîncep de la baza lor.
ops-leave-host = SALVAȚI ȘI OPRIȚI GĂZDUIREA CAMPANIEI?
    .text = Jucătorii alăturați sunt deconectați, iar războiul se oprește până îl continuați. Aeronavele aflate în aer sunt pierdute; debarcările în curs reîncep de la baza lor.
ops-leave-client = VĂ DECONECTAȚI DE LA SERVER?
    .text = Campania continuă pe server; nimic nu este salvat aici. Zborul dumneavoastră se încheie.
ops-unit-destroyed = { $unit } distrus: înapoi la operațiuni

## Misiuni

ops-section-missions = Misiuni
ops-threat = AMENINȚARE { $threat }
ops-no-missions = Nicio misiune: frontul este liniștit
ops-section-underway = Debarcări și raiduri
ops-enemy-liftoff = INAMIC la { $place }: decolare în { $mins } min
ops-liftoff = { $place }: decolare în { $mins } min
ops-enemy-helis = INAMIC la { $place }: { $count } elicoptere în apropiere
ops-helis = { $place }: { $count } elicoptere în drum
ops-raid-sead = SEAD AI la { $place }
ops-raid-drones = DRONE la { $place }
ops-raid = RAID asupra aerodromului { $place }
ops-raid-launch = { $raid }: lansare în { $mins } min
ops-raid-on-way = { $raid }: { $count } în drum
ops-resupply-ai = REAPROVIZIONARE pentru { $unit }: UH-60 în drum
ops-awacs = AWACS INAMIC în aer încă { $mins } min
ops-down = DOBORÂT: { $pilot } - salvare în { $mins } min
ops-aboard = LA BORD: { $pilot } - în drum spre casă
ops-none-underway = Nimic în desfășurare
ops-ai-helicopter = Un elicopter AI îl ia în { $time }, dacă nu o face un pilot UH-60
ops-defences = Apărare: { $sams } poziții SAM, amenințare { $threat }
ops-recon-intel = INFORMAȚII RECON
ops-order-insertion = ORDONAȚI DEBARCAREA
    .facts = Zburați escorta pentru a-i acoperi sau conduceți-i cu UH-60
ops-from = De la
ops-planned-by-group = Planificat de grup
ops-stores = { $aircraft } - armament: { $stores }
ops-fly-uh60 = ZBURAȚI CU UH-60
ops-accept = ACCEPTAȚI MISIUNEA
ops-planning = PLANIFICĂ: { $pilots }
ops-flying = ZBOARĂ: { $pilots }
ops-package = PACKAGE cu { $kinds }: îndepliniți două tipuri aici în { $mins } min, +{ $bonus } puncte fiecare
ops-sead-on-way = SEAD AI este în drum
ops-request-sead = SOLICITAȚI SEAD AI ({ $cost } materiale, { $aircraft } aeronave)
    .facts = Aeronavele de lovire prietene atacă o poziție SAM din acest loc înaintea dumneavoastră
    .disabled = Materiale sau aeronave la sol insuficiente (vezi FORȚE)

## Reguli, materiale, forțe

ops-rule-damage = avarii realiste
ops-rule-guidance = ghidare realistă
ops-rule-no-crashes = fără prăbușiri
ops-rule-time = timp continuu
ops-rule-weapons = armament realist F-117A
ops-rule-friendly-fire = foc prietenesc
ops-rule-classic = clasic
ops-materiel = MATERIALE
ops-materiel-blue = ALBASTRU { $materiel } (+{ $income }/h)
ops-materiel-red = ROȘU { $materiel } (+{ $income }/h)
ops-friendly = PRIETEN
ops-enemy = INAMIC
ops-section-forces = Forțe
ops-places-held = Locuri deținute
ops-materiel-row = Materiale
ops-income = Venit pe oră
ops-income-from = din
ops-income-sources = { $base } + { $places } locuri x { $per_place } + { $cities } orașe x { $per_city }
ops-fighters-ground = Avioane de vânătoare la sol
ops-helicopters-ground = Elicoptere la sol
ops-in-air = În aer
ops-in-air-value = { $fighters } avioane de vânătoare, { $strike } de lovire, { $helicopters } elicoptere
ops-since-start = De la început
ops-earned = câștigate
ops-repairs = reparații
ops-operations = zboruri, raiduri, debarcări
ops-new-aircraft = aeronave noi
ops-lost = pierdute (orașe, piloți)
ops-section-how = Cum funcționează
ops-how =
    Fiecare tabără câștigă materiale în fiecare oră: o sumă de bază, mai mult pentru fiecare loc deținut și pentru fiecare oraș intact dintre ele. Bombardați orașele inamicului (INTERDICTION) pentru a-i tăia veniturile.
    AI plătește tot ce face: reparații, fiecare zbor de vânătoare, raiduri și propriile debarcări. Fără materiale, avariile sale rămân nereparate, iar aeronavele rămân acasă.
    AI zboară doar cu aeronavele pe care le are; cele doborâte sunt pierdute, cele aterizate pot zbura din nou. În fiecare oră fiecare tabără cumpără altele noi cât timp are materiale.
    Dumneavoastră nu duceți niciodată lipsă: aeronavele, bazele (bombardate sau nu) și elicopterele propriilor ordine de debarcare sunt mereu disponibile.
ops-section-prices = Prețuri
ops-price-airfield = Reparația aerodromului
ops-price-sam = Reparația unei poziții SAM
ops-price-city = Reconstrucția unui oraș
ops-price-ship = Reparația unei nave
ops-price-gun = Echipă de tun sau MANPADS
ops-price-sortie = Zbor de vânătoare
ops-price-raid = Raid
ops-price-insertion = Debarcare AI
ops-price-fighter = Avion de vânătoare nou
ops-price-helicopter = Elicopter nou
ops-price-city-lost = Oraș distrus (pierdere)
ops-price-captured = Pilot capturat (pierdere)

## Harta de război

ops-click-to-watch = { $unit } - clic pentru a urmări
ops-section-legend = Legendă
ops-legend-friendly-airfield = Aerodrom prieten
ops-legend-enemy-airfield = Aerodrom inamic
ops-legend-city = Oraș prieten / inamic
ops-watch-hint = Clic pe o unitate pentru a o urmări live
ops-map-hint = Rotiță: zoom, tragere: deplasare
ops-patriot = Baterie Patriot
ops-manned-by = Deservit de { $pilot }
ops-manned-by-other = Deservit de alt jucător
ops-control = CONTROL
    .facts = Preia consola sa

## Debriefing, final, jurnal, chat

ops-no-sortie = Niciun zbor efectuat încă
ops-section-sortie = Zbor
ops-achieved = OBIECTIV ÎNDEPLINIT
ops-not-achieved = OBIECTIV NEÎNDEPLINIT
ops-waiting-raid = SE AȘTEAPTĂ RAIDUL
ops-waiting-insertion = SE AȘTEAPTĂ DEBARCAREA
ops-flight-time = Timp de zbor { $mins } min
ops-kills = Victorii: { $air } aeronave, { $ground } la sol
ops-points = { $points } puncte
ops-points-bonus = { $points } puncte (bonus de package +{ $bonus })
ops-campaign-points = { $rank } - { $points } puncte de campanie
ops-campaign-record = Campanie: { $sorties } zboruri, { $won } obiective îndeplinite
ops-section-meanwhile = Între timp
ops-victory = VICTORIE
    .text = Inamicul nu mai deține niciun aerodrom: teatrul de operații este al nostru.
ops-defeat = ÎNFRÂNGERE
    .text = Nu a mai rămas niciun aerodrom prieten: teatrul de operații este pierdut.
ops-end-day = Ziua { $day }. Locuri prietene { $friendly }, locuri inamice { $enemy }.
ops-next-campaign = URMĂTOAREA CAMPANIE ÎNCEPE ÎN { $time }
ops-next-campaign-now = URMĂTOAREA CAMPANIE ÎNCEPE
ops-next-from-menu = Porniți următoarea campanie din meniul principal.
ops-section-log = Jurnalul campaniei
ops-section-chat = Chat
ops-no-messages = Niciun mesaj încă
ops-message = Mesaj
    .hint = Scrieți un mesaj, Enter îl trimite
ops-send = TRIMITE

## Briefing

ops-briefing = BRIEFING DE MISIUNE
ops-briefing-line = De la { $base } - { $km } km până la țintă - { $aircraft } - armament: { $stores }
ops-route-locked = Rută blocată: { $pilots } o zboară
ops-route-threats = RUTĂ ȘI AMENINȚĂRI
ops-take-off = DECOLARE
ops-radars-shown = Radare afișate: detectate de aeronavele prietene, încă în funcțiune

## Jucători

ops-section-players = Jucători
ops-col-name = NUME
ops-col-status = STARE
ops-col-sorties = ZBORURI
ops-col-points = PUNCTE
ops-col-pilot = PILOT
ops-col-objectives = OBIECTIVE
ops-col-kills = VICTORII AER/SOL
ops-col-medal = CEA MAI BUNĂ MEDALIE
ops-you = (dumneavoastră)
ops-host = GAZDĂ
ops-status-planning = Planifică { $title }
ops-status-board = La tabla de misiuni
ops-status-flying = Zboară { $title } - { $aircraft }
ops-status-reconnecting = Conexiune pierdută - aeronavă păstrată { $seconds } s
ops-flown = { $flown } ({ $won } îndeplinite)
ops-section-leaderboard = Clasament
ops-alone = Zburați singur. Pentru a zbura împreună, găzduiți această campanie din MULTIPLAYER, în meniul principal.
ops-rules = Regulile acestei campanii (păstrate de la început): { $rules }
ops-no-sorties = Niciun zbor efectuat încă
