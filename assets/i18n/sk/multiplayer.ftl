# Okno kampane (src/campaign/menu.rs), okno multiplayeru (src/multiplayer_menu.rs),
# legenda vojnovej mapy a živý pohľad (src/campaign/picture.rs, unit_view.rs).

delete = ODSTRÁNIŤ
delete-campaign = ODSTRÁNIŤ TÚTO KAMPAŇ?
delete-save = ODSTRÁNIŤ TOTO ULOŽENIE?
delete-permanent = Odstránenie je trvalé

## Okno kampane

cm-group-campaign = Kampaň
cm-weather-dynamic = Dynamické
    .facts =
        Počasie sa mení každých 10-22 hodín podľa hodín oblohy
        Oblaky blokujú priamu viditeľnosť oboch strán
cm-weather-clear = Jasná obloha
    .facts = Počas celej kampane bez oblakov
cm-section-day-night = Deň a noc
cm-day-real = Reálny čas
    .facts = Deň trvá 24 hodín
cm-day-2h = 2 h / deň
    .facts = Deň trvá 2 hodiny
cm-day-1h = 1 h / deň
    .facts = Deň trvá 1 hodinu
cm-day-30min = 30 min / deň
    .facts = Deň trvá 30 minút
cm-conditions-note = Podmienky platia pre novú kampaň
cm-rules-note = Uložená kampaň si ponecháva pravidlá, s ktorými sa začala
cm-about =
    Vojna beží v reálnom čase, kým hráte, aj keď je menu otvorené.
    Plňte misie z tabule: prieskum, SEAD, CAP, CAS, boj proti lodiam, úder, eskorta.
    Vysadenie vrtuľníkmi dobýja nepriateľské mestá a letiská.
    Nikdy vás nezabijú ani nezajmú: po bojovom lete sa vrátite k tabuli.
cm-section-new = Nová kampaň
cm-new-line = { $theater }  -  { $situation }  -  protivníci { $opponents }
cm-start = ŠTART
cm-section-continue = Pokračovať
cm-no-saves = Žiadne uložené kampane
cm-played = Odohrané: { $hours } h
cm-save-gone = Uloženie už neexistuje
cm-save-unreadable = Uloženie sa nepodarilo prečítať: { $error }
cm-save-undeletable = Uloženie sa nepodarilo odstrániť: { $error }
cm-in-vr = VO VR
cm-delete =
    .facts = Odstrániť túto uloženú kampaň

## Vojnová mapa

pic-friendly-aircraft = Spojenecké lietadlo
pic-friendly-helicopter = Spojenecký vrtuľník
pic-enemy-fighter = Nepriateľská stíhačka (na radare)
pic-sam-known = Stanovište SAM (známe)
pic-guns-known = Delá / MANPADS (známe)
pic-armour = Tank / BVP (nepriateľ: na dohľad)
pic-patriot-reach = Dosah Patriot
pic-gepard-stinger = Gepard / Stinger
pic-missile-range = Dosah rakiet SAM
pic-engagement-range = Dosah paľby
pic-reconned = Preskúmané
pic-you = Vy
pic-site = Stanovište: { $place }
uv-live = { $unit } - ŽIVÉ
uv-contact-lost = KONTAKT STRATENÝ
uv-hint = Ťahanie: otáčanie  Koliesko: priblíženie

## Multiplayer

mp-disconnected = Odpojené: { $why }
mp-connecting = Pripájanie k { $address }...
mp-joining = Pripojené k { $address }, vstup do hry...
mp-browser-public-only = z prehliadača sa dá pripojiť len k serverom z verejného zoznamu
mp-title-host = MULTIPLAYER: HOSTIŤ HRU
mp-subtitle = Koop v dynamickej kampani: bojujte vo vojne spoločne
mp-your-name = Vaše meno
    .hint = Meno pilota
mp-join-address = Pripojiť sa podľa adresy
    .hint = adresa alebo meno[:port]
mp-connect = PRIPOJIŤ
mp-server-name = Názov servera
mp-udp-port = Port UDP
mp-address-note = Ostatní hráči sa pripájajú na adresu tohto počítača; ak chcete hrať cez internet, otvorte port UDP vo svojom routeri
mp-list-public = Zobraziť vo verejnom zozname serverov (ostatní nájdu hru vo svojom zozname serverov)
mp-rules = Pravidlá pre všetkých
mp-time-of-day = Denná doba (mesačné noci)
mp-realistic-weapons = Realistické zbrane F-117A
mp-friendly-fire = Paľba do vlastných
mp-day-night = Deň a noc:
mp-weather = Počasie:
mp-host-new = HOSTIŤ NOVÚ
mp-host-saved = Hostiť uloženú kampaň
mp-host = HOSTIŤ
mp-host-game = HOSTIŤ HRU
mp-port-error = Port UDP { $port }: { $error }
mp-default-server = Kampaň hráča { $pilot }
mp-public-servers = Verejné servery
mp-refresh = OBNOVIŤ
mp-server-count = Serverov: { $servers }, pilotov: { $pilots }
mp-list-error = Zoznam serverov je nedostupný: { $error }
mp-no-servers = Momentálne žiadne verejné servery
mp-col-server = SERVER
mp-col-pilots = PILOTI
mp-col-war = VOJNA
mp-dedicated = DEDIKOVANÝ
mp-other-version = iná verzia
mp-join = PRIPOJIŤ SA
