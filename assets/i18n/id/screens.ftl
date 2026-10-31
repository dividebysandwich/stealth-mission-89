# Kneeboard (font garis), perencana misi, dan misi tersimpan. Singkatan navigasi
# (PT, BRG, ETA, PRI, SEC, KM, KT) tetap dalam bahasa Inggris.

kb-brief = BRIEF
kb-route = RUTE
kb-map = PETA
kb-target = TARGET
kb-keys = TOMBOL
kb-tip = Peta, catatan misi, dan panduan tombol
kb-geran-file = GERAN-2 (ARSIP)
kb-objectives = SASARAN
kb-primary = UTAMA: { $objective }
kb-secondary = SEKUNDER: { $objective }
kb-times = WAKTU
kb-deadline = BATAS { $time }
kb-now = SEKARANG { $time }
kb-conditions = KONDISI
kb-moon = BULAN: { $phase } ({ $lit }% TERANG)
kb-armament = PERSENJATAAN
kb-roe = ATURAN KETERLIBATAN
kb-total = TOTAL { $km } KM PADA { $kt } KT
kb-fuel = BBM { $lbs } LBS
kb-preparing = MEMPERSIAPKAN...
kb-airborne = DI UDARA
kb-developing = MENCETAK...
kb-no-photo = TANPA FOTO
kb-airborne-target = TARGET UDARA
kb-scroll-vr = GULIR: THUMBSTICK, JARI DI PAPAN
kb-scroll-desktop = GULIR: RODA MOUSE DI ATAS PAPAN

## Perencana misi

planner-low = RENDAH 500 FT
planner-medium = SEDANG 5000 FT
planner-high = TINGGI 20000 FT
planner-title = PERENCANA MISI
planner-route = Rute
planner-base = PANGKALAN
planner-total = TOTAL { $km } KM  { $time }
planner-briefed = Rute briefing { $km } km, BBM { $fuel } lbs
planner-coverage = Cakupan radar untuk
    .note = Cakupan radar mengasumsikan terbang datar 480 kt, pintu ruang senjata tertutup.
planner-detected = { $km } km rute terdeteksi
planner-pulse = Jangkauan deteksi radar pulse (P)
planner-doppler = Jangkauan deteksi radar Doppler (D)
planner-missile = Jangkauan rudal
planner-uav = UAV musuh
planner-edit-hint = Seret titik untuk memindahkannya, seret peta untuk menggeser. Klik ruas untuk menambah titik, klik kanan untuk menghapusnya. Roda: zoom.
planner-reset = RESET
planner-view-hint = Seret peta untuk menggeser. Roda: zoom.
planner-layers = LAPISAN
planner-layer-sams = Radar / posisi SAM
planner-radar-types = Jenis radar
planner-layer-pulse = Jangkauan deteksi radar pulse
planner-layer-doppler = Jangkauan deteksi radar Doppler
planner-layer-airfield = Lapangan udara

## Misi tersimpan

saves-title = MISI TERSIMPAN
saves-subtitle = Disimpan saat terbang dari menu: SIMPAN MISI
saves-none = Tidak ada misi tersimpan
saves-line = { $theater }  -  { $aircraft }  -  waktu misi { $time }
saves-saved-at = Disimpan { $date }
saves-unreadable = Simpanan tidak terbaca (versi lain)
saves-delete =
    .facts = Hapus simpanan ini
saves-back =
    .facts = Kembali ke layar misi tunggal
