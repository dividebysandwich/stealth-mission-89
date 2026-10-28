# La planchette (police vectorielle), le planificateur de mission et les missions enregistrées.
# Les abréviations de navigation (PT, BRG, ETA, PRI, SEC, KM, KT) restent en anglais.

kb-brief = BRIEF
kb-route = ROUTE
kb-map = CARTE
kb-target = CIBLE
kb-keys = TOUCHES
kb-tip = Cartes, notes de mission et guide des touches
kb-geran-file = GERAN-2 (ARCHIVE)
kb-objectives = OBJECTIFS
kb-primary = PRINCIPAL : { $objective }
kb-secondary = SECONDAIRE : { $objective }
kb-times = HORAIRES
kb-deadline = LIMITE { $time }
kb-now = MAINTENANT { $time }
kb-conditions = CONDITIONS
kb-moon = LUNE : { $phase } ({ $lit } % ÉCLAIRÉE)
kb-armament = ARMEMENT
kb-roe = RÈGLES D'ENGAGEMENT
kb-total = TOTAL { $km } KM À { $kt } KT
kb-fuel = CARBURANT { $lbs } LB
kb-preparing = PRÉPARATION...
kb-airborne = EN VOL
kb-developing = DÉVELOPPEMENT...
kb-no-photo = PAS DE PHOTO
kb-airborne-target = CIBLE EN VOL
kb-scroll-vr = DÉFILER : STICK, DOIGT SUR LA PLANCHETTE
kb-scroll-desktop = DÉFILER : MOLETTE SUR LA PLANCHETTE

## Planificateur de mission

planner-low = BAS 500 FT
planner-medium = MOYEN 5000 FT
planner-high = HAUT 20000 FT
planner-title = PLANIFICATEUR DE MISSION
planner-route = Route
planner-base = BASE
planner-total = TOTAL { $km } KM  { $time }
planner-briefed = Route du briefing { $km } km, carburant { $fuel } lb
planner-coverage = Couverture radar pour
    .note = La couverture radar suppose un vol en palier à 480 kt, soutes fermées.
planner-detected = { $km } km de la route détectés
planner-pulse = Portée de détection radar à impulsions (P)
planner-doppler = Portée de détection radar Doppler (D)
planner-missile = Portée des missiles
planner-uav = Drone ennemi
planner-edit-hint = Glissez un point pour le déplacer, la carte pour la déplacer. Cliquez sur un segment pour ajouter un point, clic droit pour le supprimer. Molette : zoom.
planner-reset = RÉINITIALISER
planner-view-hint = Glissez la carte pour la déplacer. Molette : zoom.

## Missions enregistrées

saves-title = MISSIONS ENREGISTRÉES
saves-subtitle = Enregistrées en vol depuis le menu : ENREGISTRER LA MISSION
saves-none = Aucune mission enregistrée
saves-line = { $theater }  -  { $aircraft }  -  temps de mission { $time }
saves-saved-at = Enregistrée le { $date }
saves-unreadable = Sauvegarde illisible (autre version)
saves-delete =
    .facts = Supprimer cette sauvegarde
saves-back =
    .facts = Retour à l'écran de mission simple
