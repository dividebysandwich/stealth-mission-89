# Zaslon Ops kampanje (src/campaign/ops.rs).

ops-thunderstorms = GRMLJAVINSKE OLUJE
ops-wx-short = VRIJEME { $now }, { $next } U { $at }
ops-wx-line = { $weather } Promjena u: { $next }, u { $at }.
ops-clock = DAN { $day }  { $time }
ops-landed-at = SLETIO U BAZU { $base }
ops-ground-crew = Zemaljska posada je spremna
ops-rearm = NAORUŽAJ I NATOČI GORIVO
ops-end-sortie = ZAVRŠI BORBENI LET
ops-on-server = NA { $server }
ops-serving = POSLUŽUJE
ops-tab-missions = MISIJE
ops-tab-map = RATNA KARTA
ops-tab-air-defence = PROTUZRAČNA OBRANA
ops-tab-forces = SNAGE
ops-tab-players = IGRAČI
ops-tab-debrief = DEBRIEFING
ops-tab-log = DNEVNIK
ops-tab-chat = CHAT
ops-blue = PLAVI { $count }
ops-red = CRVENI { $count }
ops-war-over = Rat je gotov
ops-running = Simulacija rata je u tijeku
ops-save-exit = SPREMI I IZAĐI
ops-disconnect = PREKINI VEZU
ops-leave-local = SPREMITI I NAPUSTITI KAMPANJU?
    .text = Rat staje dok ga ne nastavite. Zrakoplovi u zraku su izgubljeni; desanti u tijeku počinju ponovno iz svoje baze.
ops-leave-host = SPREMITI I PRESTATI POSLUŽIVATI KAMPANJU?
    .text = Igrači koji su se pridružili bit će odspojeni i rat staje dok ga ne nastavite. Zrakoplovi u zraku su izgubljeni; desanti u tijeku počinju ponovno iz svoje baze.
ops-leave-client = PREKINUTI VEZU S POSLUŽITELJEM?
    .text = Kampanja se nastavlja na poslužitelju; ovdje se ništa ne sprema. Vaš borbeni let završava.
ops-unit-destroyed = Jedinica uništena ({ $unit }): povratak u operacije

## Misije

ops-section-missions = Misije
ops-threat = PRIJETNJA { $threat }
ops-no-missions = Nema misija: fronta je mirna
ops-section-underway = Desanti i napadi
ops-enemy-liftoff = NEPRIJATELJ ({ $place }): polijetanje za { $mins } min
ops-liftoff = { $place }: polijetanje za { $mins } min
ops-enemy-helis = NEPRIJATELJ ({ $place }): helikoptera na putu: { $count }
ops-helis = { $place }: helikoptera na putu: { $count }
ops-raid-sead = AI SEAD: { $place }
ops-raid-drones = DRONOVI: { $place }
ops-raid = NAPAD na aerodrom { $place }
ops-raid-launch = { $raid }: lansiranje za { $mins } min
ops-raid-on-way = { $raid }: na putu { $count }
ops-resupply-ai = OPSKRBA ({ $unit }): UH-60 je na putu
ops-awacs = NEPRIJATELJSKI AWACS u zraku još { $mins } min
ops-down = PAO: { $pilot } - spašavanje u roku od { $mins } min
ops-aboard = U HELIKOPTERU: { $pilot } - na putu kući
ops-none-underway = Nema ih u tijeku
ops-ai-helicopter = AI helikopter ga preuzima za { $time } osim ako ne preuzme pilot UH-60
ops-defences = Obrana: SAM položaja { $sams }, prijetnja { $threat }
ops-recon-intel = IZVIĐAČKI PODACI
ops-order-insertion = NAREDI DESANT
    .facts = Letite pratnju da ih pokrijete ili ih vodite u UH-60
ops-from = Iz baze
ops-planned-by-group = Planira skupina
ops-stores = { $aircraft } - naoružanje: { $stores }
ops-fly-uh60 = LETI UH-60
ops-accept = PRIHVATI MISIJU
ops-planning = PLANIRA: { $pilots }
ops-flying = LETI: { $pilots }
ops-package = PAKET s vrstama { $kinds }: postignite dvije vrste ovdje u roku od { $mins } min, +{ $bonus } bodova svaka
ops-sead-on-way = AI SEAD je na putu
ops-request-sead = ZATRAŽI AI SEAD (materijal: { $cost }, zrakoplova: { $aircraft })
    .facts = Saveznički jurišni zrakoplovi gađaju SAM položaj ovog mjesta prije vas
    .disabled = Nema dovoljno materijala ili zrakoplova na tlu (vidi SNAGE)

## Pravila, materijal, snage

ops-rule-damage = realistična oštećenja
ops-rule-guidance = realistično navođenje
ops-rule-no-crashes = bez padova
ops-rule-time = neprekinuto vrijeme
ops-rule-weapons = realistično naoružanje F-117A
ops-rule-friendly-fire = prijateljska paljba
ops-rule-classic = klasično
ops-materiel = MATERIJAL
ops-materiel-blue = PLAVI { $materiel } (+{ $income }/h)
ops-materiel-red = CRVENI { $materiel } (+{ $income }/h)
ops-friendly = SAVEZNIČKO
ops-enemy = NEPRIJATELJSKO
ops-section-forces = Snage
ops-places-held = Držana mjesta
ops-materiel-row = Materijal
ops-income = Prihod po satu
ops-income-from = iz
ops-income-sources = { $base } + { $places } mjesta x { $per_place } + { $cities } gradova x { $per_city }
ops-fighters-ground = Lovci na tlu
ops-helicopters-ground = Helikopteri na tlu
ops-in-air = U zraku
ops-in-air-value = lovaca { $fighters }, jurišnih { $strike }, helikoptera { $helicopters }
ops-since-start = Od početka
ops-earned = zarađeno
ops-repairs = popravci
ops-operations = borbeni letovi, napadi, desanti
ops-new-aircraft = novi zrakoplovi
ops-lost = izgubljeno (gradovi, piloti)
ops-section-how = Kako to radi
ops-how =
    Svaka strana zarađuje materijal svakog sata: osnovni iznos, više za svako držano mjesto i za svaki neoštećeni grad među njima. Bombardirajte neprijateljske gradove (INTERDIKCIJA) da mu smanjite prihod.
    AI plaća sve što radi: popravke, svaki borbeni let lovaca, napade i vlastite desante. Bez materijala njegova oštećenja ostaju nepopravljena, a zrakoplovi ostaju kod kuće.
    AI leti samo zrakoplovima koje ima; oboreni su izgubljeni, sletjeli opet lete. Svakog sata svaka strana kupuje nove dok joj materijala ima.
    Vama nikad ne ponestaje: vaši zrakoplovi, vaše baze (bombardirane ili ne) i helikopteri vaših naredbi za desant uvijek su tu.
ops-section-prices = Cijene
ops-price-airfield = Popravak aerodroma
ops-price-sam = Popravak SAM položaja
ops-price-city = Obnova grada
ops-price-ship = Popravak broda
ops-price-gun = Top ili MANPADS tim
ops-price-sortie = Borbeni let lovaca
ops-price-raid = Napad
ops-price-insertion = AI desant
ops-price-fighter = Novi lovac
ops-price-helicopter = Novi helikopter
ops-price-city-lost = Grad uništen (izgubljeno)
ops-price-captured = Pilot zarobljen (izgubljeno)

## Ratna karta

ops-click-to-watch = { $unit } - kliknite za praćenje
ops-section-legend = Legenda
ops-legend-friendly-airfield = Saveznički aerodrom
ops-legend-enemy-airfield = Neprijateljski aerodrom
ops-legend-city = Saveznički / neprijateljski grad
ops-watch-hint = Kliknite jedinicu da je pratite uživo
ops-map-hint = Kotačić: zumiranje, povlačenje: pomicanje
ops-patriot = Patriot baterija
ops-manned-by = Posada: { $pilot }
ops-manned-by-other = Posada: drugi igrač
ops-control = UPRAVLJAJ
    .facts = Preuzmi njezinu konzolu

## Debriefing, kraj, dnevnik, chat

ops-no-sortie = Borbeni let još nije odletan
ops-section-sortie = Borbeni let
ops-achieved = CILJ POSTIGNUT
ops-not-achieved = CILJ NIJE POSTIGNUT
ops-waiting-raid = ČEKA SE NAPAD
ops-waiting-insertion = ČEKA SE DESANT
ops-flight-time = Vrijeme leta { $mins } min
ops-kills = Uništeno: zrakoplova { $air }, na tlu { $ground }
ops-points = Bodova: { $points }
ops-points-bonus = Bodova: { $points } (bonus paketa +{ $bonus })
ops-campaign-points = { $rank } - bodova kampanje: { $points }
ops-campaign-record = Kampanja: borbenih letova { $sorties }, postignutih ciljeva { $won }
ops-section-meanwhile = U međuvremenu
ops-victory = POBJEDA
    .text = Neprijatelj više ne drži nijedan aerodrom: ratište je naše.
ops-defeat = PORAZ
    .text = Nijedan saveznički aerodrom nije ostao: ratište je izgubljeno.
ops-end-day = Dan { $day }. Savezničkih mjesta { $friendly }, neprijateljskih mjesta { $enemy }.
ops-next-campaign = SLJEDEĆA KAMPANJA POČINJE ZA { $time }
ops-next-campaign-now = SLJEDEĆA KAMPANJA POČINJE
ops-next-from-menu = Pokrenite sljedeću kampanju iz glavnog izbornika.
ops-section-log = Dnevnik kampanje
ops-section-chat = Chat
ops-no-messages = Još nema poruka
ops-message = Poruka
    .hint = Upišite poruku, Enter šalje
ops-send = POŠALJI

## Briefing

ops-briefing = BRIEFING MISIJE
ops-briefing-line = Iz baze { $base } - { $km } km do cilja - { $aircraft } - naoružanje: { $stores }
ops-route-locked = Ruta zaključana: leti { $pilots }
ops-route-threats = RUTA I PRIJETNJE
ops-take-off = POLIJETANJE
ops-radars-shown = Prikazani radari: otkriveni savezničkim zrakoplovima, još uvijek aktivni

## Igrači

ops-section-players = Igrači
ops-col-name = IME
ops-col-status = STATUS
ops-col-sorties = BORBENI LETOVI
ops-col-points = BODOVI
ops-col-pilot = PILOT
ops-col-objectives = CILJEVI
ops-col-kills = ZRAK/TLO
ops-col-medal = NAJBOLJA MEDALJA
ops-you = (vi)
ops-host = DOMAĆIN
ops-status-planning = Planira: { $title }
ops-status-board = Na ploči misija
ops-status-flying = Leti: { $title } - { $aircraft }
ops-status-reconnecting = Veza izgubljena - zrakoplov se drži još { $seconds } s
ops-flown = { $flown } (postignuto: { $won })
ops-section-leaderboard = Ljestvica
ops-alone = Letite sami. Za zajednički let pokrenite ovu kampanju iz VIŠE IGRAČA u glavnom izborniku.
ops-rules = Pravila ove kampanje (zadržana od početka): { $rules }
ops-no-sorties = Još nema odletenih borbenih letova
