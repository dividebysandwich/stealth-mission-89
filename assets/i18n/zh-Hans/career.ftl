# 飞行员生涯（src/career.rs）：军衔、勋章、服役期。

## 美国空军军衔（以中国空军对应军衔翻译），及其简称

rank-0 = 少尉
    .short = 少尉
rank-1 = 中尉
    .short = 中尉
rank-2 = 上尉
    .short = 上尉
rank-3 = 少校
    .short = 少校
rank-4 = 中校
    .short = 中校
rank-5 = 上校
    .short = 上校

## 勋章（美国勋章：保留其专有名称）

medal-air-medal = Air Medal
medal-distinguished-flying-cross = Distinguished Flying Cross
medal-silver-star = Silver Star
medal-air-force-cross = Air Force Cross
medal-medal-of-honor = Medal of Honor
medal-purple-heart = Purple Heart
medal-prisoner-of-war-medal = Prisoner of War Medal

## 出击后的消息

career-awarded = 获颁 { $medal }
career-kia = 阵亡
    .short = KIA
career-pow = 战俘
    .short = POW
career-active = 现役
career-captured = 被俘：生涯结束
career-tour-sortie = 服役期：出击 { $sortie }/{ $length }
career-tour-complete = 服役期完成：{ $points } 分（+{ $bonus }）
career-promoted = 晋升为 { $rank }

## 生涯窗口

career-title = 飞行员记录
career-group-tour = 服役期
career-set-by-tour = 由飞行员的服役期决定
career-pilots = 飞行员
career-new-pilot = 新飞行员
career-name = 姓名
career-enlist = 入伍
career-select = 选择或招募一名飞行员
career-service-record = 服役记录
career-rank = 军衔
career-points = 积分
career-sorties = 出击
career-tours = 服役期
career-status = 状态
career-decorations = 勋章
career-tour-of-duty = 服役期
career-tour-line = { $theater }  { $situation }  出击 { $sortie }/{ $length }  { $points } 分
career-destroyed = 迄今已摧毁 { $count } 个目标
career-fly-next = 执行下一次出击
career-new-tour = 在 { $theater }、{ $situation }中出击 { $count } 次，对手：{ $opponents }
    .facts = 在本服役期的剩余时间内，被您摧毁的目标不会复原
career-start-tour = 开始服役期
career-retired = 已退出现役
career-last-sorties = 最近的出击
