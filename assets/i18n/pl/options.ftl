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
options-tooltips = Tooltipy kokpitu
    .facts = Czym jest element sterowania lub wyświetlacz kokpitu, pod myszą lub pod palcem w VR
options-section-vr-size = Rozmiar kokpitu w VR
options-vr-size =
    .facts =
        Rozmiar kokpitu (i świata) w goglach
        100%: naturalna wielkość

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
