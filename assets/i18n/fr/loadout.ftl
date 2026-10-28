# L'éditeur d'armement (src/loadout.rs). Les noms des armes sont les vrais, non traduits.

loadout-none = Aucun
loadout-briefed = BRIEFING
    .facts = Les emports prévus par le briefing de la mission
loadout-clear = VIDER
    .facts = Tous les points d'emport vides
loadout-done = OK
loadout-no-hardpoints = Aucun point d'emport
loadout-no-stores = Cet appareil n'emporte pas de charges
loadout-fantasy = N'existe pas sur l'avion réel
loadout-empty = Vide
loadout-out-of-stock = Aucun en stock pour cette mission
loadout-mirror = Pareil des deux côtés
    .facts = Charge aussi { $station }
loadout-stores = Emports
loadout-needs-equipment = La mission a besoin de sa caméra ou de son équipement spécial

## À quoi sert chaque arme

# AIM-9M Sidewinder
weapon-role-0 = Air-air, guidage infrarouge, courte portée
# AIM-120A AMRAAM
weapon-role-1 = Air-air, guidage radar, moyenne portée
# AGM-88A HARM
weapon-role-2 = Missile antiradar : radars et sites SAM
# Penguin 3 ASM
weapon-role-3 = Missile antinavire
# AGM-84A Harpoon
weapon-role-4 = Missile antinavire à longue portée
# AGM-65 Maverick
weapon-role-5 = Missile à guidage TV : véhicules, bunkers, bâtiments
# GBU-12 Paveway
weapon-role-6 = Bombe à guidage laser : cibles durcies
# Mk 20 Rockeye
weapon-role-7 = Bombe à sous-munitions : cibles légères et étendues
# Durandal
weapon-role-8 = Bombe anti-piste
# Mk 82 LDGP
weapon-role-9 = Bombe lisse de 500 lb
# Mk 82 Snakeye
weapon-role-10 = Bombe freinée de 500 lb pour les largages à basse altitude
# Mk 20 Rockeye II
weapon-role-11 = Bombe à sous-munitions : blindés et cibles étendues
# Mk 122 Fireye
weapon-role-12 = Bombe incendiaire
# CBU-72 FAE
weapon-role-13 = Explosif air-carburant : cibles étendues
# Mk 35 incendiary
weapon-role-14 = Bombe à sous-munitions incendiaire
# ISCB-1 mines
weapon-role-15 = Mines d'interdiction de zone
# IR/optical camera
weapon-role-16 = Photos de reconnaissance de l'objectif
# 1900 lb fuel tank
weapon-role-17 = Carburant supplémentaire, transféré au largage
# Special cargo
weapon-role-18 = Ravitaillement ou cargaison pour la mission
# 20 mm cannon
weapon-role-19 = Le canon (pas une charge)
# AGM-114 Hellfire
weapon-role-20 = Missile antichar à guidage laser : vole vers le point visé par la nacelle

## Emploi de chaque arme (portées en km du HUD, altitudes en pieds)

employment-0 = Portée : 1-12 km, plus de face
employment-1 = Portée : 1-38 km, plus de face
employment-2 =
    Portée : 9-50 km
    Tir : 5 000-20 000 ft
employment-3 =
    Portée : 6-14 km (guidage classique : 13-14 km)
    Tir : 100-500 ft
employment-4 =
    Portée : 6-41 km
    Tir : 2 000-5 000 ft
employment-5 =
    Portée : 5-10 km
    Tir : 2 000-5 000 ft
employment-6 =
    Portée : de 1,5 km à 2,5 fois la hauteur, LOCK fixe (14 km depuis 15 000 ft, 18 km depuis 25 000 ft)
    Largage : 15 000-25 000 ft
employment-8 =
    Portée : 0,9-1,3 km avant la piste
    Largage : 200-500 ft
employment-9 =
    Portée : 5-11 km avant l'objectif
    Largage : 3 000-20 000 ft
employment-12 =
    Portée : 0,9-2,2 km avant l'objectif
    Largage : 100-500 ft
employment-16 = Portée : photos à 2-8 km (plus il y a d'adversaires, plus près), nez sur l'objectif
employment-18 = Portée : larguer à « Drop supplies now », à moins de 4 km de la zone de largage
employment-20 =
    Portée : 4-8 km
    Tir : 2 000-3 000 ft
employment-guided-cluster =
    Portée : de 1,5 km à 2,5 fois la hauteur, LOCK fixe (3,7 km depuis 3 000 ft, 6 km depuis 5 000 ft)
    Largage : 3 000-5 000 ft
employment-medium-retarded =
    Portée : 1,1-1,6 km avant l'objectif
    Largage : 300-1 000 ft
employment-low-retarded =
    Portée : 0,9-1,3 km avant l'objectif
    Largage : 200-500 ft

## Points d'emport

hardpoint-left-bay = Soute gauche
hardpoint-right-bay = Soute droite
hardpoint-left-wing-bay = Soute d'aile gauche
hardpoint-right-wing-bay = Soute d'aile droite
hardpoint-station-1-left-wingtip = Station 1, saumon d'aile gauche
hardpoint-station-2-left-outer-wing = Station 2, aile gauche extérieure
hardpoint-station-3-left-wing = Station 3, aile gauche
hardpoint-station-4-left-inner-wing = Station 4, aile gauche intérieure
hardpoint-station-5-centreline = Station 5, ventrale
hardpoint-station-6-right-inner-wing = Station 6, aile droite intérieure
hardpoint-station-7-right-wing = Station 7, aile droite
hardpoint-station-8-right-outer-wing = Station 8, aile droite extérieure
hardpoint-station-9-right-wingtip = Station 9, saumon d'aile droit
hardpoint-station-1-left-wingtip-rail = Station 1, rail de saumon gauche
hardpoint-station-2-left-outer-pylon = Station 2, pylône gauche extérieur
hardpoint-station-3-left-inner-pylon = Station 3, pylône gauche intérieur
hardpoint-station-4-right-inner-pylon = Station 4, pylône droit intérieur
hardpoint-station-5-right-outer-pylon = Station 5, pylône droit extérieur
hardpoint-station-6-right-wingtip-rail = Station 6, rail de saumon droit
hardpoint-tornado-left-wing-rail = Aile gauche, rail Sidewinder
hardpoint-tornado-left-wing-pylon = Pylône de l'aile gauche
hardpoint-tornado-left-shoulder-forward = Épaule gauche, avant
hardpoint-tornado-left-shoulder-aft = Épaule gauche, arrière
hardpoint-tornado-right-shoulder-aft = Épaule droite, arrière
hardpoint-tornado-right-shoulder-forward = Épaule droite, avant
hardpoint-tornado-right-wing-pylon = Pylône de l'aile droite
hardpoint-tornado-right-wing-rail = Aile droite, rail Sidewinder
hardpoint-a10-1-left-outer-wing = Station 1, aile gauche extérieure
hardpoint-a10-2-left-wing = Station 2, aile gauche
hardpoint-a10-3-left-wing = Station 3, aile gauche
hardpoint-a10-4-left-centre-wing = Station 4, aile gauche centrale
hardpoint-a10-5-left-fuselage = Station 5, fuselage gauche
hardpoint-a10-6-centreline = Station 6, axe
hardpoint-a10-7-right-fuselage = Station 7, fuselage droit
hardpoint-a10-8-right-centre-wing = Station 8, aile droite centrale
hardpoint-a10-9-right-wing = Station 9, aile droite
hardpoint-a10-10-right-wing = Station 10, aile droite
hardpoint-a10-11-right-outer-wing = Station 11, aile droite extérieure
weapon-role-21 = Air-air, guidage infrarouge, très courte portée
employment-21 = Portée : 1-6 km, plus loin de face
