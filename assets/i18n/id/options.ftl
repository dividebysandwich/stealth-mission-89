# Jendela OPSI (src/options.rs) dan halamannya di menu dalam penerbangan.

options-title = OPSI
options-tab-game = GIM
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

options-section-effects = Efek
options-section-retro = Retro
effect-shake = Guncangan kamera
    .facts =
        Tampilan bergetar karena meriam, ledakan dekat, stall, pendaratan, dan landasan
        VR: kokpit bergetar, kepala Anda tetap diam
effect-flashes = Kilatan pertempuran
    .facts = Ledakan flak, ledakan, dan peluncuran rudal menerangi awan, tanah, dan kokpit sesaat
effect-barrage = Tembakan penghalang malam
    .facts =
        Perang konvensional di malam hari: meriam kota menembakkan peluru pelacak secara membabi buta ke langit, lampu sorot menyapu di atasnya
        Drone musuh di atas kota tertangkap lampu sorotnya dan menarik tembakan meriamnya
        Hanya untuk tampilan: tembakan penghalang tidak mengenai apa pun
effect-cockpit-night = Pencahayaan malam kokpit
    .facts =
        Lampu merah redup di kokpit setelah gelap (biru kehijauan di bawah kacamata)
        Master caution berdenyut kuning amber di atas panel selama peringatan aktif
effect-nvg = Kacamata penglihatan malam
    .facts =
        Cahaya terang melebar menjadi lingkaran cahaya, gambar lebih berbintik dalam gelap
        Desktop: bidang pandang bulat kacamata
effect-snow = Salju
    .facts = Di atas ketinggian beku badai membawa salju, bukan hujan: rendah di musim dingin utara, tinggi di musim panas
effect-mist = Kabut tanah
    .facts =
        Shader modern: kabut menyelimuti dataran rendah dan laut saat fajar, senja, dan malam
        Tipis di gurun, tidak ada saat badai
effect-cloud-shadows = Bayangan awan
    .facts = Shader modern: bayangan awan di tanah di bawahnya
effect-vapour = Uap dan aliran rotor
    .facts =
        Saat manuver tajam, jejak uap dari ujung sayap
        Helikopter yang melayang rendah menerbangkan debu, atau percikan di atas air
        Kendaraan yang bergerak menerbangkan debu di belakangnya
effect-night-sky = Langit malam
    .facts =
        Sesekali bintang jatuh, bintang rendah di cakrawala berkelip
        Aurora di atas Tanjung Utara
effect-vga = Warna VGA
    .facts = Tampilan 1989 dengan pembayangan modern: 64 tingkat per kanal warna seperti kartu VGA, dengan dithering
effect-scanlines = Garis pindai
    .facts = Garis pindai monitor CRT (tidak di VR)

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
effect-sunbeams = Sinar matahari
    .facts = Berkas sinar matahari melalui celah awan yang terbelah
options-msaa-forced = Mati selama piksel-isasi (retro) aktif
options-volume-radio = Radio
    .facts = Pemain lain yang berbicara di radio koop, dan ATC yang diucapkan
options-volume-atc = Suara ATC
    .facts = Panggilan ATC yang diucapkan, sebagai bagian dari volume radio
options-volume-music = Musik
    .facts = Musik menu utama
options-section-atc = ATC lisan
options-section-radio = Radio koop
options-push-to-talk = Tekan untuk bicara
    .facts =
        Tahan tombol bicara (Caps Lock, atau tombol joystick atau VR yang ditetapkan ke PTT) dan semua pemain mendengar Anda
        Mikrofon terbuka saat pertama kali Anda menekan
options-mic-gain = GAIN MIKROFON
    .facts =
        Seberapa keras mikrofon Anda keluar
        100%: apa adanya
options-mic-default = Bawaan
options-voice-text-only = hanya teks
options-voices-line = Pengendali: { $controllers }   Pilot: { $pilot }
options-voices = SUARA
    .facts =
        Unduh suara untuk panggilan radio, hapus
        Pilih suara pengendali dan suara Anda sendiri, lalu dengarkan
options-voices-web = Hanya teks di peramban
options-tracking-mouse = Melihat dengan mouse saat tracking
    .facts =
        Nyala: tombol kanan mouse tetap memutar dan memperbesar tampilan kokpit
        Mati: selama tracker mengirim, tombol kanan mouse tidak melakukan apa pun di kokpit
options-atc = Pengendalian lalu lintas udara
    .facts =
        Panggilan radio menara, pendekatan, dan GCA di lapangan terbang kawan
        Izin lepas landas, overhead break, dan talkdown saat awan rendah atau jarak pandang buruk
options-pixel-block = Ukuran blok piksel-isasi
    .facts = Sisi sebuah blok, dalam piksel layar (2: halus, 5: kasar)
effect-pixelize = Piksel-isasi
    .facts =
        Seluruh tampilan 3D (medan, pesawat, bangunan, kokpit) dalam piksel persegi besar, seperti pada PC MS-DOS; menu dan tooltip tetap tajam (tidak di VR)
        Penggeser mengatur ukuran sebuah blok, dalam piksel layar (2: halus, 5: kasar)
        Anti-aliasing mati selama aktif
