# 任务讲评（src/debrief.rs）和模拟报告（stealth-sim score.rs）。

## 飞行如何结束

outcome-landed = 安全着陆
outcome-bad-landing = 着陆不佳
outcome-ejected = 已弹射 - 伞降着陆
outcome-rescued = 已弹射 - 被 CSAR 救起
outcome-captured = 已弹射 - 被俘
outcome-crashed = 坠毁
outcome-ejection-failed = 弹射失败
outcome-aborted = 任务中止
outcome-collision = 碰撞

## 评级

rating-unsatisfactory = 不合格
rating-marginal = 勉强
rating-satisfactory = 合格
rating-excellent = 优秀
rating-outstanding = 卓越

## 得分

points-primary = 主要目标
points-secondary = 次要目标
points-aircraft = 击毁飞机
points-unseen = 隐蔽击落
points-ground = 击毁地面目标
points-drones = 击落无人机
points-drones-through = 突防无人机
points-nuclear = 命中核电站
points-detected = 被探测
points-missile-hits = 被导弹命中

## 时间线（地名和机型与任务中一致）

timeline-primary = {" "}（主要）
timeline-secondary = {" "}（次要）
timeline-destroyed = 击毁：{ $place }{ $tag }
timeline-destroyed-ground = 击毁地面目标{ $tag }
timeline-shot-down = 击落：{ $aircraft }{ $tag }
timeline-hit = 被导弹击中
timeline-seen = 被发现，机型：{ $aircraft }
timeline-radar-id = 被雷达识别，来源：{ $place }
timeline-objective = 到达目标{ $tag }
timeline-got-through = 突防：{ $aircraft }
timeline-photo = 已拍照{ $tag }

## 讲评窗口

debrief-title = 任务讲评
debrief-training = 训练任务
debrief-results = 结果
debrief-flight-map = 飞行地图
debrief-section-results = 结果
debrief-accomplished = 已完成
debrief-failed = 失败
debrief-primary = 主要
debrief-secondary = 次要
debrief-aircraft = 飞机
debrief-drones = 无人机
debrief-drones-value = 击落 { $down }，突防 { $through }
debrief-ground = 地面
debrief-weapons = 武器
debrief-radar-ids = 雷达识别
debrief-visual-ids = 目视识别
debrief-hits-taken = 受击次数
debrief-time = 时间
debrief-section-points = 得分
debrief-difficulty = 难度
debrief-total = 总计
debrief-section-timeline = 时间线
debrief-no-events = 无事件
debrief-section-exposure = 雷达暴露
debrief-tracked = 被跟踪
debrief-closest = 最接近
debrief-worst = 最差
debrief-at = 时刻 { $time }
debrief-altitude = 高度
debrief-speed = 速度
debrief-margin = 裕度
debrief-strongest = 最强
debrief-tracked-by = 跟踪来源：{ $sites }
debrief-section-map = 地图
debrief-legend-safe = 不可探测
debrief-legend-close = 接近探测
debrief-legend-detectable = 可探测
debrief-legend-tracked = 被跟踪 / 导弹发射
debrief-legend-hit = 被击中 / 击毁
map-symbols = 符号
map-legend-clouds = 云 / 雷暴云塔
map-legend-snow = 雪，由弱到强
map-legend-rain = 雨，由弱到强
map-legend-thunderstorm = 雷暴单体
debrief-pause = 暂停
debrief-play = 播放
