# Il kneeboard (font vettoriale), il pianificatore di missione e le missioni salvate.
# Le abbreviazioni di navigazione (PT, BRG, ETA, PRI, SEC, KM, KT) restano in inglese.

kb-brief = BRIEF
kb-route = ROTTA
kb-map = MAPPA
kb-target = BERSAGLIO
kb-keys = TASTI
kb-tip = Mappe, note di missione e guida ai tasti
kb-geran-file = GERAN-2 (SCHEDA)
kb-objectives = OBIETTIVI
kb-primary = PRIMARIO: { $objective }
kb-secondary = SECONDARIO: { $objective }
kb-times = ORARI
kb-deadline = LIMITE { $time }
kb-now = ORA { $time }
kb-conditions = CONDIZIONI
kb-moon = LUNA: { $phase } ({ $lit }% ILLUMINATA)
kb-armament = ARMAMENTO
kb-roe = REGOLE DI INGAGGIO
kb-total = TOTALE { $km } KM A { $kt } KT
kb-fuel = CARBURANTE { $lbs } LBS
kb-preparing = PREPARAZIONE...
kb-airborne = IN VOLO
kb-developing = SVILUPPO...
kb-no-photo = NESSUNA FOTO
kb-airborne-target = BERSAGLIO IN VOLO
kb-scroll-vr = SCORRERE: THUMBSTICK, DITO SUL KNEEBOARD
kb-scroll-desktop = SCORRERE: ROTELLINA SUL KNEEBOARD

## Pianificatore di missione

planner-low = BASSA 500 FT
planner-medium = MEDIA 5000 FT
planner-high = ALTA 20000 FT
planner-title = PIANIFICATORE DI MISSIONE
planner-route = Rotta
planner-base = BASE
planner-total = TOTALE { $km } KM  { $time }
planner-briefed = Rotta del briefing { $km } km, carburante { $fuel } lbs
planner-coverage = Copertura radar per
    .note = La copertura radar presuppone un volo livellato a 480 kt, portelloni della stiva chiusi.
planner-detected = { $km } km della rotta rilevati
planner-pulse = Portata di rilevamento radar a impulsi (P)
planner-doppler = Portata di rilevamento radar Doppler (D)
planner-missile = Portata dei missili
planner-uav = UAV nemico
planner-edit-hint = Trascinare un punto per spostarlo, la mappa per scorrerla. Cliccare su una tratta per aggiungere un punto, clic destro per rimuoverlo. Rotellina: zoom.
planner-reset = REIMPOSTARE
planner-view-hint = Trascinare la mappa per scorrerla. Rotellina: zoom.
planner-layers = LIVELLI
planner-layer-sams = Sito radar / SAM
planner-radar-types = Tipi di radar
planner-layer-pulse = Portata di rilevamento radar a impulsi
planner-layer-doppler = Portata di rilevamento radar Doppler
planner-layer-airfield = Aeroporto

## Missioni salvate

saves-title = MISSIONI SALVATE
saves-subtitle = Salvate in volo dal menu: SALVARE LA MISSIONE
saves-none = Nessuna missione salvata
saves-line = { $theater }  -  { $aircraft }  -  tempo di missione { $time }
saves-saved-at = Salvata il { $date }
saves-unreadable = Salvataggio illeggibile (altra versione)
saves-delete =
    .facts = Eliminare questo salvataggio
saves-back =
    .facts = Tornare alla schermata della missione singola
