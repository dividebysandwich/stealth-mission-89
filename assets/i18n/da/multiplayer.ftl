# Kampagnevinduet (src/campaign/menu.rs), multiplayervinduet (src/multiplayer_menu.rs),
# krigskortets signaturforklaring og live-visning (src/campaign/picture.rs, unit_view.rs).

delete = SLET
delete-campaign = SLET DENNE KAMPAGNE?
delete-save = SLET DETTE SPIL?
delete-permanent = Sletning er permanent

## Kampagnevindue

cm-group-campaign = Kampagne
cm-weather-dynamic = Dynamisk
    .facts =
        Vejret skifter hver 10-22. time på himlens ur
        Skyer blokerer sigtelinjen for begge sider
cm-weather-clear = Klar himmel
    .facts = Ingen skyer i hele kampagnen
cm-section-day-night = Dag og nat
cm-day-real = Realtid
    .facts = En dag varer 24 timer
cm-day-2h = 2 t / dag
    .facts = En dag varer 2 timer
cm-day-1h = 1 t / dag
    .facts = En dag varer 1 time
cm-day-30min = 30 min / dag
    .facts = En dag varer 30 minutter
cm-conditions-note = Betingelserne gælder for en ny kampagne
cm-rules-note = En gemt kampagne beholder de regler, den blev startet med
cm-about =
    Krigen kører i realtid, mens du spiller, også mens menuen er åben.
    Flyv missionerne på tavlen: recon, SEAD, CAP, CAS, antiskib, angreb, eskorte.
    Helikopterindsættelser indtager fjendens byer og flyvepladser.
    Du bliver aldrig dræbt eller fanget: efter en flyvning er du tilbage ved tavlen.
cm-section-new = Ny kampagne
cm-new-line = { $theater }  -  { $situation }  -  modstandere { $opponents }
cm-start = START
cm-section-continue = Fortsæt
cm-no-saves = Ingen gemte kampagner
cm-played = { $hours } t spillet
cm-save-gone = Det gemte spil er væk
cm-save-unreadable = Det gemte spil kunne ikke læses: { $error }
cm-save-undeletable = Det gemte spil kunne ikke slettes: { $error }
cm-in-vr = I VR
cm-delete =
    .facts = Slet denne gemte kampagne

## Krigskort

pic-friendly-aircraft = Egne fly
pic-friendly-helicopter = Egen helikopter
pic-enemy-fighter = Fjendtligt jagerfly (på radar)
pic-sam-known = SAM-stilling (kendt)
pic-guns-known = Kanoner / MANPADS (kendt)
pic-armour = Kampvogn / pansret mandskabsvogn (fjende: inden for syne)
pic-patriot-reach = Patriots rækkevidde
pic-gepard-stinger = Gepard / Stinger
pic-missile-range = SAM-missilernes rækkevidde
pic-engagement-range = Bekæmpelsesrækkevidde
pic-reconned = Opklaret
pic-you = Dig
pic-site = Stedet { $place }
uv-live = { $unit } - LIVE
uv-contact-lost = KONTAKT TABT
uv-hint = Træk: kredsløb  Hjul: zoom

## Multiplayer

mp-disconnected = Frakoblet: { $why }
mp-connecting = Forbinder til { $address }...
mp-joining = Forbundet til { $address }, tilslutter...
mp-browser-public-only = fra browseren kan man kun tilslutte sig servere på den offentlige liste
mp-title-host = MULTIPLAYER: VÆR VÆRT FOR ET SPIL
mp-subtitle = Coop i den dynamiske kampagne: flyv krigen sammen
mp-your-name = Dit navn
    .hint = Pilotnavn
mp-join-address = Tilslut via adresse
    .hint = adresse eller navn[:port]
mp-connect = FORBIND
mp-server-name = Servernavn
mp-udp-port = UDP-port
mp-address-note = De andre spillere tilslutter sig med denne computers adresse; åbn UDP-porten i din router for at spille over internettet
mp-list-public = Vis på den offentlige serverliste (andre finder spillet i deres serverliste)
mp-rules = Regler for alle
mp-time-of-day = Tid på dagen (måneskinsnætter)
mp-realistic-weapons = Realistiske F-117A-våben
mp-friendly-fire = Egenbeskydning
mp-day-night = Dag og nat:
mp-weather = Vejr:
mp-host-new = HOST NY
mp-host-saved = Host en gemt kampagne
mp-host = HOST
mp-host-game = HOST ET SPIL
mp-port-error = UDP-port { $port }: { $error }
mp-default-server = { $pilot }s kampagne
mp-public-servers = Offentlige servere
mp-refresh = OPDATÉR
mp-server-count = { $servers } servere, { $pilots } piloter
mp-list-error = Serverlisten kan ikke nås: { $error }
mp-no-servers = Ingen offentlige servere lige nu
mp-col-server = SERVER
mp-col-pilots = PILOTER
mp-col-war = KRIG
mp-dedicated = DEDIKERET
mp-other-version = anden version
mp-join = TILSLUT
