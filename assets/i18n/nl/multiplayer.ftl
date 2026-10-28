# Het campagnevenster (src/campaign/menu.rs), het multiplayervenster (src/multiplayer_menu.rs),
# de legenda en live weergave van de oorlogskaart (src/campaign/picture.rs, unit_view.rs).

delete = VERWIJDEREN
delete-campaign = DEZE CAMPAGNE VERWIJDEREN?
delete-save = DEZE SAVE VERWIJDEREN?
delete-permanent = Verwijderen is definitief

## Campagnevenster

cm-group-campaign = Campagne
cm-weather-dynamic = Dynamisch
    .facts =
        Het weer verandert elke 10-22 uur op de klok van de hemel
        Wolken blokkeren het zicht voor beide partijen
cm-weather-clear = Onbewolkt
    .facts = Geen wolken tijdens de hele campagne
cm-section-day-night = Dag en nacht
cm-day-real = Echte tijd
    .facts = Een dag duurt 24 uur
cm-day-2h = 2 u / dag
    .facts = Een dag duurt 2 uur
cm-day-1h = 1 u / dag
    .facts = Een dag duurt 1 uur
cm-day-30min = 30 min / dag
    .facts = Een dag duurt 30 minuten
cm-conditions-note = De omstandigheden gelden voor een nieuwe campagne
cm-rules-note = Een opgeslagen campagne behoudt de regels waarmee ze is gestart
cm-about =
    De oorlog loopt in real time terwijl je speelt, ook als het menu open is.
    Vlieg de missies op het bord: recon, SEAD, CAP, CAS, antischeeps, aanval, escorte.
    Helikopterinzetten nemen de steden en vliegvelden van de vijand in.
    Je wordt nooit gedood of gevangengenomen: na een vlucht ben je terug bij het bord.
cm-section-new = Nieuwe campagne
cm-new-line = { $theater }  -  { $situation }  -  tegenstanders { $opponents }
cm-start = START
cm-section-continue = Hervatten
cm-no-saves = Geen opgeslagen campagnes
cm-played = { $hours } u gespeeld
cm-save-gone = De save bestaat niet meer
cm-save-unreadable = De save kon niet worden gelezen: { $error }
cm-save-undeletable = De save kon niet worden verwijderd: { $error }
cm-in-vr = IN VR
cm-delete =
    .facts = Deze opgeslagen campagne verwijderen

## Oorlogskaart

pic-friendly-aircraft = Bevriend toestel
pic-friendly-helicopter = Bevriende helikopter
pic-enemy-fighter = Vijandelijke jager (op radar)
pic-sam-known = SAM-site (bekend)
pic-guns-known = Kanonnen / MANPADS (bekend)
pic-reconned = Verkend
pic-you = Jij
pic-site = Site { $place }
uv-live = { $unit } - LIVE
uv-contact-lost = CONTACT VERLOREN
uv-hint = Slepen: draaien  Wiel: zoomen

## Multiplayer

mp-disconnected = Verbinding verbroken: { $why }
mp-connecting = Verbinden met { $address }...
mp-joining = Verbonden met { $address }, deelnemen...
mp-browser-public-only = vanuit de browser kun je alleen deelnemen aan servers van de openbare lijst
mp-title-host = MULTIPLAYER: HOST EEN SPEL
mp-subtitle = Coöp in de dynamische campagne: vecht de oorlog samen uit
mp-your-name = Je naam
    .hint = Pilotennaam
mp-join-address = Deelnemen via adres
    .hint = adres of naam[:poort]
mp-connect = VERBIND
mp-server-name = Servernaam
mp-udp-port = UDP-poort
mp-address-note = Andere spelers nemen deel met het adres van deze computer; open de UDP-poort in je router om via internet te spelen
mp-list-public = Op de openbare serverlijst plaatsen (anderen vinden het spel in hun serverlijst)
mp-rules = Regels voor iedereen
mp-time-of-day = Tijd van de dag (nachten met maanlicht)
mp-realistic-weapons = Realistische F-117A-wapens
mp-friendly-fire = Friendly fire
mp-day-night = Dag en nacht:
mp-weather = Weer:
mp-host-new = NIEUW HOSTEN
mp-host-saved = Een opgeslagen campagne hosten
mp-host = HOST
mp-host-game = HOST EEN SPEL
mp-port-error = UDP-poort { $port }: { $error }
mp-default-server = Campagne van { $pilot }
mp-public-servers = Openbare servers
mp-refresh = VERVERSEN
mp-server-count = { $servers } servers, { $pilots } piloten
mp-list-error = De serverlijst is niet bereikbaar: { $error }
mp-no-servers = Momenteel geen openbare servers
mp-col-server = SERVER
mp-col-pilots = PILOTEN
mp-col-war = OORLOG
mp-dedicated = DEDICATED
mp-other-version = andere versie
mp-join = DEELNEMEN
