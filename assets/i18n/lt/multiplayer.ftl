# Kampanijos langas (src/campaign/menu.rs), daugiažaidės langas (src/multiplayer_menu.rs),
# karo žemėlapio legenda ir gyvas vaizdas (src/campaign/picture.rs, unit_view.rs).

delete = IŠTRINTI
delete-campaign = IŠTRINTI ŠIĄ KAMPANIJĄ?
delete-save = IŠTRINTI ŠĮ IŠSAUGOJIMĄ?
delete-permanent = Ištrynimas negrįžtamas

## Kampanijos langas

cm-group-campaign = Kampanija
cm-weather-dynamic = Dinamiški
    .facts =
        Orai keičiasi kas 10-22 dangaus laikrodžio valandas
        Debesys užstoja matomumą abiem pusėms
cm-weather-clear = Giedras dangus
    .facts = Visoje kampanijoje be debesų
cm-section-day-night = Diena ir naktis
cm-day-real = Realus laikas
    .facts = Para trunka 24 valandas
cm-day-2h = 2 val. / para
    .facts = Para trunka 2 valandas
cm-day-1h = 1 val. / para
    .facts = Para trunka 1 valandą
cm-day-30min = 30 min / para
    .facts = Para trunka 30 minučių
cm-conditions-note = Sąlygos taikomos naujai kampanijai
cm-rules-note = Išsaugota kampanija išlaiko taisykles, su kuriomis buvo pradėta
cm-about =
    Karas vyksta realiu laiku, kol žaidžiate, taip pat ir atidarius meniu.
    Skraidykite lentoje esančias misijas: žvalgyba, SEAD, CAP, CAS, priešlaivinė kova, smūgis, palyda.
    Sraigtasparnių desantavimai užima priešo miestus ir aerodromus.
    Jūsų niekada nenužudo ir nepaima į nelaisvę: po skrydžio grįžtate prie lentos.
cm-section-new = Nauja kampanija
cm-new-line = { $theater }  -  { $situation }  -  priešininkai: { $opponents }
cm-start = PRADĖTI
cm-section-continue = Tęsti
cm-no-saves = Išsaugotų kampanijų nėra
cm-played = Žaista: { $hours } val.
cm-save-gone = Išsaugojimo nebėra
cm-save-unreadable = Išsaugojimo nepavyko perskaityti: { $error }
cm-save-undeletable = Išsaugojimo nepavyko ištrinti: { $error }
cm-in-vr = VR
cm-delete =
    .facts = Ištrinti šią išsaugotą kampaniją

## Karo žemėlapis

pic-friendly-aircraft = Draugiškas orlaivis
pic-friendly-helicopter = Draugiškas sraigtasparnis
pic-enemy-fighter = Priešo naikintuvas (radare)
pic-sam-known = SAM pozicija (žinoma)
pic-guns-known = Pabūklai / MANPADS (žinomi)
pic-armour = Tankas / PKM (priešas: matomas)
pic-patriot-reach = Patriot veikimo nuotolis
pic-gepard-stinger = Gepard / Stinger
pic-missile-range = SAM raketų nuotolis
pic-engagement-range = Poveikio nuotolis
pic-reconned = Išžvalgyta
pic-you = Jūs
pic-site = Vietovė: { $place }
uv-live = { $unit } - GYVAI
uv-contact-lost = RYŠYS PRARASTAS
uv-hint = Vilkimas: sukti  Ratukas: didinti

## Daugiažaidė

mp-disconnected = Atsijungta: { $why }
mp-connecting = Jungiamasi prie { $address }...
mp-joining = Prisijungta prie { $address }, jungiamasi į žaidimą...
mp-browser-public-only = iš naršyklės galima jungtis tik prie viešajame sąraše esančių serverių
mp-title-host = DAUGIAŽAIDĖ: PRIIMTI ŽAIDIMĄ
mp-subtitle = Kooperacija dinaminėje kampanijoje: kariaukite kartu
mp-your-name = Jūsų vardas
    .hint = Piloto vardas
mp-join-address = Prisijungti pagal adresą
    .hint = adresas arba vardas[:prievadas]
mp-connect = PRISIJUNGTI
mp-server-name = Serverio pavadinimas
mp-udp-port = UDP prievadas
mp-address-note = Kiti žaidėjai jungiasi šio kompiuterio adresu; norėdami žaisti internetu, atidarykite UDP prievadą maršrutizatoriuje
mp-list-public = Rodyti viešajame serverių sąraše (kiti suras žaidimą savo serverių sąraše)
mp-rules = Taisyklės visiems
mp-time-of-day = Paros laikas (mėnesienos naktys)
mp-realistic-weapons = Realistiški F-117A ginklai
mp-friendly-fire = Draugiška ugnis
mp-day-night = Diena ir naktis:
mp-weather = Orai:
mp-host-new = PRIIMTI NAUJĄ
mp-host-saved = Priimti išsaugotą kampaniją
mp-host = PRIIMTI
mp-host-game = PRIIMTI ŽAIDIMĄ
mp-port-error = UDP prievadas { $port }: { $error }
mp-default-server = Kampanija: { $pilot }
mp-public-servers = Viešieji serveriai
mp-refresh = ATNAUJINTI
mp-server-count = Serveriai: { $servers }, pilotai: { $pilots }
mp-list-error = Serverių sąrašas nepasiekiamas: { $error }
mp-no-servers = Šiuo metu viešų serverių nėra
mp-col-server = SERVERIS
mp-col-pilots = PILOTAI
mp-col-war = KARAS
mp-dedicated = SPECIALIZUOTAS
mp-other-version = kita versija
mp-join = PRISIJUNGTI
