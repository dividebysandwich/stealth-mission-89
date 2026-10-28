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
