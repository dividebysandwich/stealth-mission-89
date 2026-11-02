# Pilot careers (src/career.rs): ranks, decorations, the tour of duty.

## US Air Force ranks (translated in every language), their abbreviations

rank-0 = Second Lieutenant
    .short = 2nd Lt.
rank-1 = First Lieutenant
    .short = 1st Lt.
rank-2 = Captain
    .short = Capt.
rank-3 = Major
    .short = Maj.
rank-4 = Lieutenant Colonel
    .short = Lt. Col.
rank-5 = Colonel
    .short = Col.

## Decorations (US medals: their proper names)

medal-air-medal = Air Medal
medal-distinguished-flying-cross = Distinguished Flying Cross
medal-silver-star = Silver Star
medal-air-force-cross = Air Force Cross
medal-medal-of-honor = Medal of Honor
medal-purple-heart = Purple Heart
medal-prisoner-of-war-medal = Prisoner of War Medal

## News after a sortie

career-awarded = Awarded the { $medal }
career-kia = Killed in action
    .short = KIA
career-pow = Prisoner of war
    .short = POW
career-active = Active duty
career-captured = Taken prisoner: the career is over
career-tour-sortie = Tour of duty: sortie { $sortie } of { $length }
career-tour-complete = Tour of duty complete: { $points } points (+{ $bonus })
career-promoted = Promoted to { $rank }

## The career window

career-title = PILOT RECORDS
career-group-tour = Tour
career-set-by-tour = Set by the pilot's tour
career-pilots = Pilots
career-new-pilot = New pilot
career-name = Name
career-enlist = ENLIST
career-select = Select or enlist a pilot
career-service-record = Service record
career-rank = RANK
career-points = POINTS
career-sorties = SORTIES
career-tours = TOURS
career-status = STATUS
career-decorations = Decorations
career-tour-of-duty = Tour of duty
career-tour-line = { $theater }  { $situation }  sortie { $sortie } of { $length }  { $points } points
career-destroyed = { $count } targets destroyed so far
career-fly-next = FLY NEXT SORTIE
career-new-tour = { $count } sorties in { $theater }, { $situation }, against { $opponents } opponents
    .facts = What you destroy stays destroyed for the rest of the tour
career-start-tour = START TOUR
career-retired = Retired from active duty
career-last-sorties = Last sorties
