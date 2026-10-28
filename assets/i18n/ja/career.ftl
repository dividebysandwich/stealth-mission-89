# パイロットの経歴（src/career.rs）：階級、勲章、勤務期間。

## 米空軍の階級（航空自衛隊の相当階級に置き換え）、その略称

rank-0 = 3等空尉
    .short = 3尉
rank-1 = 2等空尉
    .short = 2尉
rank-2 = 1等空尉
    .short = 1尉
rank-3 = 3等空佐
    .short = 3佐
rank-4 = 2等空佐
    .short = 2佐
rank-5 = 1等空佐
    .short = 1佐

## 勲章（米国の勲章：固有名詞）

medal-air-medal = Air Medal
medal-distinguished-flying-cross = Distinguished Flying Cross
medal-silver-star = Silver Star
medal-air-force-cross = Air Force Cross
medal-medal-of-honor = Medal of Honor
medal-purple-heart = Purple Heart
medal-prisoner-of-war-medal = Prisoner of War Medal

## 出撃後のニュース

career-awarded = 叙勲：{ $medal }
career-kia = 戦死
    .short = KIA
career-pow = 捕虜
    .short = POW
career-active = 現役
career-captured = 捕虜となりました：経歴は終了です
career-tour-sortie = 勤務期間：出撃 { $sortie } / { $length }
career-tour-complete = 勤務期間完了：{ $points } ポイント（+{ $bonus }）
career-promoted = 昇進：{ $rank }

## 経歴ウィンドウ

career-title = パイロット記録
career-group-tour = 勤務期間
career-set-by-tour = パイロットの勤務期間で設定
career-pilots = パイロット
career-new-pilot = 新規パイロット
career-name = 名前
career-enlist = 入隊
career-select = パイロットを選択または入隊してください
career-service-record = 勤務記録
career-rank = 階級
career-points = ポイント
career-sorties = 出撃
career-tours = 勤務期間
career-status = 状態
career-decorations = 勲章
career-tour-of-duty = 勤務期間
career-tour-line = { $theater }  { $situation }  出撃 { $sortie } / { $length }  { $points } ポイント
career-destroyed = これまでに目標 { $count } 件を破壊
career-fly-next = 次の出撃へ
career-new-tour = { $theater }、{ $situation }、敵の練度 { $opponents }：出撃 { $count } 回
    .facts = 破壊した目標は、勤務期間の終わりまで破壊されたままです
career-start-tour = 勤務開始
career-retired = 現役を退きました
career-last-sorties = 直近の出撃
