# Der Einsatzbildschirm der Kampagne (src/campaign/ops.rs).

ops-thunderstorms = GEWITTER
ops-wx-short = WETTER { $now }, { $next } UM { $at }
ops-wx-line = { $weather } Wechsel zu { $next } um { $at }.
ops-clock = TAG { $day }  { $time }
ops-landed-at = GELANDET IN { $base }
ops-ground-crew = Die Bodenmannschaft ist bereit
ops-rearm = AUFMUNITIONIEREN UND TANKEN
ops-end-sortie = EINSATZ BEENDEN
ops-on-server = AUF { $server }
ops-serving = SERVER
ops-tab-missions = MISSIONEN
ops-tab-map = KRIEGSKARTE
ops-tab-air-defence = LUFTABWEHR
ops-tab-forces = KRÄFTE
ops-tab-players = SPIELER
ops-tab-debrief = NACHBESPRECHUNG
ops-tab-log = LOGBUCH
ops-tab-chat = CHAT
ops-blue = BLAU { $count }
ops-red = ROT { $count }
ops-war-over = Der Krieg ist vorbei
ops-running = Kriegssimulation läuft
ops-save-exit = SPEICHERN UND BEENDEN
ops-disconnect = TRENNEN
ops-leave-local = SPEICHERN UND DIE KAMPAGNE VERLASSEN?
    .text = Der Krieg hält an, bis Sie ihn fortsetzen. Flugzeuge in der Luft gehen verloren; laufende Luftlandungen beginnen wieder an ihrer Basis.
ops-leave-host = SPEICHERN UND DAS HOSTEN DER KAMPAGNE BEENDEN?
    .text = Die beigetretenen Spieler werden getrennt und der Krieg hält an, bis Sie ihn fortsetzen. Flugzeuge in der Luft gehen verloren; laufende Luftlandungen beginnen wieder an ihrer Basis.
ops-leave-client = VOM SERVER TRENNEN?
    .text = Die Kampagne geht auf dem Server weiter; hier wird nichts gespeichert. Ihr Einsatz endet.
ops-unit-destroyed = { $unit } zerstört: zurück zum Einsatzbildschirm

## Missionen

ops-section-missions = Missionen
ops-threat = BEDROHUNG { $threat }
ops-no-missions = Keine Missionen: An der Front ist es ruhig
ops-section-underway = Luftlandungen und Angriffe
ops-enemy-liftoff = FEIND bei { $place }: Abheben in { $mins } min
ops-liftoff = { $place }: Abheben in { $mins } min
ops-enemy-helis = FEIND bei { $place }: { $count } Hubschrauber im Anflug
ops-helis = { $place }: { $count } Hubschrauber unterwegs
ops-raid-sead = KI-SEAD bei { $place }
ops-raid-drones = DROHNEN auf { $place }
ops-raid = ANGRIFF auf Flugplatz { $place }
ops-raid-launch = { $raid }: Start in { $mins } min
ops-raid-on-way = { $raid }: { $count } unterwegs
ops-resupply-ai = NACHSCHUB für { $unit }: UH-60 unterwegs
ops-awacs = FEINDLICHES AWACS noch { $mins } min in der Luft
ops-down = ABGESCHOSSEN: { $pilot } - Rettung innerhalb von { $mins } min
ops-aboard = AN BORD: { $pilot } - auf dem Heimweg
ops-none-underway = Nichts im Gange
ops-ai-helicopter = Ein KI-Hubschrauber übernimmt in { $time }, wenn es kein UH-60-Pilot tut
ops-defences = Abwehr: { $sams } SAM-Stellungen, Bedrohung { $threat }
ops-recon-intel = AUFKLÄRUNGSERGEBNISSE
ops-order-insertion = LUFTLANDUNG BEFEHLEN
    .facts = Fliegen Sie Begleitschutz, um sie zu decken, oder führen Sie sie im UH-60
ops-from = Von
ops-planned-by-group = Von der Gruppe geplant
ops-stores = { $aircraft } - Außenlasten: { $stores }
ops-fly-uh60 = UH-60 FLIEGEN
ops-accept = MISSION ANNEHMEN
ops-planning = PLANT: { $pilots }
ops-flying = FLIEGT: { $pilots }
ops-package = PACKAGE mit { $kinds }: Erfüllen Sie hier zwei Arten innerhalb von { $mins } min, je +{ $bonus } Punkte
ops-sead-on-way = KI-SEAD ist unterwegs
ops-request-sead = KI-SEAD ANFORDERN ({ $cost } Material, { $aircraft } Flugzeuge)
    .facts = Eigene Angriffsflugzeuge greifen vor Ihnen eine SAM-Stellung dieses Ortes an
    .disabled = Nicht genug Material oder Flugzeuge am Boden (siehe KRÄFTE)

## Regeln, Material, Kräfte

ops-rule-damage = realistischer Schaden
ops-rule-guidance = realistische Lenkung
ops-rule-no-crashes = Absturzschutz
ops-rule-time = durchgehende Zeit
ops-rule-weapons = realistische F-117A-Bewaffnung
ops-rule-friendly-fire = Eigenbeschuss
ops-rule-classic = klassisch
ops-materiel = MATERIAL
ops-materiel-blue = BLAU { $materiel } (+{ $income }/h)
ops-materiel-red = ROT { $materiel } (+{ $income }/h)
ops-friendly = EIGENE
ops-enemy = FEIND
ops-section-forces = Kräfte
ops-places-held = Gehaltene Orte
ops-materiel-row = Material
ops-income = Einnahmen pro Stunde
ops-income-from = aus
ops-income-sources = { $base } + { $places } Orte x { $per_place } + { $cities } Städte x { $per_city }
ops-fighters-ground = Jäger am Boden
ops-helicopters-ground = Hubschrauber am Boden
ops-in-air = In der Luft
ops-in-air-value = { $fighters } Jäger, { $strike } Angriffsflugzeuge, { $helicopters } Hubschrauber
ops-since-start = Seit Beginn
ops-earned = verdient
ops-repairs = Reparaturen
ops-operations = Einsätze, Angriffe, Luftlandungen
ops-new-aircraft = neue Flugzeuge
ops-lost = verloren (Städte, Piloten)
ops-section-how = So funktioniert es
ops-how =
    Jede Seite erhält jede Stunde Material: einen Grundbetrag, mehr für jeden gehaltenen Ort und für jede unzerstörte Stadt darunter. Bombardieren Sie die Städte des Feindes (ABRIEGELUNG), um seine Einnahmen zu kürzen.
    Die KI bezahlt alles, was sie tut: Reparaturen, jeden Jägereinsatz, Angriffe und ihre eigenen Luftlandungen. Fehlt ihr Material, bleiben ihre Schäden unrepariert und ihre Flugzeuge am Boden.
    Die KI fliegt nur die Flugzeuge, die sie hat; abgeschossen sind sie weg, gelandet fliegen sie wieder. Jede Stunde kauft jede Seite neue, solange ihr Material reicht.
    Ihnen fehlt nie etwas: Ihre Flugzeuge, Ihre Basen (bombardiert oder nicht) und die Hubschrauber Ihrer eigenen Luftlandebefehle sind immer da.
ops-section-prices = Preise
ops-price-airfield = Reparatur Flugplatz
ops-price-sam = Reparatur SAM-Stellung
ops-price-city = Stadt wiederaufgebaut
ops-price-ship = Reparatur Schiff
ops-price-gun = Geschütz oder MANPADS-Trupp
ops-price-sortie = Jägereinsatz
ops-price-raid = Angriff
ops-price-insertion = KI-Luftlandung
ops-price-fighter = Neuer Jäger
ops-price-helicopter = Neuer Hubschrauber
ops-price-city-lost = Stadt zerstört (verloren)
ops-price-captured = Pilot gefangen (verloren)

## Kriegskarte

ops-click-to-watch = { $unit } - klicken zum Beobachten
ops-section-legend = Legende
ops-legend-friendly-airfield = Eigener Flugplatz
ops-legend-enemy-airfield = Feindlicher Flugplatz
ops-legend-city = Eigene / feindliche Stadt
ops-legend-airfield = Eigener / feindlicher Flugplatz
ops-legend-insertion = Luftlanderoute
ops-watch-hint = Eine Einheit anklicken, um sie live zu beobachten
ops-map-hint = Mausrad: Zoom, Ziehen: verschieben
ops-patriot = Patriot-Batterie
ops-drone-center = Drohnenleitstand
ops-manned-by = Bemannt von { $pilot }
ops-manned-by-other = Von einem anderen Spieler bemannt
ops-control = ÜBERNEHMEN
    .facts = Ihre Konsole übernehmen

## Nachbesprechung, Ende, Logbuch, Chat

ops-no-sortie = Noch kein Einsatz geflogen
ops-section-sortie = Einsatz
ops-achieved = ZIEL ERREICHT
ops-not-achieved = ZIEL NICHT ERREICHT
ops-waiting-raid = WARTEN AUF DEN ANGRIFF
ops-waiting-insertion = WARTEN AUF DIE LUFTLANDUNG
ops-flight-time = Flugzeit { $mins } min
ops-kills = Abschüsse: { $air } Flugzeuge, { $ground } am Boden
ops-points = { $points } Punkte
ops-points-bonus = { $points } Punkte (Package-Bonus +{ $bonus })
ops-campaign-points = { $rank } - { $points } Kampagnenpunkte
ops-campaign-record = Kampagne: { $sorties } Einsätze, { $won } Ziele erreicht
ops-section-meanwhile = Inzwischen
ops-victory = SIEG
    .text = Der Feind hält keinen Flugplatz mehr: Der Schauplatz gehört uns.
ops-defeat = NIEDERLAGE
    .text = Kein eigener Flugplatz ist mehr übrig: Der Schauplatz ist verloren.
ops-end-day = Tag { $day }. Eigene Orte { $friendly }, feindliche Orte { $enemy }.
ops-next-campaign = DIE NÄCHSTE KAMPAGNE BEGINNT IN { $time }
ops-next-campaign-now = DIE NÄCHSTE KAMPAGNE BEGINNT
ops-next-from-menu = Starten Sie die nächste Kampagne im Hauptmenü.
ops-section-log = Kampagnenlogbuch
ops-section-chat = Chat
ops-no-messages = Noch keine Nachrichten
ops-message = Nachricht
    .hint = Nachricht eingeben, Enter sendet
ops-send = SENDEN

## Briefing

ops-briefing = MISSIONSBRIEFING
ops-briefing-line = Von { $base } - { $km } km zum Ziel - { $aircraft } - Außenlasten: { $stores }
ops-route-locked = Route gesperrt: { $pilots } fliegt sie
ops-route-threats = ROUTE UND BEDROHUNGEN
ops-take-off = STARTEN
ops-radars-shown = Gezeigte Radare: von eigenen Flugzeugen entdeckt, noch intakt

## Spieler

ops-section-players = Spieler
ops-col-name = NAME
ops-col-status = STATUS
ops-col-sorties = EINSÄTZE
ops-col-points = PUNKTE
ops-col-pilot = PILOT
ops-col-objectives = ZIELE
ops-col-kills = ABSCH. LUFT/BODEN
ops-col-medal = BESTE AUSZEICHNUNG
ops-you = (Sie)
ops-host = HOST
ops-status-planning = Plant { $title }
ops-status-board = An der Missionstafel
ops-status-flying = Fliegt { $title } - { $aircraft }
ops-status-reconnecting = Verbindung verloren - Flugzeug { $seconds } s gehalten
ops-flown = { $flown } ({ $won } erreicht)
ops-section-leaderboard = Bestenliste
ops-alone = Sie fliegen allein. Um gemeinsam zu fliegen, hosten Sie diese Kampagne über MEHRSPIELER im Hauptmenü.
ops-rules = Regeln dieser Kampagne (seit ihrem Beginn): { $rules }
ops-no-sorties = Noch keine Einsätze geflogen
