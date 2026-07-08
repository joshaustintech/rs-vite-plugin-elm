use crate::{assets, deps, elm_make, esm, hmr, options, Error, Result};
use std::path::{Path, PathBuf};

pub struct CompileRequest {
    pub targets: Vec<PathBuf>,
    pub options: options::Options,
    pub cwd: Option<PathBuf>,
}

pub struct CompileOutput {
    pub code: String,
    pub dependencies: Vec<PathBuf>,
}

pub fn compile(request: CompileRequest) -> Result<CompileOutput> {
    let first = request
        .targets
        .first()
        .ok_or_else(|| Error::new("No Elm targets supplied"))?;
    let cwd = request
        .cwd
        .clone()
        .or_else(|| deps::find_elm_json(first).and_then(|p| p.parent().map(Path::to_path_buf)))
        .ok_or_else(|| Error::new("Could not find elm.json"))?;

    let mut dependencies = Vec::new();
    let source_dirs = deps::source_dirs_for(first)?;
    for target in &request.targets {
        dependencies.extend(deps::dependencies_with_source_dirs(target, &source_dirs)?);
    }
    dependencies.sort();
    dependencies.dedup();

    let target_strings = request
        .targets
        .iter()
        .map(|path| path.to_string_lossy().into_owned())
        .collect::<Vec<_>>();
    let compiled = elm_make::compile_to_string(&target_strings, &cwd, &request.options)?;
    let esm = esm::to_es_module(&compiled)?;
    let with_assets = assets::inject(&esm)?;
    let code = if request.options.is_build {
        with_assets
    } else {
        let trimmed = with_assets.replace(
            "console.warn('Compiled in DEBUG mode",
            "// console.warn('Compiled in DEBUG mode",
        );
        let current_dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        let deps = dependencies
            .iter()
            .map(|path| vite_project_path(path, &current_dir))
            .collect::<Vec<_>>();
        hmr::inject(&trimmed, &deps)?
    };

    Ok(CompileOutput { code, dependencies })
}

pub(crate) fn vite_project_path(path: &Path, current_dir: &Path) -> String {
    let relative = path
        .strip_prefix(current_dir)
        .ok()
        .map(path_to_posix)
        .unwrap_or_else(|| lexical_relative(current_dir, path).unwrap_or_else(|| path.to_string_lossy().into_owned()));
    format!("/{}", relative)
}

fn path_to_posix(path: &Path) -> String {
    let mut parts = Vec::new();
    for component in path.components() {
        parts.push(component.as_os_str().to_string_lossy().into_owned());
    }
    if parts.is_empty() {
        ".".into()
    } else {
        parts.join("/")
    }
}

pub(crate) fn lexical_relative(from: &Path, to: &Path) -> Option<String> {
    let from = from.components().collect::<Vec<_>>();
    let to = to.components().collect::<Vec<_>>();
    if from.first() != to.first() {
        return None;
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
        Some(".".into())
    } else {
        Some(parts.join("/"))
    }
}
