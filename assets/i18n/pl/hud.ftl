# Komunikaty gry na HUD i na nakładkach 2D lotu (src/hud, src/combat.rs, src/csar_view.rs,
# src/campaign). HUD wyświetla je wielkimi literami bez znaków diakrytycznych; słowa awioniki
# (CCIP, THR, KM, MAX...) pozostają po angielsku.

hud-no-ejection-seat = BRAK FOTELA WYRZUCANEGO
hud-waypoint-reached = Osiągnięto waypoint { $waypoint }
hud-select-camera = WYBIERZ KAMERĘ
hud-open-bay = OTWÓRZ KOMORĘ
hud-camera-page = STRONA { $page }
hud-target-in-view-shoot = CEL WIDOCZNY - ZDJĘCIE
hud-target-in-view = CEL WIDOCZNY
hud-aim-at-target = NAKIERUJ NA CEL
hud-photo = ZDJĘCIE { $state }
hud-rearmed = Uzbrojono i zatankowano
hud-landed-rearm = Wylądowano - uzupełnij uzbrojenie lub zakończ lot bojowy
hud-objective-achieved = Cel osiągnięty

## Ekran końca misji

end-accomplished = wykonany
end-not-accomplished = niewykonany
end-primary = Cel główny { $result }
end-secondary = Cel drugorzędny { $result }
end-kills = Zniszczenia: samoloty { $air }, cele naziemne { $ground }
end-time = Czas misji { $time }
end-damage = Uszkodzenia: { $hits ->
    [one] { $hits } trafienie
    [few] { $hits } trafienia
    [many] { $hits } trafień
   *[other] { $hits } trafienia
}
end-training = Misja treningowa
end-press-any-key = Naciśnij dowolny klawisz

## Panel zestrzelonego pilota

csar-flare = RACA SYGNAŁOWA
csar-faster = SZYBCIEJ
csar-normal = CZAS NORMALNY
csar-skip = POMINĄĆ EWAKUACJĘ
csar-parachute = SPADOCHRON OTWARTY
csar-on-ground = NA ZIEMI
csar-down = NA ZIEMI NA TERYTORIUM WROGA
csar-pickup = RATUNEK  UH-60 NAD POZYCJĄ - PODJĘCIE
csar-inbound = RATUNEK  UH-60 { $km } KM { $direction } - W DRODZE
csar-fire-flare = RATUNEK  UH-60 { $km } KM { $direction } - ODPAL RACĘ, GDY BĘDZIE BLISKO
csar-enemy = WRÓG     { $count } HIND, NAJBLIŻSZY { $km } KM
csar-flares = RACE     { $count }      NA ZIEMI OD { $time }

## Powiadomienia trybu kooperacji na panelu (src/player_notices.rs), uwaga konsoli obrony
## powietrznej

notice-joined = { $pilot } dołącza do gry
notice-left = { $pilot } opuszcza grę
notice-lost-connection = { $pilot } traci połączenie
st-select-track = Najpierw wybierz ślad
