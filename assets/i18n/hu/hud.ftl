# A játék üzenetei a HUD-on és a repülés 2D-s rávetítésein (src/hud, src/combat.rs,
# src/csar_view.rs, src/campaign). A HUD ékezet nélküli nagybetűkkel rajzolja őket; az avionika
# saját szavai (CCIP, THR, KM, MAX...) angolul maradnak.

hud-no-ejection-seat = NINCS KATAPULTÜLÉS
hud-waypoint-reached = Útpont { $waypoint } elérve
hud-select-camera = VÁLASSZ KAMERÁT
hud-open-bay = NYISD KI A REKESZT
hud-camera-page = { $page }. OLDAL
hud-target-in-view-shoot = CÉLPONT LÁTHATÓ - FÉNYKÉP
hud-target-in-view = CÉLPONT LÁTHATÓ
hud-aim-at-target = CÉLOZD A CÉLPONTOT
hud-photo = FÉNYKÉP { $state }
hud-rearmed = Újrafegyverezve és feltankolva
hud-landed-rearm = Leszállt - fegyverezz újra, vagy fejezd be a bevetést
hud-objective-achieved = Cél teljesítve

## A küldetés végi rávetítés

end-accomplished = teljesítve
end-not-accomplished = nem teljesítve
end-primary = Elsődleges cél: { $result }
end-secondary = Másodlagos cél: { $result }
end-kills = Lelövések: { $air } gép, { $ground } földi célpont
end-time = Küldetési idő: { $time }
end-damage = Sérülés: { $hits } találat
end-training = Gyakorló küldetés
end-press-any-key = Nyomj meg egy gombot

## A lelőtt pilóta panelje

csar-flare = JELZŐFÁKLYA
csar-faster = GYORSABBAN
csar-normal = NORMÁL IDŐ
csar-skip = KIMENTÉS KIHAGYÁSA
csar-parachute = EJTŐERNYŐ KINYÍLT
csar-on-ground = A FÖLDÖN
csar-down = AZ ELLENSÉGES TERÜLETEN
csar-pickup = MENTÉS   UH-60 FELETTED - FELVÉTEL
csar-inbound = MENTÉS   UH-60 { $km } KM { $direction } - KÖZELEDIK
csar-fire-flare = MENTÉS   UH-60 { $km } KM { $direction } - LÖJ KI FÁKLYÁT, HA KÖZEL VAN
csar-enemy = ELLENSÉG    { $count } HIND, A LEGKÖZELEBBI { $km } KM
csar-flares = FÁKLYÁK   { $count }      A FÖLDÖN { $time } ÓTA

## Kooperatív értesítések a panelen (src/player_notices.rs), a légvédelmi konzol megjegyzése

notice-joined = { $pilot } csatlakozott
notice-left = { $pilot } kilépett
notice-lost-connection = { $pilot } elvesztette a kapcsolatot
st-select-track = Előbb válassz egy követett célt
