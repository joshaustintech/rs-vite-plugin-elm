use crate::{deps, Error, Result};
use std::collections::HashMap;
use std::fs;
use std::hint::black_box;
use std::path::PathBuf;
use std::time::{Duration, Instant};

fn bench<F: FnMut()>(mut f: F, rounds: usize) -> Duration {
    let start = Instant::now();
    for _ in 0..rounds {
        f();
    }
    start.elapsed()
}

#[test]
fn speed_shared_source_dirs_beats_repeated_lookup() -> Result<()> {
    let root = temp_dir("multi");
    fs::create_dir_all(root.join("src")).map_err(|e| Error::new(e.to_string()))?;
    fs::write(root.join("elm.json"), r#"{"type":"application","source-directories":["src"]}"#)
        .map_err(|e| Error::new(e.to_string()))?;
    let imports = (0..24).map(|i| format!("import Mod{i}\n")).collect::<String>();
    let mut targets = Vec::new();
    for name in ["Main", "Other", "Third", "Fourth", "Fifth", "Sixth", "Seventh", "Eighth"] {
        let path = root.join(format!("src/{name}.elm"));
        fs::write(&path, format!("module {name} exposing (main)\n{imports}\nmain = 0\n"))
            .map_err(|e| Error::new(e.to_string()))?;
        targets.push(path);
    }
    for i in 0..24 {
        fs::write(root.join(format!("src/Mod{i}.elm")), format!("module Mod{i} exposing (x)\nx = {i}\n"))
            .map_err(|e| Error::new(e.to_string()))?;
    }
    let source_dirs = deps::source_dirs_for(&root)?;
    let fast = bench(
        || {
            let mut module_cache = HashMap::new();
            for target in &targets {
                let _ = black_box(deps::dependencies_with_source_dirs_cached(
                    target,
                    &source_dirs,
                    &mut module_cache,
                ));
            }
        },
        60,
    );
    let slow = bench(
        || {
            for target in &targets {
                let _ = black_box(deps::dependencies(target));
            }
        },
        60,
    );
    eprintln!("multi_target fast={fast:?} slow={slow:?}");
    assert!(fast < slow, "multi_target fast={fast:?} slow={slow:?}");
    let _ = fs::remove_dir_all(root);
    Ok(())
}

fn temp_dir(tag: &str) -> PathBuf {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    std::env::temp_dir().join(format!("rs-vite-plugin-elm-{tag}-{nanos}"))
}
