# Mängu teated HUD-il ja lennu 2D-paneelidel (src/hud, src/combat.rs, src/csar_view.rs,
# src/campaign). HUD joonistab need suurtähtedega ja ilma diakriitikuteta; avioonika enda
# sõnad (CCIP, THR, KM, MAX...) jäävad inglise keelde.

hud-no-ejection-seat = KATAPULTISTE PUUDUB
hud-catapult-power = TÄISVÕIMSUS STARDIKS
hud-catapult-wait = OOTAB KATAPULTI
hud-waypoint-reached = Teekonnapunkt { $waypoint } saavutatud
hud-select-camera = VALI KAAMERA
hud-open-bay = AVA RELVALUUK
hud-camera-page = { $page } LEHT
hud-target-in-view-shoot = SIHTMÄRK NÄHTAVAL - PILDISTA
hud-target-in-view = SIHTMÄRK NÄHTAVAL
hud-aim-at-target = SIHI SIHTMÄRGILE
hud-photo = FOTO { $state }
hud-rearmed = Uuesti relvastatud ja tangitud
hud-observing = Vaatlemine - lõpeta missioon menüüst
hud-landed-rearm = Maandutud - relvasta uuesti või lõpeta lend
hud-divert-accept = Ümbersuunamine: { $title } - { $key } nõustumiseks
hud-objective-achieved = Eesmärk täidetud

## Missiooni lõpuekraan

end-accomplished = täidetud
end-not-accomplished = täitmata
end-primary = Põhieesmärk { $result }
end-secondary = Teisene eesmärk { $result }
end-kills = Hävitatud: õhus { $air }, maal { $ground }
end-time = Missiooni aeg { $time }
end-damage = Saadud tabamusi: { $hits }
end-training = Treeningmissioon

## Alla tulistatud piloodi paneel

csar-flare = SIGNAALRAKETT
csar-faster = KIIREMINI
csar-normal = NORMAALAEG
csar-skip = JÄTA EVAKUEERIMINE VAHELE
csar-parachute = VARJUKAS AVATUD
csar-on-ground = MAAL
csar-down = MAHA LASKUNUD VAENLASE TERRITOORIUMIL
csar-pickup = PÄÄSTE   UH-60 PEA KOHAL - PEALEVÕTMINE
csar-inbound = PÄÄSTE   UH-60 { $km } KM { $direction } - LÄHENEB
csar-fire-flare = PÄÄSTE   UH-60 { $km } KM { $direction } - LASE RAKETT, KUI ON LÄHEDAL
csar-enemy = VAENLANE    { $count } HIND, LÄHIM { $km } KM
csar-flares = RAKETID   { $count }      MAAL { $time }

## Koostöö teated paneelil, õhutõrjekonsooli märkus

notice-joined = { $pilot } liitus
notice-left = { $pilot } lahkus
notice-lost-connection = { $pilot } kaotas ühenduse
st-select-track = Vali esmalt jälg
hud-auto-hover = AUTOMAATNE LIUGLEMINE
hud-hover-too-fast = LIIGA KIIRE LIUGLEMISEKS (ÜLE { $knots } KT)
csar-abort = KATKESTA
radio-wing-title = TIIVAMEES
radio-wing-engage = Ründa minu sihtmärki
radio-wing-sams = Ründa SAM-e
radio-wing-cover = Kata mind
radio-wing-rejoin = Liitu tagasi
radio-wing-rtb = Tagasi baasi
radio-wing-trail = Kolonni
radio-wing-spread = Kõrvale
radio-wing-callsign = Kaks
