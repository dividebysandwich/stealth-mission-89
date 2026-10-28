# It-tieqa tal-kampanja (src/campaign/menu.rs), it-tieqa tal-multiplayer (src/multiplayer_menu.rs),
# il-leġenda u l-veduta ħajja tal-mappa tal-gwerra (src/campaign/picture.rs, unit_view.rs).

delete = ĦASSAR
delete-campaign = TĦASSAR DIN IL-KAMPANJA?
delete-save = TĦASSAR DIN IL-ĦAŻNA?
delete-permanent = It-tħassir huwa permanenti

## Tieqa tal-kampanja

cm-group-campaign = Kampanja
cm-weather-dynamic = Dinamiku
    .facts =
        It-temp jinbidel kull 10-22 siegħa tal-arloġġ tas-sema
        Is-sħab jimblokka l-linja tal-vista taż-żewġ naħat
cm-weather-clear = Sema ċar
    .facts = L-ebda sħab matul il-kampanja kollha
cm-section-day-night = Jum u lejl
cm-day-real = Ħin reali
    .facts = Jum idum 24 siegħa
cm-day-2h = 2 h / jum
    .facts = Jum idum sagħtejn
cm-day-1h = 1 h / jum
    .facts = Jum idum siegħa
cm-day-30min = 30 min / jum
    .facts = Jum idum 30 minuta
cm-conditions-note = Il-kundizzjonijiet japplikaw għal kampanja ġdida
cm-rules-note = Kampanja maħżuna żżomm ir-regoli li biha nbdiet
cm-about =
    Il-gwerra tgħaddi f'ħin reali waqt li tilgħab, anki meta l-menu jkun miftuħ.
    Ttir il-missjonijiet fuq il-bord: recon, SEAD, CAP, CAS, anti-ship, strike, escort.
    L-inserzjonijiet bil-ħelikopters jieħdu l-ibliet u l-ajrudromi tal-għadu.
    Qatt ma tinqatel jew tinqabad: wara titjira terġa' lura fuq il-bord.
cm-section-new = Kampanja ġdida
cm-new-line = { $theater }  -  { $situation }  -  avversarji { $opponents }
cm-start = ABDA
cm-section-continue = Kompli
cm-no-saves = L-ebda kampanja maħżuna
cm-played = { $hours } h milgħuba
cm-save-gone = Il-ħażna m'għadhiex hemm
cm-save-unreadable = Il-ħażna ma setgħetx tinqara: { $error }
cm-save-undeletable = Il-ħażna ma setgħetx titħassar: { $error }
cm-in-vr = FIL-VR
cm-delete =
    .facts = Ħassar din il-kampanja maħżuna

## Mappa tal-gwerra

pic-friendly-aircraft = Inġenju tal-ajru ħabib
pic-friendly-helicopter = Ħelikopter ħabib
pic-enemy-fighter = Fighter tal-għadu (fuq ir-radar)
pic-sam-known = Sit SAM (magħruf)
pic-guns-known = Kanun / MANPADS (magħruf)
pic-reconned = Skoperti bir-recon
pic-you = Int
pic-site = Sit ta' { $place }
uv-live = { $unit } - LIVE
uv-contact-lost = KONTATT MIFQUD
uv-hint = Kaxkar: dur  Rota: zoom

## Multiplayer

mp-disconnected = Skonnettjat: { $why }
mp-connecting = Qed jikkonnettja ma' { $address }...
mp-joining = Konness ma' { $address }, qed jingħaqad...
mp-browser-public-only = mill-browser tista' tingħaqad biss ma' servers fil-lista pubblika
mp-title-host = MULTIPLAYER: OSPITA LOGĦBA
mp-subtitle = Coop fil-kampanja dinamika: ittiru l-gwerra flimkien
mp-your-name = Ismek
    .hint = Isem il-pilota
mp-join-address = Ingħaqad b'indirizz
    .hint = indirizz jew isem[:port]
mp-connect = IKKONNETTJA
mp-server-name = Isem is-server
mp-udp-port = Port UDP
mp-address-note = Plejers oħra jingħaqdu bl-indirizz ta' dan il-kompjuter; iftaħ il-port UDP fir-router biex tilgħab fuq l-internet
mp-list-public = Elenka fil-lista pubblika tas-servers (l-oħrajn isibu l-logħba fil-lista tas-servers)
mp-rules = Regoli għal kulħadd
mp-time-of-day = Ħin tal-jum (iljieli bil-qamar)
mp-realistic-weapons = Armi realistiċi tal-F-117A
mp-friendly-fire = Friendly fire
mp-day-night = Jum u lejl:
mp-weather = Temp:
mp-host-new = OSPITA ĠDID
mp-host-saved = Ospita kampanja maħżuna
mp-host = OSPITA
mp-host-game = OSPITA LOGĦBA
mp-port-error = Port UDP { $port }: { $error }
mp-default-server = Il-kampanja ta' { $pilot }
mp-public-servers = Servers pubbliċi
mp-refresh = AĠĠORNA
mp-server-count = { $servers } servers, { $pilots } piloti
mp-list-error = Il-lista tas-servers mhix aċċessibbli: { $error }
mp-no-servers = L-ebda server pubbliku bħalissa
mp-col-server = SERVER
mp-col-pilots = PILOTI
mp-col-war = GWERRA
mp-dedicated = DEDIKAT
mp-other-version = verżjoni oħra
mp-join = INGĦAQAD
