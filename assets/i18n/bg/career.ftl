# Кариери на пилоти (src/career.rs): звания, отличия, срокът на служба.

## Звания във ВВС на САЩ (преведени с българските им еквиваленти), съкращения

rank-0 = Лейтенант
    .short = лейт.
rank-1 = Старши лейтенант
    .short = ст. лейт.
rank-2 = Капитан
    .short = кап.
rank-3 = Майор
    .short = май.
rank-4 = Подполковник
    .short = подп.
rank-5 = Полковник
    .short = полк.

## Отличия (американски медали: собствените им имена)

medal-air-medal = Air Medal
medal-distinguished-flying-cross = Distinguished Flying Cross
medal-silver-star = Silver Star
medal-air-force-cross = Air Force Cross
medal-medal-of-honor = Medal of Honor
medal-purple-heart = Purple Heart
medal-prisoner-of-war-medal = Prisoner of War Medal

## Новини след полет

career-awarded = Награда: { $medal }
career-kia = Загинал в бой
    .short = ЗВБ
career-pow = Военнопленник
    .short = ВП
career-active = На активна служба
career-captured = Взет в плен: кариерата приключи
career-tour-sortie = Срок на служба: полет { $sortie } от { $length }
career-tour-complete = Срокът на служба приключи: { $points } точки (+{ $bonus })
career-promoted = Повишение: { $rank }

## Прозорецът на кариерата

career-title = ДОСИЕТА НА ПИЛОТИТЕ
career-group-tour = Срок
career-set-by-tour = Зададено от срока на пилота
career-pilots = Пилоти
career-new-pilot = Нов пилот
career-name = Име
career-enlist = ПОСТЪПВАНЕ НА СЛУЖБА
career-select = Изберете пилот или го запишете на служба
career-service-record = Служебно досие
career-rank = ЗВАНИЕ
career-points = ТОЧКИ
career-sorties = ПОЛЕТИ
career-tours = СРОКОВЕ
career-status = СЪСТОЯНИЕ
career-decorations = Отличия
career-tour-of-duty = Срок на служба
career-tour-line = { $theater }  { $situation }  полет { $sortie } от { $length }  { $points } точки
career-destroyed = Унищожени цели досега: { $count }
career-fly-next = СЛЕДВАЩИЯТ ПОЛЕТ
career-new-tour = { $count } полета в театър { $theater }, { $situation }, противници: { $opponents }
    .facts = Каквото унищожите, остава унищожено до края на срока
career-start-tour = ЗАПОЧВАНЕ НА СРОК
career-retired = Освободен от активна служба
career-last-sorties = Последни полети
