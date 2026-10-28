# Okno MOŽNOSTI (src/options.rs) a jeho stránky v ponuke za letu.

options-title = MOŽNOSTI
options-tab-game = HRA
options-tab-graphics = GRAFIKA
options-tab-sound = ZVUK
options-tab-controls = OVLÁDANIE
options-footer = Uložené pre ďalšie lety; prepínače realizmu sa volia pri každej hre

## HRA

options-section-language = Jazyk
options-section-classic-modern = Klasický / Moderný
options-cockpit = Kokpit
    .facts =
        Moderný: displeje s vysokým rozlíšením a tlačidlami po obvode
        Klasický: rozloženie prístrojovej dosky z čias DOSu a displeje 77x60 (F-117A)
options-hud = HUD
    .facts =
        Moderný: zelený HUD s vysokým rozlíšením v štýle F-16 s režimami NAV, AA a AG
        Klasický: HUD z čias DOSu
options-avionics = Avionika
    .facts =
        Moderný: formáty MFD v štýle F-16 na ktoromkoľvek MFD (situačný displej, HARM, zameriavacia gondola, mapa, výzbroj) s posuvným kurzorom
        Klasický: mapa a TAC vľavo, kamera, WAY, WPN a ORD vpravo
options-flight-model = Model letu
    .facts =
        Moderný: aerodynamický model letu s fly-by-wire: stick riadi rýchlosť klopenia a klonenia; po uvoľnení nos zostane na mieste a dráha letu sa drží
        Klasický: model letu z čias DOSu
options-section-flying = Pilotovanie
options-auto-trim = Automatický trim
    .facts =
        S uvoľneným stickom trim drží polohu, v akej ste ho nechali
        Vrtuľníky: nos sa už nedvíha s rýchlosťou; prúdové lietadlá: vytrimuje poškodené kormidlá
        Ručný trim: Alt + šípky, Alt + Home ho vycentruje
options-tooltips = Tooltipy v kokpite
    .facts = Čo je ovládací prvok alebo displej kokpitu, pod myšou alebo prstom vo VR
options-section-vr-size = Veľkosť kokpitu vo VR
options-vr-size =
    .facts =
        Veľkosť kokpitu (a sveta) v headsete
        100 %: v životnej veľkosti

## GRAFIKA

options-section-rendering = Vykresľovanie
options-shaders = Shadery
    .facts =
        Moderný: jemné smerové tieňovanie, plynulá hmla vo vzdialenosti
        Klasický: ploché tieňovanie
options-shadows = Tiene
    .facts =
        Moderný: tiene od slnka v kokpite a vo svete (stojí výkon GPU)
        Klasický: bez tieňov
options-point-lights = Bodové svetlá
    .facts =
        Moderný: osvetlenie kokpitu a displejov; výbuchy, dymové chvosty a svetlá dráhy osvetľujú scénu
        Klasický: iba svetlo slnka a oblohy
options-time-of-day = Denná doba
    .facts =
        Moderný: slnko a mesiac sa pohybujú podľa hodín misie, obloha tmavne a rozjasňuje sa so slnkom; v noci hviezdy a fáza mesiaca
        Moderný: mesačné svetlo určuje, odkiaľ vás v noci vidia nepriateľskí piloti (2 km bez mesiaca, 4 km za splnu)
        Klasický: pomalá zmena farieb a pevné slnko
options-msaa = Vyhladzovanie hrán (MSAA)
    .facts =
        Hladké hrany: 4x MSAA, vo VR 2x (stojí výkon GPU)
        Vyp: tvrdé pixelové hrany
options-auto-detail = Automatický detail krajiny (VR)
    .facts =
        V headsete sa dosah miest, lesov a polí zmenšuje, kým snímky meškajú, a vracia sa, keď je čas nazvyš
        Nastavenia grafiky zostávajú, ako ste ich zvolili
options-sun-moon = Slnko a mesiac
    .facts =
        Slnko na oblohe cez deň, mesiac a hviezdy v noci
        Mesiac ukazuje fázu v čase misie

## ZVUK

options-section-volume = Hlasitosť
options-volume-master = Celková
    .facts = Všetky zvuky
options-volume-engine = Motor
    .facts = Váš motor alebo rotor a lietadlá počuteľné vo vašom okolí
options-volume-effects = Efekty
    .facts = Zbrane, výbuchy, podvozok, rádio a zvyšok
options-volume-warnings = Výstrahy
    .facts = Húkačka pádu, výstrahy radaru a rakiet, požiarna siréna, tóny Sidewindera
options-section-cockpit = Kokpit
options-low-pass = Dolná prepusť motora
    .facts =
        Vo vnútri lietadla znie motor tlmene (kabína a prilba odfiltrujú výšky)
        Vyp: rovnako ako vonku

## OVLÁDANIE

options-section-bindings = Priradenia
options-keys-joystick = KLÁVESY A JOYSTICK
    .facts =
        Mapovanie klávesov, osí a tlačidiel joysticku a HOTAS
        Uložené pre ďalšie lety
options-section-head-tracking = Sledovanie hlavy/očí
options-head-tracking = Sledovanie hlavy/očí
    .facts =
        Hlava otáča a posúva pohľad v kokpite (desktop; vo VR to robí headset)
        Program na sledovanie (OpenTrack...): výstup FreeTrack alebo UDP na 127.0.0.1 port 4242
        Home: aktuálna poloha hlavy sa stane stredom
options-tracking = Sledovanie ({ $source })
options-no-tracker = Žiadny tracker nebol zachytený: spustite program na sledovanie
