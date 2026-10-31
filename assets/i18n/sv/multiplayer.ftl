# Kampanjfönstret (src/campaign/menu.rs), flerspelarfönstret (src/multiplayer_menu.rs),
# krigskartans teckenförklaring och liveläge (src/campaign/picture.rs, unit_view.rs).

delete = TA BORT
delete-campaign = TA BORT DENNA KAMPANJ?
delete-save = TA BORT DENNA SPARFIL?
delete-permanent = Borttagningen är permanent

## Kampanjfönstret

cm-group-campaign = Kampanj
cm-weather-dynamic = Dynamiskt
    .facts =
        Vädret ändras var 10-22 timmar enligt himlens klocka
        Moln blockerar sikten för båda sidor
cm-weather-clear = Klar himmel
    .facts = Inga moln under hela kampanjen
cm-section-day-night = Dag och natt
cm-day-real = Verklig tid
    .facts = Ett dygn varar 24 timmar
cm-day-2h = 2 h / dygn
    .facts = Ett dygn varar 2 timmar
cm-day-1h = 1 h / dygn
    .facts = Ett dygn varar 1 timme
cm-day-30min = 30 min / dygn
    .facts = Ett dygn varar 30 minuter
cm-conditions-note = Förhållandena gäller en ny kampanj
cm-rules-note = En sparad kampanj behåller reglerna den startades med
cm-about =
    Kriget pågår i realtid medan du spelar, även när menyn är öppen.
    Flyg uppdragen på tavlan: spaning, SEAD, CAP, CAS, sjömål, anfall, eskort.
    Helikopterinsättningar intar fiendens städer och flygfält.
    Du dödas eller tas aldrig till fånga: efter en flygning är du tillbaka vid tavlan.
cm-section-new = Ny kampanj
cm-new-line = { $theater }  -  { $situation }  -  motståndare { $opponents }
cm-start = STARTA
cm-section-continue = Fortsätt
cm-no-saves = Inga sparade kampanjer
cm-played = { $hours } h spelade
cm-save-gone = Sparfilen finns inte längre
cm-save-unreadable = Sparfilen kunde inte läsas: { $error }
cm-save-undeletable = Sparfilen kunde inte tas bort: { $error }
cm-in-vr = I VR
cm-delete =
    .facts = Ta bort denna sparade kampanj

## Krigskartan

pic-friendly-aircraft = Egna flygplan
pic-friendly-helicopter = Egen helikopter
pic-enemy-fighter = Fientligt jaktplan (på radar)
pic-sam-known = SAM-ställning (känd)
pic-guns-known = Luftvärnskanoner / MANPADS (kända)
pic-armour = Stridsvagn / stridsfordon (fiende: inom synhåll)
pic-patriot-reach = Patriots räckvidd
pic-gepard-stinger = Gepard / Stinger
pic-missile-range = SAM-robotarnas räckvidd
pic-engagement-range = Bekämpningsräckvidd
pic-reconned = Spanad
pic-you = Du
pic-site = Ställning: { $place }
uv-live = { $unit } - LIVE
uv-contact-lost = KONTAKT FÖRLORAD
uv-hint = Dra: kretsa  Hjul: zoom

## Flerspelare

mp-disconnected = Frånkopplad: { $why }
mp-connecting = Ansluter till { $address }...
mp-joining = Ansluten till { $address }, går med...
mp-browser-public-only = i webbläsaren kan man bara gå med i servrar på den publika listan
mp-title-host = FLERSPELARE: VÄRD FÖR ETT SPEL
mp-subtitle = Samarbete i den dynamiska kampanjen: flyg kriget tillsammans
mp-your-name = Ditt namn
    .hint = Pilotnamn
mp-join-address = Gå med via adress
    .hint = adress eller namn[:port]
mp-connect = ANSLUT
mp-server-name = Servernamn
mp-udp-port = UDP-port
mp-address-note = Andra spelare går med via den här datorns adress; öppna UDP-porten i routern för att spela över internet
mp-list-public = Visa i den publika serverlistan (andra hittar spelet i sin serverlista)
mp-rules = Regler för alla
mp-time-of-day = Tid på dygnet (månljusa nätter)
mp-realistic-weapons = Realistiska F-117A-vapen
mp-friendly-fire = Vådabeskjutning
mp-day-night = Dag och natt:
mp-weather = Väder:
mp-host-new = STARTA NYTT
mp-host-saved = Värd för en sparad kampanj
mp-host = VÄRD
mp-host-game = VÄRD FÖR ETT SPEL
mp-port-error = UDP-port { $port }: { $error }
mp-default-server = Kampanj: { $pilot }
mp-public-servers = Publika servrar
mp-refresh = UPPDATERA
mp-server-count = { $servers } servrar, { $pilots } piloter
mp-list-error = Serverlistan kan inte nås: { $error }
mp-no-servers = Inga publika servrar just nu
mp-col-server = SERVER
mp-col-pilots = PILOTER
mp-col-war = KRIG
mp-dedicated = DEDIKERAD
mp-other-version = annan version
mp-join = GÅ MED
