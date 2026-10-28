# 兵装エディター（src/loadout.rs）。兵装名は実際の名称のままで翻訳しません。

loadout-none = なし
loadout-briefed = ブリーフィング通り
    .facts = ミッションがブリーフィングされた時の兵装
loadout-clear = クリア
    .facts = すべてのハードポイントを空にします
loadout-done = 完了
loadout-no-hardpoints = ハードポイントなし
loadout-no-stores = この機体は兵装を搭載しません
loadout-fantasy = 実機には搭載されません
loadout-empty = 空
loadout-out-of-stock = このミッションでは在庫なし
loadout-mirror = 左右同じ
    .facts = { $station } にも搭載します
loadout-stores = 兵装
loadout-needs-equipment = ミッションにはカメラまたは特殊装備が必要です

## What each weapon is for (weapon types 0 to 20)

# AIM-9M Sidewinder
weapon-role-0 = 空対空、赤外線ホーミング、短射程
# AIM-120A AMRAAM
weapon-role-1 = 空対空、レーダーホーミング、中射程
# AGM-88A HARM
weapon-role-2 = 対レーダーミサイル：レーダーと SAM サイト
# Penguin 3 ASM
weapon-role-3 = 対艦ミサイル
# AGM-84A Harpoon
weapon-role-4 = 長射程対艦ミサイル
# AGM-65 Maverick
weapon-role-5 = TV誘導ミサイル：車両、掩蔽壕、建物
# GBU-12 Paveway
weapon-role-6 = レーザー誘導爆弾：堅固な目標
# Mk 20 Rockeye
weapon-role-7 = クラスター爆弾：軟目標と面目標
# Durandal
weapon-role-8 = 滑走路破壊爆弾
# Mk 82 LDGP
weapon-role-9 = 500 lb 自由落下爆弾
# Mk 82 Snakeye
weapon-role-10 = 低空投下用の 500 lb 減速爆弾
# Mk 20 Rockeye II
weapon-role-11 = クラスター爆弾：装甲目標と面目標
# Mk 122 Fireye
weapon-role-12 = 焼夷弾
# CBU-72 FAE
weapon-role-13 = 燃料気化爆弾：面目標
# Mk 35 incendiary
weapon-role-14 = 焼夷クラスター爆弾
# ISCB-1 mines
weapon-role-15 = 地域拒否用の地雷
# IR/optical camera
weapon-role-16 = 目標の偵察写真
# 1900 lb fuel tank
weapon-role-17 = 増槽、投下時に燃料を移送
# Special cargo
weapon-role-18 = ミッション用の物資または貨物
# 20 mm cannon
weapon-role-19 = 機関砲（兵装ではありません）
# AGM-114 Hellfire
weapon-role-20 = レーザー誘導対戦車ミサイル：ターゲティングポッドの向く先へ飛ぶ

## How each weapon is used (ranges in the HUD's km, heights in feet)

employment-0 = 射程：1-12 km、正面ならさらに遠く
employment-1 = 射程：1-38 km、正面ならさらに遠く
employment-2 =
    射程：9-50 km
    発射：5,000-20,000 ft
employment-3 =
    射程：6-14 km（クラシック誘導：13-14 km）
    発射：100-500 ft
employment-4 =
    射程：6-41 km
    発射：2,000-5,000 ft
employment-5 =
    射程：5-10 km
    発射：2,000-5,000 ft
employment-6 =
    射程：1.5 km から高度の 2.5 倍まで、LOCK 点灯（15,000 ft から 14 km、25,000 ft から 18 km）
    投下：15,000-25,000 ft
employment-8 =
    射程：滑走路の手前 0.9-1.3 km
    投下：200-500 ft
employment-9 =
    射程：目標の手前 5-11 km
    投下：3,000-20,000 ft
employment-12 =
    射程：目標の手前 0.9-2.2 km
    投下：100-500 ft
employment-16 = 射程：2-8 km から撮影（近いほど敵が増える）、機首を目標に向ける
employment-18 = 射程：「Drop supplies now」で投下、投下地点から 4 km 以内
employment-20 =
    射程：4-8 km
    発射：2,000-3,000 ft
employment-guided-cluster =
    射程：1.5 km から高度の 2.5 倍まで、LOCK 点灯（3,000 ft から 3.7 km、5,000 ft から 6 km）
    投下：3,000-5,000 ft
employment-medium-retarded =
    射程：目標の手前 1.1-1.6 km
    投下：300-1,000 ft
employment-low-retarded =
    射程：目標の手前 0.9-1.3 km
    投下：200-500 ft

## Hardpoints

hardpoint-left-bay = 左ベイ
hardpoint-right-bay = 右ベイ
hardpoint-left-wing-bay = 左翼ベイ
hardpoint-right-wing-bay = 右翼ベイ
hardpoint-station-1-left-wingtip = ステーション 1、左翼端
hardpoint-station-2-left-outer-wing = ステーション 2、左翼外側
hardpoint-station-3-left-wing = ステーション 3、左翼
hardpoint-station-4-left-inner-wing = ステーション 4、左翼内側
hardpoint-station-5-centreline = ステーション 5、中心線
hardpoint-station-6-right-inner-wing = ステーション 6、右翼内側
hardpoint-station-7-right-wing = ステーション 7、右翼
hardpoint-station-8-right-outer-wing = ステーション 8、右翼外側
hardpoint-station-9-right-wingtip = ステーション 9、右翼端
hardpoint-station-1-left-wingtip-rail = ステーション 1、左翼端レール
hardpoint-station-2-left-outer-pylon = ステーション 2、左外側パイロン
hardpoint-station-3-left-inner-pylon = ステーション 3、左内側パイロン
hardpoint-station-4-right-inner-pylon = ステーション 4、右内側パイロン
hardpoint-station-5-right-outer-pylon = ステーション 5、右外側パイロン
hardpoint-station-6-right-wingtip-rail = ステーション 6、右翼端レール
hardpoint-tornado-left-wing-rail = 左翼、サイドワインダーレール
hardpoint-tornado-left-wing-pylon = 左翼パイロン
hardpoint-tornado-left-shoulder-forward = 左ショルダー、前方
hardpoint-tornado-left-shoulder-aft = 左ショルダー、後方
hardpoint-tornado-right-shoulder-aft = 右ショルダー、後方
hardpoint-tornado-right-shoulder-forward = 右ショルダー、前方
hardpoint-tornado-right-wing-pylon = 右翼パイロン
hardpoint-tornado-right-wing-rail = 右翼、サイドワインダーレール
hardpoint-a10-1-left-outer-wing = ステーション 1、左翼外側
hardpoint-a10-2-left-wing = ステーション 2、左翼
hardpoint-a10-3-left-wing = ステーション 3、左翼
hardpoint-a10-4-left-centre-wing = ステーション 4、左翼中央
hardpoint-a10-5-left-fuselage = ステーション 5、左胴体
hardpoint-a10-6-centreline = ステーション 6、中心線
hardpoint-a10-7-right-fuselage = ステーション 7、右胴体
hardpoint-a10-8-right-centre-wing = ステーション 8、右翼中央
hardpoint-a10-9-right-wing = ステーション 9、右翼
hardpoint-a10-10-right-wing = ステーション 10、右翼
hardpoint-a10-11-right-outer-wing = ステーション 11、右翼外側
weapon-role-21 = Air-to-air, infrared homing, very short range
employment-21 = Range: 1-6 km, farther head-on
