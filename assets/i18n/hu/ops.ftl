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
ops-tab-air-defence = LÉGVÉDELEM
ops-tab-forces = ERŐK
ops-tab-players = JÁTÉKOSOK
ops-tab-debrief = KIÉRTÉKELÉS
ops-tab-log = NAPLÓ
ops-tab-chat = CSEVEGÉS
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
ops-section-underway = Deszantok és légitámadások
ops-enemy-liftoff = ELLENSÉG, { $place }: felszállás { $mins } perc múlva
ops-liftoff = { $place }: felszállás { $mins } perc múlva
ops-enemy-helis = ELLENSÉG, { $place }: { $count } helikopter közeledik
ops-helis = { $place }: { $count } helikopter úton
ops-raid-sead = MI SEAD: { $place }
ops-raid-drones = DRÓNOK: { $place }
ops-raid = LÉGITÁMADÁS: { $place } repülőtér
ops-raid-launch = { $raid }: indulás { $mins } perc múlva
ops-raid-on-way = { $raid }: { $count } úton
ops-resupply-ai = UTÁNPÓTLÁS: { $unit }, UH-60 úton
ops-awacs = ELLENSÉGES AWACS még { $mins } percig a levegőben
ops-down = LEZUHANT: { $pilot } - mentés { $mins } percen belül
ops-aboard = FEDÉLZETEN: { $pilot } - úton hazafelé
ops-none-underway = Nincs folyamatban
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
ops-rule-classic = klasszikus
ops-materiel = HADIANYAG
ops-materiel-blue = KÉK { $materiel } (+{ $income }/ó)
ops-materiel-red = PIROS { $materiel } (+{ $income }/ó)
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
ops-watch-hint = Kattints egy egységre az élő megfigyeléshez
ops-map-hint = Kerék: nagyítás, húzás: eltolás
ops-patriot = Patriot üteg
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
ops-points = { $points } pont
ops-points-bonus = { $points } pont (kötelékbónusz +{ $bonus })
ops-campaign-points = { $rank } - { $points } kampánypont
ops-campaign-record = Kampány: { $sorties } bevetés, { $won } teljesített cél
ops-section-meanwhile = Közben
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
ops-flown = { $flown } ({ $won } teljesítve)
ops-section-leaderboard = Ranglista
ops-alone = Egyedül repülsz. Együtt repüléshez indítsd el ezt a kampányt a főmenü MULTIPLAYER pontjából.
ops-rules = A kampány szabályai (az indulása óta változatlanok): { $rules }
ops-no-sorties = Még nem volt bevetés
