# Stealth Mission 89: words shared by every screen.
#
# English is the source of every translation. A message missing from a language is shown in
# English. Kept in English everywhere: what the cockpit shows (HUD, MFD and panel labels),
# acronyms (FLIR, ILS, CCIP, SAM, RWR...), aircraft, weapon and place names
# (fichiers_md/translation.md, and glossary.md in each language's folder).
#
# Fluent syntax: a line of a multi-line text must not start with "[", "*" or "."; braces are
# written {"{"} and {"}"}.

language = Language
    .facts =
        The language of the menus, tooltips, briefings and messages
        The cockpit's own labels (HUD, MFD, panel) stay in English, as in the real aircraft

## Buttons

back = BACK
    .facts = Back to the previous screen
ok = OK
cancel = CANCEL
close = CLOSE
continue = CONTINUE
yes = YES
no = NO
on = ON
off = OFF
classic = CLASSIC
modern = MODERN

## Tests of the translation system

i18n-test-count = { $n ->
    [one] { $n } hit
   *[other] { $n } hits
}
i18n-test-lines =
    First line
    Second line
i18n-test-nested = Choose: { $what }
