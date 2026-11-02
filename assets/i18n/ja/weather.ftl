# ブリーフィングと、キャンペーンの作戦画面の天候（crates/stealth-mission
# weather.rs）。雲の名称は国際分類のものです。

wx-cumulus = 積雲
wx-cumulonimbus = 積乱雲
wx-altostratus = 高層雲
wx-stratus = 層雲
wx-stratocumulus = 層積雲
wx-clear = 天候：快晴、視程 { $vis } km。
wx-layer = { $name } { $cover }% 高度 { $base } ft から
wx-layer-top = { $name } { $cover }% 高度 { $base }-{ $top } ft
wx-line = 天候：{ $sky } - { $layers }、視程 { $vis } km。
wx-sky-clear = 快晴
wx-sky-scattered = 散在雲
wx-sky-broken = 雲量多め
wx-sky-overcast = 曇天
wx-sky-storm = 雷雨
