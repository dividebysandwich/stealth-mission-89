# Prozor kampanje (src/campaign/menu.rs), prozor za više igrača (src/multiplayer_menu.rs),
# legenda ratne karte i pogled uživo (src/campaign/picture.rs, unit_view.rs).

delete = IZBRIŠI
delete-campaign = IZBRISATI OVU KAMPANJU?
delete-save = IZBRISATI OVAJ SPREMAK?
delete-permanent = Brisanje je trajno

## Prozor kampanje

cm-group-campaign = Kampanja
cm-weather-dynamic = Dinamično
    .facts =
        Vrijeme se mijenja svakih 10-22 sata nebeskog sata
        Oblaci blokiraju liniju vidljivosti objema stranama
cm-weather-clear = Vedro nebo
    .facts = Bez oblaka tijekom cijele kampanje
cm-section-day-night = Dan i noć
cm-day-real = Stvarno vrijeme
    .facts = Dan traje 24 sata
cm-day-2h = 2 h / dan
    .facts = Dan traje 2 sata
cm-day-1h = 1 h / dan
    .facts = Dan traje 1 sat
cm-day-30min = 30 min / dan
    .facts = Dan traje 30 minuta
cm-conditions-note = Uvjeti vrijede za novu kampanju
cm-rules-note = Spremljena kampanja zadržava pravila s kojima je započeta
cm-about =
    Rat teče u stvarnom vremenu dok igrate, i kad je izbornik otvoren.
    Letite misije s ploče: izviđanje, SEAD, CAP, CAS, protubrodska borba, udar, pratnja.
    Desanti helikopterima zauzimaju neprijateljske gradove i zračne luke.
    Nikad vas ne ubiju niti zarobe: nakon borbenog leta vraćate se na ploču.
cm-section-new = Nova kampanja
cm-new-line = { $theater }  -  { $situation }  -  protivnici: { $opponents }
cm-start = POKRENI
cm-section-continue = Nastavi
cm-no-saves = Nema spremljenih kampanja
cm-played = Odigrano: { $hours } h
cm-save-gone = Spremak više ne postoji
cm-save-unreadable = Spremak nije moguće pročitati: { $error }
cm-save-undeletable = Spremak nije moguće izbrisati: { $error }
cm-in-vr = U VR-U
cm-delete =
    .facts = Izbriši ovu spremljenu kampanju

## Ratna karta

pic-friendly-aircraft = Saveznički zrakoplov
pic-friendly-helicopter = Saveznički helikopter
pic-enemy-fighter = Neprijateljski lovac (na radaru)
pic-sam-known = SAM položaj (poznat)
pic-guns-known = Topovi / MANPADS (poznati)
pic-armour = Tenk / BVP (neprijatelj: na vidiku)
pic-patriot-reach = Domet Patriota
pic-gepard-stinger = Gepard / Stinger
pic-missile-range = Domet SAM projektila
pic-engagement-range = Domet gađanja
pic-reconned = Izviđano
pic-you = Vi
pic-site = Položaj: { $place }
uv-live = { $unit } - UŽIVO
uv-contact-lost = KONTAKT IZGUBLJEN
uv-hint = Povlačenje: okretanje  Kotačić: zumiranje

## Više igrača

mp-disconnected = Prekinuta veza: { $why }
mp-connecting = Povezivanje na { $address }...
mp-joining = Povezano na { $address }, pridruživanje...
mp-browser-public-only = iz preglednika se može pridružiti samo poslužiteljima s javnog popisa
mp-title-host = VIŠE IGRAČA: POKRENI IGRU
mp-subtitle = Koop u dinamičnoj kampanji: letite u ratu zajedno
mp-your-name = Vaše ime
    .hint = Ime pilota
mp-join-address = Pridruži se preko adrese
    .hint = adresa ili ime[:port]
mp-connect = POVEŽI SE
mp-server-name = Naziv poslužitelja
mp-udp-port = UDP port
mp-address-note = Ostali igrači se pridružuju adresom ovog računala; za igru preko interneta otvorite UDP port na usmjerivaču
mp-list-public = Prikaži na javnom popisu poslužitelja (ostali pronalaze igru u svom popisu poslužitelja)
mp-rules = Pravila za sve
mp-time-of-day = Doba dana (mjesečeve noći)
mp-realistic-weapons = Realistično naoružanje F-117A
mp-friendly-fire = Prijateljska paljba
mp-day-night = Dan i noć:
mp-weather = Vrijeme:
mp-host-new = POKRENI NOVU
mp-host-saved = Pokreni spremljenu kampanju
mp-host = POKRENI
mp-host-game = POKRENI IGRU
mp-port-error = UDP port { $port }: { $error }
mp-default-server = Kampanja: { $pilot }
mp-public-servers = Javni poslužitelji
mp-refresh = OSVJEŽI
mp-server-count = Poslužitelja: { $servers }, pilota: { $pilots }
mp-list-error = Popis poslužitelja nije dostupan: { $error }
mp-no-servers = Trenutno nema javnih poslužitelja
mp-col-server = POSLUŽITELJ
mp-col-pilots = PILOTI
mp-col-war = RAT
mp-dedicated = DEDICIRANI
mp-other-version = druga verzija
mp-join = PRIDRUŽI SE
