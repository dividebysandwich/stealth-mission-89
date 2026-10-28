# ニーボード（src/kneeboard.rs、ストロークフォント）、ミッションプランナー（src/planner.rs）、
# セーブ済みミッション（src/savegame.rs）。航法略語（PT、BRG、ETA、PRI、SEC、KM、KT）は
# 英語のままです。

kb-brief = ブリーフ
kb-route = ルート
kb-map = マップ
kb-target = 目標
kb-keys = キー
kb-tip = マップ、ミッションメモ、キーガイド
kb-geran-file = GERAN-2（ファイル）
kb-objectives = 目標
kb-primary = 主目標：{ $objective }
kb-secondary = 副目標：{ $objective }
kb-times = 時刻
kb-deadline = 期限 { $time }
kb-now = 現在 { $time }
kb-conditions = 状況
kb-moon = 月：{ $phase }（照度{ $lit }%）
kb-armament = 兵装
kb-roe = 交戦規則
kb-total = 合計 { $km } KM  { $kt } KT
kb-fuel = 燃料 { $lbs } LBS
kb-preparing = 準備中...
kb-airborne = 飛行中
kb-developing = 現像中...
kb-no-photo = 写真なし
kb-airborne-target = 空中目標
kb-scroll-vr = スクロール：サムスティック、指をボードに
kb-scroll-desktop = スクロール：ボード上でマウスホイール

## ミッションプランナー

planner-low = 低高度 500 FT
planner-medium = 中高度 5000 FT
planner-high = 高高度 20000 FT
planner-title = ミッションプランナー
planner-route = ルート
planner-base = 基地
planner-total = 合計 { $km } KM  { $time }
planner-briefed = ブリーフィングのルート { $km } km、燃料 { $fuel } lbs
planner-coverage = レーダー覆域：
    .note = レーダー覆域は、480ktの水平飛行、ベイ閉鎖を前提としています。
planner-detected = ルートのうち { $km } km が探知される
planner-pulse = パルスレーダー（P）の探知距離
planner-doppler = ドップラーレーダー（D）の探知距離
planner-missile = ミサイルの射程
planner-uav = 敵UAV
planner-edit-hint = 点をドラッグで移動、マップをドラッグでパン。区間をクリックで点を追加、右クリックで削除。ホイール：ズーム。
planner-reset = リセット
planner-view-hint = マップをドラッグでパン。ホイール：ズーム。

## セーブ済みミッション

saves-title = セーブ済みミッション
saves-subtitle = 飛行中にメニューの「ミッションを保存」で保存されたもの
saves-none = セーブ済みのミッションはありません
saves-line = { $theater }  -  { $aircraft }  -  ミッション時間 { $time }
saves-saved-at = 保存日時 { $date }
saves-unreadable = 読み込めないセーブ（別バージョン）
saves-delete =
    .facts = このセーブを削除
saves-back =
    .facts = 単独ミッション画面に戻る
