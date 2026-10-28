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
options-section-vr-size = VR 座舱大小
options-vr-size =
    .facts =
        头显中座舱（和世界）的大小
        100%：与实物等大

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
