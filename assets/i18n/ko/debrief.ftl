# 임무 디브리핑 (src/debrief.rs)과 시뮬레이션 보고서 (stealth-sim score.rs).

## 비행이 끝난 방식

outcome-landed = 안전 착륙
outcome-bad-landing = 불량 착륙
outcome-ejected = 탈출 - 낙하산 착지
outcome-rescued = 탈출 - CSAR 구조
outcome-captured = 탈출 - 포로가 됨
outcome-crashed = 추락
outcome-ejection-failed = 탈출 실패
outcome-aborted = 임무 중단
outcome-collision = 충돌

## 평가

rating-unsatisfactory = 불량
rating-marginal = 미흡
rating-satisfactory = 양호
rating-excellent = 우수
rating-outstanding = 탁월

## 점수

points-primary = 주 목표
points-secondary = 부 목표
points-aircraft = 파괴한 항공기
points-unseen = 발각되지 않고 격추
points-ground = 파괴한 지상 표적
points-drones = 격추한 드론
points-drones-through = 돌파한 드론
points-nuclear = 원전 타격
points-detected = 피탐
points-missile-hits = 미사일 피격

## 타임라인 (지명과 항공기 이름은 임무에 나온 그대로)

timeline-primary = {" "}(주)
timeline-secondary = {" "}(부)
timeline-destroyed = 파괴: { $place }{ $tag }
timeline-destroyed-ground = 지상 표적 파괴{ $tag }
timeline-shot-down = 격추: { $aircraft }{ $tag }
timeline-hit = 미사일 피격
timeline-seen = { $aircraft }에게 발각됨
timeline-radar-id = 레이더 식별: { $place }
timeline-objective = 목표 도달{ $tag }
timeline-got-through = { $aircraft } 돌파
timeline-photo = 사진 촬영{ $tag }

## 디브리핑 창

debrief-title = 임무 디브리핑
debrief-training = 훈련 임무
debrief-results = 결과
debrief-flight-map = 비행 지도
debrief-section-results = 결과
debrief-accomplished = 달성
debrief-failed = 실패
debrief-primary = 주 목표
debrief-secondary = 부 목표
debrief-aircraft = 항공기
debrief-drones = 드론
debrief-drones-value = 격추 { $down }, 돌파 { $through }
debrief-ground = 지상
debrief-weapons = 무기
debrief-radar-ids = 레이더 식별
debrief-visual-ids = 육안 식별
debrief-hits-taken = 피격
debrief-time = 시간
debrief-section-points = 점수
debrief-difficulty = 난이도
debrief-total = 합계
debrief-section-timeline = 타임라인
debrief-no-events = 사건 없음
debrief-section-exposure = 레이더 노출
debrief-tracked = 추적됨
debrief-closest = 최근접
debrief-worst = 최악
debrief-at = 시각 { $time }
debrief-altitude = 고도
debrief-speed = 속도
debrief-margin = 여유
debrief-strongest = 최강
debrief-tracked-by = 추적: { $sites }
debrief-section-map = 지도
debrief-legend-safe = 탐지 불가
debrief-legend-close = 탐지 직전
debrief-legend-detectable = 탐지 가능
debrief-legend-tracked = 추적 / 미사일 발사
debrief-legend-hit = 피격 / 격추
map-symbols = 기호
map-legend-clouds = 구름 / 적란운
map-legend-snow = 눈, 약함에서 강함까지
map-legend-rain = 비, 약함에서 강함까지
map-legend-thunderstorm = 뇌우 셀
debrief-pause = 일시정지
debrief-play = 재생
