# CJK fonts

`NotoSansSC-Subset.otf`, `NotoSansJP-Subset.otf` and `NotoSansKR-Subset.otf` are subsets of
[Noto Sans CJK](https://github.com/notofonts/noto-cjk) (Regular), the fonts the game draws
Chinese (Simplified), Japanese and Korean with. egui's Hack, the font of the menus, has no
glyphs for them.

Noto Sans CJK is licensed under the SIL Open Font License, Version 1.1
(<https://openfontlicense.org>), Copyright 2014-2021 Adobe. The subsets keep the fonts'
`name` table, with their copyright and license notices.

Each subset holds the characters of the language's texts (`assets/i18n/<language>`), the
common characters of the country's national standard (GB 2312, JIS X 0208 level 1, the
KS X 1001 syllables) for the players' names, the punctuation and the Latin letters.

## Making them again

Needed when a text of these languages gains a character (`cargo test` then fails in
`cjk_fonts_cover_the_texts`, `src/i18n.rs`):

1. Download the full fonts `NotoSansSC-Regular.otf`, `NotoSansJP-Regular.otf` and
   `NotoSansKR-Regular.otf` from `Sans/SubsetOTF/{SC,JP,KR}` in the noto-cjk repository and put
   them in `assets/fonts/source/` (not committed).
2. `pip install fonttools`, then `python tools/i18n_fonts.py`.
3. Commit the subsets.
