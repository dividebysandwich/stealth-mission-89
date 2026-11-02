# Stealth Mission 89: kõigile ekraanidele ühised sõnad.
#
# Inglise keel on iga tõlke allikas. Puuduv sõnum kuvatakse inglise keeles. Kõikjal inglise
# keeles: kõik, mida kokpit näitab (HUD, MFD ja paneeli sildid), lühendid (FLIR, ILS, CCIP,
# SAM, RWR...), lennukite, relvade ja kohtade nimed (fichiers_md/translation.md ja
# glossary.md iga keele kaustas).
#
# Fluenti süntaks: mitmerealise teksti rida ei tohi algada märgiga "[", "*" ega ".";
# looksulud kirjutatakse {"{"} ja {"}"}.

language = Keel
    .facts =
        Menüüde, tooltipide, briifingute ja teadete keel
        Kokpiti enda sildid (HUD, MFD, paneel) jäävad inglise keelde, nagu päris lennukis

## Nupud

back = TAGASI
    .facts = Tagasi eelmisele ekraanile
ok = OK
cancel = TÜHISTA
close = SULGE
continue = JÄTKA
yes = JAH
no = EI
on = SEES
off = VÄLJAS
classic = KLASSIKALINE
modern = KAASAEGNE

## Tõlkesüsteemi testid

i18n-test-count = { $n ->
    [one] { $n } tabamus
   *[other] { $n } tabamust
}
i18n-test-lines =
    Esimene rida
    Teine rida
i18n-test-nested = Vali: { $what }
