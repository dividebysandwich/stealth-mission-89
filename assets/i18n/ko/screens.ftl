# 니보드 (src/kneeboard.rs, 획 글꼴), 임무 계획 화면 (src/planner.rs), 저장된 임무
# (src/savegame.rs). 항법 약어 (PT, BRG, ETA, PRI, SEC, KM, KT)는 영어로 유지한다.

kb-brief = 브리핑
kb-route = 경로
kb-map = 지도
kb-target = 표적
kb-keys = 키
kb-tip = 지도, 임무 메모, 키 안내
kb-geran-file = GERAN-2 (파일)
kb-objectives = 목표
kb-primary = 주: { $objective }
kb-secondary = 부: { $objective }
kb-times = 시간
kb-deadline = 제한 시간 { $time }
kb-now = 현재 { $time }
kb-conditions = 조건
kb-moon = 달: { $phase } (조도 { $lit }%)
kb-armament = 무장
kb-roe = 교전 규칙
kb-total = 총 { $km } KM, { $kt } KT
kb-fuel = 연료 { $lbs } LBS
kb-preparing = 준비 중...
kb-airborne = 비행 중
kb-developing = 현상 중...
kb-no-photo = 사진 없음
kb-airborne-target = 공중 표적
kb-scroll-vr = 스크롤: 썸스틱, 손가락을 보드에 대고
kb-scroll-desktop = 스크롤: 보드 위에서 마우스 휠

## 임무 계획

planner-low = 저고도 500 FT
planner-medium = 중고도 5000 FT
planner-high = 고고도 20000 FT
planner-title = 임무 계획
planner-route = 경로
planner-base = 기지
planner-total = 총 { $km } KM  { $time }
planner-briefed = 브리핑 경로 { $km } km, 연료 { $fuel } lbs
planner-coverage = 레이더 탐지 범위:
    .note = 레이더 탐지 범위는 480kt 수평 비행, 무장창 닫힘을 가정합니다.
planner-detected = 경로 중 { $km } km 탐지됨
planner-pulse = 펄스 레이더 (P) 탐지 거리
planner-doppler = 도플러 레이더 (D) 탐지 거리
planner-missile = 미사일 사거리
planner-uav = 적 UAV
planner-edit-hint = 점을 끌어 이동하고, 지도를 끌어 화면을 옮깁니다. 구간을 클릭하면 점이 추가되고, 오른쪽 클릭으로 삭제합니다. 휠: 확대/축소.
planner-reset = 초기화
planner-view-hint = 지도를 끌어 화면을 옮깁니다. 휠: 확대/축소.

## 저장된 임무

saves-title = 저장된 임무
saves-subtitle = 비행 중 메뉴의 임무 저장으로 저장됨
saves-none = 저장된 임무 없음
saves-line = { $theater }  -  { $aircraft }  -  임무 시간 { $time }
saves-saved-at = 저장: { $date }
saves-unreadable = 읽을 수 없는 저장 파일 (다른 버전)
saves-delete =
    .facts = 이 저장 파일 삭제
saves-back =
    .facts = 단일 임무 화면으로 돌아가기
