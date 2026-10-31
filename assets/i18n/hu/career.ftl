# Pilótakarrierek (src/career.rs): rendfokozatok, kitüntetések, szolgálati idő.

## Az amerikai légierő rendfokozatai (a magyar légierő megfelelőivel), rövidítések

rank-0 = Hadnagy
    .short = hdgy.
rank-1 = Főhadnagy
    .short = fhdgy.
rank-2 = Százados
    .short = szds.
rank-3 = Őrnagy
    .short = őrgy.
rank-4 = Alezredes
    .short = alez.
rank-5 = Ezredes
    .short = ezds.

## Kitüntetések (amerikai érmek: tulajdonnevek)

medal-air-medal = Air Medal
medal-distinguished-flying-cross = Distinguished Flying Cross
medal-silver-star = Silver Star
medal-air-force-cross = Air Force Cross
medal-medal-of-honor = Medal of Honor
medal-purple-heart = Purple Heart
medal-prisoner-of-war-medal = Prisoner of War Medal

## Hírek egy bevetés után

career-awarded = Kitüntetés: { $medal }
career-kia = Hősi halált halt
    .short = KIA
career-pow = Hadifogoly
    .short = POW
career-active = Aktív szolgálatban
career-captured = Fogságba esett: a karrier véget ért
career-tour-sortie = Szolgálati idő: { $sortie }. bevetés / { $length }
career-tour-complete = Szolgálati idő letelt: { $points } pont (+{ $bonus })
career-promoted = Előléptetés: { $rank }

## A karrierablak

career-title = PILÓTÁK NYILVÁNTARTÁSA
career-group-tour = Szolgálati idő
career-set-by-tour = A pilóta szolgálati ideje határozza meg
career-pilots = Pilóták
career-new-pilot = Új pilóta
career-name = Név
career-enlist = BESOROZÁS
career-select = Válasszon ki vagy sorozzon be egy pilótát
career-service-record = Szolgálati lap
career-rank = RENDFOKOZAT
career-points = PONTOK
career-sorties = BEVETÉSEK
career-tours = SZOLGÁLATI IDŐK
career-status = ÁLLAPOT
career-decorations = Kitüntetések
career-tour-of-duty = Szolgálati idő
career-tour-line = { $theater }  { $situation }  { $sortie }. bevetés / { $length }  { $points } pont
career-destroyed = Eddig { $count } célpont megsemmisítve
career-fly-next = KÖVETKEZŐ BEVETÉS
career-new-tour = { $count } bevetés itt: { $theater }, { $situation }, ellenfelek: { $opponents }
    .facts = Amit megsemmisít, megsemmisítve marad a szolgálati idő végéig
career-start-tour = SZOLGÁLATI IDŐ INDÍTÁSA
career-retired = Aktív szolgálatból nyugállományba vonult
career-last-sorties = Utolsó bevetések
