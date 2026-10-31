# A kampányablak (src/campaign/menu.rs), a többjátékos ablak (src/multiplayer_menu.rs),
# a haditérkép jelmagyarázata és élő nézete (src/campaign/picture.rs, unit_view.rs).

delete = TÖRLÉS
delete-campaign = TÖRLÖD EZT A KAMPÁNYT?
delete-save = TÖRLÖD EZT A MENTÉST?
delete-permanent = A törlés végleges

## Kampányablak

cm-group-campaign = Kampány
cm-weather-dynamic = Dinamikus
    .facts =
        Az időjárás az égbolt órája szerint 10-22 óránként változik
        A felhők mindkét fél látóvonalát elzárják
cm-weather-clear = Derült ég
    .facts = Nincs felhő a teljes kampány alatt
cm-section-day-night = Nappal és éjszaka
cm-day-real = Valós idő
    .facts = Egy nap 24 óráig tart
cm-day-2h = 2 ó / nap
    .facts = Egy nap 2 óráig tart
cm-day-1h = 1 ó / nap
    .facts = Egy nap 1 óráig tart
cm-day-30min = 30 perc / nap
    .facts = Egy nap 30 percig tart
cm-conditions-note = A feltételek új kampányra érvényesek
cm-rules-note = A mentett kampány megtartja az indításkor érvényes szabályokat
cm-about =
    A háború valós időben zajlik, amíg játszol, a menü megnyitása alatt is.
    Repüld a táblán lévő küldetéseket: felderítés, SEAD, CAP, CAS, hajók elleni, csapás, kíséret.
    A helikopteres deszantok elfoglalják az ellenség városait és repülőtereit.
    Téged sosem ölnek meg és nem ejtenek fogságba: a bevetés után visszakerülsz a táblához.
cm-section-new = Új kampány
cm-new-line = { $theater }  -  { $situation }  -  ellenfelek: { $opponents }
cm-start = INDÍTÁS
cm-section-continue = Folytatás
cm-no-saves = Nincs mentett kampány
cm-played = { $hours } ó játékidő
cm-save-gone = A mentés eltűnt
cm-save-unreadable = A mentés nem olvasható: { $error }
cm-save-undeletable = A mentés nem törölhető: { $error }
cm-in-vr = VR-BEN
cm-delete =
    .facts = A mentett kampány törlése

## Haditérkép

pic-friendly-aircraft = Baráti gép
pic-friendly-helicopter = Baráti helikopter
pic-enemy-fighter = Ellenséges vadász (radaron)
pic-sam-known = SAM-állás (ismert)
pic-guns-known = Ágyúk / MANPADS (ismert)
pic-armour = Harckocsi / gyalogsági harcjármű (ellenség: látótávolságban)
pic-patriot-reach = Patriot hatótávolság
pic-gepard-stinger = Gepard / Stinger
pic-missile-range = SAM rakéták hatótávolsága
pic-engagement-range = Bevetési távolság
pic-reconned = Felderítve
pic-you = Te
pic-site = Állás: { $place }
uv-live = { $unit } - ÉLŐ
uv-contact-lost = ELVESZETT A KAPCSOLAT
uv-hint = Húzás: keringés  Kerék: nagyítás

## Többjátékos

mp-disconnected = Kapcsolat bontva: { $why }
mp-connecting = Csatlakozás ide: { $address }...
mp-joining = Csatlakozva ide: { $address }, belépés...
mp-browser-public-only = a böngészőből csak a nyilvános listán szereplő szerverekhez lehet csatlakozni
mp-title-host = TÖBBJÁTÉKOS: JÁTÉK INDÍTÁSA
mp-subtitle = Kooperatív játék a dinamikus kampányban: repüljétek együtt a háborút
mp-your-name = A neved
    .hint = Pilóta neve
mp-join-address = Csatlakozás címmel
    .hint = cím vagy név[:port]
mp-connect = CSATLAKOZÁS
mp-server-name = Szerver neve
mp-udp-port = UDP-port
mp-address-note = A többiek ennek a számítógépnek a címével csatlakoznak; az interneten át játszáshoz nyisd meg az UDP-portot a routeren
mp-list-public = Megjelenés a nyilvános szerverlistán (a többiek a szerverlistájukban találják meg a játékot)
mp-rules = Mindenki szabályai
mp-time-of-day = Napszak (holdfényes éjszakák)
mp-realistic-weapons = Valósághű F-117A fegyverzet
mp-friendly-fire = Baráti tűz
mp-day-night = Nappal és éjszaka:
mp-weather = Időjárás:
mp-host-new = ÚJ INDÍTÁSA
mp-host-saved = Mentett kampány indítása
mp-host = INDÍTÁS
mp-host-game = JÁTÉK INDÍTÁSA
mp-port-error = UDP-port { $port }: { $error }
mp-default-server = { $pilot } kampánya
mp-public-servers = Nyilvános szerverek
mp-refresh = FRISSÍTÉS
mp-server-count = { $servers } szerver, { $pilots } pilóta
mp-list-error = A szerverlista nem érhető el: { $error }
mp-no-servers = Jelenleg nincs nyilvános szerver
mp-col-server = SZERVER
mp-col-pilots = PILÓTÁK
mp-col-war = HÁBORÚ
mp-dedicated = DEDIKÁLT
mp-other-version = másik verzió
mp-join = CSATLAKOZÁS
