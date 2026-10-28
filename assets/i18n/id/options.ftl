# Jendela OPSI (src/options.rs) dan halamannya di menu dalam penerbangan.

options-title = OPSI
options-tab-game = GAME
options-tab-graphics = GRAFIK
options-tab-sound = SUARA
options-tab-controls = KONTROL
options-footer = Disimpan untuk penerbangan berikutnya; pengaturan realisme dipilih pada setiap permainan

## GAME

options-section-language = Bahasa
options-section-classic-modern = Klasik / Modern
options-cockpit = Kokpit
    .facts =
        Modern: layar beresolusi tinggi dengan tombol di bingkai
        Klasik: tata letak panel instrumen era DOS dan layar 77x60 (F-117A)
options-hud = HUD
    .facts =
        Modern: HUD hijau beresolusi tinggi ala F-16 dengan mode NAV, AA dan AG
        Klasik: HUD era DOS
options-avionics = Avionik
    .facts =
        Modern: format MFD ala F-16 pada kedua MFD (layar situasi, HARM, targeting pod, peta, persenjataan) dengan kursor yang bisa digeser
        Klasik: peta dan TAC di kiri, kamera, WAY, WPN dan ORD di kanan
options-flight-model = Model penerbangan
    .facts =
        Modern: model penerbangan aerodinamis dengan fly-by-wire: stick memerintah laju pitch dan roll; saat dilepas, hidung pesawat tetap dan lintasan terbang dipertahankan
        Klasik: model penerbangan era DOS
options-section-flying = Menerbangkan
options-auto-trim = Trim otomatis
    .facts =
        Saat stick dilepas, trim mempertahankan sikap pesawat seperti yang Anda tinggalkan
        Helikopter: hidung tidak lagi naik seiring kecepatan; jet: menyeimbangkan permukaan kendali yang rusak
        Trim manual: Alt + tombol panah, Alt + Home memusatkannya
options-tooltips = Tooltip kokpit
    .facts = Apa itu kontrol atau layar kokpit, di bawah mouse atau ujung jari VR
options-section-vr-size = Ukuran kokpit VR
options-vr-size =
    .facts =
        Ukuran kokpit (dan dunia) di headset
        100%: ukuran asli

## GRAFIK

options-section-rendering = Rendering
options-shaders = Shader
    .facts =
        Modern: bayangan arah yang halus, kabut jarak yang lembut
        Klasik: bayangan datar
options-shadows = Bayangan
    .facts =
        Modern: bayangan matahari di kokpit dan di dunia (memakai waktu GPU)
        Klasik: tanpa bayangan
options-point-lights = Lampu titik
    .facts =
        Modern: lampu sorot kokpit dan lampu layar; ledakan, kepulan asap dan lampu landasan menerangi pemandangan
        Klasik: hanya cahaya matahari dan langit
options-time-of-day = Waktu hari
    .facts =
        Modern: matahari dan bulan bergerak mengikuti jam misi, langit menggelap dan terang bersama matahari; bintang dan fase bulan di malam hari
        Modern: cahaya bulan menentukan seberapa jauh pilot musuh melihat Anda di malam hari (2 km pada malam tanpa bulan, 4 km saat bulan purnama)
        Klasik: perubahan warna yang lambat dan matahari yang tetap
options-msaa = Anti-aliasing (MSAA)
    .facts =
        Tepi halus: MSAA 4x, 2x di VR (memakai waktu GPU)
        Nonaktif: tepi piksel yang kasar
options-auto-detail = Detail pemandangan otomatis (VR)
    .facts =
        Di headset, jangkauan kota, hutan dan ladang menyusut saat frame terlambat dan pulih saat ada waktu tersisa
        Pengaturan grafik tetap seperti yang dipilih
options-sun-moon = Matahari dan bulan
    .facts =
        Matahari di langit pada siang hari, bulan dan bintang pada malam hari
        Bulan menunjukkan fase misi

## SUARA

options-section-volume = Volume
options-volume-master = Utama
    .facts = Semua suara
options-volume-engine = Mesin
    .facts = Mesin atau rotor Anda, dan pesawat yang terdengar di sekitar Anda
options-volume-effects = Efek
    .facts = Senjata, ledakan, roda pendarat, radio, dan sisanya
options-volume-warnings = Peringatan
    .facts = Klakson stall, peringatan radar dan rudal, sirene kebakaran, nada Sidewinder
options-section-cockpit = Kokpit
options-low-pass = Low-pass mesin
    .facts =
        Di dalam pesawat suara mesin terdengar teredam (kanopi dan helm meredam nada tinggi)
        Nonaktif: sama seperti di luar

## KONTROL

options-section-bindings = Pengaturan tombol
options-keys-joystick = TOMBOL DAN JOYSTICK
    .facts =
        Atur tombol peta, sumbu dan tombol joystick serta HOTAS
        Disimpan untuk penerbangan berikutnya
options-section-head-tracking = Pelacakan kepala/mata
options-head-tracking = Pelacakan Kepala/Mata
    .facts =
        Kepala Anda memutar dan menggeser pandangan di kokpit (desktop; di VR, headset yang melakukannya)
        Program pelacakan (OpenTrack...): keluaran FreeTrack, atau UDP ke 127.0.0.1 port 4242
        Home: posisi kepala saat ini menjadi titik tengah
options-tracking = Pelacakan ({ $source })
options-no-tracker = Tidak ada pelacak terdeteksi: jalankan program pelacakan
