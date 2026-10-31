# Carrières des pilotes (src/career.rs) : grades, décorations, tour d'opérations.

## Grades de l'US Air Force (traduits par leurs équivalents), abréviations

rank-0 = Sous-lieutenant
    .short = S/Lt
rank-1 = Lieutenant
    .short = Lt
rank-2 = Capitaine
    .short = Cne
rank-3 = Commandant
    .short = Cdt
rank-4 = Lieutenant-colonel
    .short = Lcl
rank-5 = Colonel
    .short = Col

## Décorations (médailles américaines : leurs noms propres)

medal-air-medal = Air Medal
medal-distinguished-flying-cross = Distinguished Flying Cross
medal-silver-star = Silver Star
medal-air-force-cross = Air Force Cross
medal-medal-of-honor = Medal of Honor
medal-purple-heart = Purple Heart
medal-prisoner-of-war-medal = Prisoner of War Medal

## Nouvelles après une sortie

career-awarded = Décoré de la { $medal }
career-kia = Mort au combat
    .short = MAC
career-pow = Prisonnier de guerre
    .short = PG
career-active = En service actif
career-captured = Fait prisonnier : la carrière est terminée
career-tour-sortie = Tour d'opérations : sortie { $sortie } sur { $length }
career-tour-complete = Tour d'opérations terminé : { $points } points (+{ $bonus })
career-promoted = Promu { $rank }

## La fenêtre de carrière

career-title = DOSSIERS DES PILOTES
career-group-tour = Tour
career-set-by-tour = Fixé par le tour du pilote
career-pilots = Pilotes
career-new-pilot = Nouveau pilote
career-name = Nom
career-enlist = ENRÔLER
career-select = Sélectionnez ou enrôlez un pilote
career-service-record = État de service
career-rank = GRADE
career-points = POINTS
career-sorties = SORTIES
career-tours = TOURS
career-status = STATUT
career-decorations = Décorations
career-tour-of-duty = Tour d'opérations
career-tour-line = { $theater }  { $situation }  sortie { $sortie } sur { $length }  { $points } points
career-destroyed = { $count } objectifs détruits jusqu'ici
career-fly-next = VOLER LA SORTIE SUIVANTE
career-new-tour = { $count } sorties en { $theater }, { $situation }, contre des adversaires de niveau { $opponents }
    .facts = Ce que vous détruisez reste détruit jusqu'à la fin du tour
career-start-tour = COMMENCER LE TOUR
career-retired = Retiré du service actif
career-last-sorties = Dernières sorties
