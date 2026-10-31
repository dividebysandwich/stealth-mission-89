# An kneeboard (src/kneeboard.rs, clófhoireann stróc), pleanálaí na misean (src/planner.rs) agus na
# misin shábháilte (src/savegame.rs). Fanann giorrúcháin loingseoireachta (PT, BRG, ETA, PRI, SEC,
# KM, KT) i mBéarla.

kb-brief = BRIEF
kb-route = BEALACH
kb-map = MAPA
kb-target = SPRIOC
kb-keys = EOCHRACHA
kb-tip = Mapaí, nótaí misin agus an treoir eochracha
kb-geran-file = GERAN-2 (COMHAD)
kb-objectives = CUSPÓIRÍ
kb-primary = PRÍOMH: { $objective }
kb-secondary = TÁNAISTEACH: { $objective }
kb-times = AMANNA
kb-deadline = TÉARMA { $time }
kb-now = ANOIS { $time }
kb-conditions = CUMAINN
kb-moon = GEALACH: { $phase } ({ $lit }% SOLASTA)
kb-armament = ARMLÓN
kb-roe = RIALACHA CATHAITHE
kb-total = IOMLÁN { $km } KM AG { $kt } KT
kb-fuel = BREOSLA { $lbs } LBS
kb-preparing = AG ULLMHÚ...
kb-airborne = SAN AER
kb-developing = AG FORBAIRT...
kb-no-photo = GAN GRIANGHRAF
kb-airborne-target = SPRIOC SAN AER
kb-scroll-vr = SCROLL: LUAMHÁN, MÉAR AR AN GCLÁR
kb-scroll-desktop = SCROLL: ROTA NA LUCHÓIGE THAR AN GCLÁR

## Pleanálaí na misean

planner-low = ÍSEAL 500 FT
planner-medium = MEÁNACH 5000 FT
planner-high = ARD 20000 FT
planner-title = PLEANÁLAÍ NA MISEAN
planner-route = Bealach
planner-base = BUNÁIT
planner-total = IOMLÁN { $km } KM  { $time }
planner-briefed = Bealach an bhriefing { $km } km, breosla { $fuel } lbs
planner-coverage = Clúdach radair do
    .note = Glacann clúdach radair le heitilt chothrom 480kt, doirse na báe dúnta.
planner-detected = { $km } km den bhealach braite
planner-pulse = Raon braite radair bhíoma (P)
planner-doppler = Raon braite radair Doppler (D)
planner-missile = Raon diúracáin
planner-uav = UAV namhad
planner-edit-hint = Tarraing pointe chun é a bhogadh, an mhapa chun pan a dhéanamh. Cliceáil ar chos chun pointe a chur leis, cliceáil dheas chun é a bhaint. Roth: zúmáil.
planner-reset = ATHSHOCRÚ
planner-view-hint = Tarraing an mhapa chun pan a dhéanamh. Roth: zúmáil.
planner-layers = SRAITHEANNA
planner-layer-sams = Radar / suíomh SAM
planner-radar-types = Cineálacha radair
planner-layer-pulse = Raon braite radair bhíoma
planner-layer-doppler = Raon braite radair Doppler
planner-layer-airfield = Aerpháirc

## Misin shábháilte

saves-title = MISIN SHÁBHÁILTE
saves-subtitle = Sábháilte i rith eitilte ón roghchlár: SÁBHÁIL AN MISEAN
saves-none = Gan misean sábháilte
saves-line = { $theater }  -  { $aircraft }  -  am misin { $time }
saves-saved-at = Sábháilte { $date }
saves-unreadable = Sábháil ólégtha (leagan eile)
saves-delete =
    .facts = Scrios an tsábháil seo
saves-back =
    .facts = Ar ais go dtí scáileán an aon mhisin
