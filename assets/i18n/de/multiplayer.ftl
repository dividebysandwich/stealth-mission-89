# Das Kampagnenfenster (src/campaign/menu.rs), das Mehrspielerfenster (src/multiplayer_menu.rs),
# die Legende und Live-Ansicht der Kriegskarte (src/campaign/picture.rs, unit_view.rs).

delete = LÖSCHEN
delete-campaign = DIESE KAMPAGNE LÖSCHEN?
delete-save = DIESEN SPIELSTAND LÖSCHEN?
delete-permanent = Das Löschen ist endgültig

## Kampagnenfenster

cm-group-campaign = Kampagne
cm-weather-dynamic = Dynamisch
    .facts =
        Das Wetter ändert sich alle 10-22 Stunden der Himmelsuhr
        Wolken versperren beiden Seiten die Sicht
cm-weather-clear = Klarer Himmel
    .facts = Keine Wolken während der ganzen Kampagne
cm-section-day-night = Tag und Nacht
cm-day-real = Echtzeit
    .facts = Ein Tag dauert 24 Stunden
cm-day-2h = 2 h / Tag
    .facts = Ein Tag dauert 2 Stunden
cm-day-1h = 1 h / Tag
    .facts = Ein Tag dauert 1 Stunde
cm-day-30min = 30 min / Tag
    .facts = Ein Tag dauert 30 Minuten
cm-conditions-note = Die Bedingungen gelten für eine neue Kampagne
cm-rules-note = Eine gespeicherte Kampagne behält die Regeln, mit denen sie begonnen wurde
cm-about =
    Der Krieg läuft in Echtzeit, während Sie spielen, auch bei offenem Menü.
    Fliegen Sie die Missionen der Tafel: Aufklärung, SEAD, CAP, CAS, Seeziele, Angriff, Begleitschutz.
    Hubschrauber-Luftlandungen erobern die Städte und Flugplätze des Feindes.
    Sie werden nie getötet oder gefangen genommen: Nach einem Einsatz sind Sie zurück an der Tafel.
cm-section-new = Neue Kampagne
cm-new-line = { $theater }  -  { $situation }  -  Gegner { $opponents }
cm-start = STARTEN
cm-section-continue = Fortsetzen
cm-no-saves = Keine gespeicherten Kampagnen
cm-played = { $hours } h gespielt
cm-save-gone = Der Spielstand existiert nicht mehr
cm-save-unreadable = Der Spielstand konnte nicht gelesen werden: { $error }
cm-save-undeletable = Der Spielstand konnte nicht gelöscht werden: { $error }
cm-in-vr = IN VR
cm-delete =
    .facts = Diese gespeicherte Kampagne löschen

## Kriegskarte

pic-friendly-aircraft = Eigenes Flugzeug
pic-friendly-helicopter = Eigener Hubschrauber
pic-enemy-fighter = Feindlicher Jäger (auf dem Radar)
pic-sam-known = SAM-Stellung (bekannt)
pic-guns-known = Flak / MANPADS (bekannt)
pic-armour = Panzer / SPz (feindlich: in Sicht)
pic-patriot-reach = Patriot-Reichweite
pic-gepard-stinger = Gepard / Stinger
pic-missile-range = Reichweite der SAM-Raketen
pic-engagement-range = Bekämpfungsreichweite
pic-reconned = Aufgeklärt
pic-you = Sie
pic-site = Stellung { $place }
uv-live = { $unit } - LIVE
uv-contact-lost = KONTAKT VERLOREN
uv-hint = Ziehen: umkreisen  Mausrad: Zoom

## Mehrspieler

mp-disconnected = Verbindung getrennt: { $why }
mp-connecting = Verbinde mit { $address }...
mp-joining = Verbunden mit { $address }, trete bei...
mp-browser-public-only = im Browser kann nur Servern der öffentlichen Liste beigetreten werden
mp-title-host = MEHRSPIELER: SPIEL HOSTEN
mp-subtitle = Koop in der dynamischen Kampagne: Fliegen Sie den Krieg gemeinsam
mp-your-name = Ihr Name
    .hint = Pilotenname
mp-join-address = Über Adresse beitreten
    .hint = Adresse oder Name[:Port]
mp-connect = VERBINDEN
mp-server-name = Servername
mp-udp-port = UDP-Port
mp-address-note = Andere Spieler treten mit der Adresse dieses Computers bei; öffnen Sie den UDP-Port in Ihrem Router, um über das Internet zu spielen
mp-list-public = In der öffentlichen Serverliste anzeigen (andere finden das Spiel in ihrer Serverliste)
mp-rules = Regeln für alle
mp-time-of-day = Tageszeit (mondhelle Nächte)
mp-realistic-weapons = Realistische F-117A-Bewaffnung
mp-friendly-fire = Eigenbeschuss
mp-day-night = Tag und Nacht:
mp-weather = Wetter:
mp-host-new = NEUE HOSTEN
mp-host-saved = Eine gespeicherte Kampagne hosten
mp-host = HOSTEN
mp-host-game = SPIEL HOSTEN
mp-port-error = UDP-Port { $port }: { $error }
mp-default-server = Kampagne von { $pilot }
mp-public-servers = Öffentliche Server
mp-refresh = AKTUALISIEREN
mp-server-count = { $servers } Server, { $pilots } Piloten
mp-list-error = Die Serverliste ist nicht erreichbar: { $error }
mp-no-servers = Zurzeit keine öffentlichen Server
mp-col-server = SERVER
mp-col-pilots = PILOTEN
mp-col-war = KRIEG
mp-dedicated = DEDIZIERT
mp-other-version = andere Version
mp-join = BEITRETEN
