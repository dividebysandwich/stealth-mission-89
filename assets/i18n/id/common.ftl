# Stealth Mission 89: kata-kata yang dipakai di semua layar (lihat en/common.ftl dan glossary.md).
#
# Bahasa Inggris adalah sumber semua terjemahan. Pesan yang tidak ada dalam suatu bahasa
# ditampilkan dalam bahasa Inggris. Tetap dalam bahasa Inggris di mana pun: tulisan kokpit
# (HUD, label MFD dan panel), akronim (FLIR, ILS, CCIP, SAM, RWR...), nama pesawat, senjata
# dan tempat (fichiers_md/translation.md, dan glossary.md di folder tiap bahasa).
#
# Sintaks Fluent: baris dalam teks multibaris tidak boleh diawali "[", "*" atau "."; kurung
# kurawal ditulis {"{"} dan {"}"}.

language = Bahasa
    .facts =
        Bahasa menu, tooltip, briefing, dan pesan
        Tulisan kokpit (HUD, MFD, panel) tetap dalam bahasa Inggris, seperti di pesawat aslinya

## Tombol

back = KEMBALI
    .facts = Kembali ke layar sebelumnya
ok = OK
cancel = BATAL
close = TUTUP
continue = LANJUT
yes = YA
no = TIDAK
on = AKTIF
off = NONAKTIF
classic = KLASIK
modern = MODERN

## Uji sistem terjemahan

i18n-test-count = { $n ->
   *[other] { $n } tembakan kena
}
i18n-test-lines =
    Baris pertama
    Baris kedua
i18n-test-nested = Pilih: { $what }
