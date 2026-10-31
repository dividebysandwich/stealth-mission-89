# Die Meldungen des Spiels im HUD und in den 2D-Einblendungen des Flugs (src/hud, src/combat.rs,
# src/csar_view.rs, src/campaign). Das HUD zeigt sie in Großbuchstaben ohne Umlaute; die
# Wörter der Avionik (CCIP, THR, KM, MAX...) bleiben englisch.

hud-no-ejection-seat = KEIN SCHLEUDERSITZ
hud-waypoint-reached = Wegpunkt { $waypoint } erreicht
hud-select-camera = KAMERA WÄHLEN
hud-open-bay = SCHACHT ÖFFNEN
hud-camera-page = SEITE { $page }
hud-target-in-view-shoot = ZIEL IM BILD - AUSLÖSEN
hud-target-in-view = ZIEL IM BILD
hud-aim-at-target = ZIEL ANVISIEREN
hud-photo = FOTO { $state }
hud-rearmed = Aufmunitioniert und betankt
hud-landed-rearm = Gelandet - aufmunitionieren oder Einsatz beenden
hud-objective-achieved = Ziel erreicht

## Einblendung am Missionsende

end-accomplished = erfüllt
end-not-accomplished = nicht erfüllt
end-primary = Primärziel { $result }
end-secondary = Sekundärziel { $result }
end-kills = Abschüsse: { $air } Flugzeuge, { $ground } Bodenziele
end-time = Missionszeit { $time }
end-damage = Schaden: { $hits } Treffer
end-training = Übungsmission
end-press-any-key = Beliebige Taste drücken

## Die Anzeige des abgeschossenen Piloten

csar-flare = SIGNALFACKEL
csar-faster = SCHNELLER
csar-normal = NORMALE ZEIT
csar-skip = BERGUNG ÜBERSPRINGEN
csar-parachute = FALLSCHIRM OFFEN
csar-on-ground = AM BODEN
csar-down = AM BODEN IN FEINDGEBIET
csar-pickup = RETTUNG   UH-60 ÜBER IHNEN - AUFNAHME
csar-inbound = RETTUNG   UH-60 { $km } KM { $direction } - IM ANFLUG
csar-fire-flare = RETTUNG   UH-60 { $km } KM { $direction } - SIGNALFACKEL ZÜNDEN, WENN NAH
csar-enemy = FEIND     { $count } HIND, NÄCHSTER { $km } KM
csar-flares = FACKELN  { $count }      AM BODEN SEIT { $time }

## Koop-Hinweise auf der Anzeige (src/player_notices.rs), der Hinweis der Luftabwehrkonsole

notice-joined = { $pilot } ist beigetreten
notice-left = { $pilot } hat das Spiel verlassen
notice-lost-connection = { $pilot } hat die Verbindung verloren
st-select-track = Zuerst eine Spur wählen
