# Zprávy hry na HUD a ve 2D překryvech letu (src/hud, src/combat.rs,
# src/csar_view.rs, src/campaign). HUD je vykresluje velkými písmeny bez diakritiky;
# vlastní výrazy avioniky (CCIP, THR, KM, MAX...) zůstávají anglicky.

hud-no-ejection-seat = ŽÁDNÉ VYSTŘELOVACÍ SEDADLO
hud-waypoint-reached = Waypoint { $waypoint } dosažen
hud-select-camera = VYBERTE KAMERU
hud-open-bay = OTEVŘETE ŠACHTU
hud-camera-page = STRANA { $page }
hud-target-in-view-shoot = CÍL V ZÁBĚRU - FOTO
hud-target-in-view = CÍL V ZÁBĚRU
hud-aim-at-target = NAMIŘTE NA CÍL
hud-photo = FOTO { $state }
hud-rearmed = Přezbrojeno a natankováno
hud-landed-rearm = Přistáno - přezbrojte nebo ukončete bojový let
hud-objective-achieved = Cíl splněn

## Překryv konce mise

end-accomplished = splněn
end-not-accomplished = nesplněn
end-primary = Hlavní cíl { $result }
end-secondary = Vedlejší cíl { $result }
end-kills = Sestřely: { $air } letadel, { $ground } pozemních cílů
end-time = Čas mise { $time }
end-damage = Poškození: { $hits } zásahů
end-training = Cvičná mise
end-press-any-key = Stiskněte libovolnou klávesu

## Panel sestřeleného pilota

csar-flare = SIGNÁLNÍ RAKETA
csar-faster = RYCHLEJI
csar-normal = NORMÁLNÍ ČAS
csar-skip = PŘESKOČIT EVAKUACI
csar-parachute = PADÁK OTEVŘEN
csar-on-ground = NA ZEMI
csar-down = NA ZEMI V NEPŘÁTELSKÉM ÚZEMÍ
csar-pickup = ZÁCHRANA   UH-60 NAD NÁMI - VYZVEDNUTÍ
csar-inbound = ZÁCHRANA   UH-60 { $km } KM { $direction } - LETÍ K VÁM
csar-fire-flare = ZÁCHRANA   UH-60 { $km } KM { $direction } - AŽ BUDE BLÍZKO, VYPALTE SIGNÁLNÍ RAKETU
csar-enemy = NEPŘÍTEL    { $count } HIND, NEJBLIŽŠÍ { $km } KM
csar-flares = SIGNÁLNÍ RAKETY   { $count }      NA ZEMI { $time }

## Oznámení kooperace na panelu (src/player_notices.rs), poznámka konzole protivzdušné obrany

notice-joined = { $pilot } se připojil
notice-left = { $pilot } odešel
notice-lost-connection = { $pilot } ztratil spojení
st-select-track = Nejprve vyberte stopu
