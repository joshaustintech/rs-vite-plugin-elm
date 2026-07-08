use crate::{Error, Result};
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::thread;

pub fn imports_from_source(source: &str) -> Vec<String> {
    let mut module_seen = false;
    let mut reading = false;
    let mut imports = Vec::new();
    let mut in_comment = false;

    for line in source.lines() {
        if !module_seen
            && (line.starts_with("module ")
                || line.starts_with("port module ")
                || line.starts_with("effect module "))
        {
            module_seen = true;
            continue;
        }

        if in_comment {
            if line.trim_end().ends_with("-}") {
                in_comment = false;
            }
            continue;
        }

        if module_seen && line.starts_with("import ") {
            reading = true;
        }

        if reading {
            if let Some(rest) = line.strip_prefix("import ") {
                if let Some(name) = rest.split_whitespace().next() {
                    imports.push(name.to_string());
                }
            } else if line.starts_with(' ') || line.trim().is_empty() || line.starts_with("--") {
            } else if line.starts_with("{-") {
                in_comment = true;
            } else {
                break;
            }
        }
    }

    imports
}

pub fn dependencies(entry: &Path) -> Result<Vec<PathBuf>> {
    let base = base_dir(entry)?;
    let source_dirs = source_dirs_for(&base)?;
    let mut known = HashSet::new();
    let mut out = HashSet::new();
    collect(entry, &source_dirs, &mut known, &mut out)?;
    let mut deps: Vec<_> = out.into_iter().collect();
    deps.sort();
    Ok(deps)
}

fn collect(
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
    let imports = imports_from_source(&source);
    let handles: Vec<_> = imports
        .into_iter()
        .map(|name| {
            let dirs = source_dirs.to_vec();
            thread::spawn(move || resolve_module(&name, &dirs))
        })
        .collect();

    for handle in handles {
        if let Ok(Some(dep)) = handle.join() {
            if out.insert(dep.clone()) && dep.extension().and_then(|e| e.to_str()) == Some("elm") {
                collect(&dep, source_dirs, known, out)?;
            }
        }
    }

    Ok(())
}

fn resolve_module(name: &str, source_dirs: &[PathBuf]) -> Option<PathBuf> {
    let rel = format!("{}.elm", name.replace('.', "/"));
    source_dirs
        .iter()
        .map(|dir| dir.join(&rel))
        .find(|path| path.exists())
}

fn base_dir(file: &Path) -> Result<PathBuf> {
    let source = fs::read_to_string(file)
        .map_err(|e| Error::new(format!("Failed to read Elm source {}: {e}", file.display())))?;
    let first = source.lines().next().unwrap_or("");
    let module = first
        .strip_prefix("module ")
        .or_else(|| first.strip_prefix("port module "))
        .or_else(|| first.strip_prefix("effect module "))
        .and_then(|rest| rest.split_whitespace().next());

    let mut dir = file.parent().unwrap_or_else(|| Path::new(".")).to_path_buf();
    if let Some(module) = module {
        for _ in module.split('.').skip(1) {
            dir.pop();
        }
    }
    Ok(dir)
}

pub fn find_elm_json(start: &Path) -> Option<PathBuf> {
    let mut dir = if start.is_file() {
        start.parent()?.to_path_buf()
    } else {
        start.to_path_buf()
    };
    loop {
        let candidate = dir.join("elm.json");
        if candidate.exists() {
            return Some(candidate);
        }
        if !dir.pop() {
            return None;
        }
    }
}

pub fn source_dirs_for(base: &Path) -> Result<Vec<PathBuf>> {
    let elm_json = find_elm_json(base).ok_or_else(|| Error::new("Could not find elm.json"))?;
    let text = fs::read_to_string(&elm_json)
        .map_err(|e| Error::new(format!("Failed to read {}: {e}", elm_json.display())))?;
    let dirs = source_dirs_from_elm_json(&text)
        .into_iter()
        .map(|dir| elm_json.parent().unwrap_or_else(|| Path::new(".")).join(dir))
        .collect();
    Ok(dirs)
}

pub fn source_dirs_from_elm_json(text: &str) -> Vec<String> {
    let Some(key) = text.find("\"source-directories\"") else {
        return Vec::new();
    };
    let Some(open) = text[key..].find('[').map(|i| key + i) else {
        return Vec::new();
    };
    let Some(close) = text[open..].find(']').map(|i| open + i) else {
        return Vec::new();
    };
    let mut dirs = Vec::new();
    let mut chars = text[open + 1..close].chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '"' {
            let mut value = String::new();
            while let Some(next) = chars.next() {
                if next == '"' {
                    break;
                }
                value.push(next);
            }
            dirs.push(value);
        }
    }
    dirs
}
