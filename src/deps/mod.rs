pub mod elm_json;
pub mod imports;

use crate::{Error, Result};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

pub fn dependencies(entry: &Path) -> Result<Vec<PathBuf>> {
    let base = base_dir(entry)?;
    let source_dirs = source_dirs_for(&base)?;
    dependencies_with_source_dirs(entry, &source_dirs)
}

pub fn dependencies_with_source_dirs(entry: &Path, source_dirs: &[PathBuf]) -> Result<Vec<PathBuf>> {
    let mut module_cache = HashMap::new();
    dependencies_with_source_dirs_cached(entry, source_dirs, &mut module_cache)
}

pub fn dependencies_with_source_dirs_cached(
    entry: &Path,
    source_dirs: &[PathBuf],
    module_cache: &mut HashMap<String, Option<PathBuf>>,
) -> Result<Vec<PathBuf>> {
    let mut known = HashSet::new();
    let mut out = HashSet::new();
    collect(entry, source_dirs, module_cache, &mut known, &mut out)?;
    let mut deps: Vec<_> = out.into_iter().collect();
    deps.sort();
    Ok(deps)
}

fn collect(
    file: &Path,
    source_dirs: &[PathBuf],
    module_cache: &mut HashMap<String, Option<PathBuf>>,
    known: &mut HashSet<PathBuf>,
    out: &mut HashSet<PathBuf>,
) -> Result<()> {
    let file = file.to_path_buf();
    if !known.insert(file.clone()) {
        return Ok(());
    }

    let source = fs::read_to_string(&file)
        .map_err(|e| Error::new(format!("Failed to read Elm source {}: {e}", file.display())))?;
    for name in imports::imports_from_source(&source) {
        if let Some(dep) = resolve_module(&name, source_dirs, module_cache) {
            if out.insert(dep.clone()) && dep.extension().and_then(|e| e.to_str()) == Some("elm") {
                collect(&dep, source_dirs, module_cache, known, out)?;
            }
        }
    }

    Ok(())
}

fn resolve_module(
    name: &str,
    source_dirs: &[PathBuf],
    module_cache: &mut HashMap<String, Option<PathBuf>>,
) -> Option<PathBuf> {
    if let Some(path) = module_cache.get(name) {
        return path.clone();
    }
    let rel = format!("{}.elm", name.replace('.', "/"));
    let resolved = source_dirs
        .iter()
        .map(|dir| dir.join(&rel))
        .find(|path| path.exists());
    module_cache.insert(name.to_string(), resolved.clone());
    resolved
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
    let dirs = elm_json::source_dirs_from_elm_json(&text)
        .into_iter()
        .map(|dir| elm_json.parent().unwrap_or_else(|| Path::new(".")).join(dir))
        .collect();
    Ok(dirs)
}
