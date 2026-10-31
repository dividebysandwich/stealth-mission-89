# Kampanjan Ops-näyttö (src/campaign/ops.rs).

ops-thunderstorms = UKKOSMYRSKYJÄ
ops-wx-short = SÄÄ { $now }, { $next } KLO { $at }
ops-wx-line = { $weather } Muuttuu: { $next } klo { $at }.
ops-clock = PÄIVÄ { $day }  { $time }
ops-landed-at = LASKEUTUNUT: { $base }
ops-ground-crew = Maahenkilöstö on valmiina
ops-rearm = ASEISTUS JA TANKKAUS
ops-end-sortie = PÄÄTÄ LENTO
ops-on-server = PALVELIMELLA { $server }
ops-serving = PALVELEE
ops-tab-missions = TEHTÄVÄT
ops-tab-map = SOTAKARTTA
ops-tab-air-defence = ILMATORJUNTA
ops-tab-forces = VOIMAT
ops-tab-players = PELAAJAT
ops-tab-debrief = PURKU
ops-tab-log = LOKI
ops-tab-chat = CHAT
ops-blue = SININEN { $count }
ops-red = PUNAINEN { $count }
ops-war-over = Sota on ohi
ops-running = Sotasimulaatio käynnissä
ops-save-exit = TALLENNA JA POISTU
ops-disconnect = KATKAISE YHTEYS
ops-leave-local = TALLENNA JA POISTU KAMPANJASTA?
    .text = Sota pysähtyy, kunnes jatkat sitä. Ilmassa olevat koneet menetetään; käynnissä olevat maihinnousut alkavat alusta tukikohdasta.
ops-leave-host = TALLENNA JA LOPETA KAMPANJAN ISÄNNÖINTI?
    .text = Liittyneiden pelaajien yhteys katkeaa ja sota pysähtyy, kunnes jatkat sitä. Ilmassa olevat koneet menetetään; käynnissä olevat maihinnousut alkavat alusta tukikohdasta.
ops-leave-client = KATKAISE YHTEYS PALVELIMEEN?
    .text = Kampanja jatkuu palvelimella; täällä ei tallenneta mitään. Lentosi päättyy.
ops-unit-destroyed = Tuhottu: { $unit }. Takaisin operaatioihin

## Tehtävät

ops-section-missions = Tehtävät
ops-threat = UHKA { $threat }
ops-no-missions = Ei tehtäviä: rintama on hiljainen
ops-section-underway = Maihinnousut ja iskut
ops-enemy-liftoff = VIHOLLINEN, { $place }: lähtö { $mins } min kuluttua
ops-liftoff = { $place }: lähtö { $mins } min kuluttua
ops-enemy-helis = VIHOLLINEN, { $place }: helikoptereita lähestymässä, { $count }
ops-helis = { $place }: helikoptereita matkalla, { $count }
ops-raid-sead = Tekoälyn SEAD: { $place }
ops-raid-drones = DROONIT: { $place }
ops-raid = ISKU lentokentälle { $place }
ops-raid-launch = { $raid }: lähtö { $mins } min kuluttua
ops-raid-on-way = { $raid }: matkalla { $count }
ops-resupply-ai = TÄYDENNYS, yksikkö { $unit }: UH-60 matkalla
ops-awacs = VIHOLLISEN AWACS ilmassa vielä { $mins } min
ops-down = ALHAALLA: { $pilot } - pelastus { $mins } min sisällä
ops-aboard = KYYDISSÄ: { $pilot } - matkalla kotiin
ops-none-underway = Ei käynnissä olevia
ops-ai-helicopter = Tekoälyn helikopteri hoitaa sen ajassa { $time }, ellei UH-60:n lentäjä ehdi ensin
ops-defences = Puolustus: SAM-asemia { $sams }, uhka { $threat }
ops-recon-intel = TIEDUSTELUTIETO
ops-order-insertion = TILAA MAIHINNOUSU
    .facts = Lennä saattajana suojaamassa niitä tai johda ne perille UH-60:llä
ops-from = Lähtö
ops-planned-by-group = Ryhmän suunnittelema
ops-stores = { $aircraft } - aseistus: { $stores }
ops-fly-uh60 = LENNÄ UH-60:LLÄ
ops-accept = HYVÄKSY TEHTÄVÄ
ops-planning = SUUNNITTELEE: { $pilots }
ops-flying = LENTÄÄ: { $pilots }
ops-package = PAKETTI, mukana { $kinds }: saavuta täällä kaksi lajia { $mins } min sisällä, +{ $bonus } pistettä kustakin
ops-sead-on-way = Tekoälyn SEAD on matkalla
ops-request-sead = PYYDÄ TEKOÄLYN SEAD ({ $cost } materiaalia, { $aircraft } konetta)
    .facts = Omat iskukoneet hyökkäävät tämän paikan SAM-asemaan ennen sinua
    .disabled = Materiaali tai koneet maassa eivät riitä (katso VOIMAT)

## Säännöt, materiaali, voimat

ops-rule-damage = realistiset vauriot
ops-rule-guidance = realistinen ohjaus
ops-rule-no-crashes = ei syöksyjä
ops-rule-time = jatkuva aika
ops-rule-weapons = realistiset F-117A-aseet
ops-rule-friendly-fire = omien tulitus
ops-rule-classic = klassinen
ops-materiel = MATERIAALI
ops-materiel-blue = SININEN { $materiel } (+{ $income }/h)
ops-materiel-red = PUNAINEN { $materiel } (+{ $income }/h)
ops-friendly = OMAT
ops-enemy = VIHOLLINEN
ops-section-forces = Voimat
ops-places-held = Hallitut paikat
ops-materiel-row = Materiaali
ops-income = Tulo tunnissa
ops-income-from = lähde
ops-income-sources = { $base } + paikkoja { $places } x { $per_place } + kaupunkeja { $cities } x { $per_city }
ops-fighters-ground = Hävittäjät maassa
ops-helicopters-ground = Helikopterit maassa
ops-in-air = Ilmassa
ops-in-air-value = hävittäjiä { $fighters }, iskukoneita { $strike }, helikoptereita { $helicopters }
ops-since-start = Alusta lähtien
ops-earned = ansaittu
ops-repairs = korjaukset
ops-operations = lennot, iskut, maihinnousut
ops-new-aircraft = uudet koneet
ops-lost = menetetty (kaupungit, lentäjät)
ops-section-how = Näin se toimii
ops-how =
    Kumpikin osapuoli ansaitsee materiaalia joka tunti: perusmäärän sekä lisää jokaisesta hallitsemastaan paikasta ja jokaisesta ehjästä kaupungista. Pommita vihollisen kaupunkeja (ERISTYS) leikataksesi sen tuloja.
    Tekoäly maksaa kaikesta, mitä se tekee: korjauksista, jokaisesta hävittäjälennosta, iskuista ja omista maihinnousuistaan. Jos materiaali loppuu, sen vauriot jäävät korjaamatta ja sen koneet pysyvät kotona.
    Tekoäly lentää vain koneilla, jotka sillä on; alasammutut ovat poissa, laskeutuneet lentävät taas. Joka tunti kumpikin osapuoli ostaa uusia, kunhan materiaalia riittää.
    Sinulta ei lopu mikään: koneesi, tukikohtasi (pommitettuina tai ei) ja omien maihinnousutilaustesi helikopterit ovat aina saatavilla.
ops-section-prices = Hinnat
ops-price-airfield = Lentokentän korjaus
ops-price-sam = SAM-aseman korjaus
ops-price-city = Kaupungin jälleenrakennus
ops-price-ship = Laivan korjaus
ops-price-gun = Tykki tai MANPADS-ryhmä
ops-price-sortie = Hävittäjälento
ops-price-raid = Isku
ops-price-insertion = Tekoälyn maihinnousu
ops-price-fighter = Uusi hävittäjä
ops-price-helicopter = Uusi helikopteri
ops-price-city-lost = Kaupunki tuhottu (menetetty)
ops-price-captured = Lentäjä vangittu (menetetty)

## Sotakartta

ops-click-to-watch = { $unit } - katso napsauttamalla
ops-section-legend = Selite
ops-legend-friendly-airfield = Oma lentokenttä
ops-legend-enemy-airfield = Vihollisen lentokenttä
ops-legend-city = Oma / vihollisen kaupunki
ops-legend-airfield = Oma / vihollisen lentokenttä
ops-legend-insertion = Maahanlaskun reitti
ops-watch-hint = Napsauta yksikköä seurataksesi sitä livenä
ops-map-hint = Pyörä: zoom, vedä: panoroi
ops-patriot = Patriot-patteri
ops-drone-center = Lennokkien johtokeskus
ops-manned-by = Miehittäjä: { $pilot }
ops-manned-by-other = Toinen pelaaja miehittää
ops-control = OHJAA
    .facts = Ota sen konsoli haltuun

## Purku, loppu, loki, chat

ops-no-sortie = Ei vielä lentoja
ops-section-sortie = Lento
ops-achieved = TAVOITE SAAVUTETTU
ops-not-achieved = TAVOITETTA EI SAAVUTETTU
ops-waiting-raid = ODOTTAA ISKUA
ops-waiting-insertion = ODOTTAA MAIHINNOUSUA
ops-flight-time = Lentoaika { $mins } min
ops-kills = Tuhotut: koneita { $air }, maassa { $ground }
ops-points = { $points } pistettä
ops-points-bonus = { $points } pistettä (pakettibonus +{ $bonus })
ops-campaign-points = { $rank } - kampanjapisteet { $points }
ops-campaign-record = Kampanja: lentoja { $sorties }, tavoitteita saavutettu { $won }
ops-section-meanwhile = Sillä välin
ops-victory = VOITTO
    .text = Vihollisella ei ole enää yhtään lentokenttää: operaatioalue on meidän.
ops-defeat = TAPPIO
    .text = Omia lentokenttiä ei ole jäljellä: operaatioalue on menetetty.
ops-end-day = Päivä { $day }. Omat paikat { $friendly }, vihollisen paikat { $enemy }.
ops-next-campaign = SEURAAVA KAMPANJA ALKAA: { $time }
ops-next-campaign-now = SEURAAVA KAMPANJA ALKAA
ops-next-from-menu = Aloita seuraava kampanja päävalikosta.
ops-section-log = Kampanjaloki
ops-section-chat = Chat
ops-no-messages = Ei vielä viestejä
ops-message = Viesti
    .hint = Kirjoita viesti, Enter lähettää
ops-send = LÄHETÄ

## Tehtävänanto

ops-briefing = TEHTÄVÄNANTO
ops-briefing-line = Lähtö: { $base } - { $km } km kohteeseen - { $aircraft } - aseistus: { $stores }
ops-route-locked = Reitti lukittu: lentäjät { $pilots }
ops-route-threats = REITTI JA UHKAT
ops-take-off = LENTOON
ops-radars-shown = Näytetyt tutkat: omien koneiden havaitsemat, yhä toiminnassa

## Pelaajat

ops-section-players = Pelaajat
ops-col-name = NIMI
ops-col-status = TILA
ops-col-sorties = LENNOT
ops-col-points = PISTEET
ops-col-pilot = LENTÄJÄ
ops-col-objectives = TAVOITTEET
ops-col-kills = TUHOT ILMA/MAA
ops-col-medal = PARAS MITALI
ops-you = (sinä)
ops-host = ISÄNTÄ
ops-status-planning = Suunnittelee: { $title }
ops-status-board = Tehtävätaululla
ops-status-flying = Lentää: { $title } - { $aircraft }
ops-status-reconnecting = Yhteys katkennut - kone odottaa { $seconds } s
ops-flown = { $flown } (saavutettu { $won })
ops-section-leaderboard = Tulostaulu
ops-alone = Lennät yksin. Lentääksesi yhdessä isännöi tämä kampanja päävalikon MONINPELI-kohdasta.
ops-rules = Tämän kampanjan säännöt (säilyvät alusta asti): { $rules }
ops-no-sorties = Ei vielä lentoja
