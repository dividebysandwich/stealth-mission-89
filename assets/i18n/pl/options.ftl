# Okno OPCJE (src/options.rs) i jego strony w menu w locie.

options-title = OPCJE
options-tab-game = GRA
options-tab-graphics = GRAFIKA
options-tab-sound = DŹWIĘK
options-tab-controls = STEROWANIE
options-footer = Zapisywane na kolejne loty; przełączniki realizmu wybiera się w każdej grze

## GRA

options-section-language = Język
options-section-classic-modern = Klasyczny / Nowoczesny
options-cockpit = Kokpit
    .facts =
        Nowoczesny: wyświetlacze wysokiej rozdzielczości z przyciskami na ramce
        Klasyczny: układ tablicy przyrządów z czasów DOS-a i wyświetlacze 77x60 (F-117A)
options-hud = HUD
    .facts =
        Nowoczesny: zielony HUD wysokiej rozdzielczości w stylu F-16, z trybami NAV, AA i AG
        Klasyczny: HUD z czasów DOS-a
options-avionics = Awionika
    .facts =
        Nowoczesna: strony MFD w stylu F-16 na każdym MFD (sytuacja, HARM, zasobnik celowniczy, mapa, podwieszenia) z ruchomym kursorem
        Klasyczna: mapa i TAC po lewej, kamera, WAY, WPN i ORD po prawej
options-flight-model = Model lotu
    .facts =
        Nowoczesny: aerodynamiczny model lotu z fly-by-wire: drążek steruje prędkością pochylania i przechylania; po puszczeniu nos zostaje na miejscu, a tor lotu jest utrzymywany
        Klasyczny: model lotu z czasów DOS-a
options-section-flying = Pilotaż
options-auto-trim = Automatyczny trym
    .facts =
        Po puszczeniu drążka trym utrzymuje położenie, w jakim samolot został zostawiony
        Śmigłowce: nos nie unosi się już z prędkością; odrzutowce: kompensuje uszkodzone powierzchnie sterowe
        Trym ręczny: Alt + strzałki, Alt + Home go centruje
options-keep-time-accel = Zachowaj przyspieszenie czasu
    .facts =
        Przyspieszenie nie wyłącza się już po wykryciu, odpaleniu broni lub zestrzeleniu
        Zmieniają je tylko przycisk TIME oraz Shift + Z / Shift + X
options-tooltips = Tooltipy kokpitu
    .facts = Czym jest element sterowania lub wyświetlacz kokpitu, pod myszą lub pod palcem w VR
options-player-labels = Nazwy graczy
    .facts = Gra wieloosobowa: nazwy innych pilotów nad ich samolotami, z każdej odległości
options-label-size =
    .facts =
        Rozmiar nazw graczy
        50%: domyślnie
options-seat-height = Wysokość fotela
    .facts =
        Jak wysoko siedzisz w kokpicie, na ekranie i w VR
        0 cm: zwykła wysokość oczu w samolocie
options-section-vr-size = Rozmiar kokpitu w VR
options-vr-size =
    .facts =
        Rozmiar kokpitu (i świata) w goglach
        100%: naturalna wielkość
options-section-gif = Nagrywanie GIF
options-gif-size = ROZMIAR
options-gif-rate = KLATKI
options-gif-scale-100 =
    .facts = Pełna rozdzielczość okna: duże pliki
options-gif-scale-25 =
    .facts = Małe pliki
options-gif-rate-50 =
    .facts = Tylko gdy gra działa w 100 fps lub więcej (nagrywana jest co druga klatka)
options-gif-hint = F12 (o ile nie zmieniono) rozpoczyna i kończy; zapis w folderze Obrazy

## GRAFIKA

options-section-rendering = Renderowanie
options-shaders = Shadery
    .facts =
        Nowoczesne: subtelne cieniowanie kierunkowe, łagodna mgiełka z odległością
        Klasyczne: płaskie cieniowanie
options-shadows = Cienie
    .facts =
        Nowoczesne: cienie od słońca w kokpicie i w świecie (kosztuje czas GPU)
        Klasyczne: bez cieni
options-point-lights = Światła punktowe
    .facts =
        Nowoczesne: oświetlenie kokpitu i wyświetlaczy; eksplozje, smugi ognia i światła pasa oświetlają scenę
        Klasyczne: tylko światło słońca i nieba
options-time-of-day = Pora dnia
    .facts =
        Nowoczesna: słońce i księżyc poruszają się zgodnie z zegarem misji, niebo ciemnieje i jaśnieje razem ze słońcem; nocą gwiazdy i faza księżyca
        Nowoczesna: światło księżyca określa, z jakiej odległości wrodzy piloci widzą samolot w nocy (2 km w bezksiężycową noc, 4 km przy pełni)
        Klasyczna: powolna zmiana kolorów i nieruchome słońce
options-msaa = Wygładzanie krawędzi (MSAA)
    .facts =
        Gładkie krawędzie: MSAA 4x, 2x w VR (kosztuje czas GPU)
        Wył.: ostre krawędzie pikseli
options-auto-detail = Automatyczna szczegółowość scenerii (VR)
    .facts =
        W goglach zasięg miast, lasów i pól maleje, gdy klatki się spóźniają, i rośnie z powrotem, gdy jest zapas czasu
        Ustawienia grafiki pozostają takie, jak wybrano
options-sun-moon = Słońce i księżyc
    .facts =
        Słońce na niebie w dzień, księżyc i gwiazdy w nocy
        Księżyc pokazuje fazę z misji
options-scenery-distance = Zasięg scenerii
    .facts = Odległe miejscowości: 7–30 km; pola: 24–30 km. Zasięgi drzew pozostają bez zmian. Automatyczna szczegółowość VR może zmniejszyć zasięgi scenerii.
options-city-light-distance = Zasięg świateł miejskich
    .facts = Latarnie, okna i światła ostrzegawcze budynków, niezależnie od zasięgu scenerii. Pogoda może ograniczyć widoczność.
options-cloud-quality = Chmury
cloud-quality-auto = Auto
    .facts = Niska w VR, wysoka na ekranie
cloud-quality-low = Niska
    .facts = Zgrubne chmury w jednej trzeciej rozdzielczości (najszybciej)
cloud-quality-medium = Średnia
    .facts = Połowa rozdzielczości, drobniejsze szczegóły
cloud-quality-high = Wysoka
    .facts = Pełna rozdzielczość, najdrobniejsze szczegóły i własny cień (kosztuje czas GPU)

## DŹWIĘK

options-section-volume = Głośność
options-volume-master = Ogólna
    .facts = Wszystkie dźwięki
options-volume-engine = Silnik
    .facts = Własny silnik lub wirnik oraz maszyny słyszane wokół
options-volume-effects = Efekty
    .facts = Uzbrojenie, eksplozje, podwozie, radio i cała reszta
options-volume-warnings = Ostrzeżenia
    .facts = Sygnał przeciągnięcia, ostrzeżenia radarowe i przed pociskami, syrena pożarowa, tony Sidewindera
options-volume-rwr = RWR
    .facts = Tony ostrzeżeń radarowych, jako część głośności ostrzeżeń
options-section-rwr-tones = Tony ostrzeżeń radarowych
rwr-tones-all = WSZYSTKIE
    .facts = Radar, który cię oświetla, każde omiecenie, które cię wykrywa, odpalenia pocisków
rwr-tones-detect = TYLKO WYKRYCIE
    .facts =
        Jeden ton, gdy radar cię wykryje (ponownie, gdy radary na chwilę cię zgubią)
        Oraz odpalenia pocisków
rwr-tones-launch = TYLKO ODPALENIA
    .facts = Tylko pociski odpalone w ciebie
rwr-tones-off = BRAK
    .facts = Bez tonów ostrzeżeń radarowych (lampki i wyświetlacze nadal ostrzegają)
options-section-cockpit = Kokpit
options-low-pass = Filtr dolnoprzepustowy silnika
    .facts =
        Wewnątrz maszyny silnik brzmi głucho (osłona kabiny i hełm tłumią wysokie tony)
        Wył.: tak samo jak na zewnątrz

## STEROWANIE

options-section-bindings = Przypisania
options-keys-joystick = KLAWISZE I JOYSTICK
    .facts =
        Przypisać klawisze oraz osie i przyciski joysticka i HOTAS
        Zapisywane na kolejne loty
options-section-head-tracking = Śledzenie głowy i oczu
options-head-tracking = Śledzenie głowy i oczu
    .facts =
        Ruchy głowy obracają i przesuwają widok w kokpicie (na monitorze; w VR robią to gogle)
        Program śledzący (OpenTrack...): wyjście FreeTrack lub UDP na 127.0.0.1 port 4242
        Home: bieżące położenie głowy staje się środkiem
options-tracking = Śledzenie aktywne ({ $source })
options-no-tracker = Brak sygnału śledzenia: uruchomić program śledzący

## EFFECTS

options-section-effects = Efekty
options-section-retro = Retro
effect-shake = Drgania kamery
    .facts =
        Widok drży przy strzale z działka, pobliskich wybuchach, przeciągnięciu, lądowaniach i na pasie
        VR: kokpit wibruje, głowa pozostaje nieruchoma
effect-flashes = Błyski bitewne
    .facts = Wybuchy pocisków przeciwlotniczych, eksplozje i starty rakiet na chwilę oświetlają chmury, ziemię i kokpit
effect-barrage = Nocny ostrzał zaporowy
    .facts =
        Wojna konwencjonalna nocą: działa miast strzelają na oślep pociskami smugowymi w niebo, reflektory omiatają je
        Wrogie drony nad miastem są łapane przez jego reflektory i ściągają ogień jego dział
        Tylko dla efektu: ostrzał niczego nie trafia
effect-cockpit-night = Nocne oświetlenie kokpitu
    .facts =
        Przygaszone czerwone światło w kokpicie po zmroku (niebiesko-zielone pod goglami)
        Master caution pulsuje bursztynowo nad panelem, dopóki trwa ostrzeżenie
effect-nvg = Gogle noktowizyjne
    .facts =
        Jasne światła rozlewają się w halo, obraz bardziej ziarnisty w ciemności
        Pulpit: okrągłe pole widzenia gogli
effect-snow = Śnieg
    .facts = Powyżej poziomu zamarzania burze przynoszą śnieg zamiast deszczu: nisko w północnej zimie, wysoko latem
effect-mist = Mgła przy ziemi
    .facts =
        Nowoczesne shadery: mgła leży nad niżem i morzem o świcie, o zmierzchu i w nocy
        Rzadka na pustyniach, brak podczas burz
effect-cloud-shadows = Cienie chmur
    .facts = Nowoczesne shadery: cienie chmur na ziemi poniżej
effect-vapour = Para i podmuch wirnika
    .facts =
        Przy ostrych manewrach smugi pary z końców skrzydeł
        Śmigłowiec w niskim zawisie wzbija kurz albo pył wodny nad wodą
        Jadące pojazdy wzbijają za sobą kurz
effect-night-sky = Nocne niebo
    .facts =
        Od czasu do czasu spadające gwiazdy, gwiazdy nisko nad horyzontem migoczą
        Zorza polarna nad Przylądkiem Północnym
effect-vga = Kolory VGA
    .facts = Wygląd z 1989 roku z nowoczesnym cieniowaniem: 64 stopnie na kanał koloru, jak w karcie VGA, z ditheringiem
effect-scanlines = Linie skanowania
    .facts = Linie skanowania monitora CRT (nie w VR)

## LANGUAGE NAMES

lang-name-en = angielski
lang-name-bg = bułgarski
lang-name-zh-Hans = chiński (uproszczony)
lang-name-hr = chorwacki
lang-name-cs = czeski
lang-name-da = duński
lang-name-nl = niderlandzki
lang-name-et = estoński
lang-name-fi = fiński
lang-name-fr = francuski
lang-name-de = niemiecki
lang-name-el = grecki
lang-name-hu = węgierski
lang-name-id = indonezyjski
lang-name-ga = irlandzki
lang-name-it = włoski
lang-name-ja = japoński
lang-name-ko = koreański
lang-name-lv = łotewski
lang-name-lt = litewski
lang-name-mt = maltański
lang-name-pl = polski
lang-name-pt-BR = portugalski (Brazylia)
lang-name-ro = rumuński
lang-name-ru = rosyjski
lang-name-sk = słowacki
lang-name-sl = słoweński
lang-name-es = hiszpański
lang-name-sv = szwedzki
lang-name-uk = ukraiński

## Browser build: offer to play in the browser's language (src/i18n.rs)

language-offer-question = Język przeglądarki: { $language }. Grać w tym języku?
language-offer-yes = Tak
language-offer-no = Nie
language-offer-never = Nie pytaj ponownie
effect-sunbeams = Promienie słońca
    .facts = Smugi światła słonecznego przez szczeliny w przerywanym zachmurzeniu
options-msaa-forced = Wyłączone, gdy pikselizacja (retro) jest włączona
options-volume-radio = Radio
    .facts = Pozostali gracze mówiący w radiu kooperacji oraz mówiony ATC
options-volume-atc = Głos ATC
    .facts = Mówione komunikaty ATC, jako część głośności radia
options-volume-music = Muzyka
    .facts = Muzyka menu głównego
options-section-atc = Mówiony ATC
options-section-radio = Radio kooperacji
options-push-to-talk = Naciśnij i mów
    .facts =
        Przytrzymaj klawisz mówienia (Caps Lock lub przycisk joysticka albo VR przypisany do PTT), a wszyscy gracze cię słyszą
        Mikrofon otwiera się przy pierwszym naciśnięciu
options-mic-gain = WZMOCNIENIE MIK.
    .facts =
        Jak głośno wychodzi twój mikrofon
        100%: bez zmian
options-mic-default = Domyślne
options-voice-text-only = tylko tekst
options-voices-line = Kontrolerzy: { $controllers }   Pilot: { $pilot }
options-voices = GŁOSY
    .facts =
        Pobierz głosy do komunikatów radiowych, usuń je
        Wybierz głosy kontrolerów i własny, i posłuchaj ich
options-voices-web = Tylko tekst w przeglądarce
options-tracking-mouse = Rozglądanie myszą przy trackingu
    .facts =
        Włączone: prawy przycisk myszy nadal obraca i przybliża widok kokpitu
        Wyłączone: gdy tracker nadaje, prawy przycisk myszy nic nie robi w kokpicie
options-atc = Kontrola ruchu lotniczego
    .facts =
        Komunikaty radiowe wieży, podejścia i GCA na lotniskach sojuszniczych
        Zgoda na start, overhead break i talkdown przy niskich chmurach lub słabej widoczności
options-pixel-block = Rozmiar bloku pikselizacji
    .facts = Bok bloku w pikselach ekranu (2: drobny, 5: gruby)
effect-pixelize = Pikselizacja
    .facts =
        Cały widok 3D (teren, samoloty, budynki, kokpit) w dużych kwadratowych pikselach, jak na komputerze z MS-DOS; menu i podpowiedzi pozostają ostre (nie w VR)
        Suwak ustawia rozmiar bloku w pikselach ekranu (2: drobny, 5: gruby)
        Antyaliasing jest wyłączony, dopóki jest włączona
        Trzy przełączniki pod nią zostawiają ostre HUD, kokpit lub MFD, przyrządy i Kneeboard
effect-sharp-hud = Bez HUD
    .facts =
        Pikselizacja: HUD (i wyświetlacz w hełmie) jest rysowany ostro nad spikselizowanym obrazem, aby jego cienkie symbole pozostały czytelne
effect-sharp-cockpit = Bez kokpitu (przyciski, przełączniki...)
    .facts =
        Pikselizacja: panel, rama, fotel, przyciski, przełączniki, drążek sterowy i przepustnica kokpitu są rysowane ostro
effect-sharp-gauges = Bez MFD, przyrządów i Kneeboardu
    .facts =
        Pikselizacja: MFD, sztuczny horyzont, przyrządy, lampki, wskazania, podpisy i Kneeboard są rysowane ostro
options-needs-pixelize = Wymaga pikselizacji
