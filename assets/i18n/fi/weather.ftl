# Sää tehtävänannoissa ja kampanjan Ops-näytöllä (crates/stealth-mission weather.rs).
# Pilvien nimet ovat kansainväliset.

wx-cumulus = cumulus
wx-cumulonimbus = cumulonimbus
wx-altostratus = altostratus
wx-stratus = stratus
wx-stratocumulus = stratocumulus
wx-clear = Sää: selkeä, näkyvyys { $vis } km.
wx-layer = { $name } { $cover } % alkaen { $base } ft
wx-layer-top = { $name } { $cover } % { $base }-{ $top } ft
wx-line = Sää: { $sky } - { $layers }, näkyvyys { $vis } km.
wx-sky-clear = selkeä
wx-sky-scattered = hajanaista pilvisyyttä
wx-sky-broken = pilvistä
wx-sky-overcast = pilvinen
wx-sky-storm = ukkosta
