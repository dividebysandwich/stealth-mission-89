# キャンペーンウィンドウ（src/campaign/menu.rs）、マルチプレイヤーウィンドウ（src/multiplayer_menu.rs）、
# 戦況図の凡例とライブビュー（src/campaign/picture.rs、unit_view.rs）。

delete = 削除
delete-campaign = このキャンペーンを削除しますか？
delete-save = このセーブを削除しますか？
delete-permanent = 削除は元に戻せません

## キャンペーンウィンドウ

cm-group-campaign = キャンペーン
cm-weather-dynamic = 変動
    .facts =
        天候は空の時計で10〜22時間ごとに変化します
        雲は両陣営の視線を遮ります
cm-weather-clear = 快晴
    .facts = キャンペーン中は雲がありません
cm-section-day-night = 昼と夜
cm-day-real = 実時間
    .facts = 1日は24時間です
cm-day-2h = 1日 = 2時間
    .facts = 1日は2時間です
cm-day-1h = 1日 = 1時間
    .facts = 1日は1時間です
cm-day-30min = 1日 = 30分
    .facts = 1日は30分です
cm-conditions-note = 条件は新規キャンペーンに適用されます
cm-rules-note = セーブ済みのキャンペーンは、開始時のルールを保持します
cm-about =
    戦争はプレイ中、メニューを開いている間もリアルタイムで進行します。
    ボードの任務を飛びましょう：偵察、SEAD、CAP、CAS、対艦、攻撃、護衛。
    ヘリコプターによる降下作戦で、敵の都市と飛行場を奪取できます。
    あなたが戦死したり捕まったりすることはありません。出撃後はボードに戻ります。
cm-section-new = 新規キャンペーン
cm-new-line = { $theater }  -  { $situation }  -  敵 { $opponents }
cm-start = 開始
cm-section-continue = 続きから
cm-no-saves = セーブ済みのキャンペーンはありません
cm-played = プレイ時間 { $hours }時間
cm-save-gone = セーブが見つかりません
cm-save-unreadable = セーブを読み込めませんでした：{ $error }
cm-save-undeletable = セーブを削除できませんでした：{ $error }
cm-in-vr = VRで
cm-delete =
    .facts = このセーブ済みキャンペーンを削除

## 戦況図

pic-friendly-aircraft = 友軍機
pic-friendly-helicopter = 友軍ヘリコプター
pic-enemy-fighter = 敵戦闘機（レーダー上）
pic-sam-known = SAM陣地（確認済み）
pic-guns-known = 火砲 / MANPADS（確認済み）
pic-armour = 戦車 / 歩兵戦闘車（敵：視認時）
pic-patriot-reach = パトリオットの射程
pic-gepard-stinger = ゲパルト / スティンガー
pic-missile-range = SAMミサイルの射程
pic-engagement-range = 交戦距離
pic-reconned = 偵察済み
pic-you = あなた
pic-site = { $place } 陣地
uv-live = { $unit } - ライブ
uv-contact-lost = 接触喪失
uv-hint = ドラッグ：旋回  ホイール：ズーム

## マルチプレイヤー

mp-disconnected = 切断されました：{ $why }
mp-connecting = { $address } に接続中...
mp-joining = { $address } に接続しました。参加中...
mp-browser-public-only = ブラウザから参加できるのは公開リストのサーバーのみです
mp-title-host = マルチプレイヤー：ゲームをホスト
mp-subtitle = ダイナミックキャンペーンでの協力プレイ：一緒に戦争を戦おう
mp-your-name = あなたの名前
    .hint = パイロット名
mp-join-address = アドレスで参加
    .hint = アドレスまたは名前[:ポート]
mp-connect = 接続
mp-server-name = サーバー名
mp-udp-port = UDPポート
mp-address-note = 他のプレイヤーはこのコンピューターのアドレスで参加します。インターネット越しにプレイするには、ルーターでUDPポートを開放してください
mp-list-public = 公開サーバーリストに掲載（他のプレイヤーがサーバーリストでゲームを見つけられます）
mp-rules = 全員共通のルール
mp-time-of-day = 時刻（月明かりの夜）
mp-realistic-weapons = リアルなF-117A兵装
mp-friendly-fire = 友軍誤射
mp-day-night = 昼と夜：
mp-weather = 天候：
mp-host-new = 新規ホスト
mp-host-saved = セーブ済みキャンペーンをホスト
mp-host = ホスト
mp-host-game = ゲームをホスト
mp-port-error = UDPポート { $port }：{ $error }
mp-default-server = { $pilot } のキャンペーン
mp-public-servers = 公開サーバー
mp-refresh = 更新
mp-server-count = サーバー{ $servers }台、パイロット{ $pilots }名
mp-list-error = サーバーリストに接続できません：{ $error }
mp-no-servers = 現在、公開サーバーはありません
mp-col-server = サーバー
mp-col-pilots = パイロット
mp-col-war = 戦争
mp-dedicated = 専用
mp-other-version = 別バージョン
mp-join = 参加
