# Das OPTIONEN-Fenster (src/options.rs) und seine Seiten im Flugmenü.

options-title = OPTIONEN
options-tab-game = SPIEL
options-tab-graphics = GRAFIK
options-tab-sound = TON
options-tab-controls = STEUERUNG
options-footer = Für die nächsten Flüge gespeichert; die Realismus-Schalter werden mit jedem Spiel gewählt

## SPIEL

options-section-language = Sprache
options-section-classic-modern = Klassisch / Modern
options-cockpit = Cockpit
    .facts =
        Modern: hochauflösende Displays mit Randtasten
        Klassisch: das Instrumentenbrett der DOS-Zeit und 77x60-Displays (F-117A)
options-hud = HUD
    .facts =
        Modern: hochauflösendes grünes HUD im F-16-Stil mit NAV-, AA- und AG-Modus
        Klassisch: das HUD der DOS-Zeit
options-avionics = Avionik
    .facts =
        Modern: MFD-Formate im F-16-Stil auf jedem MFD (Lagebild, HARM, Zielbehälter, Karte, Außenlasten) mit beweglichem Cursor
        Klassisch: Karte und TAC links, Kamera, WAY, WPN und ORD rechts
options-flight-model = Flugmodell
    .facts =
        Modern: aerodynamisches Flugmodell mit Fly-by-Wire: der Stick steuert Nick- und Rollrate; losgelassen bleibt die Nase stehen und die Flugbahn wird gehalten
        Klassisch: ein Flugmodell der DOS-Zeit
options-section-flying = Fliegen
options-auto-trim = Auto-Trimmung
    .facts =
        Bei losgelassenem Stick hält die Trimmung die Lage, in der Sie sie gelassen haben
        Hubschrauber: die Nase steigt nicht mehr mit der Fahrt; Jets: trimmt beschädigte Ruder aus
        Manuelle Trimmung: Alt + Pfeiltasten, Alt + Pos1 zentriert sie
options-tooltips = Cockpit-Tooltips
    .facts = Was ein Bedienelement oder Display des Cockpits ist, unter der Maus oder einer VR-Fingerspitze
options-section-vr-size = VR-Cockpitgröße
options-vr-size =
    .facts =
        Größe des Cockpits (und der Welt) im Headset
        100 %: Originalgröße

## GRAFIK

options-section-rendering = Darstellung
options-shaders = Shader
    .facts =
        Modern: dezente gerichtete Schattierung, weicher Entfernungsdunst
        Klassisch: flache Schattierung
options-shadows = Schatten
    .facts =
        Modern: Sonnenschatten im Cockpit und in der Welt (kostet GPU-Zeit)
        Klassisch: keine Schatten
options-point-lights = Punktlichter
    .facts =
        Modern: Cockpit- und Displaybeleuchtung; Explosionen, Rauchfahnen und Pistenlichter beleuchten die Szene
        Klassisch: nur Sonnen- und Himmelslicht
options-time-of-day = Tageszeit
    .facts =
        Modern: Sonne und Mond folgen der Missionsuhr, der Himmel wird mit der Sonne dunkler und heller; nachts Sterne und Mondphase
        Modern: das Mondlicht bestimmt, wie weit feindliche Piloten Sie nachts sehen (2 km bei Neumond, 4 km bei Vollmond)
        Klassisch: ein langsamer Farbwechsel und eine feste Sonne
options-msaa = Kantenglättung (MSAA)
    .facts =
        Glatte Kanten: 4x MSAA, 2x in VR (kostet GPU-Zeit)
        Aus: harte Pixelkanten
options-auto-detail = Automatische Szenerie-Details (VR)
    .facts =
        Im Headset schrumpft die Reichweite von Städten, Wäldern und Feldern, solange Bilder zu spät kommen, und wächst wieder, wenn Zeit übrig ist
        Die Grafikeinstellungen bleiben wie gewählt
options-sun-moon = Sonne und Mond
    .facts =
        Tagsüber die Sonne am Himmel, nachts Mond und Sterne
        Der Mond zeigt die Phase der Mission

## TON

options-section-volume = Lautstärke
options-volume-master = Gesamt
    .facts = Alle Töne
options-volume-engine = Triebwerk
    .facts = Ihr Triebwerk oder Rotor und die Flugzeuge um Sie herum
options-volume-effects = Effekte
    .facts = Waffen, Explosionen, Fahrwerk, Funk und der Rest
options-volume-warnings = Warnungen
    .facts = Überziehwarnung, Radar- und Raketenwarnungen, Feuersirene, Sidewinder-Töne
options-section-cockpit = Cockpit
options-low-pass = Triebwerks-Tiefpass
    .facts =
        Im Flugzeug klingt das Triebwerk gedämpft (Haube und Helm nehmen die Höhen weg)
        Aus: wie draußen

## STEUERUNG

options-section-bindings = Belegungen
options-keys-joystick = TASTEN UND JOYSTICK
    .facts =
        Tasten, Joystick- und HOTAS-Achsen und -Knöpfe belegen
        Für die nächsten Flüge gespeichert
options-section-head-tracking = Kopf-/Augen-Tracking
options-head-tracking = Kopf-/Augen-Tracking
    .facts =
        Ihr Kopf dreht und bewegt die Sicht im Cockpit (am Bildschirm; in VR macht das das Headset)
        Tracking-Programm (OpenTrack...): FreeTrack-Ausgabe oder UDP an 127.0.0.1 Port 4242
        Pos1: die aktuelle Kopfposition wird zur Mitte
options-tracking = Tracking ({ $source })
options-no-tracker = Kein Tracker empfangen: Starten Sie das Tracking-Programm

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

lang-name-en = Englisch
lang-name-bg = Bulgarisch
lang-name-zh-Hans = Chinesisch (vereinfacht)
lang-name-hr = Kroatisch
lang-name-cs = Tschechisch
lang-name-da = Dänisch
lang-name-nl = Niederländisch
lang-name-et = Estnisch
lang-name-fi = Finnisch
lang-name-fr = Französisch
lang-name-de = Deutsch
lang-name-el = Griechisch
lang-name-hu = Ungarisch
lang-name-id = Indonesisch
lang-name-ga = Irisch
lang-name-it = Italienisch
lang-name-ja = Japanisch
lang-name-ko = Koreanisch
lang-name-lv = Lettisch
lang-name-lt = Litauisch
lang-name-mt = Maltesisch
lang-name-pl = Polnisch
lang-name-pt-BR = Portugiesisch (Brasilien)
lang-name-ro = Rumänisch
lang-name-ru = Russisch
lang-name-sk = Slowakisch
lang-name-sl = Slowenisch
lang-name-es = Spanisch
lang-name-sv = Schwedisch
lang-name-uk = Ukrainisch

## Browser build: offer to play in the browser's language (src/i18n.rs)

language-offer-question = Browsersprache: { $language }. Das Spiel in dieser Sprache spielen?
language-offer-yes = Ja
language-offer-no = Nein
language-offer-never = Nicht mehr fragen
effect-sunbeams = Sunbeams
    .facts =
        Shafts of sunlight through the gaps of a broken overcast
