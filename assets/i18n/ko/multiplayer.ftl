# 캠페인 창 (src/campaign/menu.rs), 멀티플레이어 창 (src/multiplayer_menu.rs),
# 전황도의 범례와 실시간 화면 (src/campaign/picture.rs, unit_view.rs).

delete = 삭제
delete-campaign = 이 캠페인을 삭제하시겠습니까?
delete-save = 이 저장 파일을 삭제하시겠습니까?
delete-permanent = 삭제하면 복구할 수 없습니다

## 캠페인 창

cm-group-campaign = 캠페인
cm-weather-dynamic = 동적
    .facts =
        기상은 하늘 시계로 10-22시간마다 바뀝니다
        구름은 양측의 시선을 모두 가립니다
cm-weather-clear = 맑은 하늘
    .facts = 캠페인 내내 구름 없음
cm-section-day-night = 낮과 밤
cm-day-real = 실시간
    .facts = 하루는 24시간입니다
cm-day-2h = 하루 2시간
    .facts = 하루는 2시간입니다
cm-day-1h = 하루 1시간
    .facts = 하루는 1시간입니다
cm-day-30min = 하루 30분
    .facts = 하루는 30분입니다
cm-conditions-note = 조건은 새 캠페인에 적용됩니다
cm-rules-note = 저장된 캠페인은 시작할 때의 규칙을 유지합니다
cm-about =
    전쟁은 플레이하는 동안 실시간으로 진행되며, 메뉴를 열어 둔 동안에도 계속됩니다.
    게시판의 임무를 수행하십시오: 정찰, SEAD, CAP, CAS, 대함, 타격, 호위.
    헬기 침투 작전으로 적의 도시와 비행장을 점령합니다.
    조종사가 전사하거나 포로가 되는 일은 없으며, 출격 후에는 게시판으로 돌아옵니다.
cm-section-new = 새 캠페인
cm-new-line = { $theater }  -  { $situation }  -  적 { $opponents }
cm-start = 시작
cm-section-continue = 계속하기
cm-no-saves = 저장된 캠페인 없음
cm-played = { $hours }시간 플레이
cm-save-gone = 저장 파일이 없습니다
cm-save-unreadable = 저장 파일을 읽을 수 없음: { $error }
cm-save-undeletable = 저장 파일을 삭제할 수 없음: { $error }
cm-in-vr = VR
cm-delete =
    .facts = 이 저장된 캠페인 삭제

## 전황도

pic-friendly-aircraft = 아군 항공기
pic-friendly-helicopter = 아군 헬기
pic-enemy-fighter = 적 전투기 (레이더 포착)
pic-sam-known = SAM 기지 (확인됨)
pic-guns-known = 대공포 / MANPADS (확인됨)
pic-armour = 전차 / 보병전투차 (적: 시야 내)
pic-patriot-reach = 패트리엇 사거리
pic-gepard-stinger = 게파르트 / 스팅어
pic-missile-range = SAM 미사일 사거리
pic-engagement-range = 교전 거리
pic-reconned = 정찰 완료
pic-you = 자신
pic-site = { $place } 기지
uv-live = { $unit } - 실시간
uv-contact-lost = 접촉 상실
uv-hint = 끌기: 회전  휠: 확대/축소

## 멀티플레이어

mp-disconnected = 연결 끊김: { $why }
mp-connecting = { $address }에 연결하는 중...
mp-joining = { $address }에 연결됨, 참가하는 중...
mp-browser-public-only = 브라우저에서는 공개 목록의 서버에만 참가할 수 있습니다
mp-title-host = 멀티플레이어: 게임 호스트
mp-subtitle = 동적 캠페인 협동 플레이: 함께 전쟁을 치르십시오
mp-your-name = 이름
    .hint = 조종사 이름
mp-join-address = 주소로 참가
    .hint = 주소 또는 이름[:포트]
mp-connect = 연결
mp-server-name = 서버 이름
mp-udp-port = UDP 포트
mp-address-note = 다른 플레이어는 이 컴퓨터의 주소로 참가합니다. 인터넷으로 플레이하려면 공유기에서 UDP 포트를 여십시오
mp-list-public = 공개 서버 목록에 등록 (다른 플레이어가 서버 목록에서 이 게임을 찾을 수 있음)
mp-rules = 모든 참가자에게 적용되는 규칙
mp-time-of-day = 시간대 (달빛 밤)
mp-realistic-weapons = 사실적인 F-117A 무장
mp-friendly-fire = 아군 피해
mp-day-night = 낮과 밤:
mp-weather = 기상:
mp-host-new = 새로 호스트
mp-host-saved = 저장된 캠페인 호스트
mp-host = 호스트
mp-host-game = 게임 호스트
mp-port-error = UDP 포트 { $port }: { $error }
mp-default-server = { $pilot }: 캠페인
mp-public-servers = 공개 서버
mp-refresh = 새로고침
mp-server-count = 서버 { $servers }개, 조종사 { $pilots }명
mp-list-error = 서버 목록에 접속할 수 없음: { $error }
mp-no-servers = 현재 공개 서버 없음
mp-col-server = 서버
mp-col-pilots = 조종사
mp-col-war = 전쟁
mp-dedicated = 전용
mp-other-version = 다른 버전
mp-join = 참가
