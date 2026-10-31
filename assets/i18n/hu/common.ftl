# Stealth Mission 89: minden képernyőn közös szavak (lásd en/common.ftl és glossary.md).
#
# Az angol a fordítások forrása. A nyelvből hiányzó üzenet angolul jelenik meg. Mindenhol
# angolul marad: a pilótafülke feliratai (HUD, MFD és panelek), a rövidítések (FLIR, ILS,
# CCIP, SAM, RWR...), a repülőgépek, fegyverek és helyek nevei
# (fichiers_md/translation.md és az egyes nyelvek glossary.md fájlja).
#
# Fluent szintaxis: egy többsoros szöveg sora nem kezdődhet "[", "*" vagy "." jellel; a
# kapcsos zárójelek írása {"{"} és {"}"}.

language = Nyelv
    .facts =
        A menük, tooltipek, briefingek és üzenetek nyelve
        A pilótafülke saját feliratai (HUD, MFD, panel) angolul maradnak, mint a valódi gépben

## Gombok

back = VISSZA
    .facts = Vissza az előző képernyőre
ok = OK
cancel = MÉGSE
close = BEZÁRÁS
continue = FOLYTATÁS
yes = IGEN
no = NEM
on = BE
off = KI
classic = KLASSZIKUS
modern = MODERN

## A fordítórendszer tesztjei

i18n-test-count = { $n ->
    [one] { $n } találat
   *[other] { $n } találat
}
i18n-test-lines =
    Első sor
    Második sor
i18n-test-nested = Válasszon: { $what }
