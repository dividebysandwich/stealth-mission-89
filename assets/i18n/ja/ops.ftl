# キャンペーンの作戦画面（src/campaign/ops.rs）。

ops-thunderstorms = 雷雨
ops-wx-short = 天候 { $now }、{ $at }に{ $next }へ
ops-wx-line = { $weather } { $at }に{ $next }へ変化。
ops-clock = { $day }日目  { $time }
ops-landed-at = { $base } に着陸
ops-ground-crew = 整備員の準備は完了しています
ops-rearm = 再武装・給油
ops-end-sortie = 出撃終了
ops-on-server = { $server } 上
ops-serving = 公開中
ops-tab-missions = 任務
ops-tab-map = 戦況図
ops-tab-air-defence = 防空
ops-tab-forces = 戦力
ops-tab-players = プレイヤー
ops-tab-debrief = デブリーフィング
ops-tab-log = ログ
ops-tab-chat = チャット
ops-blue = 青軍 { $count }
ops-red = 赤軍 { $count }
ops-war-over = 戦争は終結しました
ops-running = 戦争シミュレーション実行中
ops-save-exit = 保存して終了
ops-disconnect = 切断
ops-leave-local = 保存してキャンペーンを離れますか？
    .text = 再開するまで戦争は停止します。飛行中の航空機は失われ、進行中の降下作戦は基地からやり直しになります。
ops-leave-host = 保存してキャンペーンの公開を停止しますか？
    .text = 参加中のプレイヤーは切断され、再開するまで戦争は停止します。飛行中の航空機は失われ、進行中の降下作戦は基地からやり直しになります。
ops-leave-client = サーバーから切断しますか？
    .text = キャンペーンはサーバー上で続行されます。ここには何も保存されません。出撃は終了します。
ops-unit-destroyed = { $unit } 破壊：作戦画面に戻ります

## 任務

ops-section-missions = 任務
ops-threat = 脅威 { $threat }
ops-no-missions = 任務なし：前線は静穏です
ops-ai-helicopter = UH-60のパイロットが引き受けない場合、{ $time }後にAIのヘリコプターが実施します
ops-defences = 防御：SAM陣地{ $sams }か所、脅威 { $threat }
ops-recon-intel = 偵察情報
ops-order-insertion = 降下作戦を発令
    .facts = 護衛機で掩護するか、UH-60で自ら率いて降下させます
ops-from = 出撃元
ops-planned-by-group = グループで計画済み
ops-stores = { $aircraft } - 兵装：{ $stores }
ops-fly-uh60 = UH-60で飛行
ops-accept = 任務を受諾
ops-fly-with-ai = 離陸
ops-take-control = 操縦を引き継ぐ
    .facts = AI編隊の機体を今の位置から操縦する
ops-planning = 計画中：{ $pilots }
ops-flying = 飛行中：{ $pilots }
ops-package = { $kinds } とのパッケージ：{ $mins }分以内にここで2種類を達成すると、各+{ $bonus }ポイント
ops-sead-on-way = AI SEADが向かっています
ops-request-sead = AI SEADを要請（物資{ $cost }、航空機{ $aircraft }機）
    .facts = 友軍攻撃機が、あなたに先立ってこの地のSAM陣地を攻撃します
    .disabled = 物資または地上の航空機が不足しています（戦力を参照）

## ルール、物資、戦力

ops-rule-damage = リアルな被害
ops-rule-guidance = リアルな誘導
ops-rule-no-crashes = 墜落なし
ops-rule-time = 連続時間
ops-rule-weapons = リアルなF-117A兵装
ops-rule-friendly-fire = 友軍誤射
ops-rule-airframe-losses = 失った機体を数える
ops-rule-no-ai-missions = AI任務なし
ops-rule-classic = クラシック
ops-materiel = 物資
ops-materiel-blue = 青軍 { $materiel }（+{ $income }/時）
ops-materiel-red = 赤軍 { $materiel }（+{ $income }/時）
ops-crew-tempo = 敵の作戦テンポ ×{ $pace }
ops-due = T-{ $mins }分
ops-due-now = 進行中
ops-front-blue = 前線：青軍が優勢
ops-front-red = 前線：赤軍が優勢
ops-front-even = 前線：均衡
ops-operation = { $place }占領:
ops-friendly = 友軍
ops-enemy = 敵軍
ops-section-forces = 戦力
ops-places-held = 保持拠点
ops-materiel-row = 物資
ops-income = 1時間あたりの収入
ops-income-from = 内訳
ops-income-sources = { $base } + 拠点{ $places }か所 x { $per_place } + 都市{ $cities }か所 x { $per_city }
ops-fighters-ground = 地上の戦闘機
ops-helicopters-ground = 地上のヘリコプター
ops-in-air = 飛行中
ops-in-air-value = 戦闘機{ $fighters }、攻撃機{ $strike }、ヘリコプター{ $helicopters }
ops-since-start = 開始以降
ops-earned = 獲得
ops-repairs = 修復
ops-operations = 出撃、空襲、降下作戦
ops-new-aircraft = 新規航空機
ops-lost = 喪失（都市、パイロット）
ops-section-how = 仕組み
ops-how =
    各陣営は毎時間、物資を獲得します。基本額に加えて、保持する拠点ごと、そのうち無傷の都市ごとに増えます。敵の都市を爆撃（阻止）すれば、敵の収入を削れます。
    AIは行動のすべてに物資を支払います。修復、戦闘機の出撃、空襲、独自の降下作戦などです。物資が不足すると、被害は修復されず、航空機は出撃しません。
    AIが飛ばせるのは保有する航空機だけです。撃墜されればそれまでで、着陸すれば再び飛べます。物資が続く限り、各陣営は毎時間、新しい機体を購入します。
    プレイヤーが不足することはありません。自機、自軍の基地（爆撃されても）、自分で発令した降下作戦のヘリコプターは常に使えます。
ops-how-repairs = 各陣営が同時に修理できる陣地は少数（無傷の都市が多いほど増える）で、前線に近いものから修理する。修理後すぐに再び破壊された陣地は、そのたびに修理が長くなる。
ops-how-airframes = 失った機体を数える：パイロットが失う機体1機ごとに、味方AIの戦闘機かヘリが1機減る。
ops-section-prices = 価格
ops-price-airfield = 飛行場の修復
ops-price-sam = SAM陣地の修復
ops-price-city = 都市の再建
ops-price-ship = 艦船の修理
ops-price-gun = 火砲またはMANPADSチーム
ops-price-sortie = 戦闘機の出撃
ops-price-raid = 空襲
ops-price-insertion = AIの降下作戦
ops-price-fighter = 新型戦闘機
ops-price-helicopter = 新型ヘリコプター
ops-price-city-lost = 都市の破壊（損失）
ops-price-captured = パイロットの捕縛（損失）

## 戦況図

ops-click-to-watch = { $unit } - クリックで観戦
ops-section-legend = 凡例
ops-legend-friendly-airfield = 友軍の飛行場
ops-legend-enemy-airfield = 敵の飛行場
ops-legend-city = 友軍 / 敵の都市
ops-legend-airfield = 味方 / 敵の飛行場
ops-legend-insertion = 投入ルート
ops-watch-hint = 部隊をクリックするとライブで観戦できます
ops-map-hint = ホイール：ズーム、ドラッグ：移動
ops-patriot = パトリオット部隊
ops-drone-center = ドローン指揮所
ops-manned-by = 配置：{ $pilot }
ops-manned-by-other = 別のプレイヤーが配置中
ops-control = 操作
    .facts = コンソールを引き継ぎます

## デブリーフィング、終了、ログ、チャット

ops-no-sortie = まだ出撃していません
ops-section-sortie = 出撃
ops-achieved = 目標達成
ops-not-achieved = 目標未達成
ops-waiting-raid = 空襲を待機中
ops-waiting-insertion = 降下作戦を待機中
ops-flight-time = 飛行時間 { $mins }分
ops-kills = 撃破：航空機{ $air }、地上{ $ground }
ops-points = { $points }ポイント
ops-points-bonus = { $points }ポイント（パッケージボーナス +{ $bonus }）
ops-campaign-points = { $rank } - キャンペーンポイント { $points }
ops-campaign-record = キャンペーン：出撃{ $sorties }回、目標達成{ $won }回
ops-section-meanwhile = その間に
ops-section-effects = 戦争への君の影響
ops-victory = 勝利
    .text = 敵は飛行場を1つも保持していません。戦域は我が方のものです。
ops-defeat = 敗北
    .text = 友軍の飛行場が残っていません。戦域は失われました。
ops-end-day = { $day }日目。友軍拠点 { $friendly }、敵拠点 { $enemy }。
ops-next-campaign = 次のキャンペーン開始まで { $time }
ops-next-campaign-now = 次のキャンペーンを開始します
ops-next-from-menu = 次のキャンペーンはメインメニューから開始してください。
ops-section-log = キャンペーンログ
ops-section-chat = チャット
ops-no-messages = メッセージはまだありません
ops-message = メッセージ
    .hint = メッセージを入力し、Enterで送信
ops-send = 送信

## ブリーフィング

ops-briefing = 任務ブリーフィング
ops-briefing-line = 出撃元 { $base } - 目標まで { $km } km - { $aircraft } - 兵装：{ $stores }
ops-route-locked = ルートはロック中：{ $pilots } が飛行中
ops-route-threats = ルートと脅威
ops-take-off = 離陸
ops-radars-shown = 表示レーダー：友軍機が探知済みで、まだ健在のもの

## プレイヤー

ops-section-players = プレイヤー
ops-col-name = 名前
ops-col-status = 状態
ops-col-sorties = 出撃
ops-col-points = ポイント
ops-col-pilot = パイロット
ops-col-objectives = 目標
ops-col-kills = 撃破 空/地
ops-col-medal = 最高の勲章
ops-you = （あなた）
ops-host = ホスト
ops-status-planning = { $title } を計画中
ops-status-board = 任務ボードを閲覧中
ops-status-flying = { $title } を飛行中 - { $aircraft }
ops-status-reconnecting = 接続が切れました - 機体を{ $seconds }秒間保持
ops-flown = { $flown }（達成{ $won }）
ops-section-leaderboard = リーダーボード
ops-alone = 単独で飛行中です。一緒に飛ぶには、メインメニューのマルチプレイヤーからこのキャンペーンをホストしてください。
ops-rules = このキャンペーンのルール（開始時から固定）：{ $rules }
ops-no-sorties = まだ出撃していません
ops-ai-load = UH-60のパイロットが受け取らなければ、{ $time } 後にAIヘリコプターが荷を受け取ります
ops-ai-takes-off = { $time }後に離陸
ops-ai-aircraft = AI { $aircraft }
ops-cargo-in = 貨物 { $delivered } / { $loads } 到着 - ヘリパッドはあと { $time } 時間待機
ops-rule-classic-terrain = クラシック地形
ops-rule-classic-ai-flight = クラシックAI飛行
ops-layers-hint = レイヤー (右上)：マップの表示内容
ops-map-tab = マップ
ops-on-radio = 無線中
ops-mute = ミュート
    .facts = このプレイヤーの無線を聞く (または聞かない)
ops-unmute = ミュート解除
