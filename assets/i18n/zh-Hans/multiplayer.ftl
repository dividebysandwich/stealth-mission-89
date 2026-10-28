# 战役窗口（src/campaign/menu.rs）、多人游戏窗口（src/multiplayer_menu.rs）、
# 战争地图的图例和实时视图（src/campaign/picture.rs、unit_view.rs）。

delete = 删除
delete-campaign = 删除此战役？
delete-save = 删除此存档？
delete-permanent = 删除后无法恢复

## 战役窗口

cm-group-campaign = 战役
cm-weather-dynamic = 动态
    .facts =
        天气每隔天空时钟的 10-22 小时变化一次
        云层会同时遮挡双方的视线
cm-weather-clear = 晴空
    .facts = 整个战役期间无云
cm-section-day-night = 昼夜
cm-day-real = 实时
    .facts = 一天为 24 小时
cm-day-2h = 2 小时 / 天
    .facts = 一天为 2 小时
cm-day-1h = 1 小时 / 天
    .facts = 一天为 1 小时
cm-day-30min = 30 分钟 / 天
    .facts = 一天为 30 分钟
cm-conditions-note = 条件适用于新战役
cm-rules-note = 已保存的战役沿用其开始时的规则
cm-about =
    战争在你游玩时实时进行，打开菜单时也不会停止。
    执行看板上的任务：侦察、SEAD、CAP、CAS、反舰、打击、护航。
    直升机投送可夺取敌方的城市和机场。
    你不会阵亡或被俘：出击结束后你会回到看板。
cm-section-new = 新战役
cm-new-line = { $theater }  -  { $situation }  -  对手：{ $opponents }
cm-start = 开始
cm-section-continue = 继续
cm-no-saves = 没有已保存的战役
cm-played = 已游玩 { $hours } 小时
cm-save-gone = 存档已不存在
cm-save-unreadable = 无法读取存档：{ $error }
cm-save-undeletable = 无法删除存档：{ $error }
cm-in-vr = VR 中
cm-delete =
    .facts = 删除此已保存的战役

## 战争地图

pic-friendly-aircraft = 友军飞机
pic-friendly-helicopter = 友军直升机
pic-enemy-fighter = 敌方战斗机（雷达上）
pic-sam-known = SAM 阵地（已知）
pic-guns-known = 高炮 / MANPADS（已知）
pic-reconned = 已侦察
pic-you = 你
pic-site = { $place } 阵地
uv-live = { $unit } - 实时
uv-contact-lost = 失去联系
uv-hint = 拖动：环绕  滚轮：缩放

## 多人游戏

mp-disconnected = 已断开连接：{ $why }
mp-connecting = 正在连接 { $address }...
mp-joining = 已连接 { $address }，正在加入...
mp-browser-public-only = 在浏览器中只能加入公开列表上的服务器
mp-title-host = 多人游戏：创建游戏
mp-subtitle = 动态战役中的合作模式：并肩作战
mp-your-name = 你的名字
    .hint = 飞行员名字
mp-join-address = 通过地址加入
    .hint = 地址或 名称[:端口]
mp-connect = 连接
mp-server-name = 服务器名称
mp-udp-port = UDP 端口
mp-address-note = 其他玩家使用这台电脑的地址加入；如需通过互联网游玩，请在路由器中开放 UDP 端口
mp-list-public = 列入公开服务器列表（其他人可在服务器列表中找到此游戏）
mp-rules = 适用于所有人的规则
mp-time-of-day = 时间段（有月光的夜晚）
mp-realistic-weapons = 真实的 F-117A 武器
mp-friendly-fire = 友军误伤
mp-day-night = 昼夜：
mp-weather = 天气：
mp-host-new = 创建新战役
mp-host-saved = 创建已保存的战役
mp-host = 创建
mp-host-game = 创建游戏
mp-port-error = UDP 端口 { $port }：{ $error }
mp-default-server = { $pilot } 的战役
mp-public-servers = 公开服务器
mp-refresh = 刷新
mp-server-count = { $servers } 个服务器，{ $pilots } 名飞行员
mp-list-error = 无法访问服务器列表：{ $error }
mp-no-servers = 当前没有公开服务器
mp-col-server = 服务器
mp-col-pilots = 飞行员
mp-col-war = 战争
mp-dedicated = 专用
mp-other-version = 其他版本
mp-join = 加入
