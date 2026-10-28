# Fereastra campaniei (src/campaign/menu.rs), fereastra multiplayer (src/multiplayer_menu.rs),
# legenda hărții de război și vizualizarea live (src/campaign/picture.rs, unit_view.rs).

delete = ȘTERGE
delete-campaign = ȘTERGEȚI ACEASTĂ CAMPANIE?
delete-save = ȘTERGEȚI ACEASTĂ SALVARE?
delete-permanent = Ștergerea este definitivă

## Fereastra campaniei

cm-group-campaign = Campanie
cm-weather-dynamic = Dinamică
    .facts =
        Vremea se schimbă la fiecare 10-22 de ore ale ceasului cerului
        Norii blochează linia de vedere pentru ambele tabere
cm-weather-clear = Cer senin
    .facts = Fără nori pe toată durata campaniei
cm-section-day-night = Zi și noapte
cm-day-real = Timp real
    .facts = O zi durează 24 de ore
cm-day-2h = 2 h / zi
    .facts = O zi durează 2 ore
cm-day-1h = 1 h / zi
    .facts = O zi durează 1 oră
cm-day-30min = 30 min / zi
    .facts = O zi durează 30 de minute
cm-conditions-note = Condițiile se aplică unei campanii noi
cm-rules-note = O campanie salvată păstrează regulile cu care a început
cm-about =
    Războiul se desfășoară în timp real cât joacă, chiar și cu meniul deschis.
    Zburați misiunile de pe tablă: recon, SEAD, CAP, CAS, antinavă, lovitură, escortă.
    Debarcările cu elicoptere iau orașele și aerodromurile inamicului.
    Nu sunteți niciodată ucis sau capturat: după un zbor reveniți la tablă.
cm-section-new = Campanie nouă
cm-new-line = { $theater }  -  { $situation }  -  adversari { $opponents }
cm-start = START
cm-section-continue = Continuare
cm-no-saves = Nicio campanie salvată
cm-played = { $hours } h jucate
cm-save-gone = Salvarea a dispărut
cm-save-unreadable = Salvarea nu a putut fi citită: { $error }
cm-save-undeletable = Salvarea nu a putut fi ștearsă: { $error }
cm-in-vr = ÎN VR
cm-delete =
    .facts = Șterge această campanie salvată

## Harta de război

pic-friendly-aircraft = Aeronavă prietenă
pic-friendly-helicopter = Elicopter prieten
pic-enemy-fighter = Avion de vânătoare inamic (pe radar)
pic-sam-known = Poziție SAM (cunoscută)
pic-guns-known = Tunuri / MANPADS (cunoscute)
pic-reconned = Recunoscut
pic-you = Dumneavoastră
pic-site = Poziția { $place }
uv-live = { $unit } - LIVE
uv-contact-lost = CONTACT PIERDUT
uv-hint = Tragere: orbitare  Rotiță: zoom

## Multiplayer

mp-disconnected = Deconectat: { $why }
mp-connecting = Conectare la { $address }...
mp-joining = Conectat la { $address }, se intră în joc...
mp-browser-public-only = din browser se pot alătura doar serverele din lista publică
mp-title-host = MULTIPLAYER: GĂZDUIEȘTE UN JOC
mp-subtitle = Coop în campania dinamică: luptați împreună în război
mp-your-name = Numele dumneavoastră
    .hint = Numele pilotului
mp-join-address = Alăturare prin adresă
    .hint = adresă sau nume[:port]
mp-connect = CONECTARE
mp-server-name = Numele serverului
mp-udp-port = Port UDP
mp-address-note = Ceilalți jucători se alătură cu adresa acestui calculator; deschideți portul UDP în router pentru a juca prin internet
mp-list-public = Afișare în lista publică de servere (ceilalți găsesc jocul în lista lor de servere)
mp-rules = Reguli pentru toți
mp-time-of-day = Ora din zi (nopți cu lună)
mp-realistic-weapons = Armament realist F-117A
mp-friendly-fire = Foc prietenesc
mp-day-night = Zi și noapte:
mp-weather = Vreme:
mp-host-new = GĂZDUIEȘTE NOU
mp-host-saved = Găzduiește o campanie salvată
mp-host = GĂZDUIEȘTE
mp-host-game = GĂZDUIEȘTE UN JOC
mp-port-error = Port UDP { $port }: { $error }
mp-default-server = Campania lui { $pilot }
mp-public-servers = Servere publice
mp-refresh = ACTUALIZARE
mp-server-count = { $servers } servere, { $pilots } piloți
mp-list-error = Lista de servere nu este accesibilă: { $error }
mp-no-servers = Niciun server public în acest moment
mp-col-server = SERVER
mp-col-pilots = PILOȚI
mp-col-war = RĂZBOI
mp-dedicated = DEDICAT
mp-other-version = altă versiune
mp-join = ALĂTURARE
