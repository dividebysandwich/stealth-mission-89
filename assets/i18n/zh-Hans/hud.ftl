# 游戏在 HUD 和飞行 2D 叠加层上的消息（src/hud、src/combat.rs、src/csar_view.rs、src/campaign）。
# HUD 以无重音大写绘制；航电自带的词（CCIP、THR、KM、MAX...）保持英文。

hud-no-ejection-seat = 无弹射座椅
hud-waypoint-reached = 已到达航路点 { $waypoint }
hud-select-camera = 选择摄像机
hud-open-bay = 打开弹舱
hud-camera-page = { $page } 页
hud-target-in-view-shoot = 目标在视野内 - 射击
hud-target-in-view = 目标在视野内
hud-aim-at-target = 瞄准目标
hud-photo = 拍照 { $state }
hud-rearmed = 已重新挂弹并加油
hud-landed-rearm = 已着陆 - 重新挂弹或结束出击
hud-objective-achieved = 目标已达成

## 任务结束叠加层

end-accomplished = 已完成
end-not-accomplished = 未完成
end-primary = 主要目标：{ $result }
end-secondary = 次要目标：{ $result }
end-kills = 击毁：飞机 { $air } 架，地面目标 { $ground } 个
end-time = 任务时间 { $time }
end-damage = 受损：被击中 { $hits } 次
end-training = 训练任务
end-press-any-key = 按任意键

## 被击落飞行员的面板

csar-flare = 信号弹
csar-faster = 加速
csar-normal = 正常时间
csar-skip = 跳过营救
csar-parachute = 降落伞已展开
csar-on-ground = 已着地
csar-down = 坠落在敌方领土
csar-pickup = 救援   UH-60 在头顶 - 接应
csar-inbound = 救援   UH-60 { $km } KM { $direction } - 正在赶来
csar-fire-flare = 救援   UH-60 { $km } KM { $direction } - 接近时发射信号弹
csar-enemy = 敌军    { $count } 架 HIND，最近 { $km } KM
csar-flares = 信号弹   { $count }      着地 { $time }

## 面板上的合作模式通知（src/player_notices.rs），防空控制台的提示

notice-joined = { $pilot } 已加入
notice-left = { $pilot } 已离开
notice-lost-connection = { $pilot } 已断开连接
st-select-track = 请先选择一个航迹
