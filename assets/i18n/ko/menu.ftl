# 메인 메뉴와 단일 임무 화면 (src/menu.rs).

## 전역 (stealth_host::theater::THEATERS)

theater-libya = 리비아 (1986)
    .facts =
        시드라 만과 리비아 해안
        엘 도라도 캐니언 작전 시대
        대부분 맑은 사막 하늘
theater-persian_gulf = 페르시아 만 (1984)
    .facts =
        이란-이라크 유조선 전쟁 시대
        바다와 해안 사막 상공
        모든 전역 중 가장 맑은 날씨
theater-north_cape = 노르카프
    .facts =
        북극권 노르웨이와 콜라 반도
        소련 북방함대의 본거지
        낮게 깔린 구름이 잦음
theater-central_europe = 중부 유럽
    .facts =
        독일 상공의 NATO 대 바르샤바 조약기구
        맑은 하늘은 15 %에 불과
theater-vietnam = 베트남
    .facts =
        하노이와 하이퐁 지역
        정글과 산악 지형
        임무의 30 %에서 뇌우
theater-middle_east = 중동
    .facts =
        동부 지중해와 레반트
        대체로 맑은 하늘
theater-desert_storm = 사막의 폭풍
    .facts =
        쿠웨이트와 이라크, 1991년
        F-117A의 실제 첫 실전
        대체로 맑은 사막 하늘
theater-cuba = 쿠바
    .facts =
        카리브 해의 섬 전역
        해상 및 지상 표적
        열대: 임무의 15 %에서 폭풍
theater-korea = 한국
    .facts =
        비무장지대 이북의 한반도
        산악 지형
        변덕스러운 날씨, 어떤 하늘이든 가능
theater-ukraine = 우크라이나 (2026)
    .facts =
        우크라이나 동부, 크림 반도와 러시아 남부
        하르키우에서 헤르손까지 전선이 불타고 있음
        탁 트인 농경지, 전선의 폐허가 된 마을

## 상황 (교전 규칙)

roe-0 = 냉전
    .facts =
        평시의 비밀 작전
        들키지 않는 것이 가장 중요
        초계 중인 적기가 적음
        정확한 교전 규칙은 브리핑에 있음
roe-1 = 제한전
    .facts =
        지역 무력 충돌
        적 방공망 경계 태세
        냉전보다 적기가 많음
roe-2 = 재래식 전쟁
    .facts =
        전면전
        모든 적 전력이 활동 중
        냉전보다 적기가 많음

## 임무 종류

kind-strike = 타격
    .facts =
        지상 표적을 파괴, 촬영 또는 도달
        주 목표와 부 목표
        공대지 무장
kind-air = 공대공
    .facts =
        적기 요격
        수송기, 폭격기 또는 전투기 초계
        공대공 미사일 무장
kind-strike-training = 타격 훈련
    .facts =
        연습용 타격 임무
        무기와 항법 익히기
kind-air-training = 공중전 훈련
    .facts =
        연습용 공대공 임무
        요격과 미사일 익히기
kind-drone-defense = 드론 방어
    .facts =
        우크라이나 전용
        도시나 원자력 발전소를 타격하기 전에 게란 공격 드론 무리를 저지
        이후 발사 기지를 폭격

## 적 수준

opponents-0 = 초보
    .facts =
        미숙한 적 승무원
        가장 쉬운 설정
opponents-1 = 보통
    .facts =
        평균적인 적 승무원
        기본 설정
opponents-2 = 베테랑
    .facts =
        숙련된 적 승무원
        함정과 초계기가 더 많음
opponents-3 = 엘리트
    .facts =
        최고의 적 조종사와 SAM 승무원
        가장 어려운 설정

## 날씨

sky-random = 무작위
    .facts =
        전역의 기후에 따라 결정
        새 임무마다 바뀜
        구름은 양측의 시선을 가림
sky-clear = 맑음
    .facts =
        구름 없음
        시정 40 km
        고전 DOS 시대의 하늘
sky-scattered = 조금 흐림
    .facts =
        적운 20-45 %, 운저 2500-6000 ft
        가끔 높은 곳에 얇은 고적운
        시정 30 km
sky-broken = 구름 많음
    .facts =
        적운 45-70 %, 운저 2000-5000 ft
        위쪽에 층적운이 자주 깔림
        시정 25 km
sky-overcast = 흐림
    .facts =
        층운 80-100 %, 운저 1500-5000 ft
        아래에 적운 조각 몇 개
        시정 20 km
sky-storm = 폭풍
    .facts =
        최대 40000 ft까지 치솟는 적란운
        1000 ft부터 낮은 층운과 적운
        시정 8 km
menu-random-cover = 구름량 무작위
    .facts =
        주 구름층의 양을 날씨에 따라 결정
        직접 정하려면 체크 해제
        주 구름층: 흐림과 폭풍에서는 층운, 그 외에는 적운
menu-cover-slider =
    .facts =
        주 구름층이 하늘을 덮는 비율
        맑은 하늘에서는 효과 없음

## 달

moon-date = 임무 날짜
    .facts =
        브리핑 날짜의 실제 달
        시간대 스위치 필요 (모던)
moon-0 = 삭
    .facts =
        달빛 없음: 칠흑 같은 밤, 불빛과 별뿐
        적 조종사가 2 km에서 나를 봄
moon-1 = 초승달
    .facts = 저녁 하늘의 가는 달, 자정 전에 짐
moon-2 = 상현달
    .facts = 반달, 저녁에 떠 있다가 자정 무렵 짐
moon-3 = 차오르는 볼록달
    .facts = 밝고, 오후부터 자정 넘어서까지 떠 있음
moon-4 = 보름달
    .facts =
        밤새 떠 있고 자정에 가장 높음
        적 조종사가 4 km에서 나를 봄
moon-5 = 기우는 볼록달
    .facts = 밝고, 저녁에 떠서 아침까지 떠 있음
moon-6 = 하현달
    .facts = 반달, 자정 무렵 뜸
moon-7 = 그믐달
    .facts = 새벽 전에 뜨는 가는 달

## 항공기

aircraft-f117 =
    .facts =
        스텔스 공격기
        아음속, 7 g
        모던 및 클래식 조종석
aircraft-f16 =
    .facts =
        다목적 전투기
        마하 2급, 9 g, 빠른 롤
        RWR이 있는 F-16 조종석
aircraft-tornado =
    .facts =
        저고도 타격기
        속도에 따라 가변익이 후퇴
        모던 조종석, MFD 세 개
aircraft-a10 =
    .facts =
        근접 항공 지원기
        느리지만 튼튼함, 30 mm 기관포
        A-10C 조종석, MFD 두 개
aircraft-uh60 =
    .facts =
        다목적 헬리콥터
        병력 침투: 표적 근처에 착륙
        사이클릭과 콜렉티브
aircraft-ah1z =
    .facts =
        공격 헬리콥터
        터릿 기관포, 헬파이어, AIM-9
        기관포와 헬파이어가 포드를 따라감

## 구역과 그룹

section-theater = 전역
section-opponents = 적 수준
section-situation = 상황
section-mission = 임무
section-moon = 달
section-start-time = 시작 시각
section-weather = 날씨
section-realism = 현실성
section-tacview = Tacview
section-mission-summary = 임무 요약
section-briefing = 브리핑
group-mission = 임무
group-conditions = 조건
group-vehicle = 기체
group-simulation = 시뮬레이션

## 임무 옵션

menu-start-random = 무작위
    .facts =
        클래식 모드처럼 새 임무마다 결정
        낮에는 06:00-08:00, 밤에는 18:00-20:00
        시각을 고르려면 체크 해제
menu-start-slider =
    .facts =
        이륙 시각
        밤은 18:00부터 06:00까지
        새 임무를 눌러도 유지됨
menu-realistic-weapons = 현실적인 무장
    .facts =
        실제 항공기의 제한 적용
        무장 베이 4개 대신 2개
        AIM-9, AIM-120 미사일 없음
        항공모함 이착륙 없음
menu-crash-protection = 추락 방지
    .facts =
        지면 근처에서 자동으로 기수를 듦 (기어 올림 상태)
        지면 접촉과 충돌이 치명적이지 않음
        끔: 언제든 지면에 충돌할 수 있음
menu-tacview-export = 내보내기
    .facts =
        각 비행을 Tacview 기록(.acmi)으로 저장
        게임의 파일 폴더 안 tacview 아래
menu-tacview-real-time = 실시간 텔레메트리
    .facts =
        비행 중 Tacview로 비행 정보 전송
        Tacview에서: Record > Real-Time Telemetry
        주소 127.0.0.1, 포트 42674 (--tacview-address)
switch-game = 게임
switch-real = 실제
menu-damage = 피해
    .facts =
        무기 피해
        게임: 클래식 피해. 피격이 쌓여도 항공기가 추락하지 않음
        현실적: 미사일 피격으로 항공기가 파괴될 수 있고(탈출!), 더 많은 계통이 손상되며 화재가 자주 발생해 불이 꺼지거나 기체가 폭발함
        현실적: 손상된 조종면은 항공기의 트림을 어긋나게 하거나 진동하게 함
menu-guidance = 유도
    .facts =
        미사일 유도
        클래식: 미사일이 표적을 향해 직선 비행
        현실적: 충돌점으로의 비례 항법. 모터가 다 타면 미사일은 활공하며 선회할 때마다 속도를 잃음 (연기는 모터가 타는 동안만)
        현실적: SAM이 먼 표적에는 포물선으로 발사. AIM-120은 3초간 직선 비행 (기수를 들어 로브 발사)
menu-air-defence = 방공
    .facts =
        저고도 방공망
        클래식: SAM 사이트와 전투기만
        현실적: 레이더 조준 ZSU-23-4 대공포(그 레이더는 SAM처럼 나를 탐지)와 S-60 대공포 포대가 비행장과 표적을 방어하고, 마을의 MANPADS 팀이 눈에 보이는 것에 SA-14를 발사
        방어 지역 저공 비행이 위험해짐. 폭탄과 기관포로 대공포를 파괴 가능

## 메인 화면

menu-title = STEALTH MISSION 89
menu-subtitle = 임무 계획 시스템
menu-single-mission = 단일 임무
    .facts =
        원하는 전역에서 임무 한 번
        옵션을 고르고 경로와 무장을 계획
menu-career = 경력
    .facts =
        파일럿 기록: 계급, 훈장, 출격
        복무 기간: 한 전역에서 12회 출격
        파괴한 것은 계속 파괴된 상태로 남음
menu-campaign = 다이내믹 캠페인
    .facts =
        실시간 다이내믹 캠페인
        적의 도시와 비행장을 점령
        실시간 상황에 따른 임무
menu-multiplayer = 멀티플레이어
    .facts = 다이내믹 캠페인에서 협동
    .join = 공개 목록의 서버에 참가
    .host = 서버에 참가하거나 캠페인을 호스팅하여 비행
menu-options = 옵션
    .facts =
        게임: 조종석, HUD, 항전 스타일, 자동 트림
        그래픽, 볼륨, 키와 조이스틱 설정
        다음 비행을 위해 저장됨
    .short = 게임, 그래픽, 사운드, 조작
menu-download = 다운로드
    .facts =
        Windows, Linux, macOS용 데스크톱 게임
        VR과 멀티플레이어 지원
        Steam Deck과 Steam Frame: Linux AppImage
web-play = 브라우저에서 플레이
    .facts = 바로 여기서, 설치 필요 없음
web-download-platforms = Windows, Linux, macOS, Steam Deck, Steam Frame
web-download-version = 다운로드 버전
web-gives-vr = VR
    .why = OpenXR 호환 헤드셋과 Steam Frame에서 VR로 비행
web-gives-hotas = HOTAS
    .why = 게임패드, 조이스틱, 스로틀, 페달 완전 지원
web-gives-multiplayer = 더 나은 멀티플레이어
    .why = 서버에 직접 UDP 연결로 지연 감소
web-gives-server = 나만의 서버 호스팅
    .why = 협동 캠페인을 호스팅하거나 전용 서버 운영
web-gives-head-tracking = 헤드 트래킹
    .why = OpenTrack을 통한 TrackIR 방식 트래커
web-gives-tacview = Tacview
    .why = 비행 기록과 실시간 텔레메트리
web-gives-keep = 영구 보관
    .why = 웹사이트가 닫혀도 내 사본이 남아 있음
menu-discord = DISCORD
    .facts =
        함께 비행하고, 버그를 제보하고, 임무를 공유
        개발 소식 팔로우
menu-quit = 종료
    .facts = 바탕 화면으로 나가기
menu-version = 버전 { $version }

## 단일 임무 화면

menu-career-sortie = 경력 출격
menu-tour-sortie = { $pilot }  //  복무 출격 { $sortie } / { $length }
menu-msn = 임무 계획 시스템  //  MSN { $seed }
menu-no-targets = 이번 복무 기간에 남은 표적이 없습니다
menu-plan-route = 경로 계획
    .facts =
        적 레이더와 SAM 탐지 범위가 표시된 전역 지도
        항법 지점 추가, 이동, 제거
        표적과 착륙 기지는 고정
menu-loadout = 무장
    .facts =
        각 베이나 파일런의 무장 선택
        브리핑된 무기로 시작
menu-generating = 생성 중...
menu-back-records = 파일럿 기록으로 돌아가기
menu-back-main = 메인 메뉴로 돌아가기
menu-new-sortie = 새 출격
menu-new-mission = 새 임무
    .facts =
        다른 목표를 새로 정함
        모든 옵션과 시작 시각 유지
menu-load = 불러오기
    .facts =
        비행 중 저장한 임무 (메뉴: 임무 저장)
        저장한 지점부터 이어서 비행
menu-fly = 비행
    .facts = 브리핑된 임무로 이륙
menu-fly-vr = VR로 비행
    .facts =
        VR 헤드셋에서 게임을 다시 시작
        바로 이 임무로 비행
    .no-mission = 먼저 임무를 생성하세요
    .no-runtime = 활성화된 OpenXR 런타임이 없습니다 (Monado, WiVRn, SteamVR)
menu-vr-failed = VR로 시작할 수 없습니다: { $error }

## 임무 요약

summary-theater = 전역
summary-situation = 상황
summary-mission = 임무
summary-opponents = 적 수준
summary-time = 시각
summary-weather = 날씨
summary-fuel = 연료
summary-night = 밤
summary-day = 낮
summary-time-value = { $daytime }  출발 { $depart }
summary-weather-value = { $sky }  시정 { $vis } km
summary-fuel-value = { $fuel } lbs
summary-armament = 무장: { $stores }
summary-intelligence = 정보
