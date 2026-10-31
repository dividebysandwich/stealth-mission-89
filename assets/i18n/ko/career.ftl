# 파일럿 경력 (src/career.rs): 계급, 훈장, 복무 기간.

## 미 공군 계급 (대한민국 공군의 동급 계급으로 번역), 약어

rank-0 = 소위
    .short = 소위
rank-1 = 중위
    .short = 중위
rank-2 = 대위
    .short = 대위
rank-3 = 소령
    .short = 소령
rank-4 = 중령
    .short = 중령
rank-5 = 대령
    .short = 대령

## 훈장 (미국 훈장: 고유 명칭)

medal-air-medal = Air Medal
medal-distinguished-flying-cross = Distinguished Flying Cross
medal-silver-star = Silver Star
medal-air-force-cross = Air Force Cross
medal-medal-of-honor = Medal of Honor
medal-purple-heart = Purple Heart
medal-prisoner-of-war-medal = Prisoner of War Medal

## 출격 후 소식

career-awarded = 수훈: { $medal }
career-kia = 전사
    .short = KIA
career-pow = 전쟁 포로
    .short = POW
career-active = 현역 복무 중
career-captured = 포로가 되었습니다: 경력이 끝났습니다
career-tour-sortie = 복무 기간: { $length }회 출격 중 { $sortie }회째
career-tour-complete = 복무 기간 완료: { $points }점 (+{ $bonus })
career-promoted = 진급: { $rank }

## 경력 창

career-title = 파일럿 기록
career-group-tour = 복무
career-set-by-tour = 파일럿의 복무 기간에 따라 정해짐
career-pilots = 파일럿
career-new-pilot = 신규 파일럿
career-name = 이름
career-enlist = 입대
career-select = 파일럿을 선택하거나 입대시키세요
career-service-record = 복무 기록
career-rank = 계급
career-points = 점수
career-sorties = 출격
career-tours = 복무 기간
career-status = 상태
career-decorations = 훈장
career-tour-of-duty = 복무 기간
career-tour-line = { $theater }  { $situation }  { $length }회 출격 중 { $sortie }회째  { $points }점
career-destroyed = 지금까지 목표 { $count }개 파괴
career-fly-next = 다음 출격
career-new-tour = { $theater }, { $situation }, 적 숙련도 { $opponents }: { $count }회 출격
    .facts = 파괴한 것은 복무 기간 동안 계속 파괴된 상태로 남습니다
career-start-tour = 복무 시작
career-retired = 현역에서 퇴역
career-last-sorties = 최근 출격
