# Obrazovka Ops kampane (src/campaign/ops.rs).

ops-thunderstorms = BÚRKY
ops-wx-short = POČASIE { $now }, { $next } O { $at }
ops-wx-line = { $weather } Zmena na { $next } o { $at }.
ops-clock = DEŇ { $day }  { $time }
ops-landed-at = PRISTÁLI STE NA { $base }
ops-ground-crew = Pozemný personál je pripravený
ops-rearm = PREZBROJIŤ A DOTANKOVAŤ
ops-end-sortie = UKONČIŤ BOJOVÝ LET
ops-on-server = NA SERVERI { $server }
ops-serving = HOSTITEĽ
ops-tab-missions = MISIE
ops-tab-map = VOJNOVÁ MAPA
ops-tab-air-defence = PROTIVZDUŠNÁ OBRANA
ops-tab-forces = SILY
ops-tab-players = HRÁČI
ops-tab-debrief = DEBRIEFING
ops-tab-log = DENNÍK
ops-tab-chat = CHAT
ops-blue = MODRÍ { $count }
ops-red = ČERVENÍ { $count }
ops-war-over = Vojna skončila
ops-running = Simulácia vojny beží
ops-save-exit = ULOŽIŤ A ODÍSŤ
ops-disconnect = ODPOJIŤ
ops-leave-local = ULOŽIŤ A OPUSTIŤ KAMPAŇ?
    .text = Vojna sa zastaví, kým v nej nepokračujete. Lietadlá vo vzduchu sú stratené; prebiehajúce vysadenia začnú znova zo svojej základne.
ops-leave-host = ULOŽIŤ A PRESTAŤ HOSTIŤ KAMPAŇ?
    .text = Pripojení hráči sa odpoja a vojna sa zastaví, kým v nej nepokračujete. Lietadlá vo vzduchu sú stratené; prebiehajúce vysadenia začnú znova zo svojej základne.
ops-leave-client = ODPOJIŤ SA OD SERVERA?
    .text = Kampaň pokračuje na serveri; tu sa nič neukladá. Váš bojový let sa skončí.
ops-unit-destroyed = { $unit } zničený: späť k operáciám

## Misie

ops-section-missions = Misie
ops-threat = HROZBA { $threat }
ops-no-missions = Žiadne misie: front je pokojný
ops-section-underway = Vysadenia a nálety
ops-enemy-liftoff = NEPRIATEĽ, { $place }: štart o { $mins } min
ops-liftoff = { $place }: štart o { $mins } min
ops-enemy-helis = NEPRIATEĽ, { $place }: vrtuľníky na ceste: { $count }
ops-helis = { $place }: vrtuľníky na ceste: { $count }
ops-raid-sead = SEAD s umelou inteligenciou: { $place }
ops-raid-drones = DRONY: { $place }
ops-raid = NÁLET na letisko { $place }
ops-raid-launch = { $raid }: štart o { $mins } min
ops-raid-on-way = { $raid }: na ceste { $count }
ops-resupply-ai = ZÁSOBOVANIE pre { $unit }: UH-60 na ceste
ops-awacs = NEPRIATEĽSKÝ AWACS vo vzduchu ešte { $mins } min
ops-down = ZOSTRELENÝ: { $pilot } - záchrana do { $mins } min
ops-aboard = NA PALUBE: { $pilot } - cesta domov
ops-none-underway = Nič neprebieha
ops-ai-helicopter = Vrtuľník ovládaný umelou inteligenciou ho prevezme o { $time }, ak to neurobí pilot UH-60
ops-defences = Obrana: stanovištia SAM { $sams }, hrozba { $threat }
ops-recon-intel = ÚDAJE Z PRIESKUMU
ops-order-insertion = OBJEDNAŤ VYSADENIE
    .facts = Zabezpečte eskortu, ktorá ich bude kryť, alebo ich veďte v UH-60
ops-from = Z
ops-planned-by-group = Naplánované skupinou
ops-stores = { $aircraft } - výzbroj: { $stores }
ops-fly-uh60 = LIETAŤ NA UH-60
ops-accept = PRIJAŤ MISIU
ops-planning = PLÁNUJE: { $pilots }
ops-flying = LIETA: { $pilots }
ops-package = PACKAGE s: { $kinds } - dosiahnite tu dva druhy do { $mins } min, +{ $bonus } bodov za každý
ops-sead-on-way = SEAD s umelou inteligenciou je na ceste
ops-request-sead = VYŽIADAŤ SEAD S UMELOU INTELIGENCIOU ({ $cost } materiálu, lietadiel: { $aircraft })
    .facts = Spojenecké útočné lietadlá zaútočia na stanovište SAM v tomto mieste pred vami
    .disabled = Nedostatok materiálu alebo lietadiel na zemi (pozri SILY)

## Pravidlá, materiál, sily

ops-rule-damage = realistické poškodenie
ops-rule-guidance = realistické navádzanie
ops-rule-no-crashes = bez havárií
ops-rule-time = nepretržitý čas
ops-rule-weapons = realistické zbrane F-117A
ops-rule-friendly-fire = paľba do vlastných
ops-rule-classic = klasické
ops-materiel = MATERIÁL
ops-materiel-blue = MODRÍ { $materiel } (+{ $income }/h)
ops-materiel-red = ČERVENÍ { $materiel } (+{ $income }/h)
ops-friendly = SPOJENECKÉ
ops-enemy = NEPRIATEĽSKÉ
ops-section-forces = Sily
ops-places-held = Držané miesta
ops-materiel-row = Materiál
ops-income = Príjem za hodinu
ops-income-from = z
ops-income-sources = { $base } + { $places } miest x { $per_place } + { $cities } miest x { $per_city }
ops-fighters-ground = Stíhačky na zemi
ops-helicopters-ground = Vrtuľníky na zemi
ops-in-air = Vo vzduchu
ops-in-air-value = stíhačky: { $fighters }, útočné: { $strike }, vrtuľníky: { $helicopters }
ops-since-start = Od začiatku
ops-earned = získané
ops-repairs = opravy
ops-operations = bojové lety, nálety, vysadenia
ops-new-aircraft = nové lietadlá
ops-lost = stratené (mestá, piloti)
ops-section-how = Ako to funguje
ops-how =
    Každá strana získava materiál každú hodinu: základnú sumu, viac za každé držané miesto a za každé neporušené mesto medzi nimi. Bombardujte nepriateľské mestá (INTERDICTION) a znížte jeho príjem.
    Umelá inteligencia platí za všetko, čo robí: opravy, každý bojový let stíhačiek, nálety a vlastné vysadenia. Bez materiálu zostanú jej škody neopravené a jej lietadlá zostanú doma.
    Umelá inteligencia lieta len s lietadlami, ktoré má; zostrelené sú preč, pristáté lietajú znova. Každú hodinu každá strana nakúpi nové, kým jej vydrží materiál.
    Vám nikdy nechýba: vaše lietadlá, vaše základne (bombardované či nie) a vrtuľníky vašich vlastných objednávok vysadenia sú vždy k dispozícii.
ops-section-prices = Ceny
ops-price-airfield = Oprava letiska
ops-price-sam = Oprava stanovišťa SAM
ops-price-city = Obnova mesta
ops-price-ship = Oprava lode
ops-price-gun = Delo alebo tím MANPADS
ops-price-sortie = Bojový let stíhačiek
ops-price-raid = Nálet
ops-price-insertion = Vysadenie umelou inteligenciou
ops-price-fighter = Nová stíhačka
ops-price-helicopter = Nový vrtuľník
ops-price-city-lost = Mesto zničené (strata)
ops-price-captured = Pilot zajatý (strata)

## Vojnová mapa

ops-click-to-watch = { $unit } - kliknutím sledujte
ops-section-legend = Legenda
ops-legend-friendly-airfield = Spojenecké letisko
ops-legend-enemy-airfield = Nepriateľské letisko
ops-legend-city = Spojenecké / nepriateľské mesto
ops-legend-airfield = Vlastné / nepriateľské letisko
ops-legend-insertion = Trasa výsadku
ops-watch-hint = Kliknutím na jednotku ju sledujte naživo
ops-map-hint = Koliesko: priblíženie, ťahanie: posun
ops-patriot = Batéria Patriot
ops-drone-center = Veliteľské stanovište dronov
ops-manned-by = Obsluhuje: { $pilot }
ops-manned-by-other = Obsluhuje iný hráč
ops-control = OVLÁDAŤ
    .facts = Prevziať jeho konzolu

## Debriefing, koniec, denník, chat

ops-no-sortie = Zatiaľ žiadny bojový let
ops-section-sortie = Bojový let
ops-achieved = CIEĽ SPLNENÝ
ops-not-achieved = CIEĽ NESPLNENÝ
ops-waiting-raid = ČAKÁ SA NA NÁLET
ops-waiting-insertion = ČAKÁ SA NA VYSADENIE
ops-flight-time = Čas letu: { $mins } min
ops-kills = Zostrely: { $air } lietadiel, { $ground } na zemi
ops-points = { $points } bodov
ops-points-bonus = { $points } bodov (bonus za package +{ $bonus })
ops-campaign-points = { $rank } - { $points } bodov kampane
ops-campaign-record = Kampaň: bojových letov { $sorties }, splnených cieľov { $won }
ops-section-meanwhile = Medzitým
ops-victory = VÍŤAZSTVO
    .text = Nepriateľ už nedrží žiadne letisko: operačný priestor je náš.
ops-defeat = PORAŽKA
    .text = Nezostalo žiadne spojenecké letisko: operačný priestor je stratený.
ops-end-day = Deň { $day }. Spojenecké miesta: { $friendly }, nepriateľské miesta: { $enemy }.
ops-next-campaign = ĎALŠIA KAMPAŇ ZAČNE ZA { $time }
ops-next-campaign-now = ĎALŠIA KAMPAŇ ZAČÍNA
ops-next-from-menu = Ďalšiu kampaň spustite z hlavného menu.
ops-section-log = Denník kampane
ops-section-chat = Chat
ops-no-messages = Zatiaľ žiadne správy
ops-message = Správa
    .hint = Napíšte správu, Enter ju odošle
ops-send = ODOSLAŤ

## Brífing

ops-briefing = BRÍFING MISIE
ops-briefing-line = Z { $base } - { $km } km k cieľu - { $aircraft } - výzbroj: { $stores }
ops-route-locked = Trasa uzamknutá: lieta po nej { $pilots }
ops-route-threats = TRASA A HROZBY
ops-take-off = ŠTARTOVAŤ
ops-radars-shown = Zobrazené radary: zistené spojeneckými lietadlami, ešte funkčné

## Hráči

ops-section-players = Hráči
ops-col-name = MENO
ops-col-status = STAV
ops-col-sorties = BOJOVÉ LETY
ops-col-points = BODY
ops-col-pilot = PILOT
ops-col-objectives = CIELE
ops-col-kills = ZOSTRELY VZD./ZEM
ops-col-medal = NAJLEPŠIA MEDAILA
ops-you = (vy)
ops-host = HOSTITEĽ
ops-status-planning = Plánuje: { $title }
ops-status-board = Pri tabuli misií
ops-status-flying = Lieta: { $title } - { $aircraft }
ops-status-reconnecting = Spojenie stratené - lietadlo držané { $seconds } s
ops-flown = { $flown } (splnených: { $won })
ops-section-leaderboard = Rebríček
ops-alone = Lietate sami. Ak chcete lietať spolu, hostujte túto kampaň cez MULTIPLAYER v hlavnom menu.
ops-rules = Pravidlá tejto kampane (platia od jej začiatku): { $rules }
ops-no-sorties = Zatiaľ žiadne bojové lety
