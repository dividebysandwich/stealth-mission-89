# 战役的作战界面（src/campaign/ops.rs）。

ops-thunderstorms = 雷暴
ops-wx-short = 天气 { $now }，{ $at } 转 { $next }
ops-wx-line = { $weather } { $at } 转为 { $next }。
ops-clock = 第 { $day } 天  { $time }
ops-landed-at = 已着陆于 { $base }
ops-ground-crew = 地勤已就绪
ops-rearm = 重新挂弹并加油
ops-end-sortie = 结束出击
ops-on-server = 服务器：{ $server }
ops-serving = 主持中
ops-tab-missions = 任务
ops-tab-map = 战争地图
ops-tab-air-defence = 防空
ops-tab-forces = 兵力
ops-tab-players = 玩家
ops-tab-debrief = 讲评
ops-tab-log = 日志
ops-tab-chat = 聊天
ops-blue = 蓝方 { $count }
ops-red = 红方 { $count }
ops-war-over = 战争已结束
ops-running = 战争模拟运行中
ops-save-exit = 保存并退出
ops-disconnect = 断开连接
ops-leave-local = 保存并离开战役？
    .text = 战争会停止，直到你继续。空中的飞机将损失；进行中的投送将从其基地重新开始。
ops-leave-host = 保存并停止主持战役？
    .text = 已加入的玩家将被断开连接，战争会停止，直到你继续。空中的飞机将损失；进行中的投送将从其基地重新开始。
ops-leave-client = 断开与服务器的连接？
    .text = 战役在服务器上继续进行；此处不会保存任何内容。你的出击将结束。
ops-unit-destroyed = { $unit } 已被摧毁：返回作战界面

## 任务

ops-section-missions = 任务
ops-threat = 威胁 { $threat }
ops-no-missions = 没有任务：前线平静
ops-section-underway = 投送和空袭
ops-enemy-liftoff = 敌方 { $place }：{ $mins } 分钟后起飞
ops-liftoff = { $place }：{ $mins } 分钟后起飞
ops-enemy-helis = 敌方 { $place }：{ $count } 架直升机正在逼近
ops-helis = { $place }：{ $count } 架直升机在途中
ops-raid-sead = { $place } 的 AI SEAD
ops-raid-drones = { $place } 的无人机
ops-raid = 空袭 { $place } 机场
ops-raid-launch = { $raid }：{ $mins } 分钟后发射
ops-raid-on-way = { $raid }：{ $count } 在途中
ops-resupply-ai = 为 { $unit } 补给：UH-60 在途中
ops-awacs = 敌方 AWACS 还将在空中停留 { $mins } 分钟
ops-down = 坠落：{ $pilot } - 须在 { $mins } 分钟内救援
ops-aboard = 已上机：{ $pilot } - 正在返程
ops-none-underway = 无进行中的项目
ops-ai-helicopter = 若没有 UH-60 飞行员接手，将由 AI 直升机在 { $time } 后执行
ops-defences = 防御：{ $sams } 个 SAM 阵地，威胁 { $threat }
ops-recon-intel = 侦察情报
ops-order-insertion = 下令投送
    .facts = 驾驶护航机为其提供掩护，或驾驶 UH-60 带领它们
ops-from = 出发基地
ops-planned-by-group = 由小组规划
ops-stores = { $aircraft } - 挂载：{ $stores }
ops-fly-uh60 = 驾驶 UH-60
ops-accept = 接受任务
ops-planning = 规划中：{ $pilots }
ops-flying = 飞行中：{ $pilots }
ops-package = 与 { $kinds } 组成 package：在 { $mins } 分钟内于此完成两种任务，每种 +{ $bonus } 分
ops-sead-on-way = AI SEAD 在途中
ops-request-sead = 请求 AI SEAD（物资 { $cost }，飞机 { $aircraft } 架）
    .facts = 友军攻击机会在你之前攻击此处的一个 SAM 阵地
    .disabled = 物资或地面飞机不足（见兵力）

## 规则、物资、兵力

ops-rule-damage = 真实损伤
ops-rule-guidance = 真实制导
ops-rule-no-crashes = 无坠毁
ops-rule-time = 连续时间
ops-rule-weapons = 真实的 F-117A 武器
ops-rule-friendly-fire = 友军误伤
ops-rule-classic = 经典
ops-materiel = 物资
ops-materiel-blue = 蓝方 { $materiel }（+{ $income }/h）
ops-materiel-red = 红方 { $materiel }（+{ $income }/h）
ops-friendly = 友军
ops-enemy = 敌军
ops-section-forces = 兵力
ops-places-held = 控制的地点
ops-materiel-row = 物资
ops-income = 每小时收入
ops-income-from = 来源
ops-income-sources = { $base } + { $places } 个地点 x { $per_place } + { $cities } 座城市 x { $per_city }
ops-fighters-ground = 地面上的战斗机
ops-helicopters-ground = 地面上的直升机
ops-in-air = 空中
ops-in-air-value = 战斗机 { $fighters }，攻击机 { $strike }，直升机 { $helicopters }
ops-since-start = 自开始以来
ops-earned = 获得
ops-repairs = 修复
ops-operations = 出击、空袭、投送
ops-new-aircraft = 新飞机
ops-lost = 损失（城市、飞行员）
ops-section-how = 运作方式
ops-how =
    每一方每小时获得物资：基础数额，加上控制的每个地点，以及其中每座完好的城市所带来的额外数额。轰炸敌方城市（遮断）可削减其收入。
    AI 的一切行动都要付费：修复、每次战斗机出击、空袭和它自己的投送。物资不足时，其损伤得不到修复，飞机也留在基地。
    AI 只使用它现有的飞机；被击落就没有了，着陆后可再次起飞。只要物资够用，每一方每小时都会购买新飞机。
    你不会缺乏物资：你的飞机、你的基地（无论是否被轰炸）以及你自己下令投送的直升机始终可用。
ops-section-prices = 价格
ops-price-airfield = 修复机场
ops-price-sam = 修复 SAM 阵地
ops-price-city = 重建城市
ops-price-ship = 修复舰船
ops-price-gun = 火炮或 MANPADS 小组
ops-price-sortie = 战斗机出击
ops-price-raid = 空袭
ops-price-insertion = AI 投送
ops-price-fighter = 新战斗机
ops-price-helicopter = 新直升机
ops-price-city-lost = 城市被毁（损失）
ops-price-captured = 飞行员被俘（损失）

## 战争地图

ops-click-to-watch = { $unit } - 点击观看
ops-section-legend = 图例
ops-legend-friendly-airfield = 友军机场
ops-legend-enemy-airfield = 敌方机场
ops-legend-city = 友军 / 敌方城市
ops-legend-airfield = 友方 / 敌方机场
ops-legend-insertion = 机降航线
ops-watch-hint = 点击一个单位可实时观看
ops-map-hint = 滚轮：缩放，拖动：平移
ops-patriot = Patriot 连
ops-drone-center = 无人机指挥所
ops-manned-by = 操作者：{ $pilot }
ops-manned-by-other = 由其他玩家操作
ops-control = 控制
    .facts = 接管其控制台

## 讲评、结束、日志、聊天

ops-no-sortie = 尚未执行任何出击
ops-section-sortie = 出击
ops-achieved = 目标已达成
ops-not-achieved = 目标未达成
ops-waiting-raid = 等待空袭
ops-waiting-insertion = 等待投送
ops-flight-time = 飞行时间 { $mins } 分钟
ops-kills = 击毁：飞机 { $air } 架，地面 { $ground } 个
ops-points = { $points } 分
ops-points-bonus = { $points } 分（package 奖励 +{ $bonus }）
ops-campaign-points = { $rank } - 战役得分 { $points }
ops-campaign-record = 战役：{ $sorties } 次出击，达成 { $won } 个目标
ops-section-meanwhile = 与此同时
ops-victory = 胜利
    .text = 敌军已不再拥有任何机场：战区归我方所有。
ops-defeat = 失败
    .text = 已不剩任何友军机场：战区已失守。
ops-end-day = 第 { $day } 天。友军地点 { $friendly }，敌军地点 { $enemy }。
ops-next-campaign = 下一场战役将在 { $time } 后开始
ops-next-campaign-now = 下一场战役即将开始
ops-next-from-menu = 请从主菜单开始下一场战役。
ops-section-log = 战役日志
ops-section-chat = 聊天
ops-no-messages = 暂无消息
ops-message = 消息
    .hint = 输入消息，按回车发送
ops-send = 发送

## 简报

ops-briefing = 任务简报
ops-briefing-line = 出发基地 { $base } - 距目标 { $km } km - { $aircraft } - 挂载：{ $stores }
ops-route-locked = 航线已锁定：{ $pilots } 正在飞行
ops-route-threats = 航线与威胁
ops-take-off = 起飞
ops-radars-shown = 显示的雷达：被友军飞机探测到且仍然存在

## 玩家

ops-section-players = 玩家
ops-col-name = 名字
ops-col-status = 状态
ops-col-sorties = 出击
ops-col-points = 得分
ops-col-pilot = 飞行员
ops-col-objectives = 目标
ops-col-kills = 击毁 空/地
ops-col-medal = 最佳勋章
ops-you = （你）
ops-host = 主机
ops-status-planning = 规划：{ $title }
ops-status-board = 在任务看板
ops-status-flying = 飞行中：{ $title } - { $aircraft }
ops-status-reconnecting = 连接丢失 - 飞机保留 { $seconds } 秒
ops-flown = { $flown }（达成 { $won }）
ops-section-leaderboard = 排行榜
ops-alone = 单人飞行。若要一起飞行，请在主菜单的多人游戏中创建此战役。
ops-rules = 本战役的规则（自开始时沿用）：{ $rules }
ops-no-sorties = 尚未执行任何出击
