# 弹药：各空军基地的库存、生产基地、运输飞行。

mu-sam-long = 远程 SAM
mu-sam-medium = 中程 SAM
mu-sam-short = 近程 SAM
cl-production-moved = { $base } 成为新的生产基地
cl-cargo-scheduled = 飞往 { $base } 的运输飞行将在 10 分钟后从 { $from } 起飞
cl-cargo-delivered = 运输飞行已把弹药送达 { $base }
cl-cargo-lost = 飞往 { $base } 的运输飞行连同弹药一起损失
cl-enemy-cargo-lost = 飞往 { $base } 的敌方运输飞行被击落
cb-intercept-cargo = 拦截运输飞行 - { $base }
cb-airlift = 空运 - { $base } (从 { $from } 出发)
ce-out-of-stock = { $weapon }：本基地已无库存
loadout-at-base = （基地有 { $count }）
loadout-base-out = 本基地剩余不足

## 作战界面的后勤标签页和地图

ops-tab-logistics = 后勤
mt-production-base = 生产基地（弹药）
lg-bases = 空军基地
lg-production = 生产
lg-production-base = 生产基地：制造弹药，由运输飞行送往其他基地。
lg-bombed = 已遭轰炸：修复之前运输飞行不会在此降落。
lg-bombed-production = 已遭轰炸：修复之前既无生产也无运输飞行。
lg-stocked = 库存充足
lg-out-of = 已耗尽：{ $items }
lg-low = 库存偏低：{ $items }
lg-cargo-due = 运输飞行将在 { $time } 后起飞
lg-cargo-up = 运输飞行在途
lg-cargo-player = 运输飞行由 { $pilot } 驾驶
lg-munition = 弹药
lg-stock = 库存
lg-made = 产量
lg-per-hour = { $count }/小时
lg-next-in = 下一批 { $time } 后
lg-at-cap = 仓库已满
lg-never-out = AIM-9 Sidewinder、非制导炸弹、机炮、干扰弹和燃油永远不会用完。
lg-cargo-flights = 运输飞行
lg-no-cargo = 没有在途的运输飞行。
lg-cargo-route = { $from } → { $to }
lg-cargo-load = 载荷：{ $items }
lg-forces = 兵力与物资
cl-production-lost = 已没有能制造弹药的空军基地：占领 { $base } 即可成为新的生产基地。航空母舰的弹药永远不会用完。
mt-production-target = 占领后成为生产基地
lg-no-production = 没有已占领的空军基地：占领 { $base } 即可成为生产基地。航空母舰的弹药永远不会用完。
