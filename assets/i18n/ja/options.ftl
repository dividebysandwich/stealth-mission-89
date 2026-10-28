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
options-tooltips = コックピットのツールチップ
    .facts = マウスまたは VR の指先の下にある、コックピットの操作部や表示の説明
options-section-vr-size = VRコックピットのサイズ
options-vr-size =
    .facts =
        ヘッドセット内のコックピット（と世界）のサイズ
        100%：実物大

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
options-auto-detail = 地形ディテール自動調整（VR）
    .facts =
        ヘッドセットでは、フレームが遅れている間は街、森、畑の描画距離が縮み、余裕があれば元に戻ります
        グラフィック設定は選んだままです
options-sun-moon = 太陽と月
    .facts =
        昼は空に太陽、夜は月と星
        月はミッションの月齢を示します

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

options-section-effects = Effects
options-section-retro = Retro
effect-shake = Camera shake
    .facts =
        The view shakes with the gun, nearby blasts, stall buffet, touchdowns and the runway
        VR: the cockpit vibrates, your head stays still
effect-flashes = Combat flashes
    .facts =
        Flak bursts, explosions and missile launches light the clouds, the ground and the cockpit for a moment
effect-barrage = Night barrage
    .facts =
        Conventional war at night: the towns' guns fire tracer blindly into the sky, searchlights sweep over them
        Only for the look: the barrage hits nothing, the searchlights find nothing
effect-cockpit-night = Cockpit night lighting
    .facts =
        A dim red flood light in the cockpit after dark (blue-green under the goggles)
        The master caution pulses amber over the panel while a warning is on
effect-nvg = Night vision goggles
    .facts =
        Bright lights bloom into halos, the picture grains more in the dark
        Desktop: the goggles' round field of view
effect-snow = Snow
    .facts =
        Above the freezing level storms snow instead of rain: low in a northern winter, high in summer
effect-mist = Ground mist
    .facts =
        Modern shaders: mist lies over the low ground and the sea at dawn, dusk and night
        Thin in the deserts, none in storms
effect-cloud-shadows = Cloud shadows
    .facts =
        Modern shaders: the clouds' shadows on the ground below
effect-vapour = Vapour and downwash
    .facts =
        Pulling hard, vapour trails from the wingtips
        A helicopter hovering low whips up dust, or spray over water
effect-night-sky = Night sky
    .facts =
        Shooting stars now and then, the stars low over the horizon twinkle
        The northern lights over the North Cape
effect-vga = VGA colours
    .facts =
        The 1989 look with modern shading: 64 steps a colour channel, as a VGA card's, dithered
effect-scanlines = Scanlines
    .facts =
        A CRT monitor's scanlines (not in VR)

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
