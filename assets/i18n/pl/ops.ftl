# Ekran operacji kampanii (src/campaign/ops.rs).

ops-thunderstorms = BURZE
ops-wx-short = POGODA { $now }, { $next } O { $at }
ops-wx-line = { $weather } Zmiana na: { $next } o { $at }.
ops-clock = DZIEŃ { $day }  { $time }
ops-landed-at = WYLĄDOWANO: { $base }
ops-ground-crew = Obsługa naziemna jest gotowa
ops-rearm = UZBROIĆ I ZATANKOWAĆ
ops-end-sortie = ZAKOŃCZYĆ LOT BOJOWY
ops-on-server = NA { $server }
ops-serving = SERWER
ops-tab-missions = MISJE
ops-tab-map = MAPA WOJNY
ops-tab-air-defence = OBRONA PRZECIWLOTNICZA
ops-tab-forces = SIŁY
ops-tab-players = GRACZE
ops-tab-debrief = OMÓWIENIE
ops-tab-log = DZIENNIK
ops-tab-chat = CZAT
ops-blue = NIEBIESCY { $count }
ops-red = CZERWONI { $count }
ops-war-over = Wojna się zakończyła
ops-running = Trwa symulacja wojny
ops-save-exit = ZAPISAĆ I WYJŚĆ
ops-disconnect = ROZŁĄCZYĆ
ops-leave-local = ZAPISAĆ I OPUŚCIĆ KAMPANIĘ?
    .text = Wojna zatrzyma się do czasu jej wznowienia. Samoloty w powietrzu zostaną utracone; trwające desanty wyruszą ponownie ze swoich baz.
ops-leave-host = ZAPISAĆ I PRZESTAĆ UDOSTĘPNIAĆ KAMPANIĘ?
    .text = Gracze, którzy dołączyli, zostaną rozłączeni, a wojna zatrzyma się do czasu jej wznowienia. Samoloty w powietrzu zostaną utracone; trwające desanty wyruszą ponownie ze swoich baz.
ops-leave-client = ROZŁĄCZYĆ Z SERWEREM?
    .text = Kampania trwa dalej na serwerze; nic nie zostanie tu zapisane. Bieżący lot bojowy się zakończy.
ops-unit-destroyed = Zniszczono: { $unit } - powrót do operacji

## Misje

ops-section-missions = Misje
ops-threat = ZAGROŻENIE { $threat }
ops-no-missions = Brak misji: na froncie spokój
ops-section-underway = Desanty i naloty
ops-enemy-liftoff = WRÓG - { $place }: start za { $mins } min
ops-liftoff = { $place }: start za { $mins } min
ops-enemy-helis = WRÓG - { $place }: nadlatujące śmigłowce: { $count }
ops-helis = { $place }: śmigłowce w drodze: { $count }
ops-raid-sead = SEAD SI - { $place }
ops-raid-drones = DRONY - { $place }
ops-raid = NALOT na lotnisko { $place }
ops-raid-launch = { $raid }: start za { $mins } min
ops-raid-on-way = { $raid }: w drodze: { $count }
ops-resupply-ai = ZAOPATRZENIE dla: { $unit } - UH-60 w drodze
ops-awacs = AWACS WROGA w powietrzu jeszcze przez { $mins } min
ops-down = NA ZIEMI: { $pilot } - ratunek w ciągu { $mins } min
ops-aboard = NA POKŁADZIE: { $pilot } - w drodze do bazy
ops-none-underway = Nic w toku
ops-ai-helicopter = Śmigłowiec SI przejmie zadanie za { $time }, jeśli nie zrobi tego pilot UH-60
ops-defences = Obrona: stanowiska SAM: { $sams }, zagrożenie { $threat }
ops-recon-intel = DANE Z ROZPOZNANIA
ops-order-insertion = ZLECIĆ DESANT
    .facts = Leć w eskorcie, aby go osłaniać, lub poprowadź go w UH-60
ops-from = Z
ops-planned-by-group = Zaplanowane przez grupę
ops-stores = { $aircraft } - podwieszenia: { $stores }
ops-fly-uh60 = LECIEĆ UH-60
ops-accept = PRZYJĄĆ MISJĘ
ops-planning = PLANUJĄ: { $pilots }
ops-flying = W LOCIE: { $pilots }
ops-package = PAKIET z: { $kinds } - wykonaj tu dwa rodzaje zadań w ciągu { $mins } min, +{ $bonus } pkt za każdy
ops-sead-on-way = SEAD SI jest w drodze
ops-request-sead = ZAMÓWIĆ SEAD SI (zasoby: { $cost }, samoloty: { $aircraft })
    .facts = Sojusznicze samoloty uderzeniowe atakują przed tobą stanowisko SAM w tym miejscu
    .disabled = Za mało zasobów lub samolotów na ziemi (patrz SIŁY)

## Zasady, zasoby, siły

ops-rule-damage = realistyczne uszkodzenia
ops-rule-guidance = realistyczne naprowadzanie
ops-rule-no-crashes = bez rozbić
ops-rule-time = ciągły czas
ops-rule-weapons = realistyczne uzbrojenie F-117A
ops-rule-friendly-fire = ogień do swoich
ops-rule-classic = klasyczne
ops-materiel = ZASOBY
ops-materiel-blue = NIEBIESCY { $materiel } (+{ $income }/h)
ops-materiel-red = CZERWONI { $materiel } (+{ $income }/h)
ops-friendly = SOJUSZNICY
ops-enemy = WRÓG
ops-section-forces = Siły
ops-places-held = Zajmowane miejsca
ops-materiel-row = Zasoby
ops-income = Przychód na godzinę
ops-income-from = w tym
ops-income-sources = { $base } + miejsca { $places } x { $per_place } + miasta { $cities } x { $per_city }
ops-fighters-ground = Myśliwce na ziemi
ops-helicopters-ground = Śmigłowce na ziemi
ops-in-air = W powietrzu
ops-in-air-value = myśliwce: { $fighters }, uderzeniowe: { $strike }, śmigłowce: { $helicopters }
ops-since-start = Od początku
ops-earned = zarobione
ops-repairs = naprawy
ops-operations = loty bojowe, naloty, desanty
ops-new-aircraft = nowe samoloty
ops-lost = straty (miasta, piloci)
ops-section-how = Jak to działa
ops-how =
    Każda strona co godzinę zdobywa zasoby: stałą część i więcej za każde zajmowane miejsce oraz za każde nienaruszone miasto wśród nich. Bombardowanie miast wroga (IZOLACJA) zmniejsza jego przychód.
    SI płaci za wszystko, co robi: naprawy, każdy lot myśliwców, naloty i własne desanty. Gdy brakuje jej zasobów, jej uszkodzenia nie są naprawiane, a jej samoloty zostają w bazie.
    SI lata tylko samolotami, które ma; zestrzelone przepadają, te, które wylądowały, latają ponownie. Co godzinę każda strona kupuje nowe, dopóki starcza jej zasobów.
    Gracz nigdy nie odczuwa braków: jego samoloty, bazy (zbombardowane czy nie) i śmigłowce jego własnych rozkazów desantu są zawsze dostępne.
ops-section-prices = Ceny
ops-price-airfield = Naprawa lotniska
ops-price-sam = Naprawa stanowiska SAM
ops-price-city = Odbudowa miasta
ops-price-ship = Naprawa okrętu
ops-price-gun = Działko lub zespół MANPADS
ops-price-sortie = Lot myśliwców
ops-price-raid = Nalot
ops-price-insertion = Desant SI
ops-price-fighter = Nowy myśliwiec
ops-price-helicopter = Nowy śmigłowiec
ops-price-city-lost = Miasto zniszczone (strata)
ops-price-captured = Pilot pojmany (strata)

## Mapa wojny

ops-click-to-watch = { $unit } - kliknij, aby obserwować
ops-section-legend = Legenda
ops-legend-friendly-airfield = Lotnisko sojusznicze
ops-legend-enemy-airfield = Lotnisko wroga
ops-legend-city = Miasto sojusznicze / wroga
ops-legend-airfield = Lotnisko własne / wroga
ops-legend-insertion = Trasa desantu
ops-watch-hint = Kliknij jednostkę, aby obserwować ją na żywo
ops-map-hint = Kółko: zoom, przeciąganie: przewijanie
ops-patriot = Bateria Patriot
ops-drone-center = Centrum dowodzenia dronami
ops-manned-by = Obsługuje: { $pilot }
ops-manned-by-other = Obsługuje inny gracz
ops-control = PRZEJĄĆ KONTROLĘ
    .facts = Przejąć jej konsolę

## Omówienie, koniec, dziennik, czat

ops-no-sortie = Nie wykonano jeszcze żadnego lotu bojowego
ops-section-sortie = Lot bojowy
ops-achieved = CEL OSIĄGNIĘTY
ops-not-achieved = CEL NIEOSIĄGNIĘTY
ops-waiting-raid = OCZEKIWANIE NA NALOT
ops-waiting-insertion = OCZEKIWANIE NA DESANT
ops-flight-time = Czas lotu { $mins } min
ops-kills = Zniszczenia: samoloty { $air }, cele naziemne { $ground }
ops-points = { $points } pkt
ops-points-bonus = { $points } pkt (premia za pakiet +{ $bonus })
ops-campaign-points = { $rank } - { $points } pkt kampanii
ops-campaign-record = Kampania: loty bojowe: { $sorties }, osiągnięte cele: { $won }
ops-section-meanwhile = W międzyczasie
ops-victory = ZWYCIĘSTWO
    .text = Wróg nie ma już żadnego lotniska: teatr działań jest nasz.
ops-defeat = PORAŻKA
    .text = Nie zostało żadne sojusznicze lotnisko: teatr działań jest stracony.
ops-end-day = Dzień { $day }. Miejsca sojusznicze: { $friendly }, miejsca wroga: { $enemy }.
ops-next-campaign = NASTĘPNA KAMPANIA ZACZNIE SIĘ ZA { $time }
ops-next-campaign-now = NASTĘPNA KAMPANIA SIĘ ZACZYNA
ops-next-from-menu = Następną kampanię można rozpocząć z menu głównego.
ops-section-log = Dziennik kampanii
ops-section-chat = Czat
ops-no-messages = Brak wiadomości
ops-message = Wiadomość
    .hint = Wpisz wiadomość, Enter wysyła
ops-send = WYSŁAĆ

## Odprawa

ops-briefing = ODPRAWA PRZED MISJĄ
ops-briefing-line = Z { $base } - { $km } km do celu - { $aircraft } - podwieszenia: { $stores }
ops-route-locked = Trasa zablokowana: leci nią { $pilots }
ops-route-threats = TRASA I ZAGROŻENIA
ops-take-off = STARTOWAĆ
ops-radars-shown = Pokazane radary: wykryte przez sojusznicze samoloty, wciąż działające

## Gracze

ops-section-players = Gracze
ops-col-name = NAZWA
ops-col-status = STATUS
ops-col-sorties = LOTY BOJOWE
ops-col-points = PUNKTY
ops-col-pilot = PILOT
ops-col-objectives = CELE
ops-col-kills = ZNISZCZ. POW./ZIEM.
ops-col-medal = NAJLEPSZE ODZNACZENIE
ops-you = (ty)
ops-host = HOST
ops-status-planning = Planuje: { $title }
ops-status-board = Przy tablicy misji
ops-status-flying = W locie: { $title } - { $aircraft }
ops-status-reconnecting = Utracono połączenie - samolot wstrzymany na { $seconds } s
ops-flown = { $flown } (wykonane: { $won })
ops-section-leaderboard = Ranking
ops-alone = Lot w pojedynkę. Aby latać razem, udostępnij tę kampanię z menu GRA WIELOOSOBOWA w menu głównym.
ops-rules = Zasady tej kampanii (ustalone na jej początku): { $rules }
ops-no-sorties = Nie wykonano jeszcze żadnego lotu bojowego
