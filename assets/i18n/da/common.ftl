# Stealth Mission 89: ord der bruges på alle skærme (se en/common.ftl og glossary.md).
#
# Engelsk er kilden til alle oversættelser. En manglende besked vises på engelsk. Altid på
# engelsk: det cockpittet viser (HUD, MFD og panelmærkater), forkortelser (FLIR, ILS, CCIP,
# SAM, RWR...), fly-, våben- og stednavne (fichiers_md/translation.md og glossary.md i hvert
# sprogs mappe).
#
# Fluent-syntaks: en linje i en flerlinjet tekst må ikke begynde med "[", "*" eller "."; krøllede
# parenteser skrives {"{"} og {"}"}.

language = Sprog
    .facts =
        Sproget i menuer, tooltips, briefinger og beskeder
        Cockpittets egne mærkater (HUD, MFD, panel) forbliver på engelsk, som i det rigtige fly

## Knapper

back = TILBAGE
    .facts = Tilbage til forrige skærm
ok = OK
cancel = ANNULLER
close = LUK
continue = FORTSÆT
yes = JA
no = NEJ
on = TIL
off = FRA
classic = KLASSISK
modern = MODERNE

## Test af oversættelsessystemet

i18n-test-count = { $n ->
    [one] { $n } træffer
   *[other] { $n } træffere
}
i18n-test-lines =
    Første linje
    Anden linje
i18n-test-nested = Vælg: { $what }
