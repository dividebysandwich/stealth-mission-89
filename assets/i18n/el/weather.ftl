# Ο καιρός στα briefing και στην οθόνη επιχειρήσεων της εκστρατείας (crates/stealth-mission
# weather.rs). Τα ονόματα των νεφών είναι τα διεθνή.

wx-cumulus = cumulus
wx-cumulonimbus = cumulonimbus
wx-altostratus = altostratus
wx-stratus = stratus
wx-stratocumulus = stratocumulus
wx-clear = Καιρός: αίθριος, ορατότητα { $vis } km.
wx-layer = { $name } { $cover }% από { $base } ft
wx-layer-top = { $name } { $cover }% { $base }-{ $top } ft
wx-line = Καιρός: { $sky } - { $layers }, ορατότητα { $vis } km.
wx-sky-clear = αίθριος
wx-sky-scattered = αραιή συννεφιά
wx-sky-broken = πυκνή συννεφιά
wx-sky-overcast = κλειστός ουρανός
wx-sky-storm = καταιγίδες
wx-wind-calm = Άνεμος: νηνεμία.
wx-wind = Άνεμος: { $dir }°, { $kt } kt.
wx-wind-gusts = Άνεμος: { $dir }°, { $kt } kt, ριπές { $gust } kt.
