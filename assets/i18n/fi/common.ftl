# Stealth Mission 89: kaikille näytöille yhteiset sanat.
#
# Englanti on kaikkien käännösten lähde. Puuttuva viesti näytetään englanniksi. Englanniksi
# jäävät kaikkialla: ohjaamon omat tekstit (HUD, MFD ja paneelien merkinnät), lyhenteet (FLIR,
# ILS, CCIP, SAM, RWR...), koneiden, aseiden ja paikkojen nimet
# (fichiers_md/translation.md ja glossary.md kunkin kielen kansiossa).
#
# Fluent-syntaksi: monirivisen tekstin rivi ei saa alkaa merkillä "[", "*" tai "."; aaltosulkeet
# kirjoitetaan {"{"} ja {"}"}.

language = Kieli
    .facts =
        Valikoiden, työkaluvihjeiden, tehtävänannon ja viestien kieli
        Ohjaamon omat merkinnät (HUD, MFD, paneeli) pysyvät englanniksi kuten oikeassa koneessa

## Painikkeet

back = TAKAISIN
    .facts = Takaisin edelliseen näyttöön
ok = OK
cancel = PERUUTA
close = SULJE
continue = JATKA
yes = KYLLÄ
no = EI
on = PÄÄLLÄ
off = POIS
classic = KLASSINEN
modern = MODERNI

## Käännösjärjestelmän testit

i18n-test-count = { $n ->
    [one] { $n } osuma
   *[other] { $n } osumaa
}
i18n-test-lines =
    Ensimmäinen rivi
    Toinen rivi
i18n-test-nested = Valitse: { $what }
