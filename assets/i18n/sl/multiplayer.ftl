# Okno kampanje (src/campaign/menu.rs), okno za več igralcev (src/multiplayer_menu.rs),
# legenda in pogled v živo vojne karte (src/campaign/picture.rs, unit_view.rs).

delete = IZBRIŠI
delete-campaign = IZBRIŠEM TO KAMPANJO?
delete-save = IZBRIŠEM TA ZAPIS?
delete-permanent = Brisanje je trajno

## Okno kampanje

cm-group-campaign = Kampanja
cm-weather-dynamic = Dinamično
    .facts =
        Vreme se spremeni vsakih 10-22 ur nebesne ure
        Oblaki zakrivajo vidno črto obema stranema
cm-weather-clear = Jasno nebo
    .facts = Brez oblakov skozi vso kampanjo
cm-section-day-night = Dan in noč
cm-day-real = Realni čas
    .facts = Dan traja 24 ur
cm-day-2h = 2 h / dan
    .facts = Dan traja 2 uri
cm-day-1h = 1 h / dan
    .facts = Dan traja 1 uro
cm-day-30min = 30 min / dan
    .facts = Dan traja 30 minut
cm-conditions-note = Razmere veljajo za novo kampanjo
cm-rules-note = Shranjena kampanja ohrani pravila, s katerimi je bila začeta
cm-about =
    Vojna poteka v realnem času, medtem ko igrate, tudi ko je meni odprt.
    Letite misije s table: izvidovanje, SEAD, CAP, CAS, protiladijski boj, udar, spremstvo.
    Desanti s helikopterji zavzemajo sovražnikova mesta in letališča.
    Nikoli niste ubiti ali ujeti: po bojnem letu ste spet pri tabli.
cm-section-new = Nova kampanja
cm-new-line = { $theater }  -  { $situation }  -  nasprotniki { $opponents }
cm-start = ZAČNI
cm-section-continue = Nadaljuj
cm-no-saves = Ni shranjenih kampanj
cm-played = Odigranih { $hours } h
cm-save-gone = Zapisa ni več
cm-save-unreadable = Zapisa ni bilo mogoče prebrati: { $error }
cm-save-undeletable = Zapisa ni bilo mogoče izbrisati: { $error }
cm-in-vr = V VR
cm-delete =
    .facts = Izbriši to shranjeno kampanjo

## Vojna karta

pic-friendly-aircraft = Zavezniško letalo
pic-friendly-helicopter = Zavezniški helikopter
pic-enemy-fighter = Sovražni lovec (na radarju)
pic-sam-known = Položaj SAM (znan)
pic-guns-known = Topovi / MANPADS (znani)
pic-armour = Tank / BVP (sovražnik: na vidiku)
pic-patriot-reach = Doseg Patriot
pic-gepard-stinger = Gepard / Stinger
pic-missile-range = Doseg izstrelkov SAM
pic-engagement-range = Doseg bojevanja
pic-reconned = Izvidano
pic-you = Vi
pic-site = Položaj: { $place }
uv-live = { $unit } - V ŽIVO
uv-contact-lost = STIK IZGUBLJEN
uv-hint = Vleci: kroži  Kolesce: povečava

## Več igralcev

mp-disconnected = Prekinjena povezava: { $why }
mp-connecting = Povezovanje z { $address }...
mp-joining = Povezan z { $address }, pridružitev...
mp-browser-public-only = iz brskalnika se je mogoče pridružiti le strežnikom z javnega seznama
mp-title-host = VEČ IGRALCEV: GOSTITE IGRO
mp-subtitle = Sodelovanje v dinamični kampanji: vojno bijte skupaj
mp-your-name = Vaše ime
    .hint = Ime pilota
mp-join-address = Pridružitev po naslovu
    .hint = naslov ali ime[:vrata]
mp-connect = POVEŽI
mp-server-name = Ime strežnika
mp-udp-port = Vrata UDP
mp-address-note = Drugi igralci se pridružijo z naslovom tega računalnika; za igro prek interneta odprite vrata UDP v usmerjevalniku
mp-list-public = Prikaži na javnem seznamu strežnikov (drugi najdejo igro v svojem seznamu strežnikov)
mp-rules = Pravila za vse
mp-time-of-day = Čas dneva (mesečne noči)
mp-realistic-weapons = Realistično orožje F-117A
mp-friendly-fire = Ogenj po svojih
mp-day-night = Dan in noč:
mp-weather = Vreme:
mp-host-new = GOSTI NOVO
mp-host-saved = Gostite shranjeno kampanjo
mp-host = GOSTI
mp-host-game = GOSTITE IGRO
mp-port-error = Vrata UDP { $port }: { $error }
mp-default-server = Kampanja pilota { $pilot }
mp-public-servers = Javni strežniki
mp-refresh = OSVEŽI
mp-server-count = { $servers } strežnikov, { $pilots } pilotov
mp-list-error = Seznam strežnikov ni dosegljiv: { $error }
mp-no-servers = Trenutno ni javnih strežnikov
mp-col-server = STREŽNIK
mp-col-pilots = PILOTI
mp-col-war = VOJNA
mp-dedicated = NAMENSKI
mp-other-version = druga različica
mp-join = PRIDRUŽI SE
