use std::fs;
use std::path::{Path, PathBuf};
use std::process;

const MAX_LINES: usize = 150;

fn main() {
    let src = Path::new("src");
    let mut files = Vec::new();
    if let Err(message) = collect_rs_files(src, &mut files) {
        eprintln!("{message}");
        process::exit(1);
    }

    for file in files {
        let text = match fs::read_to_string(&file) {
            Ok(text) => text,
            Err(e) => {
                eprintln!("failed to read {}: {e}", file.display());
                process::exit(1);
            }
        };
        let lines = text.lines().count();
        if lines > MAX_LINES {
            eprintln!("{} has {lines} lines; limit is {MAX_LINES}", file.display());
            process::exit(1);
        }
    }
}

fn collect_rs_files(dir: &Path, out: &mut Vec<PathBuf>) -> Result<(), String> {
    let entries = fs::read_dir(dir).map_err(|e| format!("failed to read {}: {e}", dir.display()))?;
    for entry in entries {
        let entry = entry.map_err(|e| format!("failed to read entry in {}: {e}", dir.display()))?;
        let path = entry.path();
        if path.is_dir() {
            collect_rs_files(&path, out)?;
        } else if path.extension().and_then(|ext| ext.to_str()) == Some("rs") {
            out.push(path);
        }
    }
    Ok(())
}
