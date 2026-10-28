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
options-cockpit = Cockpit
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
options-section-vr-size = Méid an chockpit i VR
options-vr-size =
    .facts =
        Méid an chockpit (agus an domhain) sa cheannsraith
        100%: méid iarbhír

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
options-section-cockpit = Cockpit
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

options-section-effects = Effects
options-section-retro = Retro
effect-shake = Camera shake
    .facts =
        The view shakes with the gun, nearby blasts, stall buffet, touchdowns and the runway
        VR: the cockpit vibrates, your head stays still
effect-flashes = Combat flashes
    .facts =
        Flak bursts, explosions and missile launches light the clouds, the ground and the cockpit for a moment
effect-barrage = Night barrage
    .facts =
        Conventional war at night: the towns' guns fire tracer blindly into the sky, searchlights sweep over them
        Only for the look: the barrage hits nothing, the searchlights find nothing
effect-cockpit-night = Cockpit night lighting
    .facts =
        A dim red flood light in the cockpit after dark (blue-green under the goggles)
        The master caution pulses amber over the panel while a warning is on
effect-nvg = Night vision goggles
    .facts =
        Bright lights bloom into halos, the picture grains more in the dark
        Desktop: the goggles' round field of view
effect-snow = Snow
    .facts =
        Above the freezing level storms snow instead of rain: low in a northern winter, high in summer
effect-mist = Ground mist
    .facts =
        Modern shaders: mist lies over the low ground and the sea at dawn, dusk and night
        Thin in the deserts, none in storms
effect-cloud-shadows = Cloud shadows
    .facts =
        Modern shaders: the clouds' shadows on the ground below
effect-vapour = Vapour and downwash
    .facts =
        Pulling hard, vapour trails from the wingtips
        A helicopter hovering low whips up dust, or spray over water
effect-night-sky = Night sky
    .facts =
        Shooting stars now and then, the stars low over the horizon twinkle
        The northern lights over the North Cape
effect-vga = VGA colours
    .facts =
        The 1989 look with modern shading: 64 steps a colour channel, as a VGA card's, dithered
effect-scanlines = Scanlines
    .facts =
        A CRT monitor's scanlines (not in VR)

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
