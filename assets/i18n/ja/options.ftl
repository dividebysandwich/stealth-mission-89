# オプションウィンドウ（src/options.rs）と、飛行中メニューのそのページ。

options-title = オプション
options-tab-game = ゲーム
options-tab-graphics = グラフィック
options-tab-sound = サウンド
options-tab-controls = 操作
options-footer = 次回以降の飛行に保存されます。リアリズムの切り替えはゲームごとに選びます

## GAME

options-section-language = 言語
options-section-classic-modern = クラシック / モダン
options-cockpit = コックピット
    .facts =
        モダン：ベゼルボタン付きの高解像度ディスプレイ
        クラシック：DOS時代の計器盤レイアウトと 77x60 ディスプレイ（F-117A）
options-hud = HUD
    .facts =
        モダン：NAV、AA、AG モードを備えた緑色の高解像度 F-16 スタイル HUD
        クラシック：DOS時代の HUD
options-avionics = アビオニクス
    .facts =
        モダン：どちらの MFD でも使える F-16 スタイルの MFD フォーマット（状況表示、HARM、ターゲティングポッド、マップ、兵装）と動かせるカーソル
        クラシック：左にマップと TAC、右にカメラ、WAY、WPN、ORD
options-flight-model = 飛行モデル
    .facts =
        モダン：フライバイワイヤの空力飛行モデル。スティックはピッチとロールのレートを指令し、離すと機首はそのまま、飛行経路が保持されます
        クラシック：DOS時代の飛行モデル
options-section-flying = 飛行
options-auto-trim = オートトリム
    .facts =
        スティックを離すと、トリムが手放した姿勢を保持します
        ヘリコプター：速度による機首上げがなくなります。ジェット機：損傷した操縦舵面をトリムで補正します
        手動トリム：Alt + 矢印キー、Alt + Home でセンターに戻します
options-keep-time-accel = 時間加速を維持
    .facts =
        発見されても、兵装を発射しても、撃墜されても加速が解除されません
        加速を変えるのは TIME ボタンと Shift + Z / Shift + X のみ
options-tooltips = コックピットのツールチップ
    .facts = マウスまたは VR の指先の下にある、コックピットの操作部や表示の説明
options-player-labels = プレイヤー名
    .facts = マルチプレイ：他のパイロットの名前を機体の上に表示、距離に関係なく
options-label-size =
    .facts =
        プレイヤー名の大きさ
        50%：既定
options-seat-height = 座席の高さ
    .facts =
        コックピットでの座る高さ（画面とVR）
        0 cm：機体の通常の目の高さ
options-section-vr-size = VRコックピットのサイズ
options-vr-size =
    .facts =
        ヘッドセット内のコックピット（と世界）のサイズ
        100%：実物大
options-section-gif = GIF録画
options-gif-size = サイズ
options-gif-rate = フレーム
options-gif-scale-100 =
    .facts = ウィンドウの最大解像度：大きなファイル
options-gif-scale-25 =
    .facts = 小さなファイル
options-gif-rate-50 =
    .facts = ゲームが100 fps以上で動作しているときのみ（1フレームおきに記録）
options-gif-hint = F12（割り当て変更なしの場合）で開始と停止。ピクチャフォルダーに保存

## GRAPHICS

options-section-rendering = レンダリング
options-shaders = シェーダー
    .facts =
        モダン：控えめな指向性シェーディング、なめらかな距離ヘイズ
        クラシック：フラットシェーディング
options-shadows = 影
    .facts =
        モダン：コックピットと世界に太陽の影（GPU負荷あり）
        クラシック：影なし
options-point-lights = ポイントライト
    .facts =
        モダン：コックピットのフラッドライトとディスプレイの光。爆発、噴煙、滑走路灯が周囲を照らします
        クラシック：太陽と空の光のみ
options-time-of-day = 時刻
    .facts =
        モダン：太陽と月がミッション時計に合わせて動き、空は太陽とともに暗くなり明るくなります。夜は星と月齢が見えます
        モダン：月明かりで、夜間に敵パイロットがあなたを視認できる距離が決まります（新月の夜は 2 km、満月では 4 km）
        クラシック：ゆっくり変わる色と固定された太陽
options-msaa = アンチエイリアス（MSAA）
    .facts =
        エッジをなめらかに：4x MSAA、VR では 2x（GPU負荷あり）
        オフ：ピクセルのくっきりしたエッジ
options-section-display = 画面
options-fullscreen = フルスクリーン
    .facts = ウィンドウを画面いっぱいに表示します (Alt Enter または F11 でも切り替え)
options-vsync = 垂直同期
    .facts = フレームが画面の更新を待ちます: ティアリングなし。オフ: コンピューターが描けるだけのフレーム
options-fps-cap = フレームレート上限
    .facts = 1 秒あたりの最大フレーム数: 消費電力・発熱・ファン音を抑えます (ノート PC)
options-fps-cap-off = オフ
options-fps-cap-vr = VR ではヘッドセットがフレームレートを決めます
options-ui-scale = メニューの大きさ
    .facts = メニューとウィンドウの大きさ (画面に収まる範囲で)。100%: 本来の大きさ
options-reset = タブをリセット
    .facts = このタブの設定を初期値に戻します (確認あり)
options-reset-confirm = { $tab } をリセットしますか?
    .text = これらの設定は新しいプレイヤーの状態に戻ります。言語、音声、キー割り当てはそのままです。
options-auto-detail = 地形ディテール自動調整（VR）
    .facts =
        ヘッドセットでは、フレームが遅れている間は街、森、畑の描画距離が縮み、余裕があれば元に戻ります
        グラフィック設定は選んだままです
options-sun-moon = 太陽と月
    .facts =
        昼は空に太陽、夜は月と星
        月はミッションの月齢を示します
options-scenery-distance = 風景の表示距離
    .facts = 遠方の市街地：7–30 km、農地：24–30 km。樹木の距離は変わりません。VRの自動詳細設定により風景の距離が短縮される場合があります。
options-city-light-distance = 街の灯りの表示距離
    .facts = 街灯、窓、建物の警告灯の距離を風景の表示距離とは独立して設定します。天候により視認距離が制限される場合があります。
options-cloud-quality = 雲
cloud-quality-auto = 自動
    .facts = VRでは低、画面では高
cloud-quality-billboard = ビルボード
    .facts = 立体的な雲の代わりに視点を向く平面の雲 (最速)
cloud-quality-low = 低
    .facts = 解像度3分の1の粗い雲（最速）
cloud-quality-medium = 中
    .facts = 解像度2分の1、より細かい表示
cloud-quality-high = 高
    .facts = 最大解像度、最も細かい表示と自己影（GPU時間を使います）
cloud-quality-ultra = 最高
    .facts = 雲層、積雲、積乱雲を通り抜けられる立体として描画（GPU時間を最も使います）

## SOUND

options-section-volume = 音量
options-volume-master = マスター
    .facts = すべての音
options-volume-engine = エンジン
    .facts = 自機のエンジンまたはローター、周囲で聞こえる航空機の音
options-volume-effects = 効果音
    .facts = 兵装、爆発、降着装置、無線など
options-volume-warnings = 警告音
    .facts = 失速警告、レーダーとミサイルの警告、火災サイレン、サイドワインダーのトーン
options-volume-rwr = RWR
    .facts = レーダー警報音、警告音量の一部として
options-section-rwr-tones = レーダー警報音
rwr-tones-all = すべて
    .facts = 自機を照射するレーダー、自機を探知する各掃引、ミサイル発射
rwr-tones-detect = 探知のみ
    .facts =
        レーダーに探知されたとき1回（レーダーがしばらく離れた後は再び）
        およびミサイル発射
rwr-tones-launch = 発射のみ
    .facts = 自機に向けて発射されたミサイルのみ
rwr-tones-off = オフ
    .facts = レーダー警報音なし（ランプと表示器は引き続き警告）
options-section-cockpit = コックピット
options-low-pass = エンジンローパス
    .facts =
        機内ではエンジン音がこもって聞こえます（キャノピーとヘルメットが高音を遮ります）
        オフ：機外と同じ

## CONTROLS

options-section-bindings = 割り当て
options-keys-joystick = キーとジョイスティック
    .facts =
        マップのキー、ジョイスティックと HOTAS の軸とボタン
        次回以降の飛行に保存されます
options-section-head-tracking = ヘッド/アイトラッキング
options-head-tracking = ヘッド/アイトラッキング
    .facts =
        頭の動きでコックピット内の視点が回転・移動します（デスクトップ。VR ではヘッドセットが行います）
        トラッキングプログラム（OpenTrack...）：FreeTrack 出力、または UDP で 127.0.0.1 のポート 4242
        Home：現在の頭の位置を中心にします
options-tracking = トラッキング（{ $source }）
options-no-tracker = トラッカーが見つかりません：トラッキングプログラムを起動してください

## EFFECTS

options-section-effects = エフェクト
options-section-retro = レトロ
effect-shake = カメラの揺れ
    .facts =
        機関砲、近くの爆発、失速、接地、滑走路で視界が揺れます
        VR：コックピットが振動し、頭は動きません
effect-flashes = 戦闘の閃光
    .facts = 対空砲火の破裂、爆発、ミサイル発射が雲、地面、コックピットを一瞬照らします
effect-barrage = 夜間阻止射撃
    .facts =
        夜の通常戦：都市の火砲が空へ曳光弾を闇雲に撃ち、探照灯が空を掃きます
        都市上空の敵ドローンは探照灯に捕らえられ、火砲の射撃を引き付けます
        見た目だけ：弾幕は何にも当たりません
effect-cockpit-night = コックピットの夜間照明
    .facts =
        暗くなるとコックピットに薄暗い赤色灯 (ゴーグル使用時は青緑)
        警告が出ている間、マスターコーションがパネル上で琥珀色に点滅します
effect-nvg = 暗視ゴーグル
    .facts =
        強い光はハロとなって広がり、暗い所で画面の粒子が増えます
        デスクトップ：ゴーグルの丸い視野
effect-snow = 雪
    .facts = 氷点より上では嵐は雨の代わりに雪を降らせます：北の冬は低く、夏は高く
effect-mist = 地面の霧
    .facts =
        最新のシェーダー：夜明け、夕暮れ、夜に低地と海の上に霧がかかります
        砂漠では薄く、嵐では出ません
effect-cloud-shadows = 雲の影
    .facts = 最新のシェーダー：下の地面に落ちる雲の影
effect-ground-detail = 地面の細部
    .facts =
        最新のシェーダー：平らな地面の表面が近くで見える
        畑の作物の列、草地の草むら、コンクリートと舗装の粒
effect-city-lights = 街の灯り
    .facts =
        最新のシェーダー：夜の街が柔らかな灯りで輝き、中心ほど密になる
        高層ビルの窓に明かりがともる
effect-glass = ガラスの反射
    .facts =
        最新のシェーダー：ガラスに空が映る
        キャノピー、窓、ガラスのビルが太陽と月の光を反射する
effect-water = 水面の反射
    .facts =
        最新のシェーダー：海と湖の波に空が映る
        太陽と月の光が水面にきらめく
effect-ocean-waves = 海洋シミュレーション
    .facts =
        水面の反射と併用：外洋の波を風のスペクトルからシミュレート
        機体の周りに本物のうねり、艦船や空母がその上で揺れる
        オフ：Steam Frame やスマートフォン向けの軽いさざ波の海
effect-vapour = ベイパーとダウンウォッシュ
    .facts =
        急旋回で翼端からベイパーが引く
        低くホバリングするヘリコプターが砂ぼこりを、水上では飛沫を巻き上げる
        走行中の車両が後ろに砂ぼこりを上げる
effect-night-sky = 夜空
    .facts =
        ときどき流れ星、地平線近くの星がまたたきます
        ノールカップ上空のオーロラ
effect-vga = VGAカラー
    .facts = 現代的なシェーディングで再現する1989年の見た目：VGAカードのように色チャンネルあたり64段階、ディザリング付き
effect-scanlines = 走査線
    .facts = CRTモニターの走査線 (VRでは無効)

## LANGUAGE NAMES

lang-name-en = 英語
lang-name-bg = ブルガリア語
lang-name-zh-Hans = 中国語（簡体字）
lang-name-hr = クロアチア語
lang-name-cs = チェコ語
lang-name-da = デンマーク語
lang-name-nl = オランダ語
lang-name-et = エストニア語
lang-name-fi = フィンランド語
lang-name-fr = フランス語
lang-name-de = ドイツ語
lang-name-el = ギリシャ語
lang-name-hu = ハンガリー語
lang-name-id = インドネシア語
lang-name-ga = アイルランド語
lang-name-it = イタリア語
lang-name-ja = 日本語
lang-name-ko = 韓国語
lang-name-lv = ラトビア語
lang-name-lt = リトアニア語
lang-name-mt = マルタ語
lang-name-pl = ポーランド語
lang-name-pt-BR = ポルトガル語（ブラジル）
lang-name-ro = ルーマニア語
lang-name-ru = ロシア語
lang-name-sk = スロバキア語
lang-name-sl = スロベニア語
lang-name-es = スペイン語
lang-name-sv = スウェーデン語
lang-name-uk = ウクライナ語

## Browser build: offer to play in the browser's language (src/i18n.rs)

language-offer-question = ブラウザの言語：{ $language }。この言語でプレイしますか？
language-offer-yes = はい
language-offer-no = いいえ
language-offer-never = 今後は確認しない
detail-benchmark-lowered = このブラウザのフレームレートに合わせてグラフィックの詳細を下げました。オプション > グラフィックで再び上げられます。
effect-sunbeams = 太陽光線
    .facts = まばらな曇りの切れ間から差す日の光の筋
options-msaa-forced = ピクセル化 (レトロ) がオンの間はオフ
options-volume-radio = 無線
    .facts = 協力無線で話す他のプレイヤーと、音声ATC
options-volume-atc = ATC音声
    .facts = 音声ATCの呼び出し。無線音量の一部です
options-volume-music = 音楽
    .facts = メインメニューの音楽
options-radio-ducking = 無線時の音量低下
    .facts = 無線で声が流れている間 (ATC、僚機、他のプレイヤー)、エンジン音と効果音をどれだけ下げるか。0%: 下げない
options-section-atc = 音声ATC
options-section-radio = 協力無線
options-push-to-talk = プッシュトゥトーク
    .facts =
        送話キー (Caps Lock、またはPTTに割り当てたジョイスティック/VRボタン) を押し続けると全プレイヤーに聞こえます
        マイクは初めて押したときに開きます
options-mic-gain = マイクゲイン
    .facts =
        マイクの送信音量
        100%：そのまま
options-mic-default = デフォルト
options-voice-text-only = テキストのみ
options-voices-line = 管制官：{ $controllers }   パイロット：{ $pilot }
options-voices = 音声
    .facts =
        無線呼び出し用の音声をダウンロード、削除します
        管制官とあなた自身の音声を選び、試聴します
options-voices-web = ブラウザではテキストのみ
options-tracking-mouse = トラッキング中のマウス視点
    .facts =
        オン：マウス右ボタンで引き続きコックピット視点を回転・ズームします
        オフ：トラッカーが送信中、マウス右ボタンはコックピットで何もしません
options-atc = 航空管制
    .facts =
        友軍飛行場でのタワー、アプローチ、GCAの無線呼び出し
        離陸許可、オーバーヘッドブレーク、低い雲や視界不良でのトークダウン
        空母の着艦信号士官：ボールコール、修正指示、ウェーブオフ
options-pixel-block = ピクセル化のブロックサイズ
    .facts = ブロックの一辺、画面ピクセル単位 (2：細かい、5：粗い)
effect-pixelize = ピクセル化
    .facts =
        3D表示全体 (地形、機体、建物、コックピット) をMS-DOSパソコンのような大きな四角いピクセルで表示します。メニューとツールチップは鮮明なままです (VRでは不可)
        スライダーでブロックの大きさを画面ピクセルで設定します (2：細かい、5：粗い)
        オンの間はアンチエイリアスがオフになります
        その下の3つのスイッチで、HUD、コックピット、MFD、計器、ニーボードを鮮明なままにできます
effect-sharp-hud = HUDを除外
    .facts =
        ピクセル化：HUD (とヘルメット表示) はピクセル化された画面の上に鮮明に描かれ、細い記号も読み取れます
effect-sharp-cockpit = コックピットを除外 (ボタン、スイッチなど)
    .facts =
        ピクセル化：コックピットのパネル、フレーム、シート、ボタン、スイッチ、スティック、スロットルが鮮明に描かれます
effect-sharp-gauges = MFD、計器、ニーボードを除外
    .facts =
        ピクセル化：MFD、姿勢指示球、計器、ランプ、数値表示、ラベル、ニーボードが鮮明に描かれます
options-needs-pixelize = ピクセル化が必要
