# Hauptmenü und Einzelmissionsbildschirm (src/menu.rs).

## Schauplätze

theater-libya = Libyen (1986)
    .facts =
        Golf von Syrte und libysche Küste
        Zeit der Operation El Dorado Canyon
        Meist klarer Wüstenhimmel
theater-persian_gulf = Persischer Golf (1984)
    .facts =
        Zeit des Tankerkriegs zwischen Iran und Irak
        Über Wasser und Küstenwüste
        Das klarste Wetter aller Schauplätze
theater-north_cape = Nordkap
    .facts =
        Arktisches Norwegen und Halbinsel Kola
        Heimat der sowjetischen Nordflotte
        Häufig tiefe geschlossene Bewölkung
theater-central_europe = Mitteleuropa
    .facts =
        NATO gegen Warschauer Pakt über Deutschland
        Nur 15 % der Zeit klarer Himmel
theater-vietnam = Vietnam
    .facts =
        Raum Hanoi und Haiphong
        Dschungel und Gebirge
        Gewitter in 30 % der Missionen
theater-middle_east = Naher Osten
    .facts =
        Östliches Mittelmeer und Levante
        Meist klarer Himmel
theater-desert_storm = Desert Storm
    .facts =
        Kuwait und Irak, 1991
        Der echte Kampfeinsatz der F-117A
        Meist klarer Wüstenhimmel
theater-cuba = Kuba
    .facts =
        Inselschauplatz in der Karibik
        Ziele auf See und an Land
        Tropisch: Gewitter in 15 % der Missionen
theater-korea = Korea
    .facts =
        Koreanische Halbinsel nördlich der DMZ
        Gebirgiges Gelände
        Wechselhaftes Wetter, jeder Himmel ist möglich
theater-ukraine = Ukraine (2026)
    .facts =
        Ostukraine, Krim und Südrussland
        Die Front brennt von Charkiw bis Cherson
        Offenes Ackerland, zerstörte Städte an der Front

## Lagen (Einsatzregeln)

roe-0 = Kalter Krieg
    .facts =
        Verdeckte Operation im Frieden
        Unentdeckt bleiben ist das Wichtigste
        Weniger feindliche Flugzeuge auf Patrouille
        Die genauen Einsatzregeln stehen im Briefing
roe-1 = Begrenzter Krieg
    .facts =
        Regionaler bewaffneter Konflikt
        Feindliche Luftabwehr in Alarmbereitschaft
        Mehr feindliche Flugzeuge als im Kalten Krieg
roe-2 = Konventioneller Krieg
    .facts =
        Krieg in vollem Umfang
        Alle feindlichen Kräfte aktiv
        Mehr feindliche Flugzeuge als im Kalten Krieg

## Missionsarten

kind-strike = Angriff
    .facts =
        Ein Bodenziel zerstören, fotografieren oder erreichen
        Primär- und Sekundärziel
        Luft-Boden-Bewaffnung
kind-air = Luft-Luft
    .facts =
        Ein feindliches Flugzeug abfangen
        Transporter, Bomber oder Jägerpatrouillen
        Luft-Luft-Raketen
kind-strike-training = Angriffsübung
    .facts =
        Übungs-Angriffsmission
        Waffen und Navigation lernen
kind-air-training = Luftkampfübung
    .facts =
        Übungs-Luft-Luft-Mission
        Abfangen und Raketen lernen
kind-drone-defense = Drohnenabwehr
    .facts =
        Nur Ukraine
        Eine Welle von Geran-Kampfdrohnen aufhalten, bevor sie eine Stadt oder ein Kernkraftwerk trifft
        Danach ihren Startplatz bombardieren

## Gegner

opponents-0 = Unerfahren
    .facts =
        Unerfahrene feindliche Besatzungen
        Leichteste Stufe
opponents-1 = Normal
    .facts =
        Durchschnittliche feindliche Besatzungen
        Standardstufe
opponents-2 = Veteran
    .facts =
        Erfahrene feindliche Besatzungen
        Mehr Schiffe und Patrouillen
opponents-3 = Elite
    .facts =
        Die besten feindlichen Piloten und SAM-Besatzungen
        Schwerste Stufe

## Wetter

sky-random = Zufällig
    .facts =
        Aus dem Klima des Schauplatzes gewürfelt
        Ändert sich mit jeder neuen Mission
        Wolken versperren beiden Seiten die Sicht
sky-clear = Klar
    .facts =
        Keine Wolken
        Sicht 40 km
        Der klassische Himmel der DOS-Zeit
sky-scattered = Aufgelockert
    .facts =
        Cumulus 20-45 %, Basis 2500-6000 ft
        Manchmal dünner Altostratus weit oben
        Sicht 30 km
sky-broken = Durchbrochen
    .facts =
        Cumulus 45-70 %, Basis 2000-5000 ft
        Oft eine Stratocumulusschicht darüber
        Sicht 25 km
sky-overcast = Bedeckt
    .facts =
        Stratus 80-100 %, Basis 1500-5000 ft
        Einige Cumulusfetzen darunter
        Sicht 20 km
sky-storm = Gewitter
    .facts =
        Cumulonimbustürme bis 40000 ft
        Tiefer Stratus und Cumulus ab 1000 ft
        Sicht 8 km
menu-random-cover = Zufällige Bedeckung
    .facts =
        Bedeckung der Hauptwolkenschicht mit dem Wetter gewürfelt
        Abwählen, um sie selbst einzustellen
        Hauptschicht: Stratus bei Bedeckt und Gewitter, sonst Cumulus
menu-cover-slider =
    .facts =
        Anteil des Himmels, den die Hauptschicht bedeckt
        Ohne Wirkung bei klarem Himmel

## Mond

moon-date = Missionsdatum
    .facts =
        Der echte Mond des Briefingdatums
        Benötigt den Tageszeit-Schalter (modern)
moon-0 = Neumond
    .facts =
        Kein Mondlicht: schwarze Nacht, nur Lichter und Sterne
        Feindliche Piloten sehen Sie auf 2 km
moon-1 = Zunehmende Sichel
    .facts = Eine schmale Mondsichel am Abendhimmel, vor Mitternacht untergegangen
moon-2 = Erstes Viertel
    .facts = Halb beleuchtet, abends sichtbar, geht gegen Mitternacht unter
moon-3 = Zunehmender Mond
    .facts = Hell, vom Nachmittag bis nach Mitternacht sichtbar
moon-4 = Vollmond
    .facts =
        Die ganze Nacht sichtbar, um Mitternacht hoch
        Feindliche Piloten sehen Sie auf 4 km
moon-5 = Abnehmender Mond
    .facts = Hell, geht abends auf, bis zum Morgen sichtbar
moon-6 = Letztes Viertel
    .facts = Halb beleuchtet, geht gegen Mitternacht auf
moon-7 = Abnehmende Sichel
    .facts = Eine schmale Sichel, die vor der Dämmerung aufgeht

## Flugzeuge

aircraft-f117 =
    .facts =
        Tarnkappen-Angriffsflugzeug
        Unterschall, 7 g
        Modernes und klassisches Cockpit
aircraft-f16 =
    .facts =
        Mehrzweckjäger
        Mach-2-Klasse, 9 g, schnelle Rolle
        F-16-Cockpit mit RWR
aircraft-tornado =
    .facts =
        Angriffsflugzeug für den Tiefflug
        Schwenkflügel, die sich mit der Geschwindigkeit verstellen
        Modernes Cockpit, drei MFDs
aircraft-a10 =
    .facts =
        Flugzeug zur Luftnahunterstützung
        Langsam und robust, 30-mm-Kanone
        A-10C-Cockpit, zwei MFDs
aircraft-uh60 =
    .facts =
        Transporthubschrauber
        Setzt Truppen ab: nahe am Ziel landen
        Zyklische und kollektive Steuerung
aircraft-ah1z =
    .facts =
        Kampfhubschrauber
        Turmkanone, Hellfire, AIM-9
        Kanone und Hellfire folgen dem Pod

## Abschnitte und Gruppen

section-theater = Schauplatz
section-opponents = Gegner
section-situation = Lage
section-mission = Mission
section-moon = Mond
section-start-time = Startzeit
section-weather = Wetter
section-realism = Realismus
section-tacview = Tacview
section-mission-summary = Missionsübersicht
section-briefing = Briefing
group-mission = Mission
group-conditions = Bedingungen
group-vehicle = Fluggerät
group-simulation = Simulation

## Optionen der Mission

menu-start-random = Zufällig
    .facts =
        Mit jeder neuen Mission gewürfelt, wie im klassischen Modus
        06:00-08:00 bei Tag oder 18:00-20:00 bei Nacht
        Abwählen, um die Uhrzeit zu wählen
menu-start-slider =
    .facts =
        Startzeit
        Nacht von 18:00 bis 06:00
        Bleibt bei NEUE MISSION erhalten
menu-realistic-weapons = Realistische Bewaffnung
    .facts =
        Die Grenzen des echten Flugzeugs
        2 Waffenschächte statt 4
        Keine AIM-9- oder AIM-120-Raketen
        Keine Starts und Landungen auf Flugzeugträgern
menu-crash-protection = Absturzschutz
    .facts =
        Zieht in Bodennähe automatisch hoch (Fahrwerk eingefahren)
        Bodenberührung und Kollisionen sind nicht tödlich
        Aus: Der Boden kann immer getroffen werden
menu-tacview-export = Export
    .facts =
        Speichert jeden Flug als Tacview-Aufzeichnung (.acmi)
        Im Ordner des Spiels für Ihre Dateien, unter tacview
menu-tacview-real-time = Echtzeit-Telemetrie
    .facts =
        Sendet den Flug während des Fliegens an Tacview
        In Tacview: Record > Real-Time Telemetry
        Adresse 127.0.0.1, Port 42674 (--tacview-address)
switch-game = SPIEL
switch-real = REAL
menu-damage = Schaden
    .facts =
        Waffenschaden
        Spiel: klassischer Schaden; Treffer summieren sich, bringen das Flugzeug aber nie zum Absturz
        Realistisch: ein Raketentreffer kann das Flugzeug zerstören (Aussteigen!), beschädigt mehr Systeme und löst oft einen Brand aus, der erlischt oder das Flugzeug sprengt
        Realistisch: beschädigte Ruder verstimmen die Trimmung oder lassen das Flugzeug schwingen
menu-guidance = Lenkung
    .facts =
        Raketenlenkung
        Klassisch: Raketen fliegen gerade auf ihr Ziel zu
        Realistisch: Proportionalnavigation zum Kollisionspunkt; die Motoren brennen aus und die Raketen gleiten weiter und verlieren bei jeder Kurve Fahrt (Rauch nur bei brennendem Motor)
        Realistisch: SAMs steigen gegen ferne Ziele hoch; die AIM-120 fliegt 3 s geradeaus (hochziehen, um sie zu werfen)
menu-air-defence = Luftabwehr
    .facts =
        Tiefflieger-Luftabwehr
        Klassisch: nur die SAM-Stellungen und Jäger
        Realistisch: radargelenkte ZSU-23-4 (ihr Radar sieht Sie wie das einer SAM) und S-60-Flakbatterien schützen Flugplätze und Ziele; MANPADS-Trupps in den Städten feuern SA-14 auf alles, was sie sehen
        Tiefflug über verteidigten Orten wird gefährlich; Bomben und Bordkanone zerstören die Geschütze

## Hauptbildschirm

menu-title = STEALTH MISSION 89
menu-subtitle = MISSIONSPLANUNGSSYSTEM
menu-single-mission = EINZELMISSION
    .facts =
        Eine Mission auf dem Schauplatz Ihrer Wahl
        Optionen wählen, Route und Bewaffnung planen
menu-career = KARRIERE
    .facts =
        Pilotenakten: Dienstgrad, Auszeichnungen, Einsätze
        Dienstzeit: 12 Einsätze auf einem Schauplatz
        Was Sie zerstören, bleibt zerstört
menu-campaign = DYNAMISCHE KAMPAGNE
    .facts =
        Dynamische Kampagne in Echtzeit
        Erobern Sie die Städte und Flugplätze des Feindes
        Missionen aus der aktuellen Lage
menu-multiplayer = MEHRSPIELER
    .facts = Koop in der dynamischen Kampagne
    .join = Einem Server der öffentlichen Liste beitreten
    .host = Einem Server beitreten oder eine Kampagne hosten und darin fliegen
menu-options = OPTIONEN
    .facts =
        Spiel: Stil von Cockpit, HUD und Avionik, Auto-Trimmung
        Grafik, Lautstärken, Tasten und Joystick
        Für die nächsten Flüge gespeichert
    .short = Spiel, Grafik, Ton und Steuerung
menu-download = HERUNTERLADEN
    .facts =
        Das Desktop-Spiel für Windows, Linux und macOS
        Mit VR und Mehrspieler
        Steam Deck und Steam Frame: das Linux-AppImage
web-play = IM BROWSER SPIELEN
    .facts = Gleich hier, ohne Installation
web-download-platforms = Windows, Linux und macOS, Steam Deck und Steam Frame
web-download-version = Die Download-Version
web-gives-vr = VR
    .why = Fliegen Sie in VR mit jedem OpenXR-kompatiblen Headset und dem Steam Frame
web-gives-hotas = HOTAS
    .why = Volle Unterstützung für Gamepads, Joysticks, Schubhebel und Pedale
web-gives-multiplayer = Besserer Mehrspieler
    .why = Direkte UDP-Verbindung zu den Servern für weniger Lag
web-gives-server = Eigenen Server betreiben
    .why = Betreiben Sie eine Koop-Kampagne oder einen dedizierten Server
web-gives-head-tracking = Head Tracking
    .why = Tracker im Stil von TrackIR über OpenTrack
web-gives-tacview = Tacview
    .why = Flugaufzeichnungen und Telemetrie in Echtzeit
web-gives-keep = Für immer Ihres
    .why = Fällt die Website aus, behalten Sie Ihre eigene Kopie
menu-discord = DISCORD
    .facts =
        Gemeinsam fliegen, Fehler melden und Missionen teilen
        Die Entwicklung verfolgen
menu-quit = BEENDEN
    .facts = Zurück zum Desktop
menu-version = VERSION { $version }

## Einzelmissionsbildschirm

menu-career-sortie = KARRIERE-EINSATZ
menu-tour-sortie = { $pilot }  //  EINSATZ { $sortie } VON { $length } DER DIENSTZEIT
menu-msn = MISSIONSPLANUNGSSYSTEM  //  MSN { $seed }
menu-no-targets = Für diese Dienstzeit stehen keine Ziele mehr
menu-plan-route = ROUTE PLANEN
    .facts =
        Die Karte des Schauplatzes mit der Abdeckung feindlicher Radare und SAMs
        Navigationspunkte hinzufügen, verschieben und entfernen
        Ziele und Landebasis bleiben fest
menu-loadout = BEWAFFNUNG
    .facts =
        Die Außenlasten jedes Schachts oder Pylons wählen
        Beginnt mit den Waffen des Briefings
menu-generating = WIRD ERZEUGT...
menu-back-records = Zurück zu den Pilotenakten
menu-back-main = Zurück zum Hauptmenü
menu-new-sortie = NEUER EINSATZ
menu-new-mission = NEUE MISSION
    .facts =
        Würfelt andere Ziele
        Behält alle Optionen und die Startzeit
menu-load = LADEN
    .facts =
        Im Flug gespeicherte Missionen (Menü: MISSION SPEICHERN)
        Dort weiterfliegen, wo gespeichert wurde
menu-fly = FLIEGEN
    .facts = Zur Mission des Briefings starten
menu-fly-vr = IN VR FLIEGEN
    .facts =
        Startet das Spiel im VR-Headset neu
        Fliegt genau diese Mission
    .no-mission = Erzeugen Sie zuerst eine Mission
    .no-runtime = Keine OpenXR-Laufzeit (Monado, WiVRn, SteamVR) aktiv
menu-vr-failed = VR-Start nicht möglich: { $error }

## Missionsübersicht

summary-theater = SCHAUPLATZ
summary-situation = LAGE
summary-mission = MISSION
summary-opponents = GEGNER
summary-time = ZEIT
summary-weather = WETTER
summary-fuel = TREIBSTOFF
summary-night = Nacht
summary-day = Tag
summary-time-value = { $daytime }  Start { $depart }
summary-weather-value = { $sky }  Sicht { $vis } km
summary-fuel-value = { $fuel } lbs
summary-armament = Bewaffnung: { $stores }
summary-intelligence = AUFKLÄRUNG
