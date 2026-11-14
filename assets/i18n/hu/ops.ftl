# A kampány Műveletek képernyője (src/campaign/ops.rs).

ops-thunderstorms = VIHAROK
ops-wx-short = IDŐJÁRÁS: { $now }, { $next } ekkor: { $at }
ops-wx-line = { $weather } Változás: { $next }, ekkor: { $at }.
ops-clock = { $day }. NAP  { $time }
ops-landed-at = LESZÁLLT: { $base }
ops-ground-crew = A földi személyzet készen áll
ops-rearm = ÚJRAFEGYVEREZÉS ÉS TANKOLÁS
ops-end-sortie = BEVETÉS BEFEJEZÉSE
ops-on-server = SZERVER: { $server }
ops-serving = SZOLGÁLTATÁS
ops-tab-missions = KÜLDETÉSEK
ops-tab-map = HADITÉRKÉP
ops-tab-flights = REPÜLÉSEK
ops-tab-air-defence = LÉGVÉDELEM
ops-tab-forces = ERŐK
ops-tab-players = JÁTÉKOSOK
ops-tab-debrief = KIÉRTÉKELÉS
ops-tab-log = NAPLÓ
ops-tab-chat = CSEVEGÉS
ops-top-campaign = HADJÁRAT
ops-view-war = HÁBORÚ
ops-view-ground = SZÁRAZFÖLD
ops-view-weather = IDŐJÁRÁS
ops-bar-missions = Küldetések a táblán: { $count }
ops-bar-hint = Kattintson egy célpontjelölőre vagy a KÜLDETÉSEK gombra a választáshoz, egy egységre az élő nézethez
ops-filter-all = MIND
ops-filter-attack = TÁMADÁS
ops-filter-support = TÁMOGATÁS
ops-watch = MEGFIGYEL
ops-blue = KÉK { $count }
ops-red = PIROS { $count }
ops-war-over = A háború véget ért
ops-running = A háborús szimuláció fut
ops-save-exit = MENTÉS ÉS KILÉPÉS
ops-disconnect = LEVÁLASZTÁS
ops-leave-local = MENTED ÉS ELHAGYOD A KAMPÁNYT?
    .text = A háború megáll, amíg folytatod. A levegőben lévő gépek elvesznek; a folyamatban lévő deszantok újra a bázisról indulnak.
ops-leave-host = MENTED ÉS LEÁLLÍTOD A KAMPÁNY SZOLGÁLTATÁSÁT?
    .text = A csatlakozott játékosok leválasztódnak, és a háború megáll, amíg folytatod. A levegőben lévő gépek elvesznek; a folyamatban lévő deszantok újra a bázisról indulnak.
ops-leave-client = LEVÁLASZTÓDSZ A SZERVERRŐL?
    .text = A kampány a szerveren tovább megy; itt semmi nem mentődik. A bevetésed véget ér.
ops-unit-destroyed = Megsemmisült: { $unit }; vissza a műveletekhez

## Küldetések

ops-section-missions = Küldetések
ops-threat = VESZÉLY { $threat }
ops-no-missions = Nincs küldetés: a front csendes
ops-ai-helicopter = Egy MI-helikopter viszi { $time } múlva, hacsak egy UH-60 pilóta nem teszi
ops-defences = Védelem: { $sams } SAM-állás, veszély { $threat }
ops-recon-intel = FELDERÍTÉSI ADATOK
ops-order-insertion = DESZANT ELRENDELÉSE
    .facts = Repüld a kíséretet a fedezésükre, vagy vezesd be őket az UH-60-nal
ops-from = Innen
ops-planned-by-group = A csoport tervezte
ops-stores = { $aircraft } - fegyverzet: { $stores }
ops-fly-uh60 = UH-60 REPÜLÉSE
ops-accept = KÜLDETÉS ELFOGADÁSA
ops-fly-with-ai = FELSZÁLLÁS
ops-take-control = IRÁNYÍTÁS ÁTVÉTELE
    .facts = Repüld tovább az MI-kötelék egyik gépét onnan, ahol van
ops-join-gunner = CSATLAKOZÁS LÖVÉSZKÉNT
    .facts = Ülj a gép lövészkonzoljához: forgasd a célzókonténert és tüzelj az oldalágyúkkal, amíg a pilótája repül
    .ai = Ülj a gép lövészkonzoljához: az MI körözik a pont körül, amelyet a célzókonténerrel tartasz
ops-planning = TERVEZI: { $pilots }
ops-flying = REPÜLI: { $pilots }
ops-package = KÖTELÉK ({ $kinds }): teljesíts itt két fajtát { $mins } percen belül, +{ $bonus } pont mindegyikért
ops-sead-on-way = Az MI SEAD úton van
ops-request-sead = MI SEAD KÉRÉSE ({ $cost } hadianyag, { $aircraft } gép)
    .facts = Baráti csapásmérők megtámadják a hely egy SAM-állását előtted
    .disabled = Nincs elég hadianyag vagy gép a földön (lásd ERŐK)

## Szabályok, hadianyag, erők

ops-rule-damage = valósághű sérülések
ops-rule-guidance = valósághű irányítás
ops-rule-no-crashes = nincs lezuhanás
ops-rule-time = folyamatos idő
ops-rule-weapons = valósághű F-117A fegyverzet
ops-rule-friendly-fire = baráti tűz
ops-rule-airframe-losses = a veszteségek erőforrásba kerülnek
ops-rule-no-ai-missions = nincsenek baráti MI-küldetések
ops-rule-classic = klasszikus
ops-materiel = HADIANYAG
ops-materiel-blue = KÉK { $materiel } (+{ $income }/ó)
ops-materiel-red = PIROS { $materiel } (+{ $income }/ó)
ops-crew-tempo = AZ ELLENSÉG ÜTEME ×{ $pace }
ops-due = T-{ $mins } perc
ops-due-now = MOST
ops-front-blue = FRONT: A KÉK TÉRT NYER
ops-front-red = FRONT: A PIROS TÉRT NYER
ops-front-even = FRONT: TART
ops-operation = { $place } elfoglalása:
ops-friendly = BARÁTI
ops-enemy = ELLENSÉGES
ops-section-forces = Erők
ops-places-held = Birtokolt helyek
ops-materiel-row = Hadianyag
ops-income = Bevétel óránként
ops-income-from = forrás
ops-income-sources = { $base } + { $places } hely x { $per_place } + { $cities } város x { $per_city }
ops-fighters-ground = Vadászok a földön
ops-helicopters-ground = Helikopterek a földön
ops-in-air = A levegőben
ops-in-air-value = { $fighters } vadász, { $strike } csapásmérő, { $helicopters } helikopter
ops-since-start = Az indulás óta
ops-earned = szerzett
ops-repairs = javítások
ops-operations = bevetések, légitámadások, deszantok
ops-new-aircraft = új gépek
ops-lost = elveszett (városok, pilóták)
ops-section-how = Hogyan működik
ops-how =
    Mindkét oldal óránként hadianyagot szerez: egy alapmennyiséget, többet minden birtokolt helyért és az ezek közötti minden ép városért. Bombázd az ellenség városait (ELLÁTÁSI VONALAK), hogy csökkentsd a bevételét.
    Az MI mindenért fizet, amit tesz: javításokért, minden vadászbevetésért, légitámadásokért és saját deszantjaiért. Ha kevés a hadianyaga, a sérülései javítatlanok maradnak, és a gépei otthon maradnak.
    Az MI csak azokkal a gépekkel repül, amelyekkel rendelkezik; a lelőttek elvesznek, a leszálltak újra repülnek. Óránként mindkét oldal újakat vásárol, amíg a hadianyaga tart.
    Neked sosem fogy el: a gépeid, a bázisaid (bombázva vagy sem) és a saját deszantparancsaid helikopterei mindig rendelkezésre állnak.
ops-how-repairs = Mindkét oldal egyszerre csak néhány állást javít (több ép várossal többet), a fronthoz legközelebbieket először, és a javítás után hamar újra lerombolt állás javítása minden alkalommal tovább tart.
ops-how-airframes = A veszteségek erőforrásba kerülnek: minden gép, amit egy pilóta elveszít, eggyel kevesebb vadász vagy helikopter a saját MI-nek.
ops-section-prices = Árak
ops-price-airfield = Repülőtér javítása
ops-price-sam = SAM-állás javítása
ops-price-city = Város újjáépítése
ops-price-ship = Hajó javítása
ops-price-gun = Ágyú vagy MANPADS-csapat
ops-price-sortie = Vadászbevetés
ops-price-raid = Légitámadás
ops-price-insertion = MI-deszant
ops-price-fighter = Új vadász
ops-price-helicopter = Új helikopter
ops-price-city-lost = Város megsemmisült (veszteség)
ops-price-captured = Pilóta fogságba esett (veszteség)

## Haditérkép

ops-click-to-watch = { $unit } - kattints a megfigyeléshez
ops-section-legend = Jelmagyarázat
ops-legend-friendly-airfield = Baráti repülőtér
ops-legend-enemy-airfield = Ellenséges repülőtér
ops-legend-city = Baráti / ellenséges város
ops-legend-airfield = Saját / ellenséges repülőtér
ops-legend-insertion = Deszant útvonala
ops-legend-missions = Küldetések a táblán
ops-legend-defence-reach = Légvédelmi állások és hatótáv
ops-watch-hint = Kattints egy egységre az élő megfigyeléshez
ops-map-hint = Kerék: nagyítás, húzás: eltolás
ops-patriot = Patriot üteg
ops-nasams = NASAMS üteg
ops-manned-by = Kezeli: { $pilot }
ops-manned-by-other = Egy másik játékos kezeli
ops-control = IRÁNYÍTÁS
    .facts = A konzoljának átvétele

## Kiértékelés, vége, napló, csevegés

ops-no-sortie = Még nem volt bevetés
ops-section-sortie = Bevetés
ops-achieved = A CÉL TELJESÜLT
ops-not-achieved = A CÉL NEM TELJESÜLT
ops-waiting-raid = VÁRAKOZÁS A LÉGITÁMADÁSRA
ops-waiting-insertion = VÁRAKOZÁS A DESZANTRA
ops-flight-time = Repülési idő: { $mins } perc
ops-kills = Lelövések: { $air } gép, { $ground } a földön
ops-crew = Személyzet: { $names }
ops-points = { $points } pont
ops-points-bonus = { $points } pont (kötelékbónusz +{ $bonus })
ops-campaign-points = { $rank } - { $points } kampánypont
ops-campaign-record = Kampány: { $sorties } bevetés, { $won } teljesített cél
ops-section-meanwhile = Közben
ops-section-effects = Hatásod a háborúra
ops-victory = GYŐZELEM
    .text = Az ellenségnek nincs több repülőtere: a hadszíntér a miénk.
ops-defeat = VERESÉG
    .text = Nem maradt baráti repülőtér: a hadszíntér elveszett.
ops-end-day = { $day }. nap. Baráti helyek: { $friendly }, ellenséges helyek: { $enemy }.
ops-next-campaign = A KÖVETKEZŐ KAMPÁNY INDUL: { $time } múlva
ops-next-campaign-now = A KÖVETKEZŐ KAMPÁNY ÉPP INDUL
ops-next-from-menu = Indítsd el a következő kampányt a főmenüből.
ops-section-log = Kampánynapló
ops-section-chat = Csevegés
ops-no-messages = Még nincs üzenet
ops-message = Üzenet
    .hint = Írj üzenetet, az Enter elküldi
ops-send = KÜLDÉS

## Eligazítás

ops-briefing = KÜLDETÉS-ELIGAZÍTÁS
ops-briefing-line = Kiindulás: { $base } - { $km } km a célpontig - { $aircraft } - fegyverzet: { $stores }
brief-choose-aircraft = VÁLASSZON GÉPET
brief-aircraft-for = Gépek itt: { $base }, feladat: { $kind }
brief-next = TOVÁBB: { $step }
brief-from = { $aircraft }, innen: { $base }, { $km } km a célig
ops-route-locked = Útvonal zárolva: repüli: { $pilots }
ops-route-threats = ÚTVONAL ÉS VESZÉLYEK
ops-take-off = FELSZÁLLÁS
ops-radars-shown = Látható radarok: baráti gépek észlelték, még működnek

## Játékosok

ops-section-players = Játékosok
ops-col-name = NÉV
ops-col-status = ÁLLAPOT
ops-col-sorties = BEVETÉSEK
ops-col-points = PONTOK
ops-col-pilot = PILÓTA
ops-col-objectives = CÉLOK
ops-col-kills = LELÖVÉS LÉGI/FÖLDI
ops-col-medal = LEGJOBB ÉREM
ops-you = (te)
ops-host = HÁZIGAZDA
ops-status-planning = Tervezi: { $title }
ops-status-board = A küldetéstáblánál
ops-status-flying = Repüli: { $title } - { $aircraft }
ops-status-reconnecting = A kapcsolat megszakadt - a gép tartva: { $seconds } mp
ops-status-gunner = Lövész { $pilot } mellett: { $title } - { $aircraft }
ops-flown = { $flown } ({ $won } teljesítve)
ops-section-leaderboard = Ranglista
ops-alone = Egyedül repülsz. Együtt repüléshez indítsd el ezt a kampányt a főmenü MULTIPLAYER pontjából.
ops-rules = A kampány szabályai (az indulása óta változatlanok): { $rules }
ops-no-sorties = Még nem volt bevetés
ops-ai-load = Egy MI-helikopter { $time } múlva átveszi a rakományt, ha nem teszi meg UH-60-pilóta
ops-ai-takes-off = Felszállás { $time } múlva
ops-ai-aircraft = MI { $aircraft }
ops-cargo-in = Rakomány { $delivered } / { $loads } megérkezett - a leszállóhely még { $time } órát vár
ops-rule-classic-terrain = klasszikus terep
ops-rule-classic-ai-flight = klasszikus MI-repülés
ops-layers-hint = RÉTEGEK (jobbra fent): amit a térkép mutat
ops-map-tab = TÉRKÉP
ops-on-radio = A RÁDIÓN
ops-mute = NÉMÍTÁS
    .facts = Hallani (vagy nem) ezt a játékost a rádión
ops-unmute = HANG BE

## Flights

ops-section-flights = Saját repülések
ops-no-flights = Nincs saját gép a levegőben vagy felszállásra várva
ops-flight-pilot = Pilóta:
ops-flight-gunner = Lövész:
ops-seat-ai = MI
ops-seat-free = szabad
ops-flight-airborne = A levegőben
ops-flight-on-ground = A földön
ops-flight-starting = Indítás
