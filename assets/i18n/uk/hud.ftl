# Повідомлення гри на HUD та 2D-накладках польоту (src/hud, src/combat.rs,
# src/csar_view.rs, src/campaign). HUD малює їх великими літерами; власні слова
# авіоніки (CCIP, THR, KM, MAX...) залишаються англійською.

hud-no-ejection-seat = НЕМАЄ КАТАПУЛЬТНОГО КРІСЛА
hud-waypoint-reached = Waypoint { $waypoint } досягнуто
hud-select-camera = ОБЕРІТЬ КАМЕРУ
hud-open-bay = ВІДКРИЙТЕ ВІДСІК
hud-camera-page = СТОРІНКА { $page }
hud-target-in-view-shoot = ЦІЛЬ У ПОЛІ ЗОРУ - ЗНІМАЙТЕ
hud-target-in-view = ЦІЛЬ У ПОЛІ ЗОРУ
hud-aim-at-target = НАВЕДІТЬ НА ЦІЛЬ
hud-photo = ФОТО { $state }
hud-rearmed = Переозброєно та заправлено
hud-landed-rearm = Посадка виконана - переозброюйтесь або завершіть вильот
hud-objective-achieved = Завдання виконано

## Екран завершення місії

end-accomplished = виконано
end-not-accomplished = не виконано
end-primary = Основне завдання: { $result }
end-secondary = Додаткове завдання: { $result }
end-kills = Знищено: літаків { $air }, наземних цілей { $ground }
end-time = Час місії { $time }
end-damage = Пошкодження: влучань { $hits }
end-training = Навчальна місія
end-press-any-key = Натисніть будь-яку клавішу

## Панель збитого пілота

csar-flare = СИГНАЛЬНА РАКЕТА
csar-faster = ШВИДШЕ
csar-normal = НОРМАЛЬНИЙ ЧАС
csar-skip = ПРОПУСТИТИ ЕВАКУАЦІЮ
csar-parachute = ПАРАШУТ РОЗКРИТО
csar-on-ground = НА ЗЕМЛІ
csar-down = НА ЗЕМЛІ НА ВОРОЖІЙ ТЕРИТОРІЇ
csar-pickup = РЯТУВАННЯ   UH-60 НАД ВАМИ - ПІДБІР
csar-inbound = РЯТУВАННЯ   UH-60 { $km } KM { $direction } - ПРЯМУЄ ДО ВАС
csar-fire-flare = РЯТУВАННЯ   UH-60 { $km } KM { $direction } - ВИПУСТІТЬ РАКЕТУ, КОЛИ БУДЕ БЛИЗЬКО
csar-enemy = ВОРОГ    { $count } HIND, НАЙБЛИЖЧИЙ { $km } KM
csar-flares = РАКЕТИ   { $count }      НА ЗЕМЛІ { $time }

## Сповіщення кооперативу на панелі, примітка консолі протиповітряної оборони

notice-joined = { $pilot } приєднався
notice-left = { $pilot } вийшов
notice-lost-connection = { $pilot } втратив з'єднання
st-select-track = Спершу оберіть трек
