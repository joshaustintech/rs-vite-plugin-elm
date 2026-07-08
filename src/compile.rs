use crate::{assets, deps, elm_make, esm, hmr, options, Error, Result};
use std::collections::HashMap;
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
    let dependencies = dependencies_for_targets(&request.targets, request.cwd.as_deref())?;
    let cwd = request
        .cwd
        .clone()
        .or_else(|| request.options.cwd.clone())
        .or_else(|| request.targets.first().and_then(|first| deps::find_elm_json(first)).and_then(|p| p.parent().map(Path::to_path_buf)))
        .ok_or_else(|| Error::new("Could not find elm.json"))?;

    let target_strings = request
        .targets
        .iter()
        .map(|path| path.to_string_lossy().into_owned())
        .collect::<Vec<_>>();
    let compiled = elm_make::compile_to_string(&target_strings, &cwd, &request.options)?;
    let esm = esm::to_es_module(&compiled)?;
    let current_dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let code = postprocess(&esm, &dependencies, request.options.is_build, &current_dir)?;

    Ok(CompileOutput { code, dependencies })
}

pub fn dependencies_for_targets(targets: &[PathBuf], cwd: Option<&Path>) -> Result<Vec<PathBuf>> {
    let first = targets.first().ok_or_else(|| Error::new("No Elm targets supplied"))?;
    let source_dirs = match cwd {
        Some(dir) => deps::source_dirs_for(dir)?,
        None => deps::source_dirs_for(first)?,
    };
    let mut dependencies = Vec::new();
    let mut module_cache = HashMap::new();
    for target in targets {
        dependencies.extend(deps::dependencies_with_source_dirs_cached(
            target,
            &source_dirs,
            &mut module_cache,
        )?);
    }
    dependencies.sort();
    dependencies.dedup();
    Ok(dependencies)
}

pub fn postprocess(code: &str, dependencies: &[PathBuf], is_build: bool, current_dir: &Path) -> Result<String> {
    let with_assets = assets::inject(code)?;
    if is_build {
        Ok(with_assets)
    } else {
        let trimmed = with_assets.replace(
            "console.warn('Compiled in DEBUG mode",
            "// console.warn('Compiled in DEBUG mode",
        );
        let deps = dependencies
            .iter()
            .map(|path| vite_project_path(path, current_dir))
            .collect::<Vec<_>>();
        hmr::inject(&trimmed, &deps)
    }
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
    path.to_string_lossy().replace('\\', "/")
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
