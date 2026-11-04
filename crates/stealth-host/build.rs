//! The browser build has no file system, and the Android app's assets are in its APK, where only
//! Bevy's asset server reads them: the data files read at run time (`assets::read`) are compiled
//! in, and the names of all the others (served beside the page, in the APK) are listed for
//! `assets::exists` and `assets::list` (`cfg(embedded_assets)`).

use std::fmt::Write;
use std::path::{Path, PathBuf};

/// The directories read with `assets::read`; the rest (models, sounds, shaders) are loaded
/// through Bevy's asset server.
const EMBEDDED: [&str; 6] = ["sim", "mission", "theaters", "palettes", "images", "fonts"];

fn main() {
    println!("cargo::rustc-check-cfg=cfg(embedded_assets)");
    if std::env::var("CARGO_CFG_TARGET_ARCH").as_deref() != Ok("wasm32")
        && std::env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("android")
    {
        return;
    }
    println!("cargo::rustc-cfg=embedded_assets");
    let manifest = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    let root = manifest.join("../../assets").canonicalize().expect("assets directory");
    println!("cargo:rerun-if-changed={}", root.display());
    let mut files = Vec::new();
    walk(&root, &root, &mut files);
    files.sort();
    let mut out = String::from("pub static EMBEDDED: &[(&str, &[u8])] = &[\n");
    for (rel, path) in &files {
        if EMBEDDED.iter().any(|d| rel.starts_with(&format!("{d}/"))) {
            println!("cargo:rerun-if-changed={}", path.display());
            writeln!(out, "    ({rel:?}, include_bytes!({:?})),", path.display().to_string())
                .unwrap();
        }
    }
    out.push_str("];\npub static FILES: &[&str] = &[\n");
    for (rel, _) in &files {
        writeln!(out, "    {rel:?},").unwrap();
    }
    out.push_str("];\n");
    let dest = PathBuf::from(std::env::var("OUT_DIR").unwrap()).join("embedded_assets.rs");
    std::fs::write(dest, out).unwrap();
}

fn walk(root: &Path, dir: &Path, files: &mut Vec<(String, PathBuf)>) {
    for e in std::fs::read_dir(dir).unwrap().flatten() {
        let path = e.path();
        if path.is_dir() {
            println!("cargo:rerun-if-changed={}", path.display());
            walk(root, &path, files);
        } else {
            let rel = path.strip_prefix(root).unwrap().to_string_lossy().replace('\\', "/");
            files.push((rel, path));
        }
    }
}
