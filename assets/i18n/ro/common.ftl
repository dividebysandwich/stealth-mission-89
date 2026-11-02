# Stealth Mission 89: cuvinte comune tuturor ecranelor.
#
# Engleza este sursa oricărei traduceri. Un mesaj lipsă într-o limbă este afișat în engleză.
# Rămân în engleză peste tot: ce afișează cockpitul (HUD, MFD și etichetele panourilor),
# acronimele (FLIR, ILS, CCIP, SAM, RWR...), numele aeronavelor, armelor și locurilor
# (fichiers_md/translation.md și glossary.md în dosarul fiecărei limbi).
#
# Sintaxa Fluent: o linie dintr-un text pe mai multe linii nu trebuie să înceapă cu "[", "*"
# sau "."; acoladele se scriu {"{"} și {"}"}.

language = Limba
    .facts =
        Limba meniurilor, a tooltip-urilor, a briefing-urilor și a mesajelor
        Etichetele cockpitului (HUD, MFD, panouri) rămân în engleză, ca pe aeronava reală

## Butoane

back = ÎNAPOI
    .facts = Înapoi la ecranul anterior
ok = OK
cancel = ANULEAZĂ
close = ÎNCHIDE
continue = CONTINUĂ
yes = DA
no = NU
on = PORNIT
off = OPRIT
classic = CLASIC
modern = MODERN

## Teste ale sistemului de traducere

i18n-test-count = { $n ->
    [one] { $n } lovitură
    [few] { $n } lovituri
   *[other] { $n } de lovituri
}
i18n-test-lines =
    Prima linie
    A doua linie
i18n-test-nested = Alegeți: { $what }
