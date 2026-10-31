# Stealth Mission 89: slová spoločné pre všetky obrazovky.
#
# Zdrojom každého prekladu je angličtina. Správa, ktorá v jazyku chýba, sa zobrazí po
# anglicky. Vždy po anglicky: čo ukazuje kokpit (označenia HUD, MFD a panelov), skratky
# (FLIR, ILS, CCIP, SAM, RWR...), názvy lietadiel, zbraní a miest
# (fichiers_md/translation.md a glossary.md v priečinku každého jazyka).
#
# Syntax Fluent: riadok viacriadkového textu nesmie začínať znakmi "[", "*" ani ".";
# zložené zátvorky sa píšu {"{"} a {"}"}.

language = Jazyk
    .facts =
        Jazyk ponúk, tooltipov, briefingov a hlásení
        Označenia samotného kokpitu (HUD, MFD, panely) zostávajú po anglicky ako v skutočnom lietadle

## Tlačidlá

back = SPÄŤ
    .facts = Späť na predchádzajúcu obrazovku
ok = OK
cancel = ZRUŠIŤ
close = ZAVRIEŤ
continue = POKRAČOVAŤ
yes = ÁNO
no = NIE
on = ZAP
off = VYP
classic = KLASICKÝ
modern = MODERNÝ

## Skúšky prekladového systému

i18n-test-count = { $n ->
    [one] { $n } zásah
    [few] { $n } zásahy
    [many] { $n } zásahu
   *[other] { $n } zásahov
}
i18n-test-lines =
    Prvý riadok
    Druhý riadok
i18n-test-nested = Vyberte: { $what }
