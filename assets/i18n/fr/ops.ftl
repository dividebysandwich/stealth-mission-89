# L'écran des opérations de la campagne (src/campaign/ops.rs).

ops-thunderstorms = ORAGES
ops-wx-short = MÉTÉO { $now }, { $next } À { $at }
ops-wx-line = { $weather } Passage à { $next } à { $at }.
ops-clock = JOUR { $day }  { $time }
ops-landed-at = POSÉ À { $base }
ops-ground-crew = L'équipe au sol est prête
ops-rearm = RÉARMER ET RAVITAILLER
ops-end-sortie = TERMINER LA SORTIE
ops-on-server = SUR { $server }
ops-serving = SERVEUR
ops-tab-missions = MISSIONS
ops-tab-map = CARTE DE GUERRE
ops-tab-air-defence = DÉFENSE AÉRIENNE
ops-tab-forces = FORCES
ops-tab-players = JOUEURS
ops-tab-debrief = DÉBRIEFING
ops-tab-log = JOURNAL
ops-tab-chat = CHAT
ops-blue = BLEU { $count }
ops-red = ROUGE { $count }
ops-war-over = La guerre est finie
ops-running = Simulation de la guerre en cours
ops-save-exit = ENREGISTRER ET QUITTER
ops-disconnect = SE DÉCONNECTER
ops-leave-local = ENREGISTRER ET QUITTER LA CAMPAGNE ?
    .text = La guerre s'arrête jusqu'à ce que vous la repreniez. Les appareils en vol sont perdus ; les insertions en cours repartent de leur base.
ops-leave-host = ENREGISTRER ET ARRÊTER DE SERVIR LA CAMPAGNE ?
    .text = Les joueurs connectés sont déconnectés et la guerre s'arrête jusqu'à ce que vous la repreniez. Les appareils en vol sont perdus ; les insertions en cours repartent de leur base.
ops-leave-client = SE DÉCONNECTER DU SERVEUR ?
    .text = La campagne continue sur le serveur ; rien n'est enregistré ici. Votre sortie se termine.
ops-unit-destroyed = { $unit } détruit : retour aux opérations

## Missions

ops-section-missions = Missions
ops-threat = MENACE { $threat }
ops-no-missions = Aucune mission : le front est calme
ops-section-underway = Insertions et raids
ops-enemy-liftoff = ENNEMI à { $place } : décollage dans { $mins } min
ops-liftoff = { $place } : décollage dans { $mins } min
ops-enemy-helis = ENNEMI à { $place } : { $count } hélicoptères en approche
ops-helis = { $place } : { $count } hélicoptères en route
ops-raid-sead = SEAD de l'IA à { $place }
ops-raid-drones = DRONES sur { $place }
ops-raid = RAID sur l'aérodrome de { $place }
ops-raid-launch = { $raid } : lancement dans { $mins } min
ops-raid-on-way = { $raid } : { $count } en route
ops-resupply-ai = RAVITAILLEMENT ({ $unit }) : UH-60 en route
ops-awacs = AWACS ENNEMI en vol encore { $mins } min
ops-down = AU SOL : { $pilot } - sauvetage dans les { $mins } min
ops-aboard = À BORD : { $pilot } - sur le chemin du retour
ops-none-underway = Rien en cours
ops-ai-helicopter = Un hélicoptère de l'IA s'en charge dans { $time } si aucun pilote de UH-60 ne le fait
ops-defences = Défenses : { $sams } sites SAM, menace { $threat }
ops-recon-intel = RENSEIGNEMENT RECO
ops-order-insertion = ORDONNER L'INSERTION
    .facts = Volez l'escorte pour les couvrir, ou menez-les en UH-60
ops-from = Depuis
ops-planned-by-group = Planifié par le groupe
ops-stores = { $aircraft } - emports : { $stores }
ops-fly-uh60 = VOLER LE UH-60
ops-accept = ACCEPTER LA MISSION
ops-planning = PLANIFIE : { $pilots }
ops-flying = EN VOL : { $pilots }
ops-package = PACKAGE avec { $kinds } : réussissez deux types ici en { $mins } min, +{ $bonus } points chacun
ops-sead-on-way = La SEAD de l'IA est en route
ops-request-sead = DEMANDER UNE SEAD DE L'IA ({ $cost } de matériel, { $aircraft } avions)
    .facts = Des avions d'attaque amis frappent un site SAM de ce lieu avant vous
    .disabled = Pas assez de matériel ou d'avions au sol (voir FORCES)

## Règles, matériel, forces

ops-rule-damage = dégâts réalistes
ops-rule-guidance = guidage réaliste
ops-rule-no-crashes = protection anti-crash
ops-rule-time = temps continu
ops-rule-weapons = armement réaliste du F-117A
ops-rule-friendly-fire = tirs fratricides
ops-rule-classic = classiques
ops-materiel = MATÉRIEL
ops-materiel-blue = BLEU { $materiel } (+{ $income }/h)
ops-materiel-red = ROUGE { $materiel } (+{ $income }/h)
ops-friendly = AMIS
ops-enemy = ENNEMIS
ops-section-forces = Forces
ops-places-held = Lieux tenus
ops-materiel-row = Matériel
ops-income = Revenu par heure
ops-income-from = dont
ops-income-sources = { $base } + { $places } lieux x { $per_place } + { $cities } villes x { $per_city }
ops-fighters-ground = Chasseurs au sol
ops-helicopters-ground = Hélicoptères au sol
ops-in-air = En vol
ops-in-air-value = { $fighters } chasseurs, { $strike } d'attaque, { $helicopters } hélicoptères
ops-since-start = Depuis le début
ops-earned = gagné
ops-repairs = réparations
ops-operations = sorties, raids, insertions
ops-new-aircraft = nouveaux appareils
ops-lost = pertes (villes, pilotes)
ops-section-how = Fonctionnement
ops-how =
    Chaque camp gagne du matériel chaque heure : une part fixe, plus pour chaque lieu tenu et pour chaque ville intacte parmi eux. Bombardez les villes ennemies (INTERDICTION) pour réduire son revenu.
    L'IA paie tout ce qu'elle fait : réparations, chaque sortie de chasse, raids et ses propres insertions. À court de matériel, ses dégâts ne sont pas réparés et ses avions restent au sol.
    L'IA ne vole qu'avec les appareils qu'elle a ; abattus, ils sont perdus, posés, ils revolent. Chaque heure, chaque camp en achète de nouveaux tant que son matériel le permet.
    Vous ne manquez jamais de rien : vos appareils, vos bases (bombardées ou non) et les hélicoptères de vos propres ordres d'insertion sont toujours là.
ops-section-prices = Prix
ops-price-airfield = Réparation d'aérodrome
ops-price-sam = Réparation de site SAM
ops-price-city = Ville reconstruite
ops-price-ship = Réparation de navire
ops-price-gun = Canon ou équipe MANPADS
ops-price-sortie = Sortie de chasse
ops-price-raid = Raid
ops-price-insertion = Insertion de l'IA
ops-price-fighter = Nouveau chasseur
ops-price-helicopter = Nouvel hélicoptère
ops-price-city-lost = Ville détruite (perte)
ops-price-captured = Pilote capturé (perte)

## Carte de guerre

ops-click-to-watch = { $unit } - cliquer pour suivre
ops-section-legend = Légende
ops-legend-friendly-airfield = Aérodrome ami
ops-legend-enemy-airfield = Aérodrome ennemi
ops-legend-city = Ville amie / ennemie
ops-legend-airfield = Aérodrome ami / ennemi
ops-legend-insertion = Route d'insertion
ops-watch-hint = Cliquez sur une unité pour la suivre en direct
ops-map-hint = Molette : zoom, glisser : déplacer
ops-patriot = Batterie Patriot
ops-drone-center = Centre de commandement de drones
ops-manned-by = Servie par { $pilot }
ops-manned-by-other = Servie par un autre joueur
ops-control = PRENDRE LES COMMANDES
    .facts = Prendre sa console

## Débriefing, fin, journal, chat

ops-no-sortie = Aucune sortie volée pour l'instant
ops-section-sortie = Sortie
ops-achieved = OBJECTIF ATTEINT
ops-not-achieved = OBJECTIF NON ATTEINT
ops-waiting-raid = EN ATTENTE DU RAID
ops-waiting-insertion = EN ATTENTE DE L'INSERTION
ops-flight-time = Temps de vol { $mins } min
ops-kills = Victoires : { $air } appareils, { $ground } au sol
ops-points = { $points } points
ops-points-bonus = { $points } points (bonus de package +{ $bonus })
ops-campaign-points = { $rank } - { $points } points de campagne
ops-campaign-record = Campagne : { $sorties } sorties, { $won } objectifs atteints
ops-section-meanwhile = Pendant ce temps
ops-victory = VICTOIRE
    .text = L'ennemi ne tient plus aucun aérodrome : le théâtre est à nous.
ops-defeat = DÉFAITE
    .text = Il ne reste plus aucun aérodrome ami : le théâtre est perdu.
ops-end-day = Jour { $day }. Lieux amis { $friendly }, lieux ennemis { $enemy }.
ops-next-campaign = LA PROCHAINE CAMPAGNE COMMENCE DANS { $time }
ops-next-campaign-now = LA PROCHAINE CAMPAGNE COMMENCE
ops-next-from-menu = Lancez la prochaine campagne depuis le menu principal.
ops-section-log = Journal de campagne
ops-section-chat = Chat
ops-no-messages = Aucun message pour l'instant
ops-message = Message
    .hint = Tapez un message, Entrée l'envoie
ops-send = ENVOYER

## Briefing

ops-briefing = BRIEFING DE MISSION
ops-briefing-line = Depuis { $base } - { $km } km jusqu'à l'objectif - { $aircraft } - emports : { $stores }
ops-route-locked = Route verrouillée : { $pilots } la vole
ops-route-threats = ROUTE ET MENACES
ops-take-off = DÉCOLLER
ops-radars-shown = Radars affichés : détectés par des appareils amis, encore debout

## Joueurs

ops-section-players = Joueurs
ops-col-name = NOM
ops-col-status = STATUT
ops-col-sorties = SORTIES
ops-col-points = POINTS
ops-col-pilot = PILOTE
ops-col-objectives = OBJECTIFS
ops-col-kills = VICT. AIR/SOL
ops-col-medal = MEILLEURE MÉDAILLE
ops-you = (vous)
ops-host = HÔTE
ops-status-planning = Planifie : { $title }
ops-status-board = Au tableau des missions
ops-status-flying = En vol : { $title } - { $aircraft }
ops-status-reconnecting = Connexion perdue - appareil maintenu { $seconds } s
ops-flown = { $flown } ({ $won } réussies)
ops-section-leaderboard = Classement
ops-alone = En solo. Pour voler ensemble, hébergez cette campagne depuis MULTIJOUEUR dans le menu principal.
ops-rules = Règles de cette campagne (fixées à son début) : { $rules }
ops-no-sorties = Aucune sortie volée pour l'instant
