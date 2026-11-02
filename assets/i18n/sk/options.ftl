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
options-player-labels = Mená hráčov
    .facts = Hra viacerých hráčov: mená ostatných pilotov nad ich lietadlami, na akúkoľvek vzdialenosť
options-label-size =
    .facts =
        Veľkosť mien hráčov
        50 %: predvolená
options-section-vr-size = Veľkosť kokpitu vo VR
options-vr-size =
    .facts =
        Veľkosť kokpitu (a sveta) v headsete
        100 %: v životnej veľkosti
options-section-gif = Nahrávanie GIF
options-gif-size = VEĽKOSŤ
options-gif-rate = SNÍMKY
options-gif-scale-100 =
    .facts = Plné rozlíšenie okna: veľké súbory
options-gif-scale-25 =
    .facts = Malé súbory
options-gif-rate-50 =
    .facts = Len keď hra beží na 100 fps a viac (zachytený je každý druhý snímok)
options-gif-hint = F12 (ak nie je premapovaná) spúšťa a zastavuje; uložené v priečinku Obrázky

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
options-cloud-quality = Oblaky
cloud-quality-auto = Auto
    .facts = Nízka vo VR, vysoká na obrazovke
cloud-quality-low = Nízka
    .facts = Hrubé oblaky v tretinovom rozlíšení (najrýchlejšie)
cloud-quality-medium = Stredná
    .facts = Polovičné rozlíšenie, jemnejšie detaily
cloud-quality-high = Vysoká
    .facts = Plné rozlíšenie, najjemnejšie detaily a vlastný tieň (stojí čas GPU)

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
options-volume-rwr = RWR
    .facts = Tóny radarovej výstrahy, ako časť hlasitosti výstrah
options-section-rwr-tones = Tóny radarovej výstrahy
rwr-tones-all = VŠETKY
    .facts = Radar, ktorý vás ožaruje, každé otočenie, ktoré vás odhalí, odpaly rakiet
rwr-tones-detect = LEN ODHALENIE
    .facts =
        Jeden tón, keď vás radar odhalí (znova, keď vás radary na chvíľu stratia)
        A odpaly rakiet
rwr-tones-launch = LEN ODPALY
    .facts = Len rakety odpálené na vás
rwr-tones-off = VYP
    .facts = Žiadne tóny radarovej výstrahy (kontrolky a displeje stále varujú)
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

## EFFECTS

options-section-effects = Efekty
options-section-retro = Retro
effect-shake = Otrasy kamery
    .facts =
        Pohľad sa trasie pri streľbe z kanóna, blízkych výbuchoch, preťahovaní, dosadnutiach a na dráhe
        VR: kabína vibruje, hlava zostáva v pokoji
effect-flashes = Záblesky boja
    .facts = Výbuchy flaku, explózie a štarty rakiet na okamih osvetlia mraky, zem a kabínu
effect-barrage = Nočná zátarasová paľba
    .facts =
        Konvenčná vojna v noci: delá miest strieľajú naslepo svietiace strely do neba, reflektory cez ne prechádzajú
        Nepriateľské drony nad mestom chytia jeho reflektory a pritiahnu paľbu jeho diel
        Len pre efekt: paľba nič nezasiahne
effect-cockpit-night = Nočné osvetlenie kabíny
    .facts =
        Slabé červené svetlo v kabíne po zotmení (modrozelené pod okuliarmi)
        Master caution pulzuje jantárovo nad panelom, kým trvá varovanie
effect-nvg = Okuliare nočného videnia
    .facts =
        Jasné svetlá sa rozlievajú do svätožiar, obraz je v tme zrnitejší
        Desktop: okrúhle zorné pole okuliarov
effect-snow = Sneh
    .facts = Nad hladinou mrazu búrky prinášajú sneh namiesto dažďa: nízko v severskej zime, vysoko v lete
effect-mist = Prízemná hmla
    .facts =
        Moderné shadery: hmla leží nad nížinou a morom za svitania, súmraku a v noci
        Riedka v púšťach, žiadna pri búrke
effect-cloud-shadows = Tiene mrakov
    .facts = Moderné shadery: tiene mrakov na zemi pod nimi
effect-vapour = Para a prúd od rotora
    .facts =
        Pri ostrých manévroch stopy pary z koncov krídel
        Nízko visiaci vrtuľník dvíha prach, nad vodou triešť
        Idúce vozidlá za sebou dvíhajú prach
effect-night-sky = Nočná obloha
    .facts =
        Z času na čas padajúce hviezdy, hviezdy nízko nad obzorom sa trblietajú
        Polárna žiara nad Nordkappom
effect-vga = Farby VGA
    .facts = Vzhľad z roku 1989 s moderným tieňovaním: 64 stupňov na farebný kanál ako pri karte VGA, s ditheringom
effect-scanlines = Rastrové riadky
    .facts = Riadky CRT monitora (nie vo VR)

## LANGUAGE NAMES

lang-name-en = angličtina
lang-name-bg = bulharčina
lang-name-zh-Hans = čínština (zjednodušená)
lang-name-hr = chorvátčina
lang-name-cs = čeština
lang-name-da = dánčina
lang-name-nl = holandčina
lang-name-et = estónčina
lang-name-fi = fínčina
lang-name-fr = francúzština
lang-name-de = nemčina
lang-name-el = gréčtina
lang-name-hu = maďarčina
lang-name-id = indonézština
lang-name-ga = írčina
lang-name-it = taliančina
lang-name-ja = japončina
lang-name-ko = kórejčina
lang-name-lv = lotyština
lang-name-lt = litovčina
lang-name-mt = maltčina
lang-name-pl = poľština
lang-name-pt-BR = portugalčina (Brazília)
lang-name-ro = rumunčina
lang-name-ru = ruština
lang-name-sk = slovenčina
lang-name-sl = slovinčina
lang-name-es = španielčina
lang-name-sv = švédčina
lang-name-uk = ukrajinčina

## Browser build: offer to play in the browser's language (src/i18n.rs)

language-offer-question = Jazyk prehliadača: { $language }. Chcete hrať v tomto jazyku?
language-offer-yes = Áno
language-offer-no = Nie
language-offer-never = Viac sa nepýtať
effect-sunbeams = Slnečné lúče
    .facts = Zväzky slnečného svetla cez medzery v rozbitej oblačnosti
options-msaa-forced = Vypnuté, kým je pixelizácia (retro) zapnutá
options-volume-radio = Rádio
    .facts = Ostatní hráči hovoriaci v koop rádiu a hovorený ATC
options-volume-atc = Hlas ATC
    .facts = Hovorené výzvy ATC, ako súčasť hlasitosti rádia
options-volume-music = Hudba
    .facts = Hudba hlavnej ponuky
options-section-atc = Hovorený ATC
options-section-radio = Koop rádio
options-push-to-talk = Stlač a hovor
    .facts =
        Podržte kláves na hovorenie (Caps Lock alebo tlačidlo joysticku či VR priradené k PTT) a všetci hráči vás počujú
        Mikrofón sa otvorí pri prvom stlačení
options-mic-gain = ZISK MIKR.
    .facts =
        Ako hlasno váš mikrofón vysiela
        100 %: bez zmeny
options-mic-default = Predvolené
options-voice-text-only = iba text
options-voices-line = Dispečeri: { $controllers }   Pilot: { $pilot }
options-voices = HLASY
    .facts =
        Stiahnuť hlasy pre rádiové výzvy, vymazať ich
        Vybrať hlasy dispečerov a vlastný a vypočuť si ich
options-voices-web = V prehliadači iba text
options-tracking-mouse = Pohľad myšou pri trackingu
    .facts =
        Zap.: pravé tlačidlo myši stále otáča a približuje pohľad z kabíny
        Vyp.: kým tracker vysiela, pravé tlačidlo myši v kabíne nič nerobí
options-atc = Riadenie letovej prevádzky
    .facts =
        Rádiové výzvy veže, priblíženia a GCA na spojeneckých letiskách
        Povolenie na vzlet, overhead break a talkdown pri nízkej oblačnosti alebo zlej viditeľnosti
options-pixel-block = Veľkosť bloku pixelizácie
    .facts = Strana bloku v pixeloch obrazovky (2: jemný, 5: hrubý)
effect-pixelize = Pixelizácia
    .facts =
        Celý 3D pohľad (terén, lietadlá, budovy, kabína) vo veľkých štvorcových pixeloch ako na PC s MS-DOS; ponuky a popisky zostávajú ostré (nie vo VR)
        Posuvník nastavuje veľkosť bloku v pixeloch obrazovky (2: jemný, 5: hrubý)
        Vyhladzovanie hrán je vypnuté, kým je zapnutá
