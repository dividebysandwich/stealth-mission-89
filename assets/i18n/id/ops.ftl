# Layar Ops kampanye.

ops-thunderstorms = BADAI PETIR
ops-wx-short = WX { $now }, { $next } PADA { $at }
ops-wx-line = { $weather } Berubah menjadi { $next } pada { $at }.
ops-clock = HARI { $day }  { $time }
ops-landed-at = MENDARAT DI { $base }
ops-ground-crew = Kru darat siap
ops-rearm = ISI ULANG PERSENJATAAN & BBM
ops-end-sortie = AKHIRI SORTI
ops-on-server = DI { $server }
ops-serving = MELAYANI
ops-tab-missions = MISI
ops-tab-map = PETA PERANG
ops-tab-air-defence = PERTAHANAN UDARA
ops-tab-forces = PASUKAN
ops-tab-players = PEMAIN
ops-tab-debrief = DEBRIEF
ops-tab-log = LOG
ops-tab-chat = CHAT
ops-blue = BIRU { $count }
ops-red = MERAH { $count }
ops-war-over = Perang telah berakhir
ops-running = Simulasi perang berjalan
ops-save-exit = SIMPAN & KELUAR
ops-disconnect = PUTUSKAN
ops-leave-local = SIMPAN DAN TINGGALKAN KAMPANYE?
    .text = Perang berhenti sampai Anda melanjutkannya. Pesawat yang sedang di udara hilang; penurunan pasukan yang sedang berjalan dimulai ulang dari pangkalannya.
ops-leave-host = SIMPAN DAN HENTIKAN PENYELENGGARAAN KAMPANYE?
    .text = Pemain yang bergabung akan terputus dan perang berhenti sampai Anda melanjutkannya. Pesawat yang sedang di udara hilang; penurunan pasukan yang sedang berjalan dimulai ulang dari pangkalannya.
ops-leave-client = PUTUSKAN DARI SERVER?
    .text = Kampanye terus berjalan di server; tidak ada yang disimpan di sini. Sorti Anda berakhir.
ops-unit-destroyed = { $unit } hancur: kembali ke operasi

## Misi

ops-section-missions = Misi
ops-threat = ANCAMAN { $threat }
ops-no-missions = Tidak ada misi: garis depan tenang
ops-section-underway = Penurunan pasukan dan serangan
ops-enemy-liftoff = MUSUH di { $place }: lepas landas dalam { $mins } mnt
ops-liftoff = { $place }: lepas landas dalam { $mins } mnt
ops-enemy-helis = MUSUH di { $place }: { $count } helikopter mendekat
ops-helis = { $place }: { $count } helikopter dalam perjalanan
ops-raid-sead = SEAD AI di { $place }
ops-raid-drones = DRONE di { $place }
ops-raid = SERANGAN ke pangkalan udara { $place }
ops-raid-launch = { $raid }: berangkat dalam { $mins } mnt
ops-raid-on-way = { $raid }: { $count } dalam perjalanan
ops-resupply-ai = PASOKAN ULANG untuk { $unit }: UH-60 dalam perjalanan
ops-awacs = AWACS MUSUH mengudara { $mins } mnt lagi
ops-down = JATUH: { $pilot } - penyelamatan dalam { $mins } mnt
ops-aboard = DI DALAM: { $pilot } - dalam perjalanan pulang
ops-none-underway = Tidak ada yang berjalan
ops-ai-helicopter = Helikopter AI mengambilnya dalam { $time } kecuali pilot UH-60 melakukannya
ops-defences = Pertahanan: { $sams } situs SAM, ancaman { $threat }
ops-recon-intel = INTEL PENGINTAIAN
ops-order-insertion = PERINTAHKAN PENURUNAN PASUKAN
    .facts = Terbangkan pengawalan untuk melindungi mereka, atau pimpin mereka dengan UH-60
ops-from = Dari
ops-planned-by-group = Direncanakan oleh grup
ops-stores = { $aircraft } - persenjataan: { $stores }
ops-fly-uh60 = TERBANGKAN UH-60
ops-accept = TERIMA MISI
ops-planning = MERENCANAKAN: { $pilots }
ops-flying = TERBANG: { $pilots }
ops-package = PAKET dengan { $kinds }: capai dua jenis di sini dalam { $mins } mnt, +{ $bonus } poin masing-masing
ops-sead-on-way = SEAD AI dalam perjalanan
ops-request-sead = MINTA SEAD AI ({ $cost } materiel, { $aircraft } pesawat)
    .facts = Pesawat serang kawan menyerang situs SAM di tempat ini mendahului Anda
    .disabled = Materiel atau pesawat di darat tidak cukup (lihat PASUKAN)

## Aturan, materiel, pasukan

ops-rule-damage = kerusakan realistis
ops-rule-guidance = pemanduan realistis
ops-rule-no-crashes = tanpa kecelakaan
ops-rule-time = waktu berkelanjutan
ops-rule-weapons = senjata F-117A realistis
ops-rule-friendly-fire = tembakan kawan
ops-rule-classic = klasik
ops-materiel = MATERIEL
ops-materiel-blue = BIRU { $materiel } (+{ $income }/jam)
ops-materiel-red = MERAH { $materiel } (+{ $income }/jam)
ops-friendly = KAWAN
ops-enemy = MUSUH
ops-section-forces = Pasukan
ops-places-held = Tempat dikuasai
ops-materiel-row = Materiel
ops-income = Pemasukan per jam
ops-income-from = dari
ops-income-sources = { $base } + { $places } tempat x { $per_place } + { $cities } kota x { $per_city }
ops-fighters-ground = Pesawat tempur di darat
ops-helicopters-ground = Helikopter di darat
ops-in-air = Di udara
ops-in-air-value = { $fighters } pesawat tempur, { $strike } pesawat serang, { $helicopters } helikopter
ops-since-start = Sejak awal
ops-earned = diperoleh
ops-repairs = perbaikan
ops-operations = sorti, serangan, penurunan pasukan
ops-new-aircraft = pesawat baru
ops-lost = hilang (kota, pilot)
ops-section-how = Cara kerjanya
ops-how =
    Tiap pihak memperoleh materiel setiap jam: jumlah dasar, lebih banyak untuk setiap tempat yang dikuasai dan setiap kota utuh di antaranya. Bom kota-kota musuh (INTERDICTION) untuk memotong pemasukannya.
    AI membayar semua yang dilakukannya: perbaikan, setiap sorti pesawat tempur, serangan, dan penurunan pasukannya sendiri. Jika materiel kurang, kerusakannya tidak diperbaiki dan pesawatnya tetap di pangkalan.
    AI hanya menerbangkan pesawat yang dimilikinya; yang tertembak jatuh hilang, yang mendarat dapat terbang lagi. Setiap jam tiap pihak membeli pesawat baru selama materielnya cukup.
    Anda tidak pernah kekurangan: pesawat Anda, pangkalan Anda (dibom atau tidak), dan helikopter dari perintah penurunan pasukan Anda selalu tersedia.
ops-section-prices = Harga
ops-price-airfield = Perbaikan pangkalan udara
ops-price-sam = Perbaikan situs SAM
ops-price-city = Pembangunan kembali kota
ops-price-ship = Perbaikan kapal
ops-price-gun = Regu kanon atau MANPADS
ops-price-sortie = Sorti pesawat tempur
ops-price-raid = Serangan
ops-price-insertion = Penurunan pasukan AI
ops-price-fighter = Pesawat tempur baru
ops-price-helicopter = Helikopter baru
ops-price-city-lost = Kota hancur (hilang)
ops-price-captured = Pilot tertawan (hilang)

## Peta perang

ops-click-to-watch = { $unit } - klik untuk menonton
ops-section-legend = Legenda
ops-legend-friendly-airfield = Pangkalan udara kawan
ops-legend-enemy-airfield = Pangkalan udara musuh
ops-legend-city = Kota kawan / musuh
ops-watch-hint = Klik unit untuk menontonnya secara langsung
ops-map-hint = Roda: zoom, seret: geser
ops-patriot = Baterai Patriot
ops-manned-by = Dioperasikan oleh { $pilot }
ops-manned-by-other = Dioperasikan oleh pemain lain
ops-control = KENDALIKAN
    .facts = Ambil alih konsolnya

## Debrief, akhir, log, chat

ops-no-sortie = Belum ada sorti yang diterbangkan
ops-section-sortie = Sorti
ops-achieved = SASARAN TERCAPAI
ops-not-achieved = SASARAN TIDAK TERCAPAI
ops-waiting-raid = MENUNGGU SERANGAN
ops-waiting-insertion = MENUNGGU PENURUNAN PASUKAN
ops-flight-time = Waktu terbang { $mins } mnt
ops-kills = Kemenangan: { $air } pesawat, { $ground } di darat
ops-points = { $points } poin
ops-points-bonus = { $points } poin (bonus paket +{ $bonus })
ops-campaign-points = { $rank } - { $points } poin kampanye
ops-campaign-record = Kampanye: { $sorties } sorti, { $won } sasaran tercapai
ops-section-meanwhile = Sementara itu
ops-victory = KEMENANGAN
    .text = Musuh tidak lagi menguasai pangkalan udara: teater operasi ini milik kita.
ops-defeat = KEKALAHAN
    .text = Tidak ada pangkalan udara kawan yang tersisa: teater operasi ini hilang.
ops-end-day = Hari { $day }. Tempat kawan { $friendly }, tempat musuh { $enemy }.
ops-next-campaign = KAMPANYE BERIKUTNYA DIMULAI DALAM { $time }
ops-next-campaign-now = KAMPANYE BERIKUTNYA SEDANG DIMULAI
ops-next-from-menu = Mulai kampanye berikutnya dari menu utama.
ops-section-log = Log kampanye
ops-section-chat = Chat
ops-no-messages = Belum ada pesan
ops-message = Pesan
    .hint = Ketik pesan, Enter untuk mengirim
ops-send = KIRIM

## Briefing

ops-briefing = BRIEFING MISI
ops-briefing-line = Dari { $base } - { $km } km ke target - { $aircraft } - persenjataan: { $stores }
ops-route-locked = Rute terkunci: { $pilots } menerbangkannya
ops-route-threats = RUTE & ANCAMAN
ops-take-off = LEPAS LANDAS
ops-radars-shown = Radar ditampilkan: terdeteksi pesawat kawan, masih berdiri

## Pemain

ops-section-players = Pemain
ops-col-name = NAMA
ops-col-status = STATUS
ops-col-sorties = SORTI
ops-col-points = POIN
ops-col-pilot = PILOT
ops-col-objectives = SASARAN
ops-col-kills = KEMENANGAN UDARA/DARAT
ops-col-medal = MEDALI TERBAIK
ops-you = (Anda)
ops-host = HOST
ops-status-planning = Merencanakan { $title }
ops-status-board = Di papan misi
ops-status-flying = Menerbangkan { $title } - { $aircraft }
ops-status-reconnecting = Koneksi terputus - pesawat ditahan { $seconds } dtk
ops-flown = { $flown } ({ $won } tercapai)
ops-section-leaderboard = Papan peringkat
ops-alone = Terbang sendiri. Untuk terbang bersama, jadikan kampanye ini host lewat MULTIPLAYER di menu utama.
ops-rules = Aturan kampanye ini (dipertahankan sejak awal): { $rules }
ops-no-sorties = Belum ada sorti yang diterbangkan
