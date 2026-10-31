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
options-player-labels = Nama pemain
    .facts = Multiplayer: nama pilot lain di atas pesawat mereka, sejauh apa pun
options-label-size =
    .facts =
        Ukuran nama pemain
        50%: bawaan
options-section-vr-size = Ukuran kokpit VR
options-vr-size =
    .facts =
        Ukuran kokpit (dan dunia) di headset
        100%: ukuran asli
options-section-gif = Rekaman GIF
options-gif-size = UKURAN
options-gif-rate = BINGKAI
options-gif-scale-100 =
    .facts = Resolusi penuh jendela: berkas besar
options-gif-scale-25 =
    .facts = Berkas kecil
options-gif-rate-50 =
    .facts = Hanya jika permainan berjalan 100 fps atau lebih (setiap bingkai kedua direkam)
options-gif-hint = F12 (kecuali diubah) memulai dan menghentikan; disimpan di folder Gambar Anda

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
options-volume-rwr = RWR
    .facts = Nada peringatan radar, sebagai bagian dari volume peringatan
options-section-rwr-tones = Nada peringatan radar
rwr-tones-all = SEMUA
    .facts = Radar yang menyorot Anda, setiap sapuan yang mendeteksi Anda, peluncuran rudal
rwr-tones-detect = HANYA DETEKSI
    .facts =
        Satu nada saat radar mendeteksi Anda (lagi setelah radar melepas Anda sebentar)
        Dan peluncuran rudal
rwr-tones-launch = HANYA PELUNCURAN
    .facts = Hanya rudal yang diluncurkan ke Anda
rwr-tones-off = MATI
    .facts = Tanpa nada peringatan radar (lampu dan layar tetap memperingatkan)
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

## EFFECTS

options-section-effects = Effects
options-section-retro = Retro
effect-shake = Camera shake
    .facts =
        The view shakes with the gun, nearby blasts, stall buffet, touchdowns and the runway
        VR: the cockpit vibrates, your head stays still
effect-flashes = Combat flashes
    .facts =
        Flak bursts, explosions and missile launches light the clouds, the ground and the cockpit for a moment
effect-barrage = Night barrage
    .facts =
        Conventional war at night: the towns' guns fire tracer blindly into the sky, searchlights sweep over them
        Only for the look: the barrage hits nothing, the searchlights find nothing
effect-cockpit-night = Cockpit night lighting
    .facts =
        A dim red flood light in the cockpit after dark (blue-green under the goggles)
        The master caution pulses amber over the panel while a warning is on
effect-nvg = Night vision goggles
    .facts =
        Bright lights bloom into halos, the picture grains more in the dark
        Desktop: the goggles' round field of view
effect-snow = Snow
    .facts =
        Above the freezing level storms snow instead of rain: low in a northern winter, high in summer
effect-mist = Ground mist
    .facts =
        Modern shaders: mist lies over the low ground and the sea at dawn, dusk and night
        Thin in the deserts, none in storms
effect-cloud-shadows = Cloud shadows
    .facts =
        Modern shaders: the clouds' shadows on the ground below
effect-vapour = Uap dan aliran rotor
    .facts =
        Saat manuver tajam, jejak uap dari ujung sayap
        Helikopter yang melayang rendah menerbangkan debu, atau percikan di atas air
        Kendaraan yang bergerak menerbangkan debu di belakangnya
effect-night-sky = Night sky
    .facts =
        Shooting stars now and then, the stars low over the horizon twinkle
        The northern lights over the North Cape
effect-vga = VGA colours
    .facts =
        The 1989 look with modern shading: 64 steps a colour channel, as a VGA card's, dithered
effect-scanlines = Scanlines
    .facts =
        A CRT monitor's scanlines (not in VR)

## LANGUAGE NAMES

lang-name-en = Inggris
lang-name-bg = Bulgaria
lang-name-zh-Hans = Tionghoa (Sederhana)
lang-name-hr = Kroasia
lang-name-cs = Ceko
lang-name-da = Denmark
lang-name-nl = Belanda
lang-name-et = Estonia
lang-name-fi = Finlandia
lang-name-fr = Prancis
lang-name-de = Jerman
lang-name-el = Yunani
lang-name-hu = Hungaria
lang-name-id = Indonesia
lang-name-ga = Irlandia
lang-name-it = Italia
lang-name-ja = Jepang
lang-name-ko = Korea
lang-name-lv = Latvia
lang-name-lt = Lituania
lang-name-mt = Malta
lang-name-pl = Polandia
lang-name-pt-BR = Portugis (Brasil)
lang-name-ro = Rumania
lang-name-ru = Rusia
lang-name-sk = Slovakia
lang-name-sl = Slovenia
lang-name-es = Spanyol
lang-name-sv = Swedia
lang-name-uk = Ukraina

## Browser build: offer to play in the browser's language (src/i18n.rs)

language-offer-question = Bahasa peramban: { $language }. Mainkan game dalam bahasa ini?
language-offer-yes = Ya
language-offer-no = Tidak
language-offer-never = Jangan tanya lagi
effect-sunbeams = Sunbeams
    .facts =
        Shafts of sunlight through the gaps of a broken overcast
