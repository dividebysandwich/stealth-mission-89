# 简报和战役作战界面中的天气（crates/stealth-mission weather.rs）。云名采用国际通用名称。

wx-cumulus = 积云
wx-cumulonimbus = 积雨云
wx-altostratus = 高层云
wx-stratus = 层云
wx-stratocumulus = 层积云
wx-clear = 天气：晴，能见度 { $vis } km。
wx-layer = { $name } { $cover }%，云底 { $base } ft
wx-layer-top = { $name } { $cover }%，{ $base }-{ $top } ft
wx-line = 天气：{ $sky } - { $layers }，能见度 { $vis } km。
wx-sky-clear = 晴
wx-sky-scattered = 疏云
wx-sky-broken = 多云
wx-sky-overcast = 阴
wx-sky-storm = 雷暴
