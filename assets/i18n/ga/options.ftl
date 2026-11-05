# An fhuinneog ROGHANNA (src/options.rs) agus a leathanaigh sa roghchlár eitilte.

options-title = ROGHANNA
options-tab-game = CLUICHE
options-tab-graphics = GRAFAICÍ
options-tab-sound = FUAIM
options-tab-controls = RIALUITHE
options-footer = Sábháilte do na heitiltí seo chugainn; roghnaítear na lasca réalachais le gach cluiche

## CLUICHE

options-section-language = Teanga
options-section-classic-modern = Clasaiceach / Nua-aimseartha
options-cockpit = Cábán
    .facts =
        Nua-aimseartha: scáileáin ardtaifigh le cnaipí ar an imeall
        Clasaiceach: leagan amach painéal uirlisí ré an DOS agus scáileáin 77x60 (F-117A)
options-hud = HUD
    .facts =
        Nua-aimseartha: HUD ardtaifigh uaine i stíl F-16 le modhanna NAV, AA agus AG
        Clasaiceach: HUD ré an DOS
options-avionics = Avionics
    .facts =
        Nua-aimseartha: formáidí MFD i stíl F-16 ar gach MFD (taispeáint cúrsa, HARM, targeting pod, léarscáil, stóir) le cúrsóir so-ghluaiste
        Clasaiceach: léarscáil agus TAC ar chlé, ceamara, WAY, WPN agus ORD ar dheis
options-flight-model = Samhail eitilte
    .facts =
        Nua-aimseartha: samhail eitilte aeraidinimiciúil le fly-by-wire: ordaíonn an stick luas pitch agus roll; nuair a scaoiltear é, fanann an srón i bhfeidhm agus coinnítear an cosán eitilte
        Clasaiceach: samhail eitilte ré an DOS
options-section-flying = Eitilt
options-auto-trim = Uath-trim
    .facts =
        Nuair a scaoiltear an stick, coinníonn an trim an dearcadh mar a d'fhág tú é
        Héileacaptair: an tsrón gan ardú le luas a thuilleadh; scairdeitleáin: cuireann dromchlaí rialaithe damáistithe i dtrim
        Trim láimhe: Alt + eochracha saighde, lárú le Alt + Home
options-tooltips = Leideanna uirlise an chockpit
    .facts = Cad is rialú nó scáileán cockpit ann, faoin luch nó faoi bharr méire i VR
options-player-labels = Ainmneacha imreoirí
    .facts = Ilimreoir: ainmneacha na bpíolótaí eile os cionn a n-aerárthaí, is cuma cé chomh fada uait
options-label-size =
    .facts =
        Méid ainmneacha na n-imreoirí
        50%: an réamhshocrú
options-seat-height = Airde an tsuíocháin
    .facts =
        Cé chomh hard is a shuíonn tú sa chockpit, ar an scáileán agus i VR
        0 cm: gnáth-airde súl an aerárthaigh
options-section-vr-size = Méid an chockpit i VR
options-vr-size =
    .facts =
        Méid an chockpit (agus an domhain) sa cheannsraith
        100%: méid iarbhír
options-section-gif = Taifeadadh GIF
options-gif-size = MÉID
options-gif-rate = FRÁMAÍ
options-gif-scale-100 =
    .facts = Taifeach iomlán na fuinneoige: comhaid mhóra
options-gif-scale-25 =
    .facts = Comhaid bheaga
options-gif-rate-50 =
    .facts = Ach amháin nuair a ritheann an cluiche ag 100 fps nó níos mó (gabhtar gach dara fráma)
options-gif-hint = F12 (mura ndearnadh athcheangal) tosaíonn agus stopann; sábháilte i d'fhillteán Pictiúir

## GRAFAICÍ

options-section-rendering = Rindreáil
options-shaders = Shaders
    .facts =
        Nua-aimseartha: scáthú treorach caolchúiseach, ceo faid mín
        Clasaiceach: scáthú comhréidh
options-shadows = Scáthanna
    .facts =
        Nua-aimseartha: scáthanna na gréine sa chockpit agus sa domhan (cosnaíonn sé am GPU)
        Clasaiceach: gan scáthanna
options-point-lights = Solais phointe
    .facts =
        Nua-aimseartha: solas tuilte an chockpit agus na scáileán; soilsíonn pléascanna, scamaill agus soilse rúidbhealaigh an radharc
        Clasaiceach: solas na gréine agus na spéire amháin
options-time-of-day = Am den lá
    .facts =
        Nua-aimseartha: leanann an ghrian agus an ghealach cloc an mhisin, dorchaíonn agus geal an spéir leis an ngrian; réaltaí agus céim na gealaí san oíche
        Nua-aimseartha: socraíonn solas na gealaí cé chomh fada uait a fheiceann píolótaí naimhdeacha thú san oíche (2 km gan gealach, 4 km faoi ghealach lán)
        Clasaiceach: athrú mall datha agus grian sheasta
options-msaa = Frith-ailiasú (MSAA)
    .facts =
        Imill mhíne: MSAA 4x, 2x i VR (cosnaíonn sé am GPU)
        As: imill chrua picteilíní
options-auto-detail = Mionsonraí uathoibríocha na tírdhreacha (VR)
    .facts =
        Sa cheannsraith, laghdaíonn raon na mbailte, na foraoisí agus na bpáirceanna nuair a bhíonn fráma déanach agus fásann sé ar ais nuair a bhíonn am le spáráil
        Fanann na socruithe grafaicí mar a roghnaíodh
options-sun-moon = An ghrian agus an ghealach
    .facts =
        An ghrian sa spéir sa lá, an ghealach agus na réaltaí san oíche
        Taispeánann an ghealach céim an mhisin
options-cloud-quality = Scamaill
cloud-quality-auto = Uathoibríoch
    .facts = Íseal i VR, ard ar an scáileán
cloud-quality-low = Íseal
    .facts = Scamaill gharbha ag trian den taifeach (is tapúla)
cloud-quality-medium = Meánach
    .facts = Leath den taifeach, mionsonraí níos mine
cloud-quality-high = Ard
    .facts = Taifeach iomlán, na mionsonraí is mine agus féinscáth (cosnaíonn sé am GPU)

## FUAIM

options-section-volume = Airde fuaime
options-volume-master = Príomh
    .facts = Gach fuaim
options-volume-engine = Inneall
    .facts = D'inneall nó do rótar, agus na haerárthaigh a chloistear timpeall ort
options-volume-effects = Éifeachtaí
    .facts = Airm, pléascanna, gear, raidió agus an chuid eile
options-volume-warnings = Rabhaidh
    .facts = Adharc stalla, rabhaidh radair agus diúracáin, siren tine, tonnta an Sidewinder
options-volume-rwr = RWR
    .facts = Toin rabhaidh radair, mar chuid d'airde na rabhadh
options-section-rwr-tones = Toin rabhaidh radair
rwr-tones-all = GACH CEANN
    .facts = Radar do do shoilsiú, gach scuabadh a bhraitheann thú, seoltaí diúracán
rwr-tones-detect = BRAITH AMHÁIN
    .facts =
        Ton amháin nuair a bhraitheann radar thú (arís tar éis do na radair thú a scaoileadh ar feadh tamaill)
        Agus seoltaí diúracán
rwr-tones-launch = SEOLTAÍ AMHÁIN
    .facts = Diúracáin a seoladh ort amháin
rwr-tones-off = AS
    .facts = Gan toin rabhaidh radair (tugann na soilse agus na scáileáin rabhadh fós)
options-section-cockpit = Cábán
options-low-pass = Scagaire ísealpháis an einnill
    .facts =
        Taobh istigh den aerárthach, bíonn fuaim múchta ar an inneall (bainíonn an canopy agus an clogad na hardtonnta de)
        As: an fhuaim chéanna agus taobh amuigh

## RIALUITHE

options-section-bindings = Ceangail
options-keys-joystick = EOCHRACHA AGUS JOYSTICK
    .facts =
        Eochracha, aiseanna agus cnaipí an joystick agus an HOTAS a cheangal
        Sábháilte do na heitiltí seo chugainn
options-section-head-tracking = Rianú cinn agus súl
options-head-tracking = Rianú cinn agus súl
    .facts =
        Casann do cheann agus bogann an radharc sa chockpit (ar an deasc; i VR, is é an cheannsraith a dhéanann é)
        Clár rianaithe (OpenTrack...): aschur FreeTrack, nó UDP chuig 127.0.0.1 port 4242
        Home: déantar lár de shuíomh reatha an chinn
options-tracking = Rianú ({ $source })
options-no-tracker = Níor aimsíodh aon rianóir: tosaigh an clár rianaithe

## EFFECTS

options-section-effects = Éifeachtaí
options-section-retro = Retro
effect-shake = Croitheadh ceamara
    .facts =
        Croitheann an radharc leis an ngunna, pléascanna in aice láimhe, stalla, tuirlingtí agus an rúidbhealach
        VR: creathann an cábán, fanann do cheann socair
effect-flashes = Splancanna comhraic
    .facts = Pléascanna frith-aerárthaigh, pléascanna agus tosaithe diúracán a las na scamaill, an talamh agus an cábán ar feadh nóiméid
effect-barrage = Baráiste oíche
    .facts =
        Cogadh gnáth san oíche: lámhachann gunnaí na gcathracha aimsitheoirí go dall isteach sa spéir, scuabann soilsitheoirí thar na gcathracha
        Gabhann a soilsitheoirí drónaí namhad os cionn cathrach agus tarraingíonn siad tine a gunnaí
        Don chuma amháin: ní bhuaileann an baráiste faic
effect-cockpit-night = Soilsiú oíche an chábáin
    .facts =
        Solas dearg breacdhorcha sa chábán tar éis titim na hoíche (gorm-ghlas faoi na spéaclaí)
        Pulsálann an master caution ómra thar an bpainéal fad atá rabhadh gníomhach
effect-nvg = Spéaclaí amhairc oíche
    .facts =
        Fairsingíonn solais ghéara ina ha-lóanna, bíonn an pictiúr níos grán sa dorchadas
        Deasc: réimse amhairc cruinn na spéaclaí
effect-snow = Sneachta
    .facts = Os cionn leibhéal an reo tugann stoirmeacha sneachta in ionad báistí: íseal i ngeimhreadh thuaidh, ard sa samhradh
effect-mist = Ceo talún
    .facts =
        Scáthaitheoirí nua-aimseartha: luíonn ceo thar thalamh íseal agus an fharraige ag breacadh an lae, i ndorchadas agus san oíche
        Tanaí sna fásaigh, ceann ar bith i stoirmeacha
effect-cloud-shadows = Scáthanna scamall
    .facts = Scáthaitheoirí nua-aimseartha: scáthanna na scamall ar an talamh thíos
effect-vapour = Gal agus sruth rótair
    .facts =
        Le hainlithe géara, rianta gaile ó bharr na sciathán
        Ardaíonn héileacaptar íseal ag ainliú deannach, nó sprae os cionn uisce
        Ardaíonn feithiclí ag gluaiseacht deannach ina ndiaidh
effect-night-sky = Spéir na hoíche
    .facts =
        Anois is arís réaltaí scuaine, croithníonn na réaltaí ísle ar bharr na spéire
        Na soilse thuaidh thar Nordkap
effect-vga = Dathanna VGA
    .facts = Cuma 1989 le scáthú nua-aimseartha: 64 chéim in aghaidh an chainéil datha, mar a bheadh ar chárta VGA, le dithering
effect-scanlines = Línte scanta
    .facts = Línte scanta monatóra CRT (ní i VR)

## LANGUAGE NAMES

lang-name-en = Béarla
lang-name-bg = Bulgáiris
lang-name-zh-Hans = Sínis (Simplithe)
lang-name-hr = Cróitis
lang-name-cs = Seicis
lang-name-da = Danmhairgis
lang-name-nl = Ollainnis
lang-name-et = Eastóinis
lang-name-fi = Fionlainnis
lang-name-fr = Fraincis
lang-name-de = Gearmáinis
lang-name-el = Gréigis
lang-name-hu = Ungáiris
lang-name-id = Indinéisis
lang-name-ga = Gaeilge
lang-name-it = Iodáilis
lang-name-ja = Seapáinis
lang-name-ko = Cóiréis
lang-name-lv = Laitvis
lang-name-lt = Liotuáinis
lang-name-mt = Máltais
lang-name-pl = Polainnis
lang-name-pt-BR = Portaingéilis (an Bhrasaíl)
lang-name-ro = Rómáinis
lang-name-ru = Rúisis
lang-name-sk = Slóvaicis
lang-name-sl = Slóivéinis
lang-name-es = Spáinnis
lang-name-sv = Sualainnis
lang-name-uk = Úcráinis

## Browser build: offer to play in the browser's language (src/i18n.rs)

language-offer-question = Teanga an bhrabhsálaí: { $language }. An bhfuil tú ag iarraidh imirt sa teanga seo?
language-offer-yes = Tá
language-offer-no = Níl
language-offer-never = Ná fiafraigh arís
effect-sunbeams = Gathanna gréine
    .facts = Bíoga solais gréine trí bhearnaí scamallachta briste
options-msaa-forced = As cé go bhfuil an pixealú (retro) air
options-volume-radio = Raidió
    .facts = An chuid eile d'imreoirí ag labhairt ar an raidió coop, agus an ATC labhartha
options-volume-atc = Guth ATC
    .facts = Glaonna labhartha ATC, mar chuid d'airde an raidió
options-volume-music = Ceol
    .facts = Ceol an phríomh-roghchláir
options-section-atc = ATC labhartha
options-section-radio = Raidió coop
options-push-to-talk = Brúigh agus labhair
    .facts =
        Coinnigh síos an eochair cainte (Caps Lock, nó cnaipe joystick nó VR sannta do PTT) agus cloiseann gach imreoir thú
        Osclaíonn an micreafón an chéad uair a bhrúann tú
options-mic-gain = GLAC MICREAFÓIN
    .facts =
        Cé chomh hard is a théann do mhicreafón amach
        100%: mar atá sé
options-mic-default = Réamhshocrú
options-voice-text-only = téacs amháin
options-voices-line = Rialaitheoirí: { $controllers }   Píolótach: { $pilot }
options-voices = GLOTHA
    .facts =
        Íoslódáil glotha do na glaonna raidió, scrios iad
        Roghnaigh glotha na rialaitheoirí agus do ghuth féin, agus éist leo
options-voices-web = Téacs amháin sa bhrabhsálaí
options-tracking-mouse = Breathnú leis an luchóg i rianú
    .facts =
        Air: casann agus súmálann cnaipe deas na luchóige radharc na cábáin fós
        As: fad a sheolann an rianaitheoir, ní dhéanann cnaipe deas na luchóige tada sa chábán
options-atc = Rialú tráchta aeir
    .facts =
        Glaonna raidió an túir, na gcóngaracha agus GCA ag aerpháirceanna cairdiúla
        Cead éirí de thalamh, overhead break agus talkdown i scamall íseal nó drochfheiceálacht
options-pixel-block = Méid bloic an phixealaithe
    .facts = Taobh bloic, i bpixil scáileáin (2: mín, 5: garbh)
effect-pixelize = Pixealú
    .facts =
        An radharc 3D ar fad (tír-raon, aerárthaigh, foirgnimh, cábán) i bpixil mhóra chearnacha mar ar ríomhaire MS-DOS; fanann na roghchláir agus na leideanna géar (ní i VR)
        Socraíonn an sleamhnán méid bloic, i bpixil scáileáin (2: mín, 5: garbh)
        Tá an frith-ailiasú as fad atá sé air
        Fágann na trí lasc faoi bhun é an HUD, an cábán nó na MFDanna, na hionstraimí agus an Kneeboard géar
effect-sharp-hud = Gan HUD
    .facts =
        Pixealú: Tarraingítear an HUD (agus scáileán an chlogaid) géar thar an radharc pixealaithe, ionas go bhfanann a shiombailí tanaí inléite
effect-sharp-cockpit = Gan cábán an phíolóta (cnaipí, lasca...)
    .facts =
        Pixealú: Tarraingítear painéal, fráma, suíochán, cnaipí, lasca, maide stiúrtha agus luamhán sháiteach cábáin an phíolóta go géar
effect-sharp-gauges = Gan MFDanna, ionstraimí ná Kneeboard
    .facts =
        Pixealú: Tarraingítear na MFDanna, an liathróid dhearcaidh, na hionstraimí, na lampaí, na léamha, na lipéid agus an Kneeboard go géar
options-needs-pixelize = Teastaíonn Pixealú
