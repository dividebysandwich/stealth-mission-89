# Obrazovka operací kampaně (src/campaign/ops.rs).

ops-thunderstorms = BOUŘKY
ops-wx-short = WX { $now }, { $next } V { $at }
ops-wx-line = { $weather } Změna na { $next } v { $at }.
ops-clock = DEN { $day }  { $time }
ops-landed-at = PŘISTÁNO: { $base }
ops-ground-crew = Pozemní personál je připraven
ops-rearm = PŘEZBROJIT A DOTANKOVAT
ops-end-sortie = UKONČIT BOJOVÝ LET
ops-on-server = NA { $server }
ops-serving = HOSTUJE
ops-tab-missions = MISE
ops-tab-map = VÁLEČNÁ MAPA
ops-tab-air-defence = PROTIVZDUŠNÁ OBRANA
ops-tab-forces = SÍLY
ops-tab-players = HRÁČI
ops-tab-debrief = DEBRIEFING
ops-tab-log = DENÍK
ops-tab-chat = CHAT
ops-blue = MODŘÍ { $count }
ops-red = ČERVENÍ { $count }
ops-war-over = Válka skončila
ops-running = Válečná simulace běží
ops-save-exit = ULOŽIT A ODEJÍT
ops-disconnect = ODPOJIT
ops-leave-local = ULOŽIT A OPUSTIT KAMPAŇ?
    .text = Válka se zastaví, dokud v ní nepokračujete. Letadla ve vzduchu jsou ztracena; probíhající výsadky začnou znovu ze své základny.
ops-leave-host = ULOŽIT A PŘESTAT HOSTOVAT KAMPAŇ?
    .text = Připojení hráči budou odpojeni a válka se zastaví, dokud v ní nepokračujete. Letadla ve vzduchu jsou ztracena; probíhající výsadky začnou znovu ze své základny.
ops-leave-client = ODPOJIT SE OD SERVERU?
    .text = Kampaň běží dál na serveru; zde se nic neukládá. Váš bojový let končí.
ops-unit-destroyed = { $unit } zničen: zpět k operacím

## Mise

ops-section-missions = Mise
ops-threat = HROZBA { $threat }
ops-no-missions = Žádné mise: fronta je klidná
ops-section-underway = Výsadky a nálety
ops-enemy-liftoff = NEPŘÍTEL, { $place }: vzlet za { $mins } min
ops-liftoff = { $place }: vzlet za { $mins } min
ops-enemy-helis = NEPŘÍTEL, { $place }: vrtulníků na cestě: { $count }
ops-helis = { $place }: vrtulníků na cestě: { $count }
ops-raid-sead = SEAD řízený AI, { $place }
ops-raid-drones = DRONY, { $place }
ops-raid = NÁLET na letiště { $place }
ops-raid-launch = { $raid }: start za { $mins } min
ops-raid-on-way = { $raid }: na cestě { $count }
ops-resupply-ai = ZÁSOBOVÁNÍ: { $unit }, UH-60 na cestě
ops-awacs = NEPŘÁTELSKÝ AWACS ve vzduchu ještě { $mins } min
ops-down = SESTŘELEN: { $pilot } - záchrana do { $mins } min
ops-aboard = NA PALUBĚ: { $pilot } - na cestě domů
ops-none-underway = Nic neprobíhá
ops-ai-helicopter = Vrtulník řízený AI to provede za { $time }, ledaže to udělá pilot UH-60
ops-defences = Obrana: stanovišť SAM { $sams }, hrozba { $threat }
ops-recon-intel = PRŮZKUMNÉ INFORMACE
ops-order-insertion = NAŘÍDIT VÝSADEK
    .facts = Veďte eskortu, která je kryje, nebo je veďte na UH-60
ops-from = Z
ops-planned-by-group = Plánuje skupina
ops-stores = { $aircraft } - výzbroj: { $stores }
ops-fly-uh60 = LÉTAT NA UH-60
ops-accept = PŘIJMOUT MISI
ops-planning = PLÁNUJE: { $pilots }
ops-flying = LETÍ: { $pilots }
ops-package = PACKAGE s { $kinds }: splňte zde dva druhy do { $mins } min, +{ $bonus } bodů za každý
ops-sead-on-way = SEAD řízený AI je na cestě
ops-request-sead = VYŽÁDAT SEAD ŘÍZENÝ AI ({ $cost } materiálu, letadel: { $aircraft })
    .facts = Přátelské úderné letouny zaútočí před vámi na stanoviště SAM tohoto místa
    .disabled = Nedostatek materiálu nebo letadel na zemi (viz SÍLY)

## Pravidla, materiál, síly

ops-rule-damage = realistické poškození
ops-rule-guidance = realistické navádění
ops-rule-no-crashes = bez havárií
ops-rule-time = nepřetržitý čas
ops-rule-weapons = realistická výzbroj F-117A
ops-rule-friendly-fire = palba do vlastních
ops-rule-classic = klasické
ops-materiel = MATERIÁL
ops-materiel-blue = MODŘÍ { $materiel } (+{ $income }/h)
ops-materiel-red = ČERVENÍ { $materiel } (+{ $income }/h)
ops-friendly = PŘÁTELÉ
ops-enemy = NEPŘÍTEL
ops-section-forces = Síly
ops-places-held = Držená místa
ops-materiel-row = Materiál
ops-income = Příjem za hodinu
ops-income-from = z
ops-income-sources = { $base } + { $places } míst x { $per_place } + { $cities } měst x { $per_city }
ops-fighters-ground = Stíhačky na zemi
ops-helicopters-ground = Vrtulníky na zemi
ops-in-air = Ve vzduchu
ops-in-air-value = stíhaček { $fighters }, úderných { $strike }, vrtulníků { $helicopters }
ops-since-start = Od začátku
ops-earned = získáno
ops-repairs = opravy
ops-operations = bojové lety, nálety, výsadky
ops-new-aircraft = nová letadla
ops-lost = ztraceno (města, piloti)
ops-section-how = Jak to funguje
ops-how =
    Každá strana získává každou hodinu materiál: základní částku, víc za každé držené místo a za každé neporušené město mezi nimi. Bombardujte nepřátelská města (IZOLACE), abyste snížili jeho příjem.
    AI platí za vše, co dělá: opravy, každý stíhací let, nálety i vlastní výsadky. Při nedostatku materiálu zůstanou jeho škody neopravené a letadla na zemi.
    AI létá jen na letadlech, která má; sestřelená jsou pryč, přistálá zase létají. Každou hodinu každá strana nakupuje nová, dokud vydrží materiál.
    Vám nikdy nic nechybí: vaše letadla, vaše základny (bombardované či ne) a vrtulníky vašich vlastních rozkazů k výsadku jsou vždy k dispozici.
ops-section-prices = Ceny
ops-price-airfield = Oprava letiště
ops-price-sam = Oprava stanoviště SAM
ops-price-city = Obnova města
ops-price-ship = Oprava lodi
ops-price-gun = Kanón nebo četa MANPADS
ops-price-sortie = Stíhací let
ops-price-raid = Nálet
ops-price-insertion = Výsadek řízený AI
ops-price-fighter = Nová stíhačka
ops-price-helicopter = Nový vrtulník
ops-price-city-lost = Město zničeno (ztráta)
ops-price-captured = Pilot zajat (ztráta)

## Válečná mapa

ops-click-to-watch = { $unit } - kliknutím sledujte
ops-section-legend = Legenda
ops-legend-friendly-airfield = Přátelské letiště
ops-legend-enemy-airfield = Nepřátelské letiště
ops-legend-city = Přátelské / nepřátelské město
ops-watch-hint = Kliknutím na jednotku ji sledujte živě
ops-map-hint = Kolečko: zoom, tažení: posun
ops-patriot = Baterie Patriot
ops-manned-by = Obsluhuje { $pilot }
ops-manned-by-other = Obsluhuje jiný hráč
ops-control = OVLÁDAT
    .facts = Převzít její konzoli

## Debriefing, konec, deník, chat

ops-no-sortie = Zatím žádný bojový let
ops-section-sortie = Bojový let
ops-achieved = CÍL SPLNĚN
ops-not-achieved = CÍL NESPLNĚN
ops-waiting-raid = ČEKÁ SE NA NÁLET
ops-waiting-insertion = ČEKÁ SE NA VÝSADEK
ops-flight-time = Doba letu { $mins } min
ops-kills = Sestřely: { $air } letadel, { $ground } na zemi
ops-points = Bodů: { $points }
ops-points-bonus = Bodů: { $points } (bonus za package +{ $bonus })
ops-campaign-points = { $rank } - bodů kampaně: { $points }
ops-campaign-record = Kampaň: bojových letů { $sorties }, splněných cílů { $won }
ops-section-meanwhile = Mezitím
ops-victory = VÍTĚZSTVÍ
    .text = Nepřítel už nedrží žádné letiště: operační prostor je náš.
ops-defeat = PORÁŽKA
    .text = Nezbylo žádné přátelské letiště: operační prostor je ztracen.
ops-end-day = Den { $day }. Přátelská místa { $friendly }, nepřátelská místa { $enemy }.
ops-next-campaign = DALŠÍ KAMPAŇ ZAČNE ZA { $time }
ops-next-campaign-now = DALŠÍ KAMPAŇ ZAČÍNÁ
ops-next-from-menu = Další kampaň spusťte z hlavní nabídky.
ops-section-log = Deník kampaně
ops-section-chat = Chat
ops-no-messages = Zatím žádné zprávy
ops-message = Zpráva
    .hint = Napište zprávu, Enter odešle
ops-send = ODESLAT

## Briefing

ops-briefing = BRIEFING MISE
ops-briefing-line = Z { $base } - { $km } km k cíli - { $aircraft } - výzbroj: { $stores }
ops-route-locked = Trasa zamčena: letí { $pilots }
ops-route-threats = TRASA A HROZBY
ops-take-off = VZLETNOUT
ops-radars-shown = Zobrazené radary: odhalené přátelskými letadly, dosud stojící

## Hráči

ops-section-players = Hráči
ops-col-name = JMÉNO
ops-col-status = STAV
ops-col-sorties = BOJ. LETY
ops-col-points = BODY
ops-col-pilot = PILOT
ops-col-objectives = CÍLE
ops-col-kills = SESTŘELY VZD./ZEM
ops-col-medal = NEJLEPŠÍ MEDAILE
ops-you = (vy)
ops-host = HOSTITEL
ops-status-planning = Plánuje: { $title }
ops-status-board = U nástěnky misí
ops-status-flying = Letí: { $title } - { $aircraft }
ops-status-reconnecting = Spojení ztraceno - letadlo drženo { $seconds } s
ops-flown = { $flown } (splněno: { $won })
ops-section-leaderboard = Žebříček
ops-alone = Létáte sami. Chcete-li létat společně, hostujte tuto kampaň z MULTIPLAYER v hlavní nabídce.
ops-rules = Pravidla této kampaně (zachovaná od jejího začátku): { $rules }
ops-no-sorties = Zatím žádné bojové lety
