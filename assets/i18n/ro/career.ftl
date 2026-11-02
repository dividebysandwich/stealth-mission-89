# Cariera piloților (src/career.rs): grade, decorații, perioada de serviciu.

## Gradele US Air Force (traduse prin echivalentele din Forțele Aeriene Române), abrevieri

rank-0 = Sublocotenent
    .short = slt.
rank-1 = Locotenent
    .short = lt.
rank-2 = Căpitan
    .short = cpt.
rank-3 = Maior
    .short = mr.
rank-4 = Locotenent-colonel
    .short = lt.col.
rank-5 = Colonel
    .short = col.

## Decorații (medalii americane: numele lor proprii)

medal-air-medal = Air Medal
medal-distinguished-flying-cross = Distinguished Flying Cross
medal-silver-star = Silver Star
medal-air-force-cross = Air Force Cross
medal-medal-of-honor = Medal of Honor
medal-purple-heart = Purple Heart
medal-prisoner-of-war-medal = Prisoner of War Medal

## Știri după un zbor

career-awarded = Decorație primită: { $medal }
career-kia = Căzut în luptă
    .short = CL
career-pow = Prizonier de război
    .short = PR
career-active = În serviciu activ
career-captured = Luat prizonier: cariera s-a încheiat
career-tour-sortie = Perioadă de serviciu: zborul { $sortie } din { $length }
career-tour-complete = Perioadă de serviciu încheiată: { $points } puncte (+{ $bonus })
career-promoted = Avansat la gradul: { $rank }

## Fereastra carierei

career-title = DOSARELE PILOȚILOR
career-group-tour = Perioadă
career-set-by-tour = Stabilit de perioada de serviciu a pilotului
career-pilots = Piloți
career-new-pilot = Pilot nou
career-name = Nume
career-enlist = ÎNROLEAZĂ
career-select = Alegeți sau înrolați un pilot
career-service-record = Stat de serviciu
career-rank = GRAD
career-points = PUNCTE
career-sorties = ZBORURI
career-tours = PERIOADE
career-status = STARE
career-decorations = Decorații
career-tour-of-duty = Perioadă de serviciu
career-tour-line = { $theater }  { $situation }  zborul { $sortie } din { $length }  { $points } puncte
career-destroyed = Obiective distruse până acum: { $count }
career-fly-next = URMĂTORUL ZBOR
career-new-tour = Zboruri: { $count }, teatru de operații: { $theater }, situație: { $situation }, adversari: { $opponents }
    .facts = Ce distrugeți rămâne distrus până la sfârșitul perioadei de serviciu
career-start-tour = ÎNCEPE PERIOADA
career-retired = Retras din serviciul activ
career-last-sorties = Ultimele zboruri
