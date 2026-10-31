# Sporočila igre na HUD-u in 2D prekrivnih elementih leta (src/hud, src/combat.rs,
# src/csar_view.rs, src/campaign). HUD jih riše z velikimi črkami brez strešic; besede
# avionike (CCIP, THR, KM, MAX...) ostanejo v angleščini.

hud-no-ejection-seat = BREZ IZSTRELNEGA SEDEŽA
hud-waypoint-reached = Waypoint { $waypoint } dosežen
hud-select-camera = IZBERITE KAMERO
hud-open-bay = ODPRITE BOMBNIK
hud-camera-page = STRAN { $page }
hud-target-in-view-shoot = CILJ V VIDEU - FOTOGRAFIRAJTE
hud-target-in-view = CILJ V VIDEU
hud-aim-at-target = NAMERITE NA CILJ
hud-photo = FOTO { $state }
hud-rearmed = Ponovno oboroženo in natočeno gorivo
hud-landed-rearm = Pristali - ponovno oborožite ali končajte bojni let
hud-objective-achieved = Cilj dosežen

## Prekrivni zaslon ob koncu misije

end-accomplished = izpolnjen
end-not-accomplished = ni izpolnjen
end-primary = Glavni cilj { $result }
end-secondary = Drugotni cilj { $result }
end-kills = Zadetki: { $air } letal, { $ground } kopenskih ciljev
end-time = Čas misije { $time }
end-damage = Škoda: { $hits } zadetkov
end-training = Vadbena misija
end-press-any-key = Pritisnite poljubno tipko

## Plošča sestreljenega pilota

csar-flare = SIGNALNA RAKETA
csar-faster = HITREJE
csar-normal = NORMALEN ČAS
csar-skip = PRESKOČI EVAKUACIJO
csar-parachute = PADALO ODPRTO
csar-on-ground = NA TLEH
csar-down = SESTRELJENI NA SOVRAŽNEM OZEMLJU
csar-pickup = REŠEVANJE   UH-60 NAD VAMI - PODVZEM
csar-inbound = REŠEVANJE   UH-60 { $km } KM { $direction } - PRIBLIŽUJE SE
csar-fire-flare = REŠEVANJE   UH-60 { $km } KM { $direction } - IZSTRELITE SIGNALNO RAKETO, KO BO BLIZU
csar-enemy = SOVRAŽNIK   { $count } HIND, NAJBLIŽJI { $km } KM
csar-flares = SIGNALNE RAKETE   { $count }      NA TLEH { $time }

## Obvestila sodelovanja na plošči, opomba konzole protizračne obrambe

notice-joined = { $pilot } se je pridružil
notice-left = { $pilot } je odšel
notice-lost-connection = { $pilot } je izgubil povezavo
st-select-track = Najprej izberite sled
