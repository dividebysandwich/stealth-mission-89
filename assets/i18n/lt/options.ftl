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
options-section-vr-size = VR kabinos dydis
options-vr-size =
    .facts =
        Kabinos (ir pasaulio) dydis ausinėje
        100 %: natūralus dydis

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
