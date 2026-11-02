# Карьера пилотов (src/career.rs): звания, награды, служебная командировка.

## Звания US Air Force (в каждом языке переводятся равными званиями), их сокращения

rank-0 = Лейтенант
    .short = л-т
rank-1 = Старший лейтенант
    .short = ст. л-т
rank-2 = Капитан
    .short = к-н
rank-3 = Майор
    .short = м-р
rank-4 = Подполковник
    .short = п/п-к
rank-5 = Полковник
    .short = п-к

## Награды (американские медали: их собственные названия)

medal-air-medal = Air Medal
medal-distinguished-flying-cross = Distinguished Flying Cross
medal-silver-star = Silver Star
medal-air-force-cross = Air Force Cross
medal-medal-of-honor = Medal of Honor
medal-purple-heart = Purple Heart
medal-prisoner-of-war-medal = Prisoner of War Medal

## Новости после вылета

career-awarded = Награда: { $medal }
career-kia = Погиб в бою
    .short = KIA
career-pow = Военнопленный
    .short = POW
career-active = На действительной службе
career-captured = Попал в плен: карьера окончена
career-tour-sortie = Служебная командировка: вылет { $sortie } из { $length }
career-tour-complete = { $points ->
    [one] Служебная командировка завершена: { $points } очко (+{ $bonus })
    [few] Служебная командировка завершена: { $points } очка (+{ $bonus })
    [many] Служебная командировка завершена: { $points } очков (+{ $bonus })
   *[other] Служебная командировка завершена: { $points } очка (+{ $bonus })
}
career-promoted = Присвоено звание: { $rank }

## Окно карьеры

career-title = ЛИЧНЫЕ ДЕЛА ПИЛОТОВ
career-group-tour = Командировка
career-set-by-tour = Задано командировкой пилота
career-pilots = Пилоты
career-new-pilot = Новый пилот
career-name = Имя
career-enlist = ЗАЧИСЛИТЬ
career-select = Выберите или зачислите пилота
career-service-record = Послужной список
career-rank = ЗВАНИЕ
career-points = ОЧКИ
career-sorties = ВЫЛЕТЫ
career-tours = КОМАНДИРОВКИ
career-status = СТАТУС
career-decorations = Награды
career-tour-of-duty = Служебная командировка
career-tour-line = { $points ->
    [one] { $theater }  { $situation }  вылет { $sortie } из { $length }  { $points } очко
    [few] { $theater }  { $situation }  вылет { $sortie } из { $length }  { $points } очка
    [many] { $theater }  { $situation }  вылет { $sortie } из { $length }  { $points } очков
   *[other] { $theater }  { $situation }  вылет { $sortie } из { $length }  { $points } очка
}
career-destroyed = { $count ->
    [one] На данный момент уничтожена { $count } цель
    [few] На данный момент уничтожены { $count } цели
    [many] На данный момент уничтожено { $count } целей
   *[other] На данный момент уничтожено { $count } цели
}
career-fly-next = СЛЕДУЮЩИЙ ВЫЛЕТ
career-new-tour = { $count ->
    [one] { $count } вылет на ТВД «{ $theater }», { $situation }, противник: { $opponents }
    [few] { $count } вылета на ТВД «{ $theater }», { $situation }, противник: { $opponents }
    [many] { $count } вылетов на ТВД «{ $theater }», { $situation }, противник: { $opponents }
   *[other] { $count } вылета на ТВД «{ $theater }», { $situation }, противник: { $opponents }
}
    .facts = Уничтоженное остаётся уничтоженным до конца командировки
career-start-tour = НАЧАТЬ КОМАНДИРОВКУ
career-retired = В отставке
career-last-sorties = Последние вылеты
