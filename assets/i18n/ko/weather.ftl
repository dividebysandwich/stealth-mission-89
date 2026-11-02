# 브리핑과 전역 작전 화면의 기상 정보 (crates/stealth-mission weather.rs). 구름 이름은
# 국제 표준 명칭이다.

wx-cumulus = 적운
wx-cumulonimbus = 적란운
wx-altostratus = 고층운
wx-stratus = 층운
wx-stratocumulus = 층적운
wx-clear = 기상: 맑음, 시정 { $vis } km.
wx-layer = { $name } { $cover }% { $base } ft부터
wx-layer-top = { $name } { $cover }% { $base }-{ $top } ft
wx-line = 기상: { $sky } - { $layers }, 시정 { $vis } km.
wx-sky-clear = 맑음
wx-sky-scattered = 구름 조금
wx-sky-broken = 구름 많음
wx-sky-overcast = 흐림
wx-sky-storm = 뇌우
