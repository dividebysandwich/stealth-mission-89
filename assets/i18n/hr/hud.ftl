# Poruke igre na HUD-u i 2D zaslonima leta (src/hud, src/combat.rs, src/csar_view.rs,
# src/campaign). HUD ih crta velikim slovima bez dijakritika; riječi avionike (CCIP, THR,
# KM, MAX...) ostaju na engleskom.

hud-no-ejection-seat = NEMA IZBACIVOG SJEDALA
hud-waypoint-reached = Waypoint { $waypoint } dosegnut
hud-select-camera = ODABERITE KAMERU
hud-open-bay = OTVORITE SPREMNIK
hud-camera-page = STRANICA { $page }
hud-target-in-view-shoot = CILJ U VIDU - SNIMITE
hud-target-in-view = CILJ U VIDU
hud-aim-at-target = ZAUZMITE CILJ
hud-photo = FOTO { $state }
hud-rearmed = Naoružan i napunjen gorivom
hud-landed-rearm = Sletjeli ste - naoružajte se ili završite borbeni let
hud-objective-achieved = Cilj postignut

## Zaslon završetka misije

end-accomplished = izvršen
end-not-accomplished = nije izvršen
end-primary = Glavni cilj: { $result }
end-secondary = Sporedni cilj: { $result }
end-kills = Uništeno: zrakoplova { $air }, ciljeva na tlu { $ground }
end-time = Trajanje misije { $time }
end-damage = Oštećenja: pogodaka { $hits }
end-training = Vježbovna misija
end-press-any-key = Pritisnite bilo koju tipku

## Ploča oborenog pilota

csar-flare = SIGNALNA RAKETA
csar-faster = BRŽE
csar-normal = NORMALNO VRIJEME
csar-skip = PRESKOČI IZVLAČENJE
csar-parachute = PADOBRAN OTVOREN
csar-on-ground = NA TLU
csar-down = PALI STE NA NEPRIJATELJSKOM PODRUČJU
csar-pickup = SPAŠAVANJE   UH-60 IZNAD - PREUZIMANJE
csar-inbound = SPAŠAVANJE   UH-60 { $km } KM { $direction } - NA PUTU
csar-fire-flare = SPAŠAVANJE   UH-60 { $km } KM { $direction } - ISPALITE SIGNALNU RAKETU KAD BUDE BLIZU
csar-enemy = NEPRIJATELJ    HIND: { $count }, NAJBLIŽI { $km } KM
csar-flares = SIGNALNE RAKETE   { $count }      NA TLU { $time }

## Obavijesti kooperacije na ploči, napomena konzole protuzračne obrane

notice-joined = { $pilot } se pridružio
notice-left = { $pilot } je otišao
notice-lost-connection = { $pilot } je izgubio vezu
st-select-track = Prvo odaberite trag
