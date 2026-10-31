# Scáileán Ops an fheachtais (src/campaign/ops.rs).

ops-thunderstorms = STOIRMEACHA TOIRNÍ
ops-wx-short = AIMSIR { $now }, { $next } AG { $at }
ops-wx-line = { $weather } Ag athrú go { $next } ag { $at }.
ops-clock = LÁ { $day }  { $time }
ops-landed-at = TUIRLINGTHE: { $base }
ops-ground-crew = Tá an fhoireann talún réidh
ops-rearm = ATHAIRM & ATHBHREOSLÚ
ops-end-sortie = CRÍOCHNAIGH AN EITILT
ops-on-server = AR { $server }
ops-serving = AG FREASTAL
ops-tab-missions = MISIN
ops-tab-map = MAPA AN CHOGAIDH
ops-tab-air-defence = AERCHOSAINT
ops-tab-forces = FORCAÍ
ops-tab-players = IMREOIRÍ
ops-tab-debrief = DÍBHRIEFÁIL
ops-tab-log = LOG
ops-tab-chat = CAINT
ops-blue = GORM { $count }
ops-red = DEARG { $count }
ops-war-over = Tá an cogadh thart
ops-running = Insamhalt an chogaidh ar siúl
ops-save-exit = SÁBHÁIL & ÉIRIGH AS
ops-disconnect = DÍCHEANGAIL
ops-leave-local = AN FEACHTAS A SHÁBHÁIL AGUS A FHÁGÁIL?
    .text = Stopann an cogadh go dtí go leanann tú air. Cailltear aerárthaigh san aer; tosaíonn insealbhuithe atá ar siúl arís óna mbunáit.
ops-leave-host = AN FEACHTAS A SHÁBHÁIL AGUS FREASTAL A STOPADH?
    .text = Dícheanglaítear na himreoirí a tháinig isteach agus stopann an cogadh go dtí go leanann tú air. Cailltear aerárthaigh san aer; tosaíonn insealbhuithe atá ar siúl arís óna mbunáit.
ops-leave-client = DÍCHEANGAIL ÓN FHREASTALAÍ?
    .text = Leanann an feachtas ar aghaidh ar an bhfreastalaí; ní shábháiltear tada anseo. Críochnaíonn d'eitilt.
ops-unit-destroyed = Scriosadh: { $unit }. Ar ais chuig na hoibríochtaí

## Misin

ops-section-missions = Misin
ops-threat = BAGAIRT { $threat }
ops-no-missions = Gan misean: tá an fronta ciúin
ops-section-underway = Insealbhuithe agus ruathair
ops-enemy-liftoff = NAMHAD ag { $place }: éirí de thalamh i gceann { $mins } nóim
ops-liftoff = { $place }: éirí de thalamh i gceann { $mins } nóim
ops-enemy-helis = NAMHAD ag { $place }: { $count } héileacaptar ag teacht
ops-helis = { $place }: { $count } héileacaptar ar an mbealach
ops-raid-sead = SEAD AI ag { $place }
ops-raid-drones = DRÓNANNA ag { $place }
ops-raid = RUATHAR ar aerfort { $place }
ops-raid-launch = { $raid }: seoladh i gceann { $mins } nóim
ops-raid-on-way = { $raid }: { $count } ar an mbealach
ops-resupply-ai = ATHSHOLÁTHAR don aonad { $unit }: UH-60 ar an mbealach
ops-awacs = AWACS NAMHAD san aer { $mins } nóim eile
ops-down = ANUAS: { $pilot } - tarrtháil laistigh de { $mins } nóim
ops-aboard = AR BORD: { $pilot } - ar an mbealach abhaile
ops-none-underway = Aon cheann ar siúl
ops-ai-helicopter = Glacann héileacaptar AI leis i gceann { $time } mura nglacann píolóta UH-60 leis
ops-defences = Cosaint: { $sams } suíomh SAM, bagairt { $threat }
ops-recon-intel = EOLAS TAISCÉALAÍOCHTA
ops-order-insertion = ORDAIGH INSEALBHÚ
    .facts = Eitil an choimhdeacht chun iad a chlúdach, nó treoraigh iad sa UH-60
ops-from = Ó
ops-planned-by-group = Pleanáilte ag an ngrúpa
ops-stores = { $aircraft } - stoic: { $stores }
ops-fly-uh60 = EITIL AN UH-60
ops-accept = GLAC LEIS AN MISEAN
ops-planning = AG PLEANÁIL: { $pilots }
ops-flying = AG EITLEOIREACHT: { $pilots }
ops-package = PACÁISTE le { $kinds }: bain amach dhá chineál anseo laistigh de { $mins } nóim, +{ $bonus } pointe an ceann
ops-sead-on-way = Tá SEAD AI ar an mbealach
ops-request-sead = IARR SEAD AI ({ $cost } trealamh, { $aircraft } aerárthach)
    .facts = Téann aerárthaigh stailce cairdiúla i ndiaidh suíomh SAM san áit seo romhat
    .disabled = Gan trealamh nó aerárthaigh go leor ar an talamh (féach FORCAÍ)

## Rialacha, trealamh, forcaí

ops-rule-damage = damáiste réalaíoch
ops-rule-guidance = treoir réalaíoch
ops-rule-no-crashes = gan tuairteanna
ops-rule-time = am leanúnach
ops-rule-weapons = airm réalaíocha F-117A
ops-rule-friendly-fire = tine chairdiúil
ops-rule-classic = clasaiceach
ops-materiel = TREALAMH
ops-materiel-blue = GORM { $materiel } (+{ $income }/u)
ops-materiel-red = DEARG { $materiel } (+{ $income }/u)
ops-friendly = CAIRDIÚIL
ops-enemy = NAMHAD
ops-section-forces = Forcaí
ops-places-held = Áiteanna i seilbh
ops-materiel-row = Trealamh
ops-income = Ioncam san uair
ops-income-from = ó
ops-income-sources = { $base } + { $places } áit x { $per_place } + { $cities } cathair x { $per_city }
ops-fighters-ground = Trodaire ar an talamh
ops-helicopters-ground = Héileacaptair ar an talamh
ops-in-air = San aer
ops-in-air-value = { $fighters } trodaire, { $strike } stailce, { $helicopters } héileacaptar
ops-since-start = Ón tús
ops-earned = tuillte
ops-repairs = deisiúcháin
ops-operations = eitiltí, ruathair, insealbhuithe
ops-new-aircraft = aerárthaigh nua
ops-lost = caillte (cathracha, píolóta)
ops-section-how = Conas a oibríonn sé
ops-how =
    Tuilleann gach taobh trealamh gach uair: suim bhunúsach, níos mó do gach áit atá ina sheilbh agus do gach cathair slán ina measc. Buamáil cathracha an namhad (INTERDICTION) chun a ioncam a ghearradh.
    Íocann an AI as gach rud a dhéanann sé: deisiúcháin, gach eitilt trodaire, ruathair agus a insealbhuithe féin. Gan trealamh, fanann a dhamáiste gan deisiú agus fanann a aerárthaigh sa bhaile.
    Ní eitlíonn an AI ach na haerárthaigh atá aige; má lámhachtar anuas iad, tá siad caillte, má thuirlingíonn siad, eitlíonn siad arís. Gach uair ceannaíonn gach taobh cinn nua fad a mhaireann a threalamh.
    Ní bhíonn tú gearr riamh: bíonn d'aerárthaigh, do bhunáiteanna (buamáilte nó nach ea) agus héileacaptair d'orduithe insealbhaithe féin ann i gcónaí.
ops-section-prices = Praghsanna
ops-price-airfield = Deisiú aerfoirt
ops-price-sam = Deisiú suímh SAM
ops-price-city = Cathair atógtha
ops-price-ship = Deisiú loinge
ops-price-gun = Foireann gunna nó MANPADS
ops-price-sortie = Eitilt trodaire
ops-price-raid = Ruathar
ops-price-insertion = Insealbhú AI
ops-price-fighter = Trodaire nua
ops-price-helicopter = Héileacaptar nua
ops-price-city-lost = Cathair scriosta (caillte)
ops-price-captured = Píolóta gafa (caillte)

## Mapa an chogaidh

ops-click-to-watch = { $unit } - cliceáil chun féachaint
ops-section-legend = Finscéal
ops-legend-friendly-airfield = Aerfort cairdiúil
ops-legend-enemy-airfield = Aerfort namhad
ops-legend-city = Cathair chairdiúil / namhad
ops-legend-airfield = Aerpháirc chairdiúil / namhad
ops-legend-insertion = Bealach an tsáite
ops-watch-hint = Cliceáil aonad chun féachaint air beo
ops-map-hint = Roth: zúmáil, tarraing: pan
ops-patriot = Cadhnra Patriot
ops-drone-center = Ionad ceannais dróin
ops-manned-by = Foireann: { $pilot }
ops-manned-by-other = Foireann: imreoir eile
ops-control = RIALAIGH
    .facts = Glac seilbh ar a chonsól

## Díbhriefáil, deireadh, log, caint

ops-no-sortie = Gan eitilt fós
ops-section-sortie = Eitilt
ops-achieved = CUSPÓIR BAINTE AMACH
ops-not-achieved = CUSPÓIR NÍOR BAINEADH AMACH
ops-waiting-raid = AG FANACHT LEIS AN RUATHAR
ops-waiting-insertion = AG FANACHT LEIS AN INSEALBHÚ
ops-flight-time = Am eitilte: { $mins } nóim
ops-kills = Scriosadh: { $air } aerárthach, { $ground } ar an talamh
ops-points = { $points } pointe
ops-points-bonus = { $points } pointe (bónas pacáiste +{ $bonus })
ops-campaign-points = { $rank } - { $points } pointe feachtais
ops-campaign-record = Feachtas: { $sorties } eitilt, { $won } cuspóir bainte amach
ops-section-meanwhile = Idir an dá linn
ops-victory = BUAIDH
    .text = Níl aerfort ar bith ag an namhad a thuilleadh: is linne an amharclann oibríochta.
ops-defeat = TREALLÚS
    .text = Níl aerfort cairdiúil ar bith fágtha: tá an amharclann oibríochta caillte.
ops-end-day = Lá { $day }. Áiteanna cairdiúla: { $friendly }, áiteanna namhad: { $enemy }.
ops-next-campaign = TOSAÍONN AN CHÉAD FHEACHTAS EILE I GCEANN { $time }
ops-next-campaign-now = TÁ AN CHÉAD FHEACHTAS EILE AG TOSÚ
ops-next-from-menu = Tosaigh an chéad fheachtas eile ón bpríomh-roghchlár.
ops-section-log = Log an fheachtais
ops-section-chat = Caint
ops-no-messages = Gan teachtaireacht fós
ops-message = Teachtaireacht
    .hint = Clóscríobh teachtaireacht, seolann Enter í
ops-send = SEOL

## Briefing

ops-briefing = BRIEFING MISIN
ops-briefing-line = Ó { $base } - { $km } km go dtí an sprioc - { $aircraft } - stoic: { $stores }
ops-route-locked = Bealach faoi ghlas: { $pilots } ag eitilt air
ops-route-threats = BEALACH BEALACH & GAIRMEANNA BAGAIRTÍ
ops-take-off = ÉIRIGH DE THALAMH
ops-radars-shown = Radair a thaispeántar: braite ag aerárthaigh chairdiúla, fós ina seasamh

## Imreoirí

ops-section-players = Imreoirí
ops-col-name = AINM
ops-col-status = STAD
ops-col-sorties = EITILTÍ
ops-col-points = POINTÍ
ops-col-pilot = PÍOLÓTA
ops-col-objectives = CUSPÓIRÍ
ops-col-kills = MARÚ AER/TALAMH
ops-col-medal = MEDAL IS FEARR
ops-you = (tusa)
ops-host = ÓSTA
ops-status-planning = Ag pleanáil: { $title }
ops-status-board = Ar an gclár misean
ops-status-flying = Ag eitilt: { $title } - { $aircraft }
ops-status-reconnecting = Nasc caillte - aerárthach coinnithe { $seconds } s
ops-flown = { $flown } ({ $won } bainte amach)
ops-section-leaderboard = Clár ceannairí
ops-alone = Ag eitilt as do stuaim féin. Chun eitilt le chéile, óstáil an feachtas seo ó ILTOISEOIR sa phríomh-roghchlár.
ops-rules = Rialacha an fheachtais seo (coinnithe óna thús): { $rules }
ops-no-sorties = Gan eitilt fós
