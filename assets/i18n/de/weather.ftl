# Das Wetter in den Briefings und auf dem Einsatzbildschirm der Kampagne (crates/stealth-mission
# weather.rs). Die Wolkennamen sind die internationalen.

wx-cumulus = Cumulus
wx-cumulonimbus = Cumulonimbus
wx-altostratus = Altostratus
wx-stratus = Stratus
wx-stratocumulus = Stratocumulus
wx-clear = Wetter: klar, Sicht { $vis } km.
wx-layer = { $name } { $cover } % ab { $base } ft
wx-layer-top = { $name } { $cover } % { $base }-{ $top } ft
wx-line = Wetter: { $sky } - { $layers }, Sicht { $vis } km.
wx-sky-clear = klar
wx-sky-scattered = aufgelockert
wx-sky-broken = durchbrochen
wx-sky-overcast = bedeckt
wx-sky-storm = Gewitter
