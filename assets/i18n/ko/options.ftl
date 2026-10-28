# 옵션 창 (src/options.rs)과 비행 중 메뉴의 해당 페이지.

options-title = 옵션
options-tab-game = 게임
options-tab-graphics = 그래픽
options-tab-sound = 사운드
options-tab-controls = 조작
options-footer = 다음 비행을 위해 저장됩니다. 현실성 스위치는 게임마다 선택합니다

## 게임

options-section-language = 언어
options-section-classic-modern = 클래식 / 모던
options-cockpit = 조종석
    .facts =
        모던: 베젤 버튼이 있는 고해상도 디스플레이
        클래식: DOS 시대의 계기판 배치와 77x60 디스플레이 (F-117A)
options-hud = HUD
    .facts =
        모던: NAV, AA, AG 모드를 갖춘 녹색 고해상도 F-16 스타일 HUD
        클래식: DOS 시대의 HUD
options-avionics = 항전
    .facts =
        모던: 두 MFD 어느 쪽에서나 F-16 스타일 MFD 포맷(상황 표시, HARM, 타기팅 포드, 지도, 무장)과 이동 가능한 커서
        클래식: 왼쪽에 지도와 TAC, 오른쪽에 카메라, WAY, WPN, ORD
options-flight-model = 비행 모델
    .facts =
        모던: 플라이 바이 와이어 방식의 공기역학 비행 모델. 스틱은 피치와 롤 속도를 지시하며, 놓으면 기수가 그대로 유지되고 비행 경로가 지켜집니다
        클래식: DOS 시대의 비행 모델
options-section-flying = 비행
options-auto-trim = 자동 트림
    .facts =
        스틱을 놓으면 트림이 놓은 자세를 유지합니다
        헬리콥터: 속도에 따라 기수가 올라가지 않음. 제트기: 손상된 조종면을 트림으로 보정
        수동 트림: Alt + 방향키, Alt + Home으로 중앙 복귀
options-tooltips = 조종석 툴팁
    .facts = 마우스 아래 또는 VR 손끝에 있는 조종석 조작부나 디스플레이가 무엇인지 표시합니다
options-section-vr-size = VR 조종석 크기
options-vr-size =
    .facts =
        헤드셋 속 조종석(과 세계)의 크기
        100%: 실물 크기

## 그래픽

options-section-rendering = 렌더링
options-shaders = 셰이더
    .facts =
        모던: 은은한 방향성 음영, 부드러운 원거리 안개
        클래식: 평면 음영
options-shadows = 그림자
    .facts =
        모던: 조종석과 세계에 태양 그림자 (GPU 부하 증가)
        클래식: 그림자 없음
options-point-lights = 점광원
    .facts =
        모던: 조종석 조명과 디스플레이 조명. 폭발, 연기 기둥, 활주로 조명이 장면을 밝힘
        클래식: 태양과 하늘 빛만
options-time-of-day = 시간대
    .facts =
        모던: 태양과 달이 임무 시계에 따라 움직이고, 하늘이 태양에 맞춰 어두워지고 밝아집니다. 밤에는 별과 달의 위상 표시
        모던: 달빛이 밤에 적 조종사가 나를 볼 수 있는 거리를 정합니다 (달 없는 밤 2 km, 보름달 아래 4 km)
        클래식: 천천히 바뀌는 색과 고정된 태양
options-msaa = 안티앨리어싱 (MSAA)
    .facts =
        가장자리를 부드럽게: 4x MSAA, VR에서는 2x (GPU 부하 증가)
        끔: 거친 픽셀 가장자리
options-auto-detail = 지형 디테일 자동 (VR)
    .facts =
        헤드셋에서 프레임이 늦어지면 마을, 숲, 들판의 표시 거리가 줄고 여유가 생기면 다시 늘어납니다
        그래픽 설정은 선택한 그대로 유지됩니다
options-sun-moon = 태양과 달
    .facts =
        낮에는 하늘의 태양, 밤에는 달과 별
        달은 임무의 위상을 보여줍니다

## 사운드

options-section-volume = 볼륨
options-volume-master = 마스터
    .facts = 모든 소리
options-volume-engine = 엔진
    .facts = 내 엔진이나 로터, 그리고 주변에서 들리는 항공기
options-volume-effects = 효과음
    .facts = 무기, 폭발, 기어, 무전, 그 밖의 소리
options-volume-warnings = 경고음
    .facts = 실속 경보, 레이더와 미사일 경보, 화재 사이렌, 사이드와인더 톤
options-section-cockpit = 조종석
options-low-pass = 엔진 로패스
    .facts =
        기내에서는 엔진 소리가 둔탁하게 들립니다 (캐노피와 헬멧이 고음을 줄임)
        끔: 외부와 동일

## 조작

options-section-bindings = 키 설정
options-keys-joystick = 키와 조이스틱
    .facts =
        지도 키, 조이스틱과 HOTAS의 축 및 버튼
        다음 비행을 위해 저장됩니다
options-section-head-tracking = 헤드/아이 트래킹
options-head-tracking = 헤드/아이 트래킹
    .facts =
        머리를 돌리고 움직이면 조종석 시야가 따라갑니다 (데스크톱. VR에서는 헤드셋이 담당)
        트래킹 프로그램(OpenTrack...): FreeTrack 출력, 또는 127.0.0.1 포트 4242로 UDP
        Home: 현재 머리 위치를 중앙으로 설정
options-tracking = 트래킹 ({ $source })
options-no-tracker = 트래커 신호 없음: 트래킹 프로그램을 시작하세요

## EFFECTS

options-section-effects = Effects
options-section-retro = Retro
effect-shake = Camera shake
    .facts =
        The view shakes with the gun, nearby blasts, stall buffet, touchdowns and the runway
        VR: the cockpit vibrates, your head stays still
effect-flashes = Combat flashes
    .facts =
        Flak bursts, explosions and missile launches light the clouds, the ground and the cockpit for a moment
effect-barrage = Night barrage
    .facts =
        Conventional war at night: the towns' guns fire tracer blindly into the sky, searchlights sweep over them
        Only for the look: the barrage hits nothing, the searchlights find nothing
effect-cockpit-night = Cockpit night lighting
    .facts =
        A dim red flood light in the cockpit after dark (blue-green under the goggles)
        The master caution pulses amber over the panel while a warning is on
effect-nvg = Night vision goggles
    .facts =
        Bright lights bloom into halos, the picture grains more in the dark
        Desktop: the goggles' round field of view
effect-snow = Snow
    .facts =
        Above the freezing level storms snow instead of rain: low in a northern winter, high in summer
effect-mist = Ground mist
    .facts =
        Modern shaders: mist lies over the low ground and the sea at dawn, dusk and night
        Thin in the deserts, none in storms
effect-cloud-shadows = Cloud shadows
    .facts =
        Modern shaders: the clouds' shadows on the ground below
effect-vapour = Vapour and downwash
    .facts =
        Pulling hard, vapour trails from the wingtips
        A helicopter hovering low whips up dust, or spray over water
effect-night-sky = Night sky
    .facts =
        Shooting stars now and then, the stars low over the horizon twinkle
        The northern lights over the North Cape
effect-vga = VGA colours
    .facts =
        The 1989 look with modern shading: 64 steps a colour channel, as a VGA card's, dithered
effect-scanlines = Scanlines
    .facts =
        A CRT monitor's scanlines (not in VR)
