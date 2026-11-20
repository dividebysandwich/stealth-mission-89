# Editor persenjataan (src/loadout.rs). Nama senjata adalah nama aslinya, tidak diterjemahkan.

loadout-none = Tidak ada
loadout-briefed = BRIEFING
    .facts = Persenjataan yang direncanakan dalam briefing misi
loadout-clear = KOSONGKAN
    .facts = Semua hardpoint kosong
loadout-done = SELESAI
loadout-no-hardpoints = Tidak ada hardpoint
loadout-no-stores = Pesawat ini tidak membawa persenjataan
loadout-fantasy = Tidak ada pada pesawat aslinya
loadout-empty = Kosong
loadout-out-of-stock = Tidak ada stok untuk misi ini
loadout-mirror = Sama di kedua sisi
    .facts = Memuat { $station } juga
loadout-stores = Persenjataan
loadout-realistic-f117 = Persenjataan F-117A realistis
loadout-stations = Stasiun
loadout-click-drawing = Atau klik stasiun pada gambar
loadout-needs-equipment = Misi membutuhkan kamera atau peralatan khusus

## Kegunaan tiap senjata (jenis senjata 0 sampai 20)

# AIM-9M Sidewinder
weapon-role-0 = Udara-ke-udara, pemandu inframerah, jarak pendek
# AIM-120A AMRAAM
weapon-role-1 = Udara-ke-udara, pemandu radar, jarak menengah
# AGM-88A HARM
weapon-role-2 = Rudal anti-radiasi: radar dan situs SAM
# Penguin 3 ASM
weapon-role-3 = Rudal anti-kapal
# AGM-84A Harpoon
weapon-role-4 = Rudal anti-kapal jarak jauh
# AGM-65 Maverick
weapon-role-5 = Rudal berpemandu TV: kendaraan, bunker, bangunan
# GBU-12 Paveway
weapon-role-6 = Bom berpemandu laser: sasaran keras
# Mk 20 Rockeye
weapon-role-7 = Bom tandan: sasaran lunak dan area
# Durandal
weapon-role-8 = Bom penghancur landasan
# Mk 82 LDGP
weapon-role-9 = Bom jatuh bebas 500 lb
# Mk 82 Snakeye
weapon-role-10 = Bom dengan rem 500 lb untuk pelepasan rendah
# Mk 20 Rockeye II
weapon-role-11 = Bom tandan: kendaraan lapis baja dan sasaran area
# Mk 122 Fireye
weapon-role-12 = Bom pembakar
# CBU-72 FAE
weapon-role-13 = Bahan peledak bahan bakar-udara: sasaran area
# Mk 35 incendiary
weapon-role-14 = Bom tandan pembakar
# ISCB-1 mines
weapon-role-15 = Ranjau penangkal area
# IR/optical camera
weapon-role-16 = Foto pengintaian sasaran
# 1900 lb fuel tank
weapon-role-17 = Bahan bakar tambahan, dipindahkan saat dilepas
# Special cargo
weapon-role-18 = Perbekalan atau kargo untuk misi
# 20 mm cannon
weapon-role-19 = Kanon (bukan persenjataan bawaan)
# AGM-114 Hellfire
weapon-role-20 = Rudal anti-lapis baja berpemandu laser: terbang ke tempat yang dilihat targeting pod

## Cara memakai tiap senjata (jangkauan dalam km HUD, ketinggian dalam kaki)

employment-0 = Jangkauan: 1-12 km, lebih jauh dari depan
employment-1 = Jangkauan: 1-38 km, lebih jauh dari depan
employment-2 =
    Jangkauan: 9-50 km
    Peluncuran: 5.000-20.000 ft
employment-3 =
    Jangkauan: 6-14 km (pemandu klasik: 13-14 km)
    Peluncuran: 100-500 ft
employment-4 =
    Jangkauan: 6-41 km
    Peluncuran: 2.000-5.000 ft
employment-5 =
    Jangkauan: 5-10 km
    Peluncuran: 2.000-5.000 ft
employment-6 =
    Jangkauan: 1,5 km sampai 2,5 kali ketinggian, LOCK tetap (14 km dari 15.000 ft, 18 km dari 25.000 ft)
    Pelepasan: 15.000-25.000 ft
employment-8 =
    Jangkauan: 0,9-1,3 km sebelum landasan
    Pelepasan: 200-500 ft
employment-9 =
    Jangkauan: 5-11 km sebelum sasaran
    Pelepasan: 3.000-20.000 ft
employment-12 =
    Jangkauan: 0,9-2,2 km sebelum sasaran
    Pelepasan: 100-500 ft
employment-16 = Jangkauan: foto dari 2-8 km (semakin banyak lawan, semakin dekat), hidung ke arah sasaran
employment-18 = Jangkauan: lepaskan saat "Drop supplies now", dalam jarak 4 km dari zona jatuh
employment-20 =
    Jangkauan: 4-8 km
    Peluncuran: 2.000-3.000 ft
employment-guided-cluster =
    Jangkauan: 1,5 km sampai 2,5 kali ketinggian, LOCK tetap (3,7 km dari 3.000 ft, 6 km dari 5.000 ft)
    Pelepasan: 3.000-5.000 ft
employment-medium-retarded =
    Jangkauan: 1,1-1,6 km sebelum sasaran
    Pelepasan: 300-1.000 ft
employment-low-retarded =
    Jangkauan: 0,9-1,3 km sebelum sasaran
    Pelepasan: 200-500 ft

## Hardpoint

hardpoint-left-bay = Bay kiri
hardpoint-right-bay = Bay kanan
hardpoint-left-wing-bay = Bay sayap kiri
hardpoint-right-wing-bay = Bay sayap kanan
hardpoint-station-1-left-wingtip = Station 1, ujung sayap kiri
hardpoint-station-2-left-outer-wing = Station 2, sayap kiri luar
hardpoint-station-3-left-wing = Station 3, sayap kiri
hardpoint-station-4-left-inner-wing = Station 4, sayap kiri dalam
hardpoint-station-5-centreline = Station 5, garis tengah
hardpoint-station-6-right-inner-wing = Station 6, sayap kanan dalam
hardpoint-station-7-right-wing = Station 7, sayap kanan
hardpoint-station-8-right-outer-wing = Station 8, sayap kanan luar
hardpoint-station-9-right-wingtip = Station 9, ujung sayap kanan
hardpoint-station-1-left-wingtip-rail = Station 1, rel ujung sayap kiri
hardpoint-station-2-left-outer-pylon = Station 2, pylon kiri luar
hardpoint-station-3-left-inner-pylon = Station 3, pylon kiri dalam
hardpoint-station-4-right-inner-pylon = Station 4, pylon kanan dalam
hardpoint-station-5-right-outer-pylon = Station 5, pylon kanan luar
hardpoint-station-6-right-wingtip-rail = Station 6, rel ujung sayap kanan
hardpoint-tornado-left-wing-rail = Sayap kiri, rel Sidewinder
hardpoint-tornado-left-wing-pylon = Pylon sayap kiri
hardpoint-tornado-left-shoulder-forward = Bahu kiri, depan
hardpoint-tornado-left-shoulder-aft = Bahu kiri, belakang
hardpoint-tornado-right-shoulder-aft = Bahu kanan, belakang
hardpoint-tornado-right-shoulder-forward = Bahu kanan, depan
hardpoint-tornado-right-wing-pylon = Pylon sayap kanan
hardpoint-tornado-right-wing-rail = Sayap kanan, rel Sidewinder
hardpoint-a10-1-left-outer-wing = Station 1, sayap kiri luar
hardpoint-a10-2-left-wing = Station 2, sayap kiri
hardpoint-a10-3-left-wing = Station 3, sayap kiri
hardpoint-a10-4-left-centre-wing = Station 4, sayap kiri tengah
hardpoint-a10-5-left-fuselage = Station 5, badan pesawat kiri
hardpoint-a10-6-centreline = Station 6, garis tengah
hardpoint-a10-7-right-fuselage = Station 7, badan pesawat kanan
hardpoint-a10-8-right-centre-wing = Station 8, sayap kanan tengah
hardpoint-a10-9-right-wing = Station 9, sayap kanan
hardpoint-a10-10-right-wing = Station 10, sayap kanan
hardpoint-a10-11-right-outer-wing = Station 11, sayap kanan luar
hardpoint-av8b-1-left-rail = Station 1, rel Sidewinder kiri
hardpoint-av8b-3-left-middle-wing = Station 3, sayap kiri, tengah
hardpoint-av8b-4-left-inboard-wing = Station 4, sayap kiri, dalam
hardpoint-av8b-6-right-inboard-wing = Station 6, sayap kanan, dalam
hardpoint-av8b-7-right-middle-wing = Station 7, sayap kanan, tengah
hardpoint-av8b-9-right-rail = Station 9, rel Sidewinder kanan
hardpoint-f22-left-side-bay = Bay samping kiri (Sidewinder)
hardpoint-f22-left-main-bay = Bay utama, bagian kiri
hardpoint-f22-right-main-bay = Bay utama, bagian kanan
hardpoint-f22-right-side-bay = Bay samping kanan (Sidewinder)
hardpoint-fa18e-8-right-inner-wing = Station 8, sayap kanan dalam
hardpoint-fa18e-10-right-outer-wing = Station 10, sayap kanan luar
hardpoint-fa18e-11-right-wingtip = Station 11, ujung sayap kanan
hardpoint-c5-hold = Ruang kargo (palet jatuhan udara)
hardpoint-ch47-cabin = Kabin (pasukan payung)
weapon-role-21 = Udara-ke-udara, pengarah inframerah, jangkauan sangat pendek
employment-21 = Jangkauan: 1-6 km, lebih jauh dari depan
loadout-none-in-stock = (stok habis)
loadout-empties = Mengosongkan stasiun { $stations } saat dimuat
loadout-empties-two = Mengosongkan stasiun { $first } dan { $second } saat dimuat
hardpoint-oh58-left-pylon = Pilon senjata kiri
hardpoint-oh58-right-pylon = Pilon senjata kanan
hardpoint-mh60r-left-outer = Stasiun luar kiri (peluncur Hellfire)
hardpoint-mh60r-left-inner = Stasiun dalam kiri
hardpoint-mh60r-right-inner = Stasiun dalam kanan
weapon-role-22 = Pod senapan mesin: ditembakkan seperti meriam, searah hidung
employment-22 = Jangkauan: hingga 1,5 km, 500 peluru per pod
weapon-role-23 = Senjata pemicu sensor: empat puluh skeet menembak ke bawah ke kendaraan
employment-23 =
    Dipandu ke titik yang ditandai pod penargetan saat dilepas
    Pelepasan: di atas ketinggian ledak di halaman SMS (500-3.000 ft)
weapon-role-24 = Torpedo ringan: turun dengan parasut, mencari di bawah air dan mengarah ke kapal selam
employment-24 =
    Diprogram di halaman WPN operator sensor: arah, jeda aktivasi, pola pencarian
    Pelepasan: di atas air, di bawah 300 ft dan 100 kt
weapon-role-25 = Rudal serang darat jarak jauh: terbang ke sasarannya, lalu pilot mengarahkannya lewat gambar pencarinya
employment-25 =
    Diprogram di halaman SLAM: ketinggian jelajah, jarak pencari aktif, sasaran utama atau sekunder
    Jangkauan: hingga 250 km, empat di udara sekaligus
weapon-role-26 = Bom luncur: meluncur ke titik yang ditentukan; jangkauannya bertambah dengan ketinggian dan kecepatan pelepasan
employment-26 =
    Diarahkan ke titik pod atau kuncian darat, jika tidak ke sasaran utama
    Jangkauan: sekitar 20 km rendah, hingga 130 km dari ketinggian; pertahanan udara bisa menembaknya jatuh
weapon-role-27 = Umpan yang diluncurkan dari udara: terbang dengan jejak pesawat ke pertahanan udara dan menarik rudalnya
employment-27 =
    Terbang ke titik yang ditentukan, lalu lurus hingga 10 menit
    Tidak berbahaya bagi sasaran
weapon-role-28 = Pasukan payung: satu regu prajurit yang diterjunkan di atas tempat yang mereka rebut
employment-28 =
    Dari pintu C-5 (dua regu) atau rampa CH-47 (satu regu) di atas zona penerjunan
    Terjun antara 500 dan 1500 ft di atas tanah, di bawah 150 kt; sebagian besar regu harus mendarat di zona
weapon-role-29 = Pod pengacau taktis: memangkas radar yang dikacau hingga jarak tembus-bakarnya
employment-29 =
    Diatur di halaman JAM: satu radar atau sektor per pod, nyala atau mati
    Mengacau membuatmu terlihat dari lebih jauh
weapon-role-30 = Dispenser serang lapangan udara: menebar bom kecil pembuat kawah dan ranjau antitank berparasut di jalur 500 m
employment-30 =
    Tetap di pylon dan menebar selama dua detik; terbang datar dan rendah di atas sasaran
    Dari ketinggian paling tinggi 1000 ft di atas tanah; tiga kawah menutup landasan, ranjau menunggu kendaraan
weapon-role-31 = Roket 70 mm tanpa pemandu dalam pod: berpasangan atau beruntun melawan kendaraan dan pasukan di tempat terbuka
employment-31 =
    Bidik dengan retikel roket (CCIP): menunjukkan tempat roket yang ditembakkan sekarang jatuh
    Tekan untuk sepasang, tahan untuk tembakan beruntun; terbaik dari menukik landai dalam 4 km
weapon-role-32 = Roket 70 mm berpemandu laser: masing-masing mengarah ke titik laser pod penjejak sasaran
employment-32 =
    Arahkan pod penjejak ke sasaran dan tembak searah hidung; roket berbelok ke titik itu
    Hingga 6 km; tahan pod pada sasaran sampai mengenai
loadout-wingman = WINGMAN
    .facts = Wingman AI dengan pesawat dan persenjataan yang sama bergabung di udara; menu radio (\) memberinya perintah
