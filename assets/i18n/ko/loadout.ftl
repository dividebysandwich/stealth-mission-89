# 무장 편집기 (src/loadout.rs). 무기 이름은 실제 명칭 그대로이며 번역하지 않습니다.

loadout-none = 없음
loadout-briefed = 브리핑대로
    .facts = 임무가 브리핑된 때의 무장
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
weapon-role-21 = Air-to-air, infrared homing, very short range
employment-21 = Range: 1-6 km, farther head-on
