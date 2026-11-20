# El editor de armamento (src/loadout.rs). Los nombres de las armas son los reales, sin traducir.

loadout-none = Ninguno
loadout-briefed = BRIEFING
    .facts = Las cargas previstas en el briefing de la misión
loadout-clear = VACIAR
    .facts = Todos los puntos de anclaje vacíos
loadout-done = HECHO
loadout-no-hardpoints = Sin puntos de anclaje
loadout-no-stores = Esta aeronave no lleva cargas
loadout-fantasy = No existe en el avión real
loadout-empty = Vacío
loadout-out-of-stock = No quedan existencias para esta misión
loadout-mirror = Igual en ambos lados
    .facts = Carga también { $station }
loadout-stores = Cargas
loadout-realistic-f117 = Armamento realista del F-117A
loadout-stations = Estaciones
loadout-click-drawing = O pulse una estación en el dibujo
loadout-needs-equipment = La misión necesita su cámara o su equipo especial

## Para qué sirve cada arma (tipos de arma 0 a 20)

# AIM-9M Sidewinder
weapon-role-0 = Aire-aire, guiado infrarrojo, corto alcance
# AIM-120A AMRAAM
weapon-role-1 = Aire-aire, guiado radar, medio alcance
# AGM-88A HARM
weapon-role-2 = Misil antirradiación: radares y emplazamientos SAM
# Penguin 3 ASM
weapon-role-3 = Misil antibuque
# AGM-84A Harpoon
weapon-role-4 = Misil antibuque de largo alcance
# AGM-65 Maverick
weapon-role-5 = Misil de guiado TV: vehículos, búnkeres, edificios
# GBU-12 Paveway
weapon-role-6 = Bomba de guiado láser: blancos reforzados
# Mk 20 Rockeye
weapon-role-7 = Bomba de racimo: blancos ligeros y de área
# Durandal
weapon-role-8 = Bomba antipista
# Mk 82 LDGP
weapon-role-9 = Bomba de caída libre de 500 lb
# Mk 82 Snakeye
weapon-role-10 = Bomba frenada de 500 lb para lanzamientos a baja cota
# Mk 20 Rockeye II
weapon-role-11 = Bomba de racimo: blindados y blancos de área
# Mk 122 Fireye
weapon-role-12 = Bomba incendiaria
# CBU-72 FAE
weapon-role-13 = Explosivo de combustible-aire: blancos de área
# Mk 35 incendiary
weapon-role-14 = Bomba de racimo incendiaria
# ISCB-1 mines
weapon-role-15 = Minas de denegación de área
# IR/optical camera
weapon-role-16 = Fotos de reconocimiento del objetivo
# 1900 lb fuel tank
weapon-role-17 = Combustible adicional, transferido al lanzarlo
# Special cargo
weapon-role-18 = Suministros o carga para la misión
# 20 mm cannon
weapon-role-19 = El cañón (no es una carga)
# AGM-114 Hellfire
weapon-role-20 = Misil contracarro de guiado láser: vuela hacia donde mira el pod de designación

## Cómo se emplea cada arma (alcances en km del HUD, alturas en pies)

employment-0 = Alcance: 1-12 km, más de frente
employment-1 = Alcance: 1-38 km, más de frente
employment-2 =
    Alcance: 9-50 km
    Disparo: 5000-20 000 ft
employment-3 =
    Alcance: 6-14 km (guiado clásico: 13-14 km)
    Disparo: 100-500 ft
employment-4 =
    Alcance: 6-41 km
    Disparo: 2000-5000 ft
employment-5 =
    Alcance: 5-10 km
    Disparo: 2000-5000 ft
employment-6 =
    Alcance: de 1,5 km a 2,5 veces la altura, LOCK estable (14 km desde 15 000 ft, 18 km desde 25 000 ft)
    Lanzamiento: 15 000-25 000 ft
employment-8 =
    Alcance: 0,9-1,3 km antes de la pista
    Lanzamiento: 200-500 ft
employment-9 =
    Alcance: 5-11 km antes del objetivo
    Lanzamiento: 3000-20 000 ft
employment-12 =
    Alcance: 0,9-2,2 km antes del objetivo
    Lanzamiento: 100-500 ft
employment-16 = Alcance: fotos a 2-8 km (cuantos más adversarios, más cerca), morro hacia el objetivo
employment-18 = Alcance: lanzar con «Drop supplies now», a menos de 4 km de la zona de lanzamiento
employment-20 =
    Alcance: 4-8 km
    Disparo: 2000-3000 ft
employment-guided-cluster =
    Alcance: de 1,5 km a 2,5 veces la altura, LOCK estable (3,7 km desde 3000 ft, 6 km desde 5000 ft)
    Lanzamiento: 3000-5000 ft
employment-medium-retarded =
    Alcance: 1,1-1,6 km antes del objetivo
    Lanzamiento: 300-1000 ft
employment-low-retarded =
    Alcance: 0,9-1,3 km antes del objetivo
    Lanzamiento: 200-500 ft

## Puntos de anclaje

hardpoint-left-bay = Bodega izquierda
hardpoint-right-bay = Bodega derecha
hardpoint-left-wing-bay = Bodega alar izquierda
hardpoint-right-wing-bay = Bodega alar derecha
hardpoint-station-1-left-wingtip = Estación 1, punta del ala izquierda
hardpoint-station-2-left-outer-wing = Estación 2, ala izquierda exterior
hardpoint-station-3-left-wing = Estación 3, ala izquierda
hardpoint-station-4-left-inner-wing = Estación 4, ala izquierda interior
hardpoint-station-5-centreline = Estación 5, ventral
hardpoint-station-6-right-inner-wing = Estación 6, ala derecha interior
hardpoint-station-7-right-wing = Estación 7, ala derecha
hardpoint-station-8-right-outer-wing = Estación 8, ala derecha exterior
hardpoint-station-9-right-wingtip = Estación 9, punta del ala derecha
hardpoint-station-1-left-wingtip-rail = Estación 1, raíl de punta de ala izquierda
hardpoint-station-2-left-outer-pylon = Estación 2, pilón izquierdo exterior
hardpoint-station-3-left-inner-pylon = Estación 3, pilón izquierdo interior
hardpoint-station-4-right-inner-pylon = Estación 4, pilón derecho interior
hardpoint-station-5-right-outer-pylon = Estación 5, pilón derecho exterior
hardpoint-station-6-right-wingtip-rail = Estación 6, raíl de punta de ala derecha
hardpoint-tornado-left-wing-rail = Ala izquierda, raíl de Sidewinder
hardpoint-tornado-left-wing-pylon = Pilón del ala izquierda
hardpoint-tornado-left-shoulder-forward = Hombro izquierdo, delantero
hardpoint-tornado-left-shoulder-aft = Hombro izquierdo, trasero
hardpoint-tornado-right-shoulder-aft = Hombro derecho, trasero
hardpoint-tornado-right-shoulder-forward = Hombro derecho, delantero
hardpoint-tornado-right-wing-pylon = Pilón del ala derecha
hardpoint-tornado-right-wing-rail = Ala derecha, raíl de Sidewinder
hardpoint-a10-1-left-outer-wing = Estación 1, ala izquierda exterior
hardpoint-a10-2-left-wing = Estación 2, ala izquierda
hardpoint-a10-3-left-wing = Estación 3, ala izquierda
hardpoint-a10-4-left-centre-wing = Estación 4, ala izquierda central
hardpoint-a10-5-left-fuselage = Estación 5, fuselaje izquierdo
hardpoint-a10-6-centreline = Estación 6, ventral
hardpoint-a10-7-right-fuselage = Estación 7, fuselaje derecho
hardpoint-a10-8-right-centre-wing = Estación 8, ala derecha central
hardpoint-a10-9-right-wing = Estación 9, ala derecha
hardpoint-a10-10-right-wing = Estación 10, ala derecha
hardpoint-a10-11-right-outer-wing = Estación 11, ala derecha exterior
hardpoint-av8b-1-left-rail = Estación 1, raíl Sidewinder izquierdo
hardpoint-av8b-3-left-middle-wing = Estación 3, ala izquierda intermedia
hardpoint-av8b-4-left-inboard-wing = Estación 4, ala izquierda interior
hardpoint-av8b-6-right-inboard-wing = Estación 6, ala derecha interior
hardpoint-av8b-7-right-middle-wing = Estación 7, ala derecha intermedia
hardpoint-av8b-9-right-rail = Estación 9, raíl Sidewinder derecho
hardpoint-f22-left-side-bay = Bodega lateral izquierda (Sidewinder)
hardpoint-f22-left-main-bay = Bodega principal, mitad izquierda
hardpoint-f22-right-main-bay = Bodega principal, mitad derecha
hardpoint-f22-right-side-bay = Bodega lateral derecha (Sidewinder)
hardpoint-fa18e-8-right-inner-wing = Estación 8, ala derecha interior
hardpoint-fa18e-10-right-outer-wing = Estación 10, ala derecha exterior
hardpoint-fa18e-11-right-wingtip = Estación 11, punta del ala derecha
hardpoint-c5-hold = Bodega de carga (palés de lanzamiento)
hardpoint-ch47-cabin = Cabina (paracaidistas)
weapon-role-21 = Aire-aire, guiado infrarrojo, alcance muy corto
employment-21 = Alcance: 1-6 km, más en frontal
loadout-none-in-stock = (sin existencias)
loadout-empties = Vacía la estación { $stations } al cargar
loadout-empties-two = Vacía las estaciones { $first } y { $second } al cargar
hardpoint-oh58-left-pylon = Pilón de armas izquierdo
hardpoint-oh58-right-pylon = Pilón de armas derecho
hardpoint-mh60r-left-outer = Estación exterior izquierda (lanzador Hellfire)
hardpoint-mh60r-left-inner = Estación interior izquierda
hardpoint-mh60r-right-inner = Estación interior derecha
weapon-role-22 = Vaina de ametralladora: se dispara como el cañón, a lo largo del morro
employment-22 = Alcance: hasta 1,5 km, 500 cartuchos por vaina
weapon-role-23 = Arma de espoleta sensora: cuarenta skeets disparan hacia abajo contra los vehículos
employment-23 =
    Guiada al punto marcado con el pod de designación al lanzarla
    Lanzamiento: por encima de la altitud de apertura de la página SMS (500-3.000 ft)
weapon-role-24 = Torpedo ligero: cae en paracaídas, busca bajo el agua y se dirige a un submarino
employment-24 =
    Se programa en la página WPN del operador de sensores: rumbo, retardo de activación, patrón de búsqueda
    Lanzamiento: sobre el agua, por debajo de 300 ft y 100 kt
weapon-role-25 = Misil de ataque terrestre a distancia: vuela hasta su objetivo y luego el piloto lo guía con la imagen de su buscador
employment-25 =
    Se programa en la página SLAM: altitud de crucero, distancia del buscador, objetivo principal o secundario
    Alcance: hasta 250 km, cuatro en vuelo a la vez
weapon-role-26 = Bomba planeadora: planea hasta el punto designado; su alcance crece con la altura y la velocidad de lanzamiento
employment-26 =
    Apunta al punto del pod o al blanco fijado en tierra, si no al objetivo principal
    Alcance: unos 20 km a baja cota, hasta 130 km desde gran altura; la defensa aérea puede derribarla
weapon-role-27 = Señuelo lanzado desde el aire: muestra la firma de un avión a la defensa aérea y atrae sus misiles
employment-27 =
    Vuela hasta el punto designado y luego sigue recto hasta 10 minutos
    Inofensivo para los objetivos
weapon-role-28 = Paracaidistas: un grupo de soldados para lanzar sobre un lugar que toman
employment-28 =
    Por las puertas del C-5 (dos grupos) o la rampa del CH-47 (un grupo) sobre la zona de salto
    Salto entre 500 y 1500 ft sobre el suelo, por debajo de 150 kt; la mayor parte del grupo debe caer en la zona
weapon-role-29 = Contenedor de perturbación táctica: reduce los radares perturbados a su alcance de penetración
employment-29 =
    Se ajusta en la página JAM: un radar o un sector por contenedor, encendido o apagado
    Perturbar te hace visible desde más lejos
weapon-role-30 = Dispensador de ataque a aeródromos: siembra bombetas de cráter y minas anticarro con paracaídas en una franja de 500 m
employment-30 =
    Se queda en el pilón y lanza durante dos segundos; vuela nivelado y bajo sobre el objetivo
    Desde no más de 1000 ft sobre el suelo; tres cráteres cierran una pista, las minas esperan a los vehículos
weapon-role-31 = Cohetes de 70 mm no guiados en lanzador: por parejas o en ráfagas contra vehículos y tropas al descubierto
employment-31 =
    Apunte con la retícula de cohetes (CCIP): muestra dónde cae un cohete disparado ahora
    Pulse para una pareja, mantenga para una ráfaga; mejor en picado suave a menos de 4 km
weapon-role-32 = Cohetes de 70 mm guiados por láser: cada uno se dirige al punto láser del pod de designación
employment-32 =
    Ponga el pod de designación sobre el blanco y dispare hacia el frente; giran hacia el punto
    Hasta 6 km; mantenga el pod sobre el blanco hasta el impacto
