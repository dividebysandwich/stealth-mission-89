# Carreras de los pilotos (src/career.rs): empleos, condecoraciones, período de servicio.

## Empleos de la US Air Force (por sus equivalentes del Ejército del Aire y del Espacio), abreviaturas

rank-0 = Alférez
    .short = Alf.
rank-1 = Teniente
    .short = Tte.
rank-2 = Capitán
    .short = Cap.
rank-3 = Comandante
    .short = Cte.
rank-4 = Teniente coronel
    .short = Tcol.
rank-5 = Coronel
    .short = Col.

## Condecoraciones (medallas estadounidenses: sus nombres propios)

medal-air-medal = Air Medal
medal-distinguished-flying-cross = Distinguished Flying Cross
medal-silver-star = Silver Star
medal-air-force-cross = Air Force Cross
medal-medal-of-honor = Medal of Honor
medal-purple-heart = Purple Heart
medal-prisoner-of-war-medal = Prisoner of War Medal

## Noticias tras una salida

career-awarded = Condecorado: { $medal }
career-kia = Muerto en combate
    .short = KIA
career-pow = Prisionero de guerra
    .short = POW
career-active = En servicio activo
career-captured = Hecho prisionero: la carrera ha terminado
career-tour-sortie = Período de servicio: salida { $sortie } de { $length }
career-tour-complete = Período de servicio completado: { $points } puntos (+{ $bonus })
career-promoted = Ascendido a { $rank }

## La ventana de carrera

career-title = EXPEDIENTES DE LOS PILOTOS
career-group-tour = Período
career-set-by-tour = Fijado por el período del piloto
career-pilots = Pilotos
career-new-pilot = Nuevo piloto
career-name = Nombre
career-enlist = ALISTAR
career-select = Seleccionar o alistar un piloto
career-service-record = Hoja de servicios
career-rank = RANGO
career-points = PUNTOS
career-sorties = SALIDAS
career-tours = PERÍODOS
career-status = ESTADO
career-decorations = Condecoraciones
career-tour-of-duty = Período de servicio
career-tour-line = { $theater }  { $situation }  salida { $sortie } de { $length }  { $points } puntos
career-destroyed = { $count } objetivos destruidos hasta ahora
career-fly-next = VOLAR LA SIGUIENTE SALIDA
career-new-tour = { $count } salidas en { $theater }, { $situation }, contra adversarios de nivel { $opponents }
    .facts = Lo que se destruye queda destruido hasta el final del período
career-start-tour = INICIAR PERÍODO
career-retired = Retirado del servicio activo
career-last-sorties = Últimas salidas
