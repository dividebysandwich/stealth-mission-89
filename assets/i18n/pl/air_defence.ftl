# Karta OBRONA PRZECIWLOTNICZA na ekranie operacji (src/campaign/air_defence.rs).

ad-map-hint = Kliknąć stanowisko, aby je wybrać - kółko: powiększenie, przeciąganie: przesunięcie
ad-sites = Stanowiska obrony przeciwlotniczej
ad-none = Brak baterii Patriot i Gepard po naszej stronie
ad-near = w pobliżu: { $place }
ad-enter = WEJŚĆ
ad-destroyed = ZNISZCZONE
ad-drones = DRONY
ad-op = OPER
ad-radar = RDR
ad-ready = GOTOWE
ad-up = W LOCIE
ad-stock = ZAPAS
ad-auto = auto
ad-clear = pusto
ad-ready-patriot = { $ready }/{ $full } { $launchers }W
ad-reloading = { $count ->
    [one] { $count } ładuje
    [few] { $count } ładują
   *[other] { $count } ładuje
}
ad-interceptors = Przechwytujące
ad-you = Ty
ad-a-player = Gracz
ds-new-site = Nowe stanowisko
ds-hint = Wskaż miejsce: UH-60 dostarczą ładunek na lądowisko w ciągu 24 h
ds-click-hint = Kliknij mapę, gdzie ma stanąć - prawy przycisk: anuluj
ds-not-open-ground = Nie na otwartym terenie
ds-new-gepard = Nowe stanowisko Gepard - kliknij, by wskazać
ds-new-patriot = Nowe stanowisko Patriot - kliknij, by wskazać
ds-new-nasams = Nowe stanowisko NASAMS - kliknij, by wskazać
ds-new-drones = Nowe centrum dowodzenia dronami - kliknij, by wskazać
ds-waiting = Czeka na ładunek
ds-fast-track = PRZYSPIESZ ({ $cost })
    .facts = Zbuduj teraz za zasoby zamiast lotów z ładunkiem
    .disabled = Za mało zasobów
ds-fast-track-confirm = PRZYSPIESZYĆ BUDOWĘ?
    .text = Zamierzasz wydać { $cost } zasobów, by od razu zbudować { $site }, bez lotów z ładunkiem. Śmigłowce SI w drodze z ładunkiem wracają do bazy.
ds-fast-track-spend = WYDAJ { $cost } ZASOBÓW
ds-too-steep = Zbyt stromy stok
