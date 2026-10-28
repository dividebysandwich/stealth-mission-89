# Fuinneog an fheachtais (src/campaign/menu.rs), fuinneog an iltoiseora (src/multiplayer_menu.rs),
# finscéal mhapa an chogaidh agus an radharc beo (src/campaign/picture.rs, unit_view.rs).

delete = SCRIOS
delete-campaign = AN FEACHTAS SEO A SCRIOS?
delete-save = AN TSÁBHÁIL SEO A SCRIOS?
delete-permanent = Is buan an scriosadh

## Fuinneog an fheachtais

cm-group-campaign = Feachtas
cm-weather-dynamic = Dinimiciúil
    .facts =
        Athraíonn an aimsir gach 10-22 uair de chloc na spéire
        Cuireann scamaill bac ar líne radhairc an dá thaobh
cm-weather-clear = Spéir ghlan
    .facts = Gan scamaill ar feadh an fheachtais ar fad
cm-section-day-night = Lá agus oíche
cm-day-real = Fíor-am
    .facts = Maireann lá 24 uair
cm-day-2h = 2 u / lá
    .facts = Maireann lá 2 uair
cm-day-1h = 1 u / lá
    .facts = Maireann lá 1 uair
cm-day-30min = 30 nóim / lá
    .facts = Maireann lá 30 nóiméad
cm-conditions-note = Baineann na cumainn le feachtas nua
cm-rules-note = Coinníonn feachtas sábháilte na rialacha lenar tosaíodh é
cm-about =
    Imíonn an cogadh i bhfíor-am agus tú ag imirt, fiú agus an roghchlár oscailte.
    Eitil na misin ar an gclár: taiscéalaíocht, SEAD, CAP, CAS, frithlong, stailc, coimhdeacht.
    Gabhann insealbhuithe héileacaptair cathracha agus aerfoirt an namhad.
    Ní mharaítear ná ní ghabhtar thú choíche: tar éis eitilte, tá tú ar ais ag an gclár.
cm-section-new = Feachtas nua
cm-new-line = { $theater }  -  { $situation }  -  naimhde: { $opponents }
cm-start = TÚS
cm-section-continue = Lean ar aghaidh
cm-no-saves = Gan feachtas sábháilte
cm-played = { $hours } u imeartha
cm-save-gone = Tá an tsábháil imithe
cm-save-unreadable = Níorbh fhéidir an tsábháil a léamh: { $error }
cm-save-undeletable = Níorbh fhéidir an tsábháil a scriosadh: { $error }
cm-in-vr = I VR
cm-delete =
    .facts = Scrios an feachtas sábháilte seo

## Mapa an chogaidh

pic-friendly-aircraft = Aerárthach cairdiúil
pic-friendly-helicopter = Héileacaptar cairdiúil
pic-enemy-fighter = Trodaire namhad (ar radar)
pic-sam-known = Suíomh SAM (ar eolas)
pic-guns-known = Gunnaí / MANPADS (ar eolas)
pic-reconned = Taiscéalaithe
pic-you = Tusa
pic-site = Suíomh: { $place }
uv-live = { $unit } - BEO
uv-contact-lost = CAILLEADH AN TEAGMHÁIL
uv-hint = Tarraing: fithisigh  Roth: zúmáil

## Iltoiseoir

mp-disconnected = Dícheangailte: { $why }
mp-connecting = Ag nascadh le { $address }...
mp-joining = Ceangailte le { $address }, ag teacht isteach...
mp-browser-public-only = ní féidir ach le freastalaithe ar an liosta poiblí a bheith páirteach ón mbrabhsálaí
mp-title-host = ILTOISEOIR: ÓSTÁIL CLUICHE
mp-subtitle = Comhoibriú sa bhfeachtas dinimiciúil: eitlígí an cogadh le chéile
mp-your-name = D'ainm
    .hint = Ainm an phíolóta
mp-join-address = Teacht isteach le seoladh
    .hint = seoladh nó ainm[:port]
mp-connect = NASC
mp-server-name = Ainm an fhreastalaí
mp-udp-port = Port UDP
mp-address-note = Tagann imreoirí eile isteach le seoladh an ríomhaire seo; oscail an port UDP i do ródaire chun imirt thar an idirlíon
mp-list-public = Liostáil ar liosta poiblí na bhfreastalaithe (aimsíonn daoine eile an cluiche ina liosta)
mp-rules = Rialacha do chách
mp-time-of-day = Am an lae (oícheanta gealaí)
mp-realistic-weapons = Airm réalaíocha F-117A
mp-friendly-fire = Tine chairdiúil
mp-day-night = Lá agus oíche:
mp-weather = Aimsir:
mp-host-new = ÓSTÁIL NUA
mp-host-saved = Óstáil feachtas sábháilte
mp-host = ÓSTÁIL
mp-host-game = ÓSTÁIL CLUICHE
mp-port-error = Port UDP { $port }: { $error }
mp-default-server = Feachtas { $pilot }
mp-public-servers = Freastalaithe poiblí
mp-refresh = ATHNUAIGH
mp-server-count = { $servers } freastalaí, { $pilots } píolóta
mp-list-error = Níl liosta na bhfreastalaithe inrochtana: { $error }
mp-no-servers = Gan freastalaí poiblí faoi láthair
mp-col-server = FREASTALAÍ
mp-col-pilots = PÍOLÓTAÍ
mp-col-war = COGADH
mp-dedicated = TIOMNAITHE
mp-other-version = leagan eile
mp-join = TEACHT ISTEACH
