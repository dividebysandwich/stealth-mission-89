# Menu utama dan layar misi tunggal (src/menu.rs).

## Teater operasi (stealth_host::theater::THEATERS)

theater-libya = Libya (1986)
    .facts =
        Teluk Sidra dan pesisir Libya
        Era Operasi El Dorado Canyon
        Langit gurun cerah hampir setiap hari
theater-persian_gulf = Teluk Persia (1984)
    .facts =
        Era Perang Tanker Iran-Irak
        Di atas laut dan gurun pesisir
        Cuaca paling cerah dari semua teater operasi
theater-north_cape = Tanjung Utara
    .facts =
        Norwegia Arktik dan Semenanjung Kola
        Markas Armada Utara Soviet
        Awan rendah menutup langit hampir sepanjang waktu
theater-central_europe = Eropa Tengah
    .facts =
        NATO melawan Pakta Warsawa di atas Jerman
        Langit cerah hanya 15% dari waktu
theater-vietnam = Vietnam
    .facts =
        Wilayah Hanoi dan Haiphong
        Medan hutan dan pegunungan
        Badai petir pada 30% misi
theater-middle_east = Timur Tengah
    .facts =
        Mediterania timur dan Levant
        Sebagian besar langit cerah
theater-desert_storm = Badai Gurun
    .facts =
        Kuwait dan Irak, 1991
        Debut tempur sesungguhnya F-117A
        Sebagian besar langit gurun cerah
theater-cuba = Kuba
    .facts =
        Teater operasi pulau di Karibia
        Sasaran laut dan darat
        Tropis: badai pada 15% misi
theater-korea = Korea
    .facts =
        Semenanjung Korea di utara DMZ
        Medan pegunungan
        Cuaca berubah-ubah, langit apa pun mungkin terjadi
theater-ukraine = Ukraina (2026)
    .facts =
        Ukraina timur, Krimea dan Rusia selatan
        Garis depan dari Kharkiv sampai Kherson terbakar
        Lahan pertanian terbuka, kota-kota hancur di garis depan

## Situasi (aturan keterlibatan)

roe-0 = Perang Dingin
    .facts =
        Operasi rahasia di masa damai
        Tidak terdeteksi adalah yang terpenting
        Lebih sedikit pesawat musuh berpatroli
        Aturan keterlibatan yang tepat ada di briefing
roe-1 = Perang Terbatas
    .facts =
        Konflik bersenjata regional
        Pertahanan udara musuh siaga
        Pesawat musuh lebih banyak daripada Perang Dingin
roe-2 = Perang Konvensional
    .facts =
        Perang skala penuh
        Semua pasukan musuh aktif
        Pesawat musuh lebih banyak daripada Perang Dingin

## Jenis misi

kind-strike = Serangan
    .facts =
        Hancurkan, foto atau capai sasaran darat
        Sasaran utama dan sekunder
        Persenjataan udara-ke-darat
kind-air = Udara-ke-udara
    .facts =
        Cegat pesawat musuh
        Pesawat angkut, pembom atau patroli tempur
        Persenjataan rudal udara-ke-udara
kind-strike-training = Latihan serangan
    .facts =
        Misi serangan latihan
        Pelajari senjata dan navigasi
kind-air-training = Latihan udara
    .facts =
        Misi udara-ke-udara latihan
        Pelajari intersepsi dan rudal
kind-drone-defense = Pertahanan drone
    .facts =
        Hanya Ukraina
        Hentikan gelombang drone serang Geran sebelum menghantam kota atau pembangkit nuklir
        Lalu bom lokasi peluncurannya

## Lawan

opponents-0 = Pemula
    .facts =
        Awak musuh yang belum berpengalaman
        Pengaturan paling mudah
opponents-1 = Reguler
    .facts =
        Awak musuh rata-rata
        Pengaturan bawaan
opponents-2 = Veteran
    .facts =
        Awak musuh yang terampil
        Lebih banyak kapal dan patroli
opponents-3 = Elite
    .facts =
        Pilot musuh dan awak SAM terbaik
        Pengaturan tersulit

## Cuaca

sky-random = Acak
    .facts =
        Diundi dari iklim teater operasi
        Berubah di setiap misi baru
        Awan menghalangi garis pandang kedua pihak
sky-clear = Cerah
    .facts =
        Tanpa awan
        Jarak pandang 40 km
        Langit klasik era DOS
sky-scattered = Berawan sebagian
    .facts =
        Cumulus 20-45%, dasar 2500-6000 ft
        Kadang ada altostratus tipis di atasnya
        Jarak pandang 30 km
sky-broken = Berawan banyak
    .facts =
        Cumulus 45-70%, dasar 2000-5000 ft
        Sering ada lapisan stratocumulus di atasnya
        Jarak pandang 25 km
sky-overcast = Mendung
    .facts =
        Stratus 80-100%, dasar 1500-5000 ft
        Beberapa serpihan cumulus di bawahnya
        Jarak pandang 20 km
sky-storm = Badai
    .facts =
        Menara cumulonimbus hingga 40000 ft
        Stratus dan cumulus rendah mulai 1000 ft
        Jarak pandang 8 km
menu-random-cover = Tutupan acak
    .facts =
        Tutupan lapisan awan utama diundi sesuai cuaca
        Hapus centang untuk mengatur sendiri
        Lapisan utama: stratus untuk mendung dan badai, selain itu cumulus
menu-cover-slider =
    .facts =
        Bagian langit yang ditutupi lapisan utama
        Tidak berpengaruh pada langit cerah

## Bulan

moon-date = Tanggal misi
    .facts =
        Bulan asli pada tanggal briefing
        Membutuhkan sakelar waktu hari (modern)
moon-0 = Bulan baru
    .facts =
        Tanpa cahaya bulan: malam gelap gulita, hanya lampu dan bintang
        Pilot musuh melihat Anda pada jarak 2 km
moon-1 = Sabit awal
    .facts = Bulan tipis di langit senja, terbenam sebelum tengah malam
moon-2 = Kuartal pertama
    .facts = Separuh terang, naik di senja hari, terbenam sekitar tengah malam
moon-3 = Cembung awal
    .facts = Terang, naik sejak sore sampai lewat tengah malam
moon-4 = Bulan purnama
    .facts =
        Terbit sepanjang malam, tinggi di tengah malam
        Pilot musuh melihat Anda pada jarak 4 km
moon-5 = Cembung akhir
    .facts = Terang, terbit di malam hari, terlihat sampai pagi
moon-6 = Kuartal terakhir
    .facts = Separuh terang, terbit sekitar tengah malam
moon-7 = Sabit akhir
    .facts = Bulan tipis terbit sebelum fajar

## Pesawat

aircraft-f117 =
    .facts =
        Pesawat serang siluman
        Subsonik, 7 g
        Kokpit modern dan klasik
aircraft-f16 =
    .facts =
        Pesawat tempur multiperan
        Kelas Mach 2, 9 g, roll cepat
        Kokpit F-16 dengan RWR
aircraft-tornado =
    .facts =
        Pesawat serang terbang rendah
        Sayap ayun menyapu sesuai kecepatan
        Kokpit modern, tiga MFD
aircraft-a10 =
    .facts =
        Pesawat dukungan udara jarak dekat
        Lambat dan tangguh, kanon 30 mm
        Kokpit A-10C, dua MFD
aircraft-av8b =
    .facts =
        Pesawat serang V/STOL
        Dorongan vektor: hover, mendarat di mana saja
        Tuas nozzle: R / F, Shift untuk penahan
aircraft-uh60 =
    .facts =
        Helikopter serbaguna
        Menurunkan pasukan: mendarat dekat sasaran
        Cyclic dan collective
aircraft-ah1z =
    .facts =
        Helikopter serang
        Kanon turret, Hellfire, AIM-9
        Kanon dan Hellfire mengikuti pod

## Bagian dan kelompok

section-theater = Teater operasi
section-opponents = Lawan
section-situation = Situasi
section-mission = Misi
section-moon = Bulan
section-start-time = Waktu mulai
section-weather = Cuaca
section-realism = Realisme
section-tacview = Tacview
section-mission-summary = Ringkasan misi
section-briefing = Briefing
group-mission = Misi
group-conditions = Kondisi
group-vehicle = Pesawat
group-simulation = Simulasi

## Opsi misi

menu-start-random = Acak
    .facts =
        Diundi di setiap misi baru, seperti pada mode klasik
        06:00-08:00 pada siang hari atau 18:00-20:00 pada malam hari
        Hapus centang untuk memilih jam
menu-start-slider =
    .facts =
        Waktu lepas landas
        Malam dari 18:00 sampai 06:00
        Dipertahankan oleh MISI BARU
menu-realistic-weapons = Senjata realistis
    .facts =
        Batas pesawat aslinya
        2 bay senjata, bukan 4
        Tanpa rudal AIM-9 atau AIM-120
        Tanpa lepas landas atau pendaratan di kapal induk
menu-crash-protection = Proteksi tabrakan
    .facts =
        Menarik naik otomatis di dekat tanah (roda pendarat naik)
        Menyentuh tanah dan bertabrakan tidak fatal
        Nonaktif: tanah selalu bisa tertabrak
menu-tacview-export = Ekspor
    .facts =
        Menyimpan setiap penerbangan sebagai rekaman Tacview (.acmi)
        Di folder file game Anda, di bawah tacview
menu-tacview-real-time = Telemetri real-time
    .facts =
        Mengirim penerbangan ke Tacview selama Anda terbang
        Di Tacview: Record > Real-Time Telemetry
        Alamat 127.0.0.1, port 42674 (--tacview-address)
switch-game = GAME
switch-real = NYATA
menu-damage = Kerusakan
    .facts =
        Kerusakan senjata
        Game: kerusakan klasik; tembakan kena menumpuk tetapi tidak pernah menjatuhkan pesawat
        Realistis: satu tembakan rudal bisa menghancurkan pesawat (eject!), merusak lebih banyak sistem dan sering memicu kebakaran, yang padam sendiri atau meledakkan pesawat
        Realistis: permukaan kendali yang rusak membuat pesawat keluar dari trim atau berosilasi
menu-guidance = Pemanduan
    .facts =
        Pemanduan rudal
        Klasik: rudal terbang lurus ke sasarannya
        Realistis: navigasi proporsional ke titik tumbukan; motor habis terbakar dan rudal meluncur, kehilangan kecepatan di setiap belokan (asap hanya selama motor menyala)
        Realistis: SAM melambung melawan sasaran jauh; AIM-120 terbang lurus selama 3 s (tarik naik untuk melambungkannya)
menu-air-defence = Pertahanan udara
    .facts =
        Pertahanan udara ketinggian rendah
        Klasik: hanya situs SAM dan pesawat tempur
        Realistis: kanon ZSU-23-4 dengan radar (radarnya melihat Anda seperti radar SAM) dan baterai flak S-60 menjaga pangkalan udara dan sasaran; regu MANPADS di kota menembakkan SA-14 ke apa yang mereka lihat
        Terbang rendah di atas tempat yang dijaga menjadi berbahaya; bom dan kanon menghancurkan meriam-meriam itu

## Layar utama

menu-title = STEALTH MISSION 89
menu-subtitle = SISTEM PERENCANAAN MISI
menu-single-mission = MISI TUNGGAL
    .facts =
        Satu misi di teater operasi pilihan Anda
        Pilih opsi, rencanakan rute dan persenjataan
menu-career = KARIER
    .facts =
        Catatan pilot: pangkat, tanda jasa, sortie
        Masa dinas: 12 sortie di satu teater operasi
        Apa yang Anda hancurkan tetap hancur
menu-campaign = KAMPANYE DINAMIS
    .facts =
        Kampanye dinamis secara real-time
        Rebut kota dan pangkalan udara musuh
        Misi dari situasi yang sedang berjalan
menu-multiplayer = MULTIPLAYER
    .facts = Koop dalam kampanye dinamis
    .join = Bergabung ke server dari daftar publik
    .host = Bergabung ke server, atau jadi host kampanye dan terbang di dalamnya
menu-options = OPSI
    .facts =
        Game: gaya kokpit, HUD dan avionik, trim otomatis
        Grafik, volume suara, pengaturan tombol dan joystick
        Disimpan untuk penerbangan berikutnya
    .short = Game, grafik, suara dan kontrol
menu-download = UNDUH
    .facts =
        Game desktop untuk Windows, Linux dan macOS
        Dengan VR dan multiplayer
        Steam Deck dan Steam Frame: AppImage Linux
web-play = MAIN DI BROWSER
    .facts = Langsung di sini, tanpa instalasi
web-download-platforms = Windows, Linux dan macOS, Steam Deck dan Steam Frame
web-download-version = Versi unduhan
web-gives-vr = VR
    .why = Terbang dalam VR dengan headset apa pun yang kompatibel dengan OpenXR dan Steam Frame
web-gives-hotas = HOTAS
    .why = Dukungan penuh untuk gamepad, joystick, throttle dan pedal
web-gives-multiplayer = Multiplayer lebih baik
    .why = Koneksi UDP langsung ke server untuk lag lebih kecil
web-gives-server = Jadikan server milik Anda
    .why = Jadi host kampanye koop, atau jalankan server khusus
web-gives-head-tracking = Pelacakan kepala
    .why = Pelacak ala TrackIR lewat OpenTrack
web-gives-tacview = Tacview
    .why = Rekaman penerbangan dan telemetri real-time
web-gives-keep = Jadi milik Anda
    .why = Jika situs web mati, Anda tetap punya salinan sendiri
menu-discord = DISCORD
    .facts =
        Terbang bersama, laporkan bug dan bagikan misi
        Ikuti perkembangan
menu-quit = KELUAR
    .facts = Keluar ke desktop
menu-version = VERSI { $version }

## Layar misi tunggal

menu-career-sortie = SORTIE KARIER
menu-tour-sortie = { $pilot }  //  SORTIE MASA DINAS { $sortie } DARI { $length }
menu-msn = SISTEM PERENCANAAN MISI  //  MSN { $seed }
menu-no-targets = Tidak ada lagi sasaran yang tersisa untuk masa dinas ini
menu-plan-route = RENCANAKAN RUTE
    .facts =
        Peta teater operasi dengan cakupan radar dan SAM musuh
        Tambah, pindahkan dan hapus titik navigasi
        Sasaran dan pangkalan pendaratan tetap
menu-loadout = PERSENJATAAN
    .facts =
        Pilih persenjataan di setiap bay atau pylon
        Dimulai dengan senjata dari briefing
menu-generating = MEMBUAT...
menu-back-records = Kembali ke catatan pilot
menu-back-main = Kembali ke menu utama
menu-new-sortie = SORTIE BARU
menu-new-mission = MISI BARU
    .facts =
        Mengundi sasaran yang berbeda
        Mempertahankan semua opsi dan waktu mulai
menu-load = MUAT
    .facts =
        Misi yang disimpan saat terbang (menu: SIMPAN MISI)
        Lanjut terbang dari tempat penyimpanan
menu-fly = TERBANG
    .facts = Lepas landas untuk misi dalam briefing
menu-fly-vr = TERBANG DALAM VR
    .facts =
        Memulai ulang game di headset VR
        Menerbangkan misi yang persis sama
    .no-mission = Buat misi terlebih dahulu
    .no-runtime = Tidak ada runtime OpenXR (Monado, WiVRn, SteamVR) yang aktif
menu-vr-failed = tidak dapat memulai dalam VR: { $error }

## Ringkasan misi

summary-theater = TEATER
summary-situation = SITUASI
summary-mission = MISI
summary-opponents = LAWAN
summary-time = WAKTU
summary-weather = CUACA
summary-fuel = BAHAN BAKAR
summary-night = Malam
summary-day = Siang
summary-time-value = { $daytime }  berangkat { $depart }
summary-weather-value = { $sky }  jarak pandang { $vis } km
summary-fuel-value = { $fuel } lbs
summary-armament = Persenjataan: { $stores }
summary-intelligence = INTELIJEN
aircraft-rah66 =
    .facts =
        Stealth scout helicopter
        Bays: Hellfires or Stingers
        Hard to see on radar, slow or hovering
