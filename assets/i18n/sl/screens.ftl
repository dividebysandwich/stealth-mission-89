# Kneeboard (src/kneeboard.rs, črtna pisava), načrtovalnik misij (src/planner.rs) in
# shranjene misije (src/savegame.rs). Navigacijske kratice (PT, BRG, ETA, PRI, SEC, KM, KT)
# ostanejo v angleščini.

kb-brief = BRIFING
kb-route = POT
kb-map = KARTA
kb-target = CILJ
kb-keys = TIPKE
kb-tip = Karte, opombe o misiji in vodnik po tipkah
kb-geran-file = GERAN-2 (DOSJE)
kb-objectives = CILJI
kb-primary = GLAVNI: { $objective }
kb-secondary = DRUGOTNI: { $objective }
kb-times = ČASI
kb-deadline = ROK { $time }
kb-now = ZDAJ { $time }
kb-conditions = RAZMERE
kb-moon = LUNA: { $phase } ({ $lit } % OSVETLJENA)
kb-armament = OBOROŽITEV
kb-roe = PRAVILA ANGAŽIRANJA
kb-total = SKUPAJ { $km } KM PRI { $kt } KT
kb-fuel = GORIVO { $lbs } LBS
kb-preparing = PRIPRAVA...
kb-airborne = V ZRAKU
kb-developing = RAZVIJANJE...
kb-no-photo = BREZ FOTOGRAFIJE
kb-airborne-target = ZRAČNI CILJ
kb-scroll-vr = DRSENJE: PALČNA PALICA, PRST NA PLOŠČI
kb-scroll-desktop = DRSENJE: KOLESCE MIŠKE NAD PLOŠČO

## Načrtovalnik misij

planner-low = NIZKO 500 FT
planner-medium = SREDNJE 5000 FT
planner-high = VISOKO 20000 FT
planner-title = NAČRTOVALNIK MISIJ
planner-route = Pot
planner-base = BAZA
planner-total = SKUPAJ { $km } KM  { $time }
planner-briefed = Pot iz brifinga { $km } km, gorivo { $fuel } lbs
planner-coverage = Radarska pokritost za
    .note = Radarska pokritost predpostavlja ravni let pri 480 kt z zaprtimi vrati bombnika.
planner-detected = { $km } km poti odkritih
planner-pulse = Doseg zaznave pulznega radarja (P)
planner-doppler = Doseg zaznave dopplerskega radarja (D)
planner-missile = Doseg izstrelkov
planner-uav = Sovražni UAV
planner-edit-hint = Povlecite točko za premik, karto za pomik. Kliknite odsek za dodajanje točke, desni klik jo odstrani. Kolesce: povečava.
planner-reset = PONASTAVI
planner-view-hint = Povlecite karto za pomik. Kolesce: povečava.
planner-layers = PLASTI
planner-layer-sams = Radar / položaj SAM
planner-radar-types = Vrste radarjev
planner-layer-pulse = Doseg zaznave pulznega radarja
planner-layer-doppler = Doseg zaznave dopplerjevega radarja
planner-layer-airfield = Letališče

## Shranjene misije

saves-title = SHRANJENE MISIJE
saves-subtitle = Shranjeno med letom iz menija: SHRANI MISIJO
saves-none = Ni shranjenih misij
saves-line = { $theater }  -  { $aircraft }  -  čas misije { $time }
saves-saved-at = Shranjeno { $date }
saves-unreadable = Neberljiv zapis (druga različica)
saves-delete =
    .facts = Izbriši ta zapis
saves-back =
    .facts = Nazaj na zaslon posamezne misije
