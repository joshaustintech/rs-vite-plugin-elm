use crate::compile;
use std::hint::black_box;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

fn bench<F: FnMut()>(mut f: F, rounds: usize) -> Duration {
    let start = Instant::now();
    for _ in 0..rounds {
        f();
    }
    start.elapsed()
}

#[test]
fn speed_vite_project_path_beats_manual_relative_baseline() {
    let current = Path::new("/Users/josh/vite_plugin_elm_work/rs-vite-plugin-elm");
    let deps = (0..200)
        .map(|i| PathBuf::from(format!("/Users/josh/vite_plugin_elm_work/rs-vite-plugin-elm/src/Mod{i}.elm")))
        .collect::<Vec<_>>();
    let expected = deps
        .iter()
        .map(|dep| compile::vite_project_path(dep, current))
        .collect::<Vec<_>>();
    let fast = bench(|| {
        for dep in &deps {
            black_box(compile::vite_project_path(dep, current));
        }
    }, 300);
    let slow = bench(|| {
        for dep in &deps {
            black_box(baseline_vite_project_path(dep, current));
        }
    }, 300);
    assert_eq!(
        expected,
        deps.iter().map(|dep| baseline_vite_project_path(dep, current)).collect::<Vec<_>>()
    );
    eprintln!("paths fast={fast:?} slow={slow:?}");
    assert!(fast < slow, "path fast={fast:?} slow={slow:?}");
}

fn baseline_vite_project_path(path: &Path, current_dir: &Path) -> String {
    let from = current_dir.components().collect::<Vec<_>>();
    let to = path.components().collect::<Vec<_>>();
    if from.first() != to.first() {
        return path.to_string_lossy().into_owned();
    }
    let mut common = 0;
    while common < from.len() && common < to.len() && from[common] == to[common] {
        common += 1;
    }
    let mut parts = Vec::new();
    for _ in common..from.len() {
        parts.push("..".to_string());
    }
    for component in &to[common..] {
        parts.push(component.as_os_str().to_string_lossy().into_owned());
    }
    if parts.is_empty() {
        "/.".to_string()
    } else {
        format!("/{}", parts.join("/"))
    }
}
