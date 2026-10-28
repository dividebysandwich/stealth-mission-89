# Stealth Mission 89：すべての画面で共通の語句。
#
# 英語がすべての翻訳の原文です。言語に存在しないメッセージは英語で表示されます。常に英語のまま：
# コックピットの表示（HUD、MFD、パネルのラベル）、略語（FLIR、ILS、CCIP、SAM、RWR...）、
# 機体名・兵装名・地名（fichiers_md/translation.md、および各言語フォルダの glossary.md）。
#
# Fluent の構文：複数行テキストの行は "[" "*" "." で始めてはいけません。波括弧は
# {"{"} と {"}"} と書きます。

language = 言語
    .facts =
        メニュー、ツールチップ、ブリーフィング、メッセージの言語です
        コックピット自体の表示（HUD、MFD、パネル）は、実機と同じく英語のままです

## Buttons

back = 戻る
    .facts = 前の画面に戻ります
ok = OK
cancel = キャンセル
close = 閉じる
continue = 続ける
yes = はい
no = いいえ
on = オン
off = オフ
classic = クラシック
modern = モダン

## Tests of the translation system

i18n-test-count = { $n ->
   *[other] { $n } 回命中
}
i18n-test-lines =
    1行目
    2行目
i18n-test-nested = 選択：{ $what }
