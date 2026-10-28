# Polvilauta (src/kneeboard.rs, viivafontti), tehtäväsuunnittelija (src/planner.rs) ja
# tallennetut tehtävät (src/savegame.rs). Navigoinnin lyhenteet (PT, BRG, ETA, PRI, SEC, KM, KT)
# pysyvät englanniksi.

kb-brief = BRIEF
kb-route = REITTI
kb-map = KARTTA
kb-target = KOHDE
kb-keys = NÄPPÄIMET
kb-tip = Kartat, tehtävämuistiinpanot ja näppäinopas
kb-geran-file = GERAN-2 (KANSIO)
kb-objectives = TAVOITTEET
kb-primary = PÄÄTAVOITE: { $objective }
kb-secondary = SIVUTAVOITE: { $objective }
kb-times = AJAT
kb-deadline = MÄÄRÄAIKA { $time }
kb-now = NYT { $time }
kb-conditions = OLOSUHTEET
kb-moon = KUU: { $phase } ({ $lit } % VALAISTU)
kb-armament = ASEISTUS
kb-roe = TULENAVAUSSÄÄNNÖT
kb-total = YHTEENSÄ { $km } KM, { $kt } KT
kb-fuel = POLTTOAINE { $lbs } LBS
kb-preparing = VALMISTELLAAN...
kb-airborne = ILMASSA
kb-developing = KEHITETÄÄN...
kb-no-photo = EI KUVAA
kb-airborne-target = ILMAKOHDE
kb-scroll-vr = VIERITÄ: PEUKALOSAUVA, SORMI TAULUN PÄÄLLÄ
kb-scroll-desktop = VIERITÄ: HIIREN PYÖRÄ TAULUN PÄÄLLÄ

## Tehtäväsuunnittelija

planner-low = MATALA 500 FT
planner-medium = KESKI 5000 FT
planner-high = KORKEA 20000 FT
planner-title = TEHTÄVÄSUUNNITTELIJA
planner-route = Reitti
planner-base = TUKIKOHTA
planner-total = YHTEENSÄ { $km } KM  { $time }
planner-briefed = Annettu reitti { $km } km, polttoaine { $fuel } lbs
planner-coverage = Tutkakattavuus korkeudella
    .note = Tutkakattavuus olettaa 480 kt tasalennon ja ruumaluukut kiinni.
planner-detected = Reitistä havaittu { $km } km
planner-pulse = Pulssitutkan (P) havaintokantama
planner-doppler = Doppler-tutkan (D) havaintokantama
planner-missile = Ohjuksen kantama
planner-uav = Vihollisen UAV
planner-edit-hint = Vedä pistettä siirtääksesi sitä, karttaa panoroidaksesi. Napsauta osuutta lisätäksesi pisteen, oikea napsautus poistaa sen. Pyörä: zoom.
planner-reset = NOLLAA
planner-view-hint = Vedä karttaa panoroidaksesi. Pyörä: zoom.

## Tallennetut tehtävät

saves-title = TALLENNETUT TEHTÄVÄT
saves-subtitle = Tallennettu lennossa valikosta: TALLENNA TEHTÄVÄ
saves-none = Ei tallennettuja tehtäviä
saves-line = { $theater }  -  { $aircraft }  -  tehtävän aika { $time }
saves-saved-at = Tallennettu { $date }
saves-unreadable = Lukukelvoton tallennus (toinen versio)
saves-delete =
    .facts = Poista tämä tallennus
saves-back =
    .facts = Takaisin yksittäisen tehtävän näyttöön
