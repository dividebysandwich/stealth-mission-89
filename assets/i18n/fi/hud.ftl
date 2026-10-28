# Pelin viestit HUD:lla ja lennon 2D-peittokuvissa (src/hud, src/combat.rs,
# src/csar_view.rs, src/campaign). HUD piirtää ne isoin kirjaimin ilman aksentteja;
# avioniikan omat sanat (CCIP, THR, KM, MAX...) pysyvät englanniksi.

hud-no-ejection-seat = EI KATAPULTTI-ISTUINTA
hud-waypoint-reached = Reittipiste { $waypoint } saavutettu
hud-select-camera = VALITSE KAMERA
hud-open-bay = AVAA RUUMA
hud-camera-page = SIVU { $page }
hud-target-in-view-shoot = KOHDE NÄKYVISSÄ - KUVAA
hud-target-in-view = KOHDE NÄKYVISSÄ
hud-aim-at-target = TÄHTÄÄ KOHTEESEEN
hud-photo = KUVA { $state }
hud-rearmed = Aseistettu ja tankattu
hud-landed-rearm = Laskeuduttu - aseista tai päätä lento
hud-objective-achieved = Tavoite saavutettu

## Tehtävän lopetusnäyttö

end-accomplished = suoritettu
end-not-accomplished = ei suoritettu
end-primary = Päätavoite { $result }
end-secondary = Sivutavoite { $result }
end-kills = Tuhotut: koneita { $air }, maakohteita { $ground }
end-time = Tehtävän kesto { $time }
end-damage = Vauriot: osumia { $hits }
end-training = Harjoitustehtävä
end-press-any-key = Paina mitä tahansa näppäintä

## Alasammutun lentäjän paneeli

csar-flare = MERKKITULI
csar-faster = NOPEAMMIN
csar-normal = NORMAALI AIKA
csar-skip = OHITA EVAKUOINTI
csar-parachute = LASKUVARJO AUKI
csar-on-ground = MAASSA
csar-down = ALHAALLA VIHOLLISEN ALUEELLA
csar-pickup = PELASTUS   UH-60 YLLÄ - NOUTO
csar-inbound = PELASTUS   UH-60 { $km } KM { $direction } - LÄHESTYY
csar-fire-flare = PELASTUS   UH-60 { $km } KM { $direction } - AMMU MERKKITULI KUN LÄHELLÄ
csar-enemy = VIHOLLINEN    HINDEJÄ { $count }, LÄHIN { $km } KM
csar-flares = MERKKITULET   { $count }      MAASSA { $time }

## Yhteistyön ilmoitukset paneelilla, ilmatorjuntakonsolin huomautus

notice-joined = { $pilot } liittyi
notice-left = { $pilot } poistui
notice-lost-connection = { $pilot } menetti yhteyden
st-select-track = Valitse ensin kohde
