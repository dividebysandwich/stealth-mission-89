//! The voices the game offers to download: freely licensed Piper voices from
//! `huggingface.co/rhasspy/piper-voices`, pinned to one revision, each file with its size and
//! SHA-256 (a changed or truncated file is refused). Only voices whose MODEL_CARD names a
//! licence that lets the game hand them out (public domain, CC0, CC BY, CC BY-SA); not the
//! non-commercial ones (hfc_female, ryan) nor those whose card only points elsewhere (alan,
//! amy).

/// The piper-voices revision the files come from.
pub const REVISION: &str = "b4e358ffcb232591bf73a540e70f7de528e4174a";

#[derive(Debug)]
pub struct CatalogVoice {
    /// The files' stem, e.g. `en_GB-vctk-medium`.
    pub id: &'static str,
    pub name: &'static str,
    pub accent: &'static str,
    /// What it sounds like ("male", "female", "many": a multi-speaker voice).
    pub kind: &'static str,
    pub speakers: u32,
    pub licence: &'static str,
    /// Where the recordings came from (the credit a CC BY licence asks for).
    pub credit: &'static str,
    /// `.onnx`, `.onnx.json` and `MODEL_CARD`.
    pub files: [VoiceFile; 3],
}

#[derive(Debug)]
pub struct VoiceFile {
    /// The file's name in the voices folder.
    pub name: &'static str,
    pub size: u64,
    pub sha256: &'static str,
}

impl CatalogVoice {
    /// The download address of one of its files.
    pub fn url(&self, file: &VoiceFile) -> String {
        // en_GB-vctk-medium: en/en_GB/vctk/medium/
        let mut parts = self.id.splitn(3, '-');
        let (lang, name, quality) =
            (parts.next().unwrap_or(""), parts.next().unwrap_or(""), parts.next().unwrap_or(""));
        let family = lang.split('_').next().unwrap_or(lang);
        // The card is plain `MODEL_CARD` there; here it carries the voice's name.
        let remote = if file.name.ends_with(".MODEL_CARD") { "MODEL_CARD" } else { file.name };
        format!(
            "https://huggingface.co/rhasspy/piper-voices/resolve/{REVISION}/{family}/{lang}/{name}/{quality}/{remote}"
        )
    }

    /// Bytes of all its files.
    pub fn size(&self) -> u64 {
        self.files.iter().map(|f| f.size).sum()
    }
}

pub fn find(id: &str) -> Option<&'static CatalogVoice> {
    CATALOG.iter().find(|v| v.id == id)
}

macro_rules! voice {
    ($id:literal, $name:literal, $accent:literal, $kind:literal, $speakers:literal,
     $licence:literal, $credit:literal,
     onnx: $os:literal $oh:literal, json: $js:literal $jh:literal, card: $cs:literal $ch:literal) => {
        CatalogVoice {
            id: $id,
            name: $name,
            accent: $accent,
            kind: $kind,
            speakers: $speakers,
            licence: $licence,
            credit: $credit,
            files: [
                VoiceFile { name: concat!($id, ".onnx"), size: $os, sha256: $oh },
                VoiceFile { name: concat!($id, ".onnx.json"), size: $js, sha256: $jh },
                VoiceFile { name: concat!($id, ".MODEL_CARD"), size: $cs, sha256: $ch },
            ],
        }
    };
}

pub const CATALOG: &[CatalogVoice] = &[
    voice!("en_GB-vctk-medium", "VCTK", "British, many regions", "many", 109, "CC BY 4.0",
        "CSTR VCTK Corpus, University of Edinburgh (datashare.ed.ac.uk/handle/10283/3443)",
        onnx: 76952753 "4e9fc85ab9009385319fc6bae7f55577f8a2d7ee77fd9159a5500eb6531f41e6",
        json: 6637 "7f85e6391ed0f7f46e4abd19345929a16be931a0c9945086f96692dce2087fa8",
        card: 326 "53ec6b1fd90d9125d5bebac55261efd27f7e8d6dc962f605a5b5f796df2ec7fb"),
    voice!("en_GB-alba-medium", "Alba", "Scottish", "female", 1, "CC BY 4.0",
        "Alba corpus, University of Edinburgh (datashare.ed.ac.uk/handle/10283/3270)",
        onnx: 63201294 "401369c4a81d09fdd86c32c5c864440811dbdcc66466cde2d64f7133a66ad03b",
        json: 4888 "aa965a2f02ecced632c2694e1fc72bbff6d65f265fab567ca945918c73dd89f4",
        card: 324 "fa166b1779404c470b0b6b4ba0238bc4a35bf89d2cd130c6788f697188b737d6"),
    voice!("en_GB-northern_english_male-medium", "Northern English", "British, northern", "male",
        1, "CC BY-SA 4.0", "OpenSLR 83, Google crowdsourced UK and Ireland English (openslr.org/83)",
        onnx: 63201294 "57a219ae8e638873db7d18893304be5069c42868f392bb95c3ff17f0690d0689",
        json: 4847 "69557ed3d974463453e9b0c09dd99a7ed0e52b8b87b64b357dbeeb2540a97d47",
        card: 305 "75661c3a50c0ba82b32b3a438e54103d8dfe5bbe8fbb864ac687a040e79948bb"),
    voice!("en_GB-cori-medium", "Cori", "British", "female", 1, "Public domain",
        "LibriVox recordings (librivox.org)",
        onnx: 63531379 "1899f98e5fb8310154f3c2973f4b8a929ba7245e722b3d3a85680b833d95f10d",
        json: 4966 "e262c16d7f192f69d4edd6b4ef8a5915379e67495fcc402f1ab15eeb33da3d36",
        card: 474 "38d76dfde845837b184668f8bab1c6426ad740b1af0fcebc349037822739e536"),
    voice!("en_US-libritts_r-medium", "LibriTTS-R", "American, many", "many", 904,
        "CC BY 4.0", "LibriTTS-R corpus (openslr.org/141)",
        onnx: 78580914 "10bb85e071d616fcf4071f369f1799d0491492ab3c5d552ec19fb548fac13195",
        json: 20123 "b471dc60d2d8335e819c393d196d6fbf792817f40051257b269878505bc9afb3",
        card: 279 "0ccde6927e5bb4d743f4ea39618a9387ba18cca3351220a8a9cfdbc68b30fcb9"),
    voice!("en_US-joe-medium", "Joe", "American", "male", 1, "CC0",
        "OHF-Voice voice datasets (github.com/OHF-Voice/voice-datasets)",
        onnx: 63201294 "58afce0321b8d9c46d7cdf9c16500cc55a793b4220212dba6b70fb788b3baf06",
        json: 4794 "3d6d5410b3795cb1950595247ef8f06190719e6fdbfa3a2356d8ec368e1aad33",
        card: 281 "d2caa63aca0fccb155105e959e393e5c0c0f03a1f388ef5ba217b83ef860c760"),
    voice!("en_US-norman-medium", "Norman", "American", "male", 1, "Public domain",
        "LibriVox recordings (librivox.org)",
        onnx: 63531379 "b9739443232a80a59c7d18810dd856899bf16a7964725f5ab81ea49b1351cb71",
        json: 4968 "6c2db7f558a4a8deb9fe822583c1c5105f6c4e834dd0f9de8ad17a888ee9fe1d",
        card: 528 "60eaf4d10009d1bd4a1aa555d337dae00f60421d72fee415b9139a021e7cb5d0"),
    voice!("en_US-john-medium", "John", "American", "male", 1, "Public domain",
        "LibriVox recordings (librivox.org)",
        onnx: 63531379 "789c6c875726e627ddee93d51d8727859abe9c091c3d141591f4b83c2072e988",
        json: 4965 "af60f177b6b550f3d7a302720c0fb89e7f94a82b5dca464775ef63b1c69ba09a",
        card: 498 "a056da65cd70dcbda96fc3420fe8d3442dbf3c8b75f49cdf800a715b42e299d1"),
    voice!("en_US-bryce-medium", "Bryce", "American", "male", 1, "Public domain",
        "Recordings released into the public domain (see its MODEL_CARD)",
        onnx: 63531379 "dc9caa6c313199ffb5ac698b6e542fa6cba388aeaf2731e25262e33b9810aef1",
        json: 4966 "7ceb1bc4af6d4e41b6d1edbb86c67e91e01eaa71f66db4cd0ae92ac704d415be",
        card: 405 "96042c4ddbd405f6e40e2938b069987f4016a35832114bbefed1ed04242ee532"),
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_voice_is_complete_and_pinned() {
        for v in CATALOG {
            assert_eq!(CATALOG.iter().filter(|o| o.id == v.id).count(), 1, "{}", v.id);
            for f in &v.files {
                assert!(f.name.starts_with(v.id), "{}", f.name);
                assert_eq!(f.sha256.len(), 64, "{}", f.name);
                assert!(f.sha256.chars().all(|c| c.is_ascii_hexdigit()), "{}", f.name);
                assert!(f.size > 0, "{}", f.name);
                assert!(v.url(f).contains(REVISION));
            }
            assert!(v.speakers >= 1);
        }
        let v = find("en_GB-vctk-medium").unwrap();
        assert_eq!(
            v.url(&v.files[0]),
            format!(
                "https://huggingface.co/rhasspy/piper-voices/resolve/{REVISION}/en/en_GB/vctk/medium/en_GB-vctk-medium.onnx"
            )
        );
    }
}
