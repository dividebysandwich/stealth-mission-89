# Prozor OPCIJE (src/options.rs) i njegove stranice u izborniku tijekom leta.

options-title = OPCIJE
options-tab-game = IGRA
options-tab-graphics = GRAFIKA
options-tab-sound = ZVUK
options-tab-controls = UPRAVLJANJE
options-footer = Spremljeno za sljedeće letove; prekidači realizma biraju se uz svaku igru

## IGRA

options-section-language = Jezik
options-section-classic-modern = Klasično / Moderno
options-cockpit = Kokpit
    .facts =
        Moderno: zasloni visoke razlučivosti s tipkama na rubu
        Klasično: raspored instrumentne ploče iz DOS-ova doba i zasloni 77x60 (F-117A)
options-hud = HUD
    .facts =
        Moderno: zeleni HUD visoke razlučivosti u stilu F-16 s načinima NAV, AA i AG
        Klasično: HUD iz DOS-ova doba
options-avionics = Avionika
    .facts =
        Moderno: MFD formati u stilu F-16 na oba MFD-a (prikaz situacije, HARM, ciljnički pod, karta, naoružanje) s pomičnim kursorom
        Klasično: karta i TAC lijevo, kamera, WAY, WPN i ORD desno
options-flight-model = Model leta
    .facts =
        Moderno: aerodinamički model leta s električnim upravljanjem (fly-by-wire): palica zapovijeda brzinu propinjanja i valjanja; otpuštena, nos ostaje na mjestu i putanja se drži
        Klasično: model leta iz DOS-ova doba
options-section-flying = Pilotiranje
options-auto-trim = Automatski trim
    .facts =
        S otpuštenom palicom trim drži stav u kojem ste je ostavili
        Helikopteri: nos se više ne diže s brzinom; mlažnjaci: kompenzira oštećene upravljačke površine
        Ručni trim: Alt + strelice, Alt + Home ga centrira
options-tooltips = Tooltipi kokpita
    .facts = Što je pojedina komanda ili zaslon u kokpitu, pod mišem ili vrhom prsta u VR-u
options-section-vr-size = Veličina kokpita u VR-u
options-vr-size =
    .facts =
        Veličina kokpita (i svijeta) u headsetu
        100 %: prirodna veličina

## GRAFIKA

options-section-rendering = Iscrtavanje
options-shaders = Shaderi
    .facts =
        Moderno: suptilno usmjereno sjenčanje, blaga izmaglica na daljinu
        Klasično: ravno sjenčanje
options-shadows = Sjene
    .facts =
        Moderno: sunčeve sjene u kokpitu i svijetu (troši GPU)
        Klasično: bez sjena
options-point-lights = Točkasta svjetla
    .facts =
        Moderno: rasvjeta kokpita i zaslona; eksplozije, stupovi dima i svjetla piste osvjetljavaju scenu
        Klasično: samo svjetlo Sunca i neba
options-time-of-day = Doba dana
    .facts =
        Moderno: Sunce i Mjesec prate sat misije, nebo tamni i svijetli sa Suncem; zvijezde i mjesečeva mijena noću
        Moderno: mjesečina određuje s koje vas daljine neprijateljski piloti vide noću (2 km u noći bez Mjeseca, 4 km pod uštapom)
        Klasično: spora promjena boja i nepomično Sunce
options-msaa = Izglađivanje rubova (MSAA)
    .facts =
        Glatki rubovi: 4x MSAA, 2x u VR-u (troši GPU)
        Isključeno: oštri rubovi piksela
options-auto-detail = Automatski detalji okoliša (VR)
    .facts =
        U headsetu se domet gradova, šuma i polja smanjuje dok sličice kasne i vraća kad ima viška vremena
        Grafičke postavke ostaju kako su odabrane
options-sun-moon = Sunce i Mjesec
    .facts =
        Sunce na nebu danju, Mjesec i zvijezde noću
        Mjesec prikazuje mijenu misije

## ZVUK

options-section-volume = Glasnoća
options-volume-master = Glavna
    .facts = Svaki zvuk
options-volume-engine = Motor
    .facts = Vaš motor ili rotor i zrakoplovi koje čujete oko sebe
options-volume-effects = Efekti
    .facts = Oružje, eksplozije, podvozje, radio i ostalo
options-volume-warnings = Upozorenja
    .facts = Sirena prekida uzgona, radarska upozorenja i upozorenja na projektile, požarna sirena, tonovi Sidewindera
options-section-cockpit = Kokpit
options-low-pass = Niski propust motora
    .facts =
        Unutar zrakoplova motor zvuči prigušeno (kabina i kaciga odsijecaju visoke tonove)
        Isključeno: isto kao izvana

## UPRAVLJANJE

options-section-bindings = Dodjele
options-keys-joystick = TIPKE I JOYSTICK
    .facts =
        Dodjela tipki, osi i gumba joysticka i HOTAS-a
        Spremljeno za sljedeće letove
options-section-head-tracking = Praćenje glave i očiju
options-head-tracking = Praćenje glave i očiju
    .facts =
        Vaša glava okreće i pomiče pogled u kokpitu (na zaslonu; u VR-u to radi headset)
        Program za praćenje (OpenTrack...): FreeTrack izlaz ili UDP na 127.0.0.1 port 4242
        Home: trenutni položaj glave postaje središte
options-tracking = Praćenje ({ $source })
options-no-tracker = Nijedan tracker nije otkriven: pokrenite program za praćenje
