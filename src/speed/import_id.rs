use crate::import_id;
use std::hint::black_box;
use std::time::{Duration, Instant};

fn bench<F: FnMut()>(mut f: F, rounds: usize) -> Duration {
    let start = Instant::now();
    for _ in 0..rounds {
        f();
    }
    start.elapsed()
}

#[test]
fn speed_import_id_parse_beats_baseline_decode_always() {
    let ids = (0..800)
        .map(|i| format!("/src/Module{i}.elm?with=./Other.elm&with=./Third.elm"))
        .collect::<Vec<_>>();
    assert!(ids.iter().all(|id| import_id::parse(id).is_ok()));
    let fast = bench(
        || {
            for id in &ids {
                let _ = black_box(import_id::parse(id));
            }
        },
        120,
    );
    let slow = bench(
        || {
            for id in &ids {
                let _ = black_box(baseline_parse(id));
            }
        },
        120,
    );
    eprintln!("import_id fast={fast:?} slow={slow:?}");
    assert!(fast < slow, "import_id fast={fast:?} slow={slow:?}");
}

fn baseline_parse(id: &str) -> crate::import_id::ParsedImportId {
    let without_file = id.strip_prefix("file://").unwrap_or(id);
    let (raw_path, query) = without_file
        .split_once('?')
        .map_or((without_file, ""), |(path, query)| (path, query));
    let path = percent_decode(raw_path);
    let params = query.split('&').filter(|part| !part.is_empty());
    let mut has_raw = false;
    let mut with = Vec::new();
    for param in params {
        let (key, value) = param.split_once('=').unwrap_or((param, ""));
        if key == "raw" {
            has_raw = true;
        } else if key == "with" {
            with.push(percent_decode(value));
        }
    }
    let valid = path.ends_with(".elm") && !has_raw;
    crate::import_id::ParsedImportId { path, with, valid }
}

fn percent_decode(input: &str) -> String {
    let bytes = input.as_bytes();
    let mut out = String::with_capacity(input.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            if let Ok(hex) = u8::from_str_radix(&input[i + 1..i + 3], 16) {
                out.push(hex as char);
                i += 3;
                continue;
            }
        }
        out.push(bytes[i] as char);
        i += 1;
    }
    out
}
