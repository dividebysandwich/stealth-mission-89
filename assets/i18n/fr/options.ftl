# La fenêtre OPTIONS (src/options.rs) et ses pages du menu en vol.

options-title = OPTIONS
options-tab-game = JEU
options-tab-graphics = GRAPHISMES
options-tab-sound = SON
options-tab-controls = COMMANDES
options-footer = Enregistré pour les prochains vols ; les réglages de réalisme se choisissent à chaque partie

## JEU

options-section-language = Langue
options-section-classic-modern = Classique / Moderne
options-cockpit = Cockpit
    .facts =
        Moderne : écrans haute résolution avec boutons en bordure
        Classique : le tableau de bord de l'ère DOS et ses écrans 77x60 (F-117A)
options-hud = HUD
    .facts =
        Moderne : HUD haute résolution vert façon F-16, avec les modes NAV, AA et AG
        Classique : le HUD de l'ère DOS
options-avionics = Avionique
    .facts =
        Moderne : pages MFD façon F-16 sur chaque MFD (situation, HARM, nacelle de désignation, carte, emports) avec un curseur mobile
        Classique : carte et TAC à gauche, caméra, WAY, WPN et ORD à droite
options-flight-model = Modèle de vol
    .facts =
        Moderne : modèle de vol aérodynamique avec commandes de vol électriques : le manche commande les vitesses de tangage et de roulis ; relâché, le nez reste en place et la trajectoire est tenue
        Classique : un modèle de vol de l'ère DOS
options-section-flying = Pilotage
options-auto-trim = Trim automatique
    .facts =
        Manche relâché, le trim tient l'assiette où vous l'avez laissée
        Hélicoptères : le nez ne se lève plus avec la vitesse ; jets : compense les gouvernes endommagées
        Trim manuel : Alt + flèches, Alt + Origine le recentre
options-tooltips = Tooltips du cockpit
    .facts = Ce qu'est une commande ou un écran du cockpit, sous la souris ou un doigt en VR
options-section-vr-size = Taille du cockpit en VR
options-vr-size =
    .facts =
        Taille du cockpit (et du monde) dans le casque
        100 % : taille réelle

## GRAPHISMES

options-section-rendering = Rendu
options-shaders = Shaders
    .facts =
        Moderne : ombrage directionnel discret, brume de distance douce
        Classique : ombrage plat
options-shadows = Ombres
    .facts =
        Moderne : ombres du soleil dans le cockpit et le monde (coûte du temps GPU)
        Classique : pas d'ombres
options-point-lights = Lumières ponctuelles
    .facts =
        Moderne : éclairage du cockpit et des écrans ; explosions, panaches et feux de piste éclairent la scène
        Classique : lumière du soleil et du ciel seulement
options-time-of-day = Heure du jour
    .facts =
        Moderne : le soleil et la lune suivent l'horloge de la mission, le ciel s'assombrit et s'éclaircit avec le soleil ; étoiles et phase de la lune la nuit
        Moderne : le clair de lune règle à quelle distance les pilotes ennemis vous voient la nuit (2 km sans lune, 4 km sous la pleine lune)
        Classique : un lent changement de couleurs et un soleil fixe
options-msaa = Anticrénelage (MSAA)
    .facts =
        Bords lissés : MSAA 4x, 2x en VR (coûte du temps GPU)
        Désactivé : bords de pixels nets
options-auto-detail = Détail automatique du décor (VR)
    .facts =
        Dans le casque, la portée des villes, forêts et champs diminue quand les images prennent du retard et revient quand il reste du temps
        Les réglages graphiques restent tels que choisis
options-sun-moon = Soleil et lune
    .facts =
        Le soleil dans le ciel le jour, la lune et les étoiles la nuit
        La lune montre la phase de la mission

## SON

options-section-volume = Volume
options-volume-master = Général
    .facts = Tous les sons
options-volume-engine = Moteur
    .facts = Votre moteur ou votre rotor, et les appareils entendus autour de vous
options-volume-effects = Effets
    .facts = Armes, explosions, train, radio et le reste
options-volume-warnings = Alarmes
    .facts = Alarme de décrochage, alertes radar et missile, sirène d'incendie, tonalités du Sidewinder
options-section-cockpit = Cockpit
options-low-pass = Filtre passe-bas du moteur
    .facts =
        Dans l'appareil, le moteur sonne étouffé (la verrière et le casque coupent les aigus)
        Désactivé : le même son qu'à l'extérieur

## COMMANDES

options-section-bindings = Affectations
options-keys-joystick = TOUCHES ET JOYSTICK
    .facts =
        Affecter les touches, les axes et les boutons du joystick et du HOTAS
        Enregistré pour les prochains vols
options-section-head-tracking = Suivi de la tête et des yeux
options-head-tracking = Suivi de la tête et des yeux
    .facts =
        Votre tête tourne et déplace la vue dans le cockpit (sur écran ; en VR, c'est le casque)
        Programme de suivi (OpenTrack...) : sortie FreeTrack, ou UDP vers 127.0.0.1 port 4242
        Origine : la position actuelle de la tête devient le centre
options-tracking = Suivi actif ({ $source })
options-no-tracker = Aucun suivi détecté : lancez le programme de suivi

## EFFECTS

options-section-effects = Effets
options-section-retro = Rétro
effect-shake = Secousses de la caméra
    .facts =
        La vue tremble avec le canon, les explosions proches, le buffeting de décrochage, les posés et la piste
        VR : le cockpit vibre, votre tête reste immobile
effect-flashes = Éclairs de combat
    .facts =
        Les tirs de DCA, explosions et départs de missiles éclairent un instant les nuages, le sol et le cockpit
effect-barrage = Barrage nocturne
    .facts =
        Guerre conventionnelle de nuit : les canons des villes tirent des traçantes à l'aveugle vers le ciel, des projecteurs balaient le ciel
        Pour l'ambiance seulement : le barrage ne touche rien, les projecteurs ne trouvent rien
effect-cockpit-night = Éclairage nocturne du cockpit
    .facts =
        Un faible éclairage rouge dans le cockpit à la nuit tombée (bleu-vert sous les jumelles)
        Le voyant d'alerte générale pulse en ambre sur le tableau tant qu'une alarme est active
effect-nvg = Jumelles de vision nocturne
    .facts =
        Les lumières vives forment des halos, l'image est plus granuleuse dans l'obscurité
        Bureau : le champ de vision rond des jumelles
effect-snow = Neige
    .facts =
        Au-dessus de l'isotherme zéro, les orages donnent de la neige au lieu de la pluie : bas en hiver nordique, haut en été
effect-mist = Brume au sol
    .facts =
        Shaders modernes : de la brume couvre les basses terres et la mer à l'aube, au crépuscule et la nuit
        Fine dans les déserts, absente par orage
effect-cloud-shadows = Ombres des nuages
    .facts =
        Shaders modernes : les ombres des nuages sur le sol en dessous
effect-vapour = Vapeur et souffle du rotor
    .facts =
        En virage serré, de la vapeur se forme aux bouts d'aile
        Un hélicoptère en vol stationnaire bas soulève de la poussière, ou des embruns au-dessus de l'eau
effect-night-sky = Ciel nocturne
    .facts =
        Des étoiles filantes de temps en temps, les étoiles basses sur l'horizon scintillent
        Les aurores boréales au-dessus du Cap Nord
effect-vga = Couleurs VGA
    .facts =
        Le look de 1989 avec l'ombrage moderne : 64 niveaux par canal de couleur, comme une carte VGA, tramés
effect-scanlines = Lignes de balayage
    .facts =
        Les lignes de balayage d'un moniteur cathodique (pas en VR)

## LANGUAGE NAMES

lang-name-en = anglais
lang-name-bg = bulgare
lang-name-zh-Hans = chinois (simplifié)
lang-name-hr = croate
lang-name-cs = tchèque
lang-name-da = danois
lang-name-nl = néerlandais
lang-name-et = estonien
lang-name-fi = finnois
lang-name-fr = français
lang-name-de = allemand
lang-name-el = grec
lang-name-hu = hongrois
lang-name-id = indonésien
lang-name-ga = irlandais
lang-name-it = italien
lang-name-ja = japonais
lang-name-ko = coréen
lang-name-lv = letton
lang-name-lt = lituanien
lang-name-mt = maltais
lang-name-pl = polonais
lang-name-pt-BR = portugais (Brésil)
lang-name-ro = roumain
lang-name-ru = russe
lang-name-sk = slovaque
lang-name-sl = slovène
lang-name-es = espagnol
lang-name-sv = suédois
lang-name-uk = ukrainien

## Browser build: offer to play in the browser's language (src/i18n.rs)

language-offer-question = Langue du navigateur : { $language }. Jouer dans cette langue ?
language-offer-yes = Oui
language-offer-no = Non
language-offer-never = Ne plus me demander
effect-sunbeams = Rayons de soleil
    .facts =
        Des rayons de soleil passent entre les trouées d'une couverture nuageuse fragmentée
