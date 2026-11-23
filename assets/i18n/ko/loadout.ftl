# 무장 편집기 (src/loadout.rs). 무기 이름은 실제 명칭 그대로이며 번역하지 않습니다.

loadout-none = 없음
loadout-briefed = 브리핑대로
    .facts = 임무가 브리핑된 때의 무장
loadout-own = 직접 고른 무장: 이 기체로 같은 종류의 다음 임무에서도 다시 사용됩니다. { loadout-briefed }로 임무 무장으로 돌아갑니다.
loadout-clear = 비우기
    .facts = 모든 하드포인트를 비웁니다
loadout-done = 완료
loadout-no-hardpoints = 하드포인트 없음
loadout-no-stores = 이 항공기는 무장을 탑재하지 않습니다
loadout-fantasy = 실제 항공기에는 없음
loadout-empty = 비어 있음
loadout-out-of-stock = 이 임무에는 재고 없음
loadout-mirror = 양쪽 동일
    .facts = { $station }에도 탑재합니다
loadout-stores = 무장
loadout-realistic-f117 = 사실적인 F-117A 무장
loadout-stations = 스테이션
loadout-click-drawing = 또는 그림에서 스테이션을 클릭
loadout-needs-equipment = 임무에 카메라나 특수 장비가 필요합니다

## 각 무기의 용도 (무기 유형 0~20)

# AIM-9M Sidewinder
weapon-role-0 = 공대공, 적외선 유도, 단거리
# AIM-120A AMRAAM
weapon-role-1 = 공대공, 레이더 유도, 중거리
# AGM-88A HARM
weapon-role-2 = 대레이더 미사일: 레이더와 SAM 사이트
# Penguin 3 ASM
weapon-role-3 = 대함 미사일
# AGM-84A Harpoon
weapon-role-4 = 장거리 대함 미사일
# AGM-65 Maverick
weapon-role-5 = TV 유도 미사일: 차량, 벙커, 건물
# GBU-12 Paveway
weapon-role-6 = 레이저 유도 폭탄: 강화 표적
# Mk 20 Rockeye
weapon-role-7 = 집속탄: 연약 표적과 면적 표적
# Durandal
weapon-role-8 = 활주로 파괴 폭탄
# Mk 82 LDGP
weapon-role-9 = 500 lb 자유낙하 폭탄
# Mk 82 Snakeye
weapon-role-10 = 저고도 투하용 500 lb 감속 폭탄
# Mk 20 Rockeye II
weapon-role-11 = 집속탄: 기갑 표적과 면적 표적
# Mk 122 Fireye
weapon-role-12 = 소이탄
# CBU-72 FAE
weapon-role-13 = 연료 공기 폭탄: 면적 표적
# Mk 35 incendiary
weapon-role-14 = 소이 집속탄
# ISCB-1 mines
weapon-role-15 = 지역 거부 지뢰
# IR/optical camera
weapon-role-16 = 표적 정찰 사진
# 1900 lb fuel tank
weapon-role-17 = 추가 연료, 투하 시 이송됨
# Special cargo
weapon-role-18 = 임무용 보급품 또는 화물
# 20 mm cannon
weapon-role-19 = 기관포 (무장 아님)
# AGM-114 Hellfire
weapon-role-20 = 레이저 유도 대전차 미사일: 타기팅 포드가 보는 곳으로 비행

## 각 무기의 사용법 (사거리는 HUD의 km, 고도는 피트)

employment-0 = 사거리: 1-12 km, 정면 교전 시 더 멀리
employment-1 = 사거리: 1-38 km, 정면 교전 시 더 멀리
employment-2 =
    사거리: 9-50 km
    발사: 5,000-20,000 ft
employment-3 =
    사거리: 6-14 km (클래식 유도: 13-14 km)
    발사: 100-500 ft
employment-4 =
    사거리: 6-41 km
    발사: 2,000-5,000 ft
employment-5 =
    사거리: 5-10 km
    발사: 2,000-5,000 ft
employment-6 =
    사거리: 1.5 km에서 고도의 2.5배까지, LOCK 유지 (15,000 ft에서 14 km, 25,000 ft에서 18 km)
    투하: 15,000-25,000 ft
employment-8 =
    사거리: 활주로 앞 0.9-1.3 km
    투하: 200-500 ft
employment-9 =
    사거리: 표적 앞 5-11 km
    투하: 3,000-20,000 ft
employment-12 =
    사거리: 표적 앞 0.9-2.2 km
    투하: 100-500 ft
employment-16 = 사거리: 2-8 km에서 촬영 (가까울수록 적이 많음), 기수를 표적에 향하기
employment-18 = 사거리: "Drop supplies now"가 뜨면 투하, 투하 지점 4 km 이내
employment-20 =
    사거리: 4-8 km
    발사: 2,000-3,000 ft
employment-guided-cluster =
    사거리: 1.5 km에서 고도의 2.5배까지, LOCK 유지 (3,000 ft에서 3.7 km, 5,000 ft에서 6 km)
    투하: 3,000-5,000 ft
employment-medium-retarded =
    사거리: 표적 앞 1.1-1.6 km
    투하: 300-1,000 ft
employment-low-retarded =
    사거리: 표적 앞 0.9-1.3 km
    투하: 200-500 ft

## 하드포인트

hardpoint-left-bay = 왼쪽 베이
hardpoint-right-bay = 오른쪽 베이
hardpoint-left-wing-bay = 왼쪽 날개 베이
hardpoint-right-wing-bay = 오른쪽 날개 베이
hardpoint-station-1-left-wingtip = 스테이션 1, 왼쪽 날개 끝
hardpoint-station-2-left-outer-wing = 스테이션 2, 왼쪽 바깥 날개
hardpoint-station-3-left-wing = 스테이션 3, 왼쪽 날개
hardpoint-station-4-left-inner-wing = 스테이션 4, 왼쪽 안쪽 날개
hardpoint-station-5-centreline = 스테이션 5, 중심선
hardpoint-station-6-right-inner-wing = 스테이션 6, 오른쪽 안쪽 날개
hardpoint-station-7-right-wing = 스테이션 7, 오른쪽 날개
hardpoint-station-8-right-outer-wing = 스테이션 8, 오른쪽 바깥 날개
hardpoint-station-9-right-wingtip = 스테이션 9, 오른쪽 날개 끝
hardpoint-station-1-left-wingtip-rail = 스테이션 1, 왼쪽 날개 끝 레일
hardpoint-station-2-left-outer-pylon = 스테이션 2, 왼쪽 바깥 파일런
hardpoint-station-3-left-inner-pylon = 스테이션 3, 왼쪽 안쪽 파일런
hardpoint-station-4-right-inner-pylon = 스테이션 4, 오른쪽 안쪽 파일런
hardpoint-station-5-right-outer-pylon = 스테이션 5, 오른쪽 바깥 파일런
hardpoint-station-6-right-wingtip-rail = 스테이션 6, 오른쪽 날개 끝 레일
hardpoint-tornado-left-wing-rail = 왼쪽 날개, 사이드와인더 레일
hardpoint-tornado-left-wing-pylon = 왼쪽 날개 파일런
hardpoint-tornado-left-shoulder-forward = 왼쪽 어깨, 전방
hardpoint-tornado-left-shoulder-aft = 왼쪽 어깨, 후방
hardpoint-tornado-right-shoulder-aft = 오른쪽 어깨, 후방
hardpoint-tornado-right-shoulder-forward = 오른쪽 어깨, 전방
hardpoint-tornado-right-wing-pylon = 오른쪽 날개 파일런
hardpoint-tornado-right-wing-rail = 오른쪽 날개, 사이드와인더 레일
hardpoint-a10-1-left-outer-wing = 스테이션 1, 왼쪽 바깥 날개
hardpoint-a10-2-left-wing = 스테이션 2, 왼쪽 날개
hardpoint-a10-3-left-wing = 스테이션 3, 왼쪽 날개
hardpoint-a10-4-left-centre-wing = 스테이션 4, 왼쪽 중앙 날개
hardpoint-a10-5-left-fuselage = 스테이션 5, 왼쪽 동체
hardpoint-a10-6-centreline = 스테이션 6, 중심선
hardpoint-a10-7-right-fuselage = 스테이션 7, 오른쪽 동체
hardpoint-a10-8-right-centre-wing = 스테이션 8, 오른쪽 중앙 날개
hardpoint-a10-9-right-wing = 스테이션 9, 오른쪽 날개
hardpoint-a10-10-right-wing = 스테이션 10, 오른쪽 날개
hardpoint-a10-11-right-outer-wing = 스테이션 11, 오른쪽 바깥 날개
hardpoint-av8b-1-left-rail = 스테이션 1, 왼쪽 사이드와인더 레일
hardpoint-av8b-3-left-middle-wing = 스테이션 3, 왼쪽 날개 중간
hardpoint-av8b-4-left-inboard-wing = 스테이션 4, 왼쪽 날개 안쪽
hardpoint-av8b-6-right-inboard-wing = 스테이션 6, 오른쪽 날개 안쪽
hardpoint-av8b-7-right-middle-wing = 스테이션 7, 오른쪽 날개 중간
hardpoint-av8b-9-right-rail = 스테이션 9, 오른쪽 사이드와인더 레일
hardpoint-f22-left-side-bay = 왼쪽 측면 베이(사이드와인더)
hardpoint-f22-left-main-bay = 주 베이, 왼쪽 절반
hardpoint-f22-right-main-bay = 주 베이, 오른쪽 절반
hardpoint-f22-right-side-bay = 오른쪽 측면 베이(사이드와인더)
hardpoint-fa18e-8-right-inner-wing = 스테이션 8, 오른쪽 안쪽 날개
hardpoint-fa18e-10-right-outer-wing = 스테이션 10, 오른쪽 바깥 날개
hardpoint-fa18e-11-right-wingtip = 스테이션 11, 오른쪽 날개 끝
hardpoint-a6-1-left-outer-wing = 스테이션 1, 왼쪽 바깥 날개
hardpoint-a6-2-left-inner-wing = 스테이션 2, 왼쪽 안쪽 날개
hardpoint-a6-4-right-inner-wing = 스테이션 4, 오른쪽 안쪽 날개
hardpoint-a6-5-right-outer-wing = 스테이션 5, 오른쪽 바깥 날개
hardpoint-c5-hold = 화물칸 (공중 투하 팔레트)
hardpoint-ch47-cabin = 객실 (공수부대)
weapon-role-21 = 공대공, 적외선 유도, 매우 짧은 사거리
employment-21 = 사거리: 1-6 km, 정면에서는 더 멀리
loadout-none-in-stock = (재고 없음)
loadout-empties = 적재할 때 스테이션 { $stations }을(를) 비웁니다
loadout-empties-two = 적재할 때 스테이션 { $first }와(과) { $second }을(를) 비웁니다
hardpoint-oh58-left-pylon = 왼쪽 무장 파일론
hardpoint-oh58-right-pylon = 오른쪽 무장 파일론
hardpoint-mh60r-left-outer = 왼쪽 바깥 스테이션 (헬파이어 발사기)
hardpoint-mh60r-left-inner = 왼쪽 안쪽 스테이션
hardpoint-mh60r-right-inner = 오른쪽 안쪽 스테이션
weapon-role-22 = 기관총 포드: 기관포처럼 기수 방향으로 발사합니다
employment-22 = 사거리: 최대 1.5 km, 포드당 500발
weapon-role-23 = 센서 신관 무기: 40 개의 스키트가 아래 차량을 향해 발사됩니다
employment-23 =
    투하 시 표적 포드로 지정한 지점으로 유도
    투하: SMS 페이지에서 설정한 산개 고도 이상 (500-3,000 ft)
weapon-role-24 = 경어뢰: 낙하산으로 강하해 수중을 탐색하고 잠수함을 추적
employment-24 =
    센서 운용사의 WPN 페이지에서 설정: 방위, 기동 지연, 탐색 패턴
    투하: 수면 위, 300 ft 및 100 kt 이하
weapon-role-25 = 원거리 지상 공격 미사일: 표적까지 순항한 뒤 조종사가 탐색기 영상을 보며 유도한다
employment-25 =
    SLAM 페이지에서 설정: 순항 고도, 탐색기 작동 거리, 주 표적 또는 보조 표적
    사거리: 최대 250 km, 동시에 4발 비행
weapon-role-26 = 활공 폭탄: 지정 지점까지 활공하며 투하 고도와 속도에 따라 사거리가 늘어남
employment-26 =
    포드 지점이나 지상 고정을, 없으면 주 목표를 겨냥
    사거리: 저공 약 20 km, 고공에서 최대 130 km. 방공망에 격추될 수 있음
weapon-role-27 = 공중 발사 기만체: 항공기 신호로 방공망에 날아들어 미사일을 끌어들임
employment-27 =
    지정 지점까지 날아간 뒤 최대 10분간 직진
    목표에는 무해
weapon-role-28 = 공수부대: 점령할 지점 상공에서 투하하는 1개 조
employment-28 =
    C-5의 문(2개 조)이나 CH-47의 램프(1개 조)로 강하 지역 상공에서
    지상 500~1500 ft, 150 kt 미만에서 강하. 조의 대부분이 지역에 착지해야 함
weapon-role-29 = 전술 재밍 포드: 재밍한 레이더를 번스루 거리까지 줄인다
employment-29 =
    JAM 페이지에서 설정: 포드마다 레이더나 섹터, 켜기/끄기
    재밍하면 더 먼 곳에서 탐지된다
weapon-role-30 = 비행장 공격 디스펜서: 활주로 파괴 자탄과 대전차 지뢰를 낙하산에 매달아 500 m 띠 모양으로 살포
employment-30 =
    파일런에 남아 2초 동안 살포합니다. 목표 위를 낮게 수평 비행하십시오
    지면 위 1000 ft 이하에서. 크레이터 세 개면 활주로가 폐쇄되고, 지뢰는 차량을 기다립니다
weapon-role-31 = 포드에 탑재된 무유도 70 mm 로켓: 2발씩 또는 연사로 차량과 노출된 병력을 공격
employment-31 =
    로켓 조준선(CCIP)으로 조준: 지금 발사한 로켓의 탄착점을 보여 준다
    누르면 2발, 누르고 있으면 연사. 4 km 이내 얕은 강하에서 가장 좋다
weapon-role-32 = 레이저 유도 70 mm 로켓: 각 로켓이 표적 포드의 레이저 점으로 향한다
employment-32 =
    표적 포드를 표적에 맞추고 기수 방향으로 발사하면 로켓이 레이저 점으로 선회한다
    최대 6 km. 명중할 때까지 포드를 표적에 유지할 것
weapon-role-33 = 1000 lb 자유낙하 폭탄
weapon-role-34 = 2000 lb 자유낙하 폭탄
weapon-role-35 = 대레이더 미사일: 레이더와 SAM 사이트, 단거리
employment-35 =
    사거리: 9-20 km
    발사: 3,000-10,000 ft
weapon-role-36 = 포드에 탑재된 무유도 127 mm 로켓: 2발씩 또는 연사로 차량과 노출된 병력을 공격
employment-36 =
    로켓 조준선(CCIP)으로 조준: 지금 발사한 로켓의 탄착점을 보여 준다
    누르면 2발, 누르고 있으면 연사. 6 km 이내 얕은 강하에서 가장 좋다
loadout-wingman = 윙맨
    .facts = 같은 기체와 무장의 AI 윙맨이 이륙 후 합류한다. 무선 메뉴(\)로 명령한다
