# Okno kampanii (src/campaign/menu.rs), okno gry wieloosobowej (src/multiplayer_menu.rs),
# legenda mapy wojny i podgląd na żywo (src/campaign/picture.rs, unit_view.rs).

delete = USUNĄĆ
delete-campaign = USUNĄĆ TĘ KAMPANIĘ?
delete-save = USUNĄĆ TEN ZAPIS?
delete-permanent = Usunięcie jest nieodwracalne

## Okno kampanii

cm-group-campaign = Kampania
cm-weather-dynamic = Dynamiczna
    .facts =
        Pogoda zmienia się co 10-22 godziny według zegara nieba
        Chmury zasłaniają widoczność obu stronom
cm-weather-clear = Czyste niebo
    .facts = Brak chmur przez całą kampanię
cm-section-day-night = Dzień i noc
cm-day-real = Czas rzeczywisty
    .facts = Doba trwa 24 godziny
cm-day-2h = 2 h / doba
    .facts = Doba trwa 2 godziny
cm-day-1h = 1 h / doba
    .facts = Doba trwa 1 godzinę
cm-day-30min = 30 min / doba
    .facts = Doba trwa 30 minut
cm-conditions-note = Warunki dotyczą nowej kampanii
cm-rules-note = Zapisana kampania zachowuje zasady, z którymi ją rozpoczęto
cm-about =
    Wojna toczy się w czasie rzeczywistym podczas gry, także przy otwartym menu.
    Misje z tablicy: rozpoznanie, SEAD, CAP, CAS, zwalczanie okrętów, uderzenie, eskorta.
    Desanty śmigłowcowe zajmują miasta i lotniska wroga.
    Pilot nigdy nie ginie ani nie trafia do niewoli: po locie bojowym wraca do tablicy.
cm-section-new = Nowa kampania
cm-new-line = { $theater }  -  { $situation }  -  przeciwnicy: { $opponents }
cm-start = ROZPOCZĄĆ
cm-section-continue = Kontynuacja
cm-no-saves = Brak zapisanych kampanii
cm-played = Czas gry: { $hours } h
cm-save-gone = Zapis już nie istnieje
cm-save-unreadable = Nie można odczytać zapisu: { $error }
cm-save-undeletable = Nie można usunąć zapisu: { $error }
cm-in-vr = W VR
cm-delete =
    .facts = Usunąć tę zapisaną kampanię

## Mapa wojny

pic-friendly-aircraft = Samolot sojuszniczy
pic-friendly-helicopter = Śmigłowiec sojuszniczy
pic-enemy-fighter = Myśliwiec wroga (na radarze)
pic-sam-known = Stanowisko SAM (znane)
pic-guns-known = Działka / MANPADS (znane)
pic-reconned = Rozpoznany
pic-you = Ty
pic-site = Stanowisko { $place }
uv-live = { $unit } - NA ŻYWO
uv-contact-lost = UTRACONO KONTAKT
uv-hint = Przeciąganie: obrót  Kółko: zoom

## Gra wieloosobowa

mp-disconnected = Rozłączono: { $why }
mp-connecting = Łączenie z { $address }...
mp-joining = Połączono z { $address }, dołączanie...
mp-browser-public-only = z przeglądarki można dołączyć tylko do serwerów z listy publicznej
mp-title-host = GRA WIELOOSOBOWA: ZAKŁADANIE GRY
mp-subtitle = Kooperacja w kampanii dynamicznej: wspólna wojna w powietrzu
mp-your-name = Twoje imię
    .hint = Imię pilota
mp-join-address = Dołączanie przez adres
    .hint = adres lub nazwa[:port]
mp-connect = POŁĄCZYĆ
mp-server-name = Nazwa serwera
mp-udp-port = Port UDP
mp-address-note = Inni gracze dołączają przez adres tego komputera; aby grać przez internet, trzeba otworzyć port UDP w routerze
mp-list-public = Pokazać na publicznej liście serwerów (inni znajdą grę na swojej liście serwerów)
mp-rules = Zasady dla wszystkich
mp-time-of-day = Pora dnia (noce przy księżycu)
mp-realistic-weapons = Realistyczne uzbrojenie F-117A
mp-friendly-fire = Ogień do swoich
mp-day-night = Dzień i noc:
mp-weather = Pogoda:
mp-host-new = ZAŁOŻYĆ NOWĄ
mp-host-saved = Założyć grę z zapisanej kampanii
mp-host = ZAŁOŻYĆ
mp-host-game = ZAŁOŻYĆ GRĘ
mp-port-error = Port UDP { $port }: { $error }
mp-default-server = Kampania pilota { $pilot }
mp-public-servers = Serwery publiczne
mp-refresh = ODŚWIEŻYĆ
mp-server-count = Serwery: { $servers }, piloci: { $pilots }
mp-list-error = Lista serwerów jest niedostępna: { $error }
mp-no-servers = Obecnie brak serwerów publicznych
mp-col-server = SERWER
mp-col-pilots = PILOCI
mp-col-war = WOJNA
mp-dedicated = DEDYKOWANY
mp-other-version = inna wersja
mp-join = DOŁĄCZYĆ
