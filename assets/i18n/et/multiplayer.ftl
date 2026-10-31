# Kampaania aken (src/campaign/menu.rs), mitmikmängu aken (src/multiplayer_menu.rs),
# sõjakaardi legend ja otsevaade (src/campaign/picture.rs, unit_view.rs).

delete = KUSTUTA
delete-campaign = KUSTUTA SEE KAMPAANIA?
delete-save = KUSTUTA SEE SALVESTUS?
delete-permanent = Kustutamine on lõplik

## Kampaania aken

cm-group-campaign = Kampaania
cm-weather-dynamic = Dünaamiline
    .facts =
        Ilm muutub iga 10-22 taevakella tunni järel
        Pilved takistavad nähtavust mõlemale poolele
cm-weather-clear = Selge taevas
    .facts = Pilvi pole kogu kampaania jooksul
cm-section-day-night = Päev ja öö
cm-day-real = Reaalaeg
    .facts = Ööpäev kestab 24 tundi
cm-day-2h = 2 h / päev
    .facts = Ööpäev kestab 2 tundi
cm-day-1h = 1 h / päev
    .facts = Ööpäev kestab 1 tunni
cm-day-30min = 30 min / päev
    .facts = Ööpäev kestab 30 minutit
cm-conditions-note = Tingimused kehtivad uuele kampaaniale
cm-rules-note = Salvestatud kampaania säilitab reeglid, millega see algas
cm-about =
    Sõda kulgeb reaalajas, kui mängid, ka siis, kui menüü on avatud.
    Lenda tahvlil olevaid missioone: luure, SEAD, CAP, CAS, laevatõrje, löök, eskort.
    Helikopteridessant vallutab vaenlase linnu ja lennuväljad.
    Sind ei tapeta ega vangistata: pärast lendu oled tagasi tahvli juures.
cm-section-new = Uus kampaania
cm-new-line = { $theater }  -  { $situation }  -  vastased { $opponents }
cm-start = ALUSTA
cm-section-continue = Jätka
cm-no-saves = Salvestatud kampaaniaid pole
cm-played = { $hours } h mängitud
cm-save-gone = Salvestust enam pole
cm-save-unreadable = Salvestust ei õnnestunud lugeda: { $error }
cm-save-undeletable = Salvestust ei õnnestunud kustutada: { $error }
cm-in-vr = VR-IS
cm-delete =
    .facts = Kustuta see salvestatud kampaania

## Sõjakaart

pic-friendly-aircraft = Oma õhusõiduk
pic-friendly-helicopter = Oma helikopter
pic-enemy-fighter = Vaenlase hävitaja (radaril)
pic-sam-known = SAM-positsioon (teada)
pic-guns-known = Kahurid / MANPADS (teada)
pic-armour = Tank / jalaväe lahingumasin (vaenlane: nähtaval)
pic-patriot-reach = Patrioti ulatus
pic-gepard-stinger = Gepard / Stinger
pic-missile-range = SAM-rakettide ulatus
pic-engagement-range = Tabamisulatus
pic-reconned = Luuratud
pic-you = Sina
pic-site = Positsioon: { $place }
uv-live = { $unit } - OTSE
uv-contact-lost = KONTAKT KAOTATUD
uv-hint = Lohista: tiirle  Ratas: suum

## Mitmikmäng

mp-disconnected = Ühendus katkes: { $why }
mp-connecting = Ühendan: { $address }...
mp-joining = Ühendatud: { $address }, liitun...
mp-browser-public-only = brauserist saab liituda ainult avaliku nimekirja serveritega
mp-title-host = MITMIKMÄNG: KORRALDA MÄNG
mp-subtitle = Koostöö dünaamilises kampaanias: lenda sõda koos
mp-your-name = Sinu nimi
    .hint = Piloodi nimi
mp-join-address = Liitu aadressi järgi
    .hint = aadress või nimi[:port]
mp-connect = ÜHENDU
mp-server-name = Serveri nimi
mp-udp-port = UDP port
mp-address-note = Teised mängijad liituvad selle arvuti aadressiga; interneti kaudu mängimiseks ava ruuteris UDP port
mp-list-public = Lisa avalikku serverite nimekirja (teised leiavad mängu oma serverite nimekirjast)
mp-rules = Reeglid kõigile
mp-time-of-day = Kellaaeg (kuuvalged ööd)
mp-realistic-weapons = Realistlikud F-117A relvad
mp-friendly-fire = Oma tuli
mp-day-night = Päev ja öö:
mp-weather = Ilm:
mp-host-new = KORRALDA UUS
mp-host-saved = Korralda salvestatud kampaania
mp-host = KORRALDA
mp-host-game = KORRALDA MÄNG
mp-port-error = UDP port { $port }: { $error }
mp-default-server = Kampaania: { $pilot }
mp-public-servers = Avalikud serverid
mp-refresh = VÄRSKENDA
mp-server-count = { $servers } serverit, { $pilots } piloti
mp-list-error = Serverite nimekiri pole kättesaadav: { $error }
mp-no-servers = Praegu avalikke servereid pole
mp-col-server = SERVER
mp-col-pilots = PILOOTE
mp-col-war = SÕDA
mp-dedicated = OMAETTE
mp-other-version = teine versioon
mp-join = LIITU
