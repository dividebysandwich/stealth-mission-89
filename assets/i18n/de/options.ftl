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
options-player-labels = Spielernamen
    .facts = Mehrspieler: die Namen der anderen Piloten über ihren Flugzeugen, egal wie weit entfernt
options-label-size =
    .facts =
        Größe der Spielernamen
        50 %: Standard
options-section-vr-size = VR-Cockpitgröße
options-vr-size =
    .facts =
        Größe des Cockpits (und der Welt) im Headset
        100 %: Originalgröße
options-section-gif = GIF-Aufnahme
options-gif-size = GRÖSSE
options-gif-rate = RATE
options-gif-scale-100 =
    .facts = Die volle Auflösung des Fensters: große Dateien
options-gif-scale-25 =
    .facts = Kleine Dateien
options-gif-rate-50 =
    .facts = Nur wenn das Spiel mit 100 fps oder mehr läuft (jedes zweite Bild wird aufgenommen)
options-gif-hint = F12 (sofern nicht neu belegt) startet und stoppt; gespeichert in Ihrem Bilder-Ordner

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
options-cloud-quality = Wolken
cloud-quality-auto = Auto
    .facts = Niedrig in VR, hoch am Bildschirm
cloud-quality-low = Niedrig
    .facts = Grobe Wolken in einem Drittel der Auflösung (am schnellsten)
cloud-quality-medium = Mittel
    .facts = Halbe Auflösung, feinere Details
cloud-quality-high = Hoch
    .facts = Volle Auflösung, feinste Details und Eigenschatten (kostet GPU-Zeit)

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
options-volume-rwr = RWR
    .facts = Die Radarwarntöne, als Anteil an der Lautstärke der Warnungen
options-section-rwr-tones = Radarwarntöne
rwr-tones-all = ALLE
    .facts = Ein Radar, das Sie erfasst, jeder Suchstrahl, der Sie ortet, Raketenstarts
rwr-tones-detect = NUR ORTUNG
    .facts =
        Ein Ton, wenn ein Radar Sie ortet (erneut, nachdem die Radare Sie eine Weile verloren haben)
        Und Raketenstarts
rwr-tones-launch = NUR STARTS
    .facts = Nur auf Sie abgefeuerte Raketen
rwr-tones-off = AUS
    .facts = Keine Radarwarntöne (Leuchten und Displays warnen weiterhin)
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

options-section-effects = Effekte
options-section-retro = Retro
effect-shake = Kamerawackeln
    .facts =
        Die Ansicht wackelt mit der Kanone, nahen Explosionen, Strömungsabriss, Landungen und der Piste
        VR: das Cockpit vibriert, der Kopf bleibt ruhig
effect-flashes = Kampfblitze
    .facts = Flak-Sprengpunkte, Explosionen und Raketenstarts erhellen Wolken, Boden und Cockpit kurz
effect-barrage = Nächtliches Sperrfeuer
    .facts =
        Konventioneller Krieg bei Nacht: die Geschütze der Städte feuern Leuchtspur blind in den Himmel, Suchscheinwerfer streichen darüber
        Feindliche Drohnen über einer Stadt werden von ihren Scheinwerfern erfasst und ziehen das Feuer ihrer Geschütze an
        Nur fürs Auge: die Sperre trifft nichts
effect-cockpit-night = Cockpitbeleuchtung bei Nacht
    .facts =
        Ein schwaches rotes Flutlicht im Cockpit nach Einbruch der Dunkelheit (blaugrün unter dem Nachtsichtgerät)
        Die Master Caution pulsiert bernsteinfarben über der Tafel, solange eine Warnung anliegt
effect-nvg = Nachtsichtgerät
    .facts =
        Helle Lichter blühen zu Lichthöfen auf, das Bild körnt im Dunkeln stärker
        Desktop: das runde Sichtfeld des Geräts
effect-snow = Schnee
    .facts = Über der Nullgradgrenze schneit es bei Gewittern statt zu regnen: tief im nördlichen Winter, hoch im Sommer
effect-mist = Bodennebel
    .facts =
        Moderne Shader: Nebel liegt bei Morgen- und Abenddämmerung und nachts über dem Tiefland und dem Meer
        Dünn in den Wüsten, keiner bei Sturm
effect-cloud-shadows = Wolkenschatten
    .facts = Moderne Shader: die Schatten der Wolken auf dem Boden darunter
effect-vapour = Kondensstreifen und Abwind
    .facts =
        Bei harten Manövern Kondensstreifen an den Flügelspitzen
        Ein tief schwebender Hubschrauber wirbelt Staub auf, über Wasser Gischt
        Fahrende Fahrzeuge ziehen eine Staubfahne hinter sich her
effect-night-sky = Nachthimmel
    .facts =
        Ab und zu Sternschnuppen, die Sterne tief über dem Horizont funkeln
        Das Nordlicht über dem Nordkap
effect-vga = VGA-Farben
    .facts = Der Look von 1989 mit moderner Schattierung: 64 Stufen pro Farbkanal wie bei einer VGA-Karte, gerastert
effect-scanlines = Scanlines
    .facts = Die Zeilen eines Röhrenmonitors (nicht in VR)

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
effect-sunbeams = Sonnenstrahlen
    .facts = Lichtbündel der Sonne durch die Lücken einer aufgelockerten Wolkendecke
options-msaa-forced = Aus, solange Pixelierung (Retro) an ist
options-volume-radio = Funk
    .facts = Die anderen Spieler im Koop-Funk und der gesprochene ATC
options-volume-atc = ATC-Stimme
    .facts = Die gesprochenen ATC-Aufrufe, als Teil der Funklautstärke
options-volume-music = Musik
    .facts = Die Musik des Hauptmenüs
options-section-atc = Gesprochener ATC
options-section-radio = Koop-Funk
options-push-to-talk = Push-to-Talk
    .facts =
        Sprechtaste halten (Feststelltaste, oder eine Joystick- oder VR-Taste, die auf PTT gelegt ist) und alle Spieler hören Sie
        Das Mikrofon öffnet sich beim ersten Drücken
options-mic-gain = MIKRO-PEGEL
    .facts =
        Wie laut Ihr Mikrofon gesendet wird
        100 %: unverändert
options-mic-default = Standard
options-voice-text-only = nur Text
options-voices-line = Lotsen: { $controllers }   Pilot: { $pilot }
options-voices = STIMMEN
    .facts =
        Stimmen für die Funkrufe herunterladen, löschen
        Die der Lotsen und die eigene wählen und anhören
options-voices-web = Im Browser nur Text
options-tracking-mouse = Mausblick beim Tracking
    .facts =
        Ein: die rechte Maustaste dreht und zoomt die Cockpitansicht weiterhin
        Aus: solange der Tracker sendet, tut die rechte Maustaste im Cockpit nichts
options-atc = Flugverkehrskontrolle
    .facts =
        Funkrufe von Turm, Anflug und GCA auf den eigenen Flugplätzen
        Starterlaubnis, Overhead-Break und ein Talkdown bei tiefen Wolken oder schlechter Sicht
options-pixel-block = Blockgröße der Pixelierung
    .facts = Die Seitenlänge eines Blocks in Bildschirmpixeln (2: fein, 5: grob)
effect-pixelize = Pixelierung
    .facts =
        Die ganze 3D-Ansicht (Gelände, Flugzeuge, Gebäude, Cockpit) in großen quadratischen Pixeln, wie auf einem MS-DOS-PC; die Menüs und Tooltips bleiben scharf (nicht in VR)
        Der Regler stellt die Größe eines Blocks in Bildschirmpixeln ein (2: fein, 5: grob)
        Die Kantenglättung ist aus, solange sie an ist
