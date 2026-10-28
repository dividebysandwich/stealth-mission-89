# Wiadomości, dziennik, tablica misji i odmowy kampanii dynamicznej (crates/stealth-campaign,
# stealth-host, stealth-net): tworzone na serwerze, tłumaczone przez grę każdego gracza. Nazwy
# miejsc i pilotów są przekazywane bez zmian.

## Wiadomości i dziennik

cl-campaign-started = Kampania rozpoczęta
cl-a-pilot = Pilot
cl-exclaim = { $text }!
cl-airfield-destroyed = Lotnisko { $place } zniszczone
cl-sam-destroyed = Stanowisko SAM - { $place } - zniszczone
cl-ship-sunk = Okręt zatopiony
cl-why-all-lost = utracono wszystkie śmigłowce
cl-why-landed-short = śmigłowce wylądowały przed celem
cl-why-lost = śmigłowce zostały utracone
cl-why-turned-back = śmigłowce zawróciły
cl-heli-lost = Stracono śmigłowiec - { $place }: pozostało { $left }
cl-enemy-heli-down = Zestrzelono śmigłowiec wroga - { $place }: pozostało { $left }
cl-insertion-failed-why = Desant - { $place } - nieudany: { $why }
cl-insertion-failed = Desant - { $place } - nieudany
cl-enemy-insertion-stopped = Desant wroga - { $place } - powstrzymany
cl-captured = Zdobyto: { $place }
cl-lost = Utracono na rzecz wroga: { $place }
cl-insertion-ordered = Zlecono desant - { $place }: śmigłowce startują z { $base } za { $mins } min
cl-insertion-command = Desant - { $place } - zlecony przez dowództwo: śmigłowce startują z { $base } za { $mins } min
cl-enemy-insertion-intel = Wywiad: śmigłowce wroga startują z { $base } za { $mins } min, cel: { $place }
cl-insertion-leads = Desant - { $place }: śmigłowce prowadzi { $who }
cl-helis-on-way = Śmigłowce w drodze, cel: { $place }
cl-helis-lift-off = Śmigłowce desantowe startują, cel: { $place }
cl-enemy-helis-on-way = Śmigłowce wroga w drodze, cel: { $place }
cl-enemy-helis-inbound = Śmigłowce wroga nadlatują, cel: { $place }!
cl-fighters-scrambled = Alarmowy start myśliwców wroga
cl-insertion-called-off = Desant - { $place } - odwołany: brak śmigłowców
cl-enemy-insertion-called-off = Desant wroga - { $place } - odwołany
cl-status-pending = Śmigłowce startują z { $base } za { $mins } min, potem ok. { $flight } min lotu, cel: { $place }
cl-status-enroute = Śmigłowce z { $base } nadlatują, cel: { $place }, przylot za ok. { $mins } min
cl-sam-repaired = { $place }: stanowisko SAM naprawione
cl-airfield-repaired = { $place }: lotnisko naprawione
cl-cap-scrambled = Alarmowy start CAP z { $base }
cl-raid-intel = Wywiad: samoloty uderzeniowe wroga zaatakują lotnisko { $place } za { $mins } min
cl-raid-turned-back = { $raid } zawróciły - { $place }
cl-raid-on-way = { $raid } w drodze, cel: { $place }
cl-raid-inbound = { $raid } nadlatują, cel: { $place }!
cl-raid-hit = { $raid } trafiły cel: { $target }
cl-target-generic = cel
cl-target-airfield = lotnisko { $place }
cl-target-city = miasto { $place }
cl-target-sam = stanowisko SAM ({ $place })
cl-raid-missed = { $raid } nie przedarły się - { $place }
cl-drones-stopped = Atak dronów - { $place } - powstrzymany
cl-raid-repelled = Nalot wroga - { $place } - odparty
cl-victory = ZWYCIĘSTWO: wróg nie ma już żadnego lotniska
cl-defeat = PORAŻKA: nie zostało żadne sojusznicze lotnisko
cl-raid-ai-sead = Samoloty SEAD SI
cl-raid-drones = Drony wroga
cl-raid-strike = Samoloty uderzeniowe wroga
cl-battery-destroyed = Bateria wroga - { $place } - zniszczona
cl-battery-intel = Wywiad: wróg wysunął baterię działek w rejon: { $place }
cl-awacs-home = AWACS wroga zawraca do bazy
cl-awacs-intel = Wywiad: AWACS wroga startuje za linią: { $place }; naprowadza myśliwce wroga, dopóki nie zostanie zestrzelony
cl-awacs-down = AWACS wroga zestrzelony
cl-awacs-down-news = AWACS wroga zestrzelony!
cl-awacs-landed = AWACS wroga wylądował
cl-awacs-landed-news = AWACS wroga wylądował
cl-drones-intel = Wywiad: drony szturmowe wroga zostaną wystrzelone za { $mins } min, cel: { $target }
cl-support = { $who } wzywa SEAD SI - { $place }: samoloty uderzeniowe z { $base }: { $count }, za { $mins } min
cl-sortie-from = { $title } - start: { $base }
cl-verdict-success = cel osiągnięty
cl-verdict-failed = cel nieosiągnięty
cl-verdict-raid = oczekiwanie na nalot
cl-verdict-rescue = oczekiwanie na ratunek
cl-verdict-insertion = oczekiwanie na desant
cl-sortie-ended = { $title }: { $end } - { $verdict }
cl-troops-out = Żołnierze wysadzeni - { $place }
cl-aboard = { $pilot } jest na pokładzie: odwieź go do bazy
cl-objective = { $title }: cel osiągnięty
cl-recon = Zdjęcia rozpoznawcze - { $place }: obrona miejsca rozpoznana
cl-package = Pakiet - { $place }: { $kinds } - +{ $bonus } dla: { $pilots }
cl-supply-patriot = bateria Patriot ({ $place })
cl-supply-gepard = Gepard ({ $place })
cl-resupply-requested = { $who } prosi o zaopatrzenie dla: { $unit }
cl-resupply-out-of-missiles = Brak rakiet w jednostce: { $unit } - zaopatrzenie zamówione
cl-resupply-out-of-rounds = Brak amunicji w jednostce: { $unit } - zaopatrzenie zamówione
cl-resupply-lost-unit = Utracono: { $unit } - zaopatrzenie odwołane
cl-resupply-ai = UH-60 z { $base } przejmuje zaopatrzenie dla: { $unit }
cl-resupply-heli-lost = Utracono śmigłowiec z zaopatrzeniem dla: { $unit } - zadanie wraca na tablicę
cl-resupplied = Zaopatrzono: { $unit }
cl-resupplied-by = Zaopatrzono: { $unit } (dostarczył: { $who })
cl-pilot-down-near = { $pilot } na ziemi - { $place }: ratunek potrzebny w ciągu { $mins } min
cl-ai-rescue = Śmigłowiec ratowniczy wydostał pilota: { $pilot }
cl-pilot-captured = { $pilot } został pojmany przez wroga
cl-lost-with-heli = { $pilot } zginął wraz ze śmigłowcem
cl-rescued = { $pilot } uratowany!
cl-pilot-down = { $pilot } na ziemi na terytorium wroga: śmigłowiec ratowniczy jest w drodze
cl-extracted-fighter = { $pilot } ewakuowany - odzyskano zasoby: { $materiel } i myśliwiec
cl-extracted-materiel = { $pilot } ewakuowany - odzyskano zasoby: { $materiel }
cl-extracted = { $pilot } ewakuowany

## Jak zakończył się lot bojowy (dziennik) i tytuł omówienia

sortie-end-landed = Lądowanie
sortie-end-ejected = Katapultowanie
sortie-end-crashed = Zestrzelony / rozbity
sortie-end-aborted = Przerwany
sortie-end-rescued = Katapultowanie - ewakuowany
sortie-end-captured = Katapultowanie - pojmany
debrief-sortie-ejected = Katapultowanie
debrief-sortie-rescued = Katapultowanie - ewakuowany przez śmigłowiec ratowniczy
debrief-sortie-captured = Katapultowanie - pojmany przez wroga
debrief-sortie-landed = Lot bojowy zakończony na ziemi
debrief-sortie-ejection-failed = Nieudane katapultowanie - pilot odzyskany
debrief-sortie-aborted = Lot bojowy przerwany
debrief-sortie-bad-landing = Nieudane lądowanie
debrief-sortie-crashed = Zestrzelony / rozbity - pilot odzyskany

## Tytuły i rodzaje misji na tablicy

cb-intercept = Przechwycenie - { $place }
cb-drone-defense = Obrona przed dronami - { $place }
cb-defend = Obrona - lotnisko { $place }
cb-awacs = AWACS - zestrzelić
cb-rescue = Ratunek - { $place }
cb-recon = Rozpoznanie - { $place }
cb-sead = SEAD - { $place }
cb-cas = CAS - { $place }
cb-strike = Uderzenie - lotnisko { $place }
cb-insertion = Desant - { $place }
cb-interdiction = Izolacja - { $place }
cb-deep-strike = Głębokie uderzenie - lotnisko { $place }
cb-cas-battery = CAS - bateria ({ $place })
cb-escort = Eskorta - desant ({ $place })
cb-anti-ship = Zwalczanie okrętów - okręt wroga
cb-cap = CAP - { $place }
cb-insertion-of = desant - { $place }
cb-resupply = Zaopatrzenie - { $place }
ck-recon = ROZPOZN.
ck-sead = SEAD
ck-cap = CAP
ck-cas = CAS
ck-anti-ship = OKRĘTY
ck-strike = UDERZENIE
ck-escort = ESKORTA
ck-insertion = DESANT
ck-intercept = PRZECHWYT
ck-defend = OBRONA
ck-interdict = IZOLACJA
ck-csar = CSAR
ck-awacs = AWACS
ck-resupply = ZAOPATRZ.

## Odrzucone polecenia

ce-uh60-only = UH-60 wykonuje desanty, akcje ratunkowe i zaopatrzenie
ce-resupply-uh60 = Zaopatrzenie wykonuje się w UH-60
ce-insertion-uh60 = Desanty wykonuje się w UH-60 lub zleca załogom śmigłowców
ce-no-place = Nie ma takiego miejsca
ce-not-enemy = { $place } nie jest w rękach wroga
ce-not-front = { $place } nie leży na froncie
ce-insertion-under-way = Desant - { $place } - już trwa
ce-nothing-to-land = Brak miejsca do lądowania
ce-no-base-in-range = Brak sojuszniczej bazy w zasięgu śmigłowców od celu: { $place }
ce-no-helicopters = Za mało śmigłowców
ce-no-materiel = Za mało zasobów
ce-mission-gone = Ta misja już nie istnieje
ce-no-place-support = Brak miejsca do wsparcia
ce-support-enemy = Wsparcie kieruje się przeciw miejscom wroga
ce-sead-under-way = SEAD SI - { $place } - jest już w drodze
ce-no-sam = { $place }: nie zostało żadne stanowisko SAM
ce-no-airfield-range = Brak sojuszniczego lotniska w zasięgu
ce-no-strike-aircraft = Brak samolotów uderzeniowych na ziemi
ce-off-board = Tej misji nie ma już na tablicy
ce-already-flying = Lot bojowy już trwa
ce-target-destroyed = Ten cel jest już zniszczony
ce-not-friendly-base = To nie jest sojusznicza baza
ce-too-many-points = Za dużo punktów trasy
ce-not-planning = Ta misja nie jest planowana
ce-route-locked = Trasa jest zablokowana podczas wykonywania misji
ce-resupply-requested = Zaopatrzenie już zamówione
ce-resupply-on-way = Śmigłowiec z zaopatrzeniem jest w drodze
ce-no-supply-airfield = Brak sojuszniczego lotniska, z którego można wystartować
ce-finish-sortie = Najpierw zakończ lot bojowy
ce-stockpile-full = Zapas jest pełny

## Odmowy konsoli obrony powietrznej (crates/stealth-sim station.rs)

st-no-campaign = Brak kampanii
st-not-at-a-console = Brak konsoli
st-no-such-track = Nie ma takiego śladu
st-resupply-goes-through-the-campaign = Zaopatrzenie odbywa się przez kampanię
st-no-such-unit = Nie ma takiej jednostki
st-only-friendly-patriot-batteries-and-gepards-can-be-manned = Obsadzić można tylko sojusznicze baterie Patriot i Gepardy
st-another-player-mans-it = Obsługuje ją inny gracz
st-no-track-locked = Brak namierzonego śladu
st-track-lost = Utracono ślad
st-friendly-track-weapons-hold = Ślad sojuszniczy: wstrzymać ogień
st-no-missiles-here = Brak tu rakiet
st-out-of-range = Poza zasięgiem
st-launcher-not-ready = Wyrzutnia niegotowa
st-not-this-battery-s-launcher = To nie jest wyrzutnia tej baterii
st-no-gun-here = Brak tu działka
st-firing = Ogień
st-out-of-ammunition = Brak amunicji
st-nothing-to-refill-there = Nie ma tu czego uzupełniać
st-destroyed = Zniszczona
st-already-refilling = Uzupełnianie już trwa
st-not-empty-yet = Jeszcze nie pusta
st-stockpile-empty = Zapas pusty
st-no-launcher-ready = Brak gotowej wyrzutni
st-unit-destroyed = Zniszczono: { $unit }

## Połączenia (crates/stealth-net)

net-version = Inna wersja gry (protokół { $client }, serwer { $server })
net-no-name = Brak nazwy
net-name-taken = { $name } już gra
net-full = W kampanii nie ma miejsca dla kolejnych pilotów
net-timed-out = Przekroczono limit czasu
net-no-answer = Brak odpowiedzi
net-server-full = Serwer jest pełny
net-reconnected = Połączono ponownie
net-relay-idle = Przekaźnik: bezczynny
net-campaign-over = Kampania się zakończyła. Rozpoczęła się następna: dołącz ponownie
net-shutting-down = Serwer jest wyłączany
net-host-left = Host opuścił grę

## Odprawy kampanii (crates/stealth-host glue.rs)

brief-battery-task = Zniszcz działka baterii wroga wysuniętej w pobliże naszej bazy
brief-shape-guns = DZIAŁKA I MANPADS
brief-shape-battery = BATERIA DZIAŁEK
brief-shape-airfield = LOTNISKO
brief-shape-carrier = LOTNISKOWIEC
brief-shape-city = MIASTO
brief-shape-sam = STANOWISKO SAM
brief-shape-ship = OKRĘT
brief-recon = Sfotografuj cel { $place } wraz z jego obroną.
brief-sead = { $count ->
    [one] Tego miejsca broni { $count } stanowisko SAM lub radar.
    [few] Tego miejsca bronią { $count } stanowiska SAM i radary.
    [many] Tego miejsca broni { $count } stanowisk SAM i radarów.
   *[other] Tego miejsca broni { $count } stanowiska SAM i radarów.
}
brief-cas = { $count ->
    [one] Tego miejsca broni { $count } działko lub zespół MANPADS.
    [few] Tego miejsca bronią { $count } działka i zespoły MANPADS.
    [many] Tego miejsca broni { $count } działek i zespołów MANPADS.
   *[other] Tego miejsca broni { $count } działka i zespołów MANPADS.
}
brief-battery = { $count ->
    [one] Wysunięta bateria wroga: { $count } działko i zespół MANPADS. Zniszcz je.
    [few] Wysunięta bateria wroga: { $count } działka i zespół MANPADS. Zniszcz je.
    [many] Wysunięta bateria wroga: { $count } działek i zespół MANPADS. Zniszcz je.
   *[other] Wysunięta bateria wroga: { $count } działka i zespół MANPADS. Zniszcz je.
}
brief-strike = Lotnisko { $place }.
brief-interdict = Miasto { $place }: jego magazyny i fabryki zaopatrują wroga.
brief-anti-ship = Okręt wroga u wybrzeża.
brief-resupply = { $unit }: potrzebny nowy zapas. { $task }
brief-status = { $status }. { $task }
brief-task-recon = Sfotografuj cel: jego obrona będzie wtedy znana przez godzinę (wykrywana z dwukrotnie większej odległości, krótszy zasięg radarów)
brief-task-sead = Zniszcz stanowiska SAM i radary broniące tego miejsca
brief-task-cap = Utrzymaj panowanie w powietrzu nad punktem przez 10 minut lub zestrzel tam dwa samoloty wroga; utrata sojuszniczego samolotu w tym rejonie zeruje zegar
brief-task-cas = Zniszcz działka i zespoły MANPADS wokół tego miejsca
brief-task-anti-ship = Zatop okręt wroga
brief-task-strike = Zniszcz lotnisko: żaden samolot wroga nie wystartuje z niego, dopóki nie zostanie naprawione
brief-task-escort = Osłaniaj śmigłowce desantowe aż do ich lądowania
brief-task-insertion = Śmigłowce przewożą żołnierzy z najbliższej sojuszniczej bazy; jedno lądowanie wystarcza do zajęcia miejsca
brief-task-intercept = Zestrzel śmigłowce desantowe wroga, zanim wylądują: jedno lądowanie wystarcza do zajęcia miejsca
brief-task-defend = Zestrzel samoloty uderzeniowe wroga, zanim zbombardują lotnisko
brief-task-interdict = Uderz na tyły wroga: miasto od razu kosztuje go zasoby i przychód aż do odbudowy, lotnisko - stacjonujące tam samoloty
brief-task-csar = Utrzymaj panowanie w powietrzu w promieniu 5 km od zestrzelonego pilota przez 2 minuty, gdy śmigłowiec go podejmuje
brief-task-awacs = Zestrzel AWACS wroga: dopóki jest w powietrzu, wróg wysyła więcej patroli i kieruje je na ciebie
brief-task-resupply = Przetransportuj zapas z lotniska do jednostki obrony powietrznej: zawiśnij poniżej 30 m w odległości do 50 m od niej na 10 s, aż ładunek zostanie opuszczony
brief-task-heli-insertion = Przewieź żołnierzy razem ze skrzydłowymi: wyląduj w odległości do 1,5 km od celu i pozostań na ziemi przez 15 s, aż wysiądą, by zająć miejsce
brief-task-heli-csar = Wyląduj w odległości do 200 m od zestrzelonego pilota, odczekaj 30 s, aż wejdzie na pokład, a potem wyląduj z nim w sojuszniczej bazie
brief-task-drones = Zestrzel drony szturmowe, zanim zanurkują w cel: jeden, który się przedrze, niszczy go
