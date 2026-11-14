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
options-keep-time-accel = 시간 가속 유지
    .facts =
        탐지되거나 무장을 발사하거나 격추되어도 가속이 해제되지 않습니다
        TIME 버튼과 Shift + Z / Shift + X로만 바꿀 수 있습니다
options-tooltips = 조종석 툴팁
    .facts = 마우스 아래 또는 VR 손끝에 있는 조종석 조작부나 디스플레이가 무엇인지 표시합니다
options-player-labels = 플레이어 이름
    .facts = 멀티플레이: 다른 조종사의 이름을 기체 위에 표시, 거리와 관계없이
options-label-size =
    .facts =
        플레이어 이름의 크기
        50%: 기본값
options-seat-height = 좌석 높이
    .facts =
        조종석에 앉는 높이 (화면과 VR)
        0 cm: 기체의 기본 눈높이
options-section-vr-size = VR 조종석 크기
options-vr-size =
    .facts =
        헤드셋 속 조종석(과 세계)의 크기
        100%: 실물 크기
options-section-gif = GIF 녹화
options-gif-size = 크기
options-gif-rate = 프레임
options-gif-scale-100 =
    .facts = 창의 전체 해상도: 큰 파일
options-gif-scale-25 =
    .facts = 작은 파일
options-gif-rate-50 =
    .facts = 게임이 100 fps 이상으로 실행될 때만(한 프레임 걸러 기록)
options-gif-hint = F12(재지정하지 않은 경우)로 시작과 정지. 사진 폴더에 저장

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
options-scenery-distance = 풍경 표시 거리
    .facts = 먼 도시: 7–30 km, 농경지: 24–30 km. 나무 표시 거리는 바뀌지 않습니다. VR 자동 세부 설정이 풍경 표시 거리를 줄일 수 있습니다.
options-city-light-distance = 도시 조명 표시 거리
    .facts = 가로등, 창문, 건물 경고등의 거리로, 풍경 표시 거리와 독립적입니다. 날씨에 따라 시야가 제한될 수 있습니다.
options-cloud-quality = 구름
cloud-quality-auto = 자동
    .facts = VR에서는 낮음, 화면에서는 높음
cloud-quality-low = 낮음
    .facts = 3분의 1 해상도의 거친 구름 (가장 빠름)
cloud-quality-medium = 중간
    .facts = 절반 해상도, 더 세밀함
cloud-quality-high = 높음
    .facts = 전체 해상도, 가장 세밀한 표현과 자체 그림자 (GPU 시간 소모)

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
options-volume-rwr = RWR
    .facts = 레이더 경보음, 경고음 음량의 일부로
options-section-rwr-tones = 레이더 경보음
rwr-tones-all = 모두
    .facts = 나를 조사하는 레이더, 나를 탐지하는 매 스캔, 미사일 발사
rwr-tones-detect = 탐지만
    .facts =
        레이더가 나를 탐지할 때 한 번(레이더가 잠시 놓친 뒤에는 다시)
        그리고 미사일 발사
rwr-tones-launch = 발사만
    .facts = 나를 향해 발사된 미사일만
rwr-tones-off = 끔
    .facts = 레이더 경보음 없음(경고등과 화면은 계속 경고)
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

options-section-effects = 효과
options-section-retro = 레트로
effect-shake = 카메라 흔들림
    .facts =
        기관포, 근처 폭발, 실속, 접지, 활주로에 따라 화면이 흔들립니다
        VR: 조종석이 진동하고 머리는 그대로입니다
effect-flashes = 전투 섬광
    .facts = 대공포 폭발, 폭발, 미사일 발사가 구름, 지면, 조종석을 잠시 밝힙니다
effect-barrage = 야간 저지 사격
    .facts =
        밤의 재래식 전쟁: 도시의 화포가 하늘로 예광탄을 무작정 쏘고 탐조등이 그 위를 훑습니다
        도시 위의 적 드론은 탐조등에 잡혀 화포의 사격을 끌어들입니다
        보기 위한 것일 뿐: 탄막은 아무것도 맞히지 않습니다
effect-cockpit-night = 조종석 야간 조명
    .facts =
        어두워지면 조종석에 희미한 붉은 조명 (고글 착용 시 청록색)
        경고가 켜져 있는 동안 마스터 코션이 패널 위에서 호박색으로 깜박입니다
effect-nvg = 야간 투시 고글
    .facts =
        밝은 빛이 후광처럼 번지고 어두울수록 화면에 노이즈가 늘어납니다
        데스크톱: 고글의 둥근 시야
effect-snow = 눈
    .facts = 어는점 위에서는 폭풍이 비 대신 눈을 내립니다: 북쪽의 겨울에는 낮게, 여름에는 높게
effect-mist = 지면 안개
    .facts =
        최신 셰이더: 새벽, 해질녘, 밤에 낮은 지대와 바다 위에 안개가 깔립니다
        사막에서는 옅고 폭풍 속에서는 없습니다
effect-cloud-shadows = 구름 그림자
    .facts = 최신 셰이더: 아래 지면에 드리운 구름 그림자
effect-vapour = 수증기와 하강풍
    .facts =
        급기동 시 날개 끝에서 수증기 흔적
        낮게 호버링하는 헬기가 먼지를, 물 위에서는 물보라를 일으킴
        움직이는 차량이 뒤로 먼지를 일으킴
effect-night-sky = 밤하늘
    .facts =
        가끔 별똥별, 수평선 가까운 별이 반짝입니다
        노르카프 위의 오로라
effect-vga = VGA 색상
    .facts = 현대적인 셰이딩으로 재현한 1989년의 모습: VGA 카드처럼 색상 채널당 64단계, 디더링 적용
effect-scanlines = 주사선
    .facts = CRT 모니터의 주사선 (VR에서는 제외)

## LANGUAGE NAMES

lang-name-en = 영어
lang-name-bg = 불가리아어
lang-name-zh-Hans = 중국어(간체)
lang-name-hr = 크로아티아어
lang-name-cs = 체코어
lang-name-da = 덴마크어
lang-name-nl = 네덜란드어
lang-name-et = 에스토니아어
lang-name-fi = 핀란드어
lang-name-fr = 프랑스어
lang-name-de = 독일어
lang-name-el = 그리스어
lang-name-hu = 헝가리어
lang-name-id = 인도네시아어
lang-name-ga = 아일랜드어
lang-name-it = 이탈리아어
lang-name-ja = 일본어
lang-name-ko = 한국어
lang-name-lv = 라트비아어
lang-name-lt = 리투아니아어
lang-name-mt = 몰타어
lang-name-pl = 폴란드어
lang-name-pt-BR = 포르투갈어(브라질)
lang-name-ro = 루마니아어
lang-name-ru = 러시아어
lang-name-sk = 슬로바키아어
lang-name-sl = 슬로베니아어
lang-name-es = 스페인어
lang-name-sv = 스웨덴어
lang-name-uk = 우크라이나어

## Browser build: offer to play in the browser's language (src/i18n.rs)

language-offer-question = 브라우저 언어: { $language }. 이 언어로 플레이하시겠습니까?
language-offer-yes = 예
language-offer-no = 아니요
language-offer-never = 다시 묻지 않기
effect-sunbeams = 햇살
    .facts = 찢어진 구름 틈 사이로 쏟아지는 햇살 다발
options-msaa-forced = 픽셀화(레트로)가 켜져 있는 동안은 꺼짐
options-volume-radio = 무선
    .facts = 협동 무선으로 말하는 다른 플레이어와 음성 ATC
options-volume-atc = ATC 음성
    .facts = 음성 ATC 호출, 무선 음량의 일부입니다
options-volume-music = 음악
    .facts = 메인 메뉴 음악
options-section-atc = 음성 ATC
options-section-radio = 협동 무선
options-push-to-talk = 눌러서 말하기
    .facts =
        송신 키 (Caps Lock 또는 PTT에 할당한 조이스틱/VR 버튼)를 누르고 있으면 모든 플레이어에게 들립니다
        마이크는 처음 누를 때 열립니다
options-mic-gain = 마이크 게인
    .facts =
        마이크 송신 음량
        100%: 그대로
options-mic-default = 기본값
options-voice-text-only = 텍스트만
options-voices-line = 관제사: { $controllers }   조종사: { $pilot }
options-voices = 음성
    .facts =
        무선 호출용 음성을 내려받고 삭제합니다
        관제사와 내 음성을 고르고 들어 봅니다
options-voices-web = 브라우저에서는 텍스트만
options-tracking-mouse = 트래킹 중 마우스 시점
    .facts =
        켜짐: 마우스 오른쪽 버튼이 계속 조종석 시점을 돌리고 확대합니다
        꺼짐: 트래커가 송신하는 동안 마우스 오른쪽 버튼은 조종석에서 아무 일도 하지 않습니다
options-atc = 항공 교통 관제
    .facts =
        아군 비행장의 관제탑, 접근, GCA 무선 호출
        이륙 허가, 오버헤드 브레이크, 낮은 구름이나 나쁜 시정에서의 토크다운
options-pixel-block = 픽셀화 블록 크기
    .facts = 블록 한 변의 크기, 화면 픽셀 단위 (2: 세밀, 5: 거칠게)
effect-pixelize = 픽셀화
    .facts =
        3D 화면 전체 (지형, 항공기, 건물, 조종석)를 MS-DOS PC처럼 큰 정사각형 픽셀로 표시합니다. 메뉴와 툴팁은 선명하게 유지됩니다 (VR 제외)
        슬라이더로 블록 크기를 화면 픽셀 단위로 정합니다 (2: 세밀, 5: 거칠게)
        켜져 있는 동안 안티앨리어싱은 꺼집니다
        그 아래의 세 스위치로 HUD, 조종석, MFD, 계기, 니보드를 선명하게 둘 수 있습니다
effect-sharp-hud = HUD 제외
    .facts =
        픽셀화: HUD(와 헬멧 디스플레이)는 픽셀화된 화면 위에 선명하게 그려져 가는 기호도 읽을 수 있습니다
effect-sharp-cockpit = 조종석 제외 (버튼, 스위치 등)
    .facts =
        픽셀화: 조종석의 패널, 프레임, 좌석, 버튼, 스위치, 조종간, 스로틀이 선명하게 그려집니다
effect-sharp-gauges = MFD, 계기, 니보드 제외
    .facts =
        픽셀화: MFD, 자세계, 계기, 램프, 수치 표시, 라벨, 니보드가 선명하게 그려집니다
options-needs-pixelize = 픽셀화 필요
