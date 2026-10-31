# La schermata Operazioni della campagna (src/campaign/ops.rs).

ops-thunderstorms = TEMPORALI
ops-wx-short = METEO { $now }, { $next } ALLE { $at }
ops-wx-line = { $weather } In evoluzione verso { $next } alle { $at }.
ops-clock = GIORNO { $day }  { $time }
ops-landed-at = ATTERRATO A { $base }
ops-ground-crew = Il personale di terra è pronto
ops-rearm = RIARMARE E RIFORNIRE
ops-end-sortie = TERMINARE LA SORTITA
ops-on-server = SU { $server }
ops-serving = SERVER
ops-tab-missions = MISSIONI
ops-tab-map = MAPPA DI GUERRA
ops-tab-air-defence = DIFESA AEREA
ops-tab-forces = FORZE
ops-tab-players = GIOCATORI
ops-tab-debrief = DEBRIEFING
ops-tab-log = REGISTRO
ops-tab-chat = CHAT
ops-blue = BLU { $count }
ops-red = ROSSO { $count }
ops-war-over = La guerra è finita
ops-running = Simulazione della guerra in corso
ops-save-exit = SALVARE E USCIRE
ops-disconnect = DISCONNETTERSI
ops-leave-local = SALVARE E LASCIARE LA CAMPAGNA?
    .text = La guerra si ferma finché non la si riprende. Gli aerei in volo vanno perduti; le inserzioni in corso ripartono dalla loro base.
ops-leave-host = SALVARE E SMETTERE DI OSPITARE LA CAMPAGNA?
    .text = I giocatori collegati vengono disconnessi e la guerra si ferma finché non la si riprende. Gli aerei in volo vanno perduti; le inserzioni in corso ripartono dalla loro base.
ops-leave-client = DISCONNETTERSI DAL SERVER?
    .text = La campagna prosegue sul server; qui non viene salvato nulla. La sortita in corso termina.
ops-unit-destroyed = { $unit } distrutto: ritorno alle operazioni

## Missioni

ops-section-missions = Missioni
ops-threat = MINACCIA { $threat }
ops-no-missions = Nessuna missione: il fronte è calmo
ops-section-underway = Inserzioni e incursioni
ops-enemy-liftoff = NEMICO a { $place }: decollo tra { $mins } min
ops-liftoff = { $place }: decollo tra { $mins } min
ops-enemy-helis = NEMICO a { $place }: { $count } elicotteri in avvicinamento
ops-helis = { $place }: { $count } elicotteri in rotta
ops-raid-sead = SEAD dell'IA a { $place }
ops-raid-drones = DRONI su { $place }
ops-raid = INCURSIONE sull'aeroporto di { $place }
ops-raid-launch = { $raid }: lancio tra { $mins } min
ops-raid-on-way = { $raid }: { $count } in rotta
ops-resupply-ai = RIFORNIMENTO ({ $unit }): UH-60 in rotta
ops-awacs = AWACS NEMICO in volo per altri { $mins } min
ops-down = A TERRA: { $pilot } - soccorso entro { $mins } min
ops-aboard = A BORDO: { $pilot } - sulla via del ritorno
ops-none-underway = Niente in corso
ops-ai-helicopter = Un elicottero dell'IA se ne occupa tra { $time } se nessun pilota di UH-60 lo fa
ops-defences = Difese: { $sams } siti SAM, minaccia { $threat }
ops-recon-intel = INTELLIGENCE RICOGNIZIONE
ops-order-insertion = ORDINARE L'INSERZIONE
    .facts = Volare la scorta per coprirli, oppure guidarli con l'UH-60
ops-from = Da
ops-planned-by-group = Pianificata dal gruppo
ops-stores = { $aircraft } - carichi: { $stores }
ops-fly-uh60 = VOLARE CON L'UH-60
ops-accept = ACCETTARE LA MISSIONE
ops-planning = IN PIANIFICAZIONE: { $pilots }
ops-flying = IN VOLO: { $pilots }
ops-package = PACKAGE con { $kinds }: completare due tipi qui entro { $mins } min, +{ $bonus } punti ciascuno
ops-sead-on-way = La SEAD dell'IA è in rotta
ops-request-sead = RICHIEDERE SEAD DELL'IA ({ $cost } di materiale, { $aircraft } aerei)
    .facts = Aerei d'attacco amici colpiscono un sito SAM di questo luogo prima di Lei
    .disabled = Materiale o aerei a terra insufficienti (vedere FORZE)

## Regole, materiale, forze

ops-rule-damage = danni realistici
ops-rule-guidance = guida realistica
ops-rule-no-crashes = protezione anti-schianto
ops-rule-time = tempo continuo
ops-rule-weapons = armamento realistico dell'F-117A
ops-rule-friendly-fire = fuoco amico
ops-rule-classic = classiche
ops-materiel = MATERIALE
ops-materiel-blue = BLU { $materiel } (+{ $income }/h)
ops-materiel-red = ROSSO { $materiel } (+{ $income }/h)
ops-friendly = AMICI
ops-enemy = NEMICI
ops-section-forces = Forze
ops-places-held = Luoghi controllati
ops-materiel-row = Materiale
ops-income = Entrate orarie
ops-income-from = di cui
ops-income-sources = { $base } + { $places } luoghi x { $per_place } + { $cities } città x { $per_city }
ops-fighters-ground = Caccia a terra
ops-helicopters-ground = Elicotteri a terra
ops-in-air = In volo
ops-in-air-value = { $fighters } caccia, { $strike } d'attacco, { $helicopters } elicotteri
ops-since-start = Dall'inizio
ops-earned = guadagnato
ops-repairs = riparazioni
ops-operations = sortite, incursioni, inserzioni
ops-new-aircraft = nuovi aerei
ops-lost = perdite (città, piloti)
ops-section-how = Come funziona
ops-how =
    Ogni schieramento guadagna materiale ogni ora: una quota fissa, di più per ogni luogo controllato e per ogni città intatta tra questi. Bombardare le città nemiche (INTERDIZIONE) ne riduce le entrate.
    L'IA paga tutto ciò che fa: riparazioni, ogni sortita dei caccia, incursioni e le proprie inserzioni. A corto di materiale, i suoi danni restano da riparare e i suoi aerei restano a terra.
    L'IA vola solo con gli aerei che possiede; se abbattuti sono perduti, se atterrati volano di nuovo. Ogni ora ogni schieramento ne acquista di nuovi finché il materiale lo consente.
    Lei non resta mai a corto di nulla: i Suoi aerei, le Sue basi (bombardate o no) e gli elicotteri dei Suoi ordini d'inserzione sono sempre disponibili.
ops-section-prices = Prezzi
ops-price-airfield = Riparazione aeroporto
ops-price-sam = Riparazione sito SAM
ops-price-city = Città ricostruita
ops-price-ship = Riparazione nave
ops-price-gun = Cannone o squadra MANPADS
ops-price-sortie = Sortita dei caccia
ops-price-raid = Incursione
ops-price-insertion = Inserzione dell'IA
ops-price-fighter = Nuovo caccia
ops-price-helicopter = Nuovo elicottero
ops-price-city-lost = Città distrutta (perdita)
ops-price-captured = Pilota catturato (perdita)

## Mappa di guerra

ops-click-to-watch = { $unit } - cliccare per seguirla
ops-section-legend = Legenda
ops-legend-friendly-airfield = Aeroporto amico
ops-legend-enemy-airfield = Aeroporto nemico
ops-legend-city = Città amica / nemica
ops-legend-airfield = Aeroporto amico / nemico
ops-legend-insertion = Rotta d'inserimento
ops-watch-hint = Cliccare su un'unità per seguirla in diretta
ops-map-hint = Rotellina: zoom, trascinare: spostare
ops-patriot = Batteria Patriot
ops-drone-center = Centro di comando droni
ops-manned-by = Servita da { $pilot }
ops-manned-by-other = Servita da un altro giocatore
ops-control = PRENDERE IL CONTROLLO
    .facts = Prendere la sua console

## Debriefing, fine, registro, chat

ops-no-sortie = Nessuna sortita ancora effettuata
ops-section-sortie = Sortita
ops-achieved = OBIETTIVO RAGGIUNTO
ops-not-achieved = OBIETTIVO NON RAGGIUNTO
ops-waiting-raid = IN ATTESA DELL'INCURSIONE
ops-waiting-insertion = IN ATTESA DELL'INSERZIONE
ops-flight-time = Tempo di volo { $mins } min
ops-kills = Abbattimenti: { $air } aerei, { $ground } a terra
ops-points = { $points } punti
ops-points-bonus = { $points } punti (bonus package +{ $bonus })
ops-campaign-points = { $rank } - { $points } punti campagna
ops-campaign-record = Campagna: { $sorties } sortite, { $won } obiettivi raggiunti
ops-section-meanwhile = Nel frattempo
ops-victory = VITTORIA
    .text = Il nemico non controlla più alcun aeroporto: il teatro è nostro.
ops-defeat = SCONFITTA
    .text = Non resta alcun aeroporto amico: il teatro è perduto.
ops-end-day = Giorno { $day }. Luoghi amici { $friendly }, luoghi nemici { $enemy }.
ops-next-campaign = LA PROSSIMA CAMPAGNA INIZIA TRA { $time }
ops-next-campaign-now = LA PROSSIMA CAMPAGNA STA INIZIANDO
ops-next-from-menu = Avviare la prossima campagna dal menu principale.
ops-section-log = Registro della campagna
ops-section-chat = Chat
ops-no-messages = Ancora nessun messaggio
ops-message = Messaggio
    .hint = Scrivere un messaggio, Invio per inviarlo
ops-send = INVIARE

## Briefing

ops-briefing = BRIEFING DI MISSIONE
ops-briefing-line = Da { $base } - { $km } km dall'obiettivo - { $aircraft } - carichi: { $stores }
ops-route-locked = Rotta bloccata: { $pilots } in volo
ops-route-threats = ROTTA E MINACCE
ops-take-off = DECOLLARE
ops-radars-shown = Radar mostrati: rilevati da aerei amici, ancora attivi

## Giocatori

ops-section-players = Giocatori
ops-col-name = NOME
ops-col-status = STATO
ops-col-sorties = SORTITE
ops-col-points = PUNTI
ops-col-pilot = PILOTA
ops-col-objectives = OBIETTIVI
ops-col-kills = ABB. ARIA/SUOLO
ops-col-medal = MIGLIORE MEDAGLIA
ops-you = (Lei)
ops-host = HOST
ops-status-planning = Pianifica: { $title }
ops-status-board = Alla bacheca delle missioni
ops-status-flying = In volo: { $title } - { $aircraft }
ops-status-reconnecting = Connessione persa - aereo mantenuto { $seconds } s
ops-flown = { $flown } ({ $won } riuscite)
ops-section-leaderboard = Classifica
ops-alone = In volo da solo. Per volare insieme, ospitare questa campagna da MULTIGIOCATORE nel menu principale.
ops-rules = Regole di questa campagna (fissate all'inizio): { $rules }
ops-no-sorties = Nessuna sortita ancora effettuata
