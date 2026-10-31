# La fenêtre de campagne, la fenêtre multijoueur, la légende de la carte de guerre et la vue
# en direct.

delete = SUPPRIMER
delete-campaign = SUPPRIMER CETTE CAMPAGNE ?
delete-save = SUPPRIMER CETTE SAUVEGARDE ?
delete-permanent = La suppression est définitive

## Fenêtre de campagne

cm-group-campaign = Campagne
cm-weather-dynamic = Dynamique
    .facts =
        La météo change toutes les 10 à 22 heures de l'horloge du ciel
        Les nuages bloquent la vue des deux camps
cm-weather-clear = Ciel dégagé
    .facts = Aucun nuage pendant toute la campagne
cm-section-day-night = Jour et nuit
cm-day-real = Temps réel
    .facts = Une journée dure 24 heures
cm-day-2h = 2 h / jour
    .facts = Une journée dure 2 heures
cm-day-1h = 1 h / jour
    .facts = Une journée dure 1 heure
cm-day-30min = 30 min / jour
    .facts = Une journée dure 30 minutes
cm-conditions-note = Les conditions s'appliquent à une nouvelle campagne
cm-rules-note = Une campagne enregistrée garde les règles de son début
cm-about =
    La guerre se déroule en temps réel pendant que vous jouez, même menu ouvert.
    Volez les missions du tableau : reco, SEAD, CAP, CAS, antinavire, frappe, escorte.
    Les insertions par hélicoptère prennent les villes et les aérodromes ennemis.
    Vous n'êtes jamais tué ni capturé : après une sortie, vous revenez au tableau.
cm-section-new = Nouvelle campagne
cm-new-line = { $theater }  -  { $situation }  -  adversaires { $opponents }
cm-start = COMMENCER
cm-section-continue = Continuer
cm-no-saves = Aucune campagne enregistrée
cm-played = { $hours } h jouées
cm-save-gone = La sauvegarde n'existe plus
cm-save-unreadable = La sauvegarde n'a pas pu être lue : { $error }
cm-save-undeletable = La sauvegarde n'a pas pu être supprimée : { $error }
cm-in-vr = EN VR
cm-delete =
    .facts = Supprimer cette campagne enregistrée

## Carte de guerre

pic-friendly-aircraft = Avion ami
pic-friendly-helicopter = Hélicoptère ami
pic-enemy-fighter = Chasseur ennemi (au radar)
pic-sam-known = Site SAM (connu)
pic-guns-known = Canons / MANPADS (connus)
pic-armour = Char / VCI (ennemi : en vue)
pic-patriot-reach = Portée des Patriot
pic-gepard-stinger = Gepard / Stinger
pic-missile-range = Portée des missiles SAM
pic-engagement-range = Portée d'engagement
pic-reconned = Reconnu
pic-you = Vous
pic-site = Site de { $place }
uv-live = { $unit } - EN DIRECT
uv-contact-lost = CONTACT PERDU
uv-hint = Glisser : tourner  Molette : zoom

## Multijoueur

mp-disconnected = Déconnecté : { $why }
mp-connecting = Connexion à { $address }...
mp-joining = Connecté à { $address }, entrée en jeu...
mp-browser-public-only = depuis le navigateur, seuls les serveurs de la liste publique peuvent être rejoints
mp-title-host = MULTIJOUEUR : HÉBERGER UNE PARTIE
mp-subtitle = Coopération dans la campagne dynamique : faites la guerre ensemble
mp-your-name = Votre nom
    .hint = Nom du pilote
mp-join-address = Rejoindre par adresse
    .hint = adresse ou nom[:port]
mp-connect = SE CONNECTER
mp-server-name = Nom du serveur
mp-udp-port = Port UDP
mp-address-note = Les autres joueurs rejoignent avec l'adresse de cet ordinateur ; ouvrez le port UDP de votre routeur pour jouer par internet
mp-list-public = Afficher sur la liste publique des serveurs (les autres trouvent la partie dans leur liste)
mp-rules = Règles pour tous
mp-time-of-day = Heure du jour (nuits au clair de lune)
mp-realistic-weapons = Armement réaliste du F-117A
mp-friendly-fire = Tirs fratricides
mp-day-night = Jour et nuit :
mp-weather = Météo :
mp-host-new = HÉBERGER UNE NOUVELLE
mp-host-saved = Héberger une campagne enregistrée
mp-host = HÉBERGER
mp-host-game = HÉBERGER UNE PARTIE
mp-port-error = Port UDP { $port } : { $error }
mp-default-server = Campagne de { $pilot }
mp-public-servers = Serveurs publics
mp-refresh = ACTUALISER
mp-server-count = { $servers } serveurs, { $pilots } pilotes
mp-list-error = La liste des serveurs est injoignable : { $error }
mp-no-servers = Aucun serveur public pour l'instant
mp-col-server = SERVEUR
mp-col-pilots = PILOTES
mp-col-war = GUERRE
mp-dedicated = DÉDIÉ
mp-other-version = autre version
mp-join = REJOINDRE
