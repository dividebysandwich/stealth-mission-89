# Kneeboard (src/kneeboard.rs, font poteza), planer misije (src/planner.rs) i spremljene
# misije (src/savegame.rs). Navigacijske kratice (PT, BRG, ETA, PRI, SEC, KM, KT) ostaju na
# engleskom.

kb-brief = BRIEF
kb-route = RUTA
kb-map = KARTA
kb-target = CILJ
kb-keys = TIPKE
kb-tip = Karte, bilješke misije i vodič kroz tipke
kb-geran-file = GERAN-2 (DOSJE)
kb-objectives = CILJEVI
kb-primary = GLAVNI: { $objective }
kb-secondary = SPOREDNI: { $objective }
kb-times = VREMENA
kb-deadline = ROK { $time }
kb-now = SADA { $time }
kb-conditions = UVJETI
kb-moon = MJESEC: { $phase } ({ $lit } % OSVIJETLJEN)
kb-armament = NAORUŽANJE
kb-roe = PRAVILA ANGAŽIRANJA
kb-total = UKUPNO { $km } KM PRI { $kt } KT
kb-fuel = GORIVO { $lbs } LBS
kb-preparing = PRIPREMA...
kb-airborne = U ZRAKU
kb-developing = RAZVIJANJE...
kb-no-photo = NEMA FOTOGRAFIJE
kb-airborne-target = ZRAČNI CILJ
kb-scroll-vr = POMICANJE: ANALOGNI ŠTAP, PRST NA PLOČI
kb-scroll-desktop = POMICANJE: KOTAČIĆ MIŠA NAD PLOČOM

## Planer misije

planner-low = NISKO 500 FT
planner-medium = SREDNJE 5000 FT
planner-high = VISOKO 20000 FT
planner-title = PLANER MISIJE
planner-route = Ruta
planner-base = BAZA
planner-total = UKUPNO { $km } KM  { $time }
planner-briefed = Zadana ruta { $km } km, gorivo { $fuel } lbs
planner-coverage = Radarska pokrivenost za
    .note = Radarska pokrivenost pretpostavlja horizontalni let od 480 kt sa zatvorenim vratima spremnika.
planner-detected = Otkriveno km rute: { $km }
planner-pulse = Domet otkrivanja pulsnog radara (P)
planner-doppler = Domet otkrivanja Dopplerovog radara (D)
planner-missile = Domet projektila
planner-uav = Neprijateljski UAV
planner-edit-hint = Povucite točku da je pomaknete, kartu da je pomičete. Kliknite na dionicu da dodate točku, desni klik je uklanja. Kotačić: zumiranje.
planner-reset = PONIŠTI
planner-view-hint = Povucite kartu da je pomičete. Kotačić: zumiranje.

## Spremljene misije

saves-title = SPREMLJENE MISIJE
saves-subtitle = Spremljeno tijekom leta iz izbornika: SPREMI MISIJU
saves-none = Nema spremljenih misija
saves-line = { $theater }  -  { $aircraft }  -  vrijeme misije { $time }
saves-saved-at = Spremljeno { $date }
saves-unreadable = Nečitljiv spremak (druga verzija)
saves-delete =
    .facts = Izbriši ovaj spremak
saves-back =
    .facts = Natrag na zaslon pojedinačne misije
