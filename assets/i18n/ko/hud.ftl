# HUD와 비행 중 2D 오버레이에 표시되는 게임 메시지 (src/hud, src/combat.rs,
# src/csar_view.rs, src/campaign). HUD는 이 메시지를 영어로 표시한다. 항전 장비 용어
# (CCIP, THR, KM, MAX...)는 영어로 유지한다.

hud-no-ejection-seat = 사출좌석 없음
hud-waypoint-reached = 웨이포인트 { $waypoint } 도달
hud-select-camera = 카메라 선택
hud-open-bay = 무장창 열기
hud-camera-page = { $page } 페이지
hud-target-in-view-shoot = 표적 확인 - 촬영
hud-target-in-view = 표적 확인
hud-aim-at-target = 표적 조준
hud-photo = 사진 { $state }
hud-rearmed = 재무장 및 재급유 완료
hud-landed-rearm = 착륙 완료 - 재무장하거나 출격을 종료하십시오
hud-objective-achieved = 목표 달성

## 임무 종료 오버레이

end-accomplished = 달성
end-not-accomplished = 미달성
end-primary = 주 목표: { $result }
end-secondary = 부 목표: { $result }
end-kills = 격추: 항공기 { $air }대, 지상 표적 { $ground }개
end-time = 임무 시간: { $time }
end-damage = 피해: 피격 { $hits }회
end-training = 훈련 임무
end-press-any-key = 아무 키나 누르십시오

## 격추된 조종사 패널

csar-flare = 신호 플레어
csar-faster = 빠르게
csar-normal = 정상 속도
csar-skip = 구조 건너뛰기
csar-parachute = 낙하산 전개
csar-on-ground = 지상
csar-down = 적지에 추락
csar-pickup = 구조   UH-60 상공 - 탑승 중
csar-inbound = 구조   UH-60 { $km } KM { $direction } - 접근 중
csar-fire-flare = 구조   UH-60 { $km } KM { $direction } - 가까워지면 플레어 발사
csar-enemy = 적   Hind { $count }기, 최근접 { $km } KM
csar-flares = 플레어   { $count }      지상 체류 { $time }

## 패널의 협동 알림 (src/player_notices.rs), 방공 콘솔의 안내

notice-joined = { $pilot }: 참가
notice-left = { $pilot }: 퇴장
notice-lost-connection = { $pilot }: 연결 끊김
st-select-track = 먼저 트랙을 선택하십시오
