# Okno NASTAVENÍ (src/options.rs) a jeho stránky v nabídce za letu.

options-title = NASTAVENÍ
options-tab-game = HRA
options-tab-graphics = GRAFIKA
options-tab-sound = ZVUK
options-tab-controls = OVLÁDÁNÍ
options-footer = Uloženo pro další lety; přepínače realismu se volí při každé hře

## HRA

options-section-language = Jazyk
options-section-classic-modern = Klasický / Moderní
options-cockpit = Kokpit
    .facts =
        Moderní: displeje ve vysokém rozlišení s tlačítky na rámečku
        Klasický: rozložení přístrojové desky z éry DOSu a displeje 77x60 (F-117A)
options-hud = HUD
    .facts =
        Moderní: zelený HUD ve vysokém rozlišení ve stylu F-16 s režimy NAV, AA a AG
        Klasický: HUD z éry DOSu
options-avionics = Avionika
    .facts =
        Moderní: stránky MFD ve stylu F-16 na obou MFD (situace, HARM, zaměřovací kontejner, mapa, výzbroj) s pohyblivým kurzorem
        Klasický: mapa a TAC vlevo, kamera, WAY, WPN a ORD vpravo
options-flight-model = Letový model
    .facts =
        Moderní: aerodynamický letový model s elektrickým řízením: páka zadává rychlost klopení a klonění; po uvolnění zůstane příď na místě a dráha letu se drží
        Klasický: letový model z éry DOSu
options-section-flying = Pilotování
options-auto-trim = Automatický trim
    .facts =
        Při uvolněné páce trim drží polohu, ve které jste ji nechali
        Vrtulníky: příď už nestoupá s rychlostí; stíhačky: vyvažuje poškozená kormidla
        Ruční trim: Alt + šipky, Alt + Home jej vystředí
options-keep-time-accel = Zachovat zrychlení času
    .facts =
        Zrychlení se již nevypne, když vás odhalí, vypustíte zbraň nebo vás sestřelí
        Změní ho jen tlačítko TIME a Shift + Z / Shift + X
options-tooltips = Tooltipy kokpitu
    .facts = Co je ovladač nebo displej kokpitu, pod myší nebo špičkou prstu ve VR
options-player-labels = Jména hráčů
    .facts = Hra více hráčů: jména ostatních pilotů nad jejich letadly, na jakoukoli vzdálenost
options-label-size =
    .facts =
        Velikost jmen hráčů
        50 %: výchozí
options-seat-height = Výška sedadla
    .facts =
        Jak vysoko sedíte v kokpitu, na počítači i ve VR
        0 cm: běžná výška očí v letounu
options-section-vr-size = Velikost kokpitu ve VR
options-vr-size =
    .facts =
        Velikost kokpitu (a světa) v headsetu
        100 %: skutečná velikost
options-section-gif = Nahrávání GIF
options-gif-size = VELIKOST
options-gif-rate = SNÍMKY
options-gif-scale-100 =
    .facts = Plné rozlišení okna: velké soubory
options-gif-scale-25 =
    .facts = Malé soubory
options-gif-rate-50 =
    .facts = Jen když hra běží na 100 fps a víc (zachycen je každý druhý snímek)
options-gif-hint = F12 (pokud není přemapována) spouští a zastavuje; uloženo ve složce Obrázky

## GRAFIKA

options-section-rendering = Vykreslování
options-shaders = Shadery
    .facts =
        Moderní: jemné směrové stínování, plynulá dálková mlha
        Klasický: ploché stínování
options-shadows = Stíny
    .facts =
        Moderní: sluneční stíny v kokpitu i ve světě (zatěžuje GPU)
        Klasický: bez stínů
options-point-lights = Bodová světla
    .facts =
        Moderní: osvětlení kokpitu a displejů; scénu osvětlují výbuchy, plameny a dráhová světla
        Klasický: pouze světlo slunce a oblohy
options-time-of-day = Denní doba
    .facts =
        Moderní: slunce a měsíc se pohybují podle hodin mise, obloha tmavne a světlá se se sluncem; v noci hvězdy a fáze měsíce
        Moderní: měsíční svit určuje, jak daleko vás v noci vidí nepřátelští piloti (2 km za bezměsíčné noci, 4 km za úplňku)
        Klasický: pomalá změna barev a pevné slunce
options-msaa = Vyhlazování hran (MSAA)
    .facts =
        Hladké hrany: 4x MSAA, ve VR 2x (zatěžuje GPU)
        Vypnuto: ostré hrany pixelů
options-auto-detail = Automatický detail krajiny (VR)
    .facts =
        V headsetu se dohled měst, lesů a polí zkracuje, když snímky zaostávají, a vrací se, když je čas nazbyt
        Nastavení grafiky zůstává, jak jste je zvolili
options-sun-moon = Slunce a měsíc
    .facts =
        Ve dne slunce na obloze, v noci měsíc a hvězdy
        Měsíc ukazuje fázi podle mise
options-cloud-quality = Mraky
cloud-quality-auto = Auto
    .facts = Nízká ve VR, vysoká na obrazovce
cloud-quality-low = Nízká
    .facts = Hrubé mraky v třetinovém rozlišení (nejrychlejší)
cloud-quality-medium = Střední
    .facts = Poloviční rozlišení, jemnější detaily
cloud-quality-high = Vysoká
    .facts = Plné rozlišení, nejjemnější detaily a vlastní stín (stojí čas GPU)

## ZVUK

options-section-volume = Hlasitost
options-volume-master = Celková
    .facts = Všechny zvuky
options-volume-engine = Motor
    .facts = Váš motor nebo rotor a letouny slyšené kolem vás
options-volume-effects = Efekty
    .facts = Zbraně, výbuchy, podvozek, rádio a vše ostatní
options-volume-warnings = Varování
    .facts = Houkačka přetažení, varování radaru a střel, siréna požáru, tóny Sidewinderu
options-volume-rwr = RWR
    .facts = Tóny radarového varování, jako část hlasitosti varování
options-section-rwr-tones = Tóny radarového varování
rwr-tones-all = VŠECHNY
    .facts = Radar, který vás ozařuje, každé přehledové otočení, které vás odhalí, odpaly střel
rwr-tones-detect = JEN ODHALENÍ
    .facts =
        Jeden tón, když vás radar odhalí (znovu, když vás radary na chvíli ztratí)
        A odpaly střel
rwr-tones-launch = JEN ODPALY
    .facts = Jen střely odpálené na vás
rwr-tones-off = VYP
    .facts = Žádné tóny radarového varování (kontrolky a displeje stále varují)
options-section-cockpit = Kokpit
options-low-pass = Dolní propust motoru
    .facts =
        Uvnitř letounu zní motor tlumeně (kabina a helma ubírají výšky)
        Vypnuto: stejně jako venku

## OVLÁDÁNÍ

options-section-bindings = Přiřazení
options-keys-joystick = KLÁVESY A JOYSTICK
    .facts =
        Klávesy mapy, osy a tlačítka joysticku a HOTAS
        Uloženo pro další lety
options-section-head-tracking = Sledování hlavy / očí
options-head-tracking = Sledování hlavy / očí
    .facts =
        Hlava otáčí a posouvá pohled v kokpitu (na PC; ve VR to dělá headset)
        Program pro sledování (OpenTrack...): výstup FreeTrack nebo UDP na 127.0.0.1 port 4242
        Home: aktuální poloha hlavy se stane středem
options-tracking = Sledování ({ $source })
options-no-tracker = Žádný tracker neodpovídá: spusťte program pro sledování

## EFFECTS

options-section-effects = Efekty
options-section-retro = Retro
effect-shake = Otřesy kamery
    .facts =
        Pohled se třese při střelbě z kanónu, blízkých výbuších, přetažení, dosednutí a na dráze
        VR: kabina vibruje, hlava zůstává v klidu
effect-flashes = Záblesky boje
    .facts = Výbuchy flaku, exploze a starty raket na okamžik osvětlí mraky, zem a kabinu
effect-barrage = Noční zátarasová palba
    .facts =
        Konvenční válka v noci: děla měst střílejí naslepo světelné střely do nebe, reflektory přes ně přejíždějí
        Nepřátelské drony nad městem chytí jeho reflektory a přitáhnou palbu jeho děl
        Jen pro efekt: palba nic nezasáhne
effect-cockpit-night = Noční osvětlení kabiny
    .facts =
        Slabé červené světlo v kabině po setmění (modrozelené pod brýlemi)
        Master caution pulzuje jantarově nad panelem, dokud trvá varování
effect-nvg = Brýle pro noční vidění
    .facts =
        Jasná světla se rozlévají v svatozáře, obraz je v tmě zrnitější
        Desktop: kulaté zorné pole brýlí
effect-snow = Sníh
    .facts = Nad hladinou mrazu bouřky přinášejí sníh místo deště: nízko v severské zimě, vysoko v létě
effect-mist = Přízemní mlha
    .facts =
        Moderní shadery: mlha leží nad nížinou a mořem za úsvitu, soumraku a v noci
        Řídká v pouštích, žádná za bouře
effect-cloud-shadows = Stíny mraků
    .facts = Moderní shadery: stíny mraků na zemi pod nimi
effect-vapour = Pára a proud od rotoru
    .facts =
        Při ostrých manévrech stopy páry z konců křídel
        Nízko visící vrtulník zvedá prach, nad vodou tříšť
        Jedoucí vozidla za sebou zvedají prach
effect-night-sky = Noční obloha
    .facts =
        Čas od času padající hvězdy, hvězdy nízko nad obzorem se třpytí
        Polární záře nad Nordkappem
effect-vga = Barvy VGA
    .facts = Vzhled z roku 1989 s moderním stínováním: 64 stupňů na barevný kanál jako u karty VGA, s ditheringem
effect-scanlines = Rastrové řádky
    .facts = Řádky CRT monitoru (ne ve VR)

## LANGUAGE NAMES

lang-name-en = angličtina
lang-name-bg = bulharština
lang-name-zh-Hans = čínština (zjednodušená)
lang-name-hr = chorvatština
lang-name-cs = čeština
lang-name-da = dánština
lang-name-nl = nizozemština
lang-name-et = estonština
lang-name-fi = finština
lang-name-fr = francouzština
lang-name-de = němčina
lang-name-el = řečtina
lang-name-hu = maďarština
lang-name-id = indonéština
lang-name-ga = irština
lang-name-it = italština
lang-name-ja = japonština
lang-name-ko = korejština
lang-name-lv = lotyština
lang-name-lt = litevština
lang-name-mt = maltština
lang-name-pl = polština
lang-name-pt-BR = portugalština (Brazílie)
lang-name-ro = rumunština
lang-name-ru = ruština
lang-name-sk = slovenština
lang-name-sl = slovinština
lang-name-es = španělština
lang-name-sv = švédština
lang-name-uk = ukrajinština

## Browser build: offer to play in the browser's language (src/i18n.rs)

language-offer-question = Jazyk prohlížeče: { $language }. Chcete hrát v tomto jazyce?
language-offer-yes = Ano
language-offer-no = Ne
language-offer-never = Znovu se neptat
effect-sunbeams = Sluneční paprsky
    .facts = Svazky slunečního světla skrz mezery v rozbité oblačnosti
options-msaa-forced = Vypnuto, dokud je pixelizace (retro) zapnutá
options-volume-radio = Rádio
    .facts = Ostatní hráči mluvící v koop rádiu a mluvený ATC
options-volume-atc = Hlas ATC
    .facts = Mluvené výzvy ATC, jako součást hlasitosti rádia
options-volume-music = Hudba
    .facts = Hudba hlavní nabídky
options-section-atc = Mluvený ATC
options-section-radio = Koop rádio
options-push-to-talk = Stiskni a mluv
    .facts =
        Podržte klávesu pro mluvení (Caps Lock nebo tlačítko joysticku či VR přiřazené k PTT) a všichni hráči vás slyší
        Mikrofon se otevře při prvním stisku
options-mic-gain = ZISK MIKR.
    .facts =
        Jak hlasitě váš mikrofon vysílá
        100 %: beze změny
options-mic-default = Výchozí
options-voice-text-only = jen text
options-voices-line = Dispečeři: { $controllers }   Pilot: { $pilot }
options-voices = HLASY
    .facts =
        Stáhnout hlasy pro rádiové výzvy, smazat je
        Vybrat hlasy dispečerů a vlastní a poslechnout si je
options-voices-web = V prohlížeči jen text
options-tracking-mouse = Pohled myší při trackingu
    .facts =
        Zap.: pravé tlačítko myši stále otáčí a přibližuje pohled z kabiny
        Vyp.: dokud tracker vysílá, pravé tlačítko myši v kabině nic nedělá
options-atc = Řízení letového provozu
    .facts =
        Rádiové výzvy věže, přiblížení a GCA na spojeneckých letištích
        Povolení ke vzletu, overhead break a talkdown při nízké oblačnosti či špatné viditelnosti
options-pixel-block = Velikost bloku pixelizace
    .facts = Strana bloku v pixelech obrazovky (2: jemný, 5: hrubý)
effect-pixelize = Pixelizace
    .facts =
        Celý 3D pohled (terén, letadla, budovy, kabina) ve velkých čtvercových pixelech jako na PC s MS-DOS; nabídky a popisky zůstávají ostré (ne ve VR)
        Posuvník nastavuje velikost bloku v pixelech obrazovky (2: jemný, 5: hrubý)
        Vyhlazování hran je vypnuto, dokud je zapnutá
        Tři přepínače pod ní nechají HUD, kokpit nebo MFD, přístroje a Kneeboard ostré
effect-sharp-hud = Bez HUD
    .facts =
        Pixelizace: HUD (i přilbový displej) se kreslí ostře přes pixelizovaný obraz, aby jeho tenké symboly zůstaly čitelné
effect-sharp-cockpit = Bez kokpitu (tlačítka, přepínače...)
    .facts =
        Pixelizace: panel, rám, sedadlo, tlačítka, přepínače, řídicí páka a plynová páka kokpitu se kreslí ostře
effect-sharp-gauges = Bez MFD, přístrojů a kneeboardu
    .facts =
        Pixelizace: MFD, umělý horizont, přístroje, kontrolky, údaje, popisky a Kneeboard se kreslí ostře
options-needs-pixelize = Vyžaduje Pixelizaci
