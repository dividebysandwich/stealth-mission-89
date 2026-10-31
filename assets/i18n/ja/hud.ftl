# ゲームのHUDメッセージとフライト中の2Dオーバーレイ（src/hud、src/combat.rs、
# src/csar_view.rs、src/campaign）。HUDは大文字・アクセントなしで描画します。
# アビオニクス固有の語（CCIP、THR、KM、MAXなど）は英語のままです。

hud-no-ejection-seat = 射出座席なし
hud-waypoint-reached = ウェイポイント { $waypoint } 到達
hud-select-camera = カメラ選択
hud-open-bay = ベイを開く
hud-camera-page = { $page } ページ
hud-target-in-view-shoot = 目標視界内 - 撮影
hud-target-in-view = 目標視界内
hud-aim-at-target = 目標に照準
hud-photo = 撮影 { $state }
hud-rearmed = 再武装・給油完了
hud-landed-rearm = 着陸 - 再武装するか出撃を終了
hud-objective-achieved = 目標達成

## 任務終了オーバーレイ

end-accomplished = 達成
end-not-accomplished = 未達成
end-primary = 主目標：{ $result }
end-secondary = 副目標：{ $result }
end-kills = 撃破：航空機{ $air }、地上目標{ $ground }
end-time = 任務時間 { $time }
end-damage = 被害：被弾{ $hits }
end-training = 訓練ミッション
end-press-any-key = 任意のキーを押してください

## 墜落したパイロットのパネル

csar-flare = 信号フレア
csar-faster = 加速
csar-normal = 通常速度
csar-skip = 回収をスキップ
csar-parachute = パラシュート展開
csar-on-ground = 地上
csar-down = 敵地に墜落
csar-pickup = 救出   UH-60が上空 - 回収中
csar-inbound = 救出   UH-60 { $direction } { $km } KM - 接近中
csar-fire-flare = 救出   UH-60 { $direction } { $km } KM - 接近したらフレアを発射
csar-enemy = 敵    ハインド{ $count }機、最寄り { $km } KM
csar-flares = フレア   { $count }      地上 { $time }

## パネルの協力プレイ通知（src/player_notices.rs）、防空コンソールの注記

notice-joined = { $pilot } が参加
notice-left = { $pilot } が退出
notice-lost-connection = { $pilot } の接続が切れた
st-select-track = 先にトラックを選択してください
