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
planner-vprofile = VERTIKAALNE PROFIIL: maastik ja radarituvastus marsruudil
planner-pulse = Impulssradari (P) avastamiskaugus
planner-doppler = Doppleri radari (D) avastamiskaugus
planner-missile = Raketi lennukaugus
planner-uav = Vaenlase UAV
planner-edit-hint = Lohista punkti selle liigutamiseks, kaarti nihutamiseks. Klõpsa lõigul punkti lisamiseks, paremklõps eemaldab. Ratas: suum.
planner-edit-hint-big = Lohistage punkti, et seda liigutada, klõpsake lõigul, et lisada, punktil, et näha selle valikuid
planner-reset = TÜHISTA
planner-add-after = LISA JÄRELE
planner-view-hint = Lohista kaarti nihutamiseks. Ratas: suum.
planner-layers = KIHID
planner-layer-sams = Radar / SAM-positsioon
planner-radar-types = Radaritüübid
planner-layer-pulse = Impulssradari avastamiskaugus
planner-layer-doppler = Doppleri radari avastamiskaugus
planner-layer-airfield = Lennuväli

## Salvestatud missioonid

saves-title = SALVESTATUD MISSIOONID
saves-none = Salvestatud missioone pole
saves-line = { $theater }  -  { $aircraft }  -  missiooni aeg { $time }
saves-saved-at = Salvestatud { $date }
saves-unreadable = Loetamatu salvestus (teine versioon)
saves-delete =
    .facts = Kustuta see salvestus
saves-rename =
    .facts = Nimeta see salvestus ümber
saves-not-renamed = Salvestust ei õnnestunud ümber nimetada: { $error }
saves-back =
    .facts = Tagasi üksikmissiooni ekraanile
saves-note = SALVESTATUD LENNU AJAL: MENÜÜ > SALVESTA MISSIOON
saves-delete-all = KUSTUTA KÕIK
    .facts = Kustuta kõik salvestused
saves-count = Salvestatud missioone: { $count }
