# La finestra della campagna, la finestra multigiocatore, la legenda della mappa di guerra e
# la vista in diretta.

delete = ELIMINARE
delete-campaign = ELIMINARE QUESTA CAMPAGNA?
delete-save = ELIMINARE QUESTO SALVATAGGIO?
delete-permanent = L'eliminazione è definitiva

## Finestra della campagna

cm-group-campaign = Campagna
cm-weather-dynamic = Dinamico
    .facts =
        Il meteo cambia ogni 10-22 ore dell'orologio del cielo
        Le nuvole bloccano la visuale a entrambi gli schieramenti
cm-weather-clear = Cielo sereno
    .facts = Nessuna nuvola per tutta la campagna
cm-section-day-night = Giorno e notte
cm-day-real = Tempo reale
    .facts = Un giorno dura 24 ore
cm-day-2h = 2 h / giorno
    .facts = Un giorno dura 2 ore
cm-day-1h = 1 h / giorno
    .facts = Un giorno dura 1 ora
cm-day-30min = 30 min / giorno
    .facts = Un giorno dura 30 minuti
cm-conditions-note = Le condizioni si applicano a una nuova campagna
cm-rules-note = Una campagna salvata mantiene le regole con cui è iniziata
cm-about =
    La guerra prosegue in tempo reale mentre si gioca, anche con il menu aperto.
    Volare le missioni della bacheca: ricognizione, SEAD, CAP, CAS, antinave, attacco, scorta.
    Le inserzioni con elicottero conquistano le città e gli aeroporti nemici.
    Lei non viene mai ucciso né catturato: dopo una sortita si torna alla bacheca.
cm-section-new = Nuova campagna
cm-new-line = { $theater }  -  { $situation }  -  avversari { $opponents }
cm-start = INIZIARE
cm-section-continue = Continuare
cm-no-saves = Nessuna campagna salvata
cm-played = { $hours } h giocate
cm-save-gone = Il salvataggio non esiste più
cm-save-unreadable = Impossibile leggere il salvataggio: { $error }
cm-save-undeletable = Impossibile eliminare il salvataggio: { $error }
cm-in-vr = IN VR
cm-delete =
    .facts = Eliminare questa campagna salvata

## Mappa di guerra

pic-friendly-aircraft = Aereo amico
pic-friendly-helicopter = Elicottero amico
pic-enemy-fighter = Caccia nemico (al radar)
pic-sam-known = Sito SAM (noto)
pic-guns-known = Cannoni / MANPADS (noti)
pic-armour = Carro / VCI (nemico: in vista)
pic-patriot-reach = Portata dei Patriot
pic-gepard-stinger = Gepard / Stinger
pic-missile-range = Portata dei missili SAM
pic-engagement-range = Portata d'ingaggio
pic-reconned = Ricognito
pic-you = Lei
pic-site = Sito di { $place }
uv-live = { $unit } - IN DIRETTA
uv-contact-lost = CONTATTO PERSO
uv-hint = Trascinare: orbita  Rotellina: zoom

## Multigiocatore

mp-disconnected = Disconnesso: { $why }
mp-connecting = Connessione a { $address }...
mp-joining = Connesso a { $address }, ingresso in partita...
mp-browser-public-only = dal browser si può accedere solo ai server della lista pubblica
mp-title-host = MULTIGIOCATORE: OSPITARE UNA PARTITA
mp-subtitle = Cooperativa nella campagna dinamica: combattere la guerra insieme
mp-your-name = Il Suo nome
    .hint = Nome del pilota
mp-join-address = Unirsi tramite indirizzo
    .hint = indirizzo o nome[:porta]
mp-connect = CONNETTERSI
mp-server-name = Nome del server
mp-udp-port = Porta UDP
mp-address-note = Gli altri giocatori si uniscono con l'indirizzo di questo computer; aprire la porta UDP nel router per giocare via internet
mp-list-public = Pubblicare nella lista pubblica dei server (gli altri trovano la partita nella loro lista)
mp-rules = Regole per tutti
mp-time-of-day = Ora del giorno (notti di luna)
mp-realistic-weapons = Armamento realistico dell'F-117A
mp-friendly-fire = Fuoco amico
mp-day-night = Giorno e notte:
mp-weather = Meteo:
mp-host-new = OSPITARE UNA NUOVA
mp-host-saved = Ospitare una campagna salvata
mp-host = OSPITARE
mp-host-game = OSPITARE UNA PARTITA
mp-port-error = Porta UDP { $port }: { $error }
mp-default-server = Campagna di { $pilot }
mp-public-servers = Server pubblici
mp-refresh = AGGIORNARE
mp-server-count = { $servers } server, { $pilots } piloti
mp-list-error = La lista dei server non è raggiungibile: { $error }
mp-no-servers = Nessun server pubblico al momento
mp-col-server = SERVER
mp-col-pilots = PILOTI
mp-col-war = GUERRA
mp-dedicated = DEDICATO
mp-other-version = altra versione
mp-join = UNIRSI
