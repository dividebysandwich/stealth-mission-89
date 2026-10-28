# メインメニューとシングルミッション画面（src/menu.rs）。

## Theaters (stealth_host::theater::THEATERS)

theater-libya = リビア（1986）
    .facts =
        シドラ湾とリビア沿岸
        エルドラド・キャニオン作戦の時代
        ほとんどの日は砂漠の晴天
theater-persian_gulf = ペルシャ湾（1984）
    .facts =
        イラン・イラク戦争のタンカー戦争の時代
        洋上と沿岸の砂漠
        全戦域で最も天気が安定
theater-north_cape = ノールカップ
    .facts =
        北極圏のノルウェーとコラ半島
        ソ連北方艦隊の本拠地
        低い雲が頻繁に垂れ込める
theater-central_europe = 中央ヨーロッパ
    .facts =
        ドイツ上空の NATO 対ワルシャワ条約機構
        快晴は 15% の時間だけ
theater-vietnam = ベトナム
    .facts =
        ハノイとハイフォン地域
        ジャングルと山岳地形
        ミッションの 30% で雷雨
theater-middle_east = 中東
    .facts =
        東地中海とレバント
        ほぼ晴天
theater-desert_storm = 砂漠の嵐
    .facts =
        クウェートとイラク、1991年
        F-117A の実戦デビュー
        ほぼ砂漠の晴天
theater-cuba = キューバ
    .facts =
        カリブ海の島の戦域
        海上と陸上の目標
        熱帯：ミッションの 15% で嵐
theater-korea = 朝鮮半島
    .facts =
        DMZ より北の朝鮮半島
        山岳地形
        天気は変わりやすく、どんな空もありえます
theater-ukraine = ウクライナ（2026）
    .facts =
        東ウクライナ、クリミア、ロシア南部
        ハルキウからヘルソンまでの前線が燃えている
        広がる農地と、前線の廃墟の町

## Situations (rules of engagement)

roe-0 = 冷戦
    .facts =
        平時の秘密作戦
        発見されないことが最も重要
        哨戒中の敵機は少なめ
        正確な交戦規則はブリーフィングに記載
roe-1 = 限定戦争
    .facts =
        地域的な武力衝突
        敵の防空は警戒態勢
        冷戦より敵機が多い
roe-2 = 通常戦争
    .facts =
        全面戦争
        敵の全戦力が活動中
        冷戦より敵機が多い

## Mission kinds

kind-strike = 攻撃
    .facts =
        地上目標を破壊、撮影、または到達
        主目標と副目標
        空対地兵装
kind-air = 空対空
    .facts =
        敵機を迎撃
        輸送機、爆撃機、または戦闘機哨戒
        空対空ミサイル兵装
kind-strike-training = 攻撃訓練
    .facts =
        攻撃ミッションの練習
        兵装とナビゲーションを学ぶ
kind-air-training = 空戦訓練
    .facts =
        空対空ミッションの練習
        迎撃とミサイルを学ぶ
kind-drone-defense = ドローン防衛
    .facts =
        ウクライナ専用
        ゲラン攻撃ドローンの波が都市や原子力発電所に到達する前に阻止
        その後、発射拠点を爆撃

## Opponents

opponents-0 = 新兵
    .facts =
        経験の浅い敵クルー
        最も簡単な設定
opponents-1 = 通常
    .facts =
        平均的な敵クルー
        既定の設定
opponents-2 = ベテラン
    .facts =
        熟練した敵クルー
        艦船と哨戒が増加
opponents-3 = エリート
    .facts =
        最強の敵パイロットと SAM クルー
        最も難しい設定

## Weather

sky-random = ランダム
    .facts =
        戦域の気候から決定
        新しいミッションごとに変わります
        雲は両陣営の視線を遮ります
sky-clear = 快晴
    .facts =
        雲なし
        視程 40 km
        DOS時代の定番の空
sky-scattered = 散在雲
    .facts =
        積雲 20-45%、雲底 2500-6000 ft
        高空に薄い高層雲が出ることも
        視程 30 km
sky-broken = 多雲
    .facts =
        積雲 45-70%、雲底 2000-5000 ft
        上層に層積雲がよく出ます
        視程 25 km
sky-overcast = 曇天
    .facts =
        層雲 80-100%、雲底 1500-5000 ft
        下に積雲の断片が少し
        視程 20 km
sky-storm = 嵐
    .facts =
        積乱雲が 40000 ft までそびえる
        1000 ft から低い層雲と積雲
        視程 8 km
menu-random-cover = 雲量ランダム
    .facts =
        主雲層の雲量を天気とともに決定
        チェックを外すと自分で設定できます
        主雲層：曇天と嵐は層雲、それ以外は積雲
menu-cover-slider =
    .facts =
        主雲層が空を覆う割合
        快晴では効果なし

## Moon

moon-date = ミッション日付
    .facts =
        ブリーフィング日付の実際の月
        時刻スイッチ（モダン）が必要
moon-0 = 新月
    .facts =
        月明かりなし：灯火と星だけの暗闇
        敵パイロットは 2 km であなたを視認
moon-1 = 三日月
    .facts = 夕空に細い月、真夜中前に沈む
moon-2 = 上弦の月
    .facts = 半分が輝き、夕方に高く、真夜中ごろに沈む
moon-3 = 十三夜の月
    .facts = 明るく、午後から真夜中過ぎまで空にある
moon-4 = 満月
    .facts =
        一晩中空にあり、真夜中に最も高い
        敵パイロットは 4 km であなたを視認
moon-5 = 十八夜の月
    .facts = 明るく、夕方に昇り、朝まで空にある
moon-6 = 下弦の月
    .facts = 半分が輝き、真夜中ごろに昇る
moon-7 = 有明の月
    .facts = 夜明け前に昇る細い月

## Aircraft

aircraft-f117 =
    .facts =
        ステルス攻撃機
        亜音速、7 g
        モダンとクラシックのコックピット
aircraft-f16 =
    .facts =
        多用途戦闘機
        マッハ 2 級、9 g、素早いロール
        RWR 付きの F-16 コックピット
aircraft-tornado =
    .facts =
        低空侵攻攻撃機
        可変翼が速度に応じて後退
        モダンなコックピット、MFD 3 基
aircraft-a10 =
    .facts =
        近接航空支援機
        低速で頑丈、30 mm 機関砲
        A-10C コックピット、MFD 2 基
aircraft-uh60 =
    .facts =
        多用途ヘリコプター
        部隊を投入：目標の近くに着陸
        サイクリックとコレクティブ
aircraft-ah1z =
    .facts =
        攻撃ヘリコプター
        ターレット機関砲、ヘルファイア、AIM-9
        機関砲とヘルファイアはポッドに追従

## Sections and groups

section-theater = 戦域
section-opponents = 敵の練度
section-situation = 状況
section-mission = ミッション
section-moon = 月
section-start-time = 開始時刻
section-weather = 天候
section-realism = リアリズム
section-tacview = Tacview
section-mission-summary = ミッション概要
section-briefing = ブリーフィング
group-mission = ミッション
group-conditions = 条件
group-vehicle = 機体
group-simulation = シミュレーション

## Options of the mission

menu-start-random = ランダム
    .facts =
        クラシックモードと同じく、新しいミッションごとに決定
        昼は 06:00-08:00、夜は 18:00-20:00
        チェックを外すと時刻を選べます
menu-start-slider =
    .facts =
        離陸時刻
        夜は 18:00 から 06:00
        新規ミッションでも保持されます
menu-realistic-weapons = リアルな兵装
    .facts =
        実機の制限
        兵装ベイは 4 ではなく 2
        AIM-9 と AIM-120 ミサイルなし
        空母での離着艦なし
menu-crash-protection = クラッシュ防止
    .facts =
        地面の近くで自動的に引き起こします（ギアアップ時）
        接地や衝突で致命傷にならない
        オフ：地面に激突することがあります
menu-tacview-export = エクスポート
    .facts =
        各飛行を Tacview 記録（.acmi）として保存
        ゲームのファイル用フォルダ内の tacview
menu-tacview-real-time = リアルタイムテレメトリ
    .facts =
        飛行中に Tacview へ飛行データを送信
        Tacview で：Record > Real-Time Telemetry
        アドレス 127.0.0.1、ポート 42674（--tacview-address）
switch-game = ゲーム
switch-real = リアル
menu-damage = ダメージ
    .facts =
        兵装のダメージ
        ゲーム：クラシックなダメージ。被弾は蓄積しても機体は墜ちません
        リアル：ミサイルの命中で機体が破壊されることがあり（脱出！）、より多くの系統が損傷し、火災がよく起きて、燃え尽きるか機体が爆発します
        リアル：損傷した操縦舵面で機体のトリムが狂ったり、振動したりします
menu-guidance = 誘導
    .facts =
        ミサイルの誘導
        クラシック：ミサイルは目標へまっすぐ飛びます
        リアル：衝突点への比例航法。モーターは燃え尽き、ミサイルは滑空して旋回のたびに速度を失います（煙はモーター燃焼中のみ）
        リアル：SAM は遠い目標に対して打ち上げ軌道を取り、AIM-120 は 3 秒間直進します（引き起こしてロブ発射）
menu-air-defence = 防空
    .facts =
        低空の防空網
        クラシック：SAM サイトと戦闘機のみ
        リアル：レーダー照準の ZSU-23-4（そのレーダーは SAM と同様にあなたを捉えます）と S-60 高射砲が飛行場と目標を守り、町の MANPADS チームは見えたものに SA-14 を撃ちます
        防御された場所の低空飛行は危険になります。爆弾と機関砲でこれらの砲を破壊できます

## Main screen

menu-title = STEALTH MISSION 89
menu-subtitle = ミッション計画システム
menu-single-mission = シングルミッション
    .facts =
        選んだ戦域での 1 ミッション
        オプション、ルート、兵装を選びます
menu-career = キャリア
    .facts =
        パイロット記録：階級、勲章、出撃
        勤務期間：1 つの戦域で 12 回の出撃
        破壊したものは破壊されたまま
menu-campaign = ダイナミックキャンペーン
    .facts =
        リアルタイムのダイナミックキャンペーン
        敵の都市と飛行場を奪取
        進行中の戦況からミッションを生成
menu-multiplayer = マルチプレイヤー
    .facts = ダイナミックキャンペーンでの協力プレイ
    .join = 公開リストのサーバーに参加
    .host = サーバーに参加、またはキャンペーンをホストして飛ぶ
menu-options = オプション
    .facts =
        ゲーム：コックピット、HUD、アビオニクスのスタイル、オートトリム
        グラフィック、音量、キーとジョイスティックの割り当て
        次回以降の飛行に保存されます
    .short = ゲーム、グラフィック、サウンド、操作
menu-download = ダウンロード
    .facts =
        Windows、Linux、macOS 向けのデスクトップ版
        VR とマルチプレイヤー対応
        Steam Deck と Steam Frame：Linux AppImage
web-play = ブラウザでプレイ
    .facts = ここですぐに、インストール不要
web-download-platforms = Windows、Linux、macOS、Steam Deck、Steam Frame
web-download-version = ダウンロード版
web-gives-vr = VR
    .why = OpenXR 対応のヘッドセットと Steam Frame で VR 飛行
web-gives-hotas = HOTAS
    .why = ゲームパッド、ジョイスティック、スロットル、ペダルを完全サポート
web-gives-multiplayer = より良いマルチプレイヤー
    .why = サーバーへの UDP 直接接続でラグが少ない
web-gives-server = 自分のサーバーをホスト
    .why = 協力キャンペーンをホスト、または専用サーバーを運用
web-gives-head-tracking = ヘッドトラッキング
    .why = OpenTrack 経由で TrackIR 型のトラッカーに対応
web-gives-tacview = Tacview
    .why = 飛行の記録とリアルタイムテレメトリ
web-gives-keep = 手元に残せます
    .why = ウェブサイトが落ちても、自分のコピーが残ります
menu-discord = DISCORD
    .facts =
        一緒に飛び、バグを報告し、ミッションを共有
        開発の最新情報
menu-quit = 終了
    .facts = デスクトップに戻ります
menu-version = バージョン { $version }

## Single mission screen

menu-career-sortie = キャリア出撃
menu-tour-sortie = { $pilot }  //  勤務期間の出撃 { $sortie } / { $length }
menu-msn = ミッション計画システム  //  MSN { $seed }
menu-no-targets = この勤務期間に残っている目標はありません
menu-plan-route = ルート計画
    .facts =
        敵のレーダーと SAM の範囲を表示した戦域マップ
        航法ポイントの追加、移動、削除
        目標と着陸基地は固定
menu-loadout = 兵装
    .facts =
        各ベイまたはパイロンの兵装を選びます
        ブリーフィングされた兵装から始まります
menu-generating = 生成中...
menu-back-records = パイロット記録に戻る
menu-back-main = メインメニューに戻る
menu-new-sortie = 新規出撃
menu-new-mission = 新規ミッション
    .facts =
        異なる目標を生成します
        すべてのオプションと開始時刻は保持されます
menu-load = ロード
    .facts =
        飛行中に保存したミッション（メニュー：ミッションを保存）
        保存した地点から続けて飛びます
menu-fly = 出撃
    .facts = ブリーフィングされたミッションで離陸します
menu-fly-vr = VRで出撃
    .facts =
        ゲームを VR ヘッドセットで再起動します
        このミッションをそのまま飛びます
    .no-mission = 先にミッションを生成してください
    .no-runtime = 有効な OpenXR ランタイム（Monado、WiVRn、SteamVR）がありません
menu-vr-failed = VR で起動できませんでした：{ $error }

## Mission summary

summary-theater = 戦域
summary-situation = 状況
summary-mission = ミッション
summary-opponents = 敵の練度
summary-time = 時刻
summary-weather = 天候
summary-fuel = 燃料
summary-night = 夜
summary-day = 昼
summary-time-value = { $daytime }  出発 { $depart }
summary-weather-value = { $sky }  視程 { $vis } km
summary-fuel-value = { $fuel } lbs
summary-armament = 兵装：{ $stores }
summary-intelligence = 情報
