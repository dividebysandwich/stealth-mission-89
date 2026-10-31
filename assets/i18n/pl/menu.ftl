# Menu główne i ekran pojedynczej misji (src/menu.rs).

## Teatry działań

theater-libya = Libia (1986)
    .facts =
        Zatoka Wielka Syrta i wybrzeże Libii
        Czasy operacji El Dorado Canyon
        Przez większość dni bezchmurne pustynne niebo
theater-persian_gulf = Zatoka Perska (1984)
    .facts =
        Czasy wojny tankowcowej Iran-Irak
        Nad morzem i nadmorską pustynią
        Najlepsza pogoda ze wszystkich teatrów działań
theater-north_cape = Przylądek Północny
    .facts =
        Arktyczna Norwegia i Półwysep Kolski
        Baza radzieckiej Floty Północnej
        Częste niskie zachmurzenie
theater-central_europe = Europa Środkowa
    .facts =
        NATO przeciw Układowi Warszawskiemu nad Niemcami
        Bezchmurne niebo tylko przez 15% czasu
theater-vietnam = Wietnam
    .facts =
        Rejon Hanoi i Hajfongu
        Dżungla i teren górzysty
        Burze w 30% misji
theater-middle_east = Bliski Wschód
    .facts =
        Wschodnia część Morza Śródziemnego i Lewant
        Przeważnie bezchmurne niebo
theater-desert_storm = Pustynna Burza
    .facts =
        Kuwejt i Irak, 1991
        Prawdziwy debiut bojowy F-117A
        Przeważnie bezchmurne pustynne niebo
theater-cuba = Kuba
    .facts =
        Wyspiarski teatr działań na Karaibach
        Cele na morzu i na lądzie
        Klimat tropikalny: burze w 15% misji
theater-korea = Korea
    .facts =
        Półwysep Koreański na północ od DMZ
        Teren górzysty
        Zmienna pogoda, każde niebo jest możliwe
theater-ukraine = Ukraina (2026)
    .facts =
        Wschodnia Ukraina, Krym i południowa Rosja
        Linia frontu płonie od Charkowa po Chersoń
        Otwarte pola uprawne, zrujnowane miasta na froncie

## Sytuacje (zasady użycia siły)

roe-0 = Zimna wojna
    .facts =
        Tajna operacja w czasie pokoju
        Najważniejsze jest pozostanie niewykrytym
        Mniej wrogich samolotów na patrolu
        Dokładne zasady użycia siły są w odprawie
roe-1 = Wojna ograniczona
    .facts =
        Regionalny konflikt zbrojny
        Wroga obrona przeciwlotnicza w gotowości
        Więcej wrogich samolotów niż w zimnej wojnie
roe-2 = Wojna konwencjonalna
    .facts =
        Wojna na pełną skalę
        Wszystkie siły wroga aktywne
        Więcej wrogich samolotów niż w zimnej wojnie

## Rodzaje misji

kind-strike = Uderzenie
    .facts =
        Zniszczyć, sfotografować lub osiągnąć cel naziemny
        Cel główny i cel dodatkowy
        Uzbrojenie powietrze-ziemia
kind-air = Powietrze-powietrze
    .facts =
        Przechwycić wrogi samolot
        Transportowce, bombowce lub patrole myśliwców
        Uzbrojenie: pociski powietrze-powietrze
kind-strike-training = Trening uderzenia
    .facts =
        Ćwiczebna misja uderzeniowa
        Nauka uzbrojenia i nawigacji
kind-air-training = Trening powietrze-powietrze
    .facts =
        Ćwiczebna misja powietrze-powietrze
        Nauka przechwytywania i pocisków
kind-drone-defense = Obrona przed dronami
    .facts =
        Tylko Ukraina
        Zatrzymać falę dronów uderzeniowych Geran, zanim trafi w miasto lub elektrownię jądrową
        Następnie zbombardować ich miejsce startu

## Przeciwnicy

opponents-0 = Nowicjusz
    .facts =
        Niedoświadczone wrogie załogi
        Najłatwiejszy poziom
opponents-1 = Standard
    .facts =
        Przeciętne wrogie załogi
        Poziom domyślny
opponents-2 = Weteran
    .facts =
        Wprawne wrogie załogi
        Więcej okrętów i patroli
opponents-3 = Elita
    .facts =
        Najlepsi wrodzy piloci i obsługi SAM
        Najtrudniejszy poziom

## Pogoda

sky-random = Losowa
    .facts =
        Losowana z klimatu teatru działań
        Zmienia się z każdą nową misją
        Chmury zasłaniają widok obu stronom
sky-clear = Bezchmurnie
    .facts =
        Brak chmur
        Widzialność 40 km
        Klasyczne niebo z czasów DOS-a
sky-scattered = Rozproszone
    .facts =
        Cumulus 20-45%, podstawa 2500-6000 ft
        Czasem cienki altostratus wysoko w górze
        Widzialność 30 km
sky-broken = Poprzerywane
    .facts =
        Cumulus 45-70%, podstawa 2000-5000 ft
        Często warstwa stratocumulusów powyżej
        Widzialność 25 km
sky-overcast = Całkowite
    .facts =
        Stratus 80-100%, podstawa 1500-5000 ft
        Kilka strzępów cumulusów poniżej
        Widzialność 20 km
sky-storm = Burza
    .facts =
        Wieże cumulonimbusów do 40000 ft
        Niskie stratusy i cumulusy od 1000 ft
        Widzialność 8 km
menu-random-cover = Losowe pokrycie
    .facts =
        Pokrycie głównej warstwy chmur losowane razem z pogodą
        Odznaczyć, aby ustawić je samodzielnie
        Główna warstwa: stratus przy zachmurzeniu całkowitym i burzy, w innych przypadkach cumulus
menu-cover-slider =
    .facts =
        Część nieba pokryta przez główną warstwę
        Bez wpływu przy bezchmurnym niebie

## Księżyc

moon-date = Data misji
    .facts =
        Prawdziwy księżyc w dniu z odprawy
        Wymaga przełącznika pory dnia (nowoczesny)
moon-0 = Nów
    .facts =
        Brak światła księżyca: czarna noc, tylko światła i gwiazdy
        Wrodzy piloci widzą samolot z 2 km
moon-1 = Sierp przybywający
    .facts = Cienki księżyc na wieczornym niebie, zachodzi przed północą
moon-2 = Pierwsza kwadra
    .facts = W połowie oświetlony, widoczny wieczorem, zachodzi około północy
moon-3 = Garb przybywający
    .facts = Jasny, widoczny od popołudnia do po północy
moon-4 = Pełnia
    .facts =
        Widoczny całą noc, wysoko o północy
        Wrodzy piloci widzą samolot z 4 km
moon-5 = Garb ubywający
    .facts = Jasny, wschodzi wieczorem, widoczny do rana
moon-6 = Ostatnia kwadra
    .facts = W połowie oświetlony, wschodzi około północy
moon-7 = Sierp ubywający
    .facts = Cienki księżyc wschodzący przed świtem

## Maszyny

aircraft-f117 =
    .facts =
        Samolot szturmowy stealth
        Poddźwiękowy, 7 g
        Kokpit nowoczesny i klasyczny
aircraft-f16 =
    .facts =
        Wielozadaniowy myśliwiec
        Klasa Mach 2, 9 g, szybki przechył
        Kokpit F-16 z RWR
aircraft-tornado =
    .facts =
        Samolot uderzeniowy do lotów na małej wysokości
        Skrzydła o zmiennej geometrii zmieniają skos wraz z prędkością
        Nowoczesny kokpit, trzy MFD
aircraft-a10 =
    .facts =
        Samolot bezpośredniego wsparcia powietrznego
        Wolny i wytrzymały, działko 30 mm
        Kokpit A-10C, dwa MFD
aircraft-av8b =
    .facts =
        Samolot szturmowy V/STOL
        Wektorowanie ciągu: zawis, lądowanie wszędzie
        Dźwignia dysz: R / F, Shift do oporów
aircraft-uh60 =
    .facts =
        Śmigłowiec wielozadaniowy
        Desantuje żołnierzy: wylądować blisko celu
        Drążek sterowania okresowego i dźwignia skoku ogólnego
aircraft-ah1z =
    .facts =
        Śmigłowiec szturmowy
        Działko w wieżyczce, Hellfire, AIM-9
        Działko i Hellfire podążają za zasobnikiem

## Sekcje i grupy

section-theater = Teatr działań
section-opponents = Przeciwnicy
section-situation = Sytuacja
section-mission = Misja
section-moon = Księżyc
section-start-time = Godzina startu
section-weather = Pogoda
section-realism = Realizm
section-tacview = Tacview
section-mission-summary = Podsumowanie misji
section-briefing = Odprawa
group-mission = Misja
group-conditions = Warunki
group-vehicle = Maszyna
group-simulation = Symulacja

## Opcje misji

menu-start-random = Losowa
    .facts =
        Losowana z każdą nową misją, jak w trybie klasycznym
        06:00-08:00 w dzień lub 18:00-20:00 w nocy
        Odznaczyć, aby wybrać godzinę
menu-start-slider =
    .facts =
        Godzina startu
        Noc od 18:00 do 06:00
        Zachowywana przez NOWA MISJA
menu-realistic-weapons = Realistyczne uzbrojenie
    .facts =
        Ograniczenia prawdziwego samolotu
        2 komory uzbrojenia zamiast 4
        Bez pocisków AIM-9 i AIM-120
        Bez startów i lądowań na lotniskowcu
menu-crash-protection = Ochrona przed rozbiciem
    .facts =
        Automatycznie wyprowadza w pobliżu ziemi (podwozie schowane)
        Kontakt z ziemią i kolizje nie są śmiertelne
        Wył.: w ziemię zawsze można uderzyć
menu-tacview-export = Eksport
    .facts =
        Zapisuje każdy lot jako nagranie Tacview (.acmi)
        W folderze gry na pliki użytkownika, w podfolderze tacview
menu-tacview-real-time = Telemetria w czasie rzeczywistym
    .facts =
        Wysyła lot do Tacview w trakcie lotu
        W Tacview: Record > Real-Time Telemetry
        Adres 127.0.0.1, port 42674 (--tacview-address)
switch-game = GRA
switch-real = REALIZM
menu-damage = Uszkodzenia
    .facts =
        Uszkodzenia od uzbrojenia
        Gra: klasyczne uszkodzenia; trafienia się sumują, ale nigdy nie strącają samolotu
        Realizm: trafienie pociskiem może zniszczyć samolot (katapultować się!), uszkadza więcej systemów i często wywołuje pożar, który gaśnie albo rozrywa samolot
        Realizm: uszkodzone powierzchnie sterowe rozstrajają trym lub wywołują oscylacje samolotu
menu-guidance = Naprowadzanie
    .facts =
        Naprowadzanie pocisków
        Klasyczne: pociski lecą prosto na cel
        Realizm: nawigacja proporcjonalna do punktu kolizji; silniki się wypalają i pociski dalej szybują, tracąc prędkość z każdym zakrętem (dym tylko podczas pracy silnika)
        Realizm: SAM-y wznoszą się przeciw odległym celom; AIM-120 leci prosto przez 3 s (poderwać nos, aby wystrzelić go po łuku)
menu-air-defence = Obrona przeciwlotnicza
    .facts =
        Obrona przeciwlotnicza na małych wysokościach
        Klasyczna: tylko stanowiska SAM i myśliwce
        Realizm: działka ZSU-23-4 naprowadzane radarem (ich radar widzi samolot jak radar SAM) i baterie artylerii S-60 strzegą lotnisk i celów; zespoły MANPADS w miastach strzelają pociskami SA-14 do tego, co widzą
        Niski lot nad bronionymi miejscami staje się niebezpieczny; bomby i działko niszczą stanowiska artylerii

## Ekran główny

menu-title = STEALTH MISSION 89
menu-subtitle = SYSTEM PLANOWANIA MISJI
menu-single-mission = POJEDYNCZA MISJA
    .facts =
        Jedna misja w wybranym teatrze działań
        Wybrać opcje, zaplanować trasę i uzbrojenie
menu-career = KARIERA
    .facts =
        Akta pilotów: stopień, odznaczenia, loty bojowe
        Tura służby: 12 lotów bojowych w jednym teatrze działań
        Co zostanie zniszczone, pozostaje zniszczone
menu-campaign = KAMPANIA DYNAMICZNA
    .facts =
        Kampania dynamiczna w czasie rzeczywistym
        Zdobywanie miast i lotnisk wroga
        Misje wynikające z bieżącej sytuacji
menu-multiplayer = GRA WIELOOSOBOWA
    .facts = Kooperacja w kampanii dynamicznej
    .join = Dołączyć do serwera z listy publicznej
    .host = Dołączyć do serwera albo założyć kampanię i w niej latać
menu-options = OPCJE
    .facts =
        Gra: styl kokpitu, HUD i awioniki, automatyczny trym
        Grafika, głośność dźwięków, klawisze i joystick
        Zapisywane na kolejne loty
    .short = Gra, grafika, dźwięk i sterowanie
menu-download = POBRAĆ
    .facts =
        Wersja gry na komputer: Windows, Linux i macOS
        Z VR i grą wieloosobową
        Steam Deck i Steam Frame: AppImage dla Linuksa
web-play = GRAĆ W PRZEGLĄDARCE
    .facts = Tutaj, bez instalowania
web-download-platforms = Windows, Linux i macOS, Steam Deck i Steam Frame
web-download-version = Wersja do pobrania
web-gives-vr = VR
    .why = Latanie w VR na każdym zestawie zgodnym z OpenXR oraz na Steam Frame
web-gives-hotas = HOTAS
    .why = Pełna obsługa padów, joysticków, przepustnic i orczyków
web-gives-multiplayer = Lepsza gra wieloosobowa
    .why = Bezpośrednie połączenie UDP z serwerami, mniejsze opóźnienia
web-gives-server = Własny serwer
    .why = Prowadzenie kampanii kooperacyjnej lub uruchomienie serwera dedykowanego
web-gives-head-tracking = Śledzenie głowy
    .why = Trackery typu TrackIR przez OpenTrack
web-gives-tacview = Tacview
    .why = Nagrania lotów i telemetria w czasie rzeczywistym
web-gives-keep = Na zawsze Twoje
    .why = Gdy strona przestanie działać, kopia zostaje u Ciebie
menu-discord = DISCORD
    .facts =
        Wspólne latanie, zgłaszanie błędów i dzielenie się misjami
        Śledzenie postępów prac
menu-quit = WYJŚĆ
    .facts = Wyjście do pulpitu
menu-version = WERSJA { $version }

## Ekran pojedynczej misji

menu-career-sortie = LOT BOJOWY W KARIERZE
menu-tour-sortie = { $pilot }  //  LOT BOJOWY { $sortie } Z { $length } W TURZE
menu-msn = SYSTEM PLANOWANIA MISJI  //  MSN { $seed }
menu-no-targets = W tej turze nie pozostał żaden cel
menu-plan-route = ZAPLANOWAĆ TRASĘ
    .facts =
        Mapa teatru działań z pokryciem wrogich radarów i SAM
        Dodawać, przesuwać i usuwać punkty nawigacyjne
        Cele i baza lądowania pozostają stałe
menu-loadout = UZBROJENIE
    .facts =
        Wybrać podwieszenia w każdej komorze lub na każdym pylonie
        Na początek uzbrojenie z odprawy
menu-generating = GENEROWANIE...
menu-back-records = Powrót do akt pilotów
menu-back-main = Powrót do menu głównego
menu-new-sortie = NOWY LOT BOJOWY
menu-new-mission = NOWA MISJA
    .facts =
        Losuje inne cele
        Zachowuje wszystkie opcje i godzinę startu
menu-load = WCZYTAĆ
    .facts =
        Misje zapisane w locie (menu: ZAPISAĆ MISJĘ)
        Kontynuacja lotu od miejsca zapisu
menu-fly = LECIEĆ
    .facts = Start do misji z odprawy
menu-fly-vr = LECIEĆ W VR
    .facts =
        Uruchamia grę ponownie w goglach VR
        Dokładnie ta sama misja
    .no-mission = Najpierw wygenerować misję
    .no-runtime = Brak aktywnego środowiska OpenXR (Monado, WiVRn, SteamVR)
menu-vr-failed = nie udało się uruchomić w VR: { $error }

## Podsumowanie misji

summary-theater = TEATR DZIAŁAŃ
summary-situation = SYTUACJA
summary-mission = MISJA
summary-opponents = PRZECIWNICY
summary-time = GODZINA
summary-weather = POGODA
summary-fuel = PALIWO
summary-night = Noc
summary-day = Dzień
summary-time-value = { $daytime }  start { $depart }
summary-weather-value = { $sky }  widz. { $vis } km
summary-fuel-value = { $fuel } lb
summary-armament = Uzbrojenie: { $stores }
summary-intelligence = ROZPOZNANIE
aircraft-rah66 =
    .facts =
        Stealth scout helicopter
        Bays: Hellfires or Stingers
        Hard to see on radar, slow or hovering
