# Кар'єра пілотів (src/career.rs): звання, нагороди, термін служби.

## Звання ПС США (замінені відповідниками Повітряних сил України), абревіатури

rank-0 = Лейтенант
    .short = лейт.
rank-1 = Старший лейтенант
    .short = ст. лейт.
rank-2 = Капітан
    .short = кап.
rank-3 = Майор
    .short = май.
rank-4 = Підполковник
    .short = підп.
rank-5 = Полковник
    .short = полк.

## Нагороди (американські медалі: власні назви)

medal-air-medal = Air Medal
medal-distinguished-flying-cross = Distinguished Flying Cross
medal-silver-star = Silver Star
medal-air-force-cross = Air Force Cross
medal-medal-of-honor = Medal of Honor
medal-purple-heart = Purple Heart
medal-prisoner-of-war-medal = Prisoner of War Medal

## Новини після вильоту

career-awarded = Нагороджено: { $medal }
career-kia = Загинув у бою
    .short = ЗУБ
career-pow = Військовополонений
    .short = ВП
career-active = На дійсній службі
career-captured = Потрапив у полон: кар'єру завершено
career-tour-sortie = Термін служби: виліт { $sortie } з { $length }
career-tour-complete = Термін служби завершено: { $points } балів (+{ $bonus })
career-promoted = Підвищено до звання: { $rank }

## Вікно кар'єри

career-title = ДОСЬЄ ПІЛОТІВ
career-group-tour = Термін
career-set-by-tour = Задано терміном служби пілота
career-pilots = Пілоти
career-new-pilot = Новий пілот
career-name = Ім'я
career-enlist = ЗАВЕРБУВАТИ
career-select = Оберіть або завербуйте пілота
career-service-record = Послужний список
career-rank = ЗВАННЯ
career-points = БАЛИ
career-sorties = ВИЛЬОТИ
career-tours = ТЕРМІНИ
career-status = СТАТУС
career-decorations = Нагороди
career-tour-of-duty = Термін служби
career-tour-line = { $theater }  { $situation }  виліт { $sortie } з { $length }  { $points } балів
career-destroyed = Досі знищено цілей: { $count }
career-fly-next = НАСТУПНИЙ ВИЛЬОТ
career-new-tour = Вильотів: { $count }, театр воєнних дій: { $theater }, ситуація: { $situation }, супротивники: { $opponents }
    .facts = Те, що ви знищили, лишається знищеним до кінця терміну служби
career-start-tour = ПОЧАТИ ТЕРМІН СЛУЖБИ
career-retired = Звільнений з дійсної служби
career-last-sorties = Останні вильоти
