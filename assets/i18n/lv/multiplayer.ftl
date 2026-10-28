# Kampaņas logs (src/campaign/menu.rs), daudzspēlētāju logs (src/multiplayer_menu.rs),
# kara kartes apzīmējumi un tiešraides skats (src/campaign/picture.rs, unit_view.rs).

delete = DZĒST
delete-campaign = DZĒST ŠO KAMPAŅU?
delete-save = DZĒST ŠO SAGLABĀJUMU?
delete-permanent = Dzēšana ir neatgriezeniska

## Kampaņas logs

cm-group-campaign = Kampaņa
cm-weather-dynamic = Dinamiski
    .facts =
        Laikapstākļi mainās ik pēc 10-22 debesu pulksteņa stundām
        Mākoņi aizsedz redzamību abām pusēm
cm-weather-clear = Skaidras debesis
    .facts = Visu kampaņu bez mākoņiem
cm-section-day-night = Diena un nakts
cm-day-real = Reāllaiks
    .facts = Diena ilgst 24 stundas
cm-day-2h = 2 h / diena
    .facts = Diena ilgst 2 stundas
cm-day-1h = 1 h / diena
    .facts = Diena ilgst 1 stundu
cm-day-30min = 30 min / diena
    .facts = Diena ilgst 30 minūtes
cm-conditions-note = Apstākļi attiecas uz jaunu kampaņu
cm-rules-note = Saglabātā kampaņa saglabā noteikumus, ar kādiem tā sākta
cm-about =
    Karš norit reāllaikā, kamēr jūs spēlējat, arī tad, kad izvēlne ir atvērta.
    Lidojiet dēlī esošās misijas: izlūkošana, SEAD, CAP, CAS, pretkuģu, trieciens, pavadība.
    Helikopteru desanti ieņem ienaidnieka pilsētas un lidlaukus.
    Jūs nekad netiekat nogalināts vai sagūstīts: pēc lidojuma atgriežaties dēlī.
cm-section-new = Jauna kampaņa
cm-new-line = { $theater }  -  { $situation }  -  pretinieki { $opponents }
cm-start = SĀKT
cm-section-continue = Turpināt
cm-no-saves = Nav saglabātu kampaņu
cm-played = Nospēlētas { $hours } h
cm-save-gone = Saglabājuma vairs nav
cm-save-unreadable = Saglabājumu neizdevās nolasīt: { $error }
cm-save-undeletable = Saglabājumu neizdevās izdzēst: { $error }
cm-in-vr = VR
cm-delete =
    .facts = Dzēst šo saglabāto kampaņu

## Kara karte

pic-friendly-aircraft = Draudzīga lidmašīna
pic-friendly-helicopter = Draudzīgs helikopters
pic-enemy-fighter = Ienaidnieka iznīcinātājs (uz radara)
pic-sam-known = SAM pozīcija (zināma)
pic-guns-known = Lielgabali / MANPADS (zināmi)
pic-reconned = Izlūkots
pic-you = Jūs
pic-site = Pozīcija: { $place }
uv-live = { $unit } - TIEŠRAIDE
uv-contact-lost = KONTAKTS ZAUDĒTS
uv-hint = Vilkšana: griezt  Ritenītis: tālummaiņa

## Daudzspēlētāju režīms

mp-disconnected = Atvienots: { $why }
mp-connecting = Savienojas ar { $address }...
mp-joining = Savienots ar { $address }, pievienojas...
mp-browser-public-only = no pārlūka var pievienoties tikai publiskajā sarakstā esošiem serveriem
mp-title-host = MULTIPLAYER: RĪKOT SPĒLI
mp-subtitle = Kopīga spēle dinamiskajā kampaņā: lidojiet karā kopā
mp-your-name = Jūsu vārds
    .hint = Pilota vārds
mp-join-address = Pievienoties pēc adreses
    .hint = adrese vai vārds[:ports]
mp-connect = SAVIENOTIES
mp-server-name = Servera nosaukums
mp-udp-port = UDP ports
mp-address-note = Citi spēlētāji pievienojas ar šī datora adresi; lai spēlētu internetā, atveriet UDP portu savā maršrutētājā
mp-list-public = Iekļaut publiskajā serveru sarakstā (citi atrod spēli savā serveru sarakstā)
mp-rules = Noteikumi visiem
mp-time-of-day = Diennakts laiks (mēness apgaismotas naktis)
mp-realistic-weapons = Reālistisks F-117A bruņojums
mp-friendly-fire = Draudzīgā uguns
mp-day-night = Diena un nakts:
mp-weather = Laikapstākļi:
mp-host-new = RĪKOT JAUNU
mp-host-saved = Rīkot saglabātu kampaņu
mp-host = RĪKOT
mp-host-game = RĪKOT SPĒLI
mp-port-error = UDP ports { $port }: { $error }
mp-default-server = Kampaņa: { $pilot }
mp-public-servers = Publiskie serveri
mp-refresh = ATJAUNOT
mp-server-count = { $servers } serveri, { $pilots } piloti
mp-list-error = Serveru saraksts nav pieejams: { $error }
mp-no-servers = Pašlaik nav publisku serveru
mp-col-server = SERVERIS
mp-col-pilots = PILOTI
mp-col-war = KARŠ
mp-dedicated = SPECIĀLAIS
mp-other-version = cita versija
mp-join = PIEVIENOTIES
