# A kneeboard (src/kneeboard.rs, vonalbetű), a küldetéstervező (src/planner.rs) és a
# mentett küldetések (src/savegame.rs). A navigációs rövidítések (PT, BRG, ETA, PRI, SEC, KM, KT)
# angolul maradnak.

kb-brief = ELIGAZÍTÁS
kb-route = ÚTVONAL
kb-map = TÉRKÉP
kb-target = CÉLPONT
kb-keys = BILLENTYŰK
kb-tip = Térképek, küldetésjegyzetek és a billentyűkalauz
kb-geran-file = GERAN-2 (AKTA)
kb-objectives = CÉLOK
kb-primary = ELSŐDLEGES: { $objective }
kb-secondary = MÁSODLAGOS: { $objective }
kb-times = IDŐPONTOK
kb-deadline = HATÁRIDŐ { $time }
kb-now = MOST { $time }
kb-conditions = KÖRÜLMÉNYEK
kb-moon = HOLD: { $phase } ({ $lit }% MEGVILÁGÍTVA)
kb-armament = FEGYVERZET
kb-roe = HARCI SZABÁLYOK
kb-total = ÖSSZESEN { $km } KM, { $kt } KT SEBESSÉGGEL
kb-fuel = ÜZEMANYAG { $lbs } LBS
kb-preparing = ELŐKÉSZÍTÉS...
kb-airborne = LEVEGŐBEN
kb-developing = ELŐHÍVÁS...
kb-no-photo = NINCS FÉNYKÉP
kb-airborne-target = LÉGI CÉLPONT
kb-scroll-vr = GÖRGETÉS: THUMBSTICK, UJJ A TÁBLÁN
kb-scroll-desktop = GÖRGETÉS: EGÉRKERÉK A TÁBLA FÖLÖTT

## Küldetéstervező

planner-low = ALACSONY 500 FT
planner-medium = KÖZEPES 5000 FT
planner-high = MAGAS 20000 FT
planner-title = KÜLDETÉSTERVEZŐ
planner-route = Útvonal
planner-base = BÁZIS
planner-total = ÖSSZESEN { $km } KM  { $time }
planner-briefed = Eligazított útvonal { $km } km, üzemanyag { $fuel } lbs
planner-coverage = Radarlefedettség ehhez:
    .note = A radarlefedettség 480 kt-os vízszintes repülést és zárt rekeszajtókat feltételez.
planner-detected = Az útvonal { $km } km-e észlelhető
planner-pulse = Impulzusradar (P) észlelési távolsága
planner-doppler = Doppler-radar (D) észlelési távolsága
planner-missile = Rakéta hatótávolsága
planner-uav = Ellenséges UAV
planner-edit-hint = Húzz egy pontot az áthelyezéséhez, a térképet az eltoláshoz. Kattints egy szakaszra pont hozzáadásához, jobb kattintás az eltávolításhoz. Kerék: nagyítás.
planner-reset = VISSZAÁLLÍTÁS
planner-view-hint = Húzd a térképet az eltoláshoz. Kerék: nagyítás.
planner-layers = RÉTEGEK
planner-layer-sams = Radar / SAM-állás
planner-radar-types = Radartípusok
planner-layer-pulse = Impulzusradar észlelési távolsága
planner-layer-doppler = Doppler-radar észlelési távolsága
planner-layer-airfield = Repülőtér

## Mentett küldetések

saves-title = MENTETT KÜLDETÉSEK
saves-subtitle = Repülés közben a menüből mentve: KÜLDETÉS MENTÉSE
saves-none = Nincs mentett küldetés
saves-line = { $theater }  -  { $aircraft }  -  küldetési idő { $time }
saves-saved-at = Mentve: { $date }
saves-unreadable = Olvashatatlan mentés (másik verzió)
saves-delete =
    .facts = A mentés törlése
saves-back =
    .facts = Vissza az egyedi küldetések képernyőjére
