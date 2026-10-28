# Planšetė (src/kneeboard.rs, linijinis šriftas), misijos planuoklis (src/planner.rs) ir
# išsaugotos misijos (src/savegame.rs). Navigacijos santrumpos (PT, BRG, ETA, PRI, SEC, KM, KT)
# lieka angliškos.

kb-brief = BRIEF
kb-route = MARŠRUTAS
kb-map = ŽEMĖLAPIS
kb-target = TAIKINYS
kb-keys = KLAVIŠAI
kb-tip = Žemėlapiai, misijos užrašai ir klavišų vadovas
kb-geran-file = GERAN-2 (BYLA)
kb-objectives = UŽDUOTYS
kb-primary = PAGRINDINĖ: { $objective }
kb-secondary = ANTRINĖ: { $objective }
kb-times = LAIKAI
kb-deadline = TERMINAS { $time }
kb-now = DABAR { $time }
kb-conditions = SĄLYGOS
kb-moon = MĖNULIS: { $phase } (APŠVIESTA { $lit } %)
kb-armament = GINKLUOTĖ
kb-roe = KOVOS TAISYKLĖS
kb-total = IŠ VISO { $km } KM, { $kt } KT
kb-fuel = KURAS { $lbs } LB
kb-preparing = RUOŠIAMA...
kb-airborne = ORE
kb-developing = DIEGIAMA...
kb-no-photo = NĖRA NUOTRAUKOS
kb-airborne-target = ORO TAIKINYS
kb-scroll-vr = VERTIMAS: SVIRTIS, PIRŠTAS ANT PLANŠETĖS
kb-scroll-desktop = VERTIMAS: PELĖS RATUKAS VIRŠ PLANŠETĖS

## Misijos planuoklis

planner-low = ŽEMAI 500 FT
planner-medium = VIDUTINIAI 5000 FT
planner-high = AUKŠTAI 20000 FT
planner-title = MISIJOS PLANUOKLIS
planner-route = Maršrutas
planner-base = BAZĖ
planner-total = IŠ VISO { $km } KM  { $time }
planner-briefed = Maršrutas pagal brifingą { $km } km, kuras { $fuel } lb
planner-coverage = Radarų aprėptis:
    .note = Radarų aprėptis skaičiuojama pagal 480 kt horizontalų skrydį, skyrius uždarytas.
planner-detected = Maršruto aptikta: { $km } km
planner-pulse = Impulsinio radaro (P) aptikimo nuotolis
planner-doppler = Doplerio radaro (D) aptikimo nuotolis
planner-missile = Raketos nuotolis
planner-uav = Priešo UAV
planner-edit-hint = Vilkite tašką, kad jį perkeltumėte, žemėlapį - kad slinktumėte. Spustelėkite atkarpą, kad pridėtumėte tašką, dešiniuoju klavišu pašalinsite. Ratukas: didinti.
planner-reset = ATSTATYTI
planner-view-hint = Vilkite žemėlapį, kad slinktumėte. Ratukas: didinti.

## Išsaugotos misijos

saves-title = IŠSAUGOTOS MISIJOS
saves-subtitle = Išsaugota skrydžio metu iš meniu: IŠSAUGOTI MISIJĄ
saves-none = Išsaugotų misijų nėra
saves-line = { $theater }  -  { $aircraft }  -  misijos laikas { $time }
saves-saved-at = Išsaugota { $date }
saves-unreadable = Neperskaitomas išsaugojimas (kita versija)
saves-delete =
    .facts = Ištrinti šį išsaugojimą
saves-back =
    .facts = Atgal į pavienės misijos ekraną
