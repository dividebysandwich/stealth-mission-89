# Teachtaireachtaí an chluiche ar an HUD agus ar fholeagáin 2T na heitilte (src/hud, src/combat.rs,
# src/csar_view.rs, src/campaign). Taispeánann an HUD iad i gcaipitil; fanann focail na n-araonaice
# féin (CCIP, THR, KM, MAX...) i mBéarla.

hud-no-ejection-seat = GAN SUÍOCHÁN EJECT
hud-waypoint-reached = Waypoint { $waypoint } bainte amach
hud-select-camera = ROGHNAIGH AN CHAMERA
hud-open-bay = OSCAIL AN BHAIE
hud-camera-page = LEATHANACH { $page }
hud-target-in-view-shoot = SPRIOC I RADHARC - GRIANGHRAF
hud-target-in-view = SPRIOC I RADHARC
hud-aim-at-target = DIRIGH AR AN SPRIOC
hud-photo = GRIANGHRAF { $state }
hud-rearmed = Athairmthe agus athbhreoslaithe
hud-landed-rearm = Tuirlingthe - athairm nó cuir deireadh leis an eitilt
hud-objective-achieved = Cuspóir bainte amach

## Scáileán deiridh na misean

end-accomplished = baineadh amach
end-not-accomplished = níor baineadh amach
end-primary = An príomhchuspóir: { $result }
end-secondary = An cuspóir tánaisteach: { $result }
end-kills = Scriosadh: { $air } aerárthach, { $ground } sprioc talún
end-time = Fad na misean: { $time }
end-damage = Damáiste: { $hits } buille
end-training = Misean traenála
end-press-any-key = Brúigh eochair ar bith

## Painéal an phíolóta a scaoileadh anuas

csar-flare = SOLAS FLARE
csar-faster = NÍOS TAPA
csar-normal = AM GNÁTH
csar-skip = SEACHAIN AN TARRTHÁIL
csar-parachute = PARAISIÚID OSCAILTE
csar-on-ground = AR AN TALAMH
csar-down = ANUAS I DTERITÓIR AN NAMHAD
csar-pickup = TARRTHÁIL   UH-60 THUASA - AG BAINT AMACH
csar-inbound = TARRTHÁIL   UH-60 { $km } KM { $direction } - AG TEACHT
csar-fire-flare = TARRTHÁIL   UH-60 { $km } KM { $direction } - SCAOIL SOLAS NUAIR ATÁ SÉ GAR
csar-enemy = NAMHAD    { $count } HIND, AN CEANN IS GAIRE { $km } KM
csar-flares = FLARES   { $count }      AR AN TALAMH { $time }

## Fógraí chomhoibríoch ar an bpainéal (src/player_notices.rs), nóta chonsól na haerchosanta

notice-joined = Tháinig { $pilot } isteach
notice-left = D'imigh { $pilot }
notice-lost-connection = Cailleadh nasc { $pilot }
st-select-track = Roghnaigh rian ar dtús
