# SEADETE aken (src/options.rs) ja selle lehed lennusisese menüüs.

options-title = SEADED
options-tab-game = MÄNG
options-tab-graphics = GRAAFIKA
options-tab-sound = HELI
options-tab-controls = JUHTIMINE
options-footer = Salvestatakse järgmisteks lendudeks; realismilülitid valitakse iga mänguga

## MÄNG

options-section-language = Keel
options-section-classic-modern = Klassikaline / kaasaegne
options-cockpit = Cockpit
    .facts =
        Kaasaegne: kõrglahutusega ekraanid raami nuppudega
        Klassikaline: DOS-ajastu instrumendipaneeli paigutus ja 77x60 ekraanid (F-117A)
options-hud = HUD
    .facts =
        Kaasaegne: kõrglahutusega F-16 stiilis roheline HUD režiimidega NAV, AA ja AG
        Klassikaline: DOS-ajastu HUD
options-avionics = Avioonika
    .facts =
        Kaasaegne: F-16 stiilis MFD formaadid kummalgi MFD-l (olukorra ekraan, HARM, targeting pod, kaart, stores) liigutatava kursoriga
        Klassikaline: kaart ja TAC vasakul, kaamera, WAY, WPN ja ORD paremal
options-flight-model = Lennumudel
    .facts =
        Kaasaegne: aerodünaamiline lennumudel fly-by-wire'iga: stick määrab tõusu- ja rullikiiruse; lahti lastuna jääb nina paigale ja lennutrajektoor säilib
        Klassikaline: DOS-ajastu lennumudel
options-section-flying = Lendamine
options-auto-trim = Auto trim
    .facts =
        Kui stick on lahti, hoiab trim lennuasendit seal, kus selle jätsid
        Kopterid: nina ei tõuse enam kiirusega; jetid: trimmib kahjustatud juhtpinnad välja
        Käsitsi trim: Alt + nooleklahvid, Alt + Home tsentreerib
options-tooltips = Cockpiti tooltips
    .facts = Mis on cockpiti juhtelement või ekraan, hiire all või VR-i sõrmeotsas
options-section-vr-size = VR-cockpiti suurus
options-vr-size =
    .facts =
        Cockpiti (ja maailma) suurus peakomplektis
        100%: elusuurus

## GRAAFIKA

options-section-rendering = Renderdus
options-shaders = Shaders
    .facts =
        Kaasaegne: õrn suunatud varjutus, sujuv kauguse uduhõõre
        Klassikaline: lame varjutus
options-shadows = Varjud
    .facts =
        Kaasaegne: päikesevarjud cockpitis ja maailmas (koormab GPU-d)
        Klassikaline: varje pole
options-point-lights = Punktvalgustid
    .facts =
        Kaasaegne: cockpiti ja ekraanide valgustus; plahvatused, suitsusabad ja rajatuled valgustavad stseeni
        Klassikaline: ainult päikese- ja taevavalgus
options-time-of-day = Kellaaeg
    .facts =
        Kaasaegne: päike ja kuu liiguvad missiooni kellaga, taevas tumeneb ja heleneb päikesega; öösel tähed ja kuufaas
        Kaasaegne: kuuvalgus määrab, kui kaugele vaenlase piloodid sind öösel näevad (2 km kuuta ööl, 4 km täiskuu all)
        Klassikaline: aeglane värvide muutus ja fikseeritud päike
options-msaa = Anti-aliasing (MSAA)
    .facts =
        Siledad servad: 4x MSAA, VR-is 2x (koormab GPU-d)
        Väljas: teravad piksliservad
options-auto-detail = Automaatne maastiku detailsus (VR)
    .facts =
        Peakomplektis kahaneb linnade, metsade ja põldude kuvamiskaugus, kui kaadrid hilinevad, ja kasvab tagasi, kui aega jätkub
        Graafikaseaded jäävad valituks
options-sun-moon = Päike ja kuu
    .facts =
        Päike taevas päeval, kuu ja tähed öösel
        Kuu näitab missiooni faasi

## HELI

options-section-volume = Helitugevus
options-volume-master = Üldine
    .facts = Kõik helid
options-volume-engine = Mootor
    .facts = Sinu mootor või rootor ja sinu ümber kuuldavad lennukid
options-volume-effects = Efektid
    .facts = Relvad, plahvatused, telik, raadio ja ülejäänu
options-volume-warnings = Hoiatused
    .facts = Kukkumishoiatus, radari ja raketi hoiatused, tulesireen, Sidewinderi toonid
options-section-cockpit = Cockpit
options-low-pass = Mootori madalpääs
    .facts =
        Lennuki sees kõlab mootor summutatult (kupu ja kiiver võtavad kõrged toonid ära)
        Väljas: sama mis väljas

## JUHTIMINE

options-section-bindings = Sidumised
options-keys-joystick = KLAHVID JA JOYSTICK
    .facts =
        Kaardi klahvid, joystick'i ja HOTAS-i teljed ja nupud
        Salvestatakse järgmisteks lendudeks
options-section-head-tracking = Pea/silmade jälgimine
options-head-tracking = Pea/silmade jälgimine
    .facts =
        Sinu pea pöörab ja liigutab vaadet cockpitis (töölaual; VR-is teeb seda peakomplekt)
        Jälgimisprogramm (OpenTrack...): FreeTracki väljund või UDP aadressile 127.0.0.1 port 4242
        Home: praegune pea asend saab keskpunktiks
options-tracking = Jälgimine ({ $source })
options-no-tracker = Jälgijat ei kuule: käivita jälgimisprogramm
