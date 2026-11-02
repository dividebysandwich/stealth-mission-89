# Carriere dei piloti (src/career.rs): gradi, decorazioni, turno di servizio.

## Gradi dell'US Air Force (tradotti con gli equivalenti dell'Aeronautica Militare), abbreviazioni

rank-0 = Sottotenente
    .short = S.Ten.
rank-1 = Tenente
    .short = Ten.
rank-2 = Capitano
    .short = Cap.
rank-3 = Maggiore
    .short = Magg.
rank-4 = Tenente colonnello
    .short = Ten.Col.
rank-5 = Colonnello
    .short = Col.

## Decorazioni (medaglie americane: i loro nomi propri)

medal-air-medal = Air Medal
medal-distinguished-flying-cross = Distinguished Flying Cross
medal-silver-star = Silver Star
medal-air-force-cross = Air Force Cross
medal-medal-of-honor = Medal of Honor
medal-purple-heart = Purple Heart
medal-prisoner-of-war-medal = Prisoner of War Medal

## Notizie dopo una sortita

career-awarded = Decorazione conferita: { $medal }
career-kia = Caduto in azione
    .short = KIA
career-pow = Prigioniero di guerra
    .short = POW
career-active = In servizio attivo
career-captured = Fatto prigioniero: la carriera è finita
career-tour-sortie = Turno di servizio: sortita { $sortie } di { $length }
career-tour-complete = Turno di servizio completato: { $points } punti (+{ $bonus })
career-promoted = Promosso { $rank }

## La finestra della carriera

career-title = FASCICOLI DEI PILOTI
career-group-tour = Turno
career-set-by-tour = Fissato dal turno del pilota
career-pilots = Piloti
career-new-pilot = Nuovo pilota
career-name = Nome
career-enlist = ARRUOLARE
career-select = Selezionare o arruolare un pilota
career-service-record = Stato di servizio
career-rank = GRADO
career-points = PUNTI
career-sorties = SORTITE
career-tours = TURNI
career-status = STATO
career-decorations = Decorazioni
career-tour-of-duty = Turno di servizio
career-tour-line = { $theater }  { $situation }  sortita { $sortie } di { $length }  { $points } punti
career-destroyed = { $count ->
    [one] { $count } obiettivo distrutto finora
   *[other] { $count } obiettivi distrutti finora
}
career-fly-next = VOLARE LA SORTITA SUCCESSIVA
career-new-tour = { $count } sortite, teatro { $theater }, { $situation }, contro avversari di livello { $opponents }
    .facts = Ciò che viene distrutto resta distrutto fino alla fine del turno
career-start-tour = INIZIARE IL TURNO
career-retired = Ritirato dal servizio attivo
career-last-sorties = Ultime sortite
