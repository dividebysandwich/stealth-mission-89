# 膝板（src/kneeboard.rs，笔画字体）、任务规划器（src/planner.rs）和已保存的任务（src/savegame.rs）。
# 导航缩写（PT、BRG、ETA、PRI、SEC、KM、KT）保持英文。

kb-brief = 简报
kb-route = 航线
kb-map = 地图
kb-target = 目标
kb-keys = 按键
kb-tip = 地图、任务备注和按键指南
kb-geran-file = GERAN-2（档案）
kb-objectives = 任务目标
kb-primary = 主要：{ $objective }
kb-secondary = 次要：{ $objective }
kb-times = 时间
kb-deadline = 截止 { $time }
kb-now = 现在 { $time }
kb-conditions = 条件
kb-moon = 月亮：{ $phase }（照亮 { $lit }%）
kb-armament = 武器
kb-roe = 交战规则
kb-total = 总计 { $km } KM，速度 { $kt } KT
kb-fuel = 燃油 { $lbs } LBS
kb-preparing = 准备中...
kb-airborne = 已升空
kb-developing = 冲洗中...
kb-no-photo = 无照片
kb-airborne-target = 空中目标
kb-scroll-vr = 滚动：拇指摇杆，手指放在板上
kb-scroll-desktop = 滚动：鼠标滚轮置于板上

## 任务规划器

planner-low = 低空 500 FT
planner-medium = 中空 5000 FT
planner-high = 高空 20000 FT
planner-title = 任务规划器
planner-route = 航线
planner-base = 基地
planner-total = 总计 { $km } KM  { $time }
planner-briefed = 简报航线 { $km } km，燃油 { $fuel } lbs
planner-coverage = 雷达覆盖范围：
    .note = 雷达覆盖范围按 480kt 平飞、弹舱门关闭计算。
planner-detected = 航线上有 { $km } km 被探测到
planner-pulse = 脉冲雷达（P）探测距离
planner-doppler = 多普勒雷达（D）探测距离
planner-missile = 导弹射程
planner-uav = 敌方无人机
planner-edit-hint = 拖动航点可移动它，拖动地图可平移。点击航段可添加航点，右键可删除。滚轮：缩放。
planner-reset = 重置
planner-view-hint = 拖动地图可平移。滚轮：缩放。

## 已保存的任务

saves-title = 已保存的任务
saves-subtitle = 在飞行中通过菜单保存：保存任务
saves-none = 没有已保存的任务
saves-line = { $theater }  -  { $aircraft }  -  任务时间 { $time }
saves-saved-at = 保存于 { $date }
saves-unreadable = 存档无法读取（其他版本）
saves-delete =
    .facts = 删除此存档
saves-back =
    .facts = 返回单任务界面
