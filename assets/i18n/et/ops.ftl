# Kampaania Ops-ekraan (src/campaign/ops.rs).

ops-thunderstorms = ÄIKESETORMID
ops-wx-short = ILM { $now }, { $next } KELL { $at }
ops-wx-line = { $weather } Muutub: { $next } kell { $at }.
ops-clock = PÄEV { $day }  { $time }
ops-landed-at = MAANDUS: { $base }
ops-ground-crew = Maapersonal on valmis
ops-rearm = RELVASTA JA TANGI
ops-end-sortie = LÕPETA LEND
ops-on-server = SERVERIS { $server }
ops-serving = KORRALDAN
ops-tab-missions = MISSIOONID
ops-tab-map = SÕJAKAART
ops-tab-air-defence = ÕHUTÕRJE
ops-tab-forces = JÕUD
ops-tab-players = MÄNGIJAD
ops-tab-debrief = DEBRIIFING
ops-tab-log = LOGI
ops-tab-chat = VESTLUS
ops-blue = SININE { $count }
ops-red = PUNANE { $count }
ops-war-over = Sõda on läbi
ops-running = Sõjasimulatsioon töötab
ops-save-exit = SALVESTA JA VÄLJU
ops-disconnect = KATKESTA ÜHENDUS
ops-leave-local = SALVESTA JA LAHKU KAMPAANIAST?
    .text = Sõda peatub, kuni sa seda jätkad. Õhus olevad õhusõidukid lähevad kaduma; pooleliolevad dessandid algavad uuesti oma baasist.
ops-leave-host = SALVESTA JA LÕPETA KAMPAANIA KORRALDAMINE?
    .text = Liitunud mängijate ühendus katkeb ja sõda peatub, kuni sa seda jätkad. Õhus olevad õhusõidukid lähevad kaduma; pooleliolevad dessandid algavad uuesti oma baasist.
ops-leave-client = KATKESTA ÜHENDUS SERVERIGA?
    .text = Kampaania jätkub serveris; siia midagi ei salvestata. Sinu lend lõpeb.
ops-unit-destroyed = { $unit } hävitatud: tagasi operatsioonide juurde

## Missioonid

ops-section-missions = Missioonid
ops-threat = OHT { $threat }
ops-no-missions = Missioone pole: rindel on vaikne
ops-section-underway = Dessandid ja rünnakud
ops-enemy-liftoff = VAENLANE, { $place }: õhkutõus { $mins } min pärast
ops-liftoff = { $place }: õhkutõus { $mins } min pärast
ops-enemy-helis = VAENLANE, { $place }: { $count } helikopterit teel
ops-helis = { $place }: { $count } helikopterit teel
ops-raid-sead = AI SEAD: { $place }
ops-raid-drones = DROONID: { $place }
ops-raid = RÜNNAK: { $place } lennuväli
ops-raid-launch = { $raid }: start { $mins } min pärast
ops-raid-on-way = { $raid }: { $count } teel
ops-resupply-ai = TÄIENDUS üksusele { $unit }: UH-60 teel
ops-awacs = VAENLASE AWACS ÕHUS veel { $mins } min
ops-down = MAHA LASKUNUD: { $pilot } - päästmine { $mins } min jooksul
ops-aboard = pardal: { $pilot } - teel koju
ops-none-underway = Pooleli pole midagi
ops-ai-helicopter = AI helikopter teeb selle { $time } pärast, kui seda ei tee UH-60 piloot
ops-defences = Kaitse: { $sams } SAM-positsiooni, oht { $threat }
ops-recon-intel = LUUREANDMED
ops-order-insertion = TELLI DESSANT
    .facts = Lenda eskortina nende katteks või juhi nad UH-60-ga kohale
ops-from = Algus
ops-planned-by-group = Planeerinud rühm
ops-stores = { $aircraft } - relvastus: { $stores }
ops-fly-uh60 = LENDA UH-60
ops-accept = VÕTA MISSIOON
ops-planning = PLANEERIB: { $pilots }
ops-flying = LENDAB: { $pilots }
ops-package = PAKETT, kuhu kuuluvad { $kinds }: täida siin kaks liiki { $mins } min jooksul, +{ $bonus } punkti kummagi eest
ops-sead-on-way = AI SEAD on teel
ops-request-sead = TELLI AI SEAD ({ $cost } materjali, { $aircraft } õhusõidukit)
    .facts = Oma löögilennukid ründavad selle koha SAM-positsiooni enne sind
    .disabled = Materjali või maapealseid õhusõidukeid ei ole piisavalt (vt JÕUD)

## Reeglid, materjal, jõud

ops-rule-damage = realistlik kahju
ops-rule-guidance = realistlik juhtimine
ops-rule-no-crashes = ilma allakukkumisteta
ops-rule-time = pidev aeg
ops-rule-weapons = realistlikud F-117A relvad
ops-rule-friendly-fire = oma tuli
ops-rule-classic = klassikaline
ops-materiel = MATERJAL
ops-materiel-blue = SININE { $materiel } (+{ $income }/h)
ops-materiel-red = PUNANE { $materiel } (+{ $income }/h)
ops-friendly = OMA
ops-enemy = VAENLANE
ops-section-forces = Jõud
ops-places-held = Valdusesolevad kohad
ops-materiel-row = Materjal
ops-income = Tulu tunnis
ops-income-from = allikad
ops-income-sources = { $base } + { $places } kohta x { $per_place } + { $cities } linna x { $per_city }
ops-fighters-ground = Hävitajad maas
ops-helicopters-ground = Helikopterid maas
ops-in-air = Õhus
ops-in-air-value = { $fighters } hävitajat, { $strike } löögilennukit, { $helicopters } helikopterit
ops-since-start = Algusest peale
ops-earned = teenitud
ops-repairs = remont
ops-operations = lennud, rünnakud, dessandid
ops-new-aircraft = uued õhusõidukid
ops-lost = kaotatud (linnad, piloodid)
ops-section-how = Kuidas see töötab
ops-how =
    Kumbki pool teenib materjali iga tund: põhisumma ning lisa iga valduses oleva koha ja iga terveks jäänud linna eest. Pommita vaenlase linnu (INTERDICTION), et tema tulu vähendada.
    AI maksab kõige eest, mida ta teeb: remont, iga hävitajalend, rünnakud ja oma dessandid. Materjali nappusel jääb tema kahju parandamata ja tema õhusõidukid jäävad koju.
    AI lendab ainult nende õhusõidukitega, mis tal on; alla tulistatud on kadunud, maandunud lendavad uuesti. Iga tund ostab kumbki pool uusi, kuni materjali jätkub.
    Sul ei saa kunagi puudu: sinu õhusõidukid, sinu baasid (pommitatud või mitte) ja sinu enda dessanditellimuste helikopterid on alati olemas.
ops-section-prices = Hinnad
ops-price-airfield = Lennuvälja remont
ops-price-sam = SAM-positsiooni remont
ops-price-city = Linna taastamine
ops-price-ship = Laeva remont
ops-price-gun = Kahur või MANPADS-meeskond
ops-price-sortie = Hävitajalend
ops-price-raid = Rünnak
ops-price-insertion = AI dessant
ops-price-fighter = Uus hävitaja
ops-price-helicopter = Uus helikopter
ops-price-city-lost = Linn hävitatud (kaotus)
ops-price-captured = Piloot vangistatud (kaotus)

## Sõjakaart

ops-click-to-watch = { $unit } - klõpsa vaatamiseks
ops-section-legend = Legend
ops-legend-friendly-airfield = Oma lennuväli
ops-legend-enemy-airfield = Vaenlase lennuväli
ops-legend-city = Oma / vaenlase linn
ops-legend-airfield = Oma / vaenlase lennuväli
ops-legend-insertion = Dessandi marsruut
ops-watch-hint = Klõpsa üksusel, et seda otse vaadata
ops-map-hint = Ratas: suum, lohistamine: nihutamine
ops-patriot = Patriot patarei
ops-drone-center = Droonide juhtimispunkt
ops-manned-by = Teenindab: { $pilot }
ops-manned-by-other = Teenindab teine mängija
ops-control = JUHI
    .facts = Võta selle konsool üle

## Debriifing, lõpp, logi, vestlus

ops-no-sortie = Ühtegi lendu pole veel tehtud
ops-section-sortie = Lend
ops-achieved = EESMÄRK SAAVUTATUD
ops-not-achieved = EESMÄRKI EI SAAVUTATUD
ops-waiting-raid = OOTAN RÜNNAKUT
ops-waiting-insertion = OOTAN DESSANTI
ops-flight-time = Lennuaeg { $mins } min
ops-kills = Hävitatud: õhus { $air }, maal { $ground }
ops-points = { $points } punkti
ops-points-bonus = { $points } punkti (paketibonus +{ $bonus })
ops-campaign-points = { $rank } - kampaania punktid: { $points }
ops-campaign-record = Kampaania: lende { $sorties }, saavutatud eesmärke { $won }
ops-section-meanwhile = Vahepeal
ops-victory = VÕIT
    .text = Vaenlasel pole enam ühtegi lennuvälja: operatsiooniala on meie.
ops-defeat = LÜÜASAAMINE
    .text = Oma lennuvälju ei ole alles: operatsiooniala on kaotatud.
ops-end-day = Päev { $day }. Omad kohad { $friendly }, vaenlase kohad { $enemy }.
ops-next-campaign = JÄRGMINE KAMPAANIA ALGAB { $time } PÄRAST
ops-next-campaign-now = JÄRGMINE KAMPAANIA ALGAB
ops-next-from-menu = Alusta järgmist kampaaniat peamenüüst.
ops-section-log = Kampaania logi
ops-section-chat = Vestlus
ops-no-messages = Sõnumeid pole veel
ops-message = Sõnum
    .hint = Kirjuta sõnum, Enter saadab
ops-send = SAADA

## Briifing

ops-briefing = MISSIOONI BRIIFING
ops-briefing-line = Algus: { $base } - { $km } km sihtmärgini - { $aircraft } - relvastus: { $stores }
ops-route-locked = Marsruut lukus: lendab { $pilots }
ops-route-threats = MARSRUUT JA OHUD
ops-take-off = TÕUSE ÕHKU
ops-radars-shown = Näidatud radarid: oma õhusõidukite avastatud, alles püsti

## Mängijad

ops-section-players = Mängijad
ops-col-name = NIMI
ops-col-status = OLEK
ops-col-sorties = LENDE
ops-col-points = PUNKTE
ops-col-pilot = PILOOT
ops-col-objectives = EESMÄRKE
ops-col-kills = HÄVITATUD ÕHK/MAA
ops-col-medal = PARIM MEDAL
ops-you = (sina)
ops-host = KORRALDAJA
ops-status-planning = Planeerib: { $title }
ops-status-board = Missioonitahvli juures
ops-status-flying = Lendab: { $title } - { $aircraft }
ops-status-reconnecting = Ühendus kadunud - õhusõidukit hoitakse { $seconds } s
ops-flown = { $flown } ({ $won } saavutatud)
ops-section-leaderboard = Edetabel
ops-alone = Lendad üksi. Koos lendamiseks korralda see kampaania peamenüü MITMIKMÄNGUST.
ops-rules = Selle kampaania reeglid (säilivad algusest peale): { $rules }
ops-no-sorties = Ühtegi lendu pole veel tehtud
