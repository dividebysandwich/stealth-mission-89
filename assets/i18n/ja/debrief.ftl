# ミッションのデブリーフィング（src/debrief.rs）とシミュレーションのレポート（stealth-sim score.rs）。

## 飛行の結末

outcome-landed = 無事着陸
outcome-bad-landing = 不時着
outcome-ejected = 脱出 - パラシュートで着地
outcome-rescued = 脱出 - CSARが救出
outcome-captured = 脱出 - 捕虜
outcome-crashed = 墜落
outcome-ejection-failed = 脱出失敗
outcome-aborted = 任務中止
outcome-collision = 衝突

## 評価

rating-unsatisfactory = 不良
rating-marginal = ぎりぎり
rating-satisfactory = 良好
rating-excellent = 優秀
rating-outstanding = 卓越

## ポイント

points-primary = 主目標
points-secondary = 副目標
points-aircraft = 撃破した航空機
points-unseen = 発見されずに撃墜
points-ground = 撃破した地上目標
points-drones = 撃墜したドローン
points-drones-through = 突破したドローン
points-nuclear = 原子力施設に命中
points-detected = 探知された
points-missile-hits = ミサイル被弾

## タイムライン（地名と機名は任務のまま）

timeline-primary = {" "}（主目標）
timeline-secondary = {" "}（副目標）
timeline-destroyed = { $place }を破壊{ $tag }
timeline-destroyed-ground = 地上目標を破壊{ $tag }
timeline-shot-down = { $aircraft }を撃墜{ $tag }
timeline-hit = ミサイルが命中
timeline-seen = { $aircraft }に視認された
timeline-radar-id = { $place }によるレーダーID
timeline-objective = 目標に到達{ $tag }
timeline-got-through = { $aircraft }が突破
timeline-photo = 撮影完了{ $tag }

## デブリーフィングのウィンドウ

debrief-title = ミッションデブリーフィング
debrief-training = 訓練ミッション
debrief-results = 結果
debrief-flight-map = 飛行マップ
debrief-section-results = 結果
debrief-accomplished = 達成
debrief-failed = 失敗
debrief-primary = 主目標
debrief-secondary = 副目標
debrief-aircraft = 航空機
debrief-drones = ドローン
debrief-drones-value = 撃墜{ $down }、突破{ $through }
debrief-ground = 地上
debrief-weapons = 兵装
debrief-radar-ids = レーダーID
debrief-visual-ids = 目視ID
debrief-hits-taken = 被弾
debrief-time = 時間
debrief-section-points = ポイント
debrief-difficulty = 難易度
debrief-total = 合計
debrief-section-timeline = タイムライン
debrief-no-events = イベントなし
debrief-section-exposure = レーダー被探知
debrief-tracked = 追尾
debrief-closest = 最接近
debrief-worst = 最悪
debrief-at = { $time }時点
debrief-altitude = 高度
debrief-speed = 速度
debrief-margin = 余裕
debrief-strongest = 最強
debrief-tracked-by = 追尾元：{ $sites }
debrief-section-map = マップ
debrief-legend-safe = 探知不可
debrief-legend-close = 探知寸前
debrief-legend-detectable = 探知可能
debrief-legend-tracked = 追尾 / ミサイル発射
debrief-legend-hit = 被弾 / 撃破
debrief-pause = 一時停止
debrief-play = 再生
