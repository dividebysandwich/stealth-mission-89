# Kariery pilotów (src/career.rs): stopnie, odznaczenia, tura służby.

## Stopnie US Air Force (przetłumaczone na odpowiedniki Sił Powietrznych), skróty

rank-0 = Podporucznik
    .short = ppor.
rank-1 = Porucznik
    .short = por.
rank-2 = Kapitan
    .short = kpt.
rank-3 = Major
    .short = mjr
rank-4 = Podpułkownik
    .short = ppłk
rank-5 = Pułkownik
    .short = płk

## Odznaczenia (medale amerykańskie: ich nazwy własne)

medal-air-medal = Air Medal
medal-distinguished-flying-cross = Distinguished Flying Cross
medal-silver-star = Silver Star
medal-air-force-cross = Air Force Cross
medal-medal-of-honor = Medal of Honor
medal-purple-heart = Purple Heart
medal-prisoner-of-war-medal = Prisoner of War Medal

## Wiadomości po locie bojowym

career-awarded = Odznaczenie: { $medal }
career-kia = Poległ w walce
    .short = KIA
career-pow = Jeniec wojenny
    .short = POW
career-active = W czynnej służbie
career-captured = Wzięty do niewoli: koniec kariery
career-tour-sortie = Tura służby: lot bojowy { $sortie } z { $length }
career-tour-complete = Tura służby zakończona: { $points } pkt (+{ $bonus })
career-promoted = Awans na stopień: { $rank }

## Okno kariery

career-title = AKTA PILOTÓW
career-group-tour = Tura
career-set-by-tour = Ustalone przez turę pilota
career-pilots = Piloci
career-new-pilot = Nowy pilot
career-name = Nazwisko
career-enlist = POWOŁAĆ
career-select = Wybrać lub powołać pilota
career-service-record = Przebieg służby
career-rank = STOPIEŃ
career-points = PUNKTY
career-sorties = LOTY BOJOWE
career-tours = TURY
career-status = STATUS
career-decorations = Odznaczenia
career-tour-of-duty = Tura służby
career-tour-line = { $theater }  { $situation }  lot bojowy { $sortie } z { $length }  { $points } pkt
career-destroyed = Dotychczas zniszczone cele: { $count }
career-fly-next = WYKONAĆ NASTĘPNY LOT BOJOWY
career-new-tour = { $theater }, { $situation }, lotów bojowych: { $count }, poziom przeciwników: { $opponents }
    .facts = Co zostanie zniszczone, pozostaje zniszczone do końca tury
career-start-tour = ROZPOCZĄĆ TURĘ
career-retired = W stanie spoczynku
career-last-sorties = Ostatnie loty bojowe
