# 选项窗口（src/options.rs）及其在飞行菜单中的页面。

options-title = 选项
options-tab-game = 游戏
options-tab-graphics = 图形
options-tab-sound = 声音
options-tab-controls = 控制
options-footer = 保存后用于之后的飞行；拟真度开关在每次游戏时选择

## 游戏

options-section-language = 语言
options-section-classic-modern = 经典 / 现代
options-cockpit = 座舱
    .facts =
        现代：带边框按钮的高分辨率显示器
        经典：DOS 时代的仪表盘布局和 77x60 显示器（F-117A）
options-hud = HUD
    .facts =
        现代：绿色的高分辨率 F-16 风格 HUD，带 NAV、AA 和 AG 模式
        经典：DOS 时代的 HUD
options-avionics = 航电
    .facts =
        现代：F-16 风格的 MFD 格式，可在任一 MFD 上显示（态势显示、HARM、瞄准吊舱、地图、外挂），带可移动光标
        经典：左侧为地图和 TAC，右侧为摄像机、WAY、WPN 和 ORD
options-flight-model = 飞行模型
    .facts =
        现代：带电传操纵的空气动力学飞行模型：驾驶杆控制俯仰和滚转速率；松杆后机头保持不动，航迹得以保持
        经典：DOS 时代的飞行模型
options-section-flying = 飞行
options-auto-trim = 自动配平
    .facts =
        松开驾驶杆后，配平会保持您松杆时的姿态
        直升机：机头不再随速度抬升；喷气机：消除受损操纵面造成的偏差
        手动配平：Alt + 方向键，Alt + Home 回中
options-tooltips = 座舱提示
    .facts = 鼠标或 VR 指尖所指的座舱控件或显示器是什么
options-player-labels = 玩家名称
    .facts = 多人游戏：其他飞行员的名字显示在其飞机上方，无论多远
options-label-size =
    .facts =
        玩家名称的大小
        50%：默认
options-section-vr-size = VR 座舱大小
options-vr-size =
    .facts =
        头显中座舱（和世界）的大小
        100%：与实物等大
options-section-gif = GIF 录制
options-gif-size = 尺寸
options-gif-rate = 帧率
options-gif-scale-100 =
    .facts = 窗口的完整分辨率：文件较大
options-gif-scale-25 =
    .facts = 文件较小
options-gif-rate-50 =
    .facts = 仅当游戏以 100 fps 或更高运行时（每隔一帧录制）
options-gif-hint = F12（除非已重新绑定）开始和停止；保存在你的图片文件夹中

## 图形

options-section-rendering = 渲染
options-shaders = 着色器
    .facts =
        现代：细腻的定向光影，平滑的远景雾霾
        经典：平面着色
options-shadows = 阴影
    .facts =
        现代：座舱和世界中的日光阴影（占用 GPU 时间）
        经典：无阴影
options-point-lights = 点光源
    .facts =
        现代：座舱泛光灯和显示器灯光；爆炸、尾焰和跑道灯会照亮场景
        经典：只有日光和天光
options-time-of-day = 昼夜变化
    .facts =
        现代：太阳和月亮随任务时钟移动，天空随太阳变暗和变亮；夜间有星星和月相
        现代：月光决定敌方飞行员在夜间能看到您的距离（无月之夜 2 km，满月下 4 km）
        经典：颜色缓慢变化，太阳固定不动
options-msaa = 抗锯齿（MSAA）
    .facts =
        平滑边缘：4x MSAA，VR 中为 2x（占用 GPU 时间）
        关闭：边缘为生硬的像素
options-auto-detail = 自动场景细节（VR）
    .facts =
        在头显中，帧率不足时城镇、森林和田野的显示范围会缩小，有余力时再恢复
        图形设置保持您的选择
options-sun-moon = 太阳和月亮
    .facts =
        白天天空中有太阳，夜间有月亮和星星
        月亮显示任务当天的月相
options-cloud-quality = 云
cloud-quality-auto = 自动
    .facts = VR 中为低，屏幕上为高
cloud-quality-low = 低
    .facts = 三分之一分辨率的粗糙云（最快）
cloud-quality-medium = 中
    .facts = 一半分辨率，细节更多
cloud-quality-high = 高
    .facts = 全分辨率，最细的细节和自身阴影（耗费 GPU 时间）

## 声音

options-section-volume = 音量
options-volume-master = 总音量
    .facts = 所有声音
options-volume-engine = 发动机
    .facts = 您自己的发动机或旋翼，以及周围听到的飞机
options-volume-effects = 音效
    .facts = 武器、爆炸、起落架、无线电及其他
options-volume-warnings = 警报
    .facts = 失速警报、雷达和导弹警告、火警警笛、Sidewinder 音调
options-volume-rwr = RWR
    .facts = 雷达告警音，作为警报音量的一部分
options-section-rwr-tones = 雷达告警音
rwr-tones-all = 全部
    .facts = 照射你的雷达、每次发现你的扫描、导弹发射
rwr-tones-detect = 仅探测
    .facts =
        雷达发现你时响一声（雷达一段时间失去你后会再次响起）
        以及导弹发射
rwr-tones-launch = 仅发射
    .facts = 仅对你发射的导弹
rwr-tones-off = 关
    .facts = 无雷达告警音（指示灯和显示器仍会告警）
options-section-cockpit = 座舱
options-low-pass = 发动机低通
    .facts =
        在飞机内部，发动机声音听起来发闷（座舱盖和头盔滤掉了高频）
        关闭：与外部相同

## 控制

options-section-bindings = 绑定
options-keys-joystick = 按键和摇杆
    .facts =
        地图按键、摇杆和 HOTAS 的轴与按钮
        保存后用于之后的飞行
options-section-head-tracking = 头部/眼部追踪
options-head-tracking = 头部/眼部追踪
    .facts =
        您的头部转动并移动座舱中的视角（桌面版；VR 中由头显完成）
        追踪程序（OpenTrack 等）：FreeTrack 输出，或通过 UDP 发送到 127.0.0.1 端口 4242
        Home：当前头部位置成为中心
options-tracking = 追踪（{ $source }）
options-no-tracker = 未收到追踪器信号：请启动追踪程序

## EFFECTS

options-section-effects = 特效
options-section-retro = 复古
effect-shake = 镜头抖动
    .facts =
        开炮、附近爆炸、失速、触地和跑道会使视图抖动
        VR：座舱振动，头部保持不动
effect-flashes = 战斗闪光
    .facts = 高炮爆炸、爆炸和导弹发射会瞬间照亮云层、地面和座舱
effect-barrage = 夜间拦阻射击
    .facts =
        夜间常规战争：城镇的火炮向天空盲目射出曳光弹，探照灯扫过
        城镇上空的敌方无人机会被其探照灯捕获，并招来其火炮的射击
        仅为观感：弹幕不会击中任何东西
effect-cockpit-night = 座舱夜间照明
    .facts = 天黑后座舱内有昏暗的红色灯光（戴护目镜时为蓝绿色）¶ 有告警时，主警告灯在面板上方以琥珀色闪动
effect-nvg = 夜视镜
    .facts =
        强光会晕成光圈，黑暗中画面噪点更多
        桌面：护目镜的圆形视野
effect-snow = 雪
    .facts = 在冰点以上，风暴降雪而非降雨：北方冬天偏低，夏天偏高
effect-mist = 地面薄雾
    .facts =
        现代着色器：黎明、黄昏和夜晚，低地和海面上有雾
        沙漠中较薄，风暴中没有
effect-cloud-shadows = 云影
    .facts = 现代着色器：云在下方地面上的阴影
effect-vapour = 水汽与下洗气流
    .facts =
        急转弯时翼尖拖出水汽
        低空悬停的直升机扬起尘土，在水面上扬起水雾
        行驶中的车辆在身后扬起尘土
effect-night-sky = 夜空
    .facts =
        偶尔有流星，地平线附近的低空星星闪烁
        北角上空的极光
effect-vga = VGA 色彩
    .facts = 以现代着色重现 1989 年的外观：与 VGA 卡一样，每个色彩通道 64 级，带抖动
effect-scanlines = 扫描线
    .facts = CRT 显示器的扫描线（VR 中不适用）

## LANGUAGE NAMES

lang-name-en = 英语
lang-name-bg = 保加利亚语
lang-name-zh-Hans = 简体中文
lang-name-hr = 克罗地亚语
lang-name-cs = 捷克语
lang-name-da = 丹麦语
lang-name-nl = 荷兰语
lang-name-et = 爱沙尼亚语
lang-name-fi = 芬兰语
lang-name-fr = 法语
lang-name-de = 德语
lang-name-el = 希腊语
lang-name-hu = 匈牙利语
lang-name-id = 印度尼西亚语
lang-name-ga = 爱尔兰语
lang-name-it = 意大利语
lang-name-ja = 日语
lang-name-ko = 韩语
lang-name-lv = 拉脱维亚语
lang-name-lt = 立陶宛语
lang-name-mt = 马耳他语
lang-name-pl = 波兰语
lang-name-pt-BR = 葡萄牙语（巴西）
lang-name-ro = 罗马尼亚语
lang-name-ru = 俄语
lang-name-sk = 斯洛伐克语
lang-name-sl = 斯洛文尼亚语
lang-name-es = 西班牙语
lang-name-sv = 瑞典语
lang-name-uk = 乌克兰语

## Browser build: offer to play in the browser's language (src/i18n.rs)

language-offer-question = 浏览器语言：{ $language }。要使用该语言进行游戏吗？
language-offer-yes = 是
language-offer-no = 否
language-offer-never = 不再询问
effect-sunbeams = 阳光
    .facts = 穿过破碎云层缝隙的束状阳光
options-msaa-forced = 像素化（复古）开启时关闭
options-volume-radio = 无线电
    .facts = 在合作无线电中说话的其他玩家，以及语音 ATC
options-volume-atc = ATC 语音
    .facts = 语音 ATC 呼叫，属于无线电音量的一部分
options-volume-music = 音乐
    .facts = 主菜单音乐
options-section-atc = 语音 ATC
options-section-radio = 合作无线电
options-push-to-talk = 按键通话
    .facts =
        按住通话键（Caps Lock，或分配给 PTT 的摇杆/VR 按钮），所有玩家都能听到你
        麦克风在你第一次按下时打开
options-mic-gain = 麦克风增益
    .facts =
        你的麦克风发送的音量
        100%：原样
options-mic-default = 默认
options-voice-text-only = 仅文字
options-voices-line = 管制员：{ $controllers }   飞行员：{ $pilot }
options-voices = 语音
    .facts =
        下载无线电呼叫用的语音，删除它们
        选择管制员和你自己的语音，并试听
options-voices-web = 浏览器中仅文字
options-tracking-mouse = 追踪时鼠标视角
    .facts =
        开：鼠标右键仍可旋转和缩放座舱视角
        关：追踪器发送期间，鼠标右键在座舱中不起作用
options-atc = 空中交通管制
    .facts =
        友军机场的塔台、进近和 GCA 无线电呼叫
        起飞许可、盘旋进场，以及低云或能见度差时的 GCA 引导
options-pixel-block = 像素化块大小
    .facts = 方块边长，以屏幕像素计（2：细，5：粗）
effect-pixelize = 像素化
    .facts = 整个 3D 视图（地形、飞机、建筑、座舱）以大方块像素显示，如同 MS-DOS 电脑；菜单和提示保持清晰（VR 中不适用）¶ 滑块设置方块大小，以屏幕像素计（2：细，5：粗）¶ 开启时抗锯齿关闭
