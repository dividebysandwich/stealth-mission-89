# Okno kampaně (src/campaign/menu.rs), okno multiplayeru (src/multiplayer_menu.rs),
# legenda a živý pohled válečné mapy (src/campaign/picture.rs, unit_view.rs).

delete = SMAZAT
delete-campaign = SMAZAT TUTO KAMPAŇ?
delete-save = SMAZAT TOTO ULOŽENÍ?
delete-permanent = Smazání je trvalé

## Okno kampaně

cm-group-campaign = Kampaň
cm-weather-dynamic = Dynamické
    .facts =
        Počasí se mění každých 10-22 hodin nebeských hodin
        Oblaka brání ve výhledu oběma stranám
cm-weather-clear = Jasná obloha
    .facts = Po celou kampaň bez oblačnosti
cm-section-day-night = Den a noc
cm-day-real = Reálný čas
    .facts = Den trvá 24 hodin
cm-day-2h = 2 h / den
    .facts = Den trvá 2 hodiny
cm-day-1h = 1 h / den
    .facts = Den trvá 1 hodinu
cm-day-30min = 30 min / den
    .facts = Den trvá 30 minut
cm-conditions-note = Podmínky platí pro novou kampaň
cm-rules-note = Uložená kampaň si ponechává pravidla, se kterými byla zahájena
cm-about =
    Válka běží v reálném čase, i když hrajete, a také při otevřené nabídce.
    Plňte mise z nástěnky: průzkum, SEAD, CAP, CAS, boj proti lodím, úder, eskorta.
    Vysazení vrtulníky dobývá nepřátelská města a letiště.
    Nikdy nezahynete ani nepadnete do zajetí: po bojovém letu jste zpět u nástěnky.
cm-section-new = Nová kampaň
cm-new-line = { $theater }  -  { $situation }  -  protivníci { $opponents }
cm-start = START
cm-section-continue = Pokračovat
cm-no-saves = Žádné uložené kampaně
cm-played = Odehráno { $hours } h
cm-save-gone = Uložení zmizelo
cm-save-unreadable = Uložení nelze přečíst: { $error }
cm-save-undeletable = Uložení nelze smazat: { $error }
cm-in-vr = VE VR
cm-delete =
    .facts = Smazat tuto uloženou kampaň

## Válečná mapa

pic-friendly-aircraft = Spojenecké letadlo
pic-friendly-helicopter = Spojenecký vrtulník
pic-enemy-fighter = Nepřátelská stíhačka (na radaru)
pic-sam-known = Stanoviště SAM (známé)
pic-guns-known = Kanóny / MANPADS (známé)
pic-armour = Tank / BVP (nepřítel: na dohled)
pic-patriot-reach = Dosah Patriot
pic-gepard-stinger = Gepard / Stinger
pic-missile-range = Dosah střel SAM
pic-engagement-range = Dosah palby
pic-reconned = Vyzvěděno
pic-you = Vy
pic-site = Stanoviště: { $place }
uv-live = { $unit } - ŽIVĚ
uv-contact-lost = KONTAKT ZTRACEN
uv-hint = Tažení: otáčení  Kolečko: zoom

## Multiplayer

mp-disconnected = Odpojeno: { $why }
mp-connecting = Připojování k { $address }...
mp-joining = Připojeno k { $address }, vstupuji do hry...
mp-browser-public-only = z prohlížeče se lze připojit pouze k serverům z veřejného seznamu
mp-title-host = MULTIPLAYER: HOSTOVAT HRU
mp-subtitle = Koop v dynamické kampani: veďte válku společně
mp-your-name = Vaše jméno
    .hint = Jméno pilota
mp-join-address = Připojit podle adresy
    .hint = adresa nebo jméno[:port]
mp-connect = PŘIPOJIT
mp-server-name = Název serveru
mp-udp-port = Port UDP
mp-address-note = Ostatní hráči se připojují na adresu tohoto počítače; pro hru přes internet otevřete port UDP v routeru
mp-list-public = Zveřejnit ve veřejném seznamu serverů (ostatní hru najdou ve svém seznamu serverů)
mp-rules = Pravidla pro všechny
mp-time-of-day = Denní doba (noci za měsíčního svitu)
mp-realistic-weapons = Realistická výzbroj F-117A
mp-friendly-fire = Palba do vlastních
mp-day-night = Den a noc:
mp-weather = Počasí:
mp-host-new = HOSTOVAT NOVOU
mp-host-saved = Hostovat uloženou kampaň
mp-host = HOSTOVAT
mp-host-game = HOSTOVAT HRU
mp-port-error = Port UDP { $port }: { $error }
mp-default-server = Kampaň hráče { $pilot }
mp-public-servers = Veřejné servery
mp-refresh = OBNOVIT
mp-server-count = { $servers } serverů, { $pilots } pilotů
mp-list-error = Seznam serverů není dostupný: { $error }
mp-no-servers = Právě žádné veřejné servery
mp-col-server = SERVER
mp-col-pilots = PILOTI
mp-col-war = VÁLKA
mp-dedicated = DEDIKOVANÝ
mp-other-version = jiná verze
mp-join = PŘIPOJIT SE
