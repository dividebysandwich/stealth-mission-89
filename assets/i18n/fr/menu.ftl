# Le menu principal et l'écran de mission simple (src/menu.rs).

## Théâtres

theater-libya = Libye (1986)
    .facts =
        Golfe de Syrte et côte libyenne
        Époque de l'opération El Dorado Canyon
        Ciel désertique dégagé la plupart du temps
theater-persian_gulf = Golfe Persique (1984)
    .facts =
        Époque de la guerre des tankers Iran-Irak
        Au-dessus de la mer et du désert côtier
        La météo la plus dégagée de tous les théâtres
theater-north_cape = Cap Nord
    .facts =
        Norvège arctique et péninsule de Kola
        Port d'attache de la flotte soviétique du Nord
        Couche nuageuse basse fréquente
theater-central_europe = Europe centrale
    .facts =
        OTAN contre Pacte de Varsovie au-dessus de l'Allemagne
        Ciel dégagé seulement 15 % du temps
theater-vietnam = Vietnam
    .facts =
        Région de Hanoï et Haïphong
        Jungle et relief montagneux
        Orages dans 30 % des missions
theater-middle_east = Moyen-Orient
    .facts =
        Méditerranée orientale et Levant
        Ciel le plus souvent dégagé
theater-desert_storm = Tempête du désert
    .facts =
        Koweït et Irak, 1991
        Le vrai baptême du feu du F-117A
        Ciel désertique le plus souvent dégagé
theater-cuba = Cuba
    .facts =
        Théâtre insulaire des Caraïbes
        Objectifs en mer et à terre
        Tropical : orages dans 15 % des missions
theater-korea = Corée
    .facts =
        Péninsule coréenne au nord de la DMZ
        Relief montagneux
        Météo changeante, tous les ciels sont possibles
theater-ukraine = Ukraine (2026)
    .facts =
        Est de l'Ukraine, Crimée et sud de la Russie
        La ligne de front brûle de Kharkiv à Kherson
        Plaines agricoles, villes en ruine sur le front

## Situations (règles d'engagement)

roe-0 = Guerre froide
    .facts =
        Opération clandestine en temps de paix
        Rester indétecté compte avant tout
        Moins d'avions ennemis en patrouille
        Les règles d'engagement exactes sont dans le briefing
roe-1 = Guerre limitée
    .facts =
        Conflit armé régional
        Défenses aériennes ennemies en alerte
        Plus d'avions ennemis qu'en guerre froide
roe-2 = Guerre conventionnelle
    .facts =
        Guerre totale
        Toutes les forces ennemies actives
        Plus d'avions ennemis qu'en guerre froide

## Types de mission

kind-strike = Frappe
    .facts =
        Détruire, photographier ou atteindre un objectif au sol
        Objectif principal et secondaire
        Armement air-sol
kind-air = Air-air
    .facts =
        Intercepter un avion ennemi
        Transports, bombardiers ou patrouilles de chasse
        Armement de missiles air-air
kind-strike-training = Entraînement frappe
    .facts =
        Mission de frappe d'entraînement
        Apprendre les armes et la navigation
kind-air-training = Entraînement air-air
    .facts =
        Mission air-air d'entraînement
        Apprendre les interceptions et les missiles
kind-drone-defense = Défense anti-drones
    .facts =
        Ukraine uniquement
        Arrêter une vague de drones d'attaque Geran avant qu'elle ne frappe une ville ou une centrale nucléaire
        Puis bombarder leur site de lancement

## Adversaires

opponents-0 = Novice
    .facts =
        Équipages ennemis inexpérimentés
        Niveau le plus facile
opponents-1 = Standard
    .facts =
        Équipages ennemis moyens
        Niveau par défaut
opponents-2 = Vétéran
    .facts =
        Équipages ennemis aguerris
        Plus de navires et de patrouilles
opponents-3 = Élite
    .facts =
        Les meilleurs pilotes et servants de SAM ennemis
        Niveau le plus difficile

## Météo

sky-random = Aléatoire
    .facts =
        Tirée du climat du théâtre
        Change à chaque nouvelle mission
        Les nuages bloquent la vue des deux camps
sky-clear = Dégagé
    .facts =
        Aucun nuage
        Visibilité 40 km
        Le ciel classique de l'ère DOS
sky-scattered = Épars
    .facts =
        Cumulus 20-45 %, base 2500-6000 ft
        Parfois un mince altostratus très haut
        Visibilité 30 km
sky-broken = Fragmenté
    .facts =
        Cumulus 45-70 %, base 2000-5000 ft
        Souvent une couche de stratocumulus au-dessus
        Visibilité 25 km
sky-overcast = Couvert
    .facts =
        Stratus 80-100 %, base 1500-5000 ft
        Quelques fragments de cumulus en dessous
        Visibilité 20 km
sky-storm = Orage
    .facts =
        Tours de cumulonimbus jusqu'à 40000 ft
        Stratus et cumulus bas dès 1000 ft
        Visibilité 8 km
menu-random-cover = Couverture aléatoire
    .facts =
        Couverture de la couche principale tirée avec la météo
        Décocher pour la régler soi-même
        Couche principale : stratus par temps couvert et orageux, cumulus sinon
menu-cover-slider =
    .facts =
        Part du ciel couverte par la couche principale
        Sans effet par ciel dégagé

## Lune

moon-date = Date de la mission
    .facts =
        La vraie lune à la date du briefing
        Demande l'heure du jour en mode moderne
moon-0 = Nouvelle lune
    .facts =
        Pas de clair de lune : nuit noire, lumières et étoiles seulement
        Les pilotes ennemis vous voient à 2 km
moon-1 = Premier croissant
    .facts = Une lune fine dans le ciel du soir, couchée avant minuit
moon-2 = Premier quartier
    .facts = À moitié éclairée, visible le soir, se couche vers minuit
moon-3 = Gibbeuse croissante
    .facts = Brillante, visible de l'après-midi à après minuit
moon-4 = Pleine lune
    .facts =
        Visible toute la nuit, haute à minuit
        Les pilotes ennemis vous voient à 4 km
moon-5 = Gibbeuse décroissante
    .facts = Brillante, se lève le soir, visible jusqu'au matin
moon-6 = Dernier quartier
    .facts = À moitié éclairée, se lève vers minuit
moon-7 = Dernier croissant
    .facts = Une lune fine qui se lève avant l'aube

## Appareils

aircraft-f117 =
    .facts =
        Avion d'attaque furtif
        Subsonique, 7 g
        Cockpit moderne et classique
aircraft-f16 =
    .facts =
        Chasseur multirôle
        Classe Mach 2, 9 g, roulis rapide
        Cockpit de F-16 avec RWR
aircraft-tornado =
    .facts =
        Avion d'attaque à basse altitude
        Ailes à géométrie variable selon la vitesse
        Cockpit moderne, trois MFD
aircraft-a10 =
    .facts =
        Avion d'appui aérien rapproché
        Lent et robuste, canon de 30 mm
        Cockpit A-10C, deux MFD
aircraft-av8b =
    .facts =
        Avion d'attaque ADAC/V
        Poussée vectorielle : vol stationnaire, se pose partout
        Levier des tuyères : R / F, Maj pour les butées
aircraft-uh60 =
    .facts =
        Hélicoptère de transport
        Dépose des troupes : se poser près de l'objectif
        Cyclique et collectif
aircraft-ah1z =
    .facts =
        Hélicoptère d'attaque
        Canon en tourelle, Hellfire, AIM-9
        Le canon et les Hellfire suivent la nacelle

## Sections et groupes

section-theater = Théâtre
section-opponents = Adversaires
section-situation = Situation
section-mission = Mission
section-moon = Lune
section-start-time = Heure de départ
section-weather = Météo
section-realism = Réalisme
section-tacview = Tacview
section-mission-summary = Résumé de la mission
section-briefing = Briefing
group-mission = Mission
group-conditions = Conditions
group-vehicle = Appareil
group-simulation = Simulation

## Options de la mission

menu-start-random = Aléatoire
    .facts =
        Tirée à chaque nouvelle mission, comme en mode classique
        06:00-08:00 de jour ou 18:00-20:00 de nuit
        Décocher pour choisir l'heure
menu-start-slider =
    .facts =
        Heure de décollage
        Nuit de 18:00 à 06:00
        Conservée par NOUVELLE MISSION
menu-realistic-weapons = Armement réaliste
    .facts =
        Les limites de l'avion réel
        2 soutes au lieu de 4
        Pas de missiles AIM-9 ni AIM-120
        Pas de décollage ni d'appontage sur porte-avions
menu-crash-protection = Protection anti-crash
    .facts =
        Redresse automatiquement près du sol (train rentré)
        Les contacts avec le sol et les collisions ne sont pas fatals
        Désactivée : le sol peut toujours être percuté
menu-tacview-export = Export
    .facts =
        Enregistre chaque vol au format Tacview (.acmi)
        Dans le dossier du jeu pour vos fichiers, sous tacview
menu-tacview-real-time = Télémétrie en temps réel
    .facts =
        Envoie le vol à Tacview pendant que vous volez
        Dans Tacview : Record > Real-Time Telemetry
        Adresse 127.0.0.1, port 42674 (--tacview-address)
switch-game = JEU
switch-real = RÉEL
menu-damage = Dégâts
    .facts =
        Dégâts des armes
        Jeu : dégâts classiques ; les impacts s'additionnent mais n'abattent jamais l'avion
        Réaliste : un missile peut détruire l'avion (éjectez-vous !), endommage plus de systèmes et déclenche souvent un incendie, qui s'éteint ou fait exploser l'avion
        Réaliste : des gouvernes endommagées dérèglent le trim ou font osciller l'avion
menu-guidance = Guidage
    .facts =
        Guidage des missiles
        Classique : les missiles volent droit sur leur cible
        Réaliste : navigation proportionnelle vers le point de collision ; les moteurs s'éteignent et les missiles planent ensuite en perdant de la vitesse à chaque virage (fumée seulement moteur allumé)
        Réaliste : les SAM prennent de la hauteur contre les cibles lointaines ; l'AIM-120 vole droit pendant 3 s (cabrez pour le lober)
menu-air-defence = Défense aérienne
    .facts =
        Défenses aériennes à basse altitude
        Classique : seulement les sites SAM et les chasseurs
        Réaliste : des canons ZSU-23-4 guidés par radar (leur radar vous voit comme celui d'un SAM) et des batteries de flak S-60 gardent les aérodromes et les objectifs ; des équipes MANPADS dans les villes tirent des SA-14 sur ce qu'elles voient
        Voler bas au-dessus des lieux défendus devient dangereux ; bombes et canon détruisent les pièces

## Écran principal

menu-title = STEALTH MISSION 89
menu-subtitle = SYSTÈME DE PLANIFICATION DE MISSION
menu-single-mission = MISSION SIMPLE
    .facts =
        Une mission dans le théâtre de votre choix
        Choisissez les options, planifiez la route et l'armement
menu-career = CARRIÈRE
    .facts =
        Dossiers des pilotes : grade, décorations, sorties
        Tour d'opérations : 12 sorties dans un théâtre
        Ce que vous détruisez reste détruit
menu-campaign = CAMPAGNE DYNAMIQUE
    .facts =
        Campagne dynamique en temps réel
        Prenez les villes et les aérodromes de l'ennemi
        Des missions tirées de la situation en cours
menu-multiplayer = MULTIJOUEUR
    .facts = Coopération dans la campagne dynamique
    .join = Rejoindre un serveur de la liste publique
    .host = Rejoindre un serveur, ou héberger une campagne et y voler
menu-options = OPTIONS
    .facts =
        Jeu : style du cockpit, du HUD et de l'avionique, trim automatique
        Graphismes, volumes sonores, touches et joystick
        Enregistrées pour les prochains vols
    .short = Jeu, graphismes, son et commandes
menu-download = TÉLÉCHARGER
    .facts =
        Le jeu de bureau pour Windows, Linux et macOS
        Avec la VR et le multijoueur
        Steam Deck et Steam Frame : l'AppImage Linux
web-play = JOUER DANS LE NAVIGATEUR
    .facts = Ici même, rien à installer
web-download-platforms = Windows, Linux et macOS, Steam Deck et Steam Frame
web-download-version = La version à télécharger
web-gives-vr = VR
    .why = Volez en VR avec tout casque compatible OpenXR et le Steam Frame
web-gives-hotas = HOTAS
    .why = Prise en charge complète des manettes, joysticks, manettes des gaz et palonniers
web-gives-multiplayer = Meilleur multijoueur
    .why = Connexion UDP directe aux serveurs pour moins de latence
web-gives-server = Hébergez votre serveur
    .why = Hébergez une campagne coop ou lancez un serveur dédié
web-gives-head-tracking = Suivi de la tête
    .why = Capteurs de type TrackIR via OpenTrack
web-gives-tacview = Tacview
    .why = Enregistrements de vol et télémétrie en temps réel
web-gives-keep = À vous pour toujours
    .why = Si le site disparaît, vous gardez votre propre copie
menu-discord = DISCORD
    .facts =
        Volez ensemble, signalez les bugs et partagez des missions
        Suivez le développement
menu-quit = QUITTER
    .facts = Retour au bureau
menu-version = VERSION { $version }

## Écran de mission simple

menu-career-sortie = SORTIE DE CARRIÈRE
menu-tour-sortie = { $pilot }  //  SORTIE { $sortie } SUR { $length } DU TOUR
menu-msn = SYSTÈME DE PLANIFICATION DE MISSION  //  MSN { $seed }
menu-no-targets = Plus aucun objectif debout pour ce tour
menu-plan-route = PLANIFIER LA ROUTE
    .facts =
        La carte du théâtre avec la couverture des radars et des SAM ennemis
        Ajoutez, déplacez et supprimez des points de navigation
        Les objectifs et la base d'atterrissage restent fixes
menu-loadout = ARMEMENT
    .facts =
        Choisissez les emports de chaque soute ou pylône
        Commence avec les armes du briefing
menu-generating = GÉNÉRATION...
menu-back-records = Retour aux dossiers des pilotes
menu-back-main = Retour au menu principal
menu-new-sortie = NOUVELLE SORTIE
menu-new-mission = NOUVELLE MISSION
    .facts =
        Tire d'autres objectifs
        Conserve toutes les options et l'heure de départ
menu-load = CHARGER
    .facts =
        Les missions enregistrées en vol (menu : ENREGISTRER LA MISSION)
        Reprenez le vol là où il a été enregistré
menu-fly = VOLER
    .facts = Décoller pour la mission du briefing
menu-fly-vr = VOLER EN VR
    .facts =
        Relance le jeu dans le casque VR
        Vole exactement cette mission
    .no-mission = Générez d'abord une mission
    .no-runtime = Aucun runtime OpenXR (Monado, WiVRn, SteamVR) n'est actif
menu-vr-failed = impossible de démarrer en VR : { $error }

## Résumé de la mission

summary-theater = THÉÂTRE
summary-situation = SITUATION
summary-mission = MISSION
summary-opponents = ADVERSAIRES
summary-time = HEURE
summary-weather = MÉTÉO
summary-fuel = CARBURANT
summary-night = Nuit
summary-day = Jour
summary-time-value = { $daytime }  départ { $depart }
summary-weather-value = { $sky }  vis. { $vis } km
summary-fuel-value = { $fuel } lb
summary-armament = Armement : { $stores }
summary-intelligence = RENSEIGNEMENT
aircraft-rah66 =
    .facts =
        Hélicoptère d'éclairage furtif
        Soutes : Hellfire ou Stinger
        Difficile à voir au radar, lent ou en stationnaire
