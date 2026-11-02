# Времето в брифингите и на екрана Ops на кампанията (crates/stealth-mission weather.rs).
# Имената на облаците са международните.

wx-cumulus = cumulus
wx-cumulonimbus = cumulonimbus
wx-altostratus = altostratus
wx-stratus = stratus
wx-stratocumulus = stratocumulus
wx-clear = Време: ясно, видимост { $vis } km.
wx-layer = { $name } { $cover }% от { $base } ft
wx-layer-top = { $name } { $cover }% { $base }-{ $top } ft
wx-line = Време: { $sky } - { $layers }, видимост { $vis } km.
wx-sky-clear = ясно
wx-sky-scattered = разкъсана облачност
wx-sky-broken = значителна облачност
wx-sky-overcast = обложено
wx-sky-storm = гръмотевични бури
