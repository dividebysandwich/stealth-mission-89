//! Downloading a voice from the catalogue, on its own thread: each file into `<file>.part`,
//! checked against its SHA-256 and size, then renamed into place, the model last (a voice is
//! only seen once it is whole). Cancelled or failed, the `.part` files go.

use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use sha2::{Digest, Sha256};

use super::catalog::{CatalogVoice, VoiceFile};

#[derive(Debug, Clone, PartialEq)]
pub enum State {
    Running,
    Done,
    Cancelled,
    Failed(String),
}

/// A download under way (or just over), shared with its thread.
pub struct Download {
    pub voice: &'static CatalogVoice,
    pub total: u64,
    done: Arc<AtomicU64>,
    cancel: Arc<AtomicBool>,
    state: Arc<Mutex<State>>,
}

impl Download {
    /// Starts fetching `voice` into `dir`.
    pub fn start(voice: &'static CatalogVoice, dir: PathBuf) -> Self {
        let urls: Vec<String> = voice.files.iter().map(|f| voice.url(f)).collect();
        Self::start_from(voice, dir, urls)
    }

    /// From these addresses (one per file; tests serve them locally).
    pub fn start_from(voice: &'static CatalogVoice, dir: PathBuf, urls: Vec<String>) -> Self {
        let d = Self {
            voice,
            total: voice.size(),
            done: Arc::default(),
            cancel: Arc::default(),
            state: Arc::new(Mutex::new(State::Running)),
        };
        let (done, cancel, state) = (d.done.clone(), d.cancel.clone(), d.state.clone());
        std::thread::Builder::new()
            .name("voice download".into())
            .spawn(move || {
                let result = fetch(voice, &dir, &urls, &done, &cancel);
                let end = match result {
                    Ok(()) => State::Done,
                    Err(_) if cancel.load(Ordering::SeqCst) => State::Cancelled,
                    Err(e) => State::Failed(e),
                };
                if end != State::Done {
                    // Nothing of it stays (the companions renamed before the model failed).
                    for f in &voice.files {
                        let _ = std::fs::remove_file(part(&dir, f));
                        let _ = std::fs::remove_file(dir.join(f.name));
                    }
                }
                if let Ok(mut s) = state.lock() {
                    *s = end;
                }
            })
            .map_err(|e| {
                if let Ok(mut s) = d.state.lock() {
                    *s = State::Failed(e.to_string());
                }
            })
            .ok();
        d
    }

    pub fn state(&self) -> State {
        self.state.lock().map(|s| s.clone()).unwrap_or(State::Failed("poisoned".into()))
    }

    /// Bytes so far.
    pub fn done(&self) -> u64 {
        self.done.load(Ordering::Relaxed)
    }

    pub fn cancel(&self) {
        self.cancel.store(true, Ordering::SeqCst);
    }
}

fn part(dir: &Path, f: &VoiceFile) -> PathBuf {
    dir.join(format!("{}.part", f.name))
}

fn fetch(
    voice: &CatalogVoice,
    dir: &Path,
    urls: &[String],
    done: &AtomicU64,
    cancel: &AtomicBool,
) -> Result<(), String> {
    std::fs::create_dir_all(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    let agent: ureq::Agent = ureq::Agent::config_builder()
        .timeout_connect(Some(Duration::from_secs(15)))
        .timeout_recv_body(Some(Duration::from_secs(30)))
        .user_agent(concat!("stealth-mission-89/", env!("CARGO_PKG_VERSION")))
        .build()
        .into();
    // The model goes last: until it is there, the voice isn't.
    let mut order: Vec<usize> = (0..voice.files.len()).collect();
    order.sort_by_key(|&k| voice.files[k].name.ends_with(".onnx"));
    for k in order {
        let (f, url) = (&voice.files[k], &urls[k]);
        let tmp = part(dir, f);
        let response = agent.get(url).call().map_err(|e| format!("{}: {e}", f.name))?;
        let mut body = response.into_body();
        let mut reader = body.with_config().limit(f.size + 1).reader();
        let mut file =
            std::fs::File::create(&tmp).map_err(|e| format!("{}: {e}", tmp.display()))?;
        let mut hash = Sha256::new();
        let mut got = 0u64;
        let mut buf = vec![0u8; 64 * 1024];
        loop {
            if cancel.load(Ordering::SeqCst) {
                return Err("cancelled".into());
            }
            let n = reader.read(&mut buf).map_err(|e| format!("{}: {e}", f.name))?;
            if n == 0 {
                break;
            }
            file.write_all(&buf[..n]).map_err(|e| format!("{}: {e}", tmp.display()))?;
            hash.update(&buf[..n]);
            got += n as u64;
            done.fetch_add(n as u64, Ordering::Relaxed);
        }
        file.flush().map_err(|e| e.to_string())?;
        drop(file);
        let sum: String = hash.finalize().iter().map(|b| format!("{b:02x}")).collect();
        if got != f.size || sum != f.sha256 {
            return Err(format!("{}: not the expected file ({got} bytes)", f.name));
        }
        std::fs::rename(&tmp, dir.join(f.name)).map_err(|e| format!("{}: {e}", f.name))?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tts::catalog::CatalogVoice;
    use std::net::TcpListener;

    /// A voice of three tiny files.
    fn voice(bodies: &[&[u8]; 3]) -> &'static CatalogVoice {
        let hex = |b: &[u8]| -> &'static str {
            Box::leak(
                Sha256::digest(b).iter().map(|x| format!("{x:02x}")).collect::<String>().into(),
            )
        };
        Box::leak(Box::new(CatalogVoice {
            id: "test-voice-x",
            name: "Test",
            accent: "",
            kind: "",
            speakers: 1,
            licence: "",
            credit: "",
            files: [
                VoiceFile {
                    name: "test-voice-x.onnx",
                    size: bodies[0].len() as u64,
                    sha256: hex(bodies[0]),
                },
                VoiceFile {
                    name: "test-voice-x.onnx.json",
                    size: bodies[1].len() as u64,
                    sha256: hex(bodies[1]),
                },
                VoiceFile {
                    name: "test-voice-x.MODEL_CARD",
                    size: bodies[2].len() as u64,
                    sha256: hex(bodies[2]),
                },
            ],
        }))
    }

    /// Serves `bodies` (by path /0, /1, /2), each `delay` per 4 KiB written.
    fn serve(bodies: Vec<Vec<u8>>, delay: Duration) -> String {
        let l = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = l.local_addr().unwrap();
        std::thread::spawn(move || {
            for s in l.incoming() {
                let Ok(mut s) = s else { continue };
                let mut req = [0u8; 1024];
                let n = s.read(&mut req).unwrap_or(0);
                let req = String::from_utf8_lossy(&req[..n]);
                let k: usize = req.split(' ').nth(1).and_then(|p| p[1..].parse().ok()).unwrap_or(0);
                let body = &bodies[k];
                let _ = write!(
                    s,
                    "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                    body.len()
                );
                for c in body.chunks(4096) {
                    if s.write_all(c).is_err() {
                        break;
                    }
                    std::thread::sleep(delay);
                }
            }
        });
        format!("http://{addr}")
    }

    fn wait(d: &Download) -> State {
        for _ in 0..500 {
            let s = d.state();
            if s != State::Running {
                return s;
            }
            std::thread::sleep(Duration::from_millis(10));
        }
        d.state()
    }

    fn leftovers(dir: &Path) -> Vec<String> {
        std::fs::read_dir(dir)
            .map(|r| r.flatten().map(|e| e.file_name().to_string_lossy().into_owned()).collect())
            .unwrap_or_default()
    }

    fn tmpdir(name: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("sm89-voice-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        d
    }

    #[test]
    fn downloads_checks_and_cleans_up() {
        let model = vec![7u8; 100_000];
        let bodies: [&[u8]; 3] = [&model, b"{}", b"card"];
        let v = voice(&bodies);
        let base = serve(bodies.iter().map(|b| b.to_vec()).collect(), Duration::ZERO);
        let urls = |b: &str| (0..3).map(|k| format!("{b}/{k}")).collect::<Vec<_>>();

        let dir = tmpdir("ok");
        let d = Download::start_from(v, dir.clone(), urls(&base));
        assert_eq!(wait(&d), State::Done);
        assert_eq!(d.done(), d.total);
        let mut files = leftovers(&dir);
        files.sort();
        assert_eq!(
            files,
            ["test-voice-x.MODEL_CARD", "test-voice-x.onnx", "test-voice-x.onnx.json"]
        );

        // A different file at the address: refused, nothing left.
        let bad = serve(vec![vec![1u8; 100_000], b"{}".to_vec(), b"card".to_vec()], Duration::ZERO);
        let dir = tmpdir("bad");
        let d = Download::start_from(v, dir.clone(), urls(&bad));
        assert!(matches!(wait(&d), State::Failed(_)));
        assert!(leftovers(&dir).is_empty(), "{:?}", leftovers(&dir));

        // Cancelled half-way through the model.
        let slow = serve(bodies.iter().map(|b| b.to_vec()).collect(), Duration::from_millis(20));
        let dir = tmpdir("cancel");
        let d = Download::start_from(v, dir.clone(), urls(&slow));
        while d.done() < 20_000 {
            std::thread::sleep(Duration::from_millis(5));
        }
        d.cancel();
        assert_eq!(wait(&d), State::Cancelled);
        assert!(leftovers(&dir).is_empty(), "{:?}", leftovers(&dir));
        for d in ["ok", "bad", "cancel"] {
            let _ = std::fs::remove_dir_all(tmpdir(d));
        }
    }
}
