# NUSTATYMŲ langas (src/options.rs) ir jo puslapiai meniu skrydžio metu.

options-title = NUSTATYMAI
options-tab-game = ŽAIDIMAS
options-tab-graphics = GRAFIKA
options-tab-sound = GARSAS
options-tab-controls = VALDYMAS
options-footer = Išsaugoma kitiems skrydžiams; realizmo jungikliai pasirenkami kiekvienam žaidimui

## ŽAIDIMAS

options-section-language = Kalba
options-section-classic-modern = Klasikinis / moderni
options-cockpit = Kabina
    .facts =
        Moderni: didelės raiškos ekranai su apvadų mygtukais
        Klasikinė: DOS eros prietaisų skydelio išdėstymas ir 77x60 ekranai (F-117A)
options-hud = HUD
    .facts =
        Moderni: didelės raiškos F-16 stiliaus žalias HUD su NAV, AA ir AG režimais
        Klasikinė: DOS eros HUD
options-avionics = Avionika
    .facts =
        Moderni: F-16 stiliaus MFD formatai bet kuriame MFD (situacijos ekranas, HARM, žymėjimo konteineris, žemėlapis, krovinys) su slenkamu žymekliu
        Klasikinė: žemėlapis ir TAC kairėje, kamera, WAY, WPN ir ORD dešinėje
options-flight-model = Skrydžio modelis
    .facts =
        Moderni: aerodinaminis skrydžio modelis su fly-by-wire: stick nurodo posvyrio ir polėkio greitį; paleidus nosis lieka vietoje ir išlaikoma skrydžio trajektorija
        Klasikinė: DOS eros skrydžio modelis
options-section-flying = Skrydis
options-auto-trim = Autom. trim
    .facts =
        Paleidus stick, trim išlaiko padėtį, kurioje ją palikote
        Sraigtasparniai: nosis nebekyla didėjant greičiui; reaktyviniai: kompensuoja pažeistus valdymo paviršius
        Rankinis trim: Alt + rodyklių klavišai, Alt + Home centruoja
options-tooltips = Kabinos paaiškinimai
    .facts = Kas yra kabinos valdiklis ar ekranas, po pele arba VR piršto galiuku
options-player-labels = Žaidėjų vardai
    .facts = Daugelio žaidėjų režimas: kitų pilotų vardai virš jų lėktuvų, bet kokiu atstumu
options-label-size =
    .facts =
        Žaidėjų vardų dydis
        50 %: numatytasis
options-section-vr-size = VR kabinos dydis
options-vr-size =
    .facts =
        Kabinos (ir pasaulio) dydis ausinėje
        100 %: natūralus dydis
options-section-gif = GIF įrašymas
options-gif-size = DYDIS
options-gif-rate = KADRAI
options-gif-scale-100 =
    .facts = Visa lango raiška: dideli failai
options-gif-scale-25 =
    .facts = Maži failai
options-gif-rate-50 =
    .facts = Tik kai žaidimas veikia 100 fps ar daugiau (įrašomas kas antras kadras)
options-gif-hint = F12 (jei nepriskirta kitaip) pradeda ir sustabdo; įrašoma į aplanką Paveikslėliai

## GRAFIKA

options-section-rendering = Vaizdavimas
options-shaders = Šeideriai
    .facts =
        Moderni: subtilus kryptinis šešėliavimas, švelnus atstumo rūkas
        Klasikinė: plokščias šešėliavimas
options-shadows = Šešėliai
    .facts =
        Moderni: saulės šešėliai kabinoje ir pasaulyje (naudoja GPU laiką)
        Klasikinė: be šešėlių
options-point-lights = Taškiniai šviestuvai
    .facts =
        Moderni: kabinos ir ekranų apšvietimas; sprogimai, dūmų kamienai ir pakilimo tako žibintai apšviečia sceną
        Klasikinė: tik saulės ir dangaus šviesa
options-time-of-day = Paros laikas
    .facts =
        Moderni: saulė ir Mėnulis juda su misijos laikrodžiu, dangus tamsėja ir šviesėja su saule; naktį žvaigždės ir Mėnulio fazė
        Moderni: mėnesiena nustato, iš kokio atstumo priešo lakūnai jus mato naktį (2 km bemėnulę naktį, 4 km per pilnatį)
        Klasikinė: lėtas spalvų kitimas ir nejudanti saulė
options-msaa = Glotninimas (MSAA)
    .facts =
        Glotnūs kraštai: 4x MSAA, 2x VR (naudoja GPU laiką)
        Išjungta: aštrūs pikselių kraštai
options-auto-detail = Automatinis aplinkos detalumas (VR)
    .facts =
        Ausinėje miestų, miškų ir laukų matomumo atstumas mažėja, kai kadrai vėluoja, ir didėja, kai yra laiko atsargos
        Grafikos nustatymai lieka tokie, kokius pasirinkote
options-sun-moon = Saulė ir Mėnulis
    .facts =
        Saulė danguje dieną, Mėnulis ir žvaigždės naktį
        Mėnulis rodo misijos fazę

## GARSAS

options-section-volume = Garsas
options-volume-master = Bendras
    .facts = Visi garsai
options-volume-engine = Variklis
    .facts = Jūsų variklis ar rotorius ir aplink girdimi orlaiviai
options-volume-effects = Efektai
    .facts = Ginklai, sprogimai, važiuoklė, radijas ir kita
options-volume-warnings = Įspėjimai
    .facts = Praradimo signalas, radaro ir raketų įspėjimai, gaisro sirena, Sidewinder tonai
options-volume-rwr = RWR
    .facts = Radaro įspėjimo tonai, kaip įspėjimų garsumo dalis
options-section-rwr-tones = Radaro įspėjimo tonai
rwr-tones-all = VISI
    .facts = Jus apšviečiantis radaras, kiekvienas jus aptinkantis apžvalgos ciklas, raketų paleidimai
rwr-tones-detect = TIK APTIKIMAS
    .facts =
        Vienas tonas, kai radaras jus aptinka (vėl, kai radarai jus kuriam laikui paleidžia)
        Ir raketų paleidimai
rwr-tones-launch = TIK PALEIDIMAI
    .facts = Tik į jus paleistos raketos
rwr-tones-off = IŠJ
    .facts = Jokių radaro įspėjimo tonų (lemputės ir ekranai vis tiek įspėja)
options-section-cockpit = Kabina
options-low-pass = Variklio žemų dažnių filtras
    .facts =
        Orlaivio viduje variklis skamba duslinamai (gaubtas ir šalmas pašalina aukštus dažnius)
        Išjungta: kaip lauke

## VALDYMAS

options-section-bindings = Priskyrimai
options-keys-joystick = KLAVIŠAI IR JOYSTICK
    .facts =
        Žemėlapio klavišai, joystick ir HOTAS ašys bei mygtukai
        Išsaugoma kitiems skrydžiams
options-section-head-tracking = Galvos / akių sekimas
options-head-tracking = Galvos / akių sekimas
    .facts =
        Jūsų galva sukioja ir juda vaizdą kabinoje (kompiuteryje; VR tai daro ausinė)
        Sekimo programa (OpenTrack...): FreeTrack išvestis arba UDP į 127.0.0.1 prievadą 4242
        Home: dabartinė galvos padėtis tampa centru
options-tracking = Sekimas ({ $source })
options-no-tracker = Sekiklis negirdimas: paleiskite sekimo programą

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
effect-vapour = Garai ir rotoriaus srautas
    .facts =
        Staigiai manevruojant nuo sparnų galų driekiasi garų pėdsakai
        Žemai kabantis sraigtasparnis kelia dulkes, virš vandens - purslus
        Važiuojančios mašinos kelia dulkes už savęs
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

lang-name-en = anglų
lang-name-bg = bulgarų
lang-name-zh-Hans = kinų (supaprastinta)
lang-name-hr = kroatų
lang-name-cs = čekų
lang-name-da = danų
lang-name-nl = olandų
lang-name-et = estų
lang-name-fi = suomių
lang-name-fr = prancūzų
lang-name-de = vokiečių
lang-name-el = graikų
lang-name-hu = vengrų
lang-name-id = indoneziečių
lang-name-ga = airių
lang-name-it = italų
lang-name-ja = japonų
lang-name-ko = korėjiečių
lang-name-lv = latvių
lang-name-lt = lietuvių
lang-name-mt = maltiečių
lang-name-pl = lenkų
lang-name-pt-BR = portugalų (Brazilija)
lang-name-ro = rumunų
lang-name-ru = rusų
lang-name-sk = slovakų
lang-name-sl = slovėnų
lang-name-es = ispanų
lang-name-sv = švedų
lang-name-uk = ukrainiečių

## Browser build: offer to play in the browser's language (src/i18n.rs)

language-offer-question = Naršyklės kalba: { $language }. Ar žaisti šia kalba?
language-offer-yes = Taip
language-offer-no = Ne
language-offer-never = Daugiau nebeklausti
effect-sunbeams = Sunbeams
    .facts =
        Shafts of sunlight through the gaps of a broken overcast
