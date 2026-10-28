# Hlavní nabídka a obrazovka jednotlivé mise (src/menu.rs).

## Operační prostory (stealth_host::theater::THEATERS)

theater-libya = Libye (1986)
    .facts =
        Zátoka Velká Syrta a libyjské pobřeží
        Doba operace El Dorado Canyon
        Většinou jasná pouštní obloha
theater-persian_gulf = Perský záliv (1984)
    .facts =
        Doba tankerové války mezi Íránem a Irákem
        Nad mořem a pobřežní pouští
        Nejjasnější počasí ze všech operačních prostorů
theater-north_cape = Nordkapp
    .facts =
        Arktické Norsko a poloostrov Kola
        Domovina sovětské Severní flotily
        Časté nízké zatažení
theater-central_europe = Střední Evropa
    .facts =
        NATO proti Varšavské smlouvě nad Německem
        Jasná obloha jen v 15 % případů
theater-vietnam = Vietnam
    .facts =
        Oblast Hanoje a Haiphongu
        Džungle a horský terén
        Bouřky u 30 % misí
theater-middle_east = Blízký východ
    .facts =
        Východní Středomoří a Levanta
        Většinou jasná obloha
theater-desert_storm = Pouštní bouře
    .facts =
        Kuvajt a Irák, 1991
        Skutečný bojový debut F-117A
        Většinou jasná pouštní obloha
theater-cuba = Kuba
    .facts =
        Karibský ostrovní operační prostor
        Námořní a pozemní cíle
        Tropy: bouřky u 15 % misí
theater-korea = Korea
    .facts =
        Korejský poloostrov severně od DMZ
        Hornatý terén
        Proměnlivé počasí, možná je jakákoli obloha
theater-ukraine = Ukrajina (2026)
    .facts =
        Východní Ukrajina, Krym a jih Ruska
        Frontová linie od Charkova po Cherson hoří
        Otevřená pole, zničená města na frontě

## Situace (pravidla nasazení)

roe-0 = Studená válka
    .facts =
        Skrytá operace v době míru
        Nejvíce záleží na tom, aby vás nikdo neodhalil
        Méně nepřátelských letounů na hlídce
        Přesná pravidla nasazení jsou v briefingu
roe-1 = Omezená válka
    .facts =
        Regionální ozbrojený konflikt
        Nepřátelská protivzdušná obrana v pohotovosti
        Více nepřátelských letounů než ve studené válce
roe-2 = Konvenční válka
    .facts =
        Válka v plném rozsahu
        Všechny nepřátelské síly jsou aktivní
        Více nepřátelských letounů než ve studené válce

## Druhy misí

kind-strike = Úder
    .facts =
        Zničit, vyfotografovat nebo zasáhnout pozemní cíl
        Hlavní a vedlejší cíl
        Výzbroj vzduch-země
kind-air = Vzduch-vzduch
    .facts =
        Zachytit nepřátelský letoun
        Transportní letouny, bombardéry nebo stíhací hlídky
        Výzbroj střel vzduch-vzduch
kind-strike-training = Trénink úderu
    .facts =
        Cvičná úderná mise
        Naučte se zbraně a navigaci
kind-air-training = Trénink vzduch-vzduch
    .facts =
        Cvičná mise vzduch-vzduch
        Naučte se zachycení a střely
kind-drone-defense = Obrana proti dronům
    .facts =
        Pouze Ukrajina
        Zastavit vlnu útočných dronů Geran dřív, než zasáhnou město nebo jadernou elektrárnu
        Poté bombardovat jejich odpalovací stanoviště

## Protivníci

opponents-0 = Nezkušení
    .facts =
        Nezkušené nepřátelské posádky
        Nejsnazší nastavení
opponents-1 = Běžní
    .facts =
        Průměrné nepřátelské posádky
        Výchozí nastavení
opponents-2 = Veteráni
    .facts =
        Zkušené nepřátelské posádky
        Více lodí a hlídek
opponents-3 = Elita
    .facts =
        Nejlepší nepřátelští piloti a posádky SAM
        Nejtěžší nastavení

## Počasí

sky-random = Náhodné
    .facts =
        Losuje se podle podnebí operačního prostoru
        Mění se s každou novou misí
        Oblaka brání ve výhledu oběma stranám
sky-clear = Jasno
    .facts =
        Bez oblačnosti
        Dohlednost 40 km
        Klasická obloha z éry DOSu
sky-scattered = Polojasno
    .facts =
        Kupovitá oblačnost 20-45 %, základna 2 500-6 000 ft
        Občas tenká vysoká oblačnost altostratus
        Dohlednost 30 km
sky-broken = Oblačno
    .facts =
        Kupovitá oblačnost 45-70 %, základna 2 000-5 000 ft
        Často vrstva stratocumulů nad ní
        Dohlednost 25 km
sky-overcast = Zataženo
    .facts =
        Stratus 80-100 %, základna 1 500-5 000 ft
        Pod ním několik cárů kupovité oblačnosti
        Dohlednost 20 km
sky-storm = Bouřka
    .facts =
        Věže cumulonimbů až do 40 000 ft
        Nízký stratus a kupovitá oblačnost od 1 000 ft
        Dohlednost 8 km
menu-random-cover = Náhodná oblačnost
    .facts =
        Pokrytí hlavní vrstvy se losuje podle počasí
        Zrušte zaškrtnutí a nastavte si je sami
        Hlavní vrstva: stratus při zatažení a bouřce, jinak kupovitá oblačnost
menu-cover-slider =
    .facts =
        Podíl oblohy, který hlavní vrstva pokrývá
        Na jasnou oblohu nemá vliv

## Měsíc

moon-date = Datum mise
    .facts =
        Skutečný měsíc v den briefingu
        Vyžaduje přepínač denní doby (moderní)
moon-0 = Nov
    .facts =
        Žádný měsíční svit: černá noc, jen světla a hvězdy
        Nepřátelští piloti vás uvidí ze 2 km
moon-1 = Dorůstající srpek
    .facts = Tenký měsíc na večerní obloze, zapadá před půlnocí
moon-2 = První čtvrť
    .facts = Z poloviny osvětlený, večer nad obzorem, zapadá kolem půlnoci
moon-3 = Dorůstající měsíc
    .facts = Jasný, nad obzorem od odpoledne až po půlnoc
moon-4 = Úplněk
    .facts =
        Nad obzorem celou noc, o půlnoci vysoko
        Nepřátelští piloti vás uvidí ze 4 km
moon-5 = Ubývající měsíc
    .facts = Jasný, vychází večer, svítí do rána
moon-6 = Poslední čtvrť
    .facts = Z poloviny osvětlený, vychází kolem půlnoci
moon-7 = Ubývající srpek
    .facts = Tenký měsíc vycházející před úsvitem

## Letouny

aircraft-f117 =
    .facts =
        Útočný letoun stealth
        Podzvukový, 7 g
        Moderní a klasický kokpit
aircraft-f16 =
    .facts =
        Víceúčelová stíhačka
        Třída Mach 2, 9 g, rychlé klonění
        Kokpit F-16 s RWR
aircraft-tornado =
    .facts =
        Útočný letoun pro nízké výšky
        Natáčecí křídla se sklápějí s rychlostí
        Moderní kokpit, tři MFD
aircraft-a10 =
    .facts =
        Letoun blízké letecké podpory
        Pomalý a odolný, kanón 30 mm
        Kokpit A-10C, dvě MFD
aircraft-uh60 =
    .facts =
        Víceúčelový vrtulník
        Vysazuje vojáky: přistaňte poblíž cíle
        Cyklická a kolektivní páka
aircraft-ah1z =
    .facts =
        Bitevní vrtulník
        Kanón ve věži, Hellfire, AIM-9
        Kanón a Hellfire sledují kontejner

## Sekce a skupiny

section-theater = Operační prostor
section-opponents = Protivníci
section-situation = Situace
section-mission = Mise
section-moon = Měsíc
section-start-time = Čas startu
section-weather = Počasí
section-realism = Realismus
section-tacview = Tacview
section-mission-summary = Shrnutí mise
section-briefing = Briefing
group-mission = Mise
group-conditions = Podmínky
group-vehicle = Stroj
group-simulation = Simulace

## Možnosti mise

menu-start-random = Náhodně
    .facts =
        Losuje se s každou novou misí, jako v klasickém režimu
        06:00-08:00 ve dne nebo 18:00-20:00 v noci
        Zrušte zaškrtnutí a zvolte hodinu
menu-start-slider =
    .facts =
        Čas vzletu
        Noc od 18:00 do 06:00
        Zachová se při NOVÉ MISI
menu-realistic-weapons = Realistické zbraně
    .facts =
        Omezení skutečného letounu
        2 zbraňové šachty místo 4
        Žádné střely AIM-9 ani AIM-120
        Žádné starty ani přistání na lodi
menu-crash-protection = Ochrana před nárazem
    .facts =
        U země automaticky přejde do stoupání (podvozek zasunut)
        Dotyk se zemí a srážky nejsou smrtelné
        Vypnuto: do země lze vždy narazit
menu-tacview-export = Export
    .facts =
        Uloží každý let jako záznam Tacview (.acmi)
        Ve složce hry pro vaše soubory, v podsložce tacview
menu-tacview-real-time = Telemetrie v reálném čase
    .facts =
        Posílá let do Tacview, zatímco letíte
        V Tacview: Record > Real-Time Telemetry
        Adresa 127.0.0.1, port 42674 (--tacview-address)
switch-game = HRA
switch-real = REÁLNĚ
menu-damage = Poškození
    .facts =
        Poškození zbraněmi
        Hra: klasické poškození; zásahy se sčítají, ale letoun nikdy nesestřelí
        Reálně: zásah střelou může letoun zničit (katapultujte se!), poškodí více systémů a často způsobí požár, který vyhasne nebo letoun roztrhne
        Reálně: poškozená kormidla vyvedou letoun z trimu nebo způsobí kmitání
menu-guidance = Navádění
    .facts =
        Navádění střel
        Klasické: střely letí přímo na cíl
        Reálně: proporcionální navigace do bodu srážky; motory dohoří a střely klouzají dál a v každé zatáčce ztrácejí rychlost (kouř jen při hoření motoru)
        Reálně: SAM stoupají obloukem proti vzdáleným cílům; AIM-120 letí 3 s rovně (přitáhněte a vrhněte ji obloukem)
menu-air-defence = Protivzdušná obrana
    .facts =
        Protivzdušná obrana v nízkých výškách
        Klasické: jen stanoviště SAM a stíhačky
        Reálně: radarem naváděné kanóny ZSU-23-4 (jejich radar vás vidí jako radar SAM) a flakové baterie S-60 hlídají letiště a cíle; týmy MANPADS ve městech pálí střelami SA-14 na to, co vidí
        Let nízko nad hájenými místy se stává nebezpečným; bomby a kanóny je ničí

## Hlavní obrazovka

menu-title = STEALTH MISSION 89
menu-subtitle = SYSTÉM PLÁNOVÁNÍ MISÍ
menu-single-mission = JEDNOTLIVÁ MISE
    .facts =
        Jedna mise v operačním prostoru podle vašeho výběru
        Zvolte možnosti, naplánujte trasu a výzbroj
menu-career = KARIÉRA
    .facts =
        Záznamy pilota: hodnost, vyznamenání, bojové lety
        Doba služby: 12 bojových letů v jednom operačním prostoru
        Co zničíte, zůstane zničené
menu-campaign = DYNAMICKÁ KAMPAŇ
    .facts =
        Dynamická kampaň v reálném čase
        Dobývejte nepřátelská města a letiště
        Mise podle živé situace
menu-multiplayer = MULTIPLAYER
    .facts = Koop v dynamické kampani
    .join = Připojit se k serveru z veřejného seznamu
    .host = Připojit se k serveru, nebo vytvořit kampaň a létat v ní
menu-options = NASTAVENÍ
    .facts =
        Hra: styl kokpitu, HUD a avioniky, automatický trim
        Grafika, hlasitost zvuku, přiřazení kláves a joysticku
        Uloženo pro další lety
    .short = Hra, grafika, zvuk a ovládání
menu-download = STÁHNOUT
    .facts =
        Desktopová hra pro Windows, Linux a macOS
        S podporou VR a multiplayeru
        Steam Deck a Steam Frame: Linux AppImage
web-play = HRÁT V PROHLÍŽEČI
    .facts = Přímo tady, nic se neinstaluje
web-download-platforms = Windows, Linux a macOS, Steam Deck a Steam Frame
web-download-version = Stahovaná verze
web-gives-vr = VR
    .why = Létejte ve VR na libovolném headsetu s OpenXR a na Steam Frame
web-gives-hotas = HOTAS
    .why = Plná podpora gamepadů, joysticků, plynových pák a pedálů
web-gives-multiplayer = Lepší multiplayer
    .why = Přímé spojení UDP se servery pro menší zpoždění
web-gives-server = Vlastní server
    .why = Hostujte koop kampaň nebo provozujte dedikovaný server
web-gives-head-tracking = Head tracking
    .why = Trackery typu TrackIR přes OpenTrack
web-gives-tacview = Tacview
    .why = Záznamy letů a telemetrie v reálném čase
web-gives-keep = Navždy vaše
    .why = I kdyby web zanikl, vlastní kopii máte stále
menu-discord = DISCORD
    .facts =
        Létejte společně, hlaste chyby a sdílejte mise
        Sledujte vývoj
menu-quit = KONEC
    .facts = Ukončit do systému
menu-version = VERZE { $version }

## Obrazovka jednotlivé mise

menu-career-sortie = BOJOVÝ LET KARIÉRY
menu-tour-sortie = { $pilot }  //  BOJOVÝ LET { $sortie } Z { $length }
menu-msn = SYSTÉM PLÁNOVÁNÍ MISÍ  //  MSN { $seed }
menu-no-targets = Pro tuto dobu služby nezbyly žádné cíle
menu-plan-route = NAPLÁNOVAT TRASU
    .facts =
        Mapa operačního prostoru s pokrytím nepřátelských radarů a SAM
        Přidávejte, posouvejte a odstraňujte navigační body
        Cíle a základna přistání zůstávají pevné
menu-loadout = VÝZBROJ
    .facts =
        Zvolte výzbroj na každé šachtě nebo pylonu
        Začíná se zbraněmi z briefingu
menu-generating = GENEROVÁNÍ...
menu-back-records = Zpět na záznamy pilotů
menu-back-main = Zpět do hlavní nabídky
menu-new-sortie = NOVÝ BOJOVÝ LET
menu-new-mission = NOVÁ MISE
    .facts =
        Vylosuje jiné cíle
        Zachová všechna nastavení a čas startu
menu-load = NAČÍST
    .facts =
        Mise uložené za letu (nabídka: ULOŽIT MISI)
        Pokračujte odtud, kde byly uloženy
menu-fly = LETĚT
    .facts = Vzlétnout na zadanou misi
menu-fly-vr = LETĚT VE VR
    .facts =
        Restartuje hru v headsetu VR
        Poletíte přesně tuto misi
    .no-mission = Nejdřív vygenerujte misi
    .no-runtime = Není aktivní žádný runtime OpenXR (Monado, WiVRn, SteamVR)
menu-vr-failed = nepodařilo se spustit ve VR: { $error }

## Shrnutí mise

summary-theater = OPERAČNÍ PROSTOR
summary-situation = SITUACE
summary-mission = MISE
summary-opponents = PROTIVNÍCI
summary-time = ČAS
summary-weather = POČASÍ
summary-fuel = PALIVO
summary-night = Noc
summary-day = Den
summary-time-value = { $daytime }  odlet { $depart }
summary-weather-value = { $sky }  dohl. { $vis } km
summary-fuel-value = { $fuel } lbs
summary-armament = Výzbroj: { $stores }
summary-intelligence = ZPRAVODAJSKÉ INFORMACE
