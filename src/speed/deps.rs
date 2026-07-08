use crate::{deps, Error, Result};
use std::collections::HashSet;
use std::fs;
use std::hint::black_box;
use std::path::{Path, PathBuf};
use std::thread;
use std::time::{Duration, Instant};

fn bench<F: FnMut()>(mut f: F, rounds: usize) -> Duration {
    let start = Instant::now();
    for _ in 0..rounds {
        f();
    }
    start.elapsed()
}

#[test]
fn speed_dependency_scan_beats_threaded_baseline() -> Result<()> {
    let root = temp_dir("deps");
    fs::create_dir_all(root.join("src")).map_err(|e| Error::new(e.to_string()))?;
    fs::write(root.join("elm.json"), r#"{"type":"application","source-directories":["src"]}"#)
        .map_err(|e| Error::new(e.to_string()))?;
    let imports = (0..24).map(|i| format!("import Mod{i}\n")).collect::<String>();
    fs::write(root.join("src/Main.elm"), format!("module Main exposing (main)\n{imports}\nmain = 0\n"))
        .map_err(|e| Error::new(e.to_string()))?;
    for i in 0..24 {
        fs::write(root.join(format!("src/Mod{i}.elm")), format!("module Mod{i} exposing (x)\nx = {i}\n"))
            .map_err(|e| Error::new(e.to_string()))?;
    }
    let entry = root.join("src/Main.elm");
    assert!(deps::dependencies(&entry).is_ok());
    assert!(threaded_dependencies(&entry).is_ok());
    let fast = bench(|| {
        let _ = black_box(deps::dependencies(&entry));
    }, 40);
    let slow = bench(|| {
        let _ = black_box(threaded_dependencies(&entry));
    }, 40);
    eprintln!("deps fast={fast:?} slow={slow:?}");
    assert!(fast < slow, "deps fast={fast:?} slow={slow:?}");
    let _ = fs::remove_dir_all(root);
    Ok(())
}

fn threaded_dependencies(entry: &Path) -> Result<Vec<PathBuf>> {
    let base = deps::find_elm_json(entry)
        .and_then(|p| p.parent().map(Path::to_path_buf))
        .ok_or_else(|| Error::new("Could not find elm.json"))?;
    let source_dirs = deps::source_dirs_for(&base)?;
    let mut known = HashSet::new();
    let mut out = HashSet::new();
    threaded_collect(entry, &source_dirs, &mut known, &mut out)?;
    let mut deps: Vec<_> = out.into_iter().collect();
    deps.sort();
    Ok(deps)
}

fn threaded_collect(
    file: &Path,
    source_dirs: &[PathBuf],
    known: &mut HashSet<PathBuf>,
    out: &mut HashSet<PathBuf>,
) -> Result<()> {
    let file = file.to_path_buf();
    if !known.insert(file.clone()) {
        return Ok(());
    }
    let source = fs::read_to_string(&file)
        .map_err(|e| Error::new(format!("Failed to read Elm source {}: {e}", file.display())))?;
    let imports = deps::imports::imports_from_source(&source);
    let handles: Vec<_> = imports
        .into_iter()
        .map(|name| {
            let dirs = source_dirs.to_vec();
            thread::spawn(move || {
                let rel = format!("{}.elm", name.replace('.', "/"));
                dirs.iter().map(|dir| dir.join(&rel)).find(|path| path.exists())
            })
        })
        .collect();
    for handle in handles {
        if let Ok(Some(dep)) = handle.join() {
            if out.insert(dep.clone()) && dep.extension().and_then(|e| e.to_str()) == Some("elm") {
                threaded_collect(&dep, source_dirs, known, out)?;
            }
        }
    }
    Ok(())
}

fn temp_dir(tag: &str) -> PathBuf {
    let nanos = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_nanos()).unwrap_or(0);
    std::env::temp_dir().join(format!("rs-vite-plugin-elm-{tag}-{nanos}"))
}
