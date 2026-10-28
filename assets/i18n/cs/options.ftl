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
options-tooltips = Tooltipy kokpitu
    .facts = Co je ovladač nebo displej kokpitu, pod myší nebo špičkou prstu ve VR
options-section-vr-size = Velikost kokpitu ve VR
options-vr-size =
    .facts =
        Velikost kokpitu (a světa) v headsetu
        100 %: skutečná velikost

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
