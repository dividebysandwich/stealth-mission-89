# Pogoda w odprawach i na ekranie operacji kampanii (crates/stealth-mission weather.rs).
# Nazwy chmur są nazwami międzynarodowymi.

wx-cumulus = cumulus
wx-cumulonimbus = cumulonimbus
wx-altostratus = altostratus
wx-stratus = stratus
wx-stratocumulus = stratocumulus
wx-clear = Pogoda: bezchmurnie, widzialność { $vis } km.
wx-layer = { $name } { $cover }% od { $base } ft
wx-layer-top = { $name } { $cover }% { $base }-{ $top } ft
wx-line = Pogoda: { $sky } - { $layers }, widzialność { $vis } km.
wx-sky-clear = bezchmurnie
wx-sky-scattered = zachmurzenie rozproszone
wx-sky-broken = zachmurzenie poprzerywane
wx-sky-overcast = zachmurzenie całkowite
wx-sky-storm = burze
