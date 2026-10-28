# Il-kneeboard (src/kneeboard.rs, font tal-linji), il-planner tal-missjoni (src/planner.rs) u
# l-missjonijiet maħżuna (src/savegame.rs). L-abbrevjazzjonijiet tan-navigazzjoni (PT, BRG, ETA,
# PRI, SEC, KM, KT) jibqgħu bl-Ingliż.

kb-brief = BRIEF
kb-route = ROTTA
kb-map = MAPPA
kb-target = MIRA
kb-keys = TASTI
kb-tip = Mapep, noti tal-missjoni u l-gwida tat-tasti
kb-geran-file = GERAN-2 (ARKIVJU)
kb-objectives = GĦANIJIET
kb-primary = PRIMARJU: { $objective }
kb-secondary = SEKONDARJU: { $objective }
kb-times = ĦINIJIET
kb-deadline = TERMINU { $time }
kb-now = ISSA { $time }
kb-conditions = KONDIZZJONIJIET
kb-moon = QAMAR: { $phase } ({ $lit }% IMNAWWAR)
kb-armament = ARMAMENTI
kb-roe = REGOLI TAL-IMPENN
kb-total = TOTAL { $km } KM B'{ $kt } KT
kb-fuel = FUEL { $lbs } LBS
kb-preparing = QED JITTAĦĦAR...
kb-airborne = FIL-AJRU
kb-developing = QED JIĠI IŻVILUPPAT...
kb-no-photo = EBDA RITRATT
kb-airborne-target = MIRA FIL-AJRU
kb-scroll-vr = SKROLLJA: THUMBSTICK, SBAGĦ FUQ IL-BORD
kb-scroll-desktop = SKROLLJA: ROTA TAL-MOUSE FUQ IL-BORD

## Planner tal-missjoni

planner-low = LOW 500 FT
planner-medium = MED 5000 FT
planner-high = HIGH 20000 FT
planner-title = PLANNER TAL-MISSJONI
planner-route = Rotta
planner-base = BAŻI
planner-total = TOTAL { $km } KM  { $time }
planner-briefed = Rotta brifjata { $km } km, fuel { $fuel } lbs
planner-coverage = Kopertura tar-radar għal
    .note = Il-kopertura tar-radar tassumi titjira ċatta f'480kt, bay doors magħluqa.
planner-detected = { $km } km tar-rotta skoperti
planner-pulse = Firxa ta' skoperta tar-radar pulse (P)
planner-doppler = Firxa ta' skoperta tar-radar doppler (D)
planner-missile = Firxa tal-missili
planner-uav = UAV tal-għadu
planner-edit-hint = Kaxkar punt biex tmexxih, il-mappa biex tgħaddiha. Ikklikkja fuq segment biex iżżid punt, ikklikkja bil-lemin biex tneħħih. Rota: zoom.
planner-reset = RESET
planner-view-hint = Kaxkar il-mappa biex tgħaddiha. Rota: zoom.

## Missjonijiet maħżuna

saves-title = MISSJONIJIET MAĦŻUNA
saves-subtitle = Maħżuna matul it-titjira mill-menu: SAVE MISSION
saves-none = L-ebda missjoni maħżuna
saves-line = { $theater }  -  { $aircraft }  -  ħin tal-missjoni { $time }
saves-saved-at = Maħżun { $date }
saves-unreadable = Ħażna li ma tinqarax (verżjoni oħra)
saves-delete =
    .facts = Ħassar din il-ħażna
saves-back =
    .facts = Lura għall-iskrin tal-missjoni waħda
