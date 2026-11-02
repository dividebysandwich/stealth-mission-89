# 《隐形任务 89》：所有界面共用的词语。
#
# 英文是所有译文的源文本。某种语言缺少的消息会显示为英文。以下内容在所有语言中保持英文：
# 座舱显示的内容（HUD、MFD 和面板标签）、缩略语（FLIR、ILS、CCIP、SAM、RWR 等）、
# 飞机、武器和地名（见 fichiers_md/translation.md，以及各语言文件夹中的 glossary.md）。
#
# Fluent 语法：多行文本中的一行不得以 "["、"*" 或 "." 开头；花括号写作 {"{"} 和 {"}"}。

language = 语言
    .facts =
        菜单、提示、简报和消息所用的语言
        座舱自身的标签（HUD、MFD、面板）保持英文，与真实飞机一致

## 按钮

back = 返回
    .facts = 返回上一界面
ok = 确定
cancel = 取消
close = 关闭
continue = 继续
yes = 是
no = 否
on = 开
off = 关
classic = 经典
modern = 现代

## 翻译系统测试

i18n-test-count = { $n ->
   *[other] { $n } 次命中
}
i18n-test-lines =
    第一行
    第二行
i18n-test-nested = 请选择：{ $what }
