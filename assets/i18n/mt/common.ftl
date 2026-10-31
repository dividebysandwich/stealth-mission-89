# Stealth Mission 89: kliem komuni għall-iskrins kollha.
#
# L-Ingliż hu s-sors ta' kull traduzzjoni. Messaġġ nieqes minn lingwa jidher bl-Ingliż. Dejjem
# bl-Ingliż: dak li juri l-kokpit (HUD, MFD u tikketti tal-pannell), l-akronimi (FLIR, ILS,
# CCIP, SAM, RWR...), l-ismijiet tal-ajruplani, tal-armi u tal-postijiet
# (fichiers_md/translation.md, u glossary.md fil-folder ta' kull lingwa).
#
# Sintassi Fluent: linja ta' test b'ħafna linji ma tistax tibda b'"[", "*" jew ".";
# il-parentesi gagħwġa jinkitbu {"{"} u {"}"}.

language = Lingwa
    .facts =
        Il-lingwa tal-menus, tat-tooltips, tal-briefings u tal-messaġġi
        It-tikketti tal-kokpit (HUD, MFD, pannell) jibqgħu bl-Ingliż, bħal fl-ajruplan veru

## Buttuni

back = LURA
    .facts = Lura għall-iskrin ta' qabel
ok = OK
cancel = IKKANĊELLA
close = AGĦLAQ
continue = IKKUNTINWA
yes = IVA
no = LE
on = ON
off = OFF
classic = KLASSIKU
modern = MODERN

## Testijiet tas-sistema tat-traduzzjoni

i18n-test-count = { $n ->
    [one] { $n } daqqa
    [two] { $n } daqqiet
    [few] { $n } daqqiet
    [many] { $n } daqqa
   *[other] { $n } daqqa
}
i18n-test-lines =
    L-ewwel linja
    It-tieni linja
i18n-test-nested = Agħżel: { $what }
