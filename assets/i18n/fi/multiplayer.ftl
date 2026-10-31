# Kampanjaikkuna (src/campaign/menu.rs), moninpeli-ikkuna (src/multiplayer_menu.rs),
# sotakartan selite ja live-näkymä (src/campaign/picture.rs, unit_view.rs).

delete = POISTA
delete-campaign = POISTETAANKO TÄMÄ KAMPANJA?
delete-save = POISTETAANKO TÄMÄ TALLENNUS?
delete-permanent = Poisto on pysyvä

## Kampanjaikkuna

cm-group-campaign = Kampanja
cm-weather-dynamic = Dynaaminen
    .facts =
        Sää muuttuu taivaan kellon mukaan 10-22 tunnin välein
        Pilvet estävät näköyhteyden molemmilta osapuolilta
cm-weather-clear = Selkeä taivas
    .facts = Ei pilviä koko kampanjan aikana
cm-section-day-night = Päivä ja yö
cm-day-real = Reaaliaika
    .facts = Päivä kestää 24 tuntia
cm-day-2h = 2 h / päivä
    .facts = Päivä kestää 2 tuntia
cm-day-1h = 1 h / päivä
    .facts = Päivä kestää 1 tunnin
cm-day-30min = 30 min / päivä
    .facts = Päivä kestää 30 minuuttia
cm-conditions-note = Olosuhteet koskevat uutta kampanjaa
cm-rules-note = Tallennettu kampanja säilyttää säännöt, joilla se aloitettiin
cm-about =
    Sota käy reaaliajassa pelatessasi, myös valikon ollessa auki.
    Lennä taululla olevat tehtävät: tiedustelu, SEAD, CAP, CAS, laivantorjunta, isku, saattaja.
    Helikopterien maihinnousut valtaavat vihollisen kaupungit ja lentokentät.
    Sinua ei koskaan tapeta eikä vangita: lennon jälkeen palaat taululle.
cm-section-new = Uusi kampanja
cm-new-line = { $theater }  -  { $situation }  -  vastustajat { $opponents }
cm-start = ALOITA
cm-section-continue = Jatka
cm-no-saves = Ei tallennettuja kampanjoita
cm-played = pelattu { $hours } h
cm-save-gone = Tallennusta ei enää ole
cm-save-unreadable = Tallennusta ei voitu lukea: { $error }
cm-save-undeletable = Tallennusta ei voitu poistaa: { $error }
cm-in-vr = VR:SSÄ
cm-delete =
    .facts = Poista tämä tallennettu kampanja

## Sotakartta

pic-friendly-aircraft = Oma kone
pic-friendly-helicopter = Oma helikopteri
pic-enemy-fighter = Vihollisen hävittäjä (tutkassa)
pic-sam-known = SAM-asema (tunnettu)
pic-guns-known = Tykit / MANPADS (tunnettu)
pic-armour = Panssarivaunu / rynnäkköpanssarivaunu (vihollinen: näkyvissä)
pic-patriot-reach = Patriot-kantama
pic-gepard-stinger = Gepard / Stinger
pic-missile-range = SAM-ohjusten kantama
pic-engagement-range = Tulituskantama
pic-reconned = Tiedusteltu
pic-you = Sinä
pic-site = Paikka: { $place }
uv-live = { $unit } - LIVE
uv-contact-lost = YHTEYS KATKESI
uv-hint = Vedä: kierrä  Pyörä: zoom

## Moninpeli

mp-disconnected = Yhteys katkennut: { $why }
mp-connecting = Yhdistetään: { $address }...
mp-joining = Yhdistetty: { $address }, liitytään...
mp-browser-public-only = selaimesta voi liittyä vain julkisen listan palvelimiin
mp-title-host = MONINPELI: ISÄNNÖI PELIÄ
mp-subtitle = Yhteistyö dynaamisessa kampanjassa: lentäkää sota yhdessä
mp-your-name = Nimesi
    .hint = Lentäjän nimi
mp-join-address = Liity osoitteella
    .hint = osoite tai nimi[:portti]
mp-connect = YHDISTÄ
mp-server-name = Palvelimen nimi
mp-udp-port = UDP-portti
mp-address-note = Muut pelaajat liittyvät tämän tietokoneen osoitteella; avaa UDP-portti reitittimessäsi pelataksesi internetin yli
mp-list-public = Näytä julkisella palvelinlistalla (muut löytävät pelin palvelinlistastaan)
mp-rules = Säännöt kaikille
mp-time-of-day = Vuorokaudenaika (kuutamoyöt)
mp-realistic-weapons = Realistiset F-117A-aseet
mp-friendly-fire = Omien tulitus
mp-day-night = Päivä ja yö:
mp-weather = Sää:
mp-host-new = ISÄNNÖI UUTTA
mp-host-saved = Isännöi tallennettua kampanjaa
mp-host = ISÄNNÖI
mp-host-game = ISÄNNÖI PELIÄ
mp-port-error = UDP-portti { $port }: { $error }
mp-default-server = Kampanja: { $pilot }
mp-public-servers = Julkiset palvelimet
mp-refresh = PÄIVITÄ
mp-server-count = Palvelimia { $servers }, lentäjiä { $pilots }
mp-list-error = Palvelinlistaa ei tavoiteta: { $error }
mp-no-servers = Ei julkisia palvelimia juuri nyt
mp-col-server = PALVELIN
mp-col-pilots = LENTÄJÄT
mp-col-war = SOTA
mp-dedicated = DEDICATED
mp-other-version = eri versio
mp-join = LIITY
