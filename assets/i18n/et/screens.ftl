# Kneeboard (src/kneeboard.rs, joonfont), missiooniplaneerija (src/planner.rs) ja
# salvestatud missioonid (src/savegame.rs). Navigatsioonilühendid (PT, BRG, ETA, PRI, SEC,
# KM, KT) jäävad inglise keelde.

kb-brief = BRIIFING
kb-route = MARSRUUT
kb-map = KAART
kb-target = SIHTMÄRK
kb-keys = KLAHVID
kb-tip = Kaardid, missiooni märkmed ja klahvijuhend
kb-geran-file = GERAN-2 (TOIMIK)
kb-objectives = EESMÄRGID
kb-primary = PÕHI: { $objective }
kb-secondary = TEISENE: { $objective }
kb-times = AJAD
kb-deadline = TÄHTAEG { $time }
kb-now = PRAEGU { $time }
kb-conditions = TINGIMUSED
kb-moon = KUU: { $phase } ({ $lit }% VALGUSTATUD)
kb-armament = RELVASTUS
kb-roe = TULE AVAMISE REEGLID
kb-total = KOKKU { $km } KM KIIRUSEGA { $kt } KT
kb-fuel = KÜTUS { $lbs } LBS
kb-preparing = ETTEVALMISTUS...
kb-airborne = ÕHUS
kb-developing = VIIMISTLEN...
kb-no-photo = FOTOT POLE
kb-airborne-target = ÕHUSIHTMÄRK
kb-scroll-vr = KERI: JUHTKANG, SÕRM TAHVLIL
kb-scroll-desktop = KERI: HIIRE RATAS TAHVLI KOHAL

## Missiooniplaneerija

planner-low = MADAL 500 FT
planner-medium = KESK 5000 FT
planner-high = KÕRGE 20000 FT
planner-title = MISSIOONIPLAANER
planner-route = Marsruut
planner-base = BAAS
planner-total = KOKKU { $km } KM  { $time }
planner-briefed = Briifitud marsruut { $km } km, kütus { $fuel } lbs
planner-coverage = Radarikate:
    .note = Radarikate eeldab 480 kt horisontaallendu, relvaluugid suletud.
planner-detected = Marsruudist avastati { $km } km
planner-pulse = Impulssradari (P) avastamiskaugus
planner-doppler = Doppleri radari (D) avastamiskaugus
planner-missile = Raketi lennukaugus
planner-uav = Vaenlase UAV
planner-edit-hint = Lohista punkti selle liigutamiseks, kaarti nihutamiseks. Klõpsa lõigul punkti lisamiseks, paremklõps eemaldab. Ratas: suum.
planner-reset = TÜHISTA
planner-view-hint = Lohista kaarti nihutamiseks. Ratas: suum.

## Salvestatud missioonid

saves-title = SALVESTATUD MISSIOONID
saves-subtitle = Salvestatud lennul menüüst: SALVESTA MISSIOON
saves-none = Salvestatud missioone pole
saves-line = { $theater }  -  { $aircraft }  -  missiooni aeg { $time }
saves-saved-at = Salvestatud { $date }
saves-unreadable = Loetamatu salvestus (teine versioon)
saves-delete =
    .facts = Kustuta see salvestus
saves-back =
    .facts = Tagasi üksikmissiooni ekraanile
