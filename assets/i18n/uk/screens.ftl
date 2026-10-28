# Планшет на коліно (src/kneeboard.rs, штриховий шрифт), планувальник місій (src/planner.rs) та
# збережені місії (src/savegame.rs). Навігаційні скорочення (PT, BRG, ETA, PRI, SEC, KM, KT)
# залишаються англійською.

kb-brief = БРИФІНГ
kb-route = МАРШРУТ
kb-map = КАРТА
kb-target = ЦІЛЬ
kb-keys = КЛАВІШІ
kb-tip = Карти, нотатки до місії та посібник із клавіш
kb-geran-file = GERAN-2 (ДОСЬЄ)
kb-objectives = ЗАВДАННЯ
kb-primary = ОСНОВНЕ: { $objective }
kb-secondary = ДОДАТКОВЕ: { $objective }
kb-times = ЧАС
kb-deadline = КІНЦЕВИЙ ТЕРМІН { $time }
kb-now = ЗАРАЗ { $time }
kb-conditions = УМОВИ
kb-moon = МІСЯЦЬ: { $phase } (ОСВІТЛЕНО { $lit }%)
kb-armament = ОЗБРОЄННЯ
kb-roe = ПРАВИЛА ВЕДЕННЯ ВОГНЮ
kb-total = УСЬОГО { $km } KM ПРИ { $kt } KT
kb-fuel = ПАЛИВО { $lbs } LBS
kb-preparing = ПІДГОТОВКА...
kb-airborne = У ПОВІТРІ
kb-developing = ПРОЯВЛЕННЯ...
kb-no-photo = НЕМАЄ ФОТО
kb-airborne-target = ПОВІТРЯНА ЦІЛЬ
kb-scroll-vr = ПРОКРУТКА: СТІК, ПАЛЕЦЬ НА ПЛАНШЕТІ
kb-scroll-desktop = ПРОКРУТКА: КОЛІСЦЕ МИШІ НАД ПЛАНШЕТОМ

## Планувальник місій

planner-low = НИЗЬКО 500 FT
planner-medium = СЕРЕД 5000 FT
planner-high = ВИСОКО 20000 FT
planner-title = ПЛАНУВАЛЬНИК МІСІЙ
planner-route = Маршрут
planner-base = БАЗА
planner-total = УСЬОГО { $km } KM  { $time }
planner-briefed = Маршрут за брифінгом: { $km } км, паливо { $fuel } lbs
planner-coverage = Зона дії радарів для
    .note = Зона дії радарів розрахована для горизонтального польоту на 480 kt із закритими відсіками.
planner-detected = Виявлено на маршруті: { $km } км
planner-pulse = Дальність виявлення імпульсним радаром (P)
planner-doppler = Дальність виявлення доплерівським радаром (D)
planner-missile = Дальність ракети
planner-uav = Ворожий БпЛА
planner-edit-hint = Перетягніть точку, щоб перемістити її, карту - щоб зсунути. Клік по ділянці додає точку, правий клік видаляє її. Колесо: масштаб.
planner-reset = СКИНУТИ
planner-view-hint = Перетягніть карту, щоб зсунути. Колесо: масштаб.

## Збережені місії

saves-title = ЗБЕРЕЖЕНІ МІСІЇ
saves-subtitle = Збережено в польоті з меню: SAVE MISSION
saves-none = Немає збережених місій
saves-line = { $theater }  -  { $aircraft }  -  час місії { $time }
saves-saved-at = Збережено { $date }
saves-unreadable = Збереження не читається (інша версія)
saves-delete =
    .facts = Видалити це збереження
saves-back =
    .facts = Назад до екрана окремої місії
