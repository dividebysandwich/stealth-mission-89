# Spēles ziņojumi uz HUD un lidojuma 2D pārklājumos (src/hud, src/combat.rs,
# src/csar_view.rs, src/campaign). HUD tos zīmē lielajiem burtiem bez garumzīmēm; avionikas
# vārdi (CCIP, THR, KM, MAX...) paliek angļu valodā.

hud-no-ejection-seat = NAV KATAPULTĒŠANĀS SĒDEKĻA
hud-waypoint-reached = Waypoint { $waypoint } sasniegts
hud-select-camera = IZVĒLIETIES KAMERU
hud-open-bay = ATVĒRT NODALĪJUMU
hud-camera-page = { $page } LAPA
hud-target-in-view-shoot = MĒRĶIS REDZAMS - FOTO
hud-target-in-view = MĒRĶIS REDZAMS
hud-aim-at-target = TĒMĒT UZ MĒRĶI
hud-photo = FOTO { $state }
hud-rearmed = Bruņojums un degviela papildināta
hud-landed-rearm = Nosēdies - papildiniet bruņojumu vai beidziet lidojumu
hud-objective-achieved = Uzdevums izpildīts

## Misijas beigu pārklājums

end-accomplished = izpildīts
end-not-accomplished = neizpildīts
end-primary = Galvenais uzdevums: { $result }
end-secondary = Papilduzdevums: { $result }
end-kills = Uzvaras: { $air } lidmašīnas, { $ground } zemes mērķi
end-time = Misijas ilgums { $time }
end-damage = Bojājumi: { $hits } trāpījumi
end-training = Treniņa misija
end-press-any-key = Nospiediet jebkuru taustiņu

## Notriektā pilota panelis

csar-flare = SIGNĀLRAĶETE
csar-faster = ĀTRĀK
csar-normal = NORMĀLS LAIKS
csar-skip = PĀRLĒKT EVAKUĀCIJU
csar-parachute = IZPLETNIS ATVĒRTS
csar-on-ground = UZ ZEMES
csar-down = NOTRIEKTS IENAIDNIEKA TERITORIJĀ
csar-pickup = GLĀBŠANA   UH-60 VIRS GALVAS - PAŅEMŠANA
csar-inbound = GLĀBŠANA   UH-60 { $km } KM { $direction } - TUVOJAS
csar-fire-flare = GLĀBŠANA   UH-60 { $km } KM { $direction } - ŠAUJIET SIGNĀLRAĶETI, KAD TUVU
csar-enemy = IENAIDNIEKS    { $count } HIND, TUVĀKAIS { $km } KM
csar-flares = SIGNĀLRAĶETES   { $count }      UZ ZEMES { $time }

## Kopīgās spēles paziņojumi panelī, pretgaisa aizsardzības konsoles piezīme

notice-joined = Pievienojās: { $pilot }
notice-left = Aizgāja: { $pilot }
notice-lost-connection = Savienojums zaudēts: { $pilot }
st-select-track = Vispirms izvēlieties mērķi
