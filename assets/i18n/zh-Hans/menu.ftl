# 主菜单和单次任务界面（src/menu.rs）。

## 战区（stealth_host::theater::THEATERS）

theater-libya = 利比亚（1986）
    .facts =
        锡德拉湾和利比亚海岸
        “黄金峡谷行动”时期
        多数时候沙漠晴空万里
theater-persian_gulf = 波斯湾（1984）
    .facts =
        两伊“油轮战争”时期
        海上和沿海沙漠
        所有战区中天气最晴朗
theater-north_cape = 北角
    .facts =
        挪威北极地区和科拉半岛
        苏联北方舰队的母港
        低云层频繁
theater-central_europe = 中欧
    .facts =
        德国上空的北约对华约
        晴天仅占 15% 的时间
theater-vietnam = 越南
    .facts =
        河内和海防地区
        丛林和山地地形
        30% 的任务有雷暴
theater-middle_east = 中东
    .facts =
        地中海东部和黎凡特
        天空大多晴朗
theater-desert_storm = 沙漠风暴
    .facts =
        科威特和伊拉克，1991 年
        F-117A 的真实首战
        沙漠天空大多晴朗
theater-cuba = 古巴
    .facts =
        加勒比海岛屿战区
        海上和陆地目标
        热带：15% 的任务有风暴
theater-korea = 朝鲜半岛
    .facts =
        非军事区以北的朝鲜半岛
        山地地形
        天气多变，各种天空都有可能
theater-ukraine = 乌克兰（2026）
    .facts =
        乌克兰东部、克里米亚和俄罗斯南部
        从哈尔科夫到赫尔松的前线燃烧不止
        开阔的农田，前线的城镇满目疮痍

## 态势（交战规则）

roe-0 = 冷战
    .facts =
        和平时期的秘密行动
        保持不被发现最为重要
        巡逻的敌机较少
        确切的交战规则见简报
roe-1 = 有限战争
    .facts =
        地区性武装冲突
        敌方防空处于戒备状态
        敌机比冷战时更多
roe-2 = 常规战争
    .facts =
        全面战争
        敌方所有部队都处于活动状态
        敌机比冷战时更多

## 任务类型

kind-strike = 打击
    .facts =
        摧毁、拍摄或抵达地面目标
        主要目标和次要目标
        对地武器配置
kind-air = 空战
    .facts =
        拦截敌机
        运输机、轰炸机或战斗机巡逻队
        空对空导弹武器配置
kind-strike-training = 打击训练
    .facts =
        打击任务练习
        学习武器和导航
kind-air-training = 空战训练
    .facts =
        空对空任务练习
        学习拦截和导弹
kind-drone-defense = 无人机防御
    .facts =
        仅限乌克兰
        在“天竺葵”攻击无人机机群袭击城市或核电站之前将其拦下
        然后轰炸其发射场

## 对手

opponents-0 = 新手
    .facts =
        缺乏经验的敌方机组
        最简单的设置
opponents-1 = 普通
    .facts =
        水平一般的敌方机组
        默认设置
opponents-2 = 老手
    .facts =
        技术娴熟的敌方机组
        舰船和巡逻队更多
opponents-3 = 精英
    .facts =
        最优秀的敌方飞行员和 SAM 机组
        最难的设置

## 天气

sky-random = 随机
    .facts =
        根据战区气候随机生成
        每次新任务都会变化
        云层会遮挡双方的视线
sky-clear = 晴朗
    .facts =
        无云
        能见度 40 km
        经典 DOS 时代的天空
sky-scattered = 少云
    .facts =
        积云 20-45%，云底 2500-6000 ft
        有时高空有一层薄高层云
        能见度 30 km
sky-broken = 多云
    .facts =
        积云 45-70%，云底 2000-5000 ft
        上方常有一层层积云
        能见度 25 km
sky-overcast = 阴天
    .facts =
        层云 80-100%，云底 1500-5000 ft
        下方有少量碎积云
        能见度 20 km
sky-storm = 风暴
    .facts =
        积雨云高达 40000 ft
        1000 ft 以上有低层云和积云
        能见度 8 km
menu-random-cover = 随机云量
    .facts =
        主云层的云量随天气随机生成
        取消勾选可自行设定
        主云层：阴天和风暴为层云，其余为积云
menu-cover-slider =
    .facts =
        主云层覆盖天空的比例
        对晴朗天空无效

## 月亮

moon-date = 任务日期
    .facts =
        简报日期的真实月相
        需要开启昼夜变化开关（现代）
moon-0 = 新月
    .facts =
        没有月光：漆黑的夜晚，只有灯光和星星
        敌方飞行员在 2 km 处看到您
moon-1 = 蛾眉月
    .facts = 傍晚天空中的一弯细月，午夜前落下
moon-2 = 上弦月
    .facts = 半边被照亮，傍晚升起，午夜前后落下
moon-3 = 盈凸月
    .facts = 明亮，下午升起，午夜后才落下
moon-4 = 满月
    .facts =
        整夜在天，午夜时最高
        敌方飞行员在 4 km 处看到您
moon-5 = 亏凸月
    .facts = 明亮，傍晚升起，一直到早晨
moon-6 = 下弦月
    .facts = 半边被照亮，午夜前后升起
moon-7 = 残月
    .facts = 黎明前升起的一弯细月

## 飞机

aircraft-f117 =
    .facts =
        隐形攻击机
        亚音速，7 g
        现代和经典座舱
aircraft-f16 =
    .facts =
        多用途战斗机
        2 马赫级，9 g，滚转迅速
        带 RWR 的 F-16 座舱
aircraft-tornado =
    .facts =
        低空突击机
        变后掠翼随速度后掠
        现代座舱，三块 MFD
aircraft-a10 =
    .facts =
        近距空中支援飞机
        速度慢，皮实耐打，30 mm 机炮
        A-10C 座舱，两块 MFD
aircraft-uh60 =
    .facts =
        通用直升机
        运送部队：降落在目标附近
        周期变距杆和总距杆
aircraft-ah1z =
    .facts =
        攻击直升机
        炮塔机炮、Hellfire、AIM-9
        机炮和 Hellfire 跟随吊舱的指向

## 分区和分组

section-theater = 战区
section-opponents = 对手
section-situation = 态势
section-mission = 任务
section-moon = 月亮
section-start-time = 起始时间
section-weather = 天气
section-realism = 拟真度
section-tacview = Tacview
section-mission-summary = 任务概要
section-briefing = 简报
group-mission = 任务
group-conditions = 条件
group-vehicle = 载具
group-simulation = 模拟

## 任务选项

menu-start-random = 随机
    .facts =
        每次新任务随机生成，与经典模式相同
        白天 06:00-08:00 或夜间 18:00-20:00
        取消勾选可自选时刻
menu-start-slider =
    .facts =
        起飞时间
        夜间为 18:00 至 06:00
        新任务时保留此设置
menu-realistic-weapons = 真实武器
    .facts =
        真实飞机的限制
        2 个弹舱而不是 4 个
        没有 AIM-9 或 AIM-120 导弹
        不能在航母上起降
menu-crash-protection = 防撞保护
    .facts =
        在近地面时自动拉起（起落架收起）
        触地和碰撞不会致命
        关闭：始终可能撞地
menu-tacview-export = 导出
    .facts =
        将每次飞行保存为 Tacview 记录（.acmi）
        保存在游戏的用户文件夹中，tacview 子目录下
menu-tacview-real-time = 实时遥测
    .facts =
        飞行时将飞行数据发送给 Tacview
        在 Tacview 中：Record > Real-Time Telemetry
        地址 127.0.0.1，端口 42674（--tacview-address）
switch-game = 游戏
switch-real = 真实
menu-damage = 损伤
    .facts =
        武器损伤
        游戏：经典损伤；命中会累积，但不会击落飞机
        真实：一发导弹命中就可能摧毁飞机（弹射！），损坏更多系统，常引发火灾，火会自行烧尽或使飞机爆炸
        真实：受损的操纵面会使飞机失去配平或产生振荡
menu-guidance = 制导
    .facts =
        导弹制导
        经典：导弹径直飞向目标
        真实：按比例导引飞向碰撞点；发动机会燃尽，导弹继续滑翔，每次转弯都会损失速度（仅在发动机燃烧时有烟）
        真实：SAM 对远距离目标采用抛物线弹道；AIM-120 先直飞 3 秒（拉起可使其抛射）
menu-air-defence = 防空
    .facts =
        低空防空火力
        经典：只有 SAM 阵地和战斗机
        真实：雷达引导的 ZSU-23-4 高炮（其雷达像 SAM 的雷达一样能发现您）和 S-60 高射炮连守卫机场和目标；城镇中的 MANPADS 小组会向所见目标发射 SA-14
        在设防地区低空飞行会变得危险；炸弹和机炮可摧毁这些火炮

## 主界面

menu-title = STEALTH MISSION 89
menu-subtitle = 任务规划系统
menu-single-mission = 单次任务
    .facts =
        在您选择的战区执行一次任务
        选择选项，规划航线和武器配置
menu-career = 生涯
    .facts =
        飞行员记录：军衔、勋章、出击
        服役期：在一个战区执行 12 次出击
        被摧毁的目标不会复原
menu-campaign = 动态战役
    .facts =
        实时动态战役
        夺取敌方的城市和机场
        任务根据实时态势生成
menu-multiplayer = 多人游戏
    .facts = 动态战役中的合作模式
    .join = 加入公共列表中的服务器
    .host = 加入服务器，或创建战役并在其中飞行
menu-options = 选项
    .facts =
        游戏：座舱、HUD 和航电风格，自动配平
        图形、音量、按键和摇杆绑定
        保存后用于之后的飞行
    .short = 游戏、图形、声音和控制
menu-download = 下载
    .facts =
        适用于 Windows、Linux 和 macOS 的桌面版游戏
        支持 VR 和多人游戏
        Steam Deck 和 Steam Frame：Linux AppImage
web-play = 在浏览器中游玩
    .facts = 就在这里，无需安装
web-download-platforms = Windows、Linux 和 macOS，Steam Deck 和 Steam Frame
web-download-version = 下载版
web-gives-vr = VR
    .why = 使用任何兼容 OpenXR 的头显和 Steam Frame 以 VR 飞行
web-gives-hotas = HOTAS
    .why = 完整支持手柄、摇杆、油门和脚舵
web-gives-multiplayer = 更好的多人游戏
    .why = 直接 UDP 连接服务器，延迟更低
web-gives-server = 自建服务器
    .why = 创建合作战役，或运行专用服务器
web-gives-head-tracking = 头部追踪
    .why = 通过 OpenTrack 支持 TrackIR 类追踪器
web-gives-tacview = Tacview
    .why = 飞行记录和实时遥测
web-gives-keep = 永久保留
    .why = 即使网站关闭，您仍拥有自己的副本
menu-discord = DISCORD
    .facts =
        一起飞行、报告错误、分享任务
        关注开发进展
menu-quit = 退出
    .facts = 退出到桌面
menu-version = 版本 { $version }

## 单次任务界面

menu-career-sortie = 生涯出击
menu-tour-sortie = { $pilot }  //  服役期出击 { $sortie }/{ $length }
menu-msn = 任务规划系统  //  MSN { $seed }
menu-no-targets = 本服役期已没有尚存的目标
menu-plan-route = 规划航线
    .facts =
        显示敌方雷达和 SAM 覆盖范围的战区地图
        添加、移动和删除导航点
        目标和着陆基地保持固定
menu-loadout = 武器配置
    .facts =
        选择每个弹舱或挂架上的外挂
        初始为简报规定的武器
menu-generating = 生成中...
menu-back-records = 返回飞行员记录
menu-back-main = 返回主菜单
menu-new-sortie = 新出击
menu-new-mission = 新任务
    .facts =
        随机生成不同的目标
        保留所有选项和起始时间
menu-load = 载入
    .facts =
        飞行中保存的任务（菜单：保存任务）
        从保存的位置继续飞行
menu-fly = 起飞
    .facts = 执行简报规定的任务并起飞
menu-fly-vr = 以 VR 飞行
    .facts =
        在 VR 头显中重新启动游戏
        飞行的就是这个任务
    .no-mission = 请先生成任务
    .no-runtime = 没有运行中的 OpenXR 运行时（Monado、WiVRn、SteamVR）
menu-vr-failed = 无法以 VR 启动：{ $error }

## 任务概要

summary-theater = 战区
summary-situation = 态势
summary-mission = 任务
summary-opponents = 对手
summary-time = 时间
summary-weather = 天气
summary-fuel = 燃油
summary-night = 夜间
summary-day = 白天
summary-time-value = { $daytime }  起飞 { $depart }
summary-weather-value = { $sky }  能见度 { $vis } km
summary-fuel-value = { $fuel } lbs
summary-armament = 武器：{ $stores }
summary-intelligence = 情报
