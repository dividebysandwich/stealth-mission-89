# Stealth Mission 89: besede, skupne vsem zaslonom.
#
# Angleščina je izvor vseh prevodov. Sporočilo, ki v jeziku manjka, se prikaže v angleščini.
# Povsod ostane v angleščini: kar kaže kabina (oznake HUD, MFD in plošč), kratice (FLIR, ILS,
# CCIP, SAM, RWR...), imena letal, orožij in krajev (fichiers_md/translation.md in
# glossary.md v mapi vsakega jezika).
#
# Sintaksa Fluent: vrstica večvrstičnega besedila se ne sme začeti z "[", "*" ali ".";
# zavite oklepaje pišemo {"{"} in {"}"}.

language = Jezik
    .facts =
        Jezik menijev, tooltipov, briefingov in sporočil
        Oznake v kabini (HUD, MFD, plošče) ostanejo v angleščini, kot v pravem letalu

## Gumbi

back = NAZAJ
    .facts = Nazaj na prejšnji zaslon
ok = V REDU
cancel = PREKLIČI
close = ZAPRI
continue = NADALJUJ
yes = DA
no = NE
on = VKLOP
off = IZKLOP
classic = KLASIČNO
modern = SODOBNO

## Preizkusi prevajalskega sistema

i18n-test-count = { $n ->
    [one] { $n } zadetek
    [two] { $n } zadetka
    [few] { $n } zadetki
   *[other] { $n } zadetkov
}
i18n-test-lines =
    Prva vrstica
    Druga vrstica
i18n-test-nested = Izberite: { $what }
