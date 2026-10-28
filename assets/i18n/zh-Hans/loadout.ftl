# 武器配置编辑器（src/loadout.rs）。武器名称是真实名称，不翻译。

loadout-none = 无
loadout-briefed = 简报配置
    .facts = 任务简报规定的外挂
loadout-clear = 清空
    .facts = 所有挂点都清空
loadout-done = 完成
loadout-no-hardpoints = 没有挂点
loadout-no-stores = 这架飞机不携带外挂
loadout-fantasy = 真实飞机上没有
loadout-empty = 空
loadout-out-of-stock = 本次任务无库存
loadout-mirror = 两侧相同
    .facts = 同时装载 { $station }
loadout-stores = 外挂
loadout-needs-equipment = 任务需要其摄像机或特殊设备

## 各武器的用途（武器类型 0 至 20）

# AIM-9M Sidewinder
weapon-role-0 = 空对空，红外制导，近程
# AIM-120A AMRAAM
weapon-role-1 = 空对空，雷达制导，中程
# AGM-88A HARM
weapon-role-2 = 反辐射导弹：打击雷达和 SAM 阵地
# Penguin 3 ASM
weapon-role-3 = 反舰导弹
# AGM-84A Harpoon
weapon-role-4 = 远程反舰导弹
# AGM-65 Maverick
weapon-role-5 = 电视制导导弹：打击车辆、掩体、建筑
# GBU-12 Paveway
weapon-role-6 = 激光制导炸弹：打击坚固目标
# Mk 20 Rockeye
weapon-role-7 = 集束炸弹：打击软目标和面目标
# Durandal
weapon-role-8 = 破坏跑道的炸弹
# Mk 82 LDGP
weapon-role-9 = 500 lb 自由落体炸弹
# Mk 82 Snakeye
weapon-role-10 = 500 lb 减速炸弹，用于低空投放
# Mk 20 Rockeye II
weapon-role-11 = 集束炸弹：打击装甲和面目标
# Mk 122 Fireye
weapon-role-12 = 燃烧弹
# CBU-72 FAE
weapon-role-13 = 燃料空气炸药：打击面目标
# Mk 35 incendiary
weapon-role-14 = 燃烧集束炸弹
# ISCB-1 mines
weapon-role-15 = 区域封锁地雷
# IR/optical camera
weapon-role-16 = 拍摄目标的侦察照片
# 1900 lb fuel tank
weapon-role-17 = 额外燃油，投放时转移
# Special cargo
weapon-role-18 = 任务所需的补给或货物
# 20 mm cannon
weapon-role-19 = 机炮（不是外挂）
# AGM-114 Hellfire
weapon-role-20 = 激光制导反装甲导弹：飞向瞄准吊舱所指之处

## 各武器的使用方式（射程为 HUD 上的 km，高度为 ft）

employment-0 = 射程：1-12 km，迎头时更远
employment-1 = 射程：1-38 km，迎头时更远
employment-2 =
    射程：9-50 km
    发射高度：5,000-20,000 ft
employment-3 =
    射程：6-14 km（经典制导：13-14 km）
    发射高度：100-500 ft
employment-4 =
    射程：6-41 km
    发射高度：2,000-5,000 ft
employment-5 =
    射程：5-10 km
    发射高度：2,000-5,000 ft
employment-6 =
    射程：1.5 km 至高度的 2.5 倍，LOCK 稳定（15,000 ft 时 14 km，25,000 ft 时 18 km）
    投放高度：15,000-25,000 ft
employment-8 =
    射程：距跑道前 0.9-1.3 km
    投放高度：200-500 ft
employment-9 =
    射程：距目标前 5-11 km
    投放高度：3,000-20,000 ft
employment-12 =
    射程：距目标前 0.9-2.2 km
    投放高度：100-500 ft
employment-16 = 射程：2-8 km 处拍摄（越近，对手越多），机头对准目标
employment-18 = 射程：在出现“Drop supplies now”时投放，距投放区 4 km 以内
employment-20 =
    射程：4-8 km
    发射高度：2,000-3,000 ft
employment-guided-cluster =
    射程：1.5 km 至高度的 2.5 倍，LOCK 稳定（3,000 ft 时 3.7 km，5,000 ft 时 6 km）
    投放高度：3,000-5,000 ft
employment-medium-retarded =
    射程：距目标前 1.1-1.6 km
    投放高度：300-1,000 ft
employment-low-retarded =
    射程：距目标前 0.9-1.3 km
    投放高度：200-500 ft

## 挂点

hardpoint-left-bay = 左弹舱
hardpoint-right-bay = 右弹舱
hardpoint-left-wing-bay = 左翼弹舱
hardpoint-right-wing-bay = 右翼弹舱
hardpoint-station-1-left-wingtip = 1 号挂点，左翼尖
hardpoint-station-2-left-outer-wing = 2 号挂点，左外翼
hardpoint-station-3-left-wing = 3 号挂点，左翼
hardpoint-station-4-left-inner-wing = 4 号挂点，左内翼
hardpoint-station-5-centreline = 5 号挂点，中线
hardpoint-station-6-right-inner-wing = 6 号挂点，右内翼
hardpoint-station-7-right-wing = 7 号挂点，右翼
hardpoint-station-8-right-outer-wing = 8 号挂点，右外翼
hardpoint-station-9-right-wingtip = 9 号挂点，右翼尖
hardpoint-station-1-left-wingtip-rail = 1 号挂点，左翼尖导轨
hardpoint-station-2-left-outer-pylon = 2 号挂点，左外侧挂架
hardpoint-station-3-left-inner-pylon = 3 号挂点，左内侧挂架
hardpoint-station-4-right-inner-pylon = 4 号挂点，右内侧挂架
hardpoint-station-5-right-outer-pylon = 5 号挂点，右外侧挂架
hardpoint-station-6-right-wingtip-rail = 6 号挂点，右翼尖导轨
hardpoint-tornado-left-wing-rail = 左翼，Sidewinder 导轨
hardpoint-tornado-left-wing-pylon = 左翼挂架
hardpoint-tornado-left-shoulder-forward = 左肩，前
hardpoint-tornado-left-shoulder-aft = 左肩，后
hardpoint-tornado-right-shoulder-aft = 右肩，后
hardpoint-tornado-right-shoulder-forward = 右肩，前
hardpoint-tornado-right-wing-pylon = 右翼挂架
hardpoint-tornado-right-wing-rail = 右翼，Sidewinder 导轨
hardpoint-a10-1-left-outer-wing = 1 号挂点，左外翼
hardpoint-a10-2-left-wing = 2 号挂点，左翼
hardpoint-a10-3-left-wing = 3 号挂点，左翼
hardpoint-a10-4-left-centre-wing = 4 号挂点，左中翼
hardpoint-a10-5-left-fuselage = 5 号挂点，左机身
hardpoint-a10-6-centreline = 6 号挂点，中线
hardpoint-a10-7-right-fuselage = 7 号挂点，右机身
hardpoint-a10-8-right-centre-wing = 8 号挂点，右中翼
hardpoint-a10-9-right-wing = 9 号挂点，右翼
hardpoint-a10-10-right-wing = 10 号挂点，右翼
hardpoint-a10-11-right-outer-wing = 11 号挂点，右外翼
weapon-role-21 = Air-to-air, infrared homing, very short range
employment-21 = Range: 1-6 km, farther head-on
