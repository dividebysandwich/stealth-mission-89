# Pilotenkarrieren (src/career.rs): Dienstgrade, Auszeichnungen, die Dienstzeit.

## Dienstgrade der US Air Force (durch die Entsprechungen der Luftwaffe), ihre Abkürzungen

rank-0 = Leutnant
    .short = Lt
rank-1 = Oberleutnant
    .short = OLt
rank-2 = Hauptmann
    .short = Hptm
rank-3 = Major
    .short = Maj
rank-4 = Oberstleutnant
    .short = OTL
rank-5 = Oberst
    .short = Oberst

## Auszeichnungen (US-Orden: ihre Eigennamen)

medal-air-medal = Air Medal
medal-distinguished-flying-cross = Distinguished Flying Cross
medal-silver-star = Silver Star
medal-air-force-cross = Air Force Cross
medal-medal-of-honor = Medal of Honor
medal-purple-heart = Purple Heart
medal-prisoner-of-war-medal = Prisoner of War Medal

## Meldungen nach einem Einsatz

career-awarded = Ausgezeichnet mit: { $medal }
career-kia = Im Einsatz gefallen
    .short = KIA
career-pow = In Kriegsgefangenschaft
    .short = POW
career-active = Im aktiven Dienst
career-captured = Gefangen genommen: die Karriere ist beendet
career-tour-sortie = Dienstzeit: Einsatz { $sortie } von { $length }
career-tour-complete = Dienstzeit abgeschlossen: { $points } Punkte (+{ $bonus })
career-promoted = Befördert zum { $rank }

## Das Karrierefenster

career-title = PILOTENAKTEN
career-group-tour = Dienstzeit
career-set-by-tour = Durch die Dienstzeit des Piloten festgelegt
career-pilots = Piloten
career-new-pilot = Neuer Pilot
career-name = Name
career-enlist = AUFNEHMEN
career-select = Wählen Sie einen Piloten oder nehmen Sie einen neuen auf
career-service-record = Dienstakte
career-rank = DIENSTGRAD
career-points = PUNKTE
career-sorties = EINSÄTZE
career-tours = DIENSTZEITEN
career-status = STATUS
career-decorations = Auszeichnungen
career-tour-of-duty = Dienstzeit
career-tour-line = { $theater }  { $situation }  Einsatz { $sortie } von { $length }  { $points } Punkte
career-destroyed = { $count ->
    [one] Bisher { $count } Ziel zerstört
   *[other] Bisher { $count } Ziele zerstört
}
career-fly-next = NÄCHSTEN EINSATZ FLIEGEN
career-new-tour = { $count } Einsätze: { $theater }, { $situation }, Gegner: { $opponents }
    .facts = Was Sie zerstören, bleibt bis zum Ende der Dienstzeit zerstört
career-start-tour = DIENSTZEIT BEGINNEN
career-retired = Aus dem aktiven Dienst ausgeschieden
career-last-sorties = Letzte Einsätze
