# 캠페인의 작전 화면 (src/campaign/ops.rs).

ops-thunderstorms = 뇌우
ops-wx-short = 기상 { $now }, { $at }에 { $next }
ops-wx-line = { $weather } { $at }에 { $next }(으)로 바뀝니다.
ops-clock = { $day }일차  { $time }
ops-landed-at = { $base } 착륙
ops-ground-crew = 지상 정비팀 준비 완료
ops-rearm = 재무장 및 재급유
ops-end-sortie = 출격 종료
ops-on-server = 서버: { $server }
ops-serving = 서비스 중
ops-tab-missions = 임무
ops-tab-map = 전황도
ops-tab-air-defence = 방공
ops-tab-forces = 전력
ops-tab-players = 플레이어
ops-tab-debrief = 디브리핑
ops-tab-log = 기록
ops-tab-chat = 채팅
ops-blue = 청군 { $count }
ops-red = 적군 { $count }
ops-war-over = 전쟁이 끝났습니다
ops-running = 전쟁 시뮬레이션 진행 중
ops-save-exit = 저장 후 종료
ops-disconnect = 연결 끊기
ops-leave-local = 저장하고 캠페인을 떠나시겠습니까?
    .text = 계속하기 전까지 전쟁이 멈춥니다. 비행 중인 항공기는 상실되며, 진행 중인 침투 작전은 기지에서 다시 시작됩니다.
ops-leave-host = 저장하고 캠페인 서비스를 중단하시겠습니까?
    .text = 참가한 플레이어의 연결이 끊기고 계속하기 전까지 전쟁이 멈춥니다. 비행 중인 항공기는 상실되며, 진행 중인 침투 작전은 기지에서 다시 시작됩니다.
ops-leave-client = 서버와의 연결을 끊으시겠습니까?
    .text = 캠페인은 서버에서 계속 진행되며 이 컴퓨터에는 아무것도 저장되지 않습니다. 출격이 종료됩니다.
ops-unit-destroyed = { $unit } 파괴: 작전 화면으로 복귀

## 임무

ops-section-missions = 임무
ops-threat = 위협도 { $threat }
ops-no-missions = 임무 없음: 전선이 조용합니다
ops-section-underway = 침투 및 습격
ops-enemy-liftoff = 적 { $place }: { $mins }분 후 이륙
ops-liftoff = { $place }: { $mins }분 후 이륙
ops-enemy-helis = 적 { $place }: 헬기 { $count }대 접근 중
ops-helis = { $place }: 헬기 { $count }대 이동 중
ops-raid-sead = AI SEAD { $place }
ops-raid-drones = 드론 { $place }
ops-raid = { $place } 비행장 습격
ops-raid-launch = { $raid }: { $mins }분 후 출격
ops-raid-on-way = { $raid }: { $count }기 이동 중
ops-resupply-ai = { $unit } 보급: UH-60 이동 중
ops-awacs = 적 AWACS 앞으로 { $mins }분간 비행
ops-down = 추락: { $pilot } - { $mins }분 이내 구조
ops-aboard = 탑승: { $pilot } - 귀환 중
ops-none-underway = 진행 중인 작전 없음
ops-ai-helicopter = UH-60 조종사가 맡지 않으면 { $time } 후 AI 헬기가 수행합니다
ops-defences = 방어: SAM 기지 { $sams }곳, 위협도 { $threat }
ops-recon-intel = 정찰 정보
ops-order-insertion = 침투 명령
    .facts = 호위기로 엄호하거나 UH-60으로 직접 인솔하십시오
ops-from = 출발
ops-planned-by-group = 그룹이 계획함
ops-stores = { $aircraft } - 무장: { $stores }
ops-fly-uh60 = UH-60 조종
ops-accept = 임무 수락
ops-planning = 계획 중: { $pilots }
ops-flying = 비행 중: { $pilots }
ops-package = { $kinds } 패키지: { $mins }분 이내에 이 중 두 종류를 달성하면 각각 +{ $bonus }점
ops-sead-on-way = AI SEAD 이동 중
ops-request-sead = AI SEAD 요청 (물자 { $cost }, 항공기 { $aircraft }기)
    .facts = 아군 공격기가 선두에서 이 지역의 SAM 기지를 공격합니다
    .disabled = 물자 또는 지상 대기 항공기가 부족합니다 (전력 탭 참조)

## 규칙, 물자, 전력

ops-rule-damage = 사실적인 피해
ops-rule-guidance = 사실적인 유도
ops-rule-no-crashes = 추락 없음
ops-rule-time = 연속 시간
ops-rule-weapons = 사실적인 F-117A 무장
ops-rule-friendly-fire = 아군 피해
ops-rule-classic = 클래식
ops-materiel = 물자
ops-materiel-blue = 청군 { $materiel } (+{ $income }/h)
ops-materiel-red = 적군 { $materiel } (+{ $income }/h)
ops-friendly = 아군
ops-enemy = 적
ops-section-forces = 전력
ops-places-held = 확보한 지역
ops-materiel-row = 물자
ops-income = 시간당 수입
ops-income-from = 산출
ops-income-sources = { $base } + 지역 { $places } x { $per_place } + 도시 { $cities } x { $per_city }
ops-fighters-ground = 지상의 전투기
ops-helicopters-ground = 지상의 헬기
ops-in-air = 공중
ops-in-air-value = 전투기 { $fighters }, 공격기 { $strike }, 헬기 { $helicopters }
ops-since-start = 시작 이후
ops-earned = 획득
ops-repairs = 수리
ops-operations = 출격, 습격, 침투
ops-new-aircraft = 신규 항공기
ops-lost = 손실 (도시, 조종사)
ops-section-how = 진행 방식
ops-how =
    각 진영은 매시간 물자를 얻습니다. 기본량에 더해 확보한 지역마다, 그리고 그중 온전한 도시마다 늘어납니다. 적의 도시를 폭격(INTERDICTION)하면 적의 수입이 줄어듭니다.
    AI는 수리, 전투기 출격, 습격, 자체 침투 등 모든 행동에 물자를 소모합니다. 물자가 부족하면 피해가 복구되지 않고 항공기도 출격하지 않습니다.
    AI는 보유한 항공기만 운용합니다. 격추되면 사라지고, 착륙하면 다시 출격합니다. 매시간 물자가 남아 있는 한 각 진영이 새 항공기를 구입합니다.
    플레이어는 물자가 부족해지지 않습니다. 자신의 항공기, 기지(폭격 여부와 무관), 직접 내린 침투 명령의 헬기는 항상 사용할 수 있습니다.
ops-section-prices = 가격
ops-price-airfield = 비행장 수리
ops-price-sam = SAM 기지 수리
ops-price-city = 도시 재건
ops-price-ship = 함선 수리
ops-price-gun = 대공포 또는 MANPADS 팀
ops-price-sortie = 전투기 출격
ops-price-raid = 습격
ops-price-insertion = AI 침투
ops-price-fighter = 신규 전투기
ops-price-helicopter = 신규 헬기
ops-price-city-lost = 도시 파괴 (손실)
ops-price-captured = 조종사 포로 (손실)

## 전황도

ops-click-to-watch = { $unit } - 클릭하여 관전
ops-section-legend = 범례
ops-legend-friendly-airfield = 아군 비행장
ops-legend-enemy-airfield = 적 비행장
ops-legend-city = 아군 / 적 도시
ops-legend-airfield = 아군 / 적 비행장
ops-legend-insertion = 투입 경로
ops-watch-hint = 부대를 클릭하면 실시간으로 관전합니다
ops-map-hint = 휠: 확대/축소, 끌기: 이동
ops-patriot = 패트리엇 포대
ops-drone-center = 드론 지휘소
ops-manned-by = 담당: { $pilot }
ops-manned-by-other = 다른 플레이어가 담당 중
ops-control = 제어
    .facts = 콘솔을 인수합니다

## 디브리핑, 종료, 기록, 채팅

ops-no-sortie = 아직 출격 기록 없음
ops-section-sortie = 출격
ops-achieved = 목표 달성
ops-not-achieved = 목표 미달성
ops-waiting-raid = 습격 대기 중
ops-waiting-insertion = 침투 대기 중
ops-flight-time = 비행 시간 { $mins }분
ops-kills = 격추: 항공기 { $air }대, 지상 { $ground }개
ops-points = { $points }점
ops-points-bonus = { $points }점 (패키지 보너스 +{ $bonus })
ops-campaign-points = { $rank } - 캠페인 점수 { $points }점
ops-campaign-record = 캠페인: 출격 { $sorties }회, 목표 달성 { $won }회
ops-section-meanwhile = 그동안
ops-victory = 승리
    .text = 적에게 남은 비행장이 없습니다. 전역은 우리 것입니다.
ops-defeat = 패배
    .text = 아군 비행장이 남아 있지 않습니다. 전역을 상실했습니다.
ops-end-day = { $day }일차. 아군 지역 { $friendly }, 적 지역 { $enemy }.
ops-next-campaign = 다음 캠페인 시작까지 { $time }
ops-next-campaign-now = 다음 캠페인이 시작됩니다
ops-next-from-menu = 다음 캠페인은 메인 메뉴에서 시작하십시오.
ops-section-log = 캠페인 기록
ops-section-chat = 채팅
ops-no-messages = 아직 메시지 없음
ops-message = 메시지
    .hint = 메시지를 입력하고 Enter로 전송
ops-send = 전송

## 브리핑

ops-briefing = 임무 브리핑
ops-briefing-line = 출발 { $base } - 표적까지 { $km } km - { $aircraft } - 무장: { $stores }
ops-route-locked = 경로 고정: { $pilots } 비행 중
ops-route-threats = 경로 및 위협
ops-take-off = 이륙
ops-radars-shown = 표시된 레이더: 아군 항공기가 탐지했고 아직 건재한 것

## 플레이어

ops-section-players = 플레이어
ops-col-name = 이름
ops-col-status = 상태
ops-col-sorties = 출격
ops-col-points = 점수
ops-col-pilot = 조종사
ops-col-objectives = 목표
ops-col-kills = 격추 공중/지상
ops-col-medal = 최고 훈장
ops-you = (자신)
ops-host = 호스트
ops-status-planning = { $title } 계획 중
ops-status-board = 임무 게시판
ops-status-flying = { $title } 비행 중 - { $aircraft }
ops-status-reconnecting = 연결 끊김 - 항공기 { $seconds }초간 유지
ops-flown = { $flown } (달성 { $won })
ops-section-leaderboard = 순위표
ops-alone = 단독 비행 중입니다. 함께 비행하려면 메인 메뉴의 멀티플레이어에서 이 캠페인을 호스트하십시오.
ops-rules = 이 캠페인의 규칙 (시작 시점 유지): { $rules }
ops-no-sorties = 아직 출격 기록 없음
