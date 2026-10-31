//! Embeds the translations: every `assets/i18n/<language>/*.ftl` goes into the library as
//! `(language, file, text)`, so that no file is read at run time (the browser build too) and a
//! new file needs no code change.

use std::fmt::Write;
use std::path::Path;

fn main() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/i18n");
    println!("cargo::rerun-if-changed={}", dir.display());
    let mut files = Vec::new();
    for lang in std::fs::read_dir(&dir).expect("assets/i18n") {
        let lang = lang.unwrap().path();
        if !lang.is_dir() {
            continue;
        }
        println!("cargo::rerun-if-changed={}", lang.display());
        for f in std::fs::read_dir(&lang).unwrap() {
            let f = f.unwrap().path();
            if f.extension().is_some_and(|e| e == "ftl") {
                println!("cargo::rerun-if-changed={}", f.display());
                files.push(f);
            }
        }
    }
    files.sort();
    let mut out = String::from("&[\n");
    for f in &files {
        let lang = f.parent().unwrap().file_name().unwrap().to_str().unwrap();
        let name = f.file_name().unwrap().to_str().unwrap();
        let path = f.to_str().unwrap();
        writeln!(out, "    ({lang:?}, {name:?}, include_str!({path:?})),").unwrap();
    }
    out += "]\n";
    let out_dir = std::env::var("OUT_DIR").unwrap();
    std::fs::write(Path::new(&out_dir).join("ftl.rs"), out).unwrap();
}
