# Съобщенията на играта на HUD и 2D наслагванията на полета (src/hud, src/combat.rs,
# src/csar_view.rs, src/campaign). Думите на авиониката (CCIP, THR, KM, MAX...) остават на
# английски.

hud-no-ejection-seat = НЯМА КАТАПУЛТНА СЕДАЛКА
hud-waypoint-reached = Waypoint { $waypoint } е достигнат
hud-select-camera = ИЗБЕРЕТЕ КАМЕРА
hud-open-bay = ОТВОРЕТЕ ОТСЕКА
hud-camera-page = СТРАНИЦА { $page }
hud-target-in-view-shoot = ЦЕЛ В ЗРЕНИЕ - СНИМАЙТЕ
hud-target-in-view = ЦЕЛ В ЗРЕНИЕ
hud-aim-at-target = ЦЕЛЕТЕ СЕ В ЦЕЛТА
hud-photo = СНИМКА { $state }
hud-rearmed = Превъоръжен и зареден
hud-landed-rearm = Кацнал - превъоръжете или завършете полета
hud-objective-achieved = Целта е постигната

## Екран за край на мисията

end-accomplished = изпълнена
end-not-accomplished = неизпълнена
end-primary = Основна цел: { $result }
end-secondary = Второстепенна цел: { $result }
end-kills = Поразени: { $air } самолета, { $ground } наземни цели
end-time = Време на мисията { $time }
end-damage = Повреди: { $hits } попадения
end-training = Тренировъчна мисия
end-press-any-key = Натиснете произволен клавиш

## Панелът на сваления пилот

csar-flare = СИГНАЛНА РАКЕТА
csar-faster = ПО-БЪРЗО
csar-normal = НОРМАЛНО ВРЕМЕ
csar-skip = ПРОПУСНИ ЕВАКУАЦИЯТА
csar-parachute = ПАРАШУТЪТ Е ОТВОРЕН
csar-on-ground = НА ЗЕМЯТА
csar-down = СВАЛЕН ВЪВ ВРАЖЕСКА ТЕРИТОРИЯ
csar-pickup = СПАСЯВАНЕ   UH-60 НАД ВАС - ВЗИМАНЕ
csar-inbound = СПАСЯВАНЕ   UH-60 { $km } KM { $direction } - ПРИБЛИЖАВА
csar-fire-flare = СПАСЯВАНЕ   UH-60 { $km } KM { $direction } - ПУСНЕТЕ СИГНАЛНА РАКЕТА, КОГАТО Е БЛИЗО
csar-enemy = ВРАГ    { $count } HIND, НАЙ-БЛИЗО { $km } KM
csar-flares = РАКЕТИ   { $count }      НА ЗЕМЯТА { $time }

## Известия в коопа на панела (src/player_notices.rs), бележка на конзолата за противовъздушна отбрана

notice-joined = { $pilot } се присъедини
notice-left = { $pilot } напусна
notice-lost-connection = { $pilot } загуби връзка
st-select-track = Първо изберете следа
